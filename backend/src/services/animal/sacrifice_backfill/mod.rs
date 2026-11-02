//! 一次性工具：依舊系統（權威）補登新系統「該已犧牲卻還掛在場」的動物的犧牲紀錄。
//!
//! - `plan`：純判斷（耳號＋出生日期唯一對應、IACUC 規則、冪等），可單元測試。
//! - `run`：CSV 讀取、查候選、逐隻補登、報告輸出。
//!
//! 補登一律走既有正規路徑：
//! - 缺 IACUC 時先經 `AnimalService::update` 補 `iacuc_no`（同交易、稽核 `ANIMAL_UPDATE`＋`IACUC_CHANGE`）；
//! - 再經 `AnimalMedicalService::upsert_sacrifice`（`confirmed_sacrifice = true`，
//!   同交易寫犧牲紀錄、改 status = euthanized、稽核 `SACRIFICE_UPSERT`）。
//!
//! 入口：`src/bin/backfill_sacrifice_from_legacy.rs`（預設 dry-run，`--apply` 才寫入）。

pub mod plan;
pub mod run;

pub use plan::{decide, Decision, LegacySacRow};
pub use run::{apply, evaluate, load_actor, read_legacy_csv, render_report, Evaluated, RowOutcome};
