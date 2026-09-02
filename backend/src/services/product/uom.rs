//! 單位字串的正規形式，與「包裝關係 → 換算列」的推導。
//!
//! ## 為什麼需要這個模組
//!
//! `products.pack_unit` / `pack_qty` 與 `product_uom_conversions` 講的是同一件事
//! （「1 箱 = 20 包」），但在本 PR 之前是兩份各自可寫、互不校驗的資料：
//!
//! - 換算表的寫入端（`insert_uom_conversions_tx` / `sync_uom_conversions_tx`）**零驗證**，
//!   `uom = base_uom` 的同名列與 `factor_to_base <= 0` 都寫得進去。同名列會讓盤點端
//!   開出把貨架清空的盤虧 ADJ、GRN 端把正確收貨判成超收（見 `document/stocktake.rs`
//!   與 `document/grn.rs` 檔頭的警告）。
//! - 反過來，改了 `pack_qty` 卻不動換算表，盤點底稿會照舊的除數算，**不會報錯**。
//!
//! 因此本模組把「什麼是合法的換算列」與「包裝關係該推導出哪一列」收斂成純函式，
//! 由 `ProductService` 在每個寫入點呼叫，讓兩份資料只有一個真相來源。
//!
//! ## 正規形式 = 中文顯示名
//!
//! 2026-09-02 裁定（依據與逐項清單見私有 docs 的單位換算遷移計畫）：既有品項的 `base_uom`
//! 已經是中文，而單據明細的 `uom` 全部等於各自品項的 `base_uom`；反方向（統一成英文代碼）
//! 要改寫已核准的單據，且部分中文單位（例：桶）根本沒有對應代碼。
//! 故正規形式取中文，前端送代碼或中文都可以，由本層收斂。

use rust_decimal::Decimal;

use crate::models::UomConversionInput;

/// 英文代碼 → 中文正規形式。
///
/// ⚠️ 必須與 `frontend/src/lib/utils.ts` 的 `UOM_MAP` 保持同一組值——前端拿它做**顯示**
/// （`formatUom`）與**反查**（產品編輯表單的 `unitToCode`），這裡拿它做**儲存**。
/// 兩邊若分岔，會出現「畫面顯示同一個單位、資料庫存的是兩個不同字串」的假重複。
///
/// 比對區分大小寫，與前端的 `UOM_MAP[value]` 查表行為一致：`KG` 會被正規化成 `kg`，
/// 但 `Kg` 查不到、原樣保留。
const UOM_CANONICAL: [(&str, &str); 21] = [
    // 計數／個體
    ("EA", "個"),
    ("pcs", "個"),
    ("PC", "支"),
    ("PR", "雙"),
    // 藥品／劑型
    ("TB", "錠"),
    ("CP", "膠囊"),
    ("BT", "瓶"),
    ("AMP", "安瓿"),
    ("VIA", "小瓶"),
    // 包裝
    ("BX", "盒"),
    ("BOX", "箱"),
    ("CTN", "箱"),
    ("PK", "包"),
    ("CASE", "件"),
    ("RL", "卷"),
    ("SET", "組"),
    // 重量
    ("G", "g"),
    ("KG", "kg"),
    ("MG", "mg"),
    // 體積／容量
    ("ML", "mL"),
    ("L", "L"),
];

/// 把單位字串收斂成正規形式：去頭尾空白後查表，查不到就原樣保留。
///
/// 查不到不是錯誤——現場可能有表上沒有的單位（例：桶、捲）。保留原字串讓它照樣可用，
/// 只是不做代碼轉換。
pub fn canonical_uom(raw: &str) -> String {
    let trimmed = raw.trim();
    UOM_CANONICAL
        .iter()
        .find(|(code, _)| *code == trimmed)
        .map(|(_, name)| (*name).to_string())
        .unwrap_or_else(|| trimmed.to_string())
}

/// `Option` 版的 `canonical_uom`：`None` 與空字串都回 `None`。
pub fn canonical_uom_opt(raw: Option<&str>) -> Option<String> {
    raw.map(canonical_uom).filter(|s| !s.is_empty())
}

/// 由 `pack_unit` / `pack_qty` 推導出「應該存在的那一列換算」。
///
/// 回 `None` 代表這組包裝沒有換算意義，**不該產生任何列**：
///
/// - 沒填 `pack_unit`。
/// - `pack_qty` 缺、`<= 1`：`factor = 1` 的列是死資料——`ProductUomTable::factor()` 對
///   base_uom 本來就回 1，而 `counting_uom()` 要求 `factor > 1` 才會用包裝單位呈現底稿。
///   它唯一的作用是讓單據的單位下拉多一個與 base_uom 等值的選項，反而讓 PO/GRN
///   那組依字串相等 join 的 `SUM(qty)` 依填法產生不同結果。
/// - 正規化後與 `base_uom` 同名：這是最危險的一種，見模組註解。**必須在正規化之後才比**
///   ——`base_uom = 盒` 配上 `pack_unit = BX`（正規化後也是「盒」）字串不同、實質同名，
///   只比原字串會整批漏掉。
pub fn derive_pack_conversion(
    base_uom: &str,
    pack_unit: Option<&str>,
    pack_qty: Option<i32>,
) -> Option<UomConversionInput> {
    let uom = canonical_uom_opt(pack_unit)?;
    let qty = pack_qty?;
    if qty < 2 {
        return None;
    }
    if uom == canonical_uom(base_uom) {
        return None;
    }
    Some(UomConversionInput {
        uom,
        factor_to_base: Decimal::from(qty),
    })
}

/// 組出「要寫進換算表的最終清單」：驗證呼叫端明確給的列，再併入包裝關係推導出的那一列。
///
/// 回傳的每一列 `uom` 都已正規化。錯誤訊息直接給使用者看，故用中文並指出是哪一個單位。
///
/// 規則：
/// 1. `uom` 去空白後不得為空。
/// 2. `factor_to_base` 必須 `> 0`——非正數乘上去會讓入庫變出庫（負數）或整行歸零而靜默漏帳，
///    與 `StockService::load_uom_tables` 略過非正數 factor 的行為一致（那邊略過等於「未定義」，
///    這裡直接不讓它寫進去）。
/// 3. 正規化後不得等於 `base_uom`。
/// 4. 正規化後不得重複（DB 有 `UNIQUE(product_id, uom)`，但先在這裡擋才給得出好訊息）。
/// 5. 明確給的列若正好就是包裝單位，換算率必須與 `pack_qty` 一致——不一致代表呼叫端對同一件事
///    給了兩個互相矛盾的答案，**不猜哪個對，直接擋下**。
pub fn merge_for_write(
    base_uom: &str,
    explicit: &[UomConversionInput],
    derived: Option<UomConversionInput>,
) -> std::result::Result<Vec<UomConversionInput>, String> {
    let base = canonical_uom(base_uom);
    if base.is_empty() {
        return Err("base_uom 不得為空".to_string());
    }

    let mut out: Vec<UomConversionInput> = Vec::with_capacity(explicit.len() + 1);
    for conv in explicit {
        let uom = canonical_uom(&conv.uom);
        if uom.is_empty() {
            return Err("換算單位不得為空".to_string());
        }
        if conv.factor_to_base <= Decimal::ZERO {
            return Err(format!("單位「{uom}」的換算率必須大於 0"));
        }
        if uom == base {
            return Err(format!(
                "單位「{uom}」與基本單位相同，不需要換算列（基本單位的換算率恆為 1）"
            ));
        }
        if out.iter().any(|existing| existing.uom == uom) {
            return Err(format!("單位「{uom}」重複"));
        }
        out.push(UomConversionInput {
            uom,
            factor_to_base: conv.factor_to_base,
        });
    }

    if let Some(derived) = derived {
        match out.iter().find(|existing| existing.uom == derived.uom) {
            Some(existing) if existing.factor_to_base != derived.factor_to_base => {
                return Err(format!(
                    "單位「{}」的換算率（{}）與包裝數量（{}）不一致",
                    derived.uom, existing.factor_to_base, derived.factor_to_base
                ));
            }
            Some(_) => {}
            None => out.push(derived),
        }
    }

    Ok(out)
}

/// 包裝關係改變時，對換算表要做的兩個動作。
///
/// 拆成純函式是為了讓「什麼情況該刪、什麼情況該留」驗得起來——這段邏輯的錯誤不會有任何
/// 錯誤訊息，只會讓盤點底稿安靜地用錯除數，靠整合測試才發現太晚。
#[derive(Debug, PartialEq, Eq)]
pub struct PackConversionPlan {
    /// 要刪掉的舊推導列。只有「值與舊推導完全一致」才刪，人工改過的不動。
    pub remove: Option<UomConversionInput>,
    /// 🔴 舊資料以**非正規寫法**存的同一列（例：`pack_unit = 'BX'` 時代建的 `uom = 'BX'`）。
    ///
    /// 與 `remove` 分開，因為它的鍵不是正規值、比對不到。不刪它的後果是無聲的：
    /// `pack_unit` 被正規化成「盒」之後，盤點底稿那句
    /// `LEFT JOIN product_uom_conversions c ON c.uom = p.pack_unit` 拿「盒」去比「BX」
    /// 永遠比不中 → `pack_factor` 為 NULL → 底稿退回 base_uom，
    /// 症狀與「這個品項根本沒建換算列」完全一樣，查不出來。
    pub remove_alias: Option<UomConversionInput>,
    /// 要寫入（或覆寫換算率）的新推導列。
    pub upsert: Option<UomConversionInput>,
}

/// 比較更新前後的包裝關係，算出換算表要做什麼。
///
/// 兩邊推導結果相同時回空計畫（`remove` 與 `upsert` 皆 `None`）——**完全不碰 DB**。
/// 這一點很重要：產品更新是高頻操作，多數不動包裝，不該每次都刪一列再插回去
/// （那會讓換算列的 id 每次更新都變動，稽核追不到同一列）。
pub fn plan_pack_conversion_change(
    before_base_uom: &str,
    before_pack_unit: Option<&str>,
    before_pack_qty: Option<i32>,
    after_base_uom: &str,
    after_pack_unit: Option<&str>,
    after_pack_qty: Option<i32>,
) -> PackConversionPlan {
    let old = derive_pack_conversion(before_base_uom, before_pack_unit, before_pack_qty);
    let new = derive_pack_conversion(after_base_uom, after_pack_unit, after_pack_qty);
    let alias = legacy_alias_row(before_pack_unit, before_pack_qty, old.as_ref());

    if old == new && alias.is_none() {
        return PackConversionPlan {
            remove: None,
            remove_alias: None,
            upsert: None,
        };
    }
    // 單位沒變、只有換算率變（50 → 24）：直接覆寫，不必先刪——刪了再插會換掉 id。
    let remove = match (&old, &new) {
        (Some(o), Some(n)) if o.uom == n.uom => None,
        _ => old,
    };
    PackConversionPlan {
        remove,
        remove_alias: alias,
        upsert: new,
    }
}

/// 舊資料可能以非正規寫法存了同一列（`pack_unit = 'BX'` 時代建的 `uom = 'BX'`）。
///
/// 只有在「原字串與正規形式不同」且「該包裝本來就推導得出換算列」時才算數——
/// 前者代表它是別名，後者代表當初真的會有那一列。回傳的 `factor_to_base` 取自
/// 當時的 `pack_qty`，供刪除時做值比對（值被人工改過就不刪，交給人負責）。
fn legacy_alias_row(
    before_pack_unit: Option<&str>,
    before_pack_qty: Option<i32>,
    old: Option<&UomConversionInput>,
) -> Option<UomConversionInput> {
    let old = old?;
    let raw = before_pack_unit?.trim();
    if raw.is_empty() || raw == canonical_uom(raw) {
        return None;
    }
    Some(UomConversionInput {
        uom: raw.to_string(),
        factor_to_base: Decimal::from(before_pack_qty?),
    })
    .filter(|alias| alias.uom != old.uom)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conv(uom: &str, factor: i64) -> UomConversionInput {
        UomConversionInput {
            uom: uom.to_string(),
            factor_to_base: Decimal::from(factor),
        }
    }

    #[test]
    fn canonical_uom_maps_codes_to_chinese() {
        assert_eq!(canonical_uom("BX"), "盒");
        assert_eq!(canonical_uom("CTN"), "箱");
        assert_eq!(canonical_uom("PK"), "包");
        assert_eq!(canonical_uom(" CASE "), "件");
    }

    #[test]
    fn canonical_uom_keeps_unmapped_values() {
        // 現場實際存在、對照表沒有的單位（桶／捲）必須原樣可用。
        assert_eq!(canonical_uom("桶"), "桶");
        assert_eq!(canonical_uom("捲"), "捲");
        // 大小寫與前端 UOM_MAP 查表一致：Kg 查不到，不會被當成 KG。
        assert_eq!(canonical_uom("Kg"), "Kg");
        assert_eq!(canonical_uom("KG"), "kg");
    }

    #[test]
    fn derive_skips_pack_qty_below_two() {
        assert_eq!(derive_pack_conversion("支", Some("BX"), Some(1)), None);
        assert_eq!(derive_pack_conversion("支", Some("BX"), Some(0)), None);
        assert_eq!(derive_pack_conversion("支", Some("BX"), None), None);
    }

    #[test]
    fn derive_skips_missing_pack_unit() {
        assert_eq!(derive_pack_conversion("支", None, Some(50)), None);
        assert_eq!(derive_pack_conversion("支", Some("  "), Some(50)), None);
    }

    #[test]
    fn derive_skips_same_name_after_normalization() {
        // 🔴 本模組存在的主因：base_uom=盒 + pack_unit=BX(→盒) 字串不同、正規化後同名。
        // 實查存在這一型的品項，且它們的 pack_qty 全部 >= 2 —— 靠 pack_qty 門檻濾不掉。
        assert_eq!(derive_pack_conversion("盒", Some("BX"), Some(100)), None);
        assert_eq!(derive_pack_conversion("包", Some("PK"), Some(1000)), None);
        // 原字串就相同的那一型同樣擋掉。
        assert_eq!(derive_pack_conversion("桶", Some("桶"), Some(4)), None);
    }

    #[test]
    fn derive_returns_canonical_row_for_real_packaging() {
        let row = derive_pack_conversion("支", Some("BX"), Some(50)).expect("應推導出換算列");
        assert_eq!(row.uom, "盒");
        assert_eq!(row.factor_to_base, Decimal::from(50));
    }

    #[test]
    fn merge_rejects_non_positive_factor() {
        assert!(merge_for_write("支", &[conv("盒", 0)], None).is_err());
        assert!(merge_for_write("支", &[conv("盒", -50)], None).is_err());
    }

    #[test]
    fn merge_rejects_row_equal_to_base_uom() {
        // 直接同名
        assert!(merge_for_write("盒", &[conv("盒", 100)], None).is_err());
        // 正規化後才同名——這是 API 目前擋不住、會寫進災難資料的路徑
        assert!(merge_for_write("盒", &[conv("BX", 100)], None).is_err());
    }

    #[test]
    fn merge_rejects_duplicate_after_normalization() {
        let err = merge_for_write("支", &[conv("盒", 50), conv("BX", 50)], None)
            .expect_err("BX 正規化後就是盒，屬重複");
        assert!(err.contains("重複"), "訊息應指出重複：{err}");
    }

    #[test]
    fn merge_normalizes_and_appends_derived_row() {
        let out = merge_for_write("支", &[conv("CTN", 500)], Some(conv("盒", 50)))
            .expect("合法組合應通過");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].uom, "箱");
        assert_eq!(out[1].uom, "盒");
    }

    #[test]
    fn merge_is_idempotent_when_derived_row_already_listed() {
        let out = merge_for_write("支", &[conv("BX", 50)], Some(conv("盒", 50)))
            .expect("同一列以代碼與中文各給一次，正規化後應視為同一列");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].uom, "盒");
        assert_eq!(out[0].factor_to_base, Decimal::from(50));
    }

    #[test]
    fn merge_rejects_factor_conflicting_with_pack_qty() {
        let err = merge_for_write("支", &[conv("盒", 24)], Some(conv("盒", 50)))
            .expect_err("同一單位給了兩個換算率，必須擋下而不是挑一個");
        assert!(err.contains("不一致"), "訊息應指出不一致：{err}");
    }

    #[test]
    fn merge_accepts_empty_input() {
        assert_eq!(
            merge_for_write("支", &[], None).expect("空清單合法").len(),
            0
        );
    }

    #[test]
    fn merge_rejects_blank_uom() {
        assert!(merge_for_write("支", &[conv("   ", 50)], None).is_err());
    }

    #[test]
    fn plan_is_empty_when_packaging_unchanged() {
        // 高頻路徑：改品名／安全庫存等欄位，包裝沒動 → 完全不碰換算表。
        // 用正規寫法，因為 `update_tx` 會把 pack_unit 收斂成正規形式後才寫入。
        let plan =
            plan_pack_conversion_change("支", Some("盒"), Some(50), "支", Some("盒"), Some(50));
        assert_eq!(plan.remove, None);
        assert_eq!(plan.remove_alias, None);
        assert_eq!(plan.upsert, None);
    }

    #[test]
    fn plan_renames_legacy_alias_row_when_representation_normalized() {
        // 🔴 CodeRabbit 於 PR #61 指出、實際成立的一條：
        // 舊品項存 pack_unit='BX'，換算表也留著 uom='BX' 的列。本次更新把 pack_unit
        // 正規化成「盒」，前後推導都是「盒」——若因此判定「無變化」，那列 BX 會留在原地，
        // 而盤點底稿的 `c.uom = p.pack_unit` 之後拿「盒」去比「BX」永遠比不中。
        // 正確行為是：刪掉別名列、寫入正規列。
        let plan =
            plan_pack_conversion_change("支", Some("BX"), Some(50), "支", Some("盒"), Some(50));
        assert_eq!(plan.remove, None, "正規鍵沒變，不該刪正規列");
        assert_eq!(plan.remove_alias, Some(conv("BX", 50)), "必須刪掉別名列");
        assert_eq!(plan.upsert, Some(conv("盒", 50)), "必須確保正規列存在");
    }

    #[test]
    fn plan_has_no_alias_when_old_packaging_had_no_row() {
        // 舊的 pack_qty=1 本來就推導不出任何列，就沒有別名列存在的可能。
        let plan =
            plan_pack_conversion_change("支", Some("BX"), Some(1), "支", Some("BX"), Some(50));
        assert_eq!(plan.remove_alias, None);
        assert_eq!(plan.upsert, Some(conv("盒", 50)));
    }

    #[test]
    fn plan_removes_alias_even_when_packaging_cleared() {
        // 清掉包裝關係時，別名列也要一起清，否則它會永遠留著。
        let plan = plan_pack_conversion_change("支", Some("BX"), Some(50), "支", None, None);
        assert_eq!(plan.remove, Some(conv("盒", 50)));
        assert_eq!(plan.remove_alias, Some(conv("BX", 50)));
        assert_eq!(plan.upsert, None);
    }

    #[test]
    fn plan_overwrites_factor_without_deleting_when_unit_unchanged() {
        // 🔴 本次要堵的缺口：pack_qty 50 → 24。舊行為是換算表停在 50，盤點底稿照 50 除且不報錯。
        let plan =
            plan_pack_conversion_change("支", Some("BX"), Some(50), "支", Some("BX"), Some(24));
        assert_eq!(plan.remove, None, "單位沒變就不該刪列（刪了會換掉 id）");
        assert_eq!(plan.upsert, Some(conv("盒", 24)));
    }

    #[test]
    fn plan_moves_row_when_pack_unit_changed() {
        let plan =
            plan_pack_conversion_change("支", Some("BX"), Some(50), "支", Some("CTN"), Some(500));
        assert_eq!(plan.remove, Some(conv("盒", 50)));
        assert_eq!(plan.upsert, Some(conv("箱", 500)));
    }

    #[test]
    fn plan_removes_row_when_packaging_cleared() {
        let plan = plan_pack_conversion_change("支", Some("BX"), Some(50), "支", None, None);
        assert_eq!(plan.remove, Some(conv("盒", 50)));
        assert_eq!(plan.upsert, None);
    }

    #[test]
    fn plan_removes_row_when_pack_qty_drops_to_one() {
        // pack_qty 由 50 降為 1 ＝ 宣告「沒有包裝關係」，殘留的 factor 50 必須清掉。
        let plan =
            plan_pack_conversion_change("支", Some("BX"), Some(50), "支", Some("BX"), Some(1));
        assert_eq!(plan.remove, Some(conv("盒", 50)));
        assert_eq!(plan.upsert, None);
    }

    #[test]
    fn plan_adds_row_when_packaging_first_declared() {
        let plan = plan_pack_conversion_change("支", None, None, "支", Some("BX"), Some(50));
        assert_eq!(plan.remove, None);
        assert_eq!(plan.upsert, Some(conv("盒", 50)));
    }

    #[test]
    fn plan_never_upserts_a_same_name_row() {
        // base_uom=盒 + pack_unit=BX(→盒)：即使 pack_qty 改了也不能寫入同名列。
        let plan =
            plan_pack_conversion_change("盒", Some("BX"), Some(100), "盒", Some("BX"), Some(50));
        assert_eq!(plan.remove, None);
        assert_eq!(plan.upsert, None);
    }
}
