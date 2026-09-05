use rust_decimal::Decimal;

use crate::time;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    models::{
        DocStatus, DocType, Document, DocumentLine, DocumentWithLines, PoReceiptItem,
        PoReceiptStatus,
    },
    AppError, Result,
};

/// 採購單明細查詢結果（避免 clippy::type_complexity）
#[derive(FromRow)]
struct PoLineRow {
    product_id: Uuid,
    sku: String,
    name: String,
    base_uom: String,
    uom: String,
    unit_price: Option<Decimal>,
    qty: Decimal,
    /// 本行 `uom` 對 `base_uom` 的換算率，恆 > 0（SQL 端已把 NULL / 非正數收斂成 1）。
    factor_to_base: Decimal,
}

use super::DocumentService;

/// 「已被沖銷的 GRN 不算數」的排除述詞，供本檔**四處** `received` 算式與
/// `services/notification/erp.rs::notify_po_pending_receipt`（R84-18）共用。
///
/// R84-5 沖銷：原單被沖銷後**仍是 `approved`**，不排除的話已沖銷的量會一直被算進去，
/// 「打錯 → 沖銷 → 重開正確的」這條唯一的補救路徑走不完。
///
/// `documents` **沒有** `reversed_by_doc_id` 欄位（見 `models/document.rs` 該欄說明），
/// 沖銷關係只能由沖銷單的 `reverses_doc_id` 反查。
///
/// ⚠️ **四處的述詞必須完全一致**，所以收斂成單一巨集而不是各自複製一份：任一處漏掉都會
/// 產生「入庫進度顯示 pending、守衛卻仍擋」或「按鈕出現、按下去卻說已全部入庫」這類
/// 極難查的症狀（2026-08-27 第一版修法只補了其中兩處，補救路徑仍然斷）。
///
/// 使用前提：SQL 裡 `documents` 當 GRN 用的那個別名必須是 `g`。
///
/// `#[macro_export]`（R84-18 加）：跨檔呼叫端寫 `crate::exclude_reversed_grn!()`。
/// 提升可見性而不是讓第五個呼叫點自己抄一份述詞——上面那段警告的就是這件事，
/// 而未入庫提醒（`notification/erp.rs`）問的雖然是「有沒有有效 GRN」而非「入了多少量」，
/// **判準必須同源**：兩邊對「這張 GRN 還算不算數」的答案不一致，就會出現
/// 「提醒說未入庫、進度卻顯示已完成」這種互相打臉的畫面。
#[macro_export]
macro_rules! exclude_reversed_grn {
    () => {
        "
              AND NOT EXISTS (
                  SELECT 1 FROM documents r
                  WHERE r.reverses_doc_id = g.id AND r.status = 'approved'
              )
        "
    };
}

// ## base_uom 一律 factor 1：本檔每個 `product_uom_conversions` 的 JOIN 都要帶
// `AND <conv>.uom <> <products>.base_uom`
//
// Rust 端的 `ProductUomTable::factor`（`services/stock/uom.rs`）**先判斷 `uom == base_uom`
// 就回 1，根本不查換算表**。SQL 端若照著 `c.uom = dl.uom` 去 join，遇到舊資料裡
// `uom = base_uom, factor_to_base = 50` 這種同名列就會撿到 50——同一張單，兩邊算出的
// 數量差 50 倍。
//
// 具體後果不只是顯示錯：`assert_no_over_receipt` 會把**完全正確的收貨**judge 成超收而擋下
// （SQL 端 received 膨脹 50 倍），`create_additional_grn` 的剩餘量也跟著錯。
//
// 這個不對稱是本 PR 引進的——本 PR 之前 grn.rs 的 `SUM(qty)` 完全不看換算表，沒有兩端之分。
// 同一個根因在盤點路徑上更兇（`document/stocktake.rs::counting_uom` 會開出把貨架清空的
// 盤虧 ADJ），該處已用 `pack_unit != base_uom` 擋掉。
//
// ⚠️ 與 `exclude_reversed_grn!` 同理：**本檔每一處都要帶，漏一處就是一個算得出不同答案的
// 路徑**。真正的根治是禁止這種列存在（DB 約束），但那需要 migration，不在本 PR 範圍。

/// 依據已入庫量與採購量決定入庫狀態字串。
pub(super) fn receipt_status_label(
    total_received: rust_decimal::Decimal,
    total_ordered: rust_decimal::Decimal,
) -> &'static str {
    if total_received == rust_decimal::Decimal::ZERO {
        "pending"
    } else if total_received < total_ordered {
        "partial"
    } else {
        "complete"
    }
}

impl DocumentService {
    /// 從採購單建立入庫單（部分入庫 / 手動建立用）
    pub async fn create_additional_grn(
        pool: &PgPool,
        po_id: Uuid,
        created_by: Uuid,
    ) -> Result<DocumentWithLines> {
        // 檢查採購單狀態
        let po = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1 AND doc_type = 'PO'",
        )
        .bind(po_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".to_string()))?;

        if po.status != DocStatus::Approved {
            return Err(AppError::BusinessRule(
                "Purchase order must be approved".to_string(),
            ));
        }

        // 取得採購單明細
        let po_lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        // 取得已入庫數量。兩件事疊在同一個算式上：
        //
        // (1) 排除已被沖銷的 GRN（#39）：原單被沖銷後仍是 `approved`，不排除的話已沖銷的量
        //     會一直被算進去，這裡會回「All items have been received」而開不出更正單。
        // (2) 換算成 base_uom 再加總（本 PR）：PO 與 GRN 對同一品項可以填不同單位
        //     （採購論箱、入庫論盒），直接 SUM(qty) 會把箱與盒相加。查無換算列即代表該行
        //     就是 base_uom（factor 1）——這個推論由 `assert_lines_uom_defined` 保證。
        //
        // 別名必須維持 `g` / `gl`：`exclude_reversed_grn!` 的述詞寫死了 `g.id`（見該巨集）。
        //
        // `AND c.uom <> gp.base_uom`：見檔頭「base_uom 一律 factor 1」。
        let received_qty: Vec<(Uuid, Decimal)> = sqlx::query_as(concat!(
            r#"
            SELECT gl.product_id,
                   COALESCE(SUM(gl.qty * CASE WHEN c.factor_to_base > 0
                                              THEN c.factor_to_base ELSE 1 END), 0) as received
            FROM documents g
            JOIN document_lines gl ON g.id = gl.document_id
            JOIN products gp ON gp.id = gl.product_id
            LEFT JOIN product_uom_conversions c
                   ON c.product_id = gl.product_id AND c.uom = gl.uom
                  AND c.uom <> gp.base_uom
            WHERE g.source_doc_id = $1
              AND g.doc_type = 'GRN'
              AND g.status = 'approved'
            "#,
            exclude_reversed_grn!(),
            r#"
            GROUP BY gl.product_id
            "#
        ))
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        let received_map: std::collections::HashMap<Uuid, Decimal> =
            received_qty.into_iter().collect();

        // 逐 PO 明細行的換算率（同一品項可能在兩行用不同單位，故以 line id 為 key）。
        // `AND c.uom <> p.base_uom`：見檔頭「base_uom 一律 factor 1」。
        let po_factors: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT dl.id, CASE WHEN c.factor_to_base > 0 THEN c.factor_to_base ELSE 1 END
            FROM document_lines dl
            JOIN products p ON p.id = dl.product_id
            LEFT JOIN product_uom_conversions c
                   ON c.product_id = dl.product_id AND c.uom = dl.uom
                  AND c.uom <> p.base_uom
            WHERE dl.document_id = $1
            "#,
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;
        let po_factor_map: std::collections::HashMap<Uuid, Decimal> =
            po_factors.into_iter().collect();

        // 計算剩餘數量：兩邊都換到 base_uom 相減，再除回本行單位開立 GRN。
        let remaining_lines: Vec<_> = po_lines
            .iter()
            .filter_map(|line| {
                let factor = po_factor_map.get(&line.id).copied().unwrap_or(Decimal::ONE);
                let received_base = received_map
                    .get(&line.product_id)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let remaining_base = line.qty * factor - received_base;
                if remaining_base > Decimal::ZERO {
                    Some((line.clone(), remaining_base / factor))
                } else {
                    None
                }
            })
            .collect();

        if remaining_lines.is_empty() {
            return Err(AppError::BusinessRule(
                "All items have been received".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

        // 產生入庫單編號 (統一格式：YYMMDD-{02})
        let doc_no = Self::generate_doc_no(&mut tx, DocType::GRN).await?;

        // 建立入庫單
        let grn_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO documents (
                id, doc_type, doc_no, status, warehouse_id, partner_id, doc_date,
                source_doc_id, remark, created_by, created_at, updated_at
            )
            VALUES ($1, 'GRN', $2, 'draft', $3, $4, $5, $6, $7, $8, NOW(), NOW())
            "#,
        )
        .bind(grn_id)
        .bind(&doc_no)
        .bind(po.warehouse_id)
        .bind(po.partner_id)
        .bind(time::today_taiwan_naive())
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
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(grn_id)
            .bind((idx + 1) as i32)
            .bind(line.product_id)
            .bind(*remaining) // 預設為剩餘數量
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

    /// Medium-2 (#290相關)：GRN 核准守衛 — 確保核准後「同品項累計已核准入庫量」
    /// 不超過採購量（防超量入庫）。
    ///
    /// 在核准 tx 內、GRN 已標 `approved` 之後呼叫 → `received` 已含本張 GRN。違反即
    /// 回 Conflict 使整個核准 tx 回滾。
    ///
    /// 併發（bot review #629）：先對 PO 行 `FOR UPDATE`，序列化同一 PO 的並發 GRN 核准——
    /// Read Committed 下兩個並發核准互看不到對方未 commit 的 `approved`，否則皆通過守衛 → 超收。
    /// 比對：以 UNION ALL 彙整「PO 採購量」與「已核准 GRN 入庫量」再 GROUP BY 比較，連
    /// 「PO 未採購品項卻被入庫」（ordered=0、received>0）也能攔截（原 LEFT JOIN 以 PO 明細
    /// 為主體會漏掉）。
    /// 註：不在 `update_po_receipt_status`（被 recompute-all 迴圈共用）內 raise，避免對既有
    /// legacy 超收資料炸錯。
    ///
    /// R84-5 沖銷（2026-08-27 修）：`received` 必須排除**已被沖銷的 GRN**。原單被沖銷後
    /// 仍是 `approved`，不排除的話補開的更正單會被本守衛擋下（「入庫數量超過採購量」），
    /// 使「打錯 → 沖銷 → 重開」這條補救路徑走不完。
    /// 排除述詞由 `exclude_reversed_grn!` 巨集提供，本檔四處 `received` 算式共用，
    /// 見該巨集的說明。
    pub(crate) async fn ensure_no_over_receipt(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        po_id: Uuid,
    ) -> Result<()> {
        // 序列化同一 PO 的並發 GRN 核准（鎖 PO 行直到本 tx 結束）
        sqlx::query("SELECT 1 FROM documents WHERE id = $1 FOR UPDATE")
            .bind(po_id)
            .execute(&mut **tx)
            .await?;

        let over: Option<(Uuid,)> = sqlx::query_as(concat!(
            r#"
            SELECT product_id
            FROM (
                SELECT pl.product_id,
                       SUM(pl.qty * CASE WHEN pc.factor_to_base > 0 THEN pc.factor_to_base ELSE 1 END) AS ordered,
                       0::numeric AS received
                FROM document_lines pl
                JOIN products pp ON pp.id = pl.product_id
                LEFT JOIN product_uom_conversions pc
                       ON pc.product_id = pl.product_id AND pc.uom = pl.uom
                      AND pc.uom <> pp.base_uom
                WHERE pl.document_id = $1
                GROUP BY pl.product_id
                UNION ALL
                SELECT gl.product_id,
                       0::numeric AS ordered,
                       SUM(gl.qty * CASE WHEN gc.factor_to_base > 0 THEN gc.factor_to_base ELSE 1 END) AS received
                FROM documents g
                JOIN document_lines gl ON g.id = gl.document_id
                JOIN products gp ON gp.id = gl.product_id
                LEFT JOIN product_uom_conversions gc
                       ON gc.product_id = gl.product_id AND gc.uom = gl.uom
                      AND gc.uom <> gp.base_uom
                WHERE g.source_doc_id = $1 AND g.doc_type = 'GRN' AND g.status = 'approved'
            "#,
            exclude_reversed_grn!(),
            r#"
                GROUP BY gl.product_id
            ) t
            GROUP BY product_id
            HAVING SUM(received) > SUM(ordered)
            LIMIT 1
            "#
        ))
        .bind(po_id)
        .fetch_optional(&mut **tx)
        .await?;

        if over.is_some() {
            return Err(AppError::Conflict(
                "入庫數量超過採購量，無法核准（請檢查是否重複入庫）".to_string(),
            ));
        }
        Ok(())
    }

    /// GRN 核准後，重新計算並回寫 PO 的 receipt_status。
    ///
    /// R84-5 沖銷（2026-08-27 修）：也由 `approve_reversal` 呼叫，使 GRN 被沖銷後
    /// PO 的入庫進度回退。`received` 排除已被沖銷的 GRN——原單沖銷後仍是 `approved`，
    /// 不排除的話 `receipt_status` 永遠停在 `complete`，前端「採購入庫」按鈕消失
    /// （`DocumentDetailPage.tsx` 要求 pending/partial），使用者開不出更正單。
    /// 排除述詞由 `exclude_reversed_grn!` 巨集提供，本檔四處共用，見該巨集的說明。
    pub(crate) async fn update_po_receipt_status(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        po_id: Uuid,
    ) -> Result<()> {
        let row: (Decimal, Decimal) = sqlx::query_as(concat!(
            r#"
            SELECT
                COALESCE(SUM(pl.qty * CASE WHEN pc.factor_to_base > 0 THEN pc.factor_to_base ELSE 1 END), 0) AS total_ordered,
                COALESCE((
                    SELECT SUM(gl.qty * CASE WHEN gc.factor_to_base > 0 THEN gc.factor_to_base ELSE 1 END)
                    FROM documents g
                    JOIN document_lines gl ON g.id = gl.document_id
                    JOIN products gp ON gp.id = gl.product_id
                    LEFT JOIN product_uom_conversions gc
                           ON gc.product_id = gl.product_id AND gc.uom = gl.uom
                          AND gc.uom <> gp.base_uom
                    WHERE g.source_doc_id = $1
                      AND g.doc_type = 'GRN'
                      AND g.status = 'approved'
            "#,
            exclude_reversed_grn!(),
            r#"
                ), 0) AS total_received
            FROM document_lines pl
            JOIN products pp ON pp.id = pl.product_id
            LEFT JOIN product_uom_conversions pc
                   ON pc.product_id = pl.product_id AND pc.uom = pl.uom
                  AND pc.uom <> pp.base_uom
            WHERE pl.document_id = $1
            "#
        ))
        .bind(po_id)
        .fetch_one(&mut **tx)
        .await?;

        let status = receipt_status_label(row.1, row.0);

        sqlx::query("UPDATE documents SET receipt_status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status)
            .bind(po_id)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }

    /// 取得採購單入庫狀態
    pub async fn get_po_receipt_status(pool: &PgPool, po_id: Uuid) -> Result<PoReceiptStatus> {
        let po = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1 AND doc_type = 'PO'",
        )
        .bind(po_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".to_string()))?;

        // 取得採購單明細。`AND c.uom <> p.base_uom`：見檔頭「base_uom 一律 factor 1」。
        let po_lines: Vec<PoLineRow> = sqlx::query_as(
            r#"
            SELECT dl.product_id, p.sku, p.name, p.base_uom, dl.uom, dl.unit_price, dl.qty,
                   CASE WHEN c.factor_to_base > 0 THEN c.factor_to_base ELSE 1 END AS factor_to_base
            FROM document_lines dl
            JOIN products p ON dl.product_id = p.id
            LEFT JOIN product_uom_conversions c
                   ON c.product_id = dl.product_id AND c.uom = dl.uom
                  AND c.uom <> p.base_uom
            WHERE dl.document_id = $1
            ORDER BY dl.line_no
            "#,
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        // 取得已入庫數量：排除已被沖銷的 GRN（否則沖銷後前端 GRN 表單仍顯示 received_qty
        // 含已沖銷量、remaining_qty 為 0），並換算成 base_uom 再加總——理由同
        // `create_additional_grn` 那一處，兩者是同一個 received 語意的兩個呼叫點。
        let received: Vec<(Uuid, Decimal)> = sqlx::query_as(concat!(
            r#"
            SELECT gl.product_id,
                   COALESCE(SUM(gl.qty * CASE WHEN c.factor_to_base > 0
                                              THEN c.factor_to_base ELSE 1 END), 0)
            FROM documents g
            JOIN document_lines gl ON g.id = gl.document_id
            JOIN products gp ON gp.id = gl.product_id
            LEFT JOIN product_uom_conversions c
                   ON c.product_id = gl.product_id AND c.uom = gl.uom
                  AND c.uom <> gp.base_uom
            WHERE g.source_doc_id = $1
              AND g.doc_type = 'GRN'
              AND g.status = 'approved'
            "#,
            exclude_reversed_grn!(),
            r#"
            GROUP BY gl.product_id
            "#
        ))
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        let received_map: std::collections::HashMap<Uuid, Decimal> = received.into_iter().collect();

        // 每行的數量以**該行的單位**呈現（採購論箱就顯示箱），但跨行加總與 status 判定
        // 一律走 base_uom——否則 3 箱 + 2 盒會被加成 5。
        let mut ordered_base = Decimal::ZERO;
        let mut received_base_total = Decimal::ZERO;
        let items: Vec<PoReceiptItem> = po_lines
            .into_iter()
            .map(|row| {
                let received_line_base = received_map
                    .get(&row.product_id)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                ordered_base += row.qty * row.factor_to_base;
                received_base_total += received_line_base;
                let received_qty = received_line_base / row.factor_to_base;
                PoReceiptItem {
                    product_id: row.product_id,
                    product_sku: row.sku,
                    product_name: row.name,
                    base_uom: row.base_uom,
                    uom: row.uom,
                    unit_price: row.unit_price,
                    ordered_qty: row.qty,
                    received_qty,
                    remaining_qty: row.qty - received_qty,
                }
            })
            .collect();

        let status = receipt_status_label(received_base_total, ordered_base).to_string();

        Ok(PoReceiptStatus {
            po_id,
            po_no: po.doc_no,
            status,
            items,
        })
    }

    /// 重新計算所有已核准 PO 的入庫狀態
    pub async fn recalculate_all_po_receipt_status(pool: &PgPool) -> Result<i64> {
        // 取得所有已核准的 PO
        let pos: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM documents WHERE doc_type = 'PO' AND status = 'approved'",
        )
        .fetch_all(pool)
        .await?;

        let mut tx = pool.begin().await?;
        let mut count = 0i64;
        for (po_id,) in &pos {
            Self::update_po_receipt_status(&mut tx, *po_id)
                .await
                .map_err(|e| {
                    tracing::error!("Failed recalculating PO {po_id}: {e}");
                    e
                })?;
            count += 1;
        }
        tx.commit().await?;

        tracing::info!("Recalculated receipt_status for {} POs", count);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::receipt_status_label;
    use rust_decimal::Decimal;

    // --- receipt_status_label ---

    #[test]
    fn test_receipt_status_pending_when_zero_received() {
        assert_eq!(
            receipt_status_label(Decimal::ZERO, Decimal::new(100, 0)),
            "pending"
        );
    }

    #[test]
    fn test_receipt_status_partial_when_some_received() {
        assert_eq!(
            receipt_status_label(Decimal::new(50, 0), Decimal::new(100, 0)),
            "partial"
        );
    }

    #[test]
    fn test_receipt_status_complete_when_fully_received() {
        assert_eq!(
            receipt_status_label(Decimal::new(100, 0), Decimal::new(100, 0)),
            "complete"
        );
    }
}
