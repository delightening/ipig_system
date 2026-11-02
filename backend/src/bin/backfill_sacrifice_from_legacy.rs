//! 一次性：依舊系統（權威）補登新系統尚掛在場、舊系統已犧牲的動物的犧牲紀錄。
//!
//! 預設 dry-run（只讀、不寫）；加 `--apply` 才寫入。寫入只經既有正規路徑
//! （`AnimalService::update` 補 IACUC、`AnimalMedicalService::upsert_sacrifice` 補登），
//! 並在補登後重算受影響欄位的 `current_count`。冪等：重跑時已完成者顯示「已完成」。
//!
//! 用法：
//!   DATABASE_URL=... backfill_sacrifice_from_legacy --csv <file.csv> --actor-email <email> [--apply]
//!
//! actor 必須是啟用中、且具 IACUC 類 view_all 角色或系統管理員的使用者（授權檢查用其真實角色）。
//! 報告含耳號（營運資料）：只印到 stdout，請勿存進 repo。

use std::collections::HashSet;

use clap::Parser;
use erp_backend::repositories::pen_alignment::PenAlignmentRepository;
use erp_backend::services::pen_alignment::render_reconciliation;
use erp_backend::services::sacrifice_backfill::{
    apply, evaluate, load_actor, read_legacy_csv, render_report, Decision, RowOutcome,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "backfill_sacrifice_from_legacy",
    about = "Backfill sacrifice records from the legacy system for animals still in-facility. Dry-run by default."
)]
struct Args {
    /// Database URL
    #[arg(env = "DATABASE_URL")]
    database_url: String,
    /// 舊系統犧牲資料 CSV（欄位見 services/animal/sacrifice_backfill/run.rs）
    #[arg(long)]
    csv: std::path::PathBuf,
    /// 稽核用的操作者 email（必須是啟用中的使用者）
    #[arg(long)]
    actor_email: String,
    /// 真的寫入；不帶就是 dry-run
    #[arg(long, default_value_t = false)]
    apply: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    let csv_file = std::fs::File::open(&args.csv)
        .map_err(|e| anyhow::anyhow!("無法開啟 CSV {}: {e}", args.csv.display()))?;
    let rows = read_legacy_csv(csv_file).map_err(|e| anyhow::anyhow!("{e}"))?;

    let pool = PgPool::connect(&args.database_url).await?;
    let actor = load_actor(&pool, &args.actor_email)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let items = evaluate(&pool, rows)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let pen_ids: Vec<Uuid> = items
        .iter()
        .filter_map(|it| match &it.decision {
            Decision::Ready { pen_id, .. } => *pen_id,
            _ => None,
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let before = PenAlignmentRepository::list_pens(&pool, Some(&pen_ids))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if !args.apply {
        println!("== dry-run（未寫入）==");
        println!("{}", render_report(&items, None));
        // 預測：每個 Ready 動物離場 → 該欄在場數 -1
        let predicted: Vec<_> = before
            .iter()
            .map(|b| {
                let leaving = items
                    .iter()
                    .filter(|it| matches!(&it.decision, Decision::Ready { pen_id: Some(p), .. } if *p == b.id))
                    .count();
                let mut a = b.clone();
                a.live_count = (b.live_count - leaving as i64).max(0);
                a.current_count = i32::try_from(a.live_count).unwrap_or(i32::MAX);
                a
            })
            .collect();
        println!("== 受影響欄 current_count 對帳（預測）==");
        println!("{}", render_reconciliation(&before, &predicted, true));
        return Ok(());
    }

    let outcomes = apply(&pool, &actor, &items).await;
    println!("== 執行結果 ==");
    println!("{}", render_report(&items, Some(&outcomes)));
    let after = PenAlignmentRepository::list_pens(&pool, Some(&pen_ids))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("== 受影響欄 current_count 對帳（改前／改後）==");
    println!("{}", render_reconciliation(&before, &after, false));

    let failed = outcomes
        .iter()
        .filter(|o| matches!(o, RowOutcome::Failed(_)))
        .count();
    if failed > 0 {
        anyhow::bail!("{failed} 隻執行失敗，見上方「執行結果」");
    }
    Ok(())
}
