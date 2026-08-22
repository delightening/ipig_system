//! 一次性：匯入「動物試驗申請須知」歷史 4 版（AD-04-01-02 A/B/C/D）。
//!
//! A/B/C 登記為歷史封存版（`is_active=false`），D 為當前生效版（送審簽署閘門指向 D）。
//! 內容取自 `Downloads\計劃書匯入\申請須知` 之 PDF（A/B/C）與 docx（D），轉為純文字正文
//! （前端以 `whitespace-pre-wrap` 呈現，故不用 markdown 語法；表格改條列、加分隔線以利閱讀）。
//! 走 `ApplicationNoticeService::create` / `update_content` / `activate`（ActorContext::System），
//! 產生完整 audit + HMAC chain。
//!
//! Idempotent + 內容同步：`version_label` 已存在且正文相同 → 跳過；正文不同 → 更新正文
//! （限尚無簽署引用，由 service 守衛）；不存在 → 建立。可安全重跑。
//!
//! ## Usage
//! ```bash
//! # 本機（.env 提供 DATABASE_URL）：
//! cargo run --bin import_application_notices -- --dry-run   # 僅預覽
//! cargo run --bin import_application_notices                # 執行
//! # prod 容器內（compose 已設 DATABASE_URL_FILE）：
//! docker compose exec api /app/import_application_notices --dry-run
//! docker compose exec api /app/import_application_notices
//! ```

use anyhow::{Context, Result};
use chrono::NaiveDate;
use sqlx::postgres::PgPoolOptions;

use erp_backend::config::{read_audit_hmac_key_strict, read_secret_strict};
use erp_backend::models::CreateApplicationNoticeRequest;
use erp_backend::repositories::application_notice::ApplicationNoticeRepository;
use erp_backend::services::{ApplicationNoticeService, AuditService};
use erp_backend::ActorContext;

/// 單一須知版本的種子資料。
struct NoticeSeed {
    version_label: &'static str,
    title: &'static str,
    /// (year, month, day)
    effective_from: (i32, u32, u32),
    content: &'static str,
    /// 是否設為當前生效版本（僅 D 為 true）。
    activate: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let dry_run = std::env::args().any(|a| a == "--dry-run");

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&read_secret_strict("DATABASE_URL")?)
        .await
        .context("Failed to connect to database")?;

    println!(
        "申請須知匯入：共 {} 版（A/B/C 封存、D 生效）",
        NOTICES.len()
    );
    if dry_run {
        print_dry_run(NOTICES);
        return Ok(());
    }

    init_audit_hmac()?;
    let actor = ActorContext::System {
        reason: "import_application_notices",
    };
    let (mut created, mut updated, mut skipped, mut activated) = (0u32, 0u32, 0u32, 0u32);
    for s in NOTICES {
        let (outcome, did_activate) = import_one(&pool, &actor, s).await?;
        match outcome {
            Outcome::Created => created += 1,
            Outcome::Updated => updated += 1,
            Outcome::Skipped => skipped += 1,
        }
        if did_activate {
            activated += 1;
        }
    }

    println!(
        "\n完成：建立 {created}、更新正文 {updated}、跳過 {skipped}、啟用 {activated}。\
         送審簽署閘門現指向生效版本。"
    );
    Ok(())
}

/// 單一版本的匯入結果。
enum Outcome {
    Created,
    Updated,
    Skipped,
}

/// `--dry-run` 預覽：僅列出將寫入的版本，不連動任何寫入。
fn print_dry_run(notices: &[NoticeSeed]) {
    for s in notices {
        let (y, m, d) = s.effective_from;
        println!(
            "[dry-run] {} 「{}」 生效日 {:04}-{:02}-{:02} {} 正文 {} 字",
            s.version_label,
            s.title,
            y,
            m,
            d,
            if s.activate { "[啟用]" } else { "[封存]" },
            s.content.chars().count(),
        );
    }
    println!("[dry-run] 未執行任何寫入。");
}

/// 載入 HMAC 金鑰，讓本次寫入的 audit row 正確進 HMAC chain（否則 integrity_hash=NULL）。
fn init_audit_hmac() -> Result<()> {
    match read_audit_hmac_key_strict()? {
        Some(key) => AuditService::init_hmac_key(Some(key)),
        None => {
            eprintln!(
                "⚠️  AUDIT_HMAC_KEY（或 AUDIT_HMAC_KEY_FILE，≥44 字元）未設定；\
                 本次寫入的 audit 將不進 HMAC chain（integrity_hash=NULL）。"
            );
            AuditService::init_hmac_key(None);
        }
    }
    Ok(())
}

/// 匯入單一版本（idempotent）：不存在則建立；已存在但正文不同則更新正文（限未簽署）；
/// 完全相同則跳過。回傳 `(結果, 是否設為生效)`。
async fn import_one(
    pool: &sqlx::PgPool,
    actor: &ActorContext,
    s: &NoticeSeed,
) -> Result<(Outcome, bool)> {
    let existing = ApplicationNoticeRepository::find_by_version_label(pool, s.version_label)
        .await
        .with_context(|| format!("lookup failed for {}", s.version_label))?;

    let (id, outcome) = match existing {
        Some(n) if n.content == s.content => {
            println!("SKIP {} 已存在（內容相同）", s.version_label);
            (n.id, Outcome::Skipped)
        }
        Some(n) => {
            let updated = ApplicationNoticeService::update_content(pool, actor, n.id, s.content)
                .await
                .with_context(|| format!("update content failed for {}", s.version_label))?;
            println!("UPD  {} 正文已更新", s.version_label);
            (updated.id, Outcome::Updated)
        }
        None => {
            let (y, m, d) = s.effective_from;
            let effective_from = NaiveDate::from_ymd_opt(y, m, d)
                .with_context(|| format!("invalid date for {}", s.version_label))?;
            let req = CreateApplicationNoticeRequest {
                version_label: s.version_label.to_string(),
                title: s.title.to_string(),
                content: s.content.to_string(),
                effective_from,
                attachment_id: None,
            };
            let notice = ApplicationNoticeService::create(pool, actor, &req)
                .await
                .with_context(|| format!("create failed for {}", s.version_label))?;
            println!("OK   {} 已建立", s.version_label);
            (notice.id, Outcome::Created)
        }
    };

    let mut activated = false;
    if s.activate {
        ApplicationNoticeService::activate(pool, actor, id)
            .await
            .with_context(|| format!("activate failed for {}", s.version_label))?;
        println!("→    {} 已設為當前生效版本", s.version_label);
        activated = true;
    }
    Ok((outcome, activated))
}

// ───────────────────────────────────────── 種子資料 ─────────────────────────────────────────

const NOTICES: &[NoticeSeed] = &[
    NoticeSeed {
        version_label: "AD-04-01-02",
        title: "動物試驗申請須知",
        effective_from: (2020, 12, 15),
        activate: false,
        content: CONTENT_A,
    },
    NoticeSeed {
        version_label: "AD-04-01-02B",
        title: "動物試驗申請須知",
        effective_from: (2023, 4, 12),
        activate: false,
        content: CONTENT_B,
    },
    NoticeSeed {
        version_label: "AD-04-01-02C",
        title: "動物試驗研究申請須知",
        effective_from: (2024, 11, 26),
        activate: false,
        content: CONTENT_C,
    },
    NoticeSeed {
        version_label: "AD-04-01-02D",
        title: "動物試驗研究申請須知",
        effective_from: (2025, 9, 15),
        activate: true,
        content: CONTENT_D,
    },
];

const CONTENT_A: &str = "動物試驗申請須知

1. 每一試驗須提出一份動物試驗申請表，使用本公司之制式表單，由試驗（委託）單位依內容要求詳實填寫並符合相關法規，經本公司「實驗動物照護及使用委員會（IACUC）」審核通過後，方可進行試驗；並請檢附已於其他單位通過之計畫書／同意書當作申請參考附件。

2. 動物試驗申請表中操作步驟應依實際執行情形詳實填寫，若需進行修正，應於實驗進行前依規定提出申請，重新審核通過。若發現實際執行內容與申請表內容不符，本公司將有權要求實驗暫停，待修正申請審核通過後，方可繼續進行。

3. 進入手術房之人員均視為動物試驗操作相關人員，皆必須經過本公司審核通過，並接受訓練及考核後，方可參與動物試驗操作進行；本公司有權拒絕核可名單外之人員進入手術房。參與試驗人員名單與申請表不符時，應於試驗進行前 1 週提出申請變更。

4. 參與試驗人員若試驗進行 3 天內曾進入其他豬隻實驗室，不得進入本公司試驗區域。

5. 動物試驗相關時間表如下，請申請人注意時間規劃，儘早提出申請，確保試驗之順利進行。
   ・動物試驗申請表／修正單審查：2～4 週
   ・動物試驗開始進行時間：申請表審查通過後 2 週
   ・實驗動物準備：2 週
   ・假日試驗時間排定：試驗 2 週前排定

6. 為符合動物福祉，本公司飼養欄可提供體重 130 公斤、體長 150 公分以下實驗豬隻飼養照護。如果飼養觀察期間超過 6 個月，不建議使用白豬為實驗動物，建議以迷你豬進行試驗。若在試驗過程中豬隻超過上限，本公司獸醫師有權通知實驗者提前結束實驗。

7. 術後或實驗期間動物體重下降超過原體重的 20%、食慾不振（無法進食）、身體虛弱、感染，持續治療或傷口清創後無改善，本公司獸醫師將發出人道犧牲通知書通知實驗者提前結束實驗，實驗者應遵守相關規定，以符合動物福祉。

8. 動物試驗申請表審查費（5000 元／件），動物試驗申請表修正審查費（3000 元／次／件）。

9. 試驗（委託）單位願意遵守以上規定，並接受監督。

────────────────────────────────────────

試驗（委託）單位：________________________________

主要研究員：________________________________（簽章／日期）

試驗（委託）單位代表人：________________________________（簽章／日期）

（2020 年 12 月修訂）
";

const CONTENT_B: &str = "動物試驗申請須知

1. 每一試驗須提出一份動物試驗研究計畫書，使用本公司之制式表單，由委託單位依內容要求詳實填寫並符合相關法規，經本公司「實驗動物照護及使用小組（IACUC）」審核通過後，方可進行試驗；並請檢附已於其他單位通過之計畫書／同意書當作申請參考附件。

2. 動物試驗研究計畫書中操作步驟應依實際執行情形詳實填寫，若需進行修正，應於實驗進行前依規定提出申請，重新審核通過。若發現實際執行內容與申請表內容不符，本公司將有權要求實驗暫停，待修正申請審核通過後，方可繼續進行。

3. 進入手術房之人員均視為動物試驗操作相關人員，皆必須經過本公司審核通過，並接受訓練及考核後，方可參與動物試驗操作進行；本公司有權拒絕核可名單外之人員進入手術房。參與試驗人員名單與申請表不符時，應於試驗進行前 1 週提出申請變更。

4. 參與試驗人員若試驗進行 3 天內曾進入其他豬隻實驗室，不得進入本公司試驗區域。

5. 動物試驗相關時間表如下，請申請人注意時間規劃，儘早提出申請，確保試驗之順利進行。
   ・動物試驗研究計畫書／修正單審查：2～4 週
   ・動物試驗開始進行時間：申請表審查通過後 2 週
   ・實驗動物準備：2 週
   ・假日試驗時間排定：試驗 2 週前排定

6. 為符合動物福祉，本公司飼養欄可提供體重 130 公斤、體長 150 公分以下實驗豬隻飼養照護。如果飼養觀察期間超過 6 個月，不建議使用白豬為實驗動物，建議以迷你豬進行試驗。若在試驗過程中豬隻超過上限，本公司獸醫師有權通知實驗者提前結束實驗。

7. 術後或實驗期間動物體重下降超過原體重的 20%、食慾不振（無法進食）、身體虛弱、感染，持續治療或傷口清創後無改善，本公司獸醫師將發出人道犧牲通知書通知實驗者提前結束實驗，實驗者應遵守相關規定，以符合動物福祉。

8. 動物試驗申請表審查費（5000 元／件），動物試驗申請表修正審查費（3000 元／次／件）。

9. 委託單位願意遵守以上規定，並接受監督。

────────────────────────────────────────

委託單位（Sponsor Name）：________________________________

研究主持人（Study Director）：________________________________（簽章／日期）

研究委託者（Sponsor）：________________________________（簽章／日期）

（版權為豬博士動物科技股份有限公司所有，禁止任何未經授權的使用。All Rights Reserved © DrPIG. Unauthorized use in any form is prohibited.）
";

const CONTENT_C: &str = "動物試驗研究申請須知

1. 每一試驗須提出一份動物試驗研究計畫書，使用本公司之制式表單，由研究主持人／委託單位依內容要求詳實填寫並符合相關法規，經本公司「實驗動物照護及使用小組（IACUC）」審核通過後，方可進行試驗；如動物試驗已於其他單位申請通過，請檢附其他單位通過之計畫書／同意書當作申請參考附件。

2. 動物試驗研究計畫書中操作步驟應依實際執行情形詳實填寫，若需進行修正，應於實驗進行前依規定提出修正申請，重新審核通過。若發現實際執行內容與申請表內容不符，本公司將有權要求實驗暫停，待修正申請審核通過後，方可繼續進行。

3. 進入手術房之人員均視為動物試驗操作相關人員，皆必須經過本公司審核通過，並接受訓練及考核後，方可參與動物試驗操作進行；本公司有權拒絕核可名單外之人員進入手術房。參與試驗人員名單與申請表不符時，應於試驗進行前 1 週提出申請變更。

4. 參與試驗人員若試驗進行前 7 天曾進入其他動物設施，不得進入本公司試驗區域。

5. 動物試驗相關時間表如下，請申請人注意時間規劃，儘早提出申請，確保試驗之順利進行。
   ・動物試驗研究計畫書／修正單審查：4～6 週
   ・動物試驗開始進行時間：申請表審查通過後 2 週
   ・實驗動物準備：2 週
   ・假日試驗時間排定：試驗 2 週前排定

6. 為符合動物福祉，本公司飼養欄可提供體重 130 公斤、體長 150 公分以下實驗豬隻飼養照護。如果飼養觀察期間超過 6 個月，不建議使用白豬為實驗動物，建議以迷你豬進行試驗。若在試驗過程中豬隻超過上限，本公司獸醫師有權通知研究主持人／委託單位提前結束實驗。

7. 術後或實驗期間動物體重下降超過原體重的 20%、食慾不振（無法進食）、身體虛弱、感染，持續治療或傷口清創後無改善，本公司獸醫師將發出人道犧牲通知書提前結束實驗，研究主持人／委託單位應遵守相關規定，以符合動物福祉。

8. 計畫執行期限最長為申請 3 年。

9. 動物試驗申請表審查費（5000 元／件），動物試驗申請表修正審查費（3000 元／次／件）。

10. 研究主持人／委託單位在送出申請文件時，即負有法律責任「保證動物試驗研究計畫書所填資料完全屬實並確認此申請案之執行與運作符合『動物保護法』及相關法規之規定」。

11. 研究主持人／委託單位願意遵守以上規定，並接受監督。

────────────────────────────────────────

委託單位（Sponsor Name）：________________________________

研究委託者（Sponsor）：________________________________（簽章／日期）

研究主持人（Study Director）：________________________________（簽章／日期）

（版權為豬博士動物科技股份有限公司所有，禁止任何未經授權的使用。All Rights Reserved © DrPIG. Unauthorized use in any form is prohibited.）
";

const CONTENT_D: &str = "動物試驗研究申請須知

1. 每一試驗須提出一份動物試驗研究計畫書，使用本公司之制式表單，由計畫主持人／委託單位依內容要求詳實填寫並符合相關法規，經本公司「實驗動物照護及使用小組（IACUC）」審核通過後，方可進行試驗；如動物試驗已於其他單位申請通過，請檢附其他單位通過之計畫書／同意書當作申請參考附件。

2. 動物試驗研究計畫書中操作步驟應依實際執行情形詳實填寫，若需進行修正，應於實驗進行前依規定提出修正申請，重新審核通過。若發現實際執行內容與申請表內容不符，本公司將有權要求實驗暫停，待修正申請審核通過後，方可繼續進行。

3. 進入手術房之人員均視為動物試驗操作相關人員，皆必須經過本公司審核通過，並接受訓練及考核後，方可參與動物試驗操作進行；本公司有權拒絕核可名單外之人員進入手術房。參與試驗人員名單與申請表不符時，應於試驗進行前 1 週提出申請變更。

4. 參與試驗人員若試驗進行前 7 天曾進入其他動物設施，不得進入本公司試驗區域。

5. 動物試驗相關時間表如下，請申請人注意時間規劃，儘早提出申請，確保試驗之順利進行。
   ・動物試驗研究計畫書／修正單審查：4～6 週
   ・動物試驗開始進行時間：申請表審查通過後 2 週
   ・實驗動物準備：2 週
   ・假日試驗時間排定：試驗 2 週前排定

6. 為符合動物福祉，本公司飼養欄可提供體重 130 公斤、體長 150 公分以下實驗豬隻飼養照護。如果飼養觀察期間超過 6 個月，不建議使用白豬為實驗動物，建議以迷你豬進行試驗。若在試驗過程中豬隻超過上限，本公司獸醫師有權通知計畫主持人／委託單位提前結束實驗。

7. 術後或實驗期間動物體重下降超過原體重的 20%、食慾不振（無法進食）、身體虛弱、感染，持續治療或傷口清創後無改善，本公司獸醫師將發出人道犧牲通知書提前結束實驗，計畫主持人／委託單位應遵守相關規定，以符合動物福祉。

8. 計畫執行期限最長為申請 3 年。

9. 動物試驗申請表審查費（5000 元／件），動物試驗申請表修正審查費（3000 元／次／件）。

10. 計畫主持人／委託單位在送出申請文件時，即負有法律責任「保證動物試驗研究計畫書所填資料完全屬實並確認此申請案之執行與運作符合『動物保護法』及相關法規之規定」。

11. 計畫主持人／委託單位願意遵守以上規定，並接受監督。

────────────────────────────────────────

委託單位（Sponsor Name）：________________________________

計畫主持人（Principal Investigator）：________________________________（簽章／日期）
";
