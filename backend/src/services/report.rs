use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::Result;

pub struct ReportService;

/// 庫存現況報表
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

/// 庫存異動報表
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

/// 採購明細報表
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

/// 銷貨明細報表
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

/// 成本摘要報表
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
    pub animal_id: Uuid,
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub item_count: i64,
    pub total_cost: Option<Decimal>,
    pub created_by_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 血液檢查分析原始列（供前端聚合與視覺化）
#[derive(Debug, FromRow, serde::Serialize)]
pub struct BloodTestAnalysisRow {
    pub animal_id: Uuid,
    pub ear_tag: String,
    pub iacuc_no: Option<String>,
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub item_name: String,
    pub template_code: Option<String>,
    pub result_value: Option<String>,
    pub result_unit: Option<String>,
    pub reference_range: Option<String>,
    pub is_abnormal: bool,
}

/// 血液檢查分析查詢參數
#[derive(Debug, serde::Deserialize)]
pub struct BloodTestAnalysisQuery {
    pub iacuc_no: Option<String>,
    pub animal_id: Option<Uuid>,
    pub item_name: Option<String>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
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
    /// 庫存現況報表
    pub async fn stock_on_hand(pool: &PgPool, query: &ReportQuery) -> Result<Vec<StockOnHandReport>> {
        let mut qb = sqlx::QueryBuilder::new(
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
            "#,
        );

        if let Some(wid) = query.warehouse_id {
            qb.push(" AND w.id = ");
            qb.push_bind(wid);
        }
        if let Some(pid) = query.product_id {
            qb.push(" AND p.id = ");
            qb.push_bind(pid);
        }
        if let Some(cid) = query.category_id {
            qb.push(" AND p.category_id = ");
            qb.push_bind(cid);
        }

        qb.push(" AND COALESCE(i.qty_on_hand, 0) != 0 ORDER BY w.code, p.sku");

        let results = qb.build_query_as::<StockOnHandReport>().fetch_all(pool).await?;
        Ok(results)
    }

    /// 庫存異動報表
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

    /// 採購明細報表
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

    /// 銷貨明細報表
    pub async fn sales_lines(pool: &PgPool, query: &ReportQuery) -> Result<Vec<SalesLinesReport>> {
        let mut qb = sqlx::QueryBuilder::new(
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
            "#,
        );

        if let Some(pid) = query.partner_id {
            qb.push(" AND d.partner_id = ");
            qb.push_bind(pid);
        }
        if let Some(ref cc) = query.customer_category {
            qb.push(" AND pa.customer_category::text = ");
            qb.push_bind(cc.clone());
        }
        if let Some(df) = query.date_from {
            qb.push(" AND d.doc_date >= ");
            qb.push_bind(df);
        }
        if let Some(dt) = query.date_to {
            qb.push(" AND d.doc_date <= ");
            qb.push_bind(dt);
        }

        qb.push(" ORDER BY d.doc_date DESC, d.doc_no, dl.line_no LIMIT 1000");

        let results = qb.build_query_as::<SalesLinesReport>().fetch_all(pool).await?;
        Ok(results)
    }

    /// 成本摘要報表
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
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT 
                a.iacuc_no,
                a.ear_tag,
                bt.animal_id,
                bt.test_date,
                bt.lab_name,
                COUNT(bti.id) as item_count,
                SUM(COALESCE(tmpl.default_price, 0)) as total_cost,
                u.display_name as created_by_name,
                bt.created_at
            FROM animal_blood_tests bt
            INNER JOIN animals a ON bt.animal_id = a.id
            LEFT JOIN animal_blood_test_items bti ON bt.id = bti.blood_test_id
            LEFT JOIN blood_test_templates tmpl ON bti.template_id = tmpl.id
            LEFT JOIN users u ON bt.created_by = u.id
            WHERE bt.deleted_at IS NULL
            "#,
        );

        if let Some(ref iacuc_no) = query.iacuc_no {
            qb.push(" AND a.iacuc_no = ");
            qb.push_bind(iacuc_no.clone());
        }
        if let Some(df) = query.date_from {
            qb.push(" AND bt.test_date >= ");
            qb.push_bind(df);
        }
        if let Some(dt) = query.date_to {
            qb.push(" AND bt.test_date <= ");
            qb.push_bind(dt);
        }
        if let Some(ref lab_name) = query.lab_name {
            qb.push(" AND bt.lab_name ILIKE ");
            qb.push_bind(format!("%{}%", lab_name));
        }

        qb.push(
            r#"
            GROUP BY a.iacuc_no, a.ear_tag, bt.animal_id, bt.test_date, bt.lab_name, u.display_name, bt.created_at
            ORDER BY bt.test_date DESC, a.iacuc_no, a.ear_tag
            LIMIT 1000
            "#,
        );

        let results = qb.build_query_as::<BloodTestCostReport>().fetch_all(pool).await?;
        Ok(results)
    }

    /// 血液檢查結果分析（扁平化原始數據，供前端聚合）
    /// * restrict_to_project_animals: true 時僅回傳已指派計畫之動物（iacuc_no IS NOT NULL），對應 view_project 權限
    pub async fn blood_test_analysis(
        pool: &PgPool,
        query: &BloodTestAnalysisQuery,
        restrict_to_project_animals: bool,
    ) -> Result<Vec<BloodTestAnalysisRow>> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT 
                a.id as animal_id,
                a.ear_tag,
                a.iacuc_no,
                bt.test_date,
                bt.lab_name,
                bti.item_name,
                tmpl.code as template_code,
                bti.result_value,
                bti.result_unit,
                bti.reference_range,
                bti.is_abnormal
            FROM animal_blood_test_items bti
            INNER JOIN animal_blood_tests bt ON bti.blood_test_id = bt.id
            INNER JOIN animals a ON bt.animal_id = a.id
            LEFT JOIN blood_test_templates tmpl ON bti.template_id = tmpl.id
            WHERE bt.deleted_at IS NULL AND a.deleted_at IS NULL
            "#,
        );
        if restrict_to_project_animals {
            qb.push(" AND a.iacuc_no IS NOT NULL ");
        }

        if let Some(ref iacuc_no) = query.iacuc_no {
            qb.push(" AND a.iacuc_no = ");
            qb.push_bind(iacuc_no.clone());
        }
        if let Some(animal_id) = query.animal_id {
            qb.push(" AND a.id = ");
            qb.push_bind(animal_id);
        }
        if let Some(ref item_name) = query.item_name {
            qb.push(" AND bti.item_name = ");
            qb.push_bind(item_name.clone());
        }
        if let Some(df) = query.date_from {
            qb.push(" AND bt.test_date >= ");
            qb.push_bind(df);
        }
        if let Some(dt) = query.date_to {
            qb.push(" AND bt.test_date <= ");
            qb.push_bind(dt);
        }

        qb.push(" ORDER BY bt.test_date ASC, a.ear_tag, bti.sort_order LIMIT 5000");

        let results = qb.build_query_as::<BloodTestAnalysisRow>().fetch_all(pool).await?;
        Ok(results)
    }
}
