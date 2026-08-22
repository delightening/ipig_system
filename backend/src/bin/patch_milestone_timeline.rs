//! 一次性：補正已匯入計畫的里程碑時間軸（patch）。更新 protocols.submitted_at/approved_at +
//! 重建匯入時間軸 protocol_activities（送件→執秘預審→獸醫審查→核准；委員 park）。
//! 僅 import_pending=true。委員一二審里程碑暫不寫（與委員意見一致 park）。
//!
//! 讀 {iacuc_no: {submitted_at, pre_review_at, vet_review_at, approved_at}} JSON。
//! ## Usage
//! ```bash
//! DATABASE_URL_FILE=../secrets/db_url_host.txt cargo run --bin patch_milestone_timeline -- \
//!   --file "<workspace-root>/_import-artifacts/protocol-milestone-<批次>.json" --dry-run
//! ```

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use std::collections::BTreeMap;
use uuid::Uuid;

use erp_backend::config::read_secret_strict;

const SYSTEM_USER_ID: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000001");

/// 單一里程碑步驟：(activity_type, from_value, to_value, date, label)。
type MilestoneStep<'a> = (
    &'a str,
    Option<&'a str>,
    &'a str,
    Option<NaiveDate>,
    &'a str,
);

#[derive(Deserialize)]
struct Milestones {
    submitted_at: Option<NaiveDate>,
    pre_review_at: Option<NaiveDate>,
    vet_review_at: Option<NaiveDate>,
    approved_at: Option<NaiveDate>,
}

fn arg_value(flag: &str) -> Option<String> {
    let a: Vec<String> = std::env::args().collect();
    a.iter()
        .position(|x| x == flag)
        .and_then(|i| a.get(i + 1).cloned())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let dry_run = std::env::args().any(|a| a == "--dry-run");
    // R85-7：中間 JSON 不進版控、住在 repo 外，故不給預設路徑（見 docs/design/README.md）。
    let path = arg_value("--file").context("必須以 --file <path> 指定里程碑 JSON 路徑")?;
    let map: BTreeMap<String, Milestones> = serde_json::from_str(
        &std::fs::read_to_string(&path).with_context(|| format!("read {path}"))?,
    )
    .context("milestone JSON 解析失敗")?;
    println!("讀到 {} 筆里程碑（{}）", map.len(), path);

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&read_secret_strict("DATABASE_URL")?)
        .await?;
    // SYSTEM actor 顯示資訊
    let sys: Option<(String, String)> =
        sqlx::query_as("SELECT COALESCE(display_name,email), email FROM users WHERE id=$1")
            .bind(SYSTEM_USER_ID)
            .fetch_optional(&pool)
            .await?;
    let (sys_name, sys_email) = sys.unwrap_or_else(|| ("系統".to_string(), String::new()));

    let (mut ok, mut skip, mut err) = (0u32, 0u32, 0u32);
    for (iacuc, m) in &map {
        let row: Option<(Uuid, bool)> = sqlx::query_as(
            "SELECT id, import_pending FROM protocols WHERE iacuc_no=$1 AND status <> 'DELETED'",
        )
        .bind(iacuc)
        .fetch_optional(&pool)
        .await?;
        let Some((id, pending)) = row else {
            skip += 1;
            eprintln!("SKIP {iacuc} | 查無");
            continue;
        };
        if !pending {
            skip += 1;
            eprintln!("SKIP {iacuc} | 非 import_pending");
            continue;
        }

        // 時間軸里程碑（委員 park）：(activity_type, from, to, date, label)
        let steps: Vec<MilestoneStep> = vec![
            (
                "SUBMITTED",
                None,
                "SUBMITTED",
                m.submitted_at,
                "[歷史匯入] 申請送件",
            ),
            (
                "STATUS_CHANGED",
                Some("SUBMITTED"),
                "PRE_REVIEW",
                m.pre_review_at,
                "[歷史匯入] 執行秘書行政預審",
            ),
            (
                "STATUS_CHANGED",
                Some("PRE_REVIEW"),
                "VET_REVIEW",
                m.vet_review_at,
                "[歷史匯入] 獸醫師審查",
            ),
            (
                "APPROVED",
                Some("VET_REVIEW"),
                "APPROVED",
                m.approved_at,
                "[歷史匯入] 核准（主席同意函）",
            ),
        ];
        let present: Vec<_> = steps.iter().filter(|s| s.3.is_some()).collect();

        if dry_run {
            ok += 1;
            let tl: Vec<String> = present
                .iter()
                .map(|s| format!("{}={}", s.2, s.3.expect("present 已過濾 date.is_some()")))
                .collect();
            println!("[dry-run] {iacuc} ({id}) → {}", tl.join(" → "));
            continue;
        }

        // 單筆包成 transaction；失敗只記該筆 err，不中斷整批。
        let res: Result<()> = async {
            let mut tx = pool.begin().await?;
            sqlx::query("UPDATE protocols SET submitted_at=$1, approved_at=$2, updated_at=now() WHERE id=$3 AND import_pending=true")
                .bind(m.submitted_at).bind(m.approved_at).bind(id)
                .execute(&mut *tx).await?;
            // 清掉匯入建立的時間軸（重建）
            sqlx::query("DELETE FROM protocol_activities WHERE protocol_id=$1 AND (remark LIKE '[歷史匯入]%' OR remark LIKE '歷史補登匯入%')")
                .bind(id).execute(&mut *tx).await?;
            for (seq, (atype, from_v, to_v, date, label)) in (0_i64..).zip(&present) {
                let d = date.expect("present 已過濾 date.is_some()");
                let ts = d
                    .and_hms_opt(0, 0, 0)
                    .expect("00:00:00 為合法時間")
                    + chrono::Duration::seconds(seq);
                sqlx::query(
                    r#"INSERT INTO protocol_activities
                       (protocol_id, activity_type, actor_id, actor_name, actor_email, from_value, to_value, remark, created_at)
                       VALUES ($1, $2::protocol_activity_type, $3, $4, $5, $6, $7, $8, $9)"#,
                )
                .bind(id).bind(atype).bind(SYSTEM_USER_ID).bind(&sys_name).bind(&sys_email)
                .bind(*from_v).bind(*to_v).bind(*label)
                .bind(ts.and_utc())
                .execute(&mut *tx).await?;
            }
            tx.commit().await?;
            Ok(())
        }
        .await;
        match res {
            Ok(()) => {
                ok += 1;
                let tl: Vec<String> = present
                    .iter()
                    .map(|s| format!("{}={}", s.2, s.3.expect("present 已過濾 date.is_some()")))
                    .collect();
                println!("OK {iacuc} ({id}) → {}", tl.join(" → "));
            }
            Err(e) => {
                err += 1;
                eprintln!("ERR {iacuc} ({id}): {e}");
            }
        }
    }
    println!(
        "\n完成：補正 {ok}，跳過 {skip}，失敗 {err}（共 {} 筆）",
        map.len()
    );
    Ok(())
}
