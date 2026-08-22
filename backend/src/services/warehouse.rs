use calamine::{open_workbook_from_rs, Data, Reader, Xls, Xlsx};
use sqlx::PgPool;
use std::io::Cursor;
use uuid::Uuid;

use std::collections::HashMap;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, CreateWarehouseRequest, PaginationParams, ShelfNode, StorageLocation,
        StorageLocationInventoryItem, StorageLocationWithInventory, UpdateWarehouseRequest,
        Warehouse, WarehouseImportErrorDetail, WarehouseImportResult, WarehouseImportRow,
        WarehouseQuery, WarehouseReportData, WarehouseReportSummary, WarehouseTreeNode,
    },
    repositories::warehouse::{
        list_stocked_locations_tx, lock_warehouse_by_location_tx, StockedLocation,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity, RequestContext},
        AuditService,
    },
    AppError, Result,
};

/// 倉庫底下尚有結存時的阻擋訊息。最多列 5 個儲位，避免單筆錯誤訊息過長。
fn stocked_locations_error(display: &str, stocked: &[StockedLocation]) -> AppError {
    const MAX_LISTED: usize = 5;
    let listed = stocked
        .iter()
        .take(MAX_LISTED)
        .map(|l| {
            let label = l.name.as_deref().unwrap_or(&l.code);
            format!("{}（{}）", label, l.on_hand_qty.normalize())
        })
        .collect::<Vec<_>>()
        .join("、");
    let more = if stocked.len() > MAX_LISTED {
        format!("等共 {} 個儲位", stocked.len())
    } else {
        String::new()
    };
    AppError::BusinessRule(format!(
        "倉庫「{display}」底下仍有庫存，無法停用：{listed}{more}。請先將庫存移轉或出庫後再停用。"
    ))
}

pub struct WarehouseService;

impl WarehouseService {
    /// 於 transaction 內取得倉庫代碼生成鎖。
    ///
    /// ⚠️ 鎖必須持有到交易 commit（`pg_advisory_xact_lock` 即為此語意）：READ COMMITTED
    /// 下若在 INSERT 尚未 commit 前放鎖，後到的交易讀不到那筆資料、會算出同一個代碼，
    /// 鎖等於失效。故取號**必須與 INSERT 在同一個交易內**——這也是本函數只收 `tx`
    /// 不收 `pool` 的原因。
    async fn acquire_code_lock(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<()> {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(crate::constants::WAREHOUSE_CODE_LOCK_KEY)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    /// 自動生成倉庫代碼（流水號格式：WH001, WH002, ...）。
    ///
    /// 必須在交易內呼叫並持鎖至 commit，否則並發會撞 `warehouses_code_key`。
    async fn generate_code_tx(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<String> {
        Self::acquire_code_lock(tx).await?;

        let prefix = "WH";

        // 取得所有以 WH 開頭的代碼
        let codes: Vec<String> =
            sqlx::query_scalar("SELECT code FROM warehouses WHERE code LIKE $1 ORDER BY code DESC")
                .bind(format!("{}%", prefix))
                .fetch_all(&mut **tx)
                .await?;

        // 解析序號並找出最大值
        let max_seq = codes
            .iter()
            .filter_map(|code| {
                if code.starts_with(prefix) && code.len() > prefix.len() {
                    let num_str = &code[prefix.len()..];
                    num_str.parse::<i32>().ok()
                } else {
                    None
                }
            })
            .max();

        let seq = max_seq.map(|s| s + 1).unwrap_or(1);
        Ok(format!("{}{:03}", prefix, seq))
    }

    /// 解析倉庫代碼序號（例如 WH001 → 1），供單元測試驗證
    #[cfg(test)]
    pub fn parse_code_sequence(code: &str, prefix: &str) -> Option<i32> {
        if !code.starts_with(prefix) || code.len() <= prefix.len() {
            return None;
        }
        let num_str = &code[prefix.len()..];
        num_str.parse::<i32>().ok()
    }

    /// 建立倉庫 — Service-driven audit
    ///
    /// Actor：允許 User 或 System（保持一致於 Partner，未來可能有系統自動建立路徑）。
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateWarehouseRequest,
        ctx: Option<RequestContext<'_>>,
    ) -> Result<Warehouse> {
        let mut tx = pool.begin().await?;

        // 自動產碼必須在交易內（並持鎖至 commit），否則 generate → INSERT 之間有窗口，
        // 並發會算出同一個 code 而撞 `warehouses_code_key`。
        let code = match req
            .code
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            Some(provided_code) => provided_code.to_string(),
            None => Self::generate_code_tx(&mut tx).await?,
        };

        let req_with_code = CreateWarehouseRequest {
            code: Some(code),
            name: req.name.clone(),
            address: req.address.clone(),
        };

        let result = Self::create_tx(&mut tx, actor, &req_with_code, ctx).await?;
        tx.commit().await?;
        Ok(result)
    }

    /// Transaction 版本：建立倉庫（用於跨服務原子操作）
    pub(super) async fn create_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        req: &CreateWarehouseRequest,
        ctx: Option<RequestContext<'_>>,
    ) -> Result<Warehouse> {
        // 如果 code 為空或未提供，則自動生成（需從 pool 取，改為簡化：要求 code 在跨服務 tx 內必須提供）
        let code = match req
            .code
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            Some(provided_code) => provided_code.to_string(),
            None => {
                return Err(AppError::Validation(
                    "Warehouse code required in transaction context".to_string(),
                ))
            }
        };

        // tx 內檢查 code 唯一
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM warehouses WHERE code = $1)")
                .bind(&code)
                .fetch_one(&mut **tx)
                .await?;

        if exists {
            return Err(AppError::Conflict(
                "Warehouse code already exists".to_string(),
            ));
        }

        let warehouse = sqlx::query_as::<_, Warehouse>(
            r#"
            INSERT INTO warehouses (id, code, name, address, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, true, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&code)
        .bind(&req.name)
        .bind(&req.address)
        .fetch_one(&mut **tx)
        .await?;

        let display = format!("{} ({})", warehouse.name, warehouse.code);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "WAREHOUSE_CREATE",
                entity: Some(AuditEntity::new("warehouse", warehouse.id, &display)),
                data_diff: Some(DataDiff::create_only(&warehouse)),
                request_context: ctx,
            },
        )
        .await?;

        Ok(warehouse)
    }

    /// Transaction 版本：更新倉庫（用於跨服務原子操作）
    pub(super) async fn update_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateWarehouseRequest,
        ctx: Option<RequestContext<'_>>,
    ) -> Result<Warehouse> {
        let before =
            sqlx::query_as::<_, Warehouse>("SELECT * FROM warehouses WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Warehouse not found".to_string()))?;

        // 把「啟用狀態」關掉等同於刪除倉庫（同一條 is_active 路徑），
        // 因此與 delete_tx 受同一道庫存閘門把關。
        if before.is_active && req.is_active == Some(false) {
            Self::ensure_no_stock_tx(tx, id, &format!("{} ({})", before.name, before.code)).await?;
        }

        let after = sqlx::query_as::<_, Warehouse>(
            r#"
            UPDATE warehouses SET
                name = COALESCE($1, name),
                address = COALESCE($2, address),
                is_active = COALESCE($3, is_active),
                updated_at = NOW()
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&req.address)
        .bind(req.is_active)
        .bind(id)
        .fetch_one(&mut **tx)
        .await?;

        let display = format!("{} ({})", after.name, after.code);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "WAREHOUSE_UPDATE",
                entity: Some(AuditEntity::new("warehouse", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: ctx,
            },
        )
        .await?;

        Ok(after)
    }

    /// 停用倉庫前的庫存閘門：底下任一儲位仍有結存就拒絕（422）。
    ///
    /// 倉庫停用後，庫存查詢樹（`list_with_shelves`）與現況報表（`get_report_data`）
    /// 都只看 `is_active = true` 的倉庫，底下的結存會從所有畫面消失，但
    /// `stock_ledger` 帳上仍在——即「隱形庫存」。2026-08-05「儲藏室 (2)」被停用時
    /// 底下尚有 4,424 單位結存，即為此情形。
    async fn ensure_no_stock_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
        display: &str,
    ) -> Result<()> {
        let stocked = list_stocked_locations_tx(tx, id).await?;
        if stocked.is_empty() {
            return Ok(());
        }
        Err(stocked_locations_error(display, &stocked))
    }

    /// Transaction 版本：刪除倉庫（用於跨服務原子操作）
    pub(super) async fn delete_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        id: Uuid,
        ctx: Option<RequestContext<'_>>,
    ) -> Result<()> {
        let before =
            sqlx::query_as::<_, Warehouse>("SELECT * FROM warehouses WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Warehouse not found".to_string()))?;

        let display = format!("{} ({})", before.name, before.code);
        Self::ensure_no_stock_tx(tx, id, &display).await?;

        let after = sqlx::query_as::<_, Warehouse>(
            "UPDATE warehouses SET is_active = false, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_one(&mut **tx)
        .await?;

        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "WAREHOUSE_DELETE",
                entity: Some(AuditEntity::new("warehouse", before.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: ctx,
            },
        )
        .await?;

        Ok(())
    }

    /// 寫入儲位庫存前的倉庫閘門：倉庫已停用則拒絕（422）。
    ///
    /// 對稱於 `ensure_no_stock_tx`（停用前檢查底下已有庫存）——這裡檢查的是庫存寫入
    /// 前，其所屬倉庫是否仍啟用（#33：既有 5 條寫入路徑沒有任何一處檢查
    /// `warehouse.is_active`，讀取端全部過濾停用倉庫、寫入端完全不看，兩者相加即為
    /// 隱形庫存）。只守「增加」路徑：出庫（decrement）不受此閘門限制，讓已停用倉庫
    /// 底下的殘留庫存仍可被領用/轉出清空。
    pub(crate) async fn ensure_active_for_location_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        storage_location_id: Uuid,
    ) -> Result<()> {
        match lock_warehouse_by_location_tx(tx, storage_location_id).await? {
            Some(w) if !w.is_active => Err(AppError::BusinessRule(format!(
                "倉庫「{} ({})」已停用，無法增加庫存。請先重新啟用倉庫再進行此操作。",
                w.name, w.code
            ))),
            Some(_) => Ok(()),
            // 現況此分支在 document_lines / storage_location_inventory 兩條呼叫路徑下皆不可達
            // （storage_locations 的 FK 皆無 CASCADE，被任一列參照時 DELETE 會被 Postgres 擋下），
            // 但明確回 NotFound 優於任其撞進後續 INSERT/UPDATE 的 FK 違反錯誤——
            // 後者訊息對呼叫端不友善，且萬一日後新增呼叫路徑繞過既有存在性檢查，這裡才是唯一防線
            // （Qodo code review，PR #85）。
            None => Err(AppError::NotFound("Storage location not found".to_string())),
        }
    }

    /// 取得倉庫列表
    pub async fn list(pool: &PgPool, query: &WarehouseQuery) -> Result<Vec<Warehouse>> {
        let pagination = PaginationParams {
            page: query.page,
            per_page: query.per_page,
        };
        let suffix = pagination.sql_suffix();

        let mut qb = sqlx::QueryBuilder::new("SELECT * FROM warehouses WHERE 1=1");

        if let Some(ref kw) = query.keyword {
            let pattern = format!("%{}%", kw);
            qb.push(" AND (code ILIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR name ILIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }

        if let Some(is_active) = query.is_active {
            qb.push(" AND is_active = ");
            qb.push_bind(is_active);
        } else {
            // P1-R4-14: 預設僅讀取「啟用中」的倉庫
            qb.push(" AND is_active = true");
        }

        qb.push(" ORDER BY code");
        qb.push(&suffix);

        let warehouses = qb.build_query_as::<Warehouse>().fetch_all(pool).await?;

        Ok(warehouses)
    }

    /// 取得倉庫樹（含所有可儲物的儲位，排除建築結構 wall/door/window）
    pub async fn list_with_shelves(pool: &PgPool) -> Result<Vec<WarehouseTreeNode>> {
        let warehouses: Vec<Warehouse> =
            sqlx::query_as("SELECT * FROM warehouses WHERE is_active = true ORDER BY code")
                .fetch_all(pool)
                .await?;

        // 無啟用倉庫時直接回空，省一次 ANY([]) 的 DB round-trip。
        if warehouses.is_empty() {
            return Ok(Vec::new());
        }

        // 一次批次取所有倉庫的儲位（WHERE warehouse_id = ANY），避免逐倉庫查詢的 N+1。
        // ORDER BY 維持與原逐筆查詢相同：各倉庫內依 row_index, col_index, code。
        let warehouse_ids: Vec<Uuid> = warehouses.iter().map(|wh| wh.id).collect();
        let shelves: Vec<(Uuid, Uuid, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT warehouse_id, id, code, name FROM storage_locations
            WHERE warehouse_id = ANY($1) AND is_active = true
              AND location_type NOT IN ('wall', 'door', 'window')
            ORDER BY row_index, col_index, code
            "#,
        )
        .bind(&warehouse_ids)
        .fetch_all(pool)
        .await?;

        // 依 warehouse_id 分組；按查詢結果順序 push，保留各倉庫內的排序。
        let mut shelves_by_warehouse: HashMap<Uuid, Vec<ShelfNode>> =
            HashMap::with_capacity(warehouses.len());
        for (warehouse_id, id, code, name) in shelves {
            shelves_by_warehouse
                .entry(warehouse_id)
                .or_default()
                .push(ShelfNode { id, code, name });
        }

        // 倉庫順序維持外層查詢的 ORDER BY code。
        let result = warehouses
            .into_iter()
            .map(|wh| WarehouseTreeNode {
                shelves: shelves_by_warehouse.remove(&wh.id).unwrap_or_default(),
                id: wh.id,
                code: wh.code,
                name: wh.name,
            })
            .collect();
        Ok(result)
    }

    /// 取得單一倉庫
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Warehouse> {
        let warehouse = sqlx::query_as::<_, Warehouse>("SELECT * FROM warehouses WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Warehouse not found".to_string()))?;

        Ok(warehouse)
    }

    /// 更新倉庫 — Service-driven audit
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateWarehouseRequest,
        ctx: Option<RequestContext<'_>>,
    ) -> Result<Warehouse> {
        let _user = actor.require_user()?;
        let mut tx = pool.begin().await?;
        let result = Self::update_tx(&mut tx, actor, id, req, ctx).await?;
        tx.commit().await?;
        Ok(result)
    }

    /// 刪除倉庫（軟刪除）— Service-driven audit
    pub async fn delete(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        ctx: Option<RequestContext<'_>>,
    ) -> Result<()> {
        let _user = actor.require_user()?;
        let mut tx = pool.begin().await?;
        Self::delete_tx(&mut tx, actor, id, ctx).await?;
        tx.commit().await?;
        Ok(())
    }

    /// 取得倉庫現況報表資料（含所有儲位及庫存）
    pub async fn get_report_data(pool: &PgPool, warehouse_id: Uuid) -> Result<WarehouseReportData> {
        let warehouse = Self::get_by_id(pool, warehouse_id).await?;

        let storage_locations = sqlx::query_as::<_, StorageLocation>(
            r#"
            SELECT * FROM storage_locations
            WHERE warehouse_id = $1 AND is_active = true
            ORDER BY row_index, col_index, code
            "#,
        )
        .bind(warehouse_id)
        .fetch_all(pool)
        .await?;

        // 批次取得所有儲位的庫存（避免 N+1）
        let all_inventory = sqlx::query_as::<_, StorageLocationInventoryItem>(
            r#"
            SELECT
                sli.id,
                sli.storage_location_id,
                sli.product_id,
                p.sku AS product_sku,
                p.name AS product_name,
                p.spec AS product_spec,
                sli.on_hand_qty,
                p.base_uom,
                sli.batch_no,
                sli.expiry_date,
                sli.updated_at
            FROM storage_location_inventory sli
            JOIN products p ON sli.product_id = p.id
            JOIN storage_locations sl ON sli.storage_location_id = sl.id
            WHERE sl.warehouse_id = $1 AND sl.is_active = true AND sli.on_hand_qty > 0
            ORDER BY p.name, sli.batch_no
            "#,
        )
        .bind(warehouse_id)
        .fetch_all(pool)
        .await?;

        // R35-3 (redo on R35-16): 庫存價值總額 — products.selling_price IS NULL 不計入。
        // 用獨立 query 避免污染 StorageLocationInventoryItem model（selling_price 不在其欄位內）。
        // R35-16 backfill 不做（產品定價空表起步），缺價時 SUM 自動跳過 → 0。
        let total_inventory_value: rust_decimal::Decimal = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(sli.on_hand_qty * p.selling_price), 0)
            FROM storage_location_inventory sli
            JOIN products p ON sli.product_id = p.id
            JOIN storage_locations sl ON sli.storage_location_id = sl.id
            WHERE sl.warehouse_id = $1
              AND sl.is_active = true
              AND sli.on_hand_qty > 0
              AND p.selling_price IS NOT NULL
            "#,
        )
        .bind(warehouse_id)
        .fetch_one(pool)
        .await?;

        // 按 storage_location_id 分組
        let mut inventory_map: HashMap<Uuid, Vec<StorageLocationInventoryItem>> = HashMap::new();
        for item in all_inventory {
            inventory_map
                .entry(item.storage_location_id)
                .or_default()
                .push(item);
        }

        let total_inventory_items: i32 = inventory_map.values().map(|v| v.len() as i32).sum();

        let locations: Vec<StorageLocationWithInventory> = storage_locations
            .iter()
            .map(|sl| {
                let inventory = inventory_map.remove(&sl.id).unwrap_or_default();
                StorageLocationWithInventory {
                    id: sl.id,
                    code: sl.code.clone(),
                    name: sl.name.clone(),
                    location_type: sl.location_type.clone(),
                    row_index: sl.row_index,
                    col_index: sl.col_index,
                    width: sl.width,
                    height: sl.height,
                    capacity: sl.capacity,
                    current_count: sl.current_count,
                    color: sl.color.clone(),
                    is_active: sl.is_active,
                    inventory,
                }
            })
            .collect();

        let active_locations = locations.len() as i32;
        let total_capacity: i32 = locations
            .iter()
            .filter_map(|l| l.capacity.filter(|&c| c > 0))
            .sum();
        let total_current_count: i32 = locations.iter().map(|l| l.current_count).sum();

        let summary = WarehouseReportSummary {
            total_locations: active_locations,
            active_locations,
            total_capacity,
            total_current_count,
            total_inventory_items,
            total_inventory_value,
        };

        Ok(WarehouseReportData {
            warehouse,
            summary,
            locations,
            generated_at: chrono::Utc::now(),
        })
    }

    // ============================================
    // 倉庫匯入
    // ============================================

    /// 匯入倉庫（CSV 或 Excel）— Service-driven audit (N+1 粒度)
    pub async fn import_warehouses(
        pool: &PgPool,
        actor: &ActorContext,
        file_data: &[u8],
        file_name: &str,
    ) -> Result<WarehouseImportResult> {
        let _user = actor.require_user()?;
        let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
        let is_csv = file_name.ends_with(".csv");

        if !is_excel && !is_csv {
            return Err(AppError::Validation(
                "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
            ));
        }

        let rows = if is_excel {
            Self::parse_warehouse_excel(file_data)?
        } else {
            Self::parse_warehouse_csv(file_data)?
        };

        if rows.is_empty() {
            return Err(AppError::Validation("檔案中沒有資料".to_string()));
        }

        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for (index, row) in rows.iter().enumerate() {
            let row_number = (index + 2) as i32;

            if row.name.trim().is_empty() {
                errors.push(WarehouseImportErrorDetail {
                    row: row_number,
                    code: None,
                    error: "名稱為必填欄位".to_string(),
                });
                error_count += 1;
                continue;
            }

            let create_req = CreateWarehouseRequest {
                code: row.code.clone().filter(|s| !s.trim().is_empty()),
                name: row.name.trim().to_string(),
                address: row.address.clone().filter(|s| !s.trim().is_empty()),
            };

            // 匯入路徑的 IP / UA 尚未從 handler 串下來（`import_warehouses` 簽名不收），
            // 與批次 summary audit（WAREHOUSE_IMPORT）現況一致，維持 None。
            match Self::create(pool, actor, &create_req, None).await {
                Ok(_warehouse) => {
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(WarehouseImportErrorDetail {
                        row: row_number,
                        code: row.code.clone(),
                        error: format!("建立失敗: {}", e),
                    });
                    error_count += 1;
                }
            }
        }

        // 批次 summary audit
        let summary_id = Uuid::new_v4();
        let display = format!(
            "import {} (success={}, errors={})",
            file_name, success_count, error_count
        );
        let mut tx = pool.begin().await?;
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "WAREHOUSE_IMPORT",
                entity: Some(AuditEntity::new(
                    "warehouse_import_job",
                    summary_id,
                    &display,
                )),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;

        Ok(WarehouseImportResult {
            success_count,
            error_count,
            errors,
        })
    }

    fn parse_warehouse_csv(file_data: &[u8]) -> Result<Vec<WarehouseImportRow>> {
        let content = String::from_utf8_lossy(file_data);
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());

        let mut rows = Vec::new();
        for (i, result) in reader.records().enumerate() {
            let record = result
                .map_err(|e| AppError::Validation(format!("CSV 解析錯誤第 {} 行: {}", i + 2, e)))?;
            if record.is_empty() {
                continue;
            }
            let name = record.get(0).unwrap_or("").to_string();
            if name.trim().is_empty() {
                continue;
            }
            let code = record
                .get(1)
                .filter(|s| !s.trim().is_empty())
                .map(String::from);
            let address = record
                .get(2)
                .filter(|s| !s.trim().is_empty())
                .map(String::from);

            rows.push(WarehouseImportRow {
                name,
                code,
                address,
            });
        }
        Ok(rows)
    }

    fn parse_warehouse_excel(file_data: &[u8]) -> Result<Vec<WarehouseImportRow>> {
        let range =
            {
                let cursor = Cursor::new(file_data);
                if let Ok(mut wb) = open_workbook_from_rs::<Xlsx<_>, _>(cursor) {
                    let sheet_name = wb.sheet_names().first().cloned().ok_or_else(|| {
                        AppError::Validation("Excel 檔案中沒有工作表".to_string())
                    })?;
                    wb.worksheet_range(&sheet_name)
                        .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))?
                } else {
                    let cursor = Cursor::new(file_data);
                    let mut wb = open_workbook_from_rs::<Xls<_>, _>(cursor).map_err(|_| {
                        AppError::Validation(
                            "無法讀取 Excel 檔案，請使用 .xlsx 或 .xls 格式".to_string(),
                        )
                    })?;
                    let sheet_name = wb.sheet_names().first().cloned().ok_or_else(|| {
                        AppError::Validation("Excel 檔案中沒有工作表".to_string())
                    })?;
                    wb.worksheet_range(&sheet_name)
                        .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))?
                }
            };

        let mut rows = Vec::new();
        let mut iter = range.rows();
        iter.next(); // 跳過標題

        for row in iter {
            if row.is_empty() {
                continue;
            }
            let name = Self::get_cell_string(row.first());
            if name.trim().is_empty() {
                continue;
            }
            let code = {
                let s = Self::get_cell_string(row.get(1));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let address = {
                let s = Self::get_cell_string(row.get(2));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };

            rows.push(WarehouseImportRow {
                name,
                code,
                address,
            });
        }
        Ok(rows)
    }

    fn get_cell_string(cell: Option<&Data>) -> String {
        cell.map(|c| match c {
            Data::String(s) => s.clone(),
            Data::Float(f) => f.to_string(),
            Data::Int(i) => i.to_string(),
            Data::Bool(b) => b.to_string(),
            Data::DateTime(dt) => format!("{:?}", dt),
            _ => String::new(),
        })
        .unwrap_or_default()
    }

    /// 產生倉庫匯入模板
    pub fn generate_import_template() -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let mut workbook = Workbook::new();
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);

        let worksheet = workbook.add_worksheet();
        worksheet.set_column_width(0, 25.0)?;
        worksheet.set_column_width(1, 15.0)?;
        worksheet.set_column_width(2, 40.0)?;

        worksheet.write_string_with_format(0, 0, "名稱*", &header_format)?;
        worksheet.write_string_with_format(0, 1, "代碼", &header_format)?;
        worksheet.write_string_with_format(0, 2, "地址", &header_format)?;

        worksheet.write_string(1, 0, "範例倉庫")?;
        worksheet.write_string(1, 1, "WH001")?;
        worksheet.write_string(1, 2, "")?;

        worksheet.set_freeze_panes(1, 0)?;
        Ok(workbook.save_to_buffer()?)
    }
}

#[cfg(test)]
mod tests {
    use super::WarehouseService;

    #[test]
    fn test_parse_code_sequence_valid() {
        assert_eq!(
            WarehouseService::parse_code_sequence("WH001", "WH"),
            Some(1)
        );
        assert_eq!(
            WarehouseService::parse_code_sequence("WH002", "WH"),
            Some(2)
        );
        assert_eq!(
            WarehouseService::parse_code_sequence("WH999", "WH"),
            Some(999)
        );
    }

    #[test]
    fn test_parse_code_sequence_prefix_mismatch() {
        assert!(WarehouseService::parse_code_sequence("WH001", "WS").is_none());
        assert!(WarehouseService::parse_code_sequence("WS001", "WH").is_none());
    }

    #[test]
    fn test_parse_code_sequence_empty_suffix() {
        assert!(WarehouseService::parse_code_sequence("WH", "WH").is_none());
    }

    #[test]
    fn test_parse_code_sequence_invalid_digits() {
        assert!(WarehouseService::parse_code_sequence("WHabc", "WH").is_none());
    }

    #[test]
    fn test_parse_code_sequence_leading_zeros() {
        assert_eq!(
            WarehouseService::parse_code_sequence("WH001", "WH"),
            Some(1)
        );
        assert_eq!(
            WarehouseService::parse_code_sequence("WH012", "WH"),
            Some(12)
        );
    }

    #[test]
    fn test_parse_code_sequence_case_sensitive() {
        // 前綴區分大小寫
        assert!(WarehouseService::parse_code_sequence("wh001", "WH").is_none());
    }
}
