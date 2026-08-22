//! 置頂待辦對帳：把「置頂中、但關聯業務實體已不存在或已在終態」的通知降級。
//!
//! 用途有二：
//! 1. **一次性修補**——清掉 2026-08-07 事故留下的殘留待辦（巡場 retract/delete 未接
//!    解除 hook 造成）。
//! 2. **定期安全網**——各業務流程新增終態路徑時難免漏接解除 hook，本作業定期補救。
//!
//! 判斷依據是業務實體的真實狀態，不是使用者意願，因此不違反
//! 「待辦只能由系統偵測完成自動消失、使用者不可手動略過」的設計。
//!
//! ## Usage
//!
//! **預設是唯讀。** 不帶參數＝dry-run，只列出將被降級的列；要真的寫入必須明確加
//! `--apply`。理由：本 repo 根目錄的 `.env` 指向 **prod**，而 `dotenvy` 會自動載入它 ——
//! 若沿用「預設寫入、加 `--dry-run` 才預覽」的慣例，在 repo 內隨手跑一次不帶參數的
//! 指令就會直接改到正式資料。這支工具的誤觸成本高於打字成本。
//!
//! ```bash
//! # 預覽（預設）
//! DATABASE_URL_FILE=../secrets/db_url_host.txt \
//!   cargo run --bin reconcile_pinned_notifications
//! # 核對筆數與目標 DB 無誤後，正式寫入
//! DATABASE_URL_FILE=../secrets/db_url_host.txt \
//!   cargo run --bin reconcile_pinned_notifications -- --apply
//! ```

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;

use erp_backend::services::NotificationService;

fn read_database_url() -> Result<String> {
    if let Ok(path) = std::env::var("DATABASE_URL_FILE") {
        return Ok(std::fs::read_to_string(&path)
            .with_context(|| format!("read DATABASE_URL_FILE {path}"))?
            .trim()
            .to_string());
    }
    std::env::var("DATABASE_URL").context("DATABASE_URL（或 DATABASE_URL_FILE）must be set")
}

const USAGE: &str = "\
用法：reconcile_pinned_notifications [--apply]

  （不帶參數）  預設 dry-run：只查不寫，列出將被降級的孤兒待辦
  --apply       實際寫入資料庫
  --help        顯示本說明

環境變數：DATABASE_URL_FILE 或 DATABASE_URL
⚠️ repo 根目錄的 .env 指向 prod，且會被 dotenvy 自動載入。執行前請確認下方印出的目標 DB。
";

/// 從連線字串取出 host/database 供執行前確認，順便避免把密碼印出來。
///
/// **順序不可顛倒**：必須先切掉 query / fragment，再以 `@` 取右段。
/// 反過來的話，若 query 裡含 `@`（例如 `?options=-c%20search_path=a@b`），
/// `rsplit('@')` 會取到 query 的尾段而不是 host —— 印出來的既不是目標位址，
/// 又可能把 query 裡的值洩到終端機。
fn describe_target(url: &str) -> String {
    let no_query = url.split(['?', '#']).next().unwrap_or(url);
    no_query.rsplit('@').next().unwrap_or(no_query).to_string()
}

/// 嚴格剖析參數，回傳 `dry_run`。
///
/// 兩層防護，缺一不可：
/// 1. **預設 dry-run**，寫入要明確 `--apply`。repo 根目錄的 `.env` 指向 prod
///    且被 `dotenvy` 自動載入，「預設寫入」等於讓隨手一跑就改到正式資料。
/// 2. **嚴格拒絕未知參數**。若用 `args().any(|a| a == "--apply")`，
///    反過來說任何拼錯都會被靜默忽略；這裡兩個方向都要堵住 ——
///    拼錯 `--aply` 應該報錯，而不是安靜地變成另一種模式。
fn parse_args() -> Result<bool> {
    let mut apply = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--apply" => apply = true,
            "--help" | "-h" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            other => anyhow::bail!("未知參數 `{other}`\n\n{USAGE}"),
        }
    }
    Ok(!apply)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let dry_run = parse_args()?;

    let url = read_database_url()?;
    // 執行前先讓人看見目標是哪個 DB —— 這支工具寫 prod 的路徑太短（.env 自動載入），
    // 「我以為連到測試庫」是最可能的誤操作。
    println!(
        "目標資料庫：{}\n模式：{}\n",
        describe_target(&url),
        if dry_run {
            "dry-run（只查不寫）"
        } else {
            "APPLY（將寫入資料庫）"
        }
    );

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .context("connect db")?;

    let svc = NotificationService::new(pool);
    let report = svc
        .reconcile_pinned_notifications(dry_run)
        .await
        .context("reconcile pinned notifications")?;

    // dry-run 時尚未寫入（「待降級」），非 dry-run 時 reconcile 回傳前已完成降級（「已降級」）。
    // 兩者混用同一組文案會讓維運者分不清「這批到底動了沒」。
    let tag = if dry_run { "[dry-run] " } else { "" };
    let verb = if dry_run { "待降級" } else { "已降級" };

    println!("{tag}{verb}的孤兒待辦（共 {} 筆）：", report.resolved.len());
    for r in &report.resolved {
        // 不印 email：對應到人請用 user_id 查 users 表（見 OrphanPinnedRow 的說明）。
        println!(
            "  {} | user={} | {} | {}",
            r.created_at.format("%Y-%m-%d %H:%M"),
            r.user_id,
            r.title,
            r.reason
        );
    }

    if !report.unknown_entity_types.is_empty() {
        println!("\n⚠️ 本作業不認得下列 entity_type，保守未做判斷（如有新待辦類型請補進 reconcile.rs）：");
        for (ty, n) in &report.unknown_entity_types {
            println!("  {ty} — {n} 筆");
        }
    }

    if report.null_entity_id > 0 {
        println!(
            "\n⚠️ {} 筆置頂待辦的 related_entity_id 為 NULL —— 沒有業務實體可追溯。",
            report.null_entity_id
        );
        println!("   本作業無從判斷其是否仍需使用者動作，保守未動；這些列會永遠留在待處理清單。");
        println!("   需人工決定處置（已知來源：2026-03 的舊聚合式採購提醒）。");
    }

    // still_pending 已扣除「未知類型」與「NULL entity」——這裡只算已判定仍需動作的，
    // 不把「未做判斷」的混進來充數。
    let affected = if dry_run {
        report.resolved.len() as u64
    } else {
        // 非 dry-run 回報 UPDATE 實際影響列數，而非候選數：
        // UPDATE 帶 `priority > 0`，併發下可能有列已被業務路徑先解除。
        report.resolved_rows_affected
    };
    println!(
        "\n{tag}完成：{verb} {affected} 筆；已判定仍需使用者動作 {} 筆",
        report.still_pending
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::describe_target;

    #[test]
    fn strips_credentials_and_query() {
        assert_eq!(
            describe_target("postgres://u:pw@localhost:5432/ipig_db"),
            "localhost:5432/ipig_db"
        );
    }

    /// query 內含 `@` 時，先切 query 才不會把 query 尾段當成 host 印出來。
    #[test]
    fn query_containing_at_sign_does_not_leak() {
        assert_eq!(
            describe_target("postgres://u:pw@localhost:5432/ipig_db?opt=a@secret"),
            "localhost:5432/ipig_db"
        );
    }
}
