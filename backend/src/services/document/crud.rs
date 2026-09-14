use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        audit_diff::DataDiff, CreateDocumentRequest, DocStatus, DocType, Document,
        DocumentAuditSnapshot, DocumentLine, DocumentLineInput, DocumentLineWithProduct,
        DocumentListItem, DocumentQuery, DocumentWithLines, ProtocolStatus, UpdateDocumentRequest,
    },
    repositories,
    services::{
        access,
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    time, AppError, Result,
};

use super::DocumentService;

/// 業務錯誤碼：line 缺 storage_location_id（GRN/DO/SO/ADJ/STK）。
/// 前端可用此 code 對應 i18n key 或 telemetry。
const ERR_DOC_LINE_SHELF_REQUIRED: &str = "doc.line.shelf_required";

/// `get_by_id` 的關聯顯示名稱中繼結果。
struct DocumentRelatedNames {
    warehouse_name: Option<String>,
    warehouse_from_name: Option<String>,
    warehouse_to_name: Option<String>,
    partner_name: Option<String>,
    protocol_no: Option<String>,
    created_by_name: String,
    approved_by_name: Option<String>,
}

/// `get_by_id` 的沖銷雙向關聯中繼結果（R84-5）。
struct ReversalLinks {
    reverses_doc_no: Option<String>,
    reversed_by_doc_id: Option<Uuid>,
    reversed_by_doc_no: Option<String>,
    reversed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl DocumentService {
    /// 明細數量 / 單價範圍驗證（defense-in-depth，#512 D 之 document 半部）。
    ///
    /// - GRN（採購入庫）：單價必填且 > 0（建立/更新兩路徑一致，避免規則分叉）。
    /// - SO（銷貨）：**不接受單價**（2026-07-21 使用者裁定：ERP 銷貨全為內部耗材領用，
    ///   不記金額；byproduct/生物材料另有專屬模組不走 SO）。核准逐行扣庫存，
    ///   會計僅記成本轉列（存貨→銷貨成本，進貨平均成本），永不認列收入。
    /// - 其餘單據：單價（若提供）一律不可為負。
    /// - STK（盤點）：計數可為 0（甚至缺貨歸零），qty 由盤點流程處理，跳過。
    /// - ADJ（調整）：qty 帶號（>0 入庫 / <0 出庫，見 ledger::process_adjustment），僅 0 為無意義。
    /// - 其餘移動單據（GRN/PR/TR/SR/RTN）：qty 必須 > 0，否則負數會反向污染庫存。
    fn validate_line_qty_price(doc_type: DocType, lines: &[DocumentLineInput]) -> Result<()> {
        for (idx, line) in lines.iter().enumerate() {
            // GRN 採購入庫單價必填且 > 0（進貨成本是平均成本法的源頭，不可缺）。
            if doc_type == DocType::GRN {
                let price_invalid = match line.unit_price {
                    None => true,
                    Some(d) if d <= Decimal::ZERO => true,
                    _ => false,
                };
                if price_invalid {
                    return Err(AppError::Validation(format!(
                        "第 {} 行：採購入庫的單價為必填，且必須大於 0",
                        idx + 1
                    )));
                }
            } else if doc_type == DocType::SO && line.unit_price.is_some() {
                return Err(AppError::Validation(format!(
                    "第 {} 行：內部銷貨不記單價，請留空",
                    idx + 1
                )));
            } else if matches!(line.unit_price, Some(p) if p < Decimal::ZERO) {
                return Err(AppError::Validation(format!(
                    "第 {} 行：單價不可為負數",
                    idx + 1
                )));
            }
            // STK 盤點：實盤數恆 >= 0。
            // 0 是合法且重要的結果——「系統有、現場沒有」＝全數短少，擋掉它等於無法登記。
            // 但負數在盤點語意下不存在（2026-09-14 使用者裁定）。原本此處直接 `continue`
            // 跳過所有數量檢查，連 -5 的實盤數都會收下。
            if doc_type == DocType::STK {
                if line.qty < Decimal::ZERO {
                    return Err(AppError::Validation(format!(
                        "第 {} 行：盤點數量不可為負數",
                        idx + 1
                    )));
                }
                continue;
            }
            let qty_invalid = if doc_type == DocType::ADJ {
                line.qty == Decimal::ZERO
            } else {
                line.qty <= Decimal::ZERO
            };
            if qty_invalid {
                return Err(AppError::Validation(format!(
                    "第 {} 行：數量不正確（調整單不可為 0，其餘單據須大於 0）",
                    idx + 1
                )));
            }
        }
        Ok(())
    }

    /// 明細單位驗證：每行的 `uom` 必須是該品項的 `base_uom`，或 `product_uom_conversions`
    /// 已定義的單位。
    ///
    /// 為什麼要在建/改單就擋，而不是留給核准時的 `StockService::to_base_lines`：
    /// (a) 使用者填錯單位不該等到送審最後一步才知道；
    /// (b) `document_lines.uom` 原本只驗 `!is_empty()`，是自由字串——GRN 與 SO 對同一品項
    ///     各填不同單位時，PO/GRN 收貨比對那組 `SUM(qty)` 會把盒與雙直接相加。那些 SQL
    ///     以「查無換算列 ⇒ 該行就是 base_uom ⇒ factor 1」推論，本函式正是那個推論的前提。
    async fn assert_lines_uom_defined(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        lines: &[DocumentLineInput],
    ) -> Result<()> {
        if lines.is_empty() {
            return Ok(());
        }
        let mut product_ids: Vec<Uuid> = lines.iter().map(|l| l.product_id).collect();
        product_ids.sort_unstable();
        product_ids.dedup();
        let tables = crate::services::StockService::load_uom_tables(tx, &product_ids).await?;

        for (idx, line) in lines.iter().enumerate() {
            let table = tables.get(&line.product_id).ok_or_else(|| {
                AppError::NotFound(format!("第 {} 行：找不到品項 {}", idx + 1, line.product_id))
            })?;
            if table.factor(&line.uom).is_none() {
                return Err(AppError::Validation(format!(
                    "第 {} 行：品項未定義單位「{}」的換算率，可用單位：{}",
                    idx + 1,
                    line.uom,
                    table.accepted_uoms().join(" / ")
                )));
            }
        }
        Ok(())
    }

    /// 跨倉錯配防護：single-warehouse 單據（GRN/DO/SO/ADJ/STK/SR/RTN/PR）的每行儲位
    /// 必須屬於單據倉庫。否則 stock_ledger.warehouse ≠ 儲位.warehouse，「未分配 / 已在儲位」
    /// 視圖對不上（倉庫級查 ledger、儲位級查 storage_location_inventory）。TR 調撥（from/to
    /// 兩倉）語意不同，其目標儲位屬 warehouse_to，不在此擋。
    /// 驗證明細儲位皆屬於單據倉庫（跨倉錯配防護）。傳入「最終有效」儲位 id（呼叫端負責
    /// 從 req.lines 或既有明細萃取），使得只改倉庫、不帶明細的部分更新也能被檢查。
    async fn assert_lines_shelf_in_warehouse(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        doc_type: DocType,
        warehouse_id: Option<Uuid>,
        loc_ids: &[Uuid],
    ) -> Result<()> {
        // TR 調撥 from/to 兩倉另計；SO 多倉銷貨每行倉庫以自身儲位為準（migration 136，ledger 逐行
        // 跟隨儲位），不受單一表頭倉庫約束，故兩者皆跳過本檢查。
        if matches!(doc_type, DocType::TR | DocType::SO) {
            return Ok(());
        }
        let Some(doc_wh) = warehouse_id else {
            return Ok(());
        };
        if loc_ids.is_empty() {
            return Ok(());
        }
        let mismatched: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM storage_locations WHERE id = ANY($1) AND warehouse_id <> $2 LIMIT 1",
        )
        .bind(loc_ids)
        .bind(doc_wh)
        .fetch_optional(&mut **tx)
        .await?;
        if mismatched.is_some() {
            return Err(AppError::BusinessRule(
                "明細的儲位不屬於本單據的倉庫（跨倉上架會造成庫存視圖不一致）".to_string(),
            ));
        }
        Ok(())
    }

    /// 批次載入品項的 batch/expiry 追蹤旗標，消除「每明細逐筆查 products」的 N+1。
    /// 回傳 product_id → (track_batch, track_expiry)；不在表中的 id 不會有 entry。
    async fn load_product_track_flags(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        product_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, (bool, bool)>> {
        // 去重 + 空集合早返回：避免重複 product_id 膨脹參數、空單免一次 DB 往返。
        let unique_ids: Vec<Uuid> = product_ids
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        if unique_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let rows: Vec<(Uuid, bool, bool)> =
            sqlx::query_as("SELECT id, track_batch, track_expiry FROM products WHERE id = ANY($1)")
                .bind(&unique_ids)
                .fetch_all(&mut **tx)
                .await?;
        Ok(rows.into_iter().map(|(id, b, e)| (id, (b, e))).collect())
    }

    /// 批次載入儲位 → 所屬倉庫映射（SO 多倉銷貨：每行倉庫從其儲位反推回填，migration 136）。
    /// 回傳 storage_location_id → warehouse_id；不存在的儲位不會有 entry。
    async fn load_shelf_warehouses(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        loc_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Uuid>> {
        let unique_ids: Vec<Uuid> = loc_ids
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        if unique_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let rows: Vec<(Uuid, Uuid)> =
            sqlx::query_as("SELECT id, warehouse_id FROM storage_locations WHERE id = ANY($1)")
                .bind(&unique_ids)
                .fetch_all(&mut **tx)
                .await?;
        Ok(rows.into_iter().collect())
    }

    /// 銷貨 / 出庫（SO / DO）開立授權（#61）。
    ///
    /// 僅「**該計畫的** SD（計劃負責人，`protocols.study_director_user_id`）」或 admin
    /// 可開立。避免任何具 `erp.document.create` 者把銷貨/出庫掛到不是自己負責的已核准計畫。
    ///
    /// 修復：舊版於 handler 只查全域 `STUDY_DIRECTOR` 角色，但計畫層級 SD 是自 `EXPERIMENT_STAFF`
    /// 挑選、通常不具該全域角色，導致計畫負責人替自己的計畫開銷貨反被擋下。此處補上「該計畫 SD」路徑。
    ///
    /// 無計畫單（外部客戶銷貨，`protocol_id = None`）：2026-07-20 裁定放寬——任一「已核准」
    /// 計畫的計畫層級 SD 亦可開立（非僅 admin）。
    ///
    /// ⚠️ **admin 那條必須是顯式的 `is_admin()`**。原本寫成
    /// `has_permission("aup.protocol.sd_authority")`，靠的是 `has_permission` 對 admin
    /// 短路回 true 這個副作用；該權限碼隨全域 `STUDY_DIRECTOR` 角色一併移除後，
    /// 若只是把那行刪掉，admin 會失去開立**無計畫**外部客戶銷貨單的能力
    /// （`is_study_director_of_any_approved` 對 admin 回 false），是實際的功能迴歸。
    async fn authorize_sales_document(
        pool: &PgPool,
        user: &CurrentUser,
        protocol_id: Option<Uuid>,
    ) -> Result<()> {
        if user.is_admin() {
            return Ok(());
        }
        match protocol_id {
            Some(pid) => {
                if access::is_study_director(pool, pid, user.id).await? {
                    return Ok(());
                }
            }
            None => {
                if access::is_study_director_of_any_approved(pool, user.id).await? {
                    return Ok(());
                }
            }
        }
        Err(AppError::Forbidden(
            "僅該計畫 SD（計劃負責人）或管理員可開立銷貨 / 出貨單".into(),
        ))
    }

    /// 建立單據（草稿）— Service-driven audit
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateDocumentRequest,
    ) -> Result<DocumentWithLines> {
        let user = actor.require_user()?;
        let created_by = user.id;

        // R84-9（2026-07-23）：原本此處有「DO 已棄用 → 封鎖新建」的特例檢查。
        // `DocType::DO` 已從 Rust enum 移除，反序列化階段就無法產生該值，
        // 此特例成為不可能路徑，故一併清除（DB enum 值仍保留，見 `DocType` 說明）。

        // #61：銷貨單（SO）僅該計畫 SD / 研究主持人 / admin 可開立（fail-fast）。
        if req.doc_type == DocType::SO {
            Self::authorize_sales_document(pool, user, req.protocol_id).await?;
        }

        let mut tx = pool.begin().await?;

        // 產生單據編號
        let doc_no = Self::generate_doc_no(&mut tx, req.doc_type).await?;

        // 如果是盤點單，根據範圍自動生成盤點項目
        let lines_to_create = if req.doc_type == DocType::STK {
            // 範圍驗證在分支**之前**：底稿自帶時不會經過 generate_stocktake_lines，
            // 但下面的 INSERT 仍會把 stocktake_scope 原樣寫進單據。驗證若只掛在
            // 產生底稿那條路上，自帶明細就能把形狀非法的範圍安靜地存進資料庫。
            Self::parse_and_validate_stocktake_scope(&req.stocktake_scope)?;

            // 盤點單可以根據範圍自動生成，也可以手動提供
            if req.lines.is_empty() {
                Self::generate_stocktake_lines(&mut tx, req.warehouse_id, &req.stocktake_scope)
                    .await?
            } else {
                req.lines.clone()
            }
        } else {
            // 其他單據必須提供明細
            if req.lines.is_empty() {
                return Err(AppError::Validation(
                    "At least one line is required".to_string(),
                ));
            }

            // GRN 單價必填且 > 0 的檢查已下沉 validate_line_qty_price（create / update
            // 共用同一驗證），確保兩路徑規則一致。

            // 強制檢查批號與效期 (特定單據類型結合品項設定)
            if req.doc_type.requires_batch_expiry() {
                // N+1 修復：一次撈本單所有品項的追蹤旗標，迴圈內查表（取代逐明細 SELECT）。
                let product_ids: Vec<Uuid> = req.lines.iter().map(|l| l.product_id).collect();
                let track_flags = Self::load_product_track_flags(&mut tx, &product_ids).await?;
                for (idx, line) in req.lines.iter().enumerate() {
                    if let Some(&(track_batch, track_expiry)) = track_flags.get(&line.product_id) {
                        if track_batch && line.batch_no.as_ref().is_none_or(|s| s.trim().is_empty())
                        {
                            return Err(AppError::Validation(format!(
                                "Line {}: Batch No is required for {} when product tracks batch",
                                idx + 1,
                                req.doc_type.prefix()
                            )));
                        }
                        if track_expiry && line.expiry_date.is_none() {
                            return Err(AppError::Validation(format!(
                                "Line {}: Expiry Date is required for {} when product tracks expiry",
                                idx + 1,
                                req.doc_type.prefix()
                            )));
                        }
                    }
                }
            }

            req.lines.clone()
        };

        // 儲位/貨架硬性必填（DO/SO/ADJ/STK/SR/RTN）— 對應前端 useDocumentSubmit.ts。
        // GRN（採購入庫）已改軟擋（見 DocType::shelf_soft_expected）：缺儲位可建/核准，
        // 事後分配上架，故不在此擋。之前僅前端擋；外部 API 或繞過前端會留下
        // storage_location_id IS NULL 的歷史單據 → 庫存資料不一致。本層為 single source of truth。
        //
        // 必須對 lines_to_create 而非 req.lines 驗證：STK 走 generate_stocktake_lines 自動產生
        // 的 lines 也須一併檢查（生成端理論上會帶 storage_location_id，但若未來實作改動或
        // 庫存源頭資料為 NULL 仍會洩漏到 audit / 庫存層）。
        if req.doc_type.requires_shelf() {
            for (idx, line) in lines_to_create.iter().enumerate() {
                if line.storage_location_id.is_none() {
                    return Err(AppError::ValidationWithCode {
                        code: ERR_DOC_LINE_SHELF_REQUIRED.to_string(),
                        message: format!("第 {} 行：儲位/貨架為必填項", idx + 1),
                    });
                }
            }
        }

        // 跨倉錯配防護（single source of truth）：單倉單據的每行儲位必須屬於本單據的倉庫。
        // 否則 stock_ledger.warehouse ≠ 儲位.warehouse，「未分配 / 已在儲位」視圖會對不上
        // （倉庫級查 ledger、儲位級查 storage_location_inventory）。assign_unassigned(#978)
        // 已於「分配」路徑擋，本層補上「建單即帶儲位」這條路徑。TR 調撥走 from/to 另計不在此擋。
        let create_loc_ids: Vec<Uuid> = lines_to_create
            .iter()
            .filter_map(|l| l.storage_location_id)
            .collect();
        Self::assert_lines_shelf_in_warehouse(
            &mut tx,
            req.doc_type,
            req.warehouse_id,
            &create_loc_ids,
        )
        .await?;

        Self::validate_line_qty_price(req.doc_type, &lines_to_create)?;
        Self::assert_lines_uom_defined(&mut tx, &lines_to_create).await?;

        // 驗證 protocol_id 對應的計畫存在且為已核准狀態
        if let Some(protocol_id) = req.protocol_id {
            let status: ProtocolStatus =
                sqlx::query_scalar("SELECT status FROM protocols WHERE id = $1")
                    .bind(protocol_id)
                    .fetch_optional(&mut *tx)
                    .await?
                    .ok_or_else(|| {
                        AppError::NotFound(format!("Protocol {} not found", protocol_id))
                    })?;

            if status != ProtocolStatus::Approved
                && status != ProtocolStatus::ApprovedWithConditions
            {
                return Err(AppError::Validation(format!(
                    "Protocol {} is not in approved status (current: {})",
                    protocol_id,
                    status.display_name()
                )));
            }
        }

        // 建立單據頭
        let document = sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (
                id, doc_type, doc_no, status, warehouse_id, warehouse_from_id, warehouse_to_id,
                partner_id, source_doc_id, doc_date, remark, stocktake_scope, iacuc_no, protocol_id, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(req.doc_type)
        .bind(&doc_no)
        .bind(DocStatus::Draft)
        .bind(req.warehouse_id)
        .bind(req.warehouse_from_id)
        .bind(req.warehouse_to_id)
        .bind(req.partner_id)
        .bind(req.source_doc_id)
        .bind(req.doc_date)
        .bind(&req.remark)
        .bind(req.stocktake_scope.as_ref().map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null)))
        .bind(&req.iacuc_no)
        .bind(req.protocol_id)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // SO 多倉銷貨：每行倉庫從其儲位反推回填（migration 136）。其他單據 warehouse_id 留 NULL，
        // 沿用表頭倉。SO requires_shelf()==true，每行必有儲位，故映射齊全。
        let line_warehouses = if req.doc_type == DocType::SO {
            Self::load_shelf_warehouses(&mut tx, &create_loc_ids).await?
        } else {
            std::collections::HashMap::new()
        };

        // 建立單據明細
        let mut lines = Vec::new();
        for (idx, line) in lines_to_create.iter().enumerate() {
            let line_warehouse_id = line
                .storage_location_id
                .and_then(|loc| line_warehouses.get(&loc).copied());
            let doc_line = sqlx::query_as::<_, DocumentLine>(
                r#"
                INSERT INTO document_lines (
                    id, document_id, line_no, product_id, qty, uom, unit_price,
                    batch_no, expiry_date, remark, storage_location_id, warehouse_id,
                    storage_location_from_id, storage_location_to_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                RETURNING *
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(document.id)
            .bind((idx + 1) as i32)
            .bind(line.product_id)
            .bind(line.qty)
            .bind(&line.uom)
            .bind(line.unit_price)
            .bind(&line.batch_no)
            .bind(line.expiry_date)
            .bind(&line.remark)
            .bind(line.storage_location_id)
            .bind(line_warehouse_id)
            .bind(line.storage_location_from_id)
            .bind(line.storage_location_to_id)
            .fetch_one(&mut *tx)
            .await?;
            lines.push(doc_line);
        }

        // R26-12：audit snapshot 包含 document + lines 同一事件內
        let display = format!("{}: {}", document.doc_type.prefix(), document.doc_no);
        let snapshot = DocumentAuditSnapshot {
            document: &document,
            lines: &lines,
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "DOC_CREATE",
                entity: Some(AuditEntity::new("document", document.id, &display)),
                data_diff: Some(DataDiff::create_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Self::get_by_id(pool, document.id).await
    }

    /// 更新單據（僅 Draft 狀態）— Service-driven audit
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateDocumentRequest,
    ) -> Result<DocumentWithLines> {
        let user = actor.require_user()?;

        let mut tx = pool.begin().await?;

        // R26-12 / Gemini pattern：tx 內 FOR UPDATE 鎖住 document row
        let existing =
            sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if existing.status != DocStatus::Draft {
            return Err(AppError::BusinessRule(
                "Only draft documents can be updated".to_string(),
            ));
        }

        // #61：SO 更新亦須通過「該計畫 SD / 研究主持人 / admin」授權，防止透過 update 把
        // protocol_id 改指到未授權計畫（IDOR 繞過）。最終有效計畫＝新值優先、否則沿用既有；
        // **無計畫單（外部客戶，兩值皆 None）同樣過閘**——傳 None 走「任一已核准計畫 SD /
        // 全域 SD / admin」判斷，不可無條件跳過（否則放寬無計畫開單後，任何具編輯權限者
        // 都能改無計畫銷貨草稿的品項/數量/單價）。
        if existing.doc_type == DocType::SO {
            Self::authorize_sales_document(pool, user, req.protocol_id.or(existing.protocol_id))
                .await?;
        }

        // before snapshot (document + lines) for audit diff
        let before_lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;

        // 驗證 protocol_id 對應的計畫存在且為已核准狀態
        if let Some(protocol_id) = req.protocol_id {
            let status: ProtocolStatus =
                sqlx::query_scalar("SELECT status FROM protocols WHERE id = $1")
                    .bind(protocol_id)
                    .fetch_optional(&mut *tx)
                    .await?
                    .ok_or_else(|| {
                        AppError::NotFound(format!("Protocol {} not found", protocol_id))
                    })?;

            if status != ProtocolStatus::Approved
                && status != ProtocolStatus::ApprovedWithConditions
            {
                return Err(AppError::Validation(format!(
                    "Protocol {} is not in approved status (current: {})",
                    protocol_id,
                    status.display_name()
                )));
            }
        }

        // 在執行任何 DB 寫入前，先對 req.lines 做完整預檢（shelf 必填）。
        // 之前位於 line insert 迴圈內 → 即使 lines 違規，UPDATE documents 已先執行（浪費 write
        // 並讓 audit before-snapshot 與 after-snapshot 之間出現失敗 tx）。預檢提早可短路。
        if let Some(ref lines) = req.lines {
            if existing.doc_type.requires_shelf() {
                for (idx, line) in lines.iter().enumerate() {
                    if line.storage_location_id.is_none() {
                        return Err(AppError::ValidationWithCode {
                            code: ERR_DOC_LINE_SHELF_REQUIRED.to_string(),
                            message: format!("第 {} 行：儲位/貨架為必填項", idx + 1),
                        });
                    }
                }
            }
            Self::validate_line_qty_price(existing.doc_type, lines)?;
            Self::assert_lines_uom_defined(&mut tx, lines).await?;
        }

        // 跨倉錯配防護（同 create）：即使本次不更新明細，只要倉庫可能變更，也要用「最終有效
        // 明細」驗證。否則只改 warehouse_id、不帶 lines 的部分更新會繞過此檢查（CodeRabbit
        // critical）：倉庫換了、既有明細仍掛在舊倉庫的儲位 → 重現本 PR 要修的跨倉錯配。
        // req.lines 提供 → 驗新明細儲位；否則沿用既有明細（before_lines）的儲位。
        let eff_wh = req.warehouse_id.or(existing.warehouse_id);
        let eff_loc_ids: Vec<Uuid> = match req.lines {
            Some(ref lines) => lines.iter().filter_map(|l| l.storage_location_id).collect(),
            None => before_lines
                .iter()
                .filter_map(|l| l.storage_location_id)
                .collect(),
        };
        Self::assert_lines_shelf_in_warehouse(&mut tx, existing.doc_type, eff_wh, &eff_loc_ids)
            .await?;

        // 更新單據頭
        sqlx::query(
            r#"
            UPDATE documents SET
                warehouse_id = COALESCE($1, warehouse_id),
                warehouse_from_id = COALESCE($2, warehouse_from_id),
                warehouse_to_id = COALESCE($3, warehouse_to_id),
                partner_id = COALESCE($4, partner_id),
                source_doc_id = COALESCE($5, source_doc_id),
                doc_date = COALESCE($6, doc_date),
                remark = COALESCE($7, remark),
                protocol_id = COALESCE($8, protocol_id),
                updated_at = NOW()
            WHERE id = $9
            "#,
        )
        .bind(req.warehouse_id)
        .bind(req.warehouse_from_id)
        .bind(req.warehouse_to_id)
        .bind(req.partner_id)
        .bind(req.source_doc_id)
        .bind(req.doc_date)
        .bind(&req.remark)
        .bind(req.protocol_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // 如果要更新明細
        if let Some(ref lines) = req.lines {
            // 刪除現有明細
            sqlx::query("DELETE FROM document_lines WHERE document_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;

            // 建立新明細
            let lines: &Vec<DocumentLineInput> = lines;
            // N+1 修復：批號/效期需檢查時，一次撈本單所有品項追蹤旗標供迴圈查表。
            let track_flags = if existing.doc_type.requires_batch_expiry() {
                let product_ids: Vec<Uuid> = lines.iter().map(|l| l.product_id).collect();
                Self::load_product_track_flags(&mut tx, &product_ids).await?
            } else {
                std::collections::HashMap::new()
            };
            // SO 多倉銷貨：每行倉庫從其儲位反推回填（migration 136）；其他單據留 NULL 沿用表頭倉。
            let line_warehouses = if existing.doc_type == DocType::SO {
                let loc_ids: Vec<Uuid> =
                    lines.iter().filter_map(|l| l.storage_location_id).collect();
                Self::load_shelf_warehouses(&mut tx, &loc_ids).await?
            } else {
                std::collections::HashMap::new()
            };
            for (idx, line) in lines.iter().enumerate() {
                // 驗證必填欄位
                if line.uom.is_empty() {
                    return Err(AppError::Validation(format!(
                        "Line {}: UOM is required",
                        idx + 1
                    )));
                }

                // 強制檢查批號與效期 (特定單據類型結合品項設定)
                if existing.doc_type.requires_batch_expiry() {
                    if let Some(&(track_batch, track_expiry)) = track_flags.get(&line.product_id) {
                        if track_batch && line.batch_no.as_ref().is_none_or(|s| s.trim().is_empty())
                        {
                            return Err(AppError::Validation(format!(
                                "Line {}: Batch No is required for {} when product tracks batch",
                                idx + 1,
                                existing.doc_type.prefix()
                            )));
                        }
                        if track_expiry && line.expiry_date.is_none() {
                            return Err(AppError::Validation(format!(
                                "Line {}: Expiry Date is required for {} when product tracks expiry",
                                idx + 1,
                                existing.doc_type.prefix()
                            )));
                        }
                    }
                }

                // shelf 必填驗證已於 UPDATE 前預檢（避免浪費 write）。

                let line_warehouse_id = line
                    .storage_location_id
                    .and_then(|loc| line_warehouses.get(&loc).copied());
                sqlx::query(
                    r#"
                    INSERT INTO document_lines (
                        id, document_id, line_no, product_id, qty, uom, unit_price,
                        batch_no, expiry_date, remark, storage_location_id, warehouse_id,
                        storage_location_from_id, storage_location_to_id
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                    "#,
                )
                .bind(Uuid::new_v4())
                .bind(id)
                .bind((idx + 1) as i32)
                .bind(line.product_id)
                .bind(line.qty)
                .bind(&line.uom)
                .bind(line.unit_price)
                .bind(&line.batch_no)
                .bind(line.expiry_date)
                .bind(&line.remark)
                .bind(line.storage_location_id)
                .bind(line_warehouse_id)
                .bind(line.storage_location_from_id)
                .bind(line.storage_location_to_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        // 讀取 after 狀態用於 audit diff
        let after_doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let after_lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;

        let display = format!("{}: {}", after_doc.doc_type.prefix(), after_doc.doc_no);
        let before_snap = DocumentAuditSnapshot {
            document: &existing,
            lines: &before_lines,
        };
        let after_snap = DocumentAuditSnapshot {
            document: &after_doc,
            lines: &after_lines,
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "DOC_UPDATE",
                entity: Some(AuditEntity::new("document", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Self::get_by_id(pool, id).await
    }

    /// 刪除單據 — Service-driven audit
    pub async fn delete(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        is_hard: bool,
    ) -> Result<()> {
        let _ = actor.require_user()?;

        let mut tx = pool.begin().await?;

        let document =
            sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        // 僅當不是硬刪除時，才要求單據必須為草稿狀態
        if !is_hard && document.status != DocStatus::Draft {
            return Err(AppError::BusinessRule(
                "Only draft documents can be deleted".to_string(),
            ));
        }

        // 連動處理整條下游鏈（規則＝使用者 2026-07-18 決策）：草稿後代 → 連動刪除
        // （各自寫 audit）；有任何非草稿後代 → 擋下（已影響庫存/帳，不可默默移除）。
        let descendants = Self::collect_descendants_tx(&mut tx, id).await?;

        let blocking: Vec<String> = descendants
            .iter()
            .filter(|r| r.status != DocStatus::Draft)
            .map(|r| r.doc_no.clone())
            .collect();
        if !blocking.is_empty() {
            return Err(AppError::BusinessRule(format!(
                "此單據被 {} 張非草稿單據引用（{}），無法刪除；請先撤銷或處理那些單據",
                blocking.len(),
                blocking.join("、")
            )));
        }

        // 後代皆草稿：由深到淺刪除（BFS 反序保證子單先於父單），最後刪主單。
        for child in descendants.iter().rev() {
            Self::delete_document_row_tx(&mut tx, actor, child, "DOC_DELETE").await?;
        }

        // 主單本身（含明細 + audit）
        let event_type = if is_hard {
            "DOC_HARD_DELETE"
        } else {
            "DOC_DELETE"
        };
        Self::delete_document_row_tx(&mut tx, actor, &document, event_type).await?;

        tx.commit().await?;

        Ok(())
    }

    /// BFS 收集所有以 source_doc_id（遞移）引用 root 的後代，逐筆 FOR UPDATE 上鎖。
    /// FK documents_source_doc_id_fkey 為 NO ACTION，只處理直接子單無法涵蓋
    /// A→草稿B→C 這種孫代（刪 B 仍會撞 C 的 FK）。回傳依 BFS 淺→深順序。
    async fn collect_descendants_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        root_id: Uuid,
    ) -> Result<Vec<Document>> {
        let mut descendants: Vec<Document> = Vec::new();
        let mut seen: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        seen.insert(root_id);
        let mut frontier: Vec<Uuid> = vec![root_id];
        while !frontier.is_empty() {
            let children = sqlx::query_as::<_, Document>(
                "SELECT * FROM documents WHERE source_doc_id = ANY($1) \
                 ORDER BY created_at FOR UPDATE",
            )
            .bind(&frontier)
            .fetch_all(&mut **tx)
            .await?;
            frontier = Vec::new();
            for child in children {
                if seen.insert(child.id) {
                    frontier.push(child.id);
                    descendants.push(child);
                }
            }
        }
        Ok(descendants)
    }

    /// 在既有交易內刪除單一單據（含明細）並寫入 DELETE-only audit snapshot。
    /// audit 必須在 DELETE 之前寫（其 entity FK 指向 document row）；同一 tx 一起 commit。
    async fn delete_document_row_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        document: &Document,
        event_type: &str,
    ) -> Result<()> {
        // 已寫入庫存異動或會計傳票的單據不可刪：硬刪會孤兒化 stock_ledger /
        // journal_entries，破壞庫存與會計帳（例如已核准並入庫的 GRN）。
        // 草稿永遠不會有這些效果，故連動刪的草稿後代必然通過此檢查。
        let has_posted_effects: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM stock_ledger WHERE doc_id = $1) \
             OR EXISTS(SELECT 1 FROM journal_entries \
                       WHERE source_entity_type = 'document' AND source_entity_id = $1)",
        )
        .bind(document.id)
        .fetch_one(&mut **tx)
        .await?;
        if has_posted_effects {
            return Err(AppError::BusinessRule(format!(
                "單據 {} 已寫入庫存異動或會計傳票，刪除將破壞庫存/帳務，不可刪除",
                document.doc_no
            )));
        }

        let before_lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(document.id)
        .fetch_all(&mut **tx)
        .await?;

        let display = format!("{}: {}", document.doc_type.prefix(), document.doc_no);
        let before_snap = DocumentAuditSnapshot {
            document,
            lines: &before_lines,
        };
        AuditService::log_activity_tx(
            &mut *tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type,
                entity: Some(AuditEntity::new("document", document.id, &display)),
                data_diff: Some(DataDiff::delete_only(&before_snap)),
                request_context: None,
            },
        )
        .await?;

        sqlx::query("DELETE FROM document_lines WHERE document_id = $1")
            .bind(document.id)
            .execute(&mut **tx)
            .await?;
        sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(document.id)
            .execute(&mut **tx)
            .await?;

        // 硬刪為終態：row 消失後，任何指向它的置頂待辦（採購單未入庫提醒，
        // 見 services/notification/erp.rs::notify_po_pending_receipt）就再也沒有
        // 業務路徑能解除 —— 未核收的已核准 PO 是可以走到這裡的。同 tx 一併解除。
        crate::services::NotificationService::resolve_pinned_notifications_tx(
            tx,
            "document",
            document.id,
        )
        .await?;

        Ok(())
    }

    /// 取得單據列表
    ///
    /// `created_by_scope`：`Some(user_id)` 時只回該使用者建立的單據（非 WM/admin 角色收斂，
    /// 資安稽核 M-4）；`None` 表示不限建立者（WM/admin 全域監督）。
    pub async fn list(
        pool: &PgPool,
        query: &DocumentQuery,
        created_by_scope: Option<Uuid>,
    ) -> Result<Vec<DocumentListItem>> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT
                d.id, d.doc_type, d.doc_no, d.status,
                w.name as warehouse_name,
                d.partner_id,
                p.name as partner_name,
                d.protocol_id,
                pr.protocol_no as protocol_no,
                d.doc_date,
                u1.display_name as created_by_name,
                u2.display_name as approved_by_name,
                d.created_at, d.approved_at,
                COUNT(dl.id) as line_count,
                -- 僅加總實際填寫單價的行；未填單價不回填歷史成本估值，
                -- 全部未填則 SUM 回傳 NULL → 前端顯示「-」（與明細頁一致）。
                SUM(dl.qty * dl.unit_price) as total_amount,
                d.iacuc_no,
                d.receipt_status,
                EXISTS (
                    SELECT 1 FROM journal_entries je
                    WHERE je.source_entity_type = 'document' AND je.source_entity_id = d.id
                ) AS has_journal_entry
            FROM documents d
            LEFT JOIN warehouses w ON d.warehouse_id = w.id
            LEFT JOIN partners p ON d.partner_id = p.id
            LEFT JOIN protocols pr ON d.protocol_id = pr.id
            LEFT JOIN users u1 ON d.created_by = u1.id
            LEFT JOIN users u2 ON d.approved_by = u2.id
            LEFT JOIN document_lines dl ON d.id = dl.document_id
            WHERE 1=1
            "#,
        );

        // 資安稽核 M-4：非 WM/admin 只看自己建立的單據。
        if let Some(creator_id) = created_by_scope {
            qb.push(" AND d.created_by = ");
            qb.push_bind(creator_id);
        }

        if let Some(doc_type) = query.doc_type {
            qb.push(" AND d.doc_type = ");
            qb.push_bind(doc_type);
        } else if let Some(ref doc_types_str) = query.doc_types {
            let parsed: Vec<DocType> = doc_types_str
                .split(',')
                .filter_map(|s| serde_json::from_str::<DocType>(&format!("\"{}\"", s.trim())).ok())
                .collect();
            if !parsed.is_empty() {
                qb.push(" AND d.doc_type IN (");
                let mut sep = qb.separated(", ");
                for t in parsed {
                    sep.push_bind(t);
                }
                qb.push(")");
            }
        }

        if let Some(status) = query.status {
            qb.push(" AND d.status = ");
            qb.push_bind(status);
        }

        if let Some(ref iacuc_no) = query.iacuc_no {
            qb.push(" AND d.iacuc_no = ");
            qb.push_bind(iacuc_no.clone());
        }

        // 產品詳情頁「相關單據」：只列含此產品明細的單據。用 EXISTS 而非 JOIN dl，
        // 避免影響 COUNT(dl.id) / SUM(dl.qty*price) 聚合（那是整張單的行數/金額）。
        if let Some(product_id) = query.product_id {
            qb.push(" AND EXISTS (SELECT 1 FROM document_lines dlp WHERE dlp.document_id = d.id AND dlp.product_id = ");
            qb.push_bind(product_id);
            qb.push(")");
        }

        qb.push(" GROUP BY d.id, w.name, d.partner_id, p.name, d.protocol_id, pr.protocol_no, u1.display_name, u2.display_name, d.doc_type, d.doc_no, d.status, d.doc_date, d.created_at, d.approved_at, d.iacuc_no, d.receipt_status ORDER BY d.created_at DESC");

        let mut documents = qb
            .build_query_as::<DocumentListItem>()
            .fetch_all(pool)
            .await?;

        // 「卡在誰」整頁批次補算：只送待審中的 id 進去，且每個關卡的候選名單在
        // 解析器內只查一次（見 pending_owner::CandidateCache），不做逐列 roundtrip。
        let pending_ids: Vec<Uuid> = documents
            .iter()
            .filter(|d| d.status == DocStatus::Submitted)
            .map(|d| d.id)
            .collect();
        if !pending_ids.is_empty() {
            let mut owners =
                crate::services::pending_owner::resolve_for_documents(pool, &pending_ids).await?;
            for doc in &mut documents {
                doc.pending_owner = owners.remove(&doc.id);
            }
        }

        Ok(documents)
    }

    /// 取得單一單據
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<DocumentWithLines> {
        let document = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        let lines = sqlx::query_as::<_, DocumentLineWithProduct>(
            r#"
            SELECT
                dl.id, dl.document_id, dl.line_no, dl.product_id,
                p.sku as product_sku, p.name as product_name,
                p.base_uom as product_base_uom,
                -- factor_to_base > 0 與 StockService::load_uom_tables 的過濾一致：
                -- 非正數 factor 的單位在後端形同未定義、必被擋成 400，列進下拉只會
                -- 給出一個選了必然失敗的選項（同 repositories/product.rs 的 alt_uoms）。
                ARRAY(
                    SELECT c.uom FROM product_uom_conversions c
                    WHERE c.product_id = p.id AND c.factor_to_base > 0
                    ORDER BY c.factor_to_base
                )::text[] AS product_alt_uoms,
                dl.qty, dl.uom, dl.unit_price, dl.batch_no, dl.expiry_date, dl.remark,
                dl.storage_location_id, dl.warehouse_id,
                dl.storage_location_from_id, dl.storage_location_to_id
            FROM document_lines dl
            INNER JOIN products p ON dl.product_id = p.id
            WHERE dl.document_id = $1
            ORDER BY dl.line_no
            "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        let names = Self::load_document_related_names(pool, &document).await?;
        let reversal = Self::load_reversal_links(pool, id, document.reverses_doc_id).await?;
        // 待審中才需要算「卡在誰」；已核准 / 草稿 / 作廢的單解析器一律回空。
        let pending_owner = crate::services::pending_owner::resolve_for_documents(pool, &[id])
            .await?
            .remove(&id);

        Ok(DocumentWithLines {
            document,
            lines,
            warehouse_name: names.warehouse_name,
            warehouse_from_name: names.warehouse_from_name,
            warehouse_to_name: names.warehouse_to_name,
            partner_name: names.partner_name,
            protocol_no: names.protocol_no,
            created_by_name: names.created_by_name,
            approved_by_name: names.approved_by_name,
            reverses_doc_no: reversal.reverses_doc_no,
            reversed_by_doc_id: reversal.reversed_by_doc_id,
            reversed_by_doc_no: reversal.reversed_by_doc_no,
            reversed_at: reversal.reversed_at,
            pending_owner,
        })
    }

    /// 單據詳情的關聯顯示名稱（倉庫／夥伴／計畫／人員）。
    /// 自 `get_by_id` 抽出——該函式加入沖銷關聯後達 107 行，超過 50 行門檻。
    async fn load_document_related_names(
        pool: &PgPool,
        document: &Document,
    ) -> Result<DocumentRelatedNames> {
        let name_of = |id: Option<Uuid>| async move {
            match id {
                Some(wid) => repositories::warehouse::find_warehouse_name_by_id(pool, wid).await,
                None => Ok(None),
            }
        };

        let partner_name: Option<String> = if let Some(pid) = document.partner_id {
            sqlx::query_scalar("SELECT name FROM partners WHERE id = $1")
                .bind(pid)
                .fetch_optional(pool)
                .await?
        } else {
            None
        };

        let protocol_no: Option<String> = if let Some(pid) = document.protocol_id {
            sqlx::query_scalar("SELECT protocol_no FROM protocols WHERE id = $1")
                .bind(pid)
                .fetch_optional(pool)
                .await?
        } else {
            None
        };

        Ok(DocumentRelatedNames {
            warehouse_name: name_of(document.warehouse_id).await?,
            warehouse_from_name: name_of(document.warehouse_from_id).await?,
            warehouse_to_name: name_of(document.warehouse_to_id).await?,
            partner_name,
            protocol_no,
            created_by_name: repositories::user::find_user_display_name_by_id(
                pool,
                document.created_by,
            )
            .await?
            .unwrap_or_else(|| "Unknown User".to_string()),
            approved_by_name: match document.approved_by {
                Some(uid) => repositories::user::find_user_display_name_by_id(pool, uid).await?,
                None => None,
            },
        })
    }

    /// R84-5 沖銷關聯（雙向）：
    /// - 本單是沖銷單 → 查被沖銷的原單單號
    /// - 本單被沖銷   → 反向查沖銷它的那張單（原單身上沒有這個欄位，只能反查）
    async fn load_reversal_links(
        pool: &PgPool,
        id: Uuid,
        reverses_doc_id: Option<Uuid>,
    ) -> Result<ReversalLinks> {
        let reverses_doc_no: Option<String> = match reverses_doc_id {
            Some(orig_id) => {
                sqlx::query_scalar("SELECT doc_no FROM documents WHERE id = $1")
                    .bind(orig_id)
                    .fetch_optional(pool)
                    .await?
            }
            None => None,
        };

        let reversed_by: Option<(Uuid, String, Option<chrono::DateTime<chrono::Utc>>)> =
            sqlx::query_as(
                "SELECT id, doc_no, approved_at FROM documents WHERE reverses_doc_id = $1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await?;

        let (reversed_by_doc_id, reversed_by_doc_no, reversed_at) = match reversed_by {
            Some((rid, rno, rat)) => (Some(rid), Some(rno), rat),
            None => (None, None, None),
        };

        Ok(ReversalLinks {
            reverses_doc_no,
            reversed_by_doc_id,
            reversed_by_doc_no,
            reversed_at,
        })
    }

    /// 產生單據編號
    /// 格式：{PREFIX}-YYMMDD-{02} (例如：SO-260115-01)
    ///
    /// 取號是 read-max-then-insert，必須在 advisory lock 內做：PostgreSQL 預設
    /// READ COMMITTED 下純 SELECT 不鎖任何列，無鎖時兩個並發交易會讀到同一個最大
    /// 序號、算出同一個新號、雙雙 INSERT，其中一個撞上唯一索引 `documents_doc_no_key`
    /// （回歸測試見 `tests/erp_doc_no_concurrency.rs`）。
    pub(crate) async fn generate_doc_no(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        doc_type: DocType,
    ) -> Result<String> {
        acquire_doc_no_lock(tx).await?;

        // 統一使用台灣日期 YYMMDD 格式
        let today = time::now_taiwan();
        let year = today.format("%y").to_string(); // 2-digit year
        let month_day = today.format("%m%d").to_string();
        let date_str = format!("{}{}", year, month_day);

        // 建立前綴：{PREFIX}-YYMMDD
        let prefix = format!("{}-{}", doc_type.prefix(), date_str);

        // 取當天所有已用編號後在 Rust 端取 max。
        // 不可用 `ORDER BY doc_no DESC LIMIT 1`——那是**字典序**，到第 100 張時
        // `"…-99"` 會排在 `"…-100"` 之前，於是重新算出 100 並撞號。
        let existing: Vec<String> =
            sqlx::query_scalar("SELECT doc_no FROM documents WHERE doc_no LIKE $1")
                .bind(format!("{}-%", prefix))
                .fetch_all(&mut **tx)
                .await?;

        let max_seq = existing
            .iter()
            .filter_map(|no| parse_doc_no_sequence(no, &prefix))
            .max()
            .unwrap_or(0);

        // `{:02}` 是最小寬度而非上限：超過 99 自然進位成 3 位數（SO-260115-100）。
        Ok(format!("{}-{:02}", prefix, max_seq + 1))
    }
}

/// 於 transaction 內取得單據取號鎖。
/// 並發執行時後到的 tx 會在此阻塞，直到先到的 tx commit/rollback
/// （`pg_advisory_xact_lock` 於交易結束時自動釋放）。
///
/// ⚠️ **鎖必須持有到交易 commit，不可「取完號就放」**。用 `pg_advisory_xact_lock`
/// （交易級）而非 `pg_advisory_unlock`（手動釋放）正是為此：READ COMMITTED 下，
/// 若在 INSERT 尚未 commit 前就放鎖，後到的交易讀不到那筆未 commit 的資料，
/// 會算出同一個號碼——鎖等於完全失效。任何「縮短持鎖時間」的優化都會靜默地
/// 讓 `tests/erp_doc_no_concurrency.rs` 所防的缺陷復活。
///
/// 已知取捨：因此本鎖會序列化**所有單據類型**的建立（含明細寫入與 audit log 的
/// 時間），且期間各自佔住一個連線。目前規模（prod 實測單日單一 doc_type 峰值 12 張）
/// 影響可忽略。若日後單量成長或出現批次匯入，應改為與主交易解耦的專用序號表
/// （快速 commit 取號、不隨主交易持鎖），而不是縮短這裡的持鎖範圍。
///
/// R28-M3：lock key 集中於 `crate::constants::DOCUMENT_NUMBERING_LOCK_KEY`。
async fn acquire_doc_no_lock(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
        .bind(crate::constants::DOCUMENT_NUMBERING_LOCK_KEY)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// 從單據編號解析流水號。
/// 例如：`parse_doc_no_sequence("SO-260115-07", "SO-260115")` → `Some(7)`
///
/// 前綴不符、或尾段非純數字者回 `None`，由呼叫端略過——例如 migration 070 補帳
/// 產生的 `ADJ-BASELINE-<倉名>` 這類不帶日期流水號的歷史編號。
fn parse_doc_no_sequence(doc_no: &str, prefix: &str) -> Option<i32> {
    doc_no
        .strip_prefix(prefix)?
        .strip_prefix('-')?
        .parse::<i32>()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(qty: Decimal, unit_price: Option<Decimal>) -> DocumentLineInput {
        DocumentLineInput {
            product_id: Uuid::nil(),
            qty,
            uom: "EA".to_string(),
            unit_price,
            batch_no: None,
            expiry_date: None,
            remark: None,
            storage_location_id: None,
            storage_location_from_id: None,
            storage_location_to_id: None,
        }
    }

    /// 取號的序號解析：三位數要能解出來（字典序 bug 的根源），
    /// 非本前綴與非數字尾段要被略過而非算成 0。
    #[test]
    fn parses_doc_no_sequence() {
        assert_eq!(parse_doc_no_sequence("SO-260115-07", "SO-260115"), Some(7));
        // 跨過 99 之後仍須解得出來——舊實作用字典序取「最後一筆」正是死在這
        assert_eq!(
            parse_doc_no_sequence("SO-260115-100", "SO-260115"),
            Some(100)
        );
        // 別的日期 / 別的單別 → 不屬於本前綴
        assert_eq!(parse_doc_no_sequence("SO-260116-01", "SO-260115"), None);
        assert_eq!(parse_doc_no_sequence("GRN-260115-01", "SO-260115"), None);
        // 前綴相符但少了分隔線（避免 "SO-2601151" 被誤認）
        assert_eq!(parse_doc_no_sequence("SO-26011501", "SO-260115"), None);
        // migration 070 補帳的歷史編號：尾段非數字 → 略過，不得算成 0
        assert_eq!(
            parse_doc_no_sequence("ADJ-BASELINE-A倉", "ADJ-260115"),
            None
        );
    }

    #[test]
    fn movement_doc_requires_positive_qty() {
        // GRN 需帶有效單價以隔離 qty 驗證（GRN 單價必填且 > 0）
        let pos = [line(Decimal::from(5), Some(Decimal::ONE))];
        assert!(DocumentService::validate_line_qty_price(DocType::GRN, &pos).is_ok());
        let zero = [line(Decimal::ZERO, Some(Decimal::ONE))];
        assert!(DocumentService::validate_line_qty_price(DocType::GRN, &zero).is_err());
        let neg = [line(Decimal::from(-1), None)];
        assert!(DocumentService::validate_line_qty_price(DocType::PR, &neg).is_err());
    }

    /// GRN 單價必填且 > 0 — create 與 update 共用 validate_line_qty_price，
    /// 故本測試同時鎖定兩條路徑的規則一致性（修補 create/update 分叉 bug）。
    #[test]
    fn grn_requires_present_positive_unit_price() {
        let missing = [line(Decimal::from(5), None)];
        assert!(DocumentService::validate_line_qty_price(DocType::GRN, &missing).is_err());
        let zero_price = [line(Decimal::from(5), Some(Decimal::ZERO))];
        assert!(DocumentService::validate_line_qty_price(DocType::GRN, &zero_price).is_err());
        let valid = [line(Decimal::from(5), Some(Decimal::ONE))];
        assert!(DocumentService::validate_line_qty_price(DocType::GRN, &valid).is_ok());
    }

    #[test]
    fn adjustment_allows_signed_qty_but_not_zero() {
        let neg = [line(Decimal::from(-3), None)];
        assert!(DocumentService::validate_line_qty_price(DocType::ADJ, &neg).is_ok());
        let pos = [line(Decimal::from(3), None)];
        assert!(DocumentService::validate_line_qty_price(DocType::ADJ, &pos).is_ok());
        let zero = [line(Decimal::ZERO, None)];
        assert!(DocumentService::validate_line_qty_price(DocType::ADJ, &zero).is_err());
    }

    /// 盤點的實盤數恆 >= 0：
    /// · 0 合法且必要——「系統有、現場沒有」＝全數短少，是盤點最該登記的結果之一；
    /// · 負數在盤點語意下不存在（2026-09-14 使用者裁定）。本檔原本對 STK 直接
    ///   `continue` 跳過整段數量檢查，連 -5 的實盤數都會收下。
    #[test]
    fn stocktake_allows_zero_but_rejects_negative_qty() {
        let zero = [line(Decimal::ZERO, None)];
        assert!(DocumentService::validate_line_qty_price(DocType::STK, &zero).is_ok());
        let pos = [line(Decimal::from(7), None)];
        assert!(DocumentService::validate_line_qty_price(DocType::STK, &pos).is_ok());
        let neg = [line(Decimal::from(-1), None)];
        assert!(DocumentService::validate_line_qty_price(DocType::STK, &neg).is_err());
    }

    #[test]
    fn negative_unit_price_rejected_for_any_doc_type() {
        let bad = [line(Decimal::from(1), Some(Decimal::from(-1)))];
        assert!(DocumentService::validate_line_qty_price(DocType::GRN, &bad).is_err());
        assert!(DocumentService::validate_line_qty_price(DocType::STK, &bad).is_err());
    }
}
