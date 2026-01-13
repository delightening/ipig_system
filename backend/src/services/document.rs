use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        CreateDocumentRequest, DocStatus, DocType, Document, DocumentLine, DocumentLineWithProduct,
        DocumentListItem, DocumentQuery, DocumentWithLines, UpdateDocumentRequest,
        PoReceiptStatus, PoReceiptItem,
    },
    services::StockService,
    AppError, Result,
};

pub struct DocumentService;

impl DocumentService {
    /// 建立單據（草稿）
    pub async fn create(
        pool: &PgPool,
        req: &CreateDocumentRequest,
        created_by: Uuid,
    ) -> Result<DocumentWithLines> {
        let mut tx = pool.begin().await?;

        // 產生單據編號
        let doc_no = Self::generate_doc_no(&mut tx, req.doc_type).await?;

        // 建立單據頭
        let document = sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (
                id, doc_type, doc_no, status, warehouse_id, warehouse_from_id, warehouse_to_id,
                partner_id, doc_date, remark, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&req.doc_type)
        .bind(&doc_no)
        .bind(DocStatus::Draft)
        .bind(req.warehouse_id)
        .bind(req.warehouse_from_id)
        .bind(req.warehouse_to_id)
        .bind(req.partner_id)
        .bind(req.doc_date)
        .bind(&req.remark)
        .bind(created_by)
        .fetch_one(&mut *tx)
        .await?;

        // 建立單據明細
        let mut lines = Vec::new();
        for (idx, line) in req.lines.iter().enumerate() {
            let doc_line = sqlx::query_as::<_, DocumentLine>(
                r#"
                INSERT INTO document_lines (
                    id, document_id, line_no, product_id, qty, uom, unit_price,
                    batch_no, expiry_date, remark
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING *
                "#
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
            .fetch_one(&mut *tx)
            .await?;
            lines.push(doc_line);
        }

        tx.commit().await?;

        Self::get_by_id(pool, document.id).await
    }

    /// 更新單據（僅 Draft 狀態）
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateDocumentRequest,
    ) -> Result<DocumentWithLines> {
        let existing = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if existing.status != DocStatus::Draft {
            return Err(AppError::BusinessRule("Only draft documents can be updated".to_string()));
        }

        let mut tx = pool.begin().await?;

        // 更新單據頭
        sqlx::query(
            r#"
            UPDATE documents SET
                warehouse_id = COALESCE($1, warehouse_id),
                warehouse_from_id = COALESCE($2, warehouse_from_id),
                warehouse_to_id = COALESCE($3, warehouse_to_id),
                partner_id = COALESCE($4, partner_id),
                doc_date = COALESCE($5, doc_date),
                remark = COALESCE($6, remark),
                updated_at = NOW()
            WHERE id = $7
            "#
        )
        .bind(req.warehouse_id)
        .bind(req.warehouse_from_id)
        .bind(req.warehouse_to_id)
        .bind(req.partner_id)
        .bind(req.doc_date)
        .bind(&req.remark)
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
            for (idx, line) in lines.iter().enumerate() {
                sqlx::query(
                    r#"
                    INSERT INTO document_lines (
                        id, document_id, line_no, product_id, qty, uom, unit_price,
                        batch_no, expiry_date, remark
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    "#
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
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Self::get_by_id(pool, id).await
    }

    /// 送審
    pub async fn submit(pool: &PgPool, id: Uuid) -> Result<DocumentWithLines> {
        let result = sqlx::query(
            r#"
            UPDATE documents SET status = $1, updated_at = NOW()
            WHERE id = $2 AND status = $3
            "#
        )
        .bind(DocStatus::Submitted)
        .bind(id)
        .bind(DocStatus::Draft)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::BusinessRule("Document not found or not in draft status".to_string()));
        }

        Self::get_by_id(pool, id).await
    }

    /// 核准（寫入庫存流水）
    /// 採購單核准後會自動產生入庫單（草稿）
    pub async fn approve(pool: &PgPool, id: Uuid, approved_by: Uuid) -> Result<DocumentWithLines> {
        let document = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status != DocStatus::Submitted {
            return Err(AppError::BusinessRule("Document must be in submitted status to approve".to_string()));
        }

        let lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no"
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        let mut tx = pool.begin().await?;

        // 檢查庫存並寫入流水
        if document.doc_type.affects_stock() {
            StockService::process_document(&mut tx, &document, &lines).await?;
        }

        // 更新單據狀態
        sqlx::query(
            r#"
            UPDATE documents SET 
                status = $1, 
                approved_by = $2, 
                approved_at = NOW(), 
                updated_at = NOW()
            WHERE id = $3
            "#
        )
        .bind(DocStatus::Approved)
        .bind(approved_by)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // 如果是採購單，自動產生入庫單（草稿）
        if document.doc_type == DocType::PO {
            Self::create_grn_from_po(&mut tx, &document, &lines, approved_by).await?;
            
            // 更新採購單的入庫狀態
            sqlx::query(
                "UPDATE documents SET receipt_status = 'pending' WHERE id = $1"
            )
            .bind(id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Self::get_by_id(pool, id).await
    }

    /// 從採購單建立入庫單（草稿）
    async fn create_grn_from_po(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        po: &Document,
        po_lines: &[DocumentLine],
        created_by: Uuid,
    ) -> Result<Uuid> {
        // 產生入庫單編號
        let today = Utc::now().format("%Y%m%d").to_string();
        let prefix = format!("GRN-{}", today);
        
        let last_no: Option<String> = sqlx::query_scalar(
            "SELECT doc_no FROM documents WHERE doc_no LIKE $1 ORDER BY doc_no DESC LIMIT 1"
        )
        .bind(format!("{}%", prefix))
        .fetch_optional(&mut **tx)
        .await?;

        let seq = if let Some(last) = last_no {
            let parts: Vec<&str> = last.split('-').collect();
            if parts.len() >= 3 {
                parts[2].parse::<i32>().unwrap_or(0) + 1
            } else {
                1
            }
        } else {
            1
        };
        let doc_no = format!("{}-{:04}", prefix, seq);

        // 建立入庫單頭
        let grn_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO documents (
                id, doc_type, doc_no, status, warehouse_id, partner_id, doc_date,
                source_doc_id, remark, created_by, created_at, updated_at
            )
            VALUES ($1, 'GRN', $2, 'draft', $3, $4, $5, $6, $7, $8, NOW(), NOW())
            "#
        )
        .bind(grn_id)
        .bind(&doc_no)
        .bind(po.warehouse_id)
        .bind(po.partner_id)
        .bind(Utc::now().date_naive())
        .bind(po.id)  // source_doc_id 關聯到採購單
        .bind(format!("自動產生自採購單 {}", po.doc_no))
        .bind(created_by)
        .execute(&mut **tx)
        .await?;

        // 建立入庫單明細（從採購單帶入）
        for (idx, line) in po_lines.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO document_lines (
                    id, document_id, line_no, product_id, qty, uom, unit_price,
                    batch_no, expiry_date, remark
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#
            )
            .bind(Uuid::new_v4())
            .bind(grn_id)
            .bind((idx + 1) as i32)
            .bind(line.product_id)
            .bind(line.qty)  // 預設數量等於採購數量
            .bind(&line.uom)
            .bind(line.unit_price)
            .bind(&line.batch_no)
            .bind(line.expiry_date)
            .bind(&line.remark)
            .execute(&mut **tx)
            .await?;
        }

        Ok(grn_id)
    }

    /// 從採購單建立額外入庫單（部分入庫用）
    pub async fn create_additional_grn(
        pool: &PgPool,
        po_id: Uuid,
        created_by: Uuid,
    ) -> Result<DocumentWithLines> {
        // 檢查採購單狀態
        let po = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1 AND doc_type = 'PO'"
        )
        .bind(po_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".to_string()))?;

        if po.status != DocStatus::Approved {
            return Err(AppError::BusinessRule("Purchase order must be approved".to_string()));
        }

        // 取得採購單明細
        let po_lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no"
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        // 取得已入庫數量
        let received_qty: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT dl.product_id, COALESCE(SUM(dl.qty), 0) as received
            FROM documents d
            JOIN document_lines dl ON d.id = dl.document_id
            WHERE d.source_doc_id = $1 
              AND d.doc_type = 'GRN' 
              AND d.status = 'approved'
            GROUP BY dl.product_id
            "#
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        let received_map: std::collections::HashMap<Uuid, Decimal> = 
            received_qty.into_iter().collect();

        // 計算剩餘數量
        let remaining_lines: Vec<_> = po_lines
            .iter()
            .filter_map(|line| {
                let received = received_map.get(&line.product_id).copied().unwrap_or(Decimal::ZERO);
                let remaining = line.qty - received;
                if remaining > Decimal::ZERO {
                    Some((line.clone(), remaining))
                } else {
                    None
                }
            })
            .collect();

        if remaining_lines.is_empty() {
            return Err(AppError::BusinessRule("All items have been received".to_string()));
        }

        let mut tx = pool.begin().await?;

        // 產生入庫單編號
        let today = Utc::now().format("%Y%m%d").to_string();
        let prefix = format!("GRN-{}", today);
        
        let last_no: Option<String> = sqlx::query_scalar(
            "SELECT doc_no FROM documents WHERE doc_no LIKE $1 ORDER BY doc_no DESC LIMIT 1"
        )
        .bind(format!("{}%", prefix))
        .fetch_optional(&mut *tx)
        .await?;

        let seq = if let Some(last) = last_no {
            let parts: Vec<&str> = last.split('-').collect();
            if parts.len() >= 3 {
                parts[2].parse::<i32>().unwrap_or(0) + 1
            } else {
                1
            }
        } else {
            1
        };
        let doc_no = format!("{}-{:04}", prefix, seq);

        // 建立入庫單
        let grn_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO documents (
                id, doc_type, doc_no, status, warehouse_id, partner_id, doc_date,
                source_doc_id, remark, created_by, created_at, updated_at
            )
            VALUES ($1, 'GRN', $2, 'draft', $3, $4, $5, $6, $7, $8, NOW(), NOW())
            "#
        )
        .bind(grn_id)
        .bind(&doc_no)
        .bind(po.warehouse_id)
        .bind(po.partner_id)
        .bind(Utc::now().date_naive())
        .bind(po.id)
        .bind(format!("追加入庫 - 採購單 {}", po.doc_no))
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        // 建立入庫單明細（只含剩餘數量）
        for (idx, (line, remaining)) in remaining_lines.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO document_lines (
                    id, document_id, line_no, product_id, qty, uom, unit_price,
                    batch_no, expiry_date, remark
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#
            )
            .bind(Uuid::new_v4())
            .bind(grn_id)
            .bind((idx + 1) as i32)
            .bind(line.product_id)
            .bind(*remaining)  // 預設為剩餘數量
            .bind(&line.uom)
            .bind(line.unit_price)
            .bind(&line.batch_no)
            .bind(line.expiry_date)
            .bind(&line.remark)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Self::get_by_id(pool, grn_id).await
    }

    /// 取得採購單入庫狀態
    pub async fn get_po_receipt_status(pool: &PgPool, po_id: Uuid) -> Result<PoReceiptStatus> {
        let po = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1 AND doc_type = 'PO'"
        )
        .bind(po_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".to_string()))?;

        // 取得採購單明細
        let po_lines: Vec<(Uuid, String, Decimal)> = sqlx::query_as(
            r#"
            SELECT dl.product_id, p.name, dl.qty
            FROM document_lines dl
            JOIN products p ON dl.product_id = p.id
            WHERE dl.document_id = $1
            ORDER BY dl.line_no
            "#
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        // 取得已入庫數量
        let received: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT dl.product_id, COALESCE(SUM(dl.qty), 0)
            FROM documents d
            JOIN document_lines dl ON d.id = dl.document_id
            WHERE d.source_doc_id = $1 
              AND d.doc_type = 'GRN' 
              AND d.status = 'approved'
            GROUP BY dl.product_id
            "#
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        let received_map: std::collections::HashMap<Uuid, Decimal> = 
            received.into_iter().collect();

        let items: Vec<PoReceiptItem> = po_lines
            .into_iter()
            .map(|(product_id, product_name, ordered_qty)| {
                let received_qty = received_map.get(&product_id).copied().unwrap_or(Decimal::ZERO);
                PoReceiptItem {
                    product_id,
                    product_name,
                    ordered_qty,
                    received_qty,
                    remaining_qty: ordered_qty - received_qty,
                }
            })
            .collect();

        let total_ordered: Decimal = items.iter().map(|i| i.ordered_qty).sum();
        let total_received: Decimal = items.iter().map(|i| i.received_qty).sum();

        let status = if total_received == Decimal::ZERO {
            "pending".to_string()
        } else if total_received < total_ordered {
            "partial".to_string()
        } else {
            "complete".to_string()
        };

        Ok(PoReceiptStatus {
            po_id,
            po_no: po.doc_no,
            status,
            items,
        })
    }

    /// 作廢
    pub async fn cancel(pool: &PgPool, id: Uuid) -> Result<DocumentWithLines> {
        let document = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status == DocStatus::Approved {
            return Err(AppError::BusinessRule("Cannot cancel approved documents. Use reversal instead.".to_string()));
        }

        sqlx::query(
            "UPDATE documents SET status = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(DocStatus::Cancelled)
        .bind(id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, id).await
    }

    /// 取得單據列表
    pub async fn list(pool: &PgPool, query: &DocumentQuery) -> Result<Vec<DocumentListItem>> {
        let mut sql = String::from(
            r#"
            SELECT 
                d.id, d.doc_type, d.doc_no, d.status,
                w.name as warehouse_name,
                p.name as partner_name,
                d.doc_date,
                u1.display_name as created_by_name,
                u2.display_name as approved_by_name,
                d.created_at, d.approved_at,
                COUNT(dl.id) as line_count,
                SUM(dl.qty * COALESCE(dl.unit_price, 0)) as total_amount
            FROM documents d
            LEFT JOIN warehouses w ON d.warehouse_id = w.id
            LEFT JOIN partners p ON d.partner_id = p.id
            LEFT JOIN users u1 ON d.created_by = u1.id
            LEFT JOIN users u2 ON d.approved_by = u2.id
            LEFT JOIN document_lines dl ON d.id = dl.document_id
            WHERE 1=1
            "#
        );

        if query.doc_type.is_some() {
            sql.push_str(" AND d.doc_type = $1");
        }

        sql.push_str(" GROUP BY d.id, w.name, p.name, u1.display_name, u2.display_name ORDER BY d.created_at DESC");

        // 簡化查詢（實際應用中應使用 query builder）
        let documents = if let Some(doc_type) = query.doc_type {
            sqlx::query_as::<_, DocumentListItem>(&sql)
                .bind(doc_type)
                .fetch_all(pool)
                .await?
        } else {
            let simple_sql = r#"
                SELECT 
                    d.id, d.doc_type, d.doc_no, d.status,
                    w.name as warehouse_name,
                    p.name as partner_name,
                    d.doc_date,
                    u1.display_name as created_by_name,
                    u2.display_name as approved_by_name,
                    d.created_at, d.approved_at,
                    COUNT(dl.id) as line_count,
                    SUM(dl.qty * COALESCE(dl.unit_price, 0)) as total_amount
                FROM documents d
                LEFT JOIN warehouses w ON d.warehouse_id = w.id
                LEFT JOIN partners p ON d.partner_id = p.id
                LEFT JOIN users u1 ON d.created_by = u1.id
                LEFT JOIN users u2 ON d.approved_by = u2.id
                LEFT JOIN document_lines dl ON d.id = dl.document_id
                GROUP BY d.id, w.name, p.name, u1.display_name, u2.display_name
                ORDER BY d.created_at DESC
            "#;
            sqlx::query_as::<_, DocumentListItem>(simple_sql)
                .fetch_all(pool)
                .await?
        };

        Ok(documents)
    }

    /// 取得單一單據
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<DocumentWithLines> {
        let document = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        let lines = sqlx::query_as::<_, DocumentLineWithProduct>(
            r#"
            SELECT 
                dl.id, dl.document_id, dl.line_no, dl.product_id,
                p.sku as product_sku, p.name as product_name,
                dl.qty, dl.uom, dl.unit_price, dl.batch_no, dl.expiry_date, dl.remark
            FROM document_lines dl
            INNER JOIN products p ON dl.product_id = p.id
            WHERE dl.document_id = $1
            ORDER BY dl.line_no
            "#
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        // 取得關聯名稱
        let warehouse_name: Option<String> = if let Some(wid) = document.warehouse_id {
            sqlx::query_scalar("SELECT name FROM warehouses WHERE id = $1")
                .bind(wid)
                .fetch_optional(pool)
                .await?
        } else {
            None
        };

        let warehouse_from_name: Option<String> = if let Some(wid) = document.warehouse_from_id {
            sqlx::query_scalar("SELECT name FROM warehouses WHERE id = $1")
                .bind(wid)
                .fetch_optional(pool)
                .await?
        } else {
            None
        };

        let warehouse_to_name: Option<String> = if let Some(wid) = document.warehouse_to_id {
            sqlx::query_scalar("SELECT name FROM warehouses WHERE id = $1")
                .bind(wid)
                .fetch_optional(pool)
                .await?
        } else {
            None
        };

        let partner_name: Option<String> = if let Some(pid) = document.partner_id {
            sqlx::query_scalar("SELECT name FROM partners WHERE id = $1")
                .bind(pid)
                .fetch_optional(pool)
                .await?
        } else {
            None
        };

        let created_by_name: String = sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
            .bind(document.created_by)
            .fetch_one(pool)
            .await?;

        let approved_by_name: Option<String> = if let Some(uid) = document.approved_by {
            sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
                .bind(uid)
                .fetch_optional(pool)
                .await?
        } else {
            None
        };

        Ok(DocumentWithLines {
            document,
            lines,
            warehouse_name,
            warehouse_from_name,
            warehouse_to_name,
            partner_name,
            created_by_name,
            approved_by_name,
        })
    }

    /// 產生單據編號
    async fn generate_doc_no(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, doc_type: DocType) -> Result<String> {
        let today = Utc::now().format("%Y%m%d").to_string();
        let prefix = format!("{}-{}", doc_type.prefix(), today);

        // 取得當天最後一個序號
        let last_no: Option<String> = sqlx::query_scalar(
            "SELECT doc_no FROM documents WHERE doc_no LIKE $1 ORDER BY doc_no DESC LIMIT 1"
        )
        .bind(format!("{}%", prefix))
        .fetch_optional(&mut **tx)
        .await?;

        let seq = if let Some(last) = last_no {
            let parts: Vec<&str> = last.split('-').collect();
            if parts.len() >= 3 {
                parts[2].parse::<i32>().unwrap_or(0) + 1
            } else {
                1
            }
        } else {
            1
        };

        Ok(format!("{}-{:04}", prefix, seq))
    }
}
