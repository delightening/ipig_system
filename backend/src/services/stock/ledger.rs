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

/// `decrement_storage_location_inventory` 的參數。
///
/// 用結構而非展開參數：補上 `doc_no`（通知要講是哪張單擋的）與 drift 收集器之後，
/// 展開寫法會有 8 個參數，超過 clippy `too_many_arguments` 的門檻 7，而本檔的
/// clippy 門檻是 `-D warnings`（`RULES_BACKEND.md` §8）。同檔的 `LedgerEntryParams`
/// 已是同一個做法。
struct DecrementParams<'a> {
    storage_location_id: Uuid,
    product_id: Uuid,
    qty: Decimal,
    batch_no: Option<String>,
    expiry_date: Option<chrono::NaiveDate>,
    /// 單號，只用於通知內容——現場看到「哪張單卡住」比看到 UUID 有用。
    doc_no: &'a str,
}

/// 儲位帳失真事件：`decrement_storage_location_inventory` 走到「儲位根本沒有 row」
/// 那條路徑時記一筆，交給 `process_document` 的呼叫端在 **commit 之後**通知倉管。
///
/// 為什麼要這個結構、而不是就地建通知：
/// 這條路徑是**成功路徑**——它靜默放行，tx 會繼續跑下去，而後面的會計過帳、
/// 狀態更新任何一步失敗都會把整張核准回滾。就地發通知會在那種情況下留下一則
/// 描述「已經沒有發生的事」的假警報。所以事件先收集，等 tx 真的 commit 再發。
///
/// 與「擋下領用」（`AppError::InsufficientStock`）互補：那條是失敗路徑、錯誤帶得出去；
/// 這條是成功路徑、沒有錯誤可搭，只能自己開一個通道。兩條合起來才涵蓋使用者要的
/// 「領不出來就通知」——因為缺 row 的品項**根本卡不住**，只看擋下來的那條
/// 會漏掉一整類情況（`ledger.rs` 的 None 分支註解記載了這個缺口的成因）。
#[derive(Debug, Clone)]
pub struct StorageDriftEvent {
    pub storage_location_id: Uuid,
    pub product_id: Uuid,
    pub qty: Decimal,
    pub batch_no: Option<String>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub doc_no: String,
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
    /// 處理單據核准後的庫存變動。
    ///
    /// 回傳的 `Vec<StorageDriftEvent>` 是**待通知事項**，不是錯誤：呼叫端必須在
    /// `tx.commit()` **之後**才據以發通知。空 Vec 是正常情況（絕大多數單據）。
    ///
    /// 為什麼不在這裡就把通知發掉：本函式全程在 tx 內，而 tx 之後還有會計過帳、
    /// 單據狀態更新等步驟，任何一步失敗都會回滾整張核准。就地發通知會在那種情況下
    /// 留下描述「已經沒有發生的事」的假警報。
    pub async fn process_document(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
    ) -> Result<Vec<StorageDriftEvent>> {
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
        let mut drift: Vec<StorageDriftEvent> = Vec::new();
        for line in lines {
            Self::process_single_line(tx, document, line, &mut drift).await?;
            for (warehouse_id, product_id) in Self::affected_items_for_line(document, line) {
                Self::update_inventory_snapshot(tx, warehouse_id, product_id).await?;
            }
        }

        Ok(drift)
    }

    /// 處理單一明細行的庫存變動
    async fn process_single_line(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
        drift: &mut Vec<StorageDriftEvent>,
    ) -> Result<()> {
        match document.doc_type {
            DocType::GRN => Self::process_grn(tx, document, line).await?,
            DocType::PR => Self::process_return_out(tx, document, line, drift).await?,
            DocType::SO => Self::process_sales_out(tx, document, line, drift).await?,
            DocType::TR => Self::process_transfer(tx, document, line, drift).await?,
            DocType::ADJ => Self::process_adjustment(tx, document, line, drift).await?,
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
        drift: &mut Vec<StorageDriftEvent>,
    ) -> Result<()> {
        let warehouse_id = document
            .warehouse_id
            .ok_or_else(|| AppError::BusinessRule("Warehouse is required for PR".to_string()))?;
        Self::process_out_from_warehouse(tx, document, line, warehouse_id, drift).await
    }

    /// SO 一段式銷貨出庫（migration 136）：倉庫**逐行**取自該行 `warehouse_id`
    /// （建/改單時由儲位反推回填），使一張 SO 可同時銷不同倉庫來源的貨。ledger 逐行跟隨儲位，
    /// 與 132/133 跨倉對帳不變式一致。
    async fn process_sales_out(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
        drift: &mut Vec<StorageDriftEvent>,
    ) -> Result<()> {
        let warehouse_id = line.warehouse_id.ok_or_else(|| {
            AppError::BusinessRule("SO 明細缺少倉庫（應於建/改單時由儲位反推回填）".to_string())
        })?;
        Self::process_out_from_warehouse(tx, document, line, warehouse_id, drift).await
    }

    /// 出庫扣帳共用 body（PR/DO 取表頭倉、SO 取逐行倉）：檢查庫存 → 寫 out 流水 → 扣儲位庫存。
    async fn process_out_from_warehouse(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
        warehouse_id: Uuid,
        drift: &mut Vec<StorageDriftEvent>,
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
                DecrementParams {
                    storage_location_id,
                    product_id: line.product_id,
                    qty: line.qty,
                    batch_no: line.batch_no.clone(),
                    expiry_date: line.expiry_date,
                    doc_no: &document.doc_no,
                },
                drift,
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
        drift: &mut Vec<StorageDriftEvent>,
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
                DecrementParams {
                    storage_location_id: from_loc,
                    product_id: line.product_id,
                    qty: line.qty,
                    batch_no: line.batch_no.clone(),
                    expiry_date: line.expiry_date,
                    doc_no: &document.doc_no,
                },
                drift,
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
        drift: &mut Vec<StorageDriftEvent>,
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
                    DecrementParams {
                        storage_location_id,
                        product_id: line.product_id,
                        qty: -line.qty,
                        batch_no: line.batch_no.clone(),
                        expiry_date: line.expiry_date,
                        doc_no: &document.doc_no,
                    },
                    drift,
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

    /// 組裝「儲位庫存不足」要回給前端的訊息。
    ///
    /// 與 DB 查詢分離成純函式，有兩個理由：
    /// 1. 這段文字是現場唯一會看到的東西，也是使用者裁定的「異狀時才盤」的觸發點
    ///    ——它該被測試釘住。埋在 async 的 DB 路徑裡就只能靠整合測試，
    ///    而整合測試需要獨立測試庫，本機跑不了（禁止對 prod 跑）。
    /// 2. 批號／效期的四種組合是純邏輯，容易寫錯，值得單獨測。
    fn insufficient_stock_message(
        loc_label: &str,
        prod_label: &str,
        batch_no: Option<&str>,
        expiry_date: Option<chrono::NaiveDate>,
        on_hand: Decimal,
        required: Decimal,
        uom: &str,
    ) -> String {
        let batch_hint = match (batch_no, expiry_date) {
            (Some(b), Some(e)) => format!("（批號 {b}、效期 {e}）"),
            (Some(b), None) => format!("（批號 {b}）"),
            (None, Some(e)) => format!("（效期 {e}）"),
            (None, None) => String::new(),
        };
        // 🔴 `normalize()` 去掉資料庫帶來的尾隨零（CodeRabbit 於 MR !3 指出）。
        //
        // `on_hand_qty` / `qty_base` 是 numeric 欄位，Decimal 忠實保留它的 scale，
        // 而 Display 又原樣印出——現場看到的會是「帳面只有 5.0000 雙」。
        // 這則訊息的用意是讓人看懂並判斷下一步，多四個零只會讓人以為系統壞了。
        //
        // ⚠️ 本檔既有的單元測試**結構上驗不出這件事**：它們用 `Decimal::from_i64`
        // 造值，scale 恆為 0。那些測試全綠不代表沒有這個 bug，只代表沒測到——
        // 故一併補了 `數量不得帶出資料庫的尾隨零`，改用 scaled Decimal 進去。
        let on_hand = on_hand.normalize();
        let required = required.normalize();
        format!(
            "{loc_label}的{prod_label}{batch_hint}帳面只有 {on_hand} {uom}，這張單要領 {required} {uom}。\
             若架上實際有貨，這是帳面與實體不符、不是缺貨——請先對該儲位開盤點單校正，再重新開單。\
             若架上確實沒有，才是真的缺貨，需要從其他倉庫調撥或採購。"
        )
    }

    /// 錯誤訊息用的儲位標示；查不到就退回 UUID。
    ///
    /// ⚠️ 這裡刻意吞掉查詢錯誤（`.ok()`），與「解析失敗不可靜默降級」不是同一件事：
    /// 這是**輔助資訊**，唯一用途是把錯誤訊息裡的 UUID 換成人看得懂的字。
    /// 讓它的失敗蓋掉呼叫端原本要回報的業務錯誤，等於用次要問題掩蓋主要問題。
    async fn storage_location_label(tx: &mut Transaction<'_, Postgres>, id: Uuid) -> String {
        let row: Option<(String, Option<String>)> =
            sqlx::query_as("SELECT sl.code, sl.name FROM storage_locations sl WHERE sl.id = $1")
                .bind(id)
                .fetch_optional(&mut **tx)
                .await
                .ok()
                .flatten();
        match row {
            Some((code, Some(name))) => format!("儲位「{code} {name}」"),
            Some((code, None)) => format!("儲位「{code}」"),
            None => format!("儲位 {id}"),
        }
    }

    /// 錯誤訊息用的品項標示與其基本單位。查不到就退回 UUID、單位留空。
    ///
    /// 帶 `base_uom` 是因為訊息裡要講「帳面只有 5 雙」——沒有單位的數字在
    /// 一盒五十雙的品項上會被誤讀成盒數，那正是這則訊息要避免的誤導。
    async fn product_label_and_uom(
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
    ) -> (String, String) {
        let row: Option<(String, String, String)> =
            sqlx::query_as("SELECT p.sku, p.name, p.base_uom FROM products p WHERE p.id = $1")
                .bind(id)
                .fetch_optional(&mut **tx)
                .await
                .ok()
                .flatten();
        match row {
            Some((sku, name, uom)) => (format!("「{sku} {name}」"), uom),
            None => (format!("品項 {id}"), String::new()),
        }
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
        params: DecrementParams<'_>,
        drift: &mut Vec<StorageDriftEvent>,
    ) -> Result<()> {
        let DecrementParams {
            storage_location_id,
            product_id,
            qty,
            batch_no,
            expiry_date,
            doc_no,
        } = params;
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
                    // 庫存不足 — 拒絕；不允許單一儲位扣成負數。
                    //
                    // 🔴 這則訊息會**原樣回給前端**（`error.rs` 的 BusinessRule → 422 + msg），
                    // 現場看到的就是它。措辭因此要指向正確的下一步。
                    //
                    // 舊版寫的是「儲位庫存不足：location=<UUID>, product=<UUID>, …」，有兩個問題：
                    // 1. 全是 UUID，現場看不出是哪個架子、哪個品項。
                    // 2. 「庫存不足」在現場的意思是「東西沒了，要叫貨」——但在
                    //    「儲藏室不做例行盤點」的規則下（見 014 migration），走到這裡最常見的
                    //    成因是**帳面與實體不符**，東西其實就在架上。照舊訊息去理解，
                    //    會有人下一張根本不需要的採購單。
                    //
                    // 這裡同時是使用者裁定的「異狀時才盤」的觸發點：擋下來的這一刻，
                    // 就是該去盤那個儲位的時候。訊息必須把這件事講出來。
                    //
                    // 查名稱的兩次查詢只在錯誤路徑執行，正常扣帳走不到，不影響熱路徑。
                    let loc_label = Self::storage_location_label(tx, storage_location_id).await;
                    let (prod_label, uom) = Self::product_label_and_uom(tx, product_id).await;
                    // 用 InsufficientStock 而非 BusinessRule：兩者對前端一模一樣（422 + 同訊息，
                    // `error.rs` 有測試釘住），差別是這個變體把儲位與品項帶得出去，
                    // 讓 `workflow.rs` 能在 tx 回滾之後通知倉管與操作者。
                    // 訊息寫在 tx 裡、通知發在 tx 外，是因為 rollback 會吃掉前者救不了後者。
                    return Err(AppError::InsufficientStock {
                        message: Self::insufficient_stock_message(
                            &loc_label,
                            &prod_label,
                            batch_no.as_deref(),
                            expiry_date,
                            on_hand,
                            qty,
                            &uom,
                        ),
                        storage_location_id,
                        product_id,
                        on_hand,
                        required: qty,
                    });
                }
                None => {
                    // baseline 缺 row — 與舊行為一致 warn 通過
                    //
                    // 🔴 **這是一條靜默放行的路徑，已知有問題，本次刻意不改行為。**
                    //
                    // 儲位沒有對應 row 時這裡只寫 log 就放行，等於該儲位可以無限透支
                    // （倉庫層級仍有 `check_stock_available` 守著，所以倉庫總量不會錯，
                    // 但儲位層級的帳就此失真且無人知曉）。
                    //
                    // 為什麼現在不修：改成拒絕之後，**目前正在靜默通過的領用會開始被擋**。
                    // prod 上若已累積不少缺 row 的儲位，上線當天可能大面積卡住現場。
                    // 使用者裁定的順序是「先量測、再決定上線策略」，量測要等帳號恢復
                    // 且經授權才能碰 prod DB。
                    //
                    // ⚠️ 這條與「儲藏室不做例行盤點」的新規則有直接衝突：SLI 的 row 靠
                    // 入庫與**盤點**建立／校正，不盤點會讓缺 row 的機率上升，於是這條從
                    // 「罕見的 baseline 遺留」變成常態路徑——而缺 row 的品項根本卡不住，
                    // 使用者設計的「領不出來才去盤」就永遠不會被觸發。
                    //
                    // 2026-09-09：**行為仍然不變**（照樣放行），但改為額外記一筆事件，
                    // 由 commit 之後的呼叫端通知倉管。使用者裁定「要通知，但不改放行行為」——
                    // 通知不會擋住任何人，所以沒有「上線當天大面積卡住現場」的風險，
                    // 而它本身就是上面說的那個量測：真實發生頻率會直接反映在通知量上。
                    //
                    // 這條補的正是下面那個缺口：缺 row 的品項卡不住，
                    // 「領不出來才去盤」對它們永遠不會觸發，於是儲位帳一路失真且無人知曉。
                    drift.push(StorageDriftEvent {
                        storage_location_id,
                        product_id,
                        qty,
                        batch_no: batch_no.clone(),
                        expiry_date,
                        doc_no: doc_no.to_string(),
                    });

                    // 下面的 `event` 標記是給量測用的：撈 `sli_decrement_missing_row`
                    // 就能統計發生頻率與涉及哪些儲位／品項。
                    tracing::warn!(
                        event = "sli_decrement_missing_row",
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
    /// `decrement_storage_location_inventory` 會回 `AppError::InsufficientStock` 使整筆 tx
    /// rollback——這是正確行為：東西已經不在了就不能假裝退回去。
    ///
    /// 與 `process_document` 一樣回傳待通知的 `StorageDriftEvent`：沖銷同樣會走到
    /// 「儲位沒有 row」那條靜默放行的路徑，沒有理由讓這條路徑上的帳失真比較不值得知道。
    /// 呼叫端須在 commit 之後才據以通知。
    pub async fn reverse_document_stock(
        tx: &mut Transaction<'_, Postgres>,
        original: &Document,
        reversal: &Document,
    ) -> Result<Vec<StorageDriftEvent>> {
        let rows = sqlx::query_as::<_, StockLedger>(
            "SELECT * FROM stock_ledger WHERE doc_id = $1 ORDER BY created_at",
        )
        .bind(original.id)
        .fetch_all(&mut **tx)
        .await?;

        if rows.is_empty() {
            return Ok(Vec::new()); // 原單未影響庫存（如 PO / STK），無庫存面可沖銷
        }

        let mut drift: Vec<StorageDriftEvent> = Vec::new();

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
                        DecrementParams {
                            storage_location_id: location_id,
                            product_id: row.product_id,
                            qty: row.qty_base,
                            batch_no: row.batch_no.clone(),
                            expiry_date: row.expiry_date,
                            doc_no: &reversal.doc_no,
                        },
                        &mut drift,
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

        Ok(drift)
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

#[cfg(test)]
mod insufficient_stock_message_tests {
    use super::*;
    use rust_decimal::prelude::FromPrimitive;

    fn dec(n: i64) -> Decimal {
        Decimal::from_i64(n).expect("i64 一定轉得成 Decimal")
    }

    fn msg(batch: Option<&str>, expiry: Option<chrono::NaiveDate>) -> String {
        StockService::insufficient_stock_message(
            "儲位「A-01 冷藏架」",
            "「CON-GLV-001 無菌手套」",
            batch,
            expiry,
            dec(5),
            dec(10),
            "雙",
        )
    }

    #[test]
    fn 無批號無效期時不出現括號() {
        let m = msg(None, None);
        assert!(!m.contains('（'), "不該有批號/效期括號：{m}");
        assert!(m.contains("儲位「A-01 冷藏架」的「CON-GLV-001 無菌手套」帳面只有 5 雙"));
        assert!(m.contains("這張單要領 10 雙"));
    }

    /// 🔴 數量不得帶出資料庫的 scale。
    ///
    /// `on_hand_qty` / `qty_base` 是 numeric 欄位，Decimal 保留 scale 而 Display 原樣印出，
    /// 於是現場看到「帳面只有 5.0000 雙」。
    ///
    /// **這條存在的理由是上面那些測試驗不到它**：`dec()` 走 `Decimal::from_i64`，
    /// scale 恆為 0，無論修不修都會通過。測試全綠只代表沒測到，不代表沒有 bug——
    /// 所以這裡刻意用 `Decimal::new(50000, 4)` 造出真實 DB 會給的形狀。
    /// （CodeRabbit 於 MR !3 指出，2026-09-09）
    #[test]
    fn 數量不得帶出資料庫的尾隨零() {
        let on_hand = Decimal::new(50_000, 4); // 5.0000
        let required = Decimal::new(100_000, 4); // 10.0000
        assert_eq!(
            on_hand.to_string(),
            "5.0000",
            "前提檢查：Decimal 確實會保留 scale，否則本測試證明不了任何事"
        );

        let m = StockService::insufficient_stock_message(
            "儲位「A-01 冷藏架」",
            "「CON-GLV-001 無菌手套」",
            None,
            None,
            on_hand,
            required,
            "雙",
        );
        assert!(m.contains("帳面只有 5 雙"), "{m}");
        assert!(m.contains("這張單要領 10 雙"), "{m}");
        assert!(
            !m.contains("5.0000"),
            "不該把 DB 的 scale 帶到現場文字：{m}"
        );
        assert!(!m.contains("10.0000"), "{m}");
    }

    #[test]
    fn 只有批號時只列批號() {
        let m = msg(Some("LOT-2026-A"), None);
        assert!(m.contains("（批號 LOT-2026-A）"), "{m}");
        assert!(!m.contains("效期"), "{m}");
    }

    #[test]
    fn 只有效期時只列效期() {
        let d = chrono::NaiveDate::from_ymd_opt(2027, 3, 31).expect("2027-03-31 是合法日期");
        let m = msg(None, Some(d));
        assert!(m.contains("（效期 2027-03-31）"), "{m}");
        assert!(!m.contains("批號"), "{m}");
    }

    #[test]
    fn 批號與效期都有時兩者都列() {
        let d = chrono::NaiveDate::from_ymd_opt(2027, 3, 31).expect("2027-03-31 是合法日期");
        let m = msg(Some("LOT-2026-A"), Some(d));
        assert!(m.contains("（批號 LOT-2026-A、效期 2027-03-31）"), "{m}");
    }

    /// 🔴 這條釘住的是本次修改的**目的**，不是格式。
    ///
    /// 舊訊息只說「儲位庫存不足」，現場會理解成「東西沒了，要叫貨」，
    /// 於是去下一張其實不需要的採購單——而在「儲藏室不做例行盤點」的規則下
    /// （見 014 migration），走到這裡最常見的成因是帳面與實體不符，東西就在架上。
    ///
    /// 這則訊息同時是使用者裁定的「異狀時才盤」的唯一觸發點。
    /// 有人把引導語刪成一句「庫存不足」，整套規則就失去入口——這條會擋下來。
    #[test]
    fn 訊息必須指向盤點而不是只說缺貨() {
        let m = msg(None, None);
        assert!(
            m.contains("帳面與實體不符"),
            "必須點出這可能不是缺貨而是帳實不符：{m}"
        );
        assert!(m.contains("盤點單"), "必須指出下一步是開盤點單：{m}");
        assert!(
            m.contains("才是真的缺貨"),
            "必須保留「架上真的沒有才是缺貨」這條分支，否則會變成一律當帳實不符：{m}"
        );
    }

    /// 數量必須帶單位。一盒五十雙的品項上，沒有單位的「5」會被讀成 5 盒。
    #[test]
    fn 數量必須帶基本單位() {
        let m = StockService::insufficient_stock_message(
            "儲位「B-02」",
            "「MED-001 範例藥品」",
            None,
            None,
            dec(2),
            dec(3),
            "盒",
        );
        assert!(m.contains("帳面只有 2 盒"), "{m}");
        assert!(m.contains("要領 3 盒"), "{m}");
    }

    /// 查不到名稱時退回 UUID 字樣，訊息本身仍要成立（不會變成空白或 panic）。
    #[test]
    fn 名稱查不到時訊息仍完整() {
        let m = StockService::insufficient_stock_message(
            "儲位 0f8b2c1e-0000-0000-0000-000000000000",
            "品項 3a7d9e11-0000-0000-0000-000000000000",
            None,
            None,
            dec(0),
            dec(1),
            "",
        );
        assert!(m.contains("帳面只有 0"), "{m}");
        assert!(m.contains("盤點單"), "{m}");
    }
}
