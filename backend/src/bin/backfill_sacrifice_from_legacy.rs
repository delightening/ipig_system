//! 一次性：依舊系統（權威）補登新系統尚掛在場、舊系統已犧牲的動物的犧牲紀錄。
//!
//! 預設 dry-run（只讀、不寫）；加 `--apply` 才寫入。寫入只經既有正規路徑
//! （`AnimalService::update` 補 IACUC、`AnimalMedicalService::upsert_sacrifice` 補登），
//! 並在補登後重算受影響欄位的 `current_count`。冪等：重跑時已完成者顯示「已完成」。
//!
//! 用法：
//!   DATABASE_URL_FILE=... AUDIT_HMAC_KEY_FILE=... \
//!     backfill_sacrifice_from_legacy --csv <file.csv> --actor-email <email> [--apply]
//!
//! 連線與金鑰一律走 `read_secret_strict` / `read_audit_hmac_key_strict`：`_FILE` 優先、
//! 讀不到即中止，不會靜默落回 `.env` 殘留的裸 `DATABASE_URL`（否則 `--apply` 可能寫到與 API
//! 不同的資料庫）。`--apply` 時 HMAC 金鑰為必要：缺金鑰會寫出 integrity_hash 為 NULL 的
//! 稽核紀錄、造成 chain 斷點且事後無法補，故缺金鑰直接中止（寧可不寫，不寫壞）。
//!
//! actor 必須是啟用中、且具 IACUC 類 view_all 角色或系統管理員的使用者（授權檢查用其真實角色）。
//! 報告含耳號（營運資料）：只印到 stdout，請勿存進 repo。

use clap::Parser;
use erp_backend::config::{read_audit_hmac_key_strict, read_secret_strict};
use erp_backend::repositories::pen_alignment::PenAlignmentRepository;
use erp_backend::services::pen_alignment::render_reconciliation;
use erp_backend::services::sacrifice_backfill::{
    affected_pen_ids, apply, evaluate, load_actor, read_legacy_csv, render_report, Decision,
    RowOutcome,
};
use erp_backend::services::AuditService;
use sqlx::PgPool;

#[derive(Parser, Debug)]
#[command(
    name = "backfill_sacrifice_from_legacy",
    about = "Backfill sacrifice records from the legacy system for animals still in-facility. Dry-run by default."
)]
struct Args {
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

    let pool = PgPool::connect(&read_secret_strict("DATABASE_URL")?).await?;
    let actor = load_actor(&pool, &args.actor_email)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let items = evaluate(&pool, rows)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // 對帳要涵蓋所有會被重算 current_count 的欄：Ready（動物離場）與 AlreadyDone（重跑時修復過期快取）
    let pen_ids = affected_pen_ids(&items);
    let before = PenAlignmentRepository::list_pens(&pool, Some(&pen_ids))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if !args.apply {
        println!("== dry-run（未寫入）==");
        println!("{}", render_report(&items, None));
        // 預測：每個 Ready 動物離場 → 該欄在場數 -1。
        // 只被 AlreadyDone 指到的欄沒有動物離場，預測值就是「重算後的實際在場數」，
        // 對照「改前」即可看出過期的快取將被修正。
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

    // 寫入前載入 HMAC 金鑰，讓本次寫入的稽核紀錄進 chain；缺金鑰就不寫（理由見檔頭）。
    let hmac_key = read_audit_hmac_key_strict()?.ok_or_else(|| {
        anyhow::anyhow!(
            "--apply 需要 AUDIT_HMAC_KEY（或 AUDIT_HMAC_KEY_FILE，≥44 字元）；\
             未設定會寫出無 integrity_hash 的稽核紀錄，已中止、未寫入任何資料"
        )
    })?;
    AuditService::init_hmac_key(Some(hmac_key));

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
