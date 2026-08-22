use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, CreateStorageLocationRequest, StorageLocation,
        StorageLocationInventoryItem, StorageLocationQuery, StorageLocationWithWarehouse,
        UpdateStorageLayoutRequest, UpdateStorageLocationRequest,
    },
    services::audit::{ActivityLogEntry, AuditEntity, AuditService},
    services::WarehouseService,
    AppError, Result,
};

/// 解析儲位代碼為 (字母, 數字)，例如 `"A07"` → `Some(('A', 7))`。
///
/// 回傳 tuple 讓呼叫端能以「字母為主鍵、數字為次鍵」取最大值——這是取代舊實作
/// `ORDER BY code DESC`（字典序）的關鍵：字典序會把 `A5` 排在 `A99` 之後（誤判為最大）。
/// 格式不符者回 `None` 由呼叫端略過。
fn parse_location_code(code: &str) -> Option<(char, i32)> {
    let mut chars = code.chars();
    let letter = chars.next()?;
    if !letter.is_ascii_uppercase() {
        return None;
    }
    let digits = chars.as_str();
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((letter, digits.parse::<i32>().ok()?))
}

pub struct StorageLocationService;

impl StorageLocationService {
    /// 建立儲位
    pub async fn create(
        pool: &PgPool,
        req: &CreateStorageLocationRequest,
    ) -> Result<StorageLocation> {
        // 整段（產碼 → 查重 → INSERT）必須在同一個交易內：舊實作三步各自打連線池，
        // 並發時兩個請求會拿到同一個 code、雙雙通過查重，最後撞
        // `storage_locations_warehouse_id_code_key`。
        let mut tx = pool.begin().await?;

        // 若沒有提供 code，自動產生一個（產碼會持鎖至 commit）
        let code = match &req.code {
            Some(c) if !c.is_empty() => c.clone(),
            _ => Self::generate_code_tx(&mut tx, req.warehouse_id).await?,
        };

        // 檢查 code 在該倉庫內是否已存在
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM storage_locations WHERE warehouse_id = $1 AND code = $2)",
        )
        .bind(req.warehouse_id)
        .bind(&code)
        .fetch_one(&mut *tx)
        .await?;

        if exists {
            return Err(AppError::Conflict(
                "Storage location code already exists in this warehouse".to_string(),
            ));
        }

        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            INSERT INTO storage_locations 
                (id, warehouse_id, code, name, location_type, row_index, col_index, 
                 width, height, capacity, color, config, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(req.warehouse_id)
        .bind(&code)
        .bind(Some(&req.name))
        .bind(req.location_type.as_deref().unwrap_or("shelf"))
        .bind(req.row_index.unwrap_or(0))
        .bind(req.col_index.unwrap_or(0))
        .bind(req.width.unwrap_or(2))
        .bind(req.height.unwrap_or(2))
        .bind(req.capacity)
        .bind(&req.color)
        .bind(&req.config)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(location)
    }

    /// 列出儲位
    pub async fn list(
        pool: &PgPool,
        query: &StorageLocationQuery,
    ) -> Result<Vec<StorageLocationWithWarehouse>> {
        let mut sql = String::from(
            r#"
            SELECT 
                sl.id, sl.warehouse_id, w.code as warehouse_code, w.name as warehouse_name,
                sl.code, sl.name, sl.location_type, sl.row_index, sl.col_index,
                sl.width, sl.height, sl.capacity, sl.current_count, sl.color,
                sl.is_active, sl.config
            FROM storage_locations sl
            JOIN warehouses w ON sl.warehouse_id = w.id
            WHERE 1=1
            "#,
        );

        let mut param_idx = 1;

        if query.warehouse_id.is_some() {
            sql.push_str(&format!(" AND sl.warehouse_id = ${}", param_idx));
            param_idx += 1;
        }
        if query.location_type.is_some() {
            sql.push_str(&format!(" AND sl.location_type = ${}", param_idx));
            param_idx += 1;
        }
        if query.is_active.is_some() {
            sql.push_str(&format!(" AND sl.is_active = ${}", param_idx));
            param_idx += 1;
        }
        if query.keyword.is_some() {
            sql.push_str(&format!(
                " AND (sl.code ILIKE ${} OR sl.name ILIKE ${})",
                param_idx, param_idx
            ));
        }

        sql.push_str(" ORDER BY sl.row_index, sl.col_index, sl.code");

        // 動態綁定參數
        let mut query_builder =
            sqlx::query_as::<_, StorageLocationWithWarehouse>(sqlx::AssertSqlSafe(sql));

        if let Some(ref warehouse_id) = query.warehouse_id {
            query_builder = query_builder.bind(warehouse_id);
        }
        if let Some(ref location_type) = query.location_type {
            query_builder = query_builder.bind(location_type);
        }
        if let Some(is_active) = query.is_active {
            query_builder = query_builder.bind(is_active);
        }
        if let Some(ref keyword) = query.keyword {
            query_builder = query_builder.bind(format!("%{}%", keyword));
        }

        let locations = query_builder.fetch_all(pool).await?;
        Ok(locations)
    }

    /// 取得單一儲位
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<StorageLocationWithWarehouse> {
        let location = sqlx::query_as::<_, StorageLocationWithWarehouse>(
            r#"
            SELECT 
                sl.id, sl.warehouse_id, w.code as warehouse_code, w.name as warehouse_name,
                sl.code, sl.name, sl.location_type, sl.row_index, sl.col_index,
                sl.width, sl.height, sl.capacity, sl.current_count, sl.color,
                sl.is_active, sl.config
            FROM storage_locations sl
            JOIN warehouses w ON sl.warehouse_id = w.id
            WHERE sl.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Storage location not found".to_string()))?;

        Ok(location)
    }

    /// 更新儲位
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateStorageLocationRequest,
    ) -> Result<StorageLocation> {
        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            UPDATE storage_locations SET
                code = COALESCE($1, code),
                name = COALESCE($2, name),
                location_type = COALESCE($3, location_type),
                row_index = COALESCE($4, row_index),
                col_index = COALESCE($5, col_index),
                width = COALESCE($6, width),
                height = COALESCE($7, height),
                capacity = COALESCE($8, capacity),
                color = COALESCE($9, color),
                is_active = COALESCE($10, is_active),
                config = COALESCE($11, config),
                updated_at = NOW()
            WHERE id = $12
            RETURNING *
            "#,
        )
        .bind(&req.code)
        .bind(&req.name)
        .bind(&req.location_type)
        .bind(req.row_index)
        .bind(req.col_index)
        .bind(req.width)
        .bind(req.height)
        .bind(req.capacity)
        .bind(&req.color)
        .bind(req.is_active)
        .bind(&req.config)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Storage location not found".to_string()))?;

        Ok(location)
    }

    /// 批次更新儲位佈局（拖拽後儲存）
    pub async fn update_layout(
        pool: &PgPool,
        warehouse_id: Uuid,
        req: &UpdateStorageLayoutRequest,
    ) -> Result<Vec<StorageLocation>> {
        let mut tx = pool.begin().await?;

        for item in &req.items {
            sqlx::query(
                r#"
                UPDATE storage_locations SET
                    row_index = $1,
                    col_index = $2,
                    width = $3,
                    height = $4,
                    updated_at = NOW()
                WHERE id = $5 AND warehouse_id = $6
                "#,
            )
            .bind(item.row_index)
            .bind(item.col_index)
            .bind(item.width)
            .bind(item.height)
            .bind(item.id)
            .bind(warehouse_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // 回傳更新後的儲位列表
        let locations = sqlx::query_as::<_, StorageLocation>(
            "SELECT * FROM storage_locations WHERE warehouse_id = $1 ORDER BY row_index, col_index, code",
        )
        .bind(warehouse_id)
        .fetch_all(pool)
        .await?;

        Ok(locations)
    }

    /// 刪除儲位
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM storage_locations WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Storage location not found".to_string()));
        }

        Ok(())
    }

    /// 於 transaction 內取得儲位代碼生成鎖。
    ///
    /// ⚠️ 鎖必須持有到交易 commit（`pg_advisory_xact_lock` 即為此語意）：READ COMMITTED
    /// 下若在 INSERT 尚未 commit 前放鎖，後到的交易讀不到那筆資料、會算出同一個代碼，
    /// 鎖等於失效。故取號**必須與 INSERT 在同一個交易內**。
    async fn acquire_code_lock(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<()> {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(crate::constants::STORAGE_LOCATION_CODE_LOCK_KEY)
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    /// 產生代碼供前端**預覽**（`GET /warehouses/{id}/storage-locations/generate-code`）。
    ///
    /// ⚠️ 這是 best-effort 建議值，不是保留。真正的配號由 `create` 在交易內重做
    /// （見 `generate_code_tx`）——兩次呼叫之間別人可能已用掉同一個號。
    pub async fn generate_code(pool: &PgPool, warehouse_id: Uuid) -> Result<String> {
        let mut tx = pool.begin().await?;
        let code = Self::generate_code_tx(&mut tx, warehouse_id).await?;
        // 唯讀交易，rollback 即可釋放 advisory lock。
        tx.rollback().await?;
        Ok(code)
    }

    /// 自動產生儲位代碼（交易內版本，取號的唯一真相源）。格式 `{A-Z}{:02}`。
    async fn generate_code_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        warehouse_id: Uuid,
    ) -> Result<String> {
        Self::acquire_code_lock(tx).await?;

        // 撈出該倉庫全部符合格式的代碼，在 Rust 端解析後取最大值。
        // 不可用 `ORDER BY code DESC LIMIT 1`——那是**字典序**：一旦出現未補零的
        // `A5`，它會排在 `A99` 之前而被誤判為最大值，於是算出 A06 撞既有代碼。
        let codes: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT code FROM storage_locations
            WHERE warehouse_id = $1 AND code ~ '^[A-Z][0-9]+$'
            "#,
        )
        .bind(warehouse_id)
        .fetch_all(&mut **tx)
        .await?;

        // 以 (字母, 數字) 排序取最大：字母是主鍵、數字是次鍵。
        let max = codes
            .iter()
            .filter_map(|code| parse_location_code(code))
            .max();

        let next_code = match max {
            None => "A01".to_string(),
            Some((letter, num)) if num < 99 => format!("{}{:02}", letter, num + 1),
            Some((letter, _)) => {
                // 該字母用滿 99，換下一個字母。'Z' 之後沒有合法字母可用——
                // 舊實作會產生 `[01`（'Z' + 1），既不符 `^[A-Z][0-9]+$` 而永遠
                // 不再被本查詢看見，之後每次都重算出同一個值而反覆撞號。
                // 明確報錯，不產出無效代碼。
                let next = ((letter as u8) + 1) as char;
                if !next.is_ascii_uppercase() {
                    return Err(AppError::BusinessRule(
                        "儲位代碼已用盡（A01–Z99），請改為手動指定代碼".to_string(),
                    ));
                }
                format!("{}01", next)
            }
        };

        Ok(next_code)
    }

    /// 取得儲位庫存明細
    pub async fn get_inventory(
        pool: &PgPool,
        storage_location_id: Uuid,
    ) -> Result<Vec<crate::models::StorageLocationInventoryItem>> {
        let items = sqlx::query_as::<_, crate::models::StorageLocationInventoryItem>(
            r#"
            SELECT 
                sli.id,
                sli.storage_location_id,
                sli.product_id,
                p.sku as product_sku,
                p.name as product_name,
                p.spec as product_spec,
                sli.on_hand_qty,
                p.base_uom,
                sli.batch_no,
                sli.expiry_date,
                sli.updated_at
            FROM storage_location_inventory sli
            JOIN products p ON sli.product_id = p.id
            WHERE sli.storage_location_id = $1 AND sli.on_hand_qty > 0
            ORDER BY p.name, sli.batch_no
            "#,
        )
        .bind(storage_location_id)
        .fetch_all(pool)
        .await?;

        Ok(items)
    }

    /// 更新儲位庫存項目數量（#197：補 actor + tx + Service-driven audit；庫存＝財務紀錄）
    pub async fn update_inventory_item(
        pool: &PgPool,
        actor: &ActorContext,
        item_id: Uuid,
        req: &crate::models::UpdateStorageLocationInventoryItemRequest,
    ) -> Result<crate::models::StorageLocationInventoryItem> {
        // Validate non-negative quantity (since validator crate doesn't support rust_decimal)
        if req.on_hand_qty < rust_decimal::Decimal::ZERO {
            return Err(AppError::Validation(
                "on_hand_qty must be non-negative".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

        // before snapshot（FOR UPDATE 鎖定該庫存列）
        let before: crate::models::StorageLocationInventoryItem = sqlx::query_as(
            r#"
            SELECT sli.id, sli.storage_location_id, sli.product_id,
                   p.sku as product_sku, p.name as product_name, p.spec as product_spec,
                   sli.on_hand_qty, p.base_uom, sli.batch_no, sli.expiry_date, sli.updated_at
            FROM storage_location_inventory sli
            JOIN products p ON sli.product_id = p.id
            WHERE sli.id = $1
            FOR UPDATE OF sli
            "#,
        )
        .bind(item_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Inventory item not found".to_string()))?;

        let storage_location_id = before.storage_location_id;
        WarehouseService::ensure_active_for_location_tx(&mut tx, storage_location_id).await?;

        // 更新庫存數量
        sqlx::query("UPDATE storage_location_inventory SET on_hand_qty = $1, updated_at = NOW() WHERE id = $2")
            .bind(req.on_hand_qty)
            .bind(item_id)
            .execute(&mut *tx)
            .await?;

        // 更新儲位的 current_count
        sqlx::query(
            r#"
            UPDATE storage_locations
            SET current_count = (
                SELECT COUNT(*) FROM storage_location_inventory
                WHERE storage_location_id = $1 AND on_hand_qty > 0
            ),
            updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(storage_location_id)
        .execute(&mut *tx)
        .await?;

        // 回傳更新後的項目
        let item = sqlx::query_as::<_, crate::models::StorageLocationInventoryItem>(
            r#"
            SELECT sli.id, sli.storage_location_id, sli.product_id,
                   p.sku as product_sku, p.name as product_name, p.spec as product_spec,
                   sli.on_hand_qty, p.base_uom, sli.batch_no, sli.expiry_date, sli.updated_at
            FROM storage_location_inventory sli
            JOIN products p ON sli.product_id = p.id
            WHERE sli.id = $1
            "#,
        )
        .bind(item_id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} @ {}", item.product_sku, item.storage_location_id);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "STORAGE_INVENTORY_UPDATE",
                entity: Some(AuditEntity::new(
                    "storage_location_inventory",
                    item.id,
                    &display,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&item))),
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(item)
    }

    /// 新增儲位庫存項目
    ///
    /// R26-13（critical review 新發現 HIGH）：改用 SELECT FOR UPDATE + 顯式
    /// INSERT/UPDATE 分支，取代原本的 ON CONFLICT DO UPDATE upsert pattern。
    ///
    /// 原 upsert 問題：INSERT...ON CONFLICT DO UPDATE 無法在 app 層區分實際
    /// 執行 INSERT 還是 UPDATE，audit 缺 before snapshot。兩並發請求同時送達
    /// 也會互相覆蓋。
    ///
    /// 修補：
    /// 1. 開 transaction
    /// 2. SELECT FOR UPDATE 鎖定既有 row（若有）— 取得 before snapshot
    /// 3. 依結果顯式決定 INSERT 或 UPDATE
    /// 4. `log_activity_tx` 在同一 tx 寫 audit（CREATE 或 UPDATE 類）
    /// 5. 更新 current_count 在同一 tx
    /// 6. Commit
    pub async fn create_inventory_item(
        pool: &PgPool,
        actor: &ActorContext,
        storage_location_id: Uuid,
        req: &crate::models::CreateStorageLocationInventoryItemRequest,
    ) -> Result<StorageLocationInventoryItem> {
        if req.on_hand_qty < rust_decimal::Decimal::ZERO {
            return Err(AppError::Validation(
                "Quantity must be non-negative".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

        // 驗證儲位存在
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM storage_locations WHERE id = $1")
            .bind(storage_location_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("Storage location not found".to_string()))?;

        WarehouseService::ensure_active_for_location_tx(&mut tx, storage_location_id).await?;

        // SELECT FOR UPDATE 既有 row（若有）— 用與 ON CONFLICT 相同的 COALESCE 匹配
        let before: Option<StorageLocationInventoryItem> = sqlx::query_as(
            r#"
            SELECT
                sli.id, sli.storage_location_id, sli.product_id,
                p.sku AS product_sku, p.name AS product_name, p.spec AS product_spec,
                sli.on_hand_qty, p.base_uom, sli.batch_no, sli.expiry_date,
                sli.updated_at
            FROM storage_location_inventory sli
            JOIN products p ON sli.product_id = p.id
            WHERE sli.storage_location_id = $1
              AND sli.product_id = $2
              AND COALESCE(sli.batch_no, '') = COALESCE($3::VARCHAR, '')
              AND COALESCE(sli.expiry_date, '1900-01-01'::date) = COALESCE($4::DATE, '1900-01-01'::date)
            FOR UPDATE OF sli
            "#,
        )
        .bind(storage_location_id)
        .bind(req.product_id)
        .bind(&req.batch_no)
        .bind(req.expiry_date)
        .fetch_optional(&mut *tx)
        .await?;

        // 依是否存在既有 row 決定 INSERT 或 UPDATE
        let (after, event_type) = if let Some(ref existing) = before {
            // UPDATE 路徑
            sqlx::query(
                r#"
                UPDATE storage_location_inventory
                SET on_hand_qty = on_hand_qty + $1, updated_at = NOW()
                WHERE id = $2
                "#,
            )
            .bind(req.on_hand_qty)
            .bind(existing.id)
            .execute(&mut *tx)
            .await?;

            let updated: StorageLocationInventoryItem = sqlx::query_as(
                r#"
                SELECT
                    sli.id, sli.storage_location_id, sli.product_id,
                    p.sku AS product_sku, p.name AS product_name, p.spec AS product_spec,
                    sli.on_hand_qty, p.base_uom, sli.batch_no, sli.expiry_date,
                    sli.updated_at
                FROM storage_location_inventory sli
                JOIN products p ON sli.product_id = p.id
                WHERE sli.id = $1
                "#,
            )
            .bind(existing.id)
            .fetch_one(&mut *tx)
            .await?;
            (updated, "STORAGE_INVENTORY_UPDATE")
        } else {
            // INSERT 路徑
            let item_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO storage_location_inventory (
                    id, storage_location_id, product_id, on_hand_qty,
                    batch_no, expiry_date, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, NOW())
                "#,
            )
            .bind(item_id)
            .bind(storage_location_id)
            .bind(req.product_id)
            .bind(req.on_hand_qty)
            .bind(&req.batch_no)
            .bind(req.expiry_date)
            .execute(&mut *tx)
            .await?;

            let inserted: StorageLocationInventoryItem = sqlx::query_as(
                r#"
                SELECT
                    sli.id, sli.storage_location_id, sli.product_id,
                    p.sku AS product_sku, p.name AS product_name, p.spec AS product_spec,
                    sli.on_hand_qty, p.base_uom, sli.batch_no, sli.expiry_date,
                    sli.updated_at
                FROM storage_location_inventory sli
                JOIN products p ON sli.product_id = p.id
                WHERE sli.id = $1
                "#,
            )
            .bind(item_id)
            .fetch_one(&mut *tx)
            .await?;
            (inserted, "STORAGE_INVENTORY_CREATE")
        };

        // audit log 在同一 tx
        let display = format!("{} @ {}", after.product_sku, after.storage_location_id);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type,
                entity: Some(AuditEntity::new(
                    "storage_location_inventory",
                    after.id,
                    &display,
                )),
                data_diff: Some(DataDiff::compute(before.as_ref(), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        // 更新儲位的 current_count（同 tx）
        sqlx::query(
            r#"
            UPDATE storage_locations
            SET current_count = (
                SELECT COUNT(*) FROM storage_location_inventory
                WHERE storage_location_id = $1 AND on_hand_qty > 0
            ),
            updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(storage_location_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(after)
    }
}

#[cfg(test)]
mod tests {
    use super::parse_location_code;

    #[test]
    fn parses_location_code() {
        assert_eq!(parse_location_code("A01"), Some(('A', 1)));
        assert_eq!(parse_location_code("Z99"), Some(('Z', 99)));
        // 未補零：舊的字典序實作正是死在這種值上
        assert_eq!(parse_location_code("A5"), Some(('A', 5)));
        // 格式不符者一律略過，不得被當成 0
        assert_eq!(parse_location_code("a01"), None);
        assert_eq!(parse_location_code("AA1"), None);
        assert_eq!(parse_location_code("A"), None);
        assert_eq!(parse_location_code(""), None);
        // 舊實作在 Z99 之後會產生的無效代碼，不可再被解析回來
        assert_eq!(parse_location_code("[01"), None);
    }

    /// 取最大值必須是 (字母, 數字) 的字典序 → 數值序，而非整串字串的字典序。
    #[test]
    fn max_is_numeric_not_lexicographic() {
        let codes = ["A01", "A5", "A99", "B01"];
        let max = codes
            .iter()
            .filter_map(|c| parse_location_code(c))
            .max()
            .expect("有可解析的代碼");
        assert_eq!(max, ('B', 1), "字母才是主鍵，B01 應大於 A99");

        // 同字母內比數字：字串字典序會誤判 "A5" > "A99"
        let same_letter = ["A01", "A5", "A99"];
        let max = same_letter
            .iter()
            .filter_map(|c| parse_location_code(c))
            .max()
            .expect("有可解析的代碼");
        assert_eq!(max, ('A', 99), "同字母取數值最大，不是字典序最大的 A5");
    }
}
