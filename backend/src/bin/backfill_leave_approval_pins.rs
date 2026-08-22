//! 一次性：回填 PR #67（置頂待辦機制）上線前送審／轉關的假單，補建正確收件人的置頂待辦。
//!
//! #67 之前，`submit_leave` / `approve_leave` 沒有 tx 內建立置頂待辦這回事——那批假單當時
//! 還在 `PENDING_PROXY` / `PENDING_L1` / `PENDING_DIRECTOR` 就已經送出，代理人／審核人
//! 從未收到任何通知。#67 只讓「以後的轉關」正確，不會回頭補這批舊資料；
//! `reconcile_pinned_notifications` 也只單向降級孤兒待辦，不會反向建立。本作業補這個缺口。
//!
//! 冪等：已有對應 `recipient_role` 置頂列者略過，可重複執行（例如先 dry-run 核對，
//! 隔一段時間才 `--apply`，中間若有單子被業務路徑正常轉關也不會重複建立）。
//!
//! ## Usage
//! ```bash
//! # 預覽（預設）
//! DATABASE_URL_FILE=../secrets/db_url_host.txt \
//!   cargo run --bin backfill_leave_approval_pins
//! # 核對筆數與收件人無誤後，正式寫入
//! DATABASE_URL_FILE=../secrets/db_url_host.txt \
//!   cargo run --bin backfill_leave_approval_pins -- --apply
//! ```

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;

use erp_backend::services::HrService;

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
用法：backfill_leave_approval_pins [--apply]

  （不帶參數）  預設 dry-run：只查不寫，列出將補建的置頂待辦
  --apply       實際寫入資料庫
  --help        顯示本說明

環境變數：DATABASE_URL_FILE 或 DATABASE_URL
⚠️ repo 根目錄的 .env 指向 prod，且會被 dotenvy 自動載入。執行前請確認下方印出的目標 DB。
";

/// 從連線字串取出 host/database 供執行前確認，順便避免把密碼印出來。
fn describe_target(url: &str) -> String {
    let no_query = url.split(['?', '#']).next().unwrap_or(url);
    no_query.rsplit('@').next().unwrap_or(no_query).to_string()
}

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

    let report = HrService::backfill_missing_leave_pins(&pool, dry_run)
        .await
        .context("backfill missing leave pins")?;

    let tag = if dry_run { "[dry-run] " } else { "" };
    let verb = if dry_run { "將建立" } else { "已建立" };

    // 單位一律用「張假單」：report 的每個計數欄位都是以假單為單位，不是通知列數
    // （一張假單這一關可能有多位收件人、建立多則待辦）。這支工具的重點就是讓操作者
    // 能拿數字對帳，單位含糊會直接毀掉那個用途（CodeRabbit review）。
    println!(
        "{tag}{verb}待辦的假單（共 {} 張，每張可能通知多位收件人）：",
        report.created.len()
    );
    for r in &report.created {
        println!(
            "  leave={} | {} ({}) | {} → {}",
            r.leave_id,
            r.applicant_name,
            r.status,
            r.recipient_role,
            r.recipient_names.join("、")
        );
    }

    // 「這張單這一關沒有任何人簽得下去」——本工具要偵測的病症本身，最優先印出。
    if !report.no_recipients.is_empty() {
        println!(
            "\n🚨 {} 張假單算不出任何合法審核人（連 admin 代批名單都空手）＝ 目前無人簽得下去，已略過：",
            report.no_recipients.len()
        );
        for leave_id in &report.no_recipients {
            println!("  leave={leave_id}");
        }
        println!("   請人工確認該部門主管 / 負責人 / 管理員帳號是否在職且設定正確。");
    }

    if report.missing_proxy > 0 {
        println!(
            "\n⚠️ {} 張假單為 PENDING_PROXY 卻缺 proxy_user_id，理論上不可能，已略過，請人工核對。",
            report.missing_proxy
        );
    }

    if report.state_changed > 0 {
        println!(
            "\nℹ️ {} 張假單在本次執行期間已離開待處理關卡（被正常操作處理掉／轉關），略過。",
            report.state_changed
        );
    }

    // 五個數字皆以假單為單位，加總應等於候選假單總數——這行就是給操作者對帳用的。
    let total = report.created.len() as i64
        + report.already_pinned
        + report.no_recipients.len() as i64
        + report.missing_proxy
        + report.state_changed;
    println!(
        "\n{tag}完成（單位：張假單）：{verb} {}；所有收件人皆已有待辦 {}；無合法審核人 {}；缺代理人 {}；狀態已變 {}；候選合計 {}",
        report.created.len(),
        report.already_pinned,
        report.no_recipients.len(),
        report.missing_proxy,
        report.state_changed,
        total
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
}
