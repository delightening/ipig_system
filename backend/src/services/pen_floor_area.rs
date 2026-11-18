//! 欄位地板面積規則（純函式，不碰資料庫）。
//!
//! 依「欄內動物的最新體重」與「欄內隻數」對照最小地板面積表，算出所需面積合計，
//! 與欄位面積比較。**只用來警示，不擋操作**；唯一的硬限制是放入完全沒有體重的動物
//! （見 `AnimalService::ensure_animal_has_weight`）。
//!
//! 全程用整數運算，避免浮點誤差影響「剛好等於」的邊界：
//! - 體重以 0.1 kg 為單位（資料庫欄位本來就是一位小數）。
//! - 單隻所需面積以 0.01 m² 為單位（表上的值都是兩位小數）。
//! - 欄位面積 = 長（0.01 m）× 寬（0.01 m），以 0.0001 m² 為單位。
//!
//! 邊界體重取**較嚴格（面積較大）的區間**：表上 25／50／100 kg 同時落在相鄰兩個區間，
//! 一律歸入較重的那一段；200 kg 只在「100–200」，「>200」不含 200。

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use crate::models::{FloorAreaStatus, PenFloorArea};
use crate::{AppError, Result};

/// 長或寬的上限（公尺）。欄位是動物欄，超過這個值幾乎一定是輸入錯誤（例如把公分當公尺）。
const MAX_SIDE_M: i64 = 100;

/// 長寬最多兩位小數（與資料庫 `numeric(5,2)` 一致，避免靜默四捨五入）。
const MAX_SIDE_SCALE: u32 = 2;

/// 一段體重區間：`(下界含, 單隻所需面積)`，單位分別是 0.1 kg 與 0.01 m²。
/// 依下界由小到大排列；最後一段的下界 2001 表示「> 200 kg」。
type Band = (i64, i64);

/// 單獨飼養（欄內只有 1 隻）。
const SINGLE: [Band; 6] = [
    (0, 72),     // < 15 kg：0.72 m²
    (150, 108),  // 15–25 kg：1.08 m²
    (250, 135),  // 25–50 kg：1.35 m²
    (500, 216),  // 50–100 kg：2.16 m²
    (1000, 432), // 100–200 kg：4.32 m²
    (2001, 540), // > 200 kg：5.40 m²
];

/// 群飼 2–5 隻。
const GROUP_2_TO_5: [Band; 5] = [
    (0, 54),     // < 25 kg：0.54 m²
    (250, 90),   // 25–50 kg：0.90 m²
    (500, 180),  // 50–100 kg：1.80 m²
    (1000, 360), // 100–200 kg：3.60 m²
    (2001, 468), // > 200 kg：4.68 m²
];

/// 群飼 6 隻以上。
const GROUP_6_PLUS: [Band; 5] = [
    (0, 54),     // < 25 kg：0.54 m²
    (250, 81),   // 25–50 kg：0.81 m²
    (500, 162),  // 50–100 kg：1.62 m²
    (1000, 324), // 100–200 kg：3.24 m²
    (2001, 432), // > 200 kg：4.32 m²
];

/// 依欄內隻數選表。
fn table_for(animal_count: usize) -> &'static [Band] {
    match animal_count {
        0 | 1 => &SINGLE,
        2..=5 => &GROUP_2_TO_5,
        _ => &GROUP_6_PLUS,
    }
}

/// 體重（公斤）轉 0.1 kg 整數；無效值（非正數、超出範圍）回 `None`，視同缺體重。
///
/// 往上取整：萬一有多於一位小數的值，歸入較重（較嚴格）的一側。
fn weight_to_tenths(weight_kg: Decimal) -> Option<i64> {
    if weight_kg <= Decimal::ZERO {
        return None;
    }
    weight_kg.checked_mul(Decimal::from(10))?.ceil().to_i64()
}

/// 公尺轉 0.01 m 整數（長寬已由 `validate_dimensions` 限制為兩位小數，這裡只是換單位）。
fn meters_to_hundredths(meters: Decimal) -> Option<i64> {
    meters.checked_mul(Decimal::from(100))?.round().to_i64()
}

/// 單隻在指定隻數段下所需的最小面積（0.01 m²）。
fn required_hundredths(table: &[Band], weight_tenths: i64) -> i64 {
    table
        .iter()
        .rev()
        .find(|(min, _)| weight_tenths >= *min)
        .map(|(_, area)| *area)
        // 表的第一段下界是 0，weight_tenths 一定是正數，不會走到這裡；
        // 萬一表被改壞，取最嚴格（最後一段）而不是靜默當成 0。
        .unwrap_or_else(|| table.last().map(|(_, area)| *area).unwrap_or(0))
}

/// 驗證並整理請求帶來的欄位尺寸。
///
/// - 兩個都沒給 → `Ok(None)`（沒填尺寸，呼叫端維持原狀）。
/// - 只給其中一個 → 400（尺寸要同時有長與寬才有意義，也是資料庫的限制）。
/// - 兩個都給 → 各自必須 > 0、≤ 100 m、最多兩位小數。
pub fn validate_dimensions(
    length_m: Option<Decimal>,
    width_m: Option<Decimal>,
) -> Result<Option<(Decimal, Decimal)>> {
    let (length, width) = match (length_m, width_m) {
        (None, None) => return Ok(None),
        (Some(length), Some(width)) => (length, width),
        _ => {
            return Err(AppError::BadRequest(
                "欄位尺寸需同時提供長度與寬度".to_string(),
            ))
        }
    };
    for (label, side) in [("長度", length), ("寬度", width)] {
        if side <= Decimal::ZERO {
            return Err(AppError::BadRequest(format!("欄位{label}必須大於 0")));
        }
        if side > Decimal::from(MAX_SIDE_M) {
            return Err(AppError::BadRequest(format!(
                "欄位{label}不可超過 {MAX_SIDE_M} 公尺"
            )));
        }
        if side.normalize().scale() > MAX_SIDE_SCALE {
            return Err(AppError::BadRequest(format!(
                "欄位{label}最多兩位小數（公尺）"
            )));
        }
    }
    Ok(Some((length, width)))
}

/// 算出欄位的地板面積檢視。
///
/// - `dimensions`：欄位長寬（公尺）；沒填傳 `None`，結果為 `NoArea`。
/// - `weights`：欄內每隻動物的最新體重（公斤），`None` 表示缺體重。**隻數 = `weights.len()`**，
///   缺體重的動物仍算在隻數裡（牠實際佔了位子，會影響適用哪一段表），但不計入所需面積。
pub fn assess(dimensions: Option<(Decimal, Decimal)>, weights: &[Option<Decimal>]) -> PenFloorArea {
    let table = table_for(weights.len());
    let mut required: i64 = 0;
    let mut missing: i64 = 0;
    for weight in weights {
        match weight.and_then(weight_to_tenths) {
            Some(tenths) => required += required_hundredths(table, tenths),
            None => missing += 1,
        }
    }

    // 面積換成 0.0001 m² 整數；換算失敗（理論上不會，長寬已驗證）視同沒填。
    let area_1e4 = dimensions.and_then(|(length, width)| {
        let l = meters_to_hundredths(length)?;
        let w = meters_to_hundredths(width)?;
        l.checked_mul(w)
    });

    let status = match area_1e4 {
        None => FloorAreaStatus::NoArea,
        // 所需面積 0.01 m² → 0.0001 m² 要乘 100，再與欄位面積比
        Some(area) if required.saturating_mul(100) > area => FloorAreaStatus::Exceeded,
        Some(_) => FloorAreaStatus::Ok,
    };

    PenFloorArea {
        status,
        area_m2: area_1e4.map(|a| Decimal::new(a, 4).round_dp(2)),
        required_area_m2: Decimal::new(required, 2),
        animal_count: weights.len() as i64,
        animals_missing_weight: missing,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).expect("測試用的十進位數字")
    }

    fn w(s: &str) -> Option<Decimal> {
        Some(d(s))
    }

    /// 1.20 × 2.40 = 2.88 m²
    fn small_pen() -> Option<(Decimal, Decimal)> {
        Some((d("1.20"), d("2.40")))
    }

    fn same(weight: &str, n: usize) -> Vec<Option<Decimal>> {
        vec![w(weight); n]
    }

    fn status(dims: Option<(Decimal, Decimal)>, weights: &[Option<Decimal>]) -> FloorAreaStatus {
        assess(dims, weights).status
    }

    // ---- 單獨飼養表 ----

    #[test]
    fn single_animal_uses_the_single_table_for_every_weight_band() {
        let cases = [
            ("14.9", "0.72"),
            ("15.0", "1.08"),
            ("24.9", "1.08"),
            ("25.0", "1.35"),
            ("49.9", "1.35"),
            ("50.0", "2.16"),
            ("99.9", "2.16"),
            ("100.0", "4.32"),
            ("200.0", "4.32"),
            ("200.1", "5.40"),
        ];
        for (weight, expected) in cases {
            let result = assess(small_pen(), &[w(weight)]);
            assert_eq!(
                result.required_area_m2,
                d(expected),
                "單獨 {weight} kg 應需要 {expected} m²"
            );
        }
    }

    // ---- 群飼表 ----

    #[test]
    fn group_of_two_to_five_uses_the_group_table() {
        let cases = [
            ("20.0", "0.54"),
            ("25.0", "0.90"),
            ("50.0", "1.80"),
            ("100.0", "3.60"),
            ("200.1", "4.68"),
        ];
        for (weight, per_animal) in cases {
            for n in [2usize, 5] {
                let result = assess(small_pen(), &same(weight, n));
                let expected = d(per_animal) * Decimal::from(n as i64);
                assert_eq!(result.required_area_m2, expected, "{n} 隻 {weight} kg");
            }
        }
    }

    #[test]
    fn group_of_six_or_more_uses_the_larger_group_table() {
        let cases = [
            ("20.0", "0.54"),
            ("25.0", "0.81"),
            ("50.0", "1.62"),
            ("100.0", "3.24"),
            ("200.1", "4.32"),
        ];
        for (weight, per_animal) in cases {
            let result = assess(small_pen(), &same(weight, 6));
            let expected = d(per_animal) * Decimal::from(6);
            assert_eq!(result.required_area_m2, expected, "6 隻 {weight} kg");
        }
    }

    // ---- 邊界取較嚴格的區間 ----

    #[test]
    fn boundary_weights_take_the_stricter_band() {
        // 25 kg：同時是「<25」與「25–50」的邊界 → 歸入較大面積的 25–50
        assert_eq!(assess(None, &same("25.0", 2)).required_area_m2, d("1.80"));
        // 50 kg → 50–100；100 kg → 100–200
        assert_eq!(assess(None, &same("50.0", 2)).required_area_m2, d("3.60"));
        assert_eq!(assess(None, &same("100.0", 2)).required_area_m2, d("7.20"));
        // 200 kg 只屬於「100–200」，「>200」不含 200
        assert_eq!(assess(None, &same("200.0", 2)).required_area_m2, d("7.20"));
    }

    // ---- 小欄位（1.20 × 2.40 = 2.88 m²）的可容納隻數 ----

    #[test]
    fn small_pen_capacity_by_weight_matches_the_table() {
        // < 25 kg：最多 5 隻（5 × 0.54 = 2.70 ≤ 2.88；6 隻改用 6 隻以上的表，6 × 0.54 = 3.24 > 2.88）
        assert_eq!(status(small_pen(), &same("20.0", 5)), FloorAreaStatus::Ok);
        assert_eq!(
            status(small_pen(), &same("20.0", 6)),
            FloorAreaStatus::Exceeded
        );
        // 25–50 kg：最多 3 隻（3 × 0.90 = 2.70；4 × 0.90 = 3.60 > 2.88）
        assert_eq!(status(small_pen(), &same("30.0", 3)), FloorAreaStatus::Ok);
        assert_eq!(
            status(small_pen(), &same("30.0", 4)),
            FloorAreaStatus::Exceeded
        );
        // 50–100 kg：只能 1 隻（2.16 ≤ 2.88；2 × 1.80 = 3.60 > 2.88）
        assert_eq!(status(small_pen(), &same("60.0", 1)), FloorAreaStatus::Ok);
        assert_eq!(
            status(small_pen(), &same("60.0", 2)),
            FloorAreaStatus::Exceeded
        );
        // > 100 kg：單獨放就超標（4.32 > 2.88）
        assert_eq!(
            status(small_pen(), &same("120.0", 1)),
            FloorAreaStatus::Exceeded
        );
    }

    #[test]
    fn exactly_equal_to_the_pen_area_is_ok() {
        // 欄 1.20 × 0.90 = 1.08 m²，單隻 15–25 kg 剛好需要 1.08 → 不算超標
        let pen = Some((d("1.20"), d("0.90")));
        assert_eq!(status(pen, &[w("20.0")]), FloorAreaStatus::Ok);
        // 少 0.0012 m²（1.20 × 0.89 = 1.068）就超標
        let tighter = Some((d("1.20"), d("0.89")));
        assert_eq!(status(tighter, &[w("20.0")]), FloorAreaStatus::Exceeded);
    }

    // ---- 混合體重 ----

    #[test]
    fn mixed_weights_are_summed_per_animal() {
        // 3 隻（2–5 表）：20 kg 0.54 + 30 kg 0.90 + 60 kg 1.80 = 3.24
        let result = assess(small_pen(), &[w("20.0"), w("30.0"), w("60.0")]);
        assert_eq!(result.required_area_m2, d("3.24"));
        assert_eq!(result.status, FloorAreaStatus::Exceeded);
        assert_eq!(result.animal_count, 3);
    }

    // ---- 缺資料 ----

    #[test]
    fn missing_weight_is_counted_in_headcount_but_not_in_required_area() {
        // 2 隻：一隻 60 kg、一隻缺體重 → 用 2–5 表，只計 60 kg 的 1.80
        let result = assess(small_pen(), &[w("60.0"), None]);
        assert_eq!(result.animal_count, 2);
        assert_eq!(result.animals_missing_weight, 1);
        assert_eq!(result.required_area_m2, d("1.80"));
        assert_eq!(result.status, FloorAreaStatus::Ok);
    }

    #[test]
    fn non_positive_weight_is_treated_as_missing() {
        let result = assess(small_pen(), &[w("0"), w("-3.0")]);
        assert_eq!(result.animals_missing_weight, 2);
        assert_eq!(result.required_area_m2, Decimal::ZERO);
    }

    #[test]
    fn pen_without_dimensions_reports_no_area() {
        let result = assess(None, &same("20.0", 3));
        assert_eq!(result.status, FloorAreaStatus::NoArea);
        assert_eq!(result.area_m2, None);
        // 仍然算出所需面積，供畫面顯示參考（3 隻 × 0.54）
        assert_eq!(result.required_area_m2, d("1.62"));
    }

    #[test]
    fn empty_pen_is_ok_when_it_has_an_area() {
        let result = assess(small_pen(), &[]);
        assert_eq!(result.status, FloorAreaStatus::Ok);
        assert_eq!(result.animal_count, 0);
        assert_eq!(result.required_area_m2, Decimal::ZERO);
    }

    #[test]
    fn area_is_length_times_width_rounded_to_two_places() {
        let result = assess(Some((d("5.45"), d("2.29"))), &[]);
        // 5.45 × 2.29 = 12.4805
        assert_eq!(result.area_m2, Some(d("12.48")));
        let result = assess(small_pen(), &[]);
        assert_eq!(result.area_m2, Some(d("2.88")));
    }

    // ---- 尺寸驗證 ----

    #[test]
    fn dimensions_validation_accepts_both_or_none() {
        assert_eq!(validate_dimensions(None, None).expect("兩者皆空"), None);
        assert_eq!(
            validate_dimensions(Some(d("1.2")), Some(d("2.40"))).expect("合法尺寸"),
            Some((d("1.2"), d("2.40")))
        );
    }

    #[test]
    fn dimensions_validation_rejects_a_single_side() {
        assert!(validate_dimensions(Some(d("1.2")), None).is_err());
        assert!(validate_dimensions(None, Some(d("2.4"))).is_err());
    }

    #[test]
    fn dimensions_validation_rejects_bad_values() {
        // 非正數
        assert!(validate_dimensions(Some(d("0")), Some(d("2.4"))).is_err());
        assert!(validate_dimensions(Some(d("1.2")), Some(d("-1"))).is_err());
        // 過大（多半是把公分當公尺）
        assert!(validate_dimensions(Some(d("120")), Some(d("2.4"))).is_err());
        // 超過兩位小數
        assert!(validate_dimensions(Some(d("1.234")), Some(d("2.4"))).is_err());
        // 尾端零不算多一位：1.200 等同 1.2
        assert!(validate_dimensions(Some(d("1.200")), Some(d("2.4"))).is_ok());
    }
}
