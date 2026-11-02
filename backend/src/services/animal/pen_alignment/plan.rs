//! 欄位對齊的純規劃邏輯（無 I/O，可單元測試）。
//!
//! 輸入：舊系統清單（old_pen, ear, birth）、新系統在場動物快照、欄位快照。
//! 輸出：每一列的處置——已對齊 / 要改 / 被容量擋 / 待人工。
//!
//! 容量是**序列模擬**：搬走一隻會空出位置給後面要進來的那隻，所以用多輪掃描直到
//! 不再有進展；執行時照同一個 `seq` 順序逐隻更新，真實的容量檢查
//! （`validate_pen_for_assignment`）才會與模擬一致。

use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::NaiveDate;
use uuid::Uuid;

use super::super::utils::AnimalUtils;
use crate::repositories::pen_alignment::{AnimalSnap, PenSnap};

/// 可收容動物的欄位狀態。與 `core/pen_validation.rs` 的 `PEN_ALLOWED_STATUSES` 相同；
/// 那個常數是 core 的私有項，這裡只用來預判，真正的裁決仍是執行時的 service 檢查。
const PEN_ALLOWED_STATUSES: &[&str] = &["active", "empty"];

/// 舊系統清單的一列（CSV 只取 old_pen、ear、birth）。
/// 出生日期用來確認「同耳號」是同一隻（耳號可能被重用）；CSV 讀取時即要求必填。
#[derive(Debug, Clone)]
pub struct LegacyRow {
    pub old_pen: String,
    pub ear: String,
    pub birth: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct PlannedMove {
    pub animal_id: Uuid,
    pub version: i32,
    pub from_pen_id: Option<Uuid>,
    pub from_pen_code: Option<String>,
    pub from_location: Option<String>,
    pub to_pen_id: Uuid,
    pub to_code: String,
    /// 本隻已經在目標欄（pen_id 相同、只是 pen_location 文字不一致）時為 false：
    /// 不重送 pen_id，避免滿欄把「自己」算成佔位而誤擋。寫入後兩欄仍一致。
    pub send_pen_id: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockReason {
    Capacity { capacity: i32, occupied: i64 },
    PenUnavailable(String),
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Aligned,
    /// `seq` 是執行順序（模擬時的解決順序）。
    Move {
        mv: PlannedMove,
        seq: usize,
    },
    Blocked {
        mv: PlannedMove,
        reason: BlockReason,
    },
    Manual(String),
}

#[derive(Debug, Clone)]
pub struct PlanItem {
    pub ear: String,
    pub old_pen: String,
    pub outcome: Outcome,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub items: Vec<PlanItem>,
    /// 模擬全部「要改」之後，各欄的即時在場數（供 dry-run 對帳的「預測後」）。
    pub predicted_live: HashMap<Uuid, i64>,
}

impl Plan {
    /// 要改的項目，依執行順序排列。
    pub fn moves_in_order(&self) -> Vec<(&PlanItem, &PlannedMove)> {
        let mut v: Vec<(usize, &PlanItem, &PlannedMove)> = self
            .items
            .iter()
            .filter_map(|it| match &it.outcome {
                Outcome::Move { mv, seq } => Some((*seq, it, mv)),
                _ => None,
            })
            .collect();
        v.sort_by_key(|(seq, _, _)| *seq);
        v.into_iter().map(|(_, it, mv)| (it, mv)).collect()
    }

    /// 受影響的欄（要改項目的舊欄與新欄），供對帳。
    pub fn affected_pen_ids(&self) -> Vec<Uuid> {
        let mut set = BTreeSet::new();
        for (_, mv) in self.moves_in_order() {
            set.insert(mv.to_pen_id);
            if let Some(f) = mv.from_pen_id {
                set.insert(f);
            }
        }
        set.into_iter().collect()
    }
}

/// 一個欄位 code 能不能被 `AnimalService::update` 原樣寫入 pen_location
/// （update 會再過一次 `format_pen_location`）。
fn code_survives_formatting(code: &str) -> bool {
    AnimalUtils::format_pen_location(code) == code
}

fn pen_usable(pen: &PenSnap) -> Result<(), String> {
    if !pen.is_active {
        return Err(format!("欄位「{}」已停用", pen.code));
    }
    if !PEN_ALLOWED_STATUSES.contains(&pen.status.as_str()) {
        return Err(format!("欄位「{}」狀態為「{}」", pen.code, pen.status));
    }
    Ok(())
}

/// 以欄號找唯一的欄：優先啟用中的；啟用的有多個、或完全找不到 → Err（待人工）。
fn find_target_pen<'a>(old_pen: &str, pens: &'a [PenSnap]) -> Result<&'a PenSnap, String> {
    let wanted = AnimalUtils::format_pen_location(old_pen);
    let matches: Vec<&PenSnap> = pens
        .iter()
        .filter(|p| p.code == old_pen || p.code == wanted)
        .collect();
    let active: Vec<&PenSnap> = matches.iter().copied().filter(|p| p.is_active).collect();
    let chosen = match (active.len(), matches.len()) {
        (1, _) => active[0],
        (0, 1) => matches[0],
        (0, 0) => return Err(format!("新系統無欄號「{old_pen}」")),
        _ => return Err(format!("欄號「{old_pen}」對到多個欄位，無法唯一決定")),
    };
    if !code_survives_formatting(&chosen.code) {
        return Err(format!(
            "欄 code「{}」經 update 正規化後會變成「{}」，無法保證 pen_location 與 pens.code 一致",
            chosen.code,
            AnimalUtils::format_pen_location(&chosen.code)
        ));
    }
    Ok(chosen)
}

pub fn build_plan(legacy: &[LegacyRow], animals: &[AnimalSnap], pens: &[PenSnap]) -> Plan {
    build_plan_excluding(legacy, animals, pens, &[])
}

/// 同 `build_plan`，另接受「指定排除」的耳號：這些耳號一律列入待人工、不動。
/// 排除清單由執行者以命令列帶入，程式碼不內建任何耳號。
pub fn build_plan_excluding(
    legacy: &[LegacyRow],
    animals: &[AnimalSnap],
    pens: &[PenSnap],
    excluded_ears: &[String],
) -> Plan {
    let excluded: BTreeSet<String> = excluded_ears
        .iter()
        .map(|e| AnimalUtils::format_ear_tag(e.trim()))
        .collect();
    // 舊系統清單依「正規化耳號」分組；同耳號出現不同欄 → 待人工。
    // 同耳號若出現不同出生日期，也無法確定是哪一隻 → 待人工。
    let mut by_ear: BTreeMap<String, (BTreeSet<String>, BTreeSet<NaiveDate>)> = BTreeMap::new();
    for r in legacy {
        let entry = by_ear
            .entry(AnimalUtils::format_ear_tag(r.ear.trim()))
            .or_default();
        entry.0.insert(r.old_pen.trim().to_string());
        entry.1.insert(r.birth);
    }

    let mut animals_by_ear: HashMap<String, Vec<&AnimalSnap>> = HashMap::new();
    for a in animals {
        animals_by_ear
            .entry(AnimalUtils::format_ear_tag(a.ear_tag.trim()))
            .or_default()
            .push(a);
    }
    let pen_code_by_id: HashMap<Uuid, &str> =
        pens.iter().map(|p| (p.id, p.code.as_str())).collect();
    let mut occ: HashMap<Uuid, i64> = pens.iter().map(|p| (p.id, p.live_count)).collect();

    let mut items: Vec<PlanItem> = Vec::with_capacity(by_ear.len());
    let mut pending: Vec<(usize, PlannedMove, &PenSnap)> = Vec::new();

    for (ear, (pen_set, birth_set)) in &by_ear {
        let old_pen = pen_set.iter().cloned().collect::<Vec<_>>().join(" / ");
        // 用巨集而非閉包：閉包會長期借走 `items`，後面還要讀 `items.len()`。
        macro_rules! push {
            ($outcome:expr) => {
                items.push(PlanItem {
                    ear: ear.clone(),
                    old_pen: old_pen.clone(),
                    outcome: $outcome,
                })
            };
        }

        if excluded.contains(ear.as_str()) {
            push!(Outcome::Manual("指定排除（--exclude-ears）".into()));
            continue;
        }
        if pen_set.len() > 1 {
            push!(Outcome::Manual("舊系統清單同一耳號對到多個欄".into()));
            continue;
        }
        if birth_set.len() > 1 {
            push!(Outcome::Manual("舊系統清單同一耳號對到多個出生日期".into()));
            continue;
        }
        let single = old_pen.as_str();
        if single.is_empty() {
            push!(Outcome::Manual("舊系統欄號為空".into()));
            continue;
        }
        let animal = match animals_by_ear.get(ear).map(Vec::as_slice) {
            Some([one]) => *one,
            Some(many) if many.len() > 1 => {
                push!(Outcome::Manual(format!(
                    "新系統有 {} 筆同耳號的在場動物",
                    many.len()
                )));
                continue;
            }
            _ => {
                push!(Outcome::Manual("新系統找不到此耳號的在場動物".into()));
                continue;
            }
        };
        // 出生日期核對：不一致或新系統沒有，都不動（耳號可能被重用，寧可待人工）。
        if let Some(legacy_birth) = birth_set.iter().next() {
            match animal.birth_date {
                Some(b) if b == *legacy_birth => {}
                Some(b) => {
                    push!(Outcome::Manual(format!(
                        "出生日期不一致（新系統 {b}／舊系統 {legacy_birth}），不移動"
                    )));
                    continue;
                }
                None => {
                    push!(Outcome::Manual(
                        "新系統此動物沒有出生日期，無法核對，不移動".into()
                    ));
                    continue;
                }
            }
        }
        let pen = match find_target_pen(single, pens) {
            Ok(p) => p,
            Err(why) => {
                push!(Outcome::Manual(why));
                continue;
            }
        };

        if animal.pen_id == Some(pen.id)
            && animal.pen_location.as_deref() == Some(pen.code.as_str())
        {
            push!(Outcome::Aligned);
            continue;
        }

        let mv = PlannedMove {
            animal_id: animal.id,
            version: animal.version,
            from_pen_id: animal.pen_id,
            from_pen_code: animal
                .pen_id
                .and_then(|id| pen_code_by_id.get(&id).map(|c| (*c).to_string())),
            from_location: animal.pen_location.clone(),
            to_pen_id: pen.id,
            to_code: pen.code.clone(),
            send_pen_id: animal.pen_id != Some(pen.id),
        };
        let idx = items.len();
        push!(Outcome::Manual(String::new())); // 佔位，下面模擬後覆寫
        pending.push((idx, mv, pen));
    }

    // 容量序列模擬：多輪掃描，搬走的先釋出位置。
    let mut seq = 0usize;
    let mut remaining = pending;
    loop {
        let mut deferred: Vec<(usize, PlannedMove, &PenSnap)> = Vec::new();
        let mut progressed = false;
        for (idx, mv, pen) in remaining {
            if let Err(why) = pen_usable(pen) {
                items[idx].outcome = Outcome::Blocked {
                    mv,
                    reason: BlockReason::PenUnavailable(why),
                };
                continue;
            }
            let occupied = occ.get(&pen.id).copied().unwrap_or(0);
            let has_room =
                !mv.send_pen_id || pen.capacity <= 0 || occupied < i64::from(pen.capacity);
            if !has_room {
                deferred.push((idx, mv, pen));
                continue;
            }
            if mv.send_pen_id {
                *occ.entry(pen.id).or_insert(0) += 1;
                if let Some(from) = mv.from_pen_id {
                    *occ.entry(from).or_insert(0) -= 1;
                }
            }
            items[idx].outcome = Outcome::Move { mv, seq };
            seq += 1;
            progressed = true;
        }
        if deferred.is_empty() {
            break;
        }
        if !progressed {
            for (idx, mv, pen) in deferred {
                let occupied = occ.get(&pen.id).copied().unwrap_or(0);
                items[idx].outcome = Outcome::Blocked {
                    mv,
                    reason: BlockReason::Capacity {
                        capacity: pen.capacity,
                        occupied,
                    },
                };
            }
            break;
        }
        remaining = deferred;
    }

    Plan {
        items,
        predicted_live: occ,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pen(code: &str, capacity: i32, live: i64) -> PenSnap {
        PenSnap {
            id: Uuid::new_v4(),
            code: code.into(),
            capacity,
            current_count: live as i32,
            status: "active".into(),
            is_active: true,
            live_count: live,
        }
    }

    fn birth() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 3, 3).expect("date")
    }

    fn animal(ear: &str, pen: Option<&PenSnap>, loc: Option<&str>) -> AnimalSnap {
        AnimalSnap {
            id: Uuid::new_v4(),
            ear_tag: ear.into(),
            pen_id: pen.map(|p| p.id),
            pen_location: loc.map(str::to_string),
            birth_date: Some(birth()),
            version: 1,
        }
    }

    fn row(old_pen: &str, ear: &str) -> LegacyRow {
        LegacyRow {
            old_pen: old_pen.into(),
            ear: ear.into(),
            birth: birth(),
        }
    }

    #[test]
    fn birth_date_mismatch_or_missing_goes_to_manual() {
        let a1 = pen("A01", 1, 0);
        let mut mismatched = animal("010", None, None);
        mismatched.birth_date = NaiveDate::from_ymd_opt(2024, 1, 1);
        let mut unknown = animal("011", None, None);
        unknown.birth_date = None;
        let plan = build_plan(
            &[row("A01", "010"), row("A01", "011")],
            &[mismatched, unknown],
            &[a1],
        );
        assert!(matches!(outcome(&plan, "010"), Outcome::Manual(m) if m.contains("不一致")));
        assert!(matches!(outcome(&plan, "011"), Outcome::Manual(m) if m.contains("沒有出生日期")));
        assert!(plan.moves_in_order().is_empty());
    }

    #[test]
    fn same_ear_with_different_birth_dates_is_manual() {
        let a1 = pen("A01", 1, 0);
        let mut other = row("A01", "010");
        other.birth = NaiveDate::from_ymd_opt(2024, 1, 1).expect("date");
        let plan = build_plan(
            &[row("A01", "010"), other],
            &[animal("010", None, None)],
            &[a1],
        );
        assert!(matches!(outcome(&plan, "010"), Outcome::Manual(_)));
    }

    fn outcome<'a>(plan: &'a Plan, ear: &str) -> &'a Outcome {
        &plan
            .items
            .iter()
            .find(|i| i.ear == ear)
            .expect("plan item for ear")
            .outcome
    }

    #[test]
    fn already_aligned_is_untouched() {
        let a1 = pen("A01", 1, 1);
        let animals = [animal("010", Some(&a1), Some("A01"))];
        let plan = build_plan(&[row("A01", "010")], &animals, &[a1]);
        assert!(matches!(outcome(&plan, "010"), Outcome::Aligned));
    }

    #[test]
    fn null_pen_id_with_matching_text_gets_pen_id() {
        let a1 = pen("A01", 1, 0);
        let animals = [animal("010", None, Some("A01"))];
        let plan = build_plan(&[row("A01", "010")], &animals, std::slice::from_ref(&a1));
        match outcome(&plan, "010") {
            Outcome::Move { mv, .. } => {
                assert!(mv.send_pen_id);
                assert_eq!(mv.to_pen_id, a1.id);
            }
            other => panic!("expected Move, got {other:?}"),
        }
    }

    #[test]
    fn same_pen_id_wrong_text_skips_capacity_and_pen_id() {
        // 滿欄（容量 1、已有 1）裡的「自己」只是 pen_location 文字不一致：不可被容量擋
        let a1 = pen("A01", 1, 1);
        let animals = [animal("010", Some(&a1), Some("B07"))];
        let plan = build_plan(&[row("A01", "010")], &animals, &[a1]);
        match outcome(&plan, "010") {
            Outcome::Move { mv, .. } => assert!(!mv.send_pen_id),
            other => panic!("expected Move, got {other:?}"),
        }
    }

    #[test]
    fn full_pen_blocks_and_reports() {
        let a1 = pen("A01", 1, 1);
        let b1 = pen("B01", 1, 1);
        let animals = [
            animal("010", Some(&b1), Some("B01")),
            animal("011", Some(&a1), Some("A01")),
        ];
        // 010 要去已被 011 佔住的 A01，而 011 沒有要搬 → 容量擋
        let plan = build_plan(&[row("A01", "010")], &animals, &[a1, b1]);
        assert!(matches!(
            outcome(&plan, "010"),
            Outcome::Blocked {
                reason: BlockReason::Capacity {
                    capacity: 1,
                    occupied: 1
                },
                ..
            }
        ));
    }

    #[test]
    fn chain_moves_resolve_in_dependency_order() {
        // 010: A01→B01（B01 被 011 佔）；011: B01→C01（空）。010 要等 011 先搬。
        let a1 = pen("A01", 1, 1);
        let b1 = pen("B01", 1, 1);
        let c1 = pen("C01", 1, 0);
        let animals = [
            animal("010", Some(&a1), Some("A01")),
            animal("011", Some(&b1), Some("B01")),
        ];
        let plan = build_plan(
            &[row("B01", "010"), row("C01", "011")],
            &animals,
            &[a1.clone(), b1.clone(), c1.clone()],
        );
        let order: Vec<&str> = plan
            .moves_in_order()
            .iter()
            .map(|(it, _)| it.ear.as_str())
            .collect();
        assert_eq!(order, vec!["011", "010"]);
        assert_eq!(plan.predicted_live[&a1.id], 0);
        assert_eq!(plan.predicted_live[&b1.id], 1);
        assert_eq!(plan.predicted_live[&c1.id], 1);
    }

    #[test]
    fn swap_deadlock_is_blocked_not_forced() {
        let a1 = pen("A01", 1, 1);
        let b1 = pen("B01", 1, 1);
        let animals = [
            animal("010", Some(&a1), Some("A01")),
            animal("011", Some(&b1), Some("B01")),
        ];
        let plan = build_plan(&[row("B01", "010"), row("A01", "011")], &animals, &[a1, b1]);
        assert!(plan.moves_in_order().is_empty());
        assert!(matches!(outcome(&plan, "010"), Outcome::Blocked { .. }));
        assert!(matches!(outcome(&plan, "011"), Outcome::Blocked { .. }));
    }

    #[test]
    fn group_pen_accepts_up_to_capacity() {
        let g3 = pen("G03", 2, 0);
        let animals = [
            animal("010", None, None),
            animal("011", None, None),
            animal("012", None, None),
        ];
        let plan = build_plan(
            &[row("G03", "010"), row("G03", "011"), row("G03", "012")],
            &animals,
            &[g3],
        );
        assert_eq!(plan.moves_in_order().len(), 2);
        assert!(matches!(outcome(&plan, "012"), Outcome::Blocked { .. }));
    }

    #[test]
    fn excluded_and_unknown_ears_go_to_manual() {
        let a1 = pen("A01", 1, 0);
        let animals = [animal("020", None, None), animal("010", None, None)];
        let excluded = ["20".to_string(), "030".to_string()];
        let plan = build_plan_excluding(
            &[row("A01", "20"), row("A01", "030"), row("A01", "999")],
            &animals,
            &[a1],
            &excluded,
        );
        assert!(matches!(outcome(&plan, "020"), Outcome::Manual(_)));
        assert!(matches!(outcome(&plan, "030"), Outcome::Manual(_)));
        assert!(matches!(outcome(&plan, "999"), Outcome::Manual(_)));
    }

    #[test]
    fn duplicate_live_ear_and_missing_pen_go_to_manual() {
        let a1 = pen("A01", 1, 0);
        let animals = [animal("010", None, None), animal("010", None, None)];
        let plan = build_plan(&[row("A01", "010"), row("Z99", "011")], &animals, &[a1]);
        assert!(matches!(outcome(&plan, "010"), Outcome::Manual(_)));
        assert!(matches!(outcome(&plan, "011"), Outcome::Manual(_)));
    }

    #[test]
    fn inactive_pen_is_blocked_unavailable() {
        let mut a1 = pen("A01", 1, 0);
        a1.is_active = false;
        let animals = [animal("010", None, None)];
        let plan = build_plan(&[row("A01", "010")], &animals, &[a1]);
        assert!(matches!(
            outcome(&plan, "010"),
            Outcome::Blocked {
                reason: BlockReason::PenUnavailable(_),
                ..
            }
        ));
    }

    #[test]
    fn rerun_after_alignment_has_nothing_to_move() {
        let a1 = pen("A01", 1, 1);
        let animals = [animal("010", Some(&a1), Some("A01"))];
        let plan = build_plan(&[row("A01", "010")], &animals, &[a1]);
        assert!(plan.moves_in_order().is_empty());
        assert!(matches!(outcome(&plan, "010"), Outcome::Aligned));
    }
}
