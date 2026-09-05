use std::collections::HashMap;

use rust_decimal::Decimal;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{models::DocumentLine, AppError, Result};

use super::StockService;

/// 單一品項的換算查詢結果。base_uom 本身不存在於 `product_uom_conversions`，factor 恆為 1。
#[derive(sqlx::FromRow)]
struct UomFactorRow {
    product_id: Uuid,
    uom: String,
    factor_to_base: Decimal,
}

/// 一個品項的基本單位，與它可接受的所有換算單位。
pub(crate) struct ProductUomTable {
    base_uom: String,
    factors: HashMap<String, Decimal>,
}

impl ProductUomTable {
    /// 取 `uom` 對 base_uom 的換算率。base_uom 自身恆為 1；未定義的單位回 `None`。
    pub(crate) fn factor(&self, uom: &str) -> Option<Decimal> {
        if uom == self.base_uom {
            return Some(Decimal::ONE);
        }
        self.factors.get(uom).copied()
    }

    /// 該品項所有可接受的單位，排序後供錯誤訊息列舉（讓使用者知道能填什麼）。
    pub(crate) fn accepted_uoms(&self) -> Vec<&str> {
        let mut uoms: Vec<&str> = std::iter::once(self.base_uom.as_str())
            .chain(self.factors.keys().map(String::as_str))
            .collect();
        uoms.sort_unstable();
        uoms
    }
}

impl StockService {
    /// 批次載入多個品項的單位換算表（一次兩查，避免逐明細行 N+1）。
    pub(crate) async fn load_uom_tables(
        tx: &mut Transaction<'_, Postgres>,
        product_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, ProductUomTable>> {
        if product_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let bases: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, base_uom FROM products WHERE id = ANY($1)")
                .bind(product_ids)
                .fetch_all(&mut **tx)
                .await?;

        let mut tables: HashMap<Uuid, ProductUomTable> = bases
            .into_iter()
            .map(|(id, base_uom)| {
                (
                    id,
                    ProductUomTable {
                        base_uom,
                        factors: HashMap::new(),
                    },
                )
            })
            .collect();

        let rows: Vec<UomFactorRow> = sqlx::query_as(
            "SELECT product_id, uom, factor_to_base FROM product_uom_conversions \
             WHERE product_id = ANY($1)",
        )
        .bind(product_ids)
        .fetch_all(&mut **tx)
        .await?;

        for row in rows {
            // factor <= 0 是壞資料：乘上去會讓入庫變出庫（負數）或整行歸零而靜默漏帳。
            // 略過即讓該單位落回「未定義」，由 to_base_lines / assert_lines_uom_defined 擋成 400，
            // 而不是拿一個會算錯的數字繼續跑。
            if row.factor_to_base <= Decimal::ZERO {
                continue;
            }
            if let Some(table) = tables.get_mut(&row.product_id) {
                table.factors.insert(row.uom, row.factor_to_base);
            }
        }

        Ok(tables)
    }

    /// 把單據明細的 (qty, uom) 一次換算成以 base_uom 計的明細。
    ///
    /// `stock_ledger.qty_base` 與 `inventory_snapshots.on_hand_qty_base` 的單位**永遠是
    /// `products.base_uom`**；`unit_cost` 則是「每一個 base_uom 的成本」——
    /// `update_inventory_snapshot` 直接 `AVG(unit_cost)` 當平均成本。因此 qty 乘上 factor 的
    /// 同時，`unit_price` 必須除以同一個 factor，否則以盒（factor 50）入庫會把 avg_cost
    /// 放大 50 倍。
    ///
    /// 換算表沒有該單位就回 400，**不做「當作 1」的靜默降級**——那正是本函式要根治的錯帳：
    /// 舊版把 `line.qty` 原封不動寫進 `qty_base`，一盒（50 雙）與一雙會被直接相加。
    ///
    /// 回傳明細的 `uom` 已改寫為 base_uom，因此對已換算過的明細再跑一次是冪等的
    /// （盤點差異產生的 ADJ 會先後經過本函式兩次）。
    pub(crate) async fn to_base_lines(
        tx: &mut Transaction<'_, Postgres>,
        lines: &[DocumentLine],
    ) -> Result<Vec<DocumentLine>> {
        if lines.is_empty() {
            return Ok(Vec::new());
        }

        let mut product_ids: Vec<Uuid> = lines.iter().map(|l| l.product_id).collect();
        product_ids.sort_unstable();
        product_ids.dedup();
        let tables = Self::load_uom_tables(tx, &product_ids).await?;

        lines
            .iter()
            .map(|line| {
                let table = tables.get(&line.product_id).ok_or_else(|| {
                    AppError::NotFound(format!(
                        "第 {} 行：找不到品項 {}",
                        line.line_no, line.product_id
                    ))
                })?;
                let factor = table.factor(&line.uom).ok_or_else(|| {
                    AppError::Validation(format!(
                        "第 {} 行：品項未定義單位「{}」的換算率，可用單位：{}",
                        line.line_no,
                        line.uom,
                        table.accepted_uoms().join(" / ")
                    ))
                })?;
                let qty = line.qty.checked_mul(factor).ok_or_else(|| {
                    AppError::Validation(format!("第 {} 行：數量換算後溢位", line.line_no))
                })?;
                // 單價與數量同除同乘，維持 qty × unit_cost = 該行總額不變。
                let unit_price = match line.unit_price {
                    Some(price) => Some(price.checked_div(factor).ok_or_else(|| {
                        AppError::Validation(format!("第 {} 行：單價換算後溢位", line.line_no))
                    })?),
                    None => None,
                };
                Ok(DocumentLine {
                    qty,
                    uom: table.base_uom.clone(),
                    unit_price,
                    ..line.clone()
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glove_table() -> ProductUomTable {
        // 乳膠手套：base_uom = 雙，一盒 50 雙，一箱 10 盒 = 500 雙
        ProductUomTable {
            base_uom: "雙".to_string(),
            factors: HashMap::from([
                ("盒".to_string(), Decimal::from(50)),
                ("箱".to_string(), Decimal::from(500)),
            ]),
        }
    }

    #[test]
    fn base_uom_factor_is_one_without_a_conversion_row() {
        assert_eq!(glove_table().factor("雙"), Some(Decimal::ONE));
    }

    #[test]
    fn pack_uom_uses_the_conversion_factor() {
        let table = glove_table();
        assert_eq!(table.factor("盒"), Some(Decimal::from(50)));
        assert_eq!(table.factor("箱"), Some(Decimal::from(500)));
    }

    #[test]
    fn undefined_uom_is_rejected_not_defaulted_to_one() {
        // 這是本模組存在的理由：舊版把未知單位當 1，盒與雙會被直接相加。
        assert_eq!(glove_table().factor("打"), None);
    }

    #[test]
    fn accepted_uoms_lists_base_and_conversions_sorted() {
        assert_eq!(glove_table().accepted_uoms(), vec!["盒", "箱", "雙"]);
    }
}
