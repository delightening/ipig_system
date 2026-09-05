use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    models::{
        DocType, Document, DocumentLine, LotMovement, LotMovementsQuery, LotMovementsResponse,
        LotReconciliation, LotReconciliationStatus, StockDirection, StockLedger, StockLedgerDetail,
        StockLedgerQuery,
    },
    services::WarehouseService,
    AppError, Result,
};

use super::StockService;

/// 庫存流水記錄所需參數
struct LedgerEntryParams<'a> {
    warehouse_id: Uuid,
    product_id: Uuid,
    document: &'a Document,
    line: &'a DocumentLine,
    direction: StockDirection,
    qty: Decimal,
    unit_price: Option<Decimal>,
    /// 異動發生的儲位 ID（migration 069）。對 TR 兩端各自記錄 from/to；
    /// 其他 doc type 沿用 `line.storage_location_id`。可為 None 表示
    /// 未指定儲位（warehouse-only 顆粒度）。
    storage_location_id: Option<Uuid>,
}

/// R84-6 批號對帳：依單據類型/方向分類加總的中繼結果（見 `get_lot_movements`）
#[derive(sqlx::FromRow)]
struct LotCategorizedTotals {
    received: Decimal,
    customer_returned: Decimal,
    internal_consumed: Decimal,
    returned_to_supplier: Decimal,
    adjusted_net: Decimal,
}

/// R84-6 批號對帳：品項層級（跨全部批號）的中繼結果，用於區分「批號歸屬問題」與「真的帳實不符」
#[derive(sqlx::FromRow)]
struct LotProductTotals {
    derived_total: Decimal,
    unattributed_adjust_net: Decimal,
}

impl StockService {
    /// 處理單據核准後的庫存變動
    pub async fn process_document(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
    ) -> Result<()> {
        // 單位換算（唯一入口）：以下所有計算——庫存足量檢查、stock_ledger、
        // storage_location_inventory 增減、inventory_snapshots 重算——一律以
        // products.base_uom 進行。明細若以「盒」開立，在此換成「雙」再往下走。
        // 換算表沒有該單位即 400，不靜默當 1（見 uom::to_base_lines）。
        let base_lines = Self::to_base_lines(tx, lines).await?;
        let lines = base_lines.as_slice();

        // 固定 (warehouse_id, product_id) 順序、且在**任何列鎖之前**先取 advisory lock：
        // 涉及重疊 (倉,品) 的並發核准在此先序列化。若等到逐行處理才鎖，
        // check_stock_available 的 FOR UPDATE 會照行順序取列鎖，兩張行序相反的
        // 跨倉單（SO1:[A,B] / SO2:[B,A]）互持等待即死鎖（40P01）。
        let mut affected_items: Vec<(Uuid, Uuid)> = Self::collect_affected_items(document, lines)
            .into_iter()
            .collect();
        affected_items.sort();
        for (warehouse_id, product_id) in &affected_items {
            Self::acquire_snapshot_lock(tx, *warehouse_id, *product_id).await?;
        }

        // R84-1：快照重算改成逐行進行（而非整單跑完才統一重算）。
        // 舊版在此迴圈跑完後才統一重算快照，導致同一張單裡兩行同 (倉,品)：
        // 第二行的 check_stock_available 讀到的仍是「本單處理前」的舊快照，
        // 兩行各自檢查都可能通過，實際加總卻已超賣（確定性重現，非低機率 race）。
        // update_inventory_snapshot 是從 stock_ledger 全量 SUM 重算（冪等），
        // 同一 (倉,品) 被多行命中時重複呼叫沒有正確性風險，只多一點 DB 往返。
        for line in lines {
            Self::process_single_line(tx, document, line).await?;
            for (warehouse_id, product_id) in Self::affected_items_for_line(document, line) {
                Self::update_inventory_snapshot(tx, warehouse_id, product_id).await?;
            }
        }

        Ok(())
    }

    /// 處理單一明細行的庫存變動
    async fn process_single_line(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
    ) -> Result<()> {
        match document.doc_type {
            DocType::GRN => Self::process_grn(tx, document, line).await?,
            DocType::PR => Self::process_return_out(tx, document, line).await?,
            DocType::SO => Self::process_sales_out(tx, document, line).await?,
            DocType::TR => Self::process_transfer(tx, document, line).await?,
            DocType::ADJ => Self::process_adjustment(tx, document, line).await?,
            // R84-13：原本此處還有 `SR | RTN => process_return_in`。SR/RTN 已從 DocType
            // 移除（業務上不存在銷貨退貨），此分支成為不可能路徑，一併清除。
            _ => {} // PO, STK 等不直接影響庫存
        }
        Ok(())
    }

    /// GRN 採購入庫
    async fn process_grn(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
    ) -> Result<()> {
        let warehouse_id = document
            .warehouse_id
            .ok_or_else(|| AppError::BusinessRule("Warehouse is required for GRN".to_string()))?;

        Self::create_ledger_entry(
            tx,
            LedgerEntryParams {
                warehouse_id,
                product_id: line.product_id,
                document,
                line,
                direction: StockDirection::In,
                qty: line.qty,
                unit_price: line.unit_price,
                storage_location_id: line.storage_location_id,
            },
        )
        .await?;

        if let Some(storage_location_id) = line.storage_location_id {
            Self::upsert_storage_location_inventory(
                tx,
                storage_location_id,
                line.product_id,
                line.qty,
                line.batch_no.clone(),
                line.expiry_date,
            )
            .await?;
        }
        Ok(())
    }

    /// PR 採購退貨（扣減庫存）
    /// 2026-05-20 (migration 069): 補上 storage_location_inventory 扣減 — 過去只動
    /// stock_ledger 不動 storage_location_inventory 造成 storage drift。
    /// R84-9（2026-07-23）：原有 `doc_label` 參數用於區分 PR 與 DO 的錯誤訊息；
    /// DO 移除後只剩單一呼叫端，參數一併清除。
    async fn process_return_out(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
    ) -> Result<()> {
        let warehouse_id = document
            .warehouse_id
            .ok_or_else(|| AppError::BusinessRule("Warehouse is required for PR".to_string()))?;
        Self::process_out_from_warehouse(tx, document, line, warehouse_id).await
    }

    /// SO 一段式銷貨出庫（migration 136）：倉庫**逐行**取自該行 `warehouse_id`
    /// （建/改單時由儲位反推回填），使一張 SO 可同時銷不同倉庫來源的貨。ledger 逐行跟隨儲位，
    /// 與 132/133 跨倉對帳不變式一致。
    async fn process_sales_out(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
    ) -> Result<()> {
        let warehouse_id = line.warehouse_id.ok_or_else(|| {
            AppError::BusinessRule("SO 明細缺少倉庫（應於建/改單時由儲位反推回填）".to_string())
        })?;
        Self::process_out_from_warehouse(tx, document, line, warehouse_id).await
    }

    /// 出庫扣帳共用 body（PR/DO 取表頭倉、SO 取逐行倉）：檢查庫存 → 寫 out 流水 → 扣儲位庫存。
    async fn process_out_from_warehouse(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
        warehouse_id: Uuid,
    ) -> Result<()> {
        Self::check_stock_available(tx, warehouse_id, line.product_id, line.qty).await?;
        Self::create_ledger_entry(
            tx,
            LedgerEntryParams {
                warehouse_id,
                product_id: line.product_id,
                document,
                line,
                direction: StockDirection::Out,
                qty: line.qty,
                unit_price: line.unit_price,
                storage_location_id: line.storage_location_id,
            },
        )
        .await?;

        if let Some(storage_location_id) = line.storage_location_id {
            Self::decrement_storage_location_inventory(
                tx,
                storage_location_id,
                line.product_id,
                line.qty,
                line.batch_no.clone(),
                line.expiry_date,
            )
            .await?;
        }
        Ok(())
    }

    /// TR 調撥
    /// 2026-05-20 (migration 069): 改用 per-line storage_location_from_id / to_id；
    /// 兩端 stock_ledger entry 各自記錄對應 storage_location_id；同步
    /// upsert storage_location_inventory（from 端 decrement、to 端 increment）。
    async fn process_transfer(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
    ) -> Result<()> {
        let from_warehouse = document.warehouse_from_id.ok_or_else(|| {
            AppError::BusinessRule("Source warehouse is required for transfer".to_string())
        })?;
        let to_warehouse = document.warehouse_to_id.ok_or_else(|| {
            AppError::BusinessRule("Target warehouse is required for transfer".to_string())
        })?;

        Self::check_stock_available(tx, from_warehouse, line.product_id, line.qty).await?;

        Self::create_ledger_entry(
            tx,
            LedgerEntryParams {
                warehouse_id: from_warehouse,
                product_id: line.product_id,
                document,
                line,
                direction: StockDirection::TransferOut,
                qty: line.qty,
                unit_price: None,
                storage_location_id: line.storage_location_from_id,
            },
        )
        .await?;
        Self::create_ledger_entry(
            tx,
            LedgerEntryParams {
                warehouse_id: to_warehouse,
                product_id: line.product_id,
                document,
                line,
                direction: StockDirection::TransferIn,
                qty: line.qty,
                unit_price: None,
                storage_location_id: line.storage_location_to_id,
            },
        )
        .await?;

        if let Some(from_loc) = line.storage_location_from_id {
            Self::decrement_storage_location_inventory(
                tx,
                from_loc,
                line.product_id,
                line.qty,
                line.batch_no.clone(),
                line.expiry_date,
            )
            .await?;
        }
        if let Some(to_loc) = line.storage_location_to_id {
            Self::upsert_storage_location_inventory(
                tx,
                to_loc,
                line.product_id,
                line.qty,
                line.batch_no.clone(),
                line.expiry_date,
            )
            .await?;
        }
        Ok(())
    }

    /// ADJ 調整
    async fn process_adjustment(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
    ) -> Result<()> {
        let warehouse_id = document.warehouse_id.ok_or_else(|| {
            AppError::BusinessRule("Warehouse is required for adjustment".to_string())
        })?;

        if line.qty > Decimal::ZERO {
            Self::create_ledger_entry(
                tx,
                LedgerEntryParams {
                    warehouse_id,
                    product_id: line.product_id,
                    document,
                    line,
                    direction: StockDirection::AdjustIn,
                    qty: line.qty,
                    unit_price: line.unit_price,
                    storage_location_id: line.storage_location_id,
                },
            )
            .await?;
        } else {
            Self::check_stock_available(tx, warehouse_id, line.product_id, -line.qty).await?;
            Self::create_ledger_entry(
                tx,
                LedgerEntryParams {
                    warehouse_id,
                    product_id: line.product_id,
                    document,
                    line,
                    direction: StockDirection::AdjustOut,
                    qty: -line.qty,
                    unit_price: line.unit_price,
                    storage_location_id: line.storage_location_id,
                },
            )
            .await?;
        }

        if let Some(storage_location_id) = line.storage_location_id {
            if line.qty > Decimal::ZERO {
                Self::upsert_storage_location_inventory(
                    tx,
                    storage_location_id,
                    line.product_id,
                    line.qty,
                    line.batch_no.clone(),
                    line.expiry_date,
                )
                .await?;
            } else if line.qty < Decimal::ZERO {
                // ADJ 出庫（qty < 0）改走 decrement，含 `on_hand_qty >= qty` 下限檢查。
                // 原本一律走 upsert 會以 `existing + 負值` 把單一儲位扣成負數
                // （甚至在無 row 時 INSERT 出負庫存），與 warehouse-level check 不一致。
                // qty == 0 為 no-op，不觸發多餘 UPDATE 與誤導性 drift warning（gemini review）。
                Self::decrement_storage_location_inventory(
                    tx,
                    storage_location_id,
                    line.product_id,
                    -line.qty,
                    line.batch_no.clone(),
                    line.expiry_date,
                )
                .await?;
            }
        }
        Ok(())
    }

    /// 收集所有涉及的 (warehouse_id, product_id) 組合
    fn collect_affected_items(
        document: &Document,
        lines: &[DocumentLine],
    ) -> std::collections::HashSet<(Uuid, Uuid)> {
        let mut items = std::collections::HashSet::new();
        for line in lines {
            items.extend(Self::affected_items_for_line(document, line));
        }
        items
    }

    /// 單一明細行涉及的 (warehouse_id, product_id) 組合（TR 調撥兩端各算一組）。
    /// 供 `process_document` 逐行重算快照使用（R84-1），邏輯與 `collect_affected_items`
    /// 逐行的判斷共用同一份，避免兩處 match 漂移。
    fn affected_items_for_line(document: &Document, line: &DocumentLine) -> Vec<(Uuid, Uuid)> {
        match document.doc_type {
            DocType::GRN | DocType::PR | DocType::ADJ => document
                .warehouse_id
                .map(|warehouse_id| vec![(warehouse_id, line.product_id)])
                .unwrap_or_default(),
            // SO 多倉銷貨：倉庫逐行取自該行 warehouse_id（migration 136），非表頭倉。
            DocType::SO => line
                .warehouse_id
                .map(|warehouse_id| vec![(warehouse_id, line.product_id)])
                .unwrap_or_default(),
            DocType::TR => match (document.warehouse_from_id, document.warehouse_to_id) {
                (Some(from_wh), Some(to_wh)) => {
                    vec![(from_wh, line.product_id), (to_wh, line.product_id)]
                }
                _ => vec![],
            },
            _ => vec![],
        }
    }

    /// 取得 (warehouse, product) 的 tx 級 advisory lock（重入安全，隨 tx commit/rollback 釋放）。
    ///
    /// CSO #2：snapshot 從整個 ledger 重算 SUM。兩張同 (warehouse, product) 單據並發
    /// 核准時，後 commit 者的 SUM 在 READ COMMITTED 下可能漏看前者剛 commit 的 ledger
    /// row → 短暫快照漂移。以 (warehouse, product) advisory lock 讓同產品的
    /// snapshot 重算序列化；process_document 於進場即依序取鎖，兼防跨倉多行死鎖。
    /// hashtext($n) 回傳 int4，對應 pg_advisory_xact_lock(int4, int4) overload。
    /// （勿用 hashtextextended：其回傳 bigint，會解析成不存在的
    ///  pg_advisory_xact_lock(bigint, bigint) → 42883，使所有影響庫存的核准失敗。）
    async fn acquire_snapshot_lock(
        tx: &mut Transaction<'_, Postgres>,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<()> {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1), hashtext($2))")
            .bind(warehouse_id.to_string())
            .bind(product_id.to_string())
            .execute(&mut **tx)
            .await?;
        Ok(())
    }

    /// 更新庫存快照 (核准單據後呼叫)
    ///
    /// 前置條件：呼叫端須已持有本 (warehouse, product) 的 advisory lock——唯一呼叫端
    /// `process_document` 於進場即依序取鎖（見 `acquire_snapshot_lock`），此處不重取
    /// 以省每品項一次 DB 往返。若未來新增裸呼叫路徑，必須自行先取鎖。
    async fn update_inventory_snapshot(
        tx: &mut Transaction<'_, Postgres>,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO inventory_snapshots (warehouse_id, product_id, on_hand_qty_base, avg_cost, updated_at)
            SELECT
                $1, $2,
                COALESCE(SUM(
                    CASE
                        WHEN direction IN ('in', 'transfer_in', 'adjust_in') THEN qty_base
                        WHEN direction IN ('out', 'transfer_out', 'adjust_out') THEN -qty_base
                        ELSE 0
                    END
                ), 0),
                -- 平均成本只看入向：out 行的 unit_cost 是**售價**（SO/DO 認列營收用），
                -- 混入會把快照 avg_cost 往售價方向拉高（與 find_avg_cost_by_product 同準則）。
                AVG(unit_cost) FILTER (WHERE direction IN ('in', 'transfer_in', 'adjust_in')),
                NOW()
            FROM stock_ledger
            WHERE warehouse_id = $1 AND product_id = $2
            ON CONFLICT (warehouse_id, product_id) DO UPDATE
            SET
                on_hand_qty_base = EXCLUDED.on_hand_qty_base,
                avg_cost = EXCLUDED.avg_cost,
                updated_at = NOW()
            "#,
        )
        .bind(warehouse_id)
        .bind(product_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// 建立庫存流水記錄
    async fn create_ledger_entry(
        tx: &mut Transaction<'_, Postgres>,
        params: LedgerEntryParams<'_>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO stock_ledger (
                id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no,
                line_id, direction, qty_base, unit_cost, batch_no, expiry_date,
                storage_location_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(params.warehouse_id)
        .bind(params.product_id)
        .bind(Utc::now())
        .bind(params.document.doc_type)
        .bind(params.document.id)
        .bind(&params.document.doc_no)
        .bind(params.line.id)
        .bind(params.direction)
        .bind(params.qty)
        .bind(params.unit_price)
        .bind(&params.line.batch_no)
        .bind(params.line.expiry_date)
        .bind(params.storage_location_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// 扣減儲位庫存 (PR/DO/SR/TR-out 出庫時使用；UPDATE-only，不 INSERT)。
    /// migration 069 起 PR/DO/SR/RTN/TR 都會呼叫，修復過去只增不減的 drift。
    ///
    /// 三種 rows_affected=0 情境的區分（CodeRabbit PR #467 Critical review）：
    /// 1. UPDATE 成功（rows_affected=1）→ Ok
    /// 2. row 存在但 on_hand_qty < qty → `AppError::BusinessRule` 拒絕，避免儲位庫存
    ///    被扣成負數（warehouse 級 `check_stock_available` 只守倉庫總量，無法防 single
    ///    location 不足）
    /// 3. row 不存在（drift baseline 之前的單據對應）→ warn 不 fail；warehouse 級已守
    async fn decrement_storage_location_inventory(
        tx: &mut Transaction<'_, Postgres>,
        storage_location_id: Uuid,
        product_id: Uuid,
        qty: Decimal,
        batch_no: Option<String>,
        expiry_date: Option<chrono::NaiveDate>,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE storage_location_inventory
            SET on_hand_qty = on_hand_qty - $3,
                updated_at = NOW()
            WHERE storage_location_id = $1
              AND product_id = $2
              AND COALESCE(batch_no, '') = COALESCE($4, '')
              AND COALESCE(expiry_date, '1900-01-01'::date) = COALESCE($5, '1900-01-01'::date)
              AND on_hand_qty >= $3
            "#,
        )
        .bind(storage_location_id)
        .bind(product_id)
        .bind(qty)
        .bind(&batch_no)
        .bind(expiry_date)
        .execute(&mut **tx)
        .await?;

        if result.rows_affected() == 0 {
            // 區分「row 存在但庫存不足」vs「row 不存在（baseline drift）」
            let existing_qty: Option<Decimal> = sqlx::query_scalar(
                r#"
                SELECT on_hand_qty
                FROM storage_location_inventory
                WHERE storage_location_id = $1
                  AND product_id = $2
                  AND COALESCE(batch_no, '') = COALESCE($3, '')
                  AND COALESCE(expiry_date, '1900-01-01'::date) = COALESCE($4, '1900-01-01'::date)
                "#,
            )
            .bind(storage_location_id)
            .bind(product_id)
            .bind(&batch_no)
            .bind(expiry_date)
            .fetch_optional(&mut **tx)
            .await?;

            match existing_qty {
                Some(on_hand) => {
                    // 庫存不足 — 拒絕；不允許單一儲位扣成負數
                    return Err(AppError::BusinessRule(format!(
                        "儲位庫存不足：location={}, product={}, on_hand={}, required={}, batch={:?}, expiry={:?}",
                        storage_location_id, product_id, on_hand, qty, batch_no, expiry_date,
                    )));
                }
                None => {
                    // baseline 缺 row — 與舊行為一致 warn 通過
                    tracing::warn!(
                        "storage_location_inventory decrement no-op: location={}, product={}, qty={}, batch={:?}, expiry={:?} \
                         — 可能 storage_inventory drift baseline 之前的單據對應，未影響 warehouse-level 庫存正確性",
                        storage_location_id, product_id, qty, batch_no, expiry_date,
                    );
                }
            }
        }
        Ok(())
    }

    /// 更新/新增儲位庫存 (GRN 入庫 / SR 退貨入庫 / TR-in 等使用)
    async fn upsert_storage_location_inventory(
        tx: &mut Transaction<'_, Postgres>,
        storage_location_id: Uuid,
        product_id: Uuid,
        qty: Decimal,
        batch_no: Option<String>,
        expiry_date: Option<chrono::NaiveDate>,
    ) -> Result<()> {
        WarehouseService::ensure_active_for_location_tx(tx, storage_location_id).await?;

        sqlx::query(
            r#"
            INSERT INTO storage_location_inventory (
                id, storage_location_id, product_id, on_hand_qty, batch_no, expiry_date, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (storage_location_id, product_id, COALESCE(batch_no, ''), COALESCE(expiry_date, '1900-01-01'::date))
            DO UPDATE SET
                on_hand_qty = storage_location_inventory.on_hand_qty + EXCLUDED.on_hand_qty,
                updated_at = NOW()
            "#
        )
        .bind(Uuid::new_v4())
        .bind(storage_location_id)
        .bind(product_id)
        .bind(qty)
        .bind(&batch_no)
        .bind(expiry_date)
        .execute(&mut **tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE storage_locations SET
                current_count = (
                    SELECT COUNT(DISTINCT product_id)
                    FROM storage_location_inventory
                    WHERE storage_location_id = $1
                ),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(storage_location_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// 檢查庫存是否足夠
    async fn check_stock_available(
        tx: &mut Transaction<'_, Postgres>,
        warehouse_id: Uuid,
        product_id: Uuid,
        required_qty: Decimal,
    ) -> Result<()> {
        // H2: FOR UPDATE 鎖定庫存列，防止並發扣減導致負庫存（Race Condition）
        let on_hand: Option<Decimal> = sqlx::query_scalar(
            r#"
            SELECT on_hand_qty_base
            FROM inventory_snapshots
            WHERE warehouse_id = $1 AND product_id = $2
            FOR UPDATE
            "#,
        )
        .bind(warehouse_id)
        .bind(product_id)
        .fetch_optional(&mut **tx)
        .await?;

        let on_hand = on_hand.unwrap_or(Decimal::ZERO);
        if on_hand < required_qty {
            let product_name: String =
                sqlx::query_scalar("SELECT name FROM products WHERE id = $1")
                    .bind(product_id)
                    .fetch_one(&mut **tx)
                    .await?;

            return Err(AppError::BusinessRule(format!(
                "Insufficient stock for product '{}'. Available: {}, Required: {}",
                product_name, on_hand, required_qty
            )));
        }
        Ok(())
    }

    /// 查詢庫存流水（使用 QueryBuilder 避免 format! 動態 SQL）
    pub async fn get_ledger(
        pool: &PgPool,
        query: &StockLedgerQuery,
    ) -> Result<Vec<StockLedgerDetail>> {
        use sqlx::QueryBuilder;

        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            r#"
            SELECT
                sl.id, sl.warehouse_id, w.name as warehouse_name,
                sl.product_id, p.sku as product_sku, p.name as product_name,
                sl.trx_date, sl.doc_type, sl.doc_id, sl.doc_no,
                sl.direction, sl.qty_base, sl.unit_cost,
                sl.batch_no, sl.expiry_date,
                NULL::numeric as running_balance,
                d.iacuc_no,
                sl.storage_location_id,
                loc.name as storage_location_name
            FROM stock_ledger sl
            INNER JOIN warehouses w ON sl.warehouse_id = w.id
            INNER JOIN products p ON sl.product_id = p.id
            LEFT JOIN documents d ON sl.doc_id = d.id
            LEFT JOIN storage_locations loc ON sl.storage_location_id = loc.id
            WHERE 1=1
            "#,
        );

        if let Some(warehouse_id) = query.warehouse_id {
            qb.push(" AND sl.warehouse_id = ");
            qb.push_bind(warehouse_id);
        }
        if let Some(product_id) = query.product_id {
            qb.push(" AND sl.product_id = ");
            qb.push_bind(product_id);
        }
        if let Some(batch_no) = &query.batch_no {
            qb.push(" AND sl.batch_no = ");
            qb.push_bind(batch_no);
        }
        if let Some(date_from) = query.date_from {
            qb.push(" AND sl.trx_date >= ");
            qb.push_bind(date_from);
        }
        if let Some(date_to) = query.date_to {
            qb.push(" AND sl.trx_date <= ");
            qb.push_bind(date_to);
        }
        if let Some(doc_type) = query.doc_type {
            qb.push(" AND sl.doc_type = ");
            qb.push_bind(doc_type);
        }

        let limit = query
            .limit
            .unwrap_or(100)
            .clamp(1, crate::constants::MAX_PAGE_SIZE);
        let offset = query.offset.unwrap_or(0).max(0);
        qb.push(" ORDER BY sl.trx_date DESC, sl.created_at DESC LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);

        let ledger = qb
            .build_query_as::<StockLedgerDetail>()
            .fetch_all(pool)
            .await?;
        Ok(ledger)
    }

    /// 批號完整生命週期查詢（R84-6）：時間軸 + 數量對帳，跨倉彙總
    /// 批號身分＝(product_id, batch_no, expiry_date) 三欄一組，見 ERP流程.md §6.2.2
    pub async fn get_lot_movements(
        pool: &PgPool,
        query: &LotMovementsQuery,
    ) -> Result<LotMovementsResponse> {
        let movements = sqlx::query_as::<_, LotMovement>(
            r#"
            SELECT
                sl.id, sl.warehouse_id, w.name as warehouse_name,
                sl.trx_date, sl.doc_type, sl.doc_id, sl.doc_no,
                sl.direction, sl.qty_base
            FROM stock_ledger sl
            INNER JOIN warehouses w ON sl.warehouse_id = w.id
            WHERE sl.product_id = $1
              AND sl.batch_no = $2
              AND sl.expiry_date IS NOT DISTINCT FROM $3
            ORDER BY sl.trx_date ASC, sl.created_at ASC
            "#,
        )
        .bind(query.product_id)
        .bind(&query.batch_no)
        .bind(query.expiry_date)
        .fetch_all(pool)
        .await?;

        let categorized = sqlx::query_as::<_, LotCategorizedTotals>(
            r#"
            SELECT
                -- R84-19：每一格都要收「同 doc_type 但方向相反」的列，否則沖銷鏡射列不落任何一格。
                -- 沖銷鏡射（`reverse_document_stock`）沿用原單的 `doc_type`、只把 `direction` 反轉，
                -- 所以單看 `doc_type='GRN' AND direction='in'` 會漏掉沖銷產生的 `GRN/out`：
                -- received 不會扣回、又不屬於 returned_to_supplier，derived_remaining 因此永遠高估，
                -- 對帳顯示 unbalanced 而查不出缺口在哪。
                --
                -- 以 `direction` 決定正負號（而非 JOIN documents 判斷是否沖銷單）是刻意的：
                -- stock_ledger 是流水帳，`GRN/out` 語意上就是「這筆入庫被退回去了」，
                -- 不論成因為何都該相減。這個寫法不依賴「沖銷是 GRN/out 的唯一來源」這個前提。
                COALESCE(SUM(CASE
                    WHEN sl.doc_type = 'GRN' AND sl.direction = 'in'  THEN sl.qty_base
                    WHEN sl.doc_type = 'GRN' AND sl.direction = 'out' THEN -sl.qty_base
                    ELSE 0
                END), 0) AS received,
                -- R84-13：SR/RTN 已封鎖新建（業務上不存在銷貨退貨），customer_returned 恆為 0
                -- （保留欄位維持前端契約不變）。
                0::NUMERIC AS customer_returned,
                COALESCE(SUM(CASE
                    WHEN sl.doc_type = 'SO' AND sl.direction = 'out' THEN sl.qty_base
                    WHEN sl.doc_type = 'SO' AND sl.direction = 'in'  THEN -sl.qty_base
                    ELSE 0
                END), 0) AS internal_consumed,
                COALESCE(SUM(CASE
                    WHEN sl.doc_type = 'PR' AND sl.direction = 'out' THEN sl.qty_base
                    WHEN sl.doc_type = 'PR' AND sl.direction = 'in'  THEN -sl.qty_base
                    ELSE 0
                END), 0) AS returned_to_supplier,
                COALESCE(SUM(CASE
                    WHEN sl.doc_type = 'ADJ' AND sl.direction = 'adjust_in' THEN sl.qty_base
                    WHEN sl.doc_type = 'ADJ' AND sl.direction = 'adjust_out' THEN -sl.qty_base
                    ELSE 0
                END), 0) AS adjusted_net
            FROM stock_ledger sl
            WHERE sl.product_id = $1
              AND sl.batch_no = $2
              AND sl.expiry_date IS NOT DISTINCT FROM $3
            "#,
        )
        .bind(query.product_id)
        .bind(&query.batch_no)
        .bind(query.expiry_date)
        .fetch_one(pool)
        .await?;

        let remaining: Decimal = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(on_hand_qty), 0)
            FROM storage_location_inventory
            WHERE product_id = $1
              AND batch_no = $2
              AND expiry_date IS NOT DISTINCT FROM $3
            "#,
        )
        .bind(query.product_id)
        .bind(&query.batch_no)
        .bind(query.expiry_date)
        .fetch_one(pool)
        .await?;

        let product = Self::lot_product_totals(pool, query.product_id).await?;
        let product_remaining_total: Decimal = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(on_hand_qty), 0)
            FROM storage_location_inventory
            WHERE product_id = $1
            "#,
        )
        .bind(query.product_id)
        .fetch_one(pool)
        .await?;

        let derived_remaining = categorized.received + categorized.customer_returned
            - categorized.internal_consumed
            - categorized.returned_to_supplier
            + categorized.adjusted_net;
        let balanced = remaining == derived_remaining;

        let reconciliation = LotReconciliation {
            received: categorized.received,
            customer_returned: categorized.customer_returned,
            internal_consumed: categorized.internal_consumed,
            returned_to_supplier: categorized.returned_to_supplier,
            adjusted_net: categorized.adjusted_net,
            remaining,
            derived_remaining,
            balanced,
            status: Self::lot_reconciliation_status(
                balanced,
                product.derived_total,
                product_remaining_total,
            ),
            product_derived_total: product.derived_total,
            product_remaining_total,
            unattributed_adjust_net: product.unattributed_adjust_net,
        };

        Ok(LotMovementsResponse {
            movements,
            reconciliation,
        })
    }

    /// R84-5 沖銷：把原單**實際寫入**的庫存效果原封不動地反向鏡射到沖銷單身上。
    ///
    /// 刻意**不**重跑 `process_single_line`——沖銷是「紅字沖銷」，鏡射的是原單當初真的寫了
    /// 什麼，而非「用現在的庫存狀態重算一次」（後者會因中間發生的其他異動而算出不同結果）。
    ///
    /// ⚠️ 兩本帳都要動，缺一不可（這正是 2026-05-20 migration 069 之前的 storage drift 成因，
    /// 詳見 `process_return_out` 上方註解與 R84-11 的調查）：
    /// - `stock_ledger`：逐筆寫方向相反、數量相同的新流水，掛在沖銷單下。
    /// - `storage_location_inventory`：**增量維護、不從 ledger 推導**，必須顯式反向增減。
    /// - `inventory_snapshots`：從 ledger 全量 SUM 重算（冪等），寫完鏡射列後重算即自動對齊。
    ///
    /// 反向扣減 SLI 時若庫存不足（例如原入庫的貨已被領用），
    /// `decrement_storage_location_inventory` 會回 `BusinessRule` 錯誤使整筆 tx rollback——
    /// 這是正確行為：東西已經不在了就不能假裝退回去。
    pub async fn reverse_document_stock(
        tx: &mut Transaction<'_, Postgres>,
        original: &Document,
        reversal: &Document,
    ) -> Result<()> {
        let rows = sqlx::query_as::<_, StockLedger>(
            "SELECT * FROM stock_ledger WHERE doc_id = $1 ORDER BY created_at",
        )
        .bind(original.id)
        .fetch_all(&mut **tx)
        .await?;

        if rows.is_empty() {
            return Ok(()); // 原單未影響庫存（如 PO / STK），無庫存面可沖銷
        }

        // 先依序取所有涉及 (倉,品) 的 advisory lock，與 process_document 同一套防死鎖策略
        let mut affected: Vec<(Uuid, Uuid)> = rows
            .iter()
            .map(|r| (r.warehouse_id, r.product_id))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        affected.sort();
        for (warehouse_id, product_id) in &affected {
            Self::acquire_snapshot_lock(tx, *warehouse_id, *product_id).await?;
        }

        for row in &rows {
            let reversed = Self::reverse_direction(row.direction);

            sqlx::query(
                r#"
                INSERT INTO stock_ledger (
                    id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no,
                    line_id, direction, qty_base, unit_cost, batch_no, expiry_date,
                    storage_location_id
                )
                VALUES ($1, $2, $3, NOW(), $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(row.warehouse_id)
            .bind(row.product_id)
            .bind(reversal.doc_type)
            .bind(reversal.id)
            .bind(&reversal.doc_no)
            .bind(row.line_id)
            .bind(reversed)
            .bind(row.qty_base)
            .bind(row.unit_cost)
            .bind(&row.batch_no)
            .bind(row.expiry_date)
            .bind(row.storage_location_id)
            .execute(&mut **tx)
            .await?;

            // SLI 反向：原本入庫的要扣回、原本出庫的要加回
            if let Some(location_id) = row.storage_location_id {
                if Self::is_inbound(row.direction) {
                    Self::decrement_storage_location_inventory(
                        tx,
                        location_id,
                        row.product_id,
                        row.qty_base,
                        row.batch_no.clone(),
                        row.expiry_date,
                    )
                    .await?;
                } else {
                    Self::upsert_storage_location_inventory(
                        tx,
                        location_id,
                        row.product_id,
                        row.qty_base,
                        row.batch_no.clone(),
                        row.expiry_date,
                    )
                    .await?;
                }
            }
        }

        for (warehouse_id, product_id) in &affected {
            Self::update_inventory_snapshot(tx, *warehouse_id, *product_id).await?;
        }

        Ok(())
    }

    /// 沖銷用的方向反轉；in↔out、transfer_in↔transfer_out、adjust_in↔adjust_out。
    fn reverse_direction(direction: StockDirection) -> StockDirection {
        match direction {
            StockDirection::In => StockDirection::Out,
            StockDirection::Out => StockDirection::In,
            StockDirection::TransferIn => StockDirection::TransferOut,
            StockDirection::TransferOut => StockDirection::TransferIn,
            StockDirection::AdjustIn => StockDirection::AdjustOut,
            StockDirection::AdjustOut => StockDirection::AdjustIn,
        }
    }

    /// 該方向是否為「使庫存增加」——決定沖銷時 SLI 該扣還是該加。
    fn is_inbound(direction: StockDirection) -> bool {
        matches!(
            direction,
            StockDirection::In | StockDirection::TransferIn | StockDirection::AdjustIn
        )
    }

    /// 品項層級（跨全部批號）推導總量與未帶批號的 ADJ 淨額（R84-6 對帳分級用）
    async fn lot_product_totals(pool: &PgPool, product_id: Uuid) -> Result<LotProductTotals> {
        let totals = sqlx::query_as::<_, LotProductTotals>(
            r#"
            SELECT
                -- R84-13：SR/RTN 已封鎖新建，移除原本的 `WHEN sl.doc_type IN ('SR', 'RTN')` 分支
                -- （業務上不存在銷貨退貨，該分支恆不成立）。
                -- R84-19：同 `get_lot_movements` 的 categorized——每個 doc_type 都要收方向相反的
                -- 鏡射列，否則沖銷後 derived_total 與實際庫存分岔，而 `lot_reconciliation_status`
                -- 正是用這個值判斷「批號不平但品項總量相符」的 AttributionOnly 分級，
                -- 漏收會讓一個已被沖銷抹平的批號被誤報成 Unbalanced。
                COALESCE(SUM(CASE WHEN sl.doc_type = 'GRN' AND sl.direction = 'in' THEN sl.qty_base
                    WHEN sl.doc_type = 'GRN' AND sl.direction = 'out' THEN -sl.qty_base
                    WHEN sl.doc_type = 'SO' AND sl.direction = 'out' THEN -sl.qty_base
                    WHEN sl.doc_type = 'SO' AND sl.direction = 'in' THEN sl.qty_base
                    WHEN sl.doc_type = 'PR' AND sl.direction = 'out' THEN -sl.qty_base
                    WHEN sl.doc_type = 'PR' AND sl.direction = 'in' THEN sl.qty_base
                    WHEN sl.doc_type = 'ADJ' AND sl.direction = 'adjust_in' THEN sl.qty_base
                    WHEN sl.doc_type = 'ADJ' AND sl.direction = 'adjust_out' THEN -sl.qty_base
                    ELSE 0 END), 0) AS derived_total,
                COALESCE(SUM(CASE
                    WHEN sl.doc_type = 'ADJ' AND sl.batch_no IS NULL AND sl.direction = 'adjust_in' THEN sl.qty_base
                    WHEN sl.doc_type = 'ADJ' AND sl.batch_no IS NULL AND sl.direction = 'adjust_out' THEN -sl.qty_base
                    ELSE 0
                END), 0) AS unattributed_adjust_net
            FROM stock_ledger sl
            WHERE sl.product_id = $1
            "#,
        )
        .bind(product_id)
        .fetch_one(pool)
        .await?;

        Ok(totals)
    }

    /// 對帳分級：批號不平時，先看品項總量是否相符再決定嚴重度。
    /// 歷史補帳（R62-2 / PHANTOMFIX）把品項總量補平卻未帶批號，只看批號會誤報成帳實不符。
    fn lot_reconciliation_status(
        balanced: bool,
        product_derived_total: Decimal,
        product_remaining_total: Decimal,
    ) -> LotReconciliationStatus {
        if balanced {
            LotReconciliationStatus::Balanced
        } else if product_derived_total == product_remaining_total {
            LotReconciliationStatus::AttributionOnly
        } else {
            LotReconciliationStatus::Unbalanced
        }
    }
}
