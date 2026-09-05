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
    /// 來源單據 ID（R84-4：供前端連結到單據詳情頁）
    pub doc_id: Uuid,
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
    /// 單據 ID（R84-4：供前端連結到單據詳情頁）
    pub doc_id: Uuid,
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

/// 領用明細報表（欄位/API 名稱沿用歷史「銷貨」命名，見 docs/spec/modules/ERP_SYSTEM.md §5.1）
#[derive(Debug, FromRow, serde::Serialize)]
pub struct SalesLinesReport {
    /// 單據 ID（R84-4：供前端連結到單據詳情頁）
    pub doc_id: Uuid,
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

/// 案件消耗報表：一列 = 一個案件 × 一個品項的淨消耗。
///
/// 前端用同一份資料轉三種聚合軸（案件→品項、品項→案件、交叉表），
/// 所以這裡回的是最細的顆粒度，不在後端先聚合掉任何一軸。
///
/// 數量一律以 `base_uom` 計——2026-09-04 裁定：**案件側論領用單位、倉庫側才論包裝單位**。
/// 倉庫側的呈現是另一支報表（`stock_on_hand`）的事，本報表不做包裝換算。
#[derive(Debug, FromRow, serde::Serialize)]
pub struct ProtocolConsumptionReport {
    pub protocol_id: Uuid,
    pub protocol_no: String,
    /// 核准編號。DRAFT 階段的計畫尚未取得，故可為 NULL。
    pub iacuc_no: Option<String>,
    pub protocol_title: Option<String>,
    pub product_id: Uuid,
    pub product_sku: String,
    pub product_name: String,
    pub category_name: Option<String>,
    pub base_uom: String,
    /// 淨消耗量（`SO/out` 減去沖銷產生的 `SO/in` 鏡射列），以 `base_uom` 計。
    pub qty_base: Decimal,
    /// 貢獻此消耗的單據張數（沖銷單也算一張）。
    pub doc_count: i64,
    pub first_trx_date: chrono::DateTime<chrono::Utc>,
    pub last_trx_date: chrono::DateTime<chrono::Utc>,
    /// 依 `stock_ledger.unit_cost` 加總。未維護成本的異動以 0 計，故可能低估。
    pub total_cost: Option<Decimal>,
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
    /// 案件消耗報表用。⚠️ 不要改用 `iacuc_no` 篩領用——領用單（SO）只填
    /// `protocol_id`，`iacuc_no` 在該單別是停用欄位（`useDocumentForm.ts:120`），
    /// 按它篩會一筆都抓不到。
    pub protocol_id: Option<Uuid>,
}

impl ReportService {
    /// 庫存現況報表
    pub async fn stock_on_hand(
        pool: &PgPool,
        query: &ReportQuery,
    ) -> Result<Vec<StockOnHandReport>> {
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

        let results = qb
            .build_query_as::<StockOnHandReport>()
            .fetch_all(pool)
            .await?;
        Ok(results)
    }

    /// 庫存異動報表（支援篩選）
    pub async fn stock_ledger(
        pool: &PgPool,
        query: &ReportQuery,
    ) -> Result<Vec<StockLedgerReport>> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT
                sl.doc_id,
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
            WHERE 1=1
            "#,
        );

        if let Some(wid) = query.warehouse_id {
            qb.push(" AND sl.warehouse_id = ");
            qb.push_bind(wid);
        }
        if let Some(pid) = query.product_id {
            qb.push(" AND sl.product_id = ");
            qb.push_bind(pid);
        }
        if let Some(df) = query.date_from {
            qb.push(" AND sl.trx_date >= ");
            qb.push_bind(df);
        }
        if let Some(dt) = query.date_to {
            qb.push(" AND sl.trx_date <= ");
            qb.push_bind(dt);
        }

        qb.push(" ORDER BY sl.trx_date DESC, sl.doc_no LIMIT 1000");

        let results = qb
            .build_query_as::<StockLedgerReport>()
            .fetch_all(pool)
            .await?;
        Ok(results)
    }

    /// 採購明細報表（支援篩選）
    pub async fn purchase_lines(
        pool: &PgPool,
        query: &ReportQuery,
    ) -> Result<Vec<PurchaseLinesReport>> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT
                d.id as doc_id,
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
            "#,
        );

        if let Some(pid) = query.partner_id {
            qb.push(" AND d.partner_id = ");
            qb.push_bind(pid);
        }
        if let Some(wid) = query.warehouse_id {
            qb.push(" AND d.warehouse_id = ");
            qb.push_bind(wid);
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

        let results = qb
            .build_query_as::<PurchaseLinesReport>()
            .fetch_all(pool)
            .await?;
        Ok(results)
    }

    /// 銷貨明細報表
    pub async fn sales_lines(pool: &PgPool, query: &ReportQuery) -> Result<Vec<SalesLinesReport>> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT
                d.id as doc_id,
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
            WHERE d.doc_type = 'SO'
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

        let results = qb
            .build_query_as::<SalesLinesReport>()
            .fetch_all(pool)
            .await?;
        Ok(results)
    }

    /// 案件消耗報表（依計畫 × 品項統計內部領用）
    ///
    /// ## 為什麼讀 `stock_ledger` 而不是 `documents + document_lines`
    ///
    /// 其他明細報表（`purchase_lines` / `sales_lines`）讀單據明細，那是「開了哪些單」。
    /// 本報表要回答的是「實際消耗了多少」，兩者在兩個地方會分歧：
    ///
    /// 1. **未核准的單**：`document_lines` 含 draft／submitted，但那些還沒扣帳。
    ///    `stock_ledger` 只在核准時寫入，天然排除。
    /// 2. **沖銷**：沖銷是**另一張單**（`documents.reverses_doc_id`），讀明細會把
    ///    已沖銷的領用照算。而沖銷在 ledger 裡是沿用原 `doc_type`、只反轉 `direction`
    ///    的鏡射列（`reverse_document_stock` / `reverse_direction`），所以在這裡減得掉。
    ///
    /// ## 消耗的定義
    ///
    /// `doc_type = 'SO' AND direction = 'out'` 加，`'SO' AND 'in'` 減。
    /// 這與 `stock::ledger` 的 `internal_consumed` 是**同一個定義**，刻意不另立一套
    /// ——兩處若分岔，同一筆消耗會在對帳頁與本報表算出不同答案。
    ///
    /// ERP 的出庫全為內部耗材領用，`SO` 從不認列收入（見 `models::document` 的說明），
    /// 所以「銷貨單」在語意上就是領用單。
    ///
    /// ## 分組鍵是 `protocol_id`，不是 `iacuc_no`
    ///
    /// 領用單只填 `protocol_id`；`iacuc_no` 對 SO 是停用欄位，只有 PO／PR 的費用歸屬
    /// 會填。按 `iacuc_no` 分組會一筆領用都抓不到。`iacuc_no` 在此只作顯示，
    /// 且 DRAFT 階段的計畫還沒有，故為 `Option`。
    pub async fn protocol_consumption(
        pool: &PgPool,
        query: &ReportQuery,
    ) -> Result<Vec<ProtocolConsumptionReport>> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT
                d.protocol_id,
                pr.protocol_no,
                pr.iacuc_no,
                pr.title as protocol_title,
                p.id as product_id,
                p.sku as product_sku,
                p.name as product_name,
                pc.name as category_name,
                p.base_uom,
                SUM(CASE WHEN sl.direction = 'out' THEN sl.qty_base ELSE -sl.qty_base END)
                    as qty_base,
                COUNT(DISTINCT sl.doc_id) as doc_count,
                MIN(sl.trx_date) as first_trx_date,
                MAX(sl.trx_date) as last_trx_date,
                SUM(
                    CASE WHEN sl.direction = 'out' THEN sl.qty_base ELSE -sl.qty_base END
                    * COALESCE(sl.unit_cost, 0)
                ) as total_cost
            FROM stock_ledger sl
            INNER JOIN documents d ON d.id = sl.doc_id
            INNER JOIN protocols pr ON pr.id = d.protocol_id
            INNER JOIN products p ON p.id = sl.product_id
            LEFT JOIN product_categories pc ON pc.id = p.category_id
            WHERE sl.doc_type = 'SO'
            "#,
        );

        if let Some(pid) = query.protocol_id {
            qb.push(" AND d.protocol_id = ");
            qb.push_bind(pid);
        }
        if let Some(prod) = query.product_id {
            qb.push(" AND p.id = ");
            qb.push_bind(prod);
        }
        if let Some(cid) = query.category_id {
            qb.push(" AND p.category_id = ");
            qb.push_bind(cid);
        }
        if let Some(wid) = query.warehouse_id {
            qb.push(" AND sl.warehouse_id = ");
            qb.push_bind(wid);
        }
        // 以異動時點篩，不是單據日期——單據日期可回填，扣帳時點不會。
        //
        // 🔴 日期邊界必須明確錨在台灣時間，不能讓裸日期直接跟 timestamptz 比。
        //
        // `sl.trx_date` 是 `timestamp with time zone`（002_schema.sql:5246），使用者送來的
        // 則是不帶時區的日期。直接比較時 PostgreSQL 會用 **session 時區**把日期解讀成該時區
        // 的午夜——而本專案沒有任何地方設過 session 時區（`startup/database.rs` 的
        // after_connect 只設 statement_timeout，db 容器也沒給 TZ），實際上就是 UTC。
        //
        // 後果：使用者選 2026-09-05 想看台灣的 9/5，邊界卻落在台灣時間 9/5 08:00，
        // **台灣時間 9/5 00:00–07:59 的領用會被算進 9/4**。這支報表是給 IACUC 稽核查
        // 「這段期間消耗了什麼」用的，差 8 小時會被追問。
        //
        // `AT TIME ZONE 'Asia/Taipei'` 把「不帶時區的當地午夜」轉成正確的 timestamptz，
        // 且由 tzdata 處理該時區的規則，不寫死 +08。
        //
        // ⚠️ 同檔的 `stock_ledger()`（:292-298）有一模一樣的問題，本 PR 不動它——
        // 那是既有函式，改它要連帶重驗既有報表，屬於另一件事。
        if let Some(df) = query.date_from {
            qb.push(" AND sl.trx_date >= (");
            qb.push_bind(df);
            qb.push("::date::timestamp AT TIME ZONE 'Asia/Taipei')");
        }
        if let Some(dt) = query.date_to {
            // 結束日當天要含在內，故比到隔天台灣時間零時之前。
            qb.push(" AND sl.trx_date < ((");
            qb.push_bind(dt);
            qb.push("::date + 1)::timestamp AT TIME ZONE 'Asia/Taipei')");
        }

        qb.push(
            r#"
            GROUP BY d.protocol_id, pr.protocol_no, pr.iacuc_no, pr.title,
                     p.id, p.sku, p.name, pc.name, p.base_uom
            "#,
        );
        // 淨額為 0 = 領了又整筆沖掉，對「消耗了多少」這個問題沒有資訊，只會佔版面。
        qb.push(
            " HAVING SUM(CASE WHEN sl.direction = 'out' THEN sl.qty_base \
             ELSE -sl.qty_base END) <> 0",
        );
        // 🔴 取 1001 筆而不是 1000：多出來的那一筆是給呼叫端的**截斷訊號**。
        //
        // 只取 1000 的話，「剛好 1000 組」與「被截掉了」在回應上長得一模一樣，
        // 前端無從分辨，只能用 `length >= 1000` 猜——而那會把前者誤報成後者。
        // 多要一筆就能精確判定：拿到 1001 筆 ⇒ 確定有更多；1000 筆 ⇒ 確定剛好取完。
        //
        // 為什麼不改成回 `{ rows, has_more }`：ERP 報表的回應一律是裸陣列
        // （見 `guest-demo/routes.ts` 的註解），為一支報表破例會讓前端的
        // 報表資料流多一條分支。多帶一筆的成本是一列，換掉一個 API 形狀的例外。
        qb.push(" ORDER BY pr.protocol_no, p.sku LIMIT 1001");

        let results = qb
            .build_query_as::<ProtocolConsumptionReport>()
            .fetch_all(pool)
            .await?;
        Ok(results)
    }

    /// 成本摘要報表
    pub async fn cost_summary(
        pool: &PgPool,
        _query: &ReportQuery,
    ) -> Result<Vec<CostSummaryReport>> {
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
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(results)
    }

    /// 血液檢查費用報表（以專案、日期區間、實驗室篩選）
    pub async fn blood_test_cost(
        pool: &PgPool,
        query: &ReportQuery,
    ) -> Result<Vec<BloodTestCostReport>> {
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

        let results = qb
            .build_query_as::<BloodTestCostReport>()
            .fetch_all(pool)
            .await?;
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

        let results = qb
            .build_query_as::<BloodTestAnalysisRow>()
            .fetch_all(pool)
            .await?;
        Ok(results)
    }

    /// 進銷貨彙總 — 按月份
    pub async fn purchase_sales_monthly(
        pool: &PgPool,
        query: &ReportQuery,
    ) -> Result<Vec<PurchaseSalesMonthlySummary>> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            WITH doc_totals AS (
                SELECT
                    d.doc_type,
                    TO_CHAR(d.doc_date, 'YYYY-MM') as year_month,
                    COALESCE(SUM(dl.qty * COALESCE(dl.unit_price, 0)), 0) as total_amount
                FROM documents d
                INNER JOIN document_lines dl ON d.id = dl.document_id
                WHERE d.status = 'approved'
                  AND d.doc_type IN ('GRN', 'PR', 'SO')
            "#,
        );
        if let Some(df) = query.date_from {
            qb.push(" AND d.doc_date >= ");
            qb.push_bind(df);
        }
        if let Some(dt) = query.date_to {
            qb.push(" AND d.doc_date <= ");
            qb.push_bind(dt);
        }
        qb.push(
            r#"
                GROUP BY d.doc_type, TO_CHAR(d.doc_date, 'YYYY-MM')
            ),
            cogs_monthly AS (
                SELECT
                    TO_CHAR(je.entry_date, 'YYYY-MM') as year_month,
                    COALESCE(SUM(jel.debit_amount), 0) as cogs
                FROM journal_entry_lines jel
                INNER JOIN journal_entries je ON je.id = jel.journal_entry_id
                INNER JOIN chart_of_accounts coa ON coa.id = jel.account_id AND coa.code = '5200'
                WHERE je.source_entity_type = 'document'
            "#,
        );
        if let Some(df) = query.date_from {
            qb.push(" AND je.entry_date >= ");
            qb.push_bind(df);
        }
        if let Some(dt) = query.date_to {
            qb.push(" AND je.entry_date <= ");
            qb.push_bind(dt);
        }
        qb.push(
            r#"
                GROUP BY TO_CHAR(je.entry_date, 'YYYY-MM')
            ),
            months AS (
                SELECT DISTINCT year_month FROM doc_totals
                UNION
                SELECT DISTINCT year_month FROM cogs_monthly
            )
            SELECT
                m.year_month,
                COALESCE((SELECT total_amount FROM doc_totals WHERE doc_type::text = 'GRN' AND year_month = m.year_month), 0) as purchase_total,
                COALESCE((SELECT total_amount FROM doc_totals WHERE doc_type::text = 'PR' AND year_month = m.year_month), 0) as purchase_return,
                COALESCE((SELECT total_amount FROM doc_totals WHERE doc_type::text = 'GRN' AND year_month = m.year_month), 0)
                    - COALESCE((SELECT total_amount FROM doc_totals WHERE doc_type::text = 'PR' AND year_month = m.year_month), 0) as net_purchase,
                COALESCE((SELECT SUM(total_amount) FROM doc_totals WHERE doc_type::text = 'SO' AND year_month = m.year_month), 0) as sales_total,
                -- R84-13：SR/RTN 已封鎖新建（業務上不存在銷貨退貨），sales_return 恆為 0，
                -- net_sales/gross_profit 不再需要扣減（保留欄位維持前端契約不變）。
                0::NUMERIC(18,4) as sales_return,
                COALESCE((SELECT SUM(total_amount) FROM doc_totals WHERE doc_type::text = 'SO' AND year_month = m.year_month), 0) as net_sales,
                COALESCE(c.cogs, 0) as cogs_total,
                COALESCE((SELECT SUM(total_amount) FROM doc_totals WHERE doc_type::text = 'SO' AND year_month = m.year_month), 0)
                    - COALESCE(c.cogs, 0) as gross_profit
            FROM months m
            LEFT JOIN cogs_monthly c ON c.year_month = m.year_month
            ORDER BY m.year_month DESC
            "#,
        );

        let results = qb
            .build_query_as::<PurchaseSalesMonthlySummary>()
            .fetch_all(pool)
            .await?;
        Ok(results)
    }

    /// 進銷貨彙總 — 按供應商/客戶
    pub async fn purchase_sales_by_partner(
        pool: &PgPool,
        query: &ReportQuery,
    ) -> Result<Vec<PurchaseSalesPartnerSummary>> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT
                pa.id as partner_id,
                pa.code as partner_code,
                pa.name as partner_name,
                pa.partner_type::text as partner_type,
                COALESCE(SUM(CASE WHEN d.doc_type IN ('GRN', 'SO') THEN dl.qty * COALESCE(dl.unit_price, 0) ELSE 0 END), 0) as total_amount,
                -- R84-13：SR/RTN 已封鎖新建，退貨只剩 PR（採購退貨）。
                COALESCE(SUM(CASE WHEN d.doc_type = 'PR' THEN dl.qty * COALESCE(dl.unit_price, 0) ELSE 0 END), 0) as return_amount,
                COALESCE(SUM(CASE WHEN d.doc_type IN ('GRN', 'SO') THEN dl.qty * COALESCE(dl.unit_price, 0) ELSE 0 END), 0)
                    - COALESCE(SUM(CASE WHEN d.doc_type = 'PR' THEN dl.qty * COALESCE(dl.unit_price, 0) ELSE 0 END), 0) as net_amount,
                COUNT(DISTINCT d.id) as doc_count
            FROM documents d
            INNER JOIN document_lines dl ON d.id = dl.document_id
            INNER JOIN partners pa ON d.partner_id = pa.id
            WHERE d.status = 'approved'
              AND d.doc_type IN ('GRN', 'PR', 'SO')
            "#,
        );

        if let Some(df) = query.date_from {
            qb.push(" AND d.doc_date >= ");
            qb.push_bind(df);
        }
        if let Some(dt) = query.date_to {
            qb.push(" AND d.doc_date <= ");
            qb.push_bind(dt);
        }

        qb.push(
            " GROUP BY pa.id, pa.code, pa.name, pa.partner_type ORDER BY net_amount DESC LIMIT 100",
        );

        let results = qb
            .build_query_as::<PurchaseSalesPartnerSummary>()
            .fetch_all(pool)
            .await?;
        Ok(results)
    }

    /// 進銷貨彙總 — 按產品類別
    pub async fn purchase_sales_by_category(
        pool: &PgPool,
        query: &ReportQuery,
    ) -> Result<Vec<PurchaseSalesCategorySummary>> {
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT
                COALESCE(pc.name, '未分類') as category_name,
                COALESCE(SUM(CASE WHEN d.doc_type = 'GRN' THEN dl.qty * COALESCE(dl.unit_price, 0) WHEN d.doc_type = 'PR' THEN -(dl.qty * COALESCE(dl.unit_price, 0)) ELSE 0 END), 0) as purchase_amount,
                -- R84-13：SR/RTN 已封鎖新建（業務上不存在銷貨退貨），sales_amount 不再需要扣減分支。
                COALESCE(SUM(CASE WHEN d.doc_type = 'SO' THEN dl.qty * COALESCE(dl.unit_price, 0) ELSE 0 END), 0) as sales_amount,
                0::NUMERIC(18,4) as cogs_amount,
                COALESCE(SUM(CASE WHEN d.doc_type = 'SO' THEN dl.qty * COALESCE(dl.unit_price, 0) ELSE 0 END), 0) as gross_profit
            FROM documents d
            INNER JOIN document_lines dl ON d.id = dl.document_id
            INNER JOIN products p ON dl.product_id = p.id
            LEFT JOIN product_categories pc ON p.category_id = pc.id
            WHERE d.status = 'approved'
              AND d.doc_type IN ('GRN', 'PR', 'SO')
            "#,
        );

        if let Some(df) = query.date_from {
            qb.push(" AND d.doc_date >= ");
            qb.push_bind(df);
        }
        if let Some(dt) = query.date_to {
            qb.push(" AND d.doc_date <= ");
            qb.push_bind(dt);
        }

        qb.push(" GROUP BY pc.name ORDER BY purchase_amount DESC");

        let results = qb
            .build_query_as::<PurchaseSalesCategorySummary>()
            .fetch_all(pool)
            .await?;
        Ok(results)
    }
}

/// 進銷貨月份彙總
#[derive(Debug, FromRow, serde::Serialize)]
pub struct PurchaseSalesMonthlySummary {
    pub year_month: String,
    pub purchase_total: Decimal,
    pub purchase_return: Decimal,
    pub net_purchase: Decimal,
    pub sales_total: Decimal,
    pub sales_return: Decimal,
    pub net_sales: Decimal,
    pub cogs_total: Decimal,
    pub gross_profit: Decimal,
}

/// 進銷貨夥伴彙總
#[derive(Debug, FromRow, serde::Serialize)]
pub struct PurchaseSalesPartnerSummary {
    pub partner_id: Uuid,
    pub partner_code: String,
    pub partner_name: String,
    pub partner_type: String,
    pub total_amount: Decimal,
    pub return_amount: Decimal,
    pub net_amount: Decimal,
    pub doc_count: i64,
}

/// 進銷貨產品類別彙總
#[derive(Debug, FromRow, serde::Serialize)]
pub struct PurchaseSalesCategorySummary {
    pub category_name: String,
    pub purchase_amount: Decimal,
    pub sales_amount: Decimal,
    pub cogs_amount: Decimal,
    pub gross_profit: Decimal,
}
