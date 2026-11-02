//! 欄位對齊報告的文字輸出（純函式，可單元測試）。
//!
//! 內容含耳號與欄號，屬營運資料：只印到 stdout，由執行者自行處理，不寫進 repo。

use std::collections::HashMap;
use std::fmt::Write as _;

use uuid::Uuid;

use super::plan::{BlockReason, Outcome, Plan, PlannedMove};
use super::run::ApplyOutcome;
use crate::repositories::pen_alignment::PenSnap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    DryRun,
    Apply,
}

fn loc_display(mv: &PlannedMove) -> String {
    let text = mv.from_location.as_deref().unwrap_or("NULL");
    let by_id = match (&mv.from_pen_id, &mv.from_pen_code) {
        (None, _) => "NULL".to_string(),
        (Some(_), Some(code)) => code.clone(),
        (Some(id), None) => format!("未知欄 {id}"),
    };
    format!("文字={text} / pen_id→{by_id}")
}

/// 改前／改後 current_count 對帳（受影響的欄）。
pub fn render_reconciliation(
    before: &[PenSnap],
    after: &[PenSnap],
    after_is_predicted: bool,
) -> String {
    let after_by_id: HashMap<Uuid, &PenSnap> = after.iter().map(|p| (p.id, p)).collect();
    let tag = if after_is_predicted {
        "預測後"
    } else {
        "實際後"
    };
    let mut s = String::new();
    let _ = writeln!(
        s,
        "欄 | 容量 | 改前 current_count | 改前 實際在場 | {tag} current_count | {tag} 實際在場"
    );
    for b in before {
        match after_by_id.get(&b.id) {
            Some(a) => {
                let flag = if i64::from(a.current_count) != a.live_count {
                    "  ⚠ current_count 與實際在場不符"
                } else {
                    ""
                };
                let _ = writeln!(
                    s,
                    "{} | {} | {} | {} | {} | {}{}",
                    b.code,
                    b.capacity,
                    b.current_count,
                    b.live_count,
                    a.current_count,
                    a.live_count,
                    flag
                );
            }
            None => {
                let _ = writeln!(s, "{} | {} | (查無改後資料)", b.code, b.capacity);
            }
        }
    }
    s
}

pub fn render_plan(plan: &Plan, mode: Mode, applied: Option<&ApplyOutcome>) -> String {
    let mut aligned = 0usize;
    let mut moves = 0usize;
    let mut blocked = 0usize;
    let mut manual = 0usize;
    for it in &plan.items {
        match it.outcome {
            Outcome::Aligned => aligned += 1,
            Outcome::Move { .. } => moves += 1,
            Outcome::Blocked { .. } => blocked += 1,
            Outcome::Manual(_) => manual += 1,
        }
    }

    let mut s = String::new();
    let mode_label = match mode {
        Mode::DryRun => "DRY-RUN（未寫入任何資料；加 --apply 才寫入）",
        Mode::Apply => "APPLY（已寫入）",
    };
    let _ = writeln!(s, "模式：{mode_label}");
    let _ = writeln!(
        s,
        "舊系統清單 {} 隻：已對齊 {aligned}｜要改 {moves}｜容量／欄位不可用而不改 {blocked}｜待人工 {manual}\n",
        plan.items.len()
    );

    let _ = writeln!(s, "== 要改（依執行順序）==");
    for (it, mv) in plan.moves_in_order() {
        let _ = writeln!(
            s,
            "耳號 {} ：{}  →  {}（pen_location 與 pen_id 都寫 {}）{}",
            it.ear,
            loc_display(mv),
            mv.to_code,
            mv.to_code,
            if mv.send_pen_id {
                ""
            } else {
                "［已在該欄，只修文字，不重送 pen_id］"
            }
        );
    }

    let _ = writeln!(
        s,
        "\n== 不改：被容量／欄位狀態擋下（不調整 pens.capacity）=="
    );
    for it in &plan.items {
        if let Outcome::Blocked { mv, reason } = &it.outcome {
            let why = match reason {
                BlockReason::Capacity { capacity, occupied } => {
                    format!("容量不足（容量 {capacity}，序列模擬後已佔 {occupied}）")
                }
                BlockReason::PenUnavailable(w) => format!("欄位不可用：{w}"),
            };
            let _ = writeln!(
                s,
                "耳號 {} ：{}  →  {}　【{why}】",
                it.ear,
                loc_display(mv),
                mv.to_code
            );
        }
    }

    let _ = writeln!(s, "\n== 待人工 ==");
    for it in &plan.items {
        if let Outcome::Manual(why) = &it.outcome {
            let _ = writeln!(s, "耳號 {}（舊欄 {}）：{why}", it.ear, it.old_pen);
        }
    }

    if let Some(done) = applied {
        let _ = writeln!(
            s,
            "\n== 執行結果 ==\n成功 {}｜失敗 {}",
            done.applied,
            done.failed.len()
        );
        for f in &done.failed {
            let _ = writeln!(s, "失敗：耳號 {} → {}：{}", f.ear, f.to_code, f.reason);
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pen(code: &str, cur: i32, live: i64) -> PenSnap {
        PenSnap {
            id: Uuid::new_v4(),
            code: code.into(),
            capacity: 1,
            current_count: cur,
            status: "active".into(),
            is_active: true,
            live_count: live,
        }
    }

    #[test]
    fn reconciliation_flags_mismatch_after() {
        let b = pen("A01", 0, 0);
        let mut a = b.clone();
        a.current_count = 0;
        a.live_count = 1;
        let text = render_reconciliation(&[b], &[a], false);
        assert!(text.contains("A01"));
        assert!(text.contains("不符"));
    }

    #[test]
    fn reconciliation_labels_predicted() {
        let b = pen("A01", 0, 0);
        let mut a = b.clone();
        a.current_count = 1;
        a.live_count = 1;
        let text = render_reconciliation(&[b], &[a], true);
        assert!(text.contains("預測後"));
        assert!(!text.contains("不符"));
    }
}
