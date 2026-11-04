//! 一次性工具：把新系統在場動物的欄位（欄號）對齊舊系統（權威）。
//!
//! - `plan`：純規劃（容量序列模擬、冪等判斷、待人工分流），可單元測試。
//! - `run`：CSV 讀取、快照載入、逐隻經 `AnimalService::update` 寫入。
//! - `report`：dry-run／apply 報告與 current_count 對帳的文字輸出。
//!
//! 入口：`src/bin/align_pens_to_legacy.rs`（預設 dry-run，`--apply` 才寫入）。

pub mod plan;
pub mod report;
pub mod run;

pub use plan::{
    build_plan, build_plan_excluding, BlockReason, LegacyRow, Outcome, Plan, PlanItem, PlannedMove,
};
pub use report::{render_plan, render_reconciliation, Mode};
pub use run::{
    apply_plan, load_affected_pens, load_snapshots, read_legacy_csv, ApplyFailure, ApplyOutcome,
    Snapshots,
};
