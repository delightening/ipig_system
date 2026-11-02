//! 犧牲補登的純判斷（不碰 DB，可單元測試）。
//!
//! 輸入：舊系統的一筆犧牲紀錄 + 以「耳號 + 出生日期」查到的新系統候選動物。
//! 輸出：這隻該 `Ready`（可補登）、`AlreadyDone`（已補過，冪等）或 `Manual`（交人工，附原因）。
//! 原則：任何不確定都走 `Manual`，絕不猜。

use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::repositories::sacrifice_backfill::AnimalCandidate;

/// 舊系統一筆犧牲紀錄（已逐隻人工確認對應到新系統的哪一隻）。
#[derive(Debug, Clone)]
pub struct LegacySacRow {
    pub ear: String,
    pub legacy_id: String,
    pub birth: NaiveDate,
    /// 舊系統的 IACUC No.；空 = 舊系統沒有。
    pub iacuc: Option<String>,
    pub sac_date: Option<NaiveDate>,
    pub zoletil: Option<String>,
    pub electro: bool,
    pub bleed: bool,
    pub other: Option<String>,
    pub sampling: Option<String>,
    pub sampling_other: Option<String>,
    pub blood_ml: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Ready {
        animal_id: Uuid,
        version: i32,
        pen_id: Option<Uuid>,
        /// 新系統的 iacuc_no 為空、需要用舊系統的值補上時才有值。
        set_iacuc: Option<String>,
        from_status: String,
    },
    /// 已有犧牲紀錄且狀態已是 euthanized：不重複寫。
    AlreadyDone {
        animal_id: Uuid,
    },
    Manual(String),
}

/// 可以直接轉成 euthanized 的狀態（對應 `AnimalStatus::can_transition_to`；
/// 真正寫入時 `upsert_sacrifice` 仍會再驗一次）。
const ELIGIBLE_STATUS: &[&str] = &["unassigned", "in_experiment", "completed"];

pub fn decide(row: &LegacySacRow, cands: &[AnimalCandidate]) -> Decision {
    let cand = match cands {
        [] => {
            return Decision::Manual("新系統查無此耳號＋出生日期（未刪除）".to_string());
        }
        [one] => one,
        many => {
            return Decision::Manual(format!(
                "新系統同耳號＋出生日期有 {} 筆，無法唯一對應",
                many.len()
            ));
        }
    };

    if cand.status == "euthanized" && cand.has_sacrifice {
        return Decision::AlreadyDone { animal_id: cand.id };
    }
    if !ELIGIBLE_STATUS.contains(&cand.status.as_str()) {
        return Decision::Manual(format!("新系統狀態為 {}，不能補登犧牲", cand.status));
    }
    if cand.has_sacrifice {
        return Decision::Manual("新系統已有犧牲紀錄但狀態不是 euthanized，需人工檢視".to_string());
    }
    if row.sac_date.is_none() {
        return Decision::Manual("舊系統沒有犧牲日期".to_string());
    }

    let legacy_iacuc = row
        .iacuc
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let set_iacuc = match (cand.iacuc_no.as_deref(), legacy_iacuc) {
        (Some(have), Some(legacy)) if have != legacy => {
            return Decision::Manual(format!(
                "IACUC 不一致（新系統 {have}／舊系統 {legacy}），不覆蓋"
            ));
        }
        (Some(_), _) => None,
        (None, Some(legacy)) => Some(legacy.to_string()),
        (None, None) => {
            return Decision::Manual(
                "新舊系統都沒有 IACUC No.，犧牲紀錄需要計畫，無法補".to_string(),
            );
        }
    };

    Decision::Ready {
        animal_id: cand.id,
        version: cand.version,
        pen_id: cand.pen_id,
        set_iacuc,
        from_status: cand.status.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(iacuc: Option<&str>, sac: bool) -> LegacySacRow {
        LegacySacRow {
            ear: "AAA".into(),
            legacy_id: "1".into(),
            birth: NaiveDate::from_ymd_opt(2025, 1, 1).expect("date"),
            iacuc: iacuc.map(str::to_string),
            sac_date: sac.then(|| NaiveDate::from_ymd_opt(2026, 1, 1).expect("date")),
            zoletil: None,
            electro: false,
            bleed: true,
            other: Some("KCl".into()),
            sampling: None,
            sampling_other: None,
            blood_ml: None,
        }
    }

    fn cand(status: &str, iacuc: Option<&str>, has_sac: bool) -> AnimalCandidate {
        AnimalCandidate {
            id: Uuid::nil(),
            status: status.into(),
            iacuc_no: iacuc.map(str::to_string),
            pen_id: None,
            version: 3,
            has_sacrifice: has_sac,
        }
    }

    #[test]
    fn missing_or_duplicate_candidate_is_manual() {
        assert!(matches!(
            decide(&row(Some("P1"), true), &[]),
            Decision::Manual(_)
        ));
        let two = [
            cand("unassigned", None, false),
            cand("unassigned", None, false),
        ];
        assert!(matches!(
            decide(&row(Some("P1"), true), &two),
            Decision::Manual(_)
        ));
    }

    #[test]
    fn iacuc_is_filled_from_legacy_only_when_new_is_empty() {
        let d = decide(&row(Some("P1"), true), &[cand("unassigned", None, false)]);
        assert!(matches!(&d, Decision::Ready { set_iacuc: Some(s), .. } if s == "P1"));
        let d = decide(
            &row(Some("P1"), true),
            &[cand("in_experiment", Some("P1"), false)],
        );
        assert!(matches!(
            d,
            Decision::Ready {
                set_iacuc: None,
                ..
            }
        ));
        // 都有但不同 → 不覆蓋
        let d = decide(
            &row(Some("P1"), true),
            &[cand("in_experiment", Some("P2"), false)],
        );
        assert!(matches!(d, Decision::Manual(_)));
        // 新舊都沒有 → 人工
        let d = decide(&row(None, true), &[cand("unassigned", None, false)]);
        assert!(matches!(d, Decision::Manual(_)));
        // 新系統有、舊系統沒有 → 照新系統的，不動
        let d = decide(&row(None, true), &[cand("unassigned", Some("P2"), false)]);
        assert!(matches!(
            d,
            Decision::Ready {
                set_iacuc: None,
                ..
            }
        ));
    }

    #[test]
    fn no_sacrifice_date_or_terminal_status_is_manual() {
        let d = decide(&row(Some("P1"), false), &[cand("unassigned", None, false)]);
        assert!(matches!(d, Decision::Manual(_)));
        let d = decide(&row(Some("P1"), true), &[cand("transferred", None, false)]);
        assert!(matches!(d, Decision::Manual(_)));
    }

    #[test]
    fn rerun_after_apply_is_already_done() {
        let d = decide(
            &row(Some("P1"), true),
            &[cand("euthanized", Some("P1"), true)],
        );
        assert_eq!(
            d,
            Decision::AlreadyDone {
                animal_id: Uuid::nil()
            }
        );
    }
}
