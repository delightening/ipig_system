use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, CreateDocumentRequest, DocStatus, DocType, Document,
        DocumentAuditSnapshot, DocumentLine, DocumentLineInput, DocumentLineWithProduct,
        DocumentListItem, DocumentQuery, DocumentWithLines, ProtocolStatus, UpdateDocumentRequest,
    },
    repositories,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    time, AppError, Result,
};

use super::DocumentService;

impl DocumentService {
    /// 建立單據（草稿）— Service-driven audit
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateDocumentRequest,
    ) -> Result<DocumentWithLines> {
        let user = actor.require_user()?;
        let created_by = user.id;

        let mut tx = pool.begin().await?;

        // 產生單據編號
        let doc_no = Self::generate_doc_no(&mut tx, req.doc_type).await?;

        // 如果是盤點單，根據範圍自動生成盤點項目
        let lines_to_create = if req.doc_type == DocType::STK {
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

            // GRN 單據的單價必填且不可為 0
            if req.doc_type == DocType::GRN {
                for (idx, line) in req.lines.iter().enumerate() {
                    let invalid = match line.unit_price {
                        None => true,
                        Some(d) if d <= Decimal::ZERO => true,
                        _ => false,
                    };
                    if invalid {
                        return Err(AppError::Validation(format!(
                            "第 {} 行：採購入庫的單價為必填，且必須大於 0",
                            idx + 1
                        )));
                    }
                }
            }

            // 強制檢查批號與效期 (特定單據類型結合品項設定)
            if req.doc_type.requires_batch_expiry() {
                for (idx, line) in req.lines.iter().enumerate() {
                    // 查詢品項設定
                    let product: Option<(bool, bool)> = sqlx::query_as(
                        "SELECT track_batch, track_expiry FROM products WHERE id = $1"
                    )
                    .bind(line.product_id)
                    .fetch_optional(&mut *tx)
                    .await?;

                    if let Some((track_batch, track_expiry)) = product {
                        if track_batch && line.batch_no.as_ref().is_none_or(|s| s.trim().is_empty()) {
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

        // 驗證 protocol_id 對應的計畫存在且為已核准狀態
        if let Some(protocol_id) = req.protocol_id {
            let status: ProtocolStatus = sqlx::query_scalar(
                "SELECT status FROM protocols WHERE id = $1",
            )
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

        // 建立單據明細
        let mut lines = Vec::new();
        for (idx, line) in lines_to_create.iter().enumerate() {
            let doc_line = sqlx::query_as::<_, DocumentLine>(
                r#"
                INSERT INTO document_lines (
                    id, document_id, line_no, product_id, qty, uom, unit_price,
                    batch_no, expiry_date, remark, storage_location_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
        let _ = actor.require_user()?;

        let mut tx = pool.begin().await?;

        // R26-12 / Gemini pattern：tx 內 FOR UPDATE 鎖住 document row
        let existing = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if existing.status != DocStatus::Draft {
            return Err(AppError::BusinessRule(
                "Only draft documents can be updated".to_string(),
            ));
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
            let status: ProtocolStatus = sqlx::query_scalar(
                "SELECT status FROM protocols WHERE id = $1",
            )
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
                    // 查詢品項設定
                    let product: Option<(bool, bool)> = sqlx::query_as(
                        "SELECT track_batch, track_expiry FROM products WHERE id = $1"
                    )
                    .bind(line.product_id)
                    .fetch_optional(&mut *tx)
                    .await?;

                    if let Some((track_batch, track_expiry)) = product {
                        if track_batch && line.batch_no.as_ref().is_none_or(|s| s.trim().is_empty()) {
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

                sqlx::query(
                    r#"
                    INSERT INTO document_lines (
                        id, document_id, line_no, product_id, qty, uom, unit_price,
                        batch_no, expiry_date, remark, storage_location_id
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
    pub async fn delete(pool: &PgPool, actor: &ActorContext, id: Uuid, is_hard: bool) -> Result<()> {
        let _ = actor.require_user()?;

        let mut tx = pool.begin().await?;

        let document = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1 FOR UPDATE",
        )
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

        // before snapshot 含明細（audit）
        let before_lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;

        // 寫入 audit 後才真正刪除（audit 的 FK 指向 document row — 在 DELETE
        // 之前寫入才不會失效；整段 tx 結束一起 commit）
        let display = format!("{}: {}", document.doc_type.prefix(), document.doc_no);
        let before_snap = DocumentAuditSnapshot {
            document: &document,
            lines: &before_lines,
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: if is_hard { "DOC_HARD_DELETE" } else { "DOC_DELETE" },
                entity: Some(AuditEntity::new("document", id, &display)),
                data_diff: Some(DataDiff::delete_only(&before_snap)),
                request_context: None,
            },
        )
        .await?;

        // 刪除單據明細
        sqlx::query("DELETE FROM document_lines WHERE document_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // 刪除單據
        sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// 取得單據列表
    pub async fn list(pool: &PgPool, query: &DocumentQuery) -> Result<Vec<DocumentListItem>> {
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
                SUM(dl.qty * COALESCE(dl.unit_price,
                    (SELECT AVG(sl.unit_cost) FROM stock_ledger sl
                     WHERE sl.product_id = dl.product_id AND sl.unit_cost IS NOT NULL),
                    0)) as total_amount,
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

        if let Some(doc_type) = query.doc_type {
            qb.push(" AND d.doc_type = ");
            qb.push_bind(doc_type);
        } else if let Some(ref doc_types_str) = query.doc_types {
            let parsed: Vec<DocType> = doc_types_str
                .split(',')
                .filter_map(|s| {
                    serde_json::from_str::<DocType>(&format!("\"{}\"", s.trim())).ok()
                })
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

        qb.push(" GROUP BY d.id, w.name, d.partner_id, p.name, d.protocol_id, pr.protocol_no, u1.display_name, u2.display_name, d.doc_type, d.doc_no, d.status, d.doc_date, d.created_at, d.approved_at, d.iacuc_no, d.receipt_status ORDER BY d.created_at DESC");

        let documents = qb
            .build_query_as::<DocumentListItem>()
            .fetch_all(pool)
            .await?;

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
                dl.qty, dl.uom, dl.unit_price, dl.batch_no, dl.expiry_date, dl.remark,
                dl.storage_location_id
            FROM document_lines dl
            INNER JOIN products p ON dl.product_id = p.id
            WHERE dl.document_id = $1
            ORDER BY dl.line_no
            "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        // 取得關聯名稱
        let warehouse_name = match document.warehouse_id {
            Some(wid) => repositories::warehouse::find_warehouse_name_by_id(pool, wid).await?,
            None => None,
        };
        let warehouse_from_name = match document.warehouse_from_id {
            Some(wid) => repositories::warehouse::find_warehouse_name_by_id(pool, wid).await?,
            None => None,
        };
        let warehouse_to_name = match document.warehouse_to_id {
            Some(wid) => repositories::warehouse::find_warehouse_name_by_id(pool, wid).await?,
            None => None,
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

        let created_by_name: String =
            repositories::user::find_user_display_name_by_id(pool, document.created_by)
                .await?
                .unwrap_or_else(|| "Unknown User".to_string());

        let approved_by_name: Option<String> = match document.approved_by {
            Some(uid) => {
                repositories::user::find_user_display_name_by_id(pool, uid).await?
            }
            None => None,
        };

        Ok(DocumentWithLines {
            document,
            lines,
            warehouse_name,
            warehouse_from_name,
            warehouse_to_name,
            partner_name,
            protocol_no,
            created_by_name,
            approved_by_name,
        })
    }

    /// 產生單據編號
    /// 格式：{PREFIX}-YYMMDD-{02} (例如：SO-260115-01)
    pub(crate) async fn generate_doc_no(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        doc_type: DocType,
    ) -> Result<String> {
        // 統一使用台灣日期 YYMMDD 格式
        let today = time::now_taiwan();
        let year = today.format("%y").to_string(); // 2-digit year
        let month_day = today.format("%m%d").to_string();
        let date_str = format!("{}{}", year, month_day);

        // 建立前綴：{PREFIX}-YYMMDD
        let prefix = format!("{}-{}", doc_type.prefix(), date_str);

        // 取得當天最後一個序號（查詢格式：{PREFIX}-YYMMDD-XX）
        let last_no: Option<String> = sqlx::query_scalar(
            "SELECT doc_no FROM documents WHERE doc_no LIKE $1 ORDER BY doc_no DESC LIMIT 1",
        )
        .bind(format!("{}%", prefix))
        .fetch_optional(&mut **tx)
        .await?;

        let seq = if let Some(last) = last_no {
            // 解析格式：{PREFIX}-YYMMDD-XX
            let parts: Vec<&str> = last.split('-').collect();
            if parts.len() >= 3 {
                parts[2].parse::<i32>().unwrap_or(0) + 1
            } else {
                1
            }
        } else {
            1
        };

        // 統一使用 2 位數序號格式：{PREFIX}-YYMMDD-{02}
        let doc_no = format!("{}-{:02}", prefix, seq);

        Ok(doc_no)
    }
}
