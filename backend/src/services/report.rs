use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::Result;

pub struct ReportService;

/// 摨怠??暹??梯”?
#[derive(Debug, FromRow, serde::Serialize)]
pub struct StockOnHandReport {
    pub warehouse_id: Uuid,
    pub warehouse_code: String,
    pub warehouse_name: String,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub category_name: Option<String>,
    pub base_uom: String,
    pub qty_on_hand: Decimal,
    pub avg_cost: Option<Decimal>,
    pub total_value: Option<Decimal>,
    pub safety_stock: Option<Decimal>,
    pub reorder_point: Option<Decimal>,
}

/// 摨怠?瘚偌?梯”?
#[derive(Debug, FromRow, serde::Serialize)]
pub struct StockLedgerReport {
    pub trx_date: chrono::DateTime<chrono::Utc>,
    pub warehouse_code: String,
    pub warehouse_name: String,
    pub product_sku: String,
    pub product_name: String,
    pub doc_type: String,
    pub doc_no: String,
    pub direction: String,
    pub qty_base: Decimal,
    pub unit_cost: Option<Decimal>,
    pub batch_no: Option<String>,
    pub expiry_date: Option<NaiveDate>,
}

/// ?∟頃?敦?梯”?
#[derive(Debug, FromRow, serde::Serialize)]
pub struct PurchaseLinesReport {
    pub doc_date: NaiveDate,
    pub doc_no: String,
    pub status: String,
    pub partner_code: Option<String>,
    pub partner_name: Option<String>,
    pub warehouse_name: Option<String>,
    pub product_sku: String,
    pub product_name: String,
    pub qty: Decimal,
    pub uom: String,
    pub unit_price: Option<Decimal>,
    pub line_total: Option<Decimal>,
    pub created_by_name: String,
    pub approved_by_name: Option<String>,
}

/// ?瑕?敦?梯”?
#[derive(Debug, FromRow, serde::Serialize)]
pub struct SalesLinesReport {
    pub doc_date: NaiveDate,
    pub doc_no: String,
    pub status: String,
    pub partner_code: Option<String>,
    pub partner_name: Option<String>,
    pub customer_category: Option<String>,
    pub warehouse_name: Option<String>,
    pub product_sku: String,
    pub product_name: String,
    pub qty: Decimal,
    pub uom: String,
    pub unit_price: Option<Decimal>,
    pub line_total: Option<Decimal>,
    pub created_by_name: String,
    pub approved_by_name: Option<String>,
}

/// ????梯”?/// 成本摘要報表
#[derive(Debug, FromRow, serde::Serialize)]
pub struct CostSummaryReport {
    pub warehouse_id: Uuid,
    pub warehouse_code: String,
    pub warehouse_name: String,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub category_name: Option<String>,
    pub qty_on_hand: Decimal,
    pub avg_cost: Option<Decimal>,
    pub total_value: Option<Decimal>,
}

/// 血液檢查費用報表
#[derive(Debug, FromRow, serde::Serialize)]
pub struct BloodTestCostReport {
    pub iacuc_no: Option<String>,
    pub ear_tag: String,
    pub pig_id: Uuid,
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub item_count: i64,
    pub total_cost: Option<Decimal>,
    pub created_by_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 報表查詢參數
#[derive(Debug, serde::Deserialize)]
pub struct ReportQuery {
    pub warehouse_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub partner_id: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    pub iacuc_no: Option<String>,
    pub lab_name: Option<String>,
    pub customer_category: Option<String>,
}

impl ReportService {
    /// 摨怠??暹??梯”
    pub async fn stock_on_hand(pool: &PgPool, query: &ReportQuery) -> Result<Vec<StockOnHandReport>> {
        let mut sql = String::from(
            r#"
            WITH inventory AS (
                SELECT 
                    warehouse_id, 
                    product_id,
                    SUM(CASE 
                        WHEN direction IN ('in', 'transfer_in', 'adjust_in') THEN qty_base 
                        ELSE -qty_base 
                    END) as qty_on_hand,
                    AVG(unit_cost) FILTER (WHERE unit_cost IS NOT NULL) as avg_cost
                FROM stock_ledger
                GROUP BY warehouse_id, product_id
            )
            SELECT 
                w.id as warehouse_id,
                w.code as warehouse_code,
                w.name as warehouse_name,
                p.id as product_id,
                p.sku as product_sku,
                p.name as product_name,
                pc.name as category_name,
                p.base_uom,
                COALESCE(i.qty_on_hand, 0) as qty_on_hand,
                i.avg_cost,
                COALESCE(i.qty_on_hand, 0) * COALESCE(i.avg_cost, 0) as total_value,
                p.safety_stock,
                p.reorder_point
            FROM warehouses w
            CROSS JOIN products p
            LEFT JOIN inventory i ON w.id = i.warehouse_id AND p.id = i.product_id
            LEFT JOIN product_categories pc ON p.category_id = pc.id
            WHERE w.is_active = true AND p.is_active = true
            "#
        );
        let mut param_idx = 1;
        if query.warehouse_id.is_some() {
            sql.push_str(&format!(" AND w.id = ${}", param_idx));
            param_idx += 1;
        }
        if query.product_id.is_some() {
            sql.push_str(&format!(" AND p.id = ${}", param_idx));
            param_idx += 1;
        }
        if query.category_id.is_some() {
            sql.push_str(&format!(" AND p.category_id = ${}", param_idx));
        }

        sql.push_str(" AND COALESCE(i.qty_on_hand, 0) != 0");
        sql.push_str(" ORDER BY w.code, p.sku");

        let mut query_builder = sqlx::query_as::<_, StockOnHandReport>(&sql);
        
        if let Some(wid) = query.warehouse_id {
            query_builder = query_builder.bind(wid);
        }
        if let Some(pid) = query.product_id {
            query_builder = query_builder.bind(pid);
        }
        if let Some(cid) = query.category_id {
            query_builder = query_builder.bind(cid);
        }

        // 蝪∪??閰ｇ?銝蝙?典???摰?
        let results = query_builder.fetch_all(pool).await?;

        Ok(results)
    }

    /// 摨怠?瘚偌?梯”
    pub async fn stock_ledger(pool: &PgPool, _query: &ReportQuery) -> Result<Vec<StockLedgerReport>> {
        let results = sqlx::query_as::<_, StockLedgerReport>(
            r#"
            SELECT 
                sl.trx_date,
                w.code as warehouse_code,
                w.name as warehouse_name,
                p.sku as product_sku,
                p.name as product_name,
                sl.doc_type::text as doc_type,
                sl.doc_no,
                sl.direction::text as direction,
                sl.qty_base,
                sl.unit_cost,
                sl.batch_no,
                sl.expiry_date
            FROM stock_ledger sl
            INNER JOIN warehouses w ON sl.warehouse_id = w.id
            INNER JOIN products p ON sl.product_id = p.id
            ORDER BY sl.trx_date DESC, sl.doc_no
            LIMIT 1000
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(results)
    }

    /// ?∟頃?敦?梯”
    pub async fn purchase_lines(pool: &PgPool, _query: &ReportQuery) -> Result<Vec<PurchaseLinesReport>> {
        let results = sqlx::query_as::<_, PurchaseLinesReport>(
            r#"
            SELECT 
                d.doc_date,
                d.doc_no,
                d.status::text as status,
                pa.code as partner_code,
                pa.name as partner_name,
                w.name as warehouse_name,
                p.sku as product_sku,
                p.name as product_name,
                dl.qty,
                dl.uom,
                dl.unit_price,
                dl.qty * COALESCE(dl.unit_price, 0) as line_total,
                u1.display_name as created_by_name,
                u2.display_name as approved_by_name
            FROM documents d
            INNER JOIN document_lines dl ON d.id = dl.document_id
            INNER JOIN products p ON dl.product_id = p.id
            LEFT JOIN warehouses w ON d.warehouse_id = w.id
            LEFT JOIN partners pa ON d.partner_id = pa.id
            INNER JOIN users u1 ON d.created_by = u1.id
            LEFT JOIN users u2 ON d.approved_by = u2.id
            WHERE d.doc_type IN ('PO', 'GRN', 'PR')
            ORDER BY d.doc_date DESC, d.doc_no, dl.line_no
            LIMIT 1000
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(results)
    }

    /// 銷售明細報表
    pub async fn sales_lines(pool: &PgPool, query: &ReportQuery) -> Result<Vec<SalesLinesReport>> {
        let mut sql = String::from(
            r#"
            SELECT 
                d.doc_date,
                d.doc_no,
                d.status::text as status,
                pa.code as partner_code,
                pa.name as partner_name,
                pa.customer_category::text as customer_category,
                w.name as warehouse_name,
                p.sku as product_sku,
                p.name as product_name,
                dl.qty,
                dl.uom,
                COALESCE(dl.unit_price,
                    (SELECT AVG(sl.unit_cost) FROM stock_ledger sl
                     WHERE sl.product_id = dl.product_id AND sl.unit_cost IS NOT NULL)
                ) as unit_price,
                dl.qty * COALESCE(dl.unit_price,
                    (SELECT AVG(sl.unit_cost) FROM stock_ledger sl
                     WHERE sl.product_id = dl.product_id AND sl.unit_cost IS NOT NULL),
                    0) as line_total,
                u1.display_name as created_by_name,
                u2.display_name as approved_by_name
            FROM documents d
            INNER JOIN document_lines dl ON d.id = dl.document_id
            INNER JOIN products p ON dl.product_id = p.id
            LEFT JOIN warehouses w ON d.warehouse_id = w.id
            LEFT JOIN partners pa ON d.partner_id = pa.id
            INNER JOIN users u1 ON d.created_by = u1.id
            LEFT JOIN users u2 ON d.approved_by = u2.id
            WHERE d.doc_type IN ('SO', 'DO')
            "#
        );

        let mut param_idx = 1;
        if query.partner_id.is_some() {
            sql.push_str(&format!(" AND d.partner_id = ${}", param_idx));
            param_idx += 1;
        }
        if query.customer_category.is_some() {
            sql.push_str(&format!(" AND pa.customer_category::text = ${}", param_idx));
            param_idx += 1;
        }
        if query.date_from.is_some() {
            sql.push_str(&format!(" AND d.doc_date >= ${}", param_idx));
            param_idx += 1;
        }
        if query.date_to.is_some() {
            sql.push_str(&format!(" AND d.doc_date <= ${}", param_idx));
            // param_idx += 1;
        }

        sql.push_str(" ORDER BY d.doc_date DESC, d.doc_no, dl.line_no LIMIT 1000");

        let mut query_builder = sqlx::query_as::<_, SalesLinesReport>(&sql);
        if let Some(pid) = query.partner_id {
            query_builder = query_builder.bind(pid);
        }
        if let Some(ref cc) = query.customer_category {
            query_builder = query_builder.bind(cc);
        }
        if let Some(df) = query.date_from {
            query_builder = query_builder.bind(df);
        }
        if let Some(dt) = query.date_to {
            query_builder = query_builder.bind(dt);
        }

        let results = query_builder.fetch_all(pool).await?;
        Ok(results)
    }

    /// ????梯”
    pub async fn cost_summary(pool: &PgPool, _query: &ReportQuery) -> Result<Vec<CostSummaryReport>> {
        let results = sqlx::query_as::<_, CostSummaryReport>(
            r#"
            WITH inventory AS (
                SELECT 
                    warehouse_id, 
                    product_id,
                    SUM(CASE 
                        WHEN direction IN ('in', 'transfer_in', 'adjust_in') THEN qty_base 
                        ELSE -qty_base 
                    END) as qty_on_hand,
                    AVG(unit_cost) FILTER (WHERE unit_cost IS NOT NULL) as avg_cost
                FROM stock_ledger
                GROUP BY warehouse_id, product_id
                HAVING SUM(CASE 
                    WHEN direction IN ('in', 'transfer_in', 'adjust_in') THEN qty_base 
                    ELSE -qty_base 
                END) > 0
            )
            SELECT 
                w.id as warehouse_id,
                w.code as warehouse_code,
                w.name as warehouse_name,
                p.id as product_id,
                p.sku as product_sku,
                p.name as product_name,
                pc.name as category_name,
                i.qty_on_hand,
                i.avg_cost,
                i.qty_on_hand * COALESCE(i.avg_cost, 0) as total_value
            FROM inventory i
            INNER JOIN warehouses w ON i.warehouse_id = w.id
            INNER JOIN products p ON i.product_id = p.id
            LEFT JOIN product_categories pc ON p.category_id = pc.id
            ORDER BY w.code, p.sku
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(results)
    }

    /// 血液檢查費用報表（以專案、日期區間、實驗室篩選）
    pub async fn blood_test_cost(pool: &PgPool, query: &ReportQuery) -> Result<Vec<BloodTestCostReport>> {
        let mut sql = String::from(
            r#"
            SELECT 
                pig.iacuc_no,
                pig.ear_tag,
                bt.pig_id,
                bt.test_date,
                bt.lab_name,
                COUNT(bti.id) as item_count,
                SUM(COALESCE(tmpl.default_price, 0)) as total_cost,
                u.display_name as created_by_name,
                bt.created_at
            FROM pig_blood_tests bt
            INNER JOIN pigs pig ON bt.pig_id = pig.id
            LEFT JOIN pig_blood_test_items bti ON bt.id = bti.blood_test_id
            LEFT JOIN blood_test_templates tmpl ON bti.template_id = tmpl.id
            LEFT JOIN users u ON bt.created_by = u.id
            WHERE bt.is_deleted = false
            "#
        );

        let mut param_idx = 1;
        if query.iacuc_no.is_some() {
            sql.push_str(&format!(" AND pig.iacuc_no = ${}", param_idx));
            param_idx += 1;
        }
        if query.date_from.is_some() {
            sql.push_str(&format!(" AND bt.test_date >= ${}", param_idx));
            param_idx += 1;
        }
        if query.date_to.is_some() {
            sql.push_str(&format!(" AND bt.test_date <= ${}", param_idx));
            param_idx += 1;
        }
        if query.lab_name.is_some() {
            sql.push_str(&format!(" AND bt.lab_name ILIKE ${}", param_idx));
            // param_idx += 1; // 最後一個參數不需要遞增
        }

        sql.push_str(
            r#"
            GROUP BY pig.iacuc_no, pig.ear_tag, bt.pig_id, bt.test_date, bt.lab_name, u.display_name, bt.created_at
            ORDER BY bt.test_date DESC, pig.iacuc_no, pig.ear_tag
            LIMIT 1000
            "#
        );

        let mut query_builder = sqlx::query_as::<_, BloodTestCostReport>(&sql);

        if let Some(ref iacuc_no) = query.iacuc_no {
            query_builder = query_builder.bind(iacuc_no);
        }
        if let Some(date_from) = query.date_from {
            query_builder = query_builder.bind(date_from);
        }
        if let Some(date_to) = query.date_to {
            query_builder = query_builder.bind(date_to);
        }
        if let Some(ref lab_name) = query.lab_name {
            query_builder = query_builder.bind(format!("%{}%", lab_name));
        }

        let results = query_builder.fetch_all(pool).await?;
        Ok(results)
    }
}
