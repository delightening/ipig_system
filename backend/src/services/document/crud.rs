use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        CreateDocumentRequest, DocStatus, DocType, Document, DocumentLine, DocumentLineInput,
        DocumentLineWithProduct, DocumentListItem, DocumentQuery, DocumentWithLines,
        UpdateDocumentRequest,
    },
    AppError, Result,
};

use super::DocumentService;

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
            req.lines.clone()
        };

        // 建立單據頭
        let document = sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (
                id, doc_type, doc_no, status, warehouse_id, warehouse_from_id, warehouse_to_id,
                partner_id, doc_date, remark, stocktake_scope, iacuc_no, created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NOW())
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
        .bind(req.doc_date)
        .bind(&req.remark)
        .bind(req.stocktake_scope.as_ref().map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null)))
        .bind(&req.iacuc_no)
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

        tx.commit().await?;

        Self::get_by_id(pool, document.id).await
    }

    /// 更新單據（僅 Draft 狀態）
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateDocumentRequest,
    ) -> Result<DocumentWithLines> {
        let existing = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if existing.status != DocStatus::Draft {
            return Err(AppError::BusinessRule(
                "Only draft documents can be updated".to_string(),
            ));
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
            "#,
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
            let lines: &Vec<DocumentLineInput> = lines;
            for (idx, line) in lines.iter().enumerate() {
                // 驗證必填欄位
                if line.uom.is_empty() {
                    return Err(AppError::Validation(format!(
                        "Line {}: UOM is required",
                        idx + 1
                    )));
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

        tx.commit().await?;

        Self::get_by_id(pool, id).await
    }

    /// 刪除單據（僅限草稿狀態）
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let document = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        // 只允許刪除草稿狀態的單據
        if document.status != DocStatus::Draft {
            return Err(AppError::BusinessRule(
                "Only draft documents can be deleted".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

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
                SUM(dl.qty * COALESCE(dl.unit_price,
                    (SELECT AVG(sl.unit_cost) FROM stock_ledger sl
                     WHERE sl.product_id = dl.product_id AND sl.unit_cost IS NOT NULL),
                    0)) as total_amount,
                d.iacuc_no
            FROM documents d
            LEFT JOIN warehouses w ON d.warehouse_id = w.id
            LEFT JOIN partners p ON d.partner_id = p.id
            LEFT JOIN users u1 ON d.created_by = u1.id
            LEFT JOIN users u2 ON d.approved_by = u2.id
            LEFT JOIN document_lines dl ON d.id = dl.document_id
            WHERE 1=1
            "#,
        );

        // 動態建構查詢條件
        let mut param_count = 0;

        if query.doc_type.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND d.doc_type = ${}", param_count));
        }

        if query.iacuc_no.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND d.iacuc_no = ${}", param_count));
        }

        sql.push_str(" GROUP BY d.id, w.name, p.name, u1.display_name, u2.display_name, d.iacuc_no ORDER BY d.created_at DESC");

        // 動態綁定參數
        let mut query_builder = sqlx::query_as::<_, DocumentListItem>(&sql);

        if let Some(doc_type) = query.doc_type {
            query_builder = query_builder.bind(doc_type);
        }

        if let Some(ref iacuc_no) = query.iacuc_no {
            query_builder = query_builder.bind(iacuc_no);
        }

        let documents = query_builder.fetch_all(pool).await?;

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

        let created_by_name: String =
            sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
                .bind(document.created_by)
                .fetch_optional(pool)
                .await?
                .unwrap_or_else(|| "Unknown User".to_string());

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
    /// 格式：{PREFIX}-YYMMDD-{02} (例如：SO-260115-01)
    pub(crate) async fn generate_doc_no(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        doc_type: DocType,
    ) -> Result<String> {
        // 統一使用 YYMMDD 格式
        let today = Utc::now();
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
