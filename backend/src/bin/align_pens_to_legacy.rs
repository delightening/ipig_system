//! 一次性：把新系統在場動物的欄位（欄號）對齊舊系統（權威）。
//!
//! 預設 dry-run（只讀、不寫）；加 `--apply` 才寫入。寫入一律經 `AnimalService::update`
//! （同交易 UPDATE animals + 重算舊／新欄 current_count + 寫 ANIMAL_UPDATE 稽核）。
//!
//! 用法：
//!   DATABASE_URL=... align_pens_to_legacy --csv <pen-diff.csv> --actor-email <email> \
//!       [--exclude-ears <耳號,耳號,...>] [--apply]
//!
//! 報告含耳號與欄號（營運資料）：只印到 stdout，請勿存進 repo。

use clap::Parser;
use erp_backend::middleware::{ActorContext, CurrentUser};
use erp_backend::services::pen_alignment::{
    apply_plan, build_plan_excluding, load_affected_pens, load_snapshots, read_legacy_csv,
    render_plan, render_reconciliation, Mode,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "align_pens_to_legacy",
    about = "Align in-facility animals' pen (pen_location + pen_id) to the legacy system. Dry-run by default."
)]
struct Args {
    /// Database URL
    #[arg(env = "DATABASE_URL")]
    database_url: String,
    /// 舊系統清單 CSV（只用 old_pen 與 ear 兩欄）
    #[arg(long)]
    csv: std::path::PathBuf,
    /// 稽核用的操作者 email（必須是啟用中的使用者）
    #[arg(long)]
    actor_email: String,
    /// 指定排除的耳號（逗號分隔）；這些耳號一律列入待人工、不動
    #[arg(long, value_delimiter = ',')]
    exclude_ears: Vec<String>,
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
    let legacy = read_legacy_csv(csv_file).map_err(|e| anyhow::anyhow!("{e}"))?;

    let pool = PgPool::connect(&args.database_url).await?;

    let actor_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM users WHERE lower(email) = lower($1) AND is_active = true LIMIT 1",
    )
    .bind(&args.actor_email)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("找不到啟用中的 actor：{}", args.actor_email))?;
    let actor = ActorContext::User(CurrentUser {
        id: actor_id,
        email: args.actor_email.clone(),
        roles: vec![],
        permissions: vec![],
        jti: String::new(),
        exp: 0,
        impersonated_by: None,
    });

    let snaps = load_snapshots(&pool).await?;
    let plan = build_plan_excluding(&legacy, &snaps.animals, &snaps.pens, &args.exclude_ears);
    let before = load_affected_pens(&pool, &plan).await?;

    if !args.apply {
        println!("{}", render_plan(&plan, Mode::DryRun, None));
        let predicted: Vec<_> = before
            .iter()
            .map(|b| {
                let mut a = b.clone();
                let n = plan
                    .predicted_live
                    .get(&b.id)
                    .copied()
                    .unwrap_or(b.live_count);
                a.live_count = n;
                a.current_count = i32::try_from(n).unwrap_or(i32::MAX);
                a
            })
            .collect();
        println!("== 受影響欄 current_count 對帳（預測）==");
        println!("{}", render_reconciliation(&before, &predicted, true));
        return Ok(());
    }

    let outcome = apply_plan(&pool, &actor, &plan).await;
    println!("{}", render_plan(&plan, Mode::Apply, Some(&outcome)));
    let after = load_affected_pens(&pool, &plan).await?;
    println!("== 受影響欄 current_count 對帳（改前／改後）==");
    println!("{}", render_reconciliation(&before, &after, false));

    if !outcome.failed.is_empty() {
        anyhow::bail!("{} 隻執行失敗，見上方「執行結果」", outcome.failed.len());
    }
    Ok(())
}
