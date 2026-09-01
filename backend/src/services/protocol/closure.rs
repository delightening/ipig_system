//! 結案雙簽（設計 A）：PI 與 SD 各簽一次，兩簽齊備才允許轉 `CLOSED`。
//!
//! 設計文件：`docs/design/protocol-sd-close-and-handover.md` §5
//!
//! # 這個模組存在的理由
//!
//! `protocols.close_pi_signature_id` / `close_sd_signature_id` 是外鍵，而
//! **外鍵只保證「那一列簽章存在」**——不保證它是這份計畫的、結案用的、還有效的。
//! 只要有任何路徑能把一張既有簽章的 id 寫進那兩欄（程式 bug、資料修補、
//! 日後新增的端點），「兩欄皆非 NULL」這個判準就會放行。
//!
//! 所以 gate 寫在這裡，用一條 SQL 一次驗完 7 條條件（§5.2a），
//! 而不是散在呼叫端各自檢查。

use serde_json::Value as JsonValue;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::status::ensure_no_live_animals_for_closure;
use super::ProtocolService;
use crate::{
    middleware::ActorContext,
    models::{ChangeStatusRequest, Protocol, ProtocolStatus},
    services::{SignatureService, SignatureType},
    AppError, Result,
};

/// 簽哪一邊。
///
/// ⚠️ 兩支端點刻意分開（設計文件 §5.7 紅線 3），**不做成「依角色自動判斷要寫哪一欄」**。
/// 分開才能讓每支各自只有一種權責檢查、各自只寫一欄，也才擋得住「PI 誤觸 SD 那一簽」。
/// 這個 enum 只是讓兩支共用底下的 tx 流程，不是把它們合併回一支。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosureSigner {
    Pi,
    StudyDirector,
}

impl ClosureSigner {
    fn label(self) -> &'static str {
        match self {
            ClosureSigner::Pi => "計畫主持人（PI）",
            ClosureSigner::StudyDirector => "計劃負責人（Study Director）",
        }
    }
}

/// 結案簽章專用的 `entity_type`（裁定 5）。
///
/// ⚠️ **刻意與核准簽章的 `"protocol"` 分開。**
/// `status.rs` 的核准前檢查是「該 protocol 有任何一張 `entity_type='protocol'`
/// 的有效簽章即通過」。結案簽章若也寫 `'protocol'`，一份計畫只要被結案簽過，
/// 就等於預先滿足了核准閘門。
///
/// 分開的代價是：**任何「這份計畫有哪些簽章」的查詢若只查 `'protocol'`，
/// 結案雙簽會靜默消失**（不是報錯，是查不到）。新增這類查詢時兩種都要查。
pub const CLOSURE_ENTITY_TYPE: &str = "protocol_closure";

/// 雙簽是否齊備且有效。呼叫端必須**已經** `SELECT ... FOR UPDATE` 鎖住該 protocol。
///
/// # 7 條條件（設計文件 §5.2a）
///
/// | # | 條件 | 防的是 |
/// |---|---|---|
/// | 1 | 兩欄皆非 NULL | 缺簽 |
/// | 2 | `entity_type = 'protocol_closure'` | 把核准簽章掛過來充數 |
/// | 3 | `entity_id = 本 protocol.id` | 把**別份計畫**的結案簽章掛過來 |
/// | 4 | `signature_type = 'CONFIRM'` | 語意錯置（§11.50 的 meaning） |
/// | 5 | `is_valid = true` | 已被 `invalidate` 作廢的簽章仍放行 |
/// | 6 | PI 那張的 signer 對得上 `pi_user_id`，或是持有指向本計畫的代理授權（`protocol_pi_delegates`，見 migration 010）代簽；SD 那張對得上 `study_director_user_id` | 兩欄互換、無關人員代簽 |
/// | 7 | 兩張的 signer **不同人** | 自簽自證 |
///
/// # ⚠️ 為什麼要對簽章列也下 `FOR UPDATE`
///
/// 條件 5 讀的 `is_valid` 會被 `SignatureService::invalidate_tx` 改。沒有鎖的話存在
/// TOCTOU：「gate 檢查通過 → 另一個 transaction 把簽章作廢 → 本 transaction 提交結案」。
/// 比照 `status.rs` 既有核准閘門的 `FOR UPDATE` 寫法。
///
/// ⚠️ `FOR UPDATE` 不能跟 `LEFT JOIN` 的 nullable 側一起用（Postgres 會報
/// "FOR UPDATE cannot be applied to the nullable side of an outer join"），
/// 所以這裡先取兩個 id、再用 `= ANY` 一次鎖住並讀回，而不是 join 回 protocols。
pub async fn dual_signature_ready(
    tx: &mut Transaction<'_, Postgres>,
    protocol_id: Uuid,
    pi_user_id: Uuid,
    study_director_user_id: Option<Uuid>,
    close_pi_signature_id: Option<Uuid>,
    close_sd_signature_id: Option<Uuid>,
) -> Result<bool> {
    // 條件 1：兩欄皆非 NULL。順帶擋掉「計畫根本沒有 SD」——沒有 SD 就不會有 SD 那一簽。
    let (Some(pi_sig_id), Some(sd_sig_id), Some(sd_user_id)) = (
        close_pi_signature_id,
        close_sd_signature_id,
        study_director_user_id,
    ) else {
        return Ok(false);
    };

    // 條件 7 的前置：兩欄指向同一張簽章時，signer 必然相同，直接擋。
    // 單獨列出是因為下面的查詢用 `= ANY`，同一個 id 只會回一列，
    // 「回傳列數 == 2」的檢查會失敗但錯誤原因看不出來。
    if pi_sig_id == sd_sig_id {
        return Ok(false);
    }

    // 條件 2/3/4/5 在 WHERE 裡驗；FOR UPDATE 鎖住這兩列直到本 tx 結束。
    let rows: Vec<(Uuid, Uuid, Option<Uuid>)> = sqlx::query_as(
        r#"SELECT id, signer_id, delegation_id
           FROM electronic_signatures
           WHERE id = ANY($1)
             AND entity_type = $2
             AND entity_id = $3
             AND signature_type = 'CONFIRM'
             AND is_valid = true
           FOR UPDATE"#,
    )
    .bind(vec![pi_sig_id, sd_sig_id])
    .bind(CLOSURE_ENTITY_TYPE)
    // entity_id 在 electronic_signatures 是 text（該表服務多種 entity），故轉字串比對
    .bind(protocol_id.to_string())
    .fetch_all(&mut **tx)
    .await?;

    // 兩張都要通過上面的過濾才算數
    if rows.len() != 2 {
        return Ok(false);
    }

    // 條件 6：各自的 signer 要對得上各自的角色。
    // ⚠️ 這裡不能只檢查「兩個 signer 分別是 PI 與 SD」——那樣 PI 簽在 SD 欄、
    // SD 簽在 PI 欄也會過。要逐欄對應。
    let signer_of = |id: Uuid| {
        rows.iter()
            .find(|(sig_id, _, _)| *sig_id == id)
            .map(|(_, s, _)| *s)
    };
    let delegation_of = |id: Uuid| {
        rows.iter()
            .find(|(sig_id, _, _)| *sig_id == id)
            .and_then(|(_, _, d)| *d)
    };
    let (Some(pi_signer), Some(sd_signer)) = (signer_of(pi_sig_id), signer_of(sd_sig_id)) else {
        return Ok(false);
    };

    // PI 那欄：本人簽，或持有指向本計畫、指名這位 signer 的代理授權（migration 010）。
    // ⚠️ 不檢查該筆授權是否仍生效中（`revoked_at IS NULL`）——撤銷是「今後不能再用
    // 這筆授權簽新東西」，不是讓已經簽下的既有簽章事後失真。這裡只驗簽章當下
    // 是否真的依這筆授權簽的（授權存在、屬於本計畫、代理人與 signer 一致）。
    let pi_signer_authorized = if pi_signer == pi_user_id {
        true
    } else if let Some(delegation_id) = delegation_of(pi_sig_id) {
        let (linked,): (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM protocol_pi_delegates
                WHERE id = $1 AND protocol_id = $2 AND delegate_user_id = $3
            )"#,
        )
        .bind(delegation_id)
        .bind(protocol_id)
        .bind(pi_signer)
        .fetch_one(&mut **tx)
        .await?;
        linked
    } else {
        false
    };
    if !pi_signer_authorized || sd_signer != sd_user_id {
        return Ok(false);
    }

    // 條件 7：兩簽不同人。
    //
    // ⚠️ 這條在 PI≠SD 規則（裁定 16）之外**仍然必要**，不是重複檢查：
    // 那條規則管的是「指派當下」，而 `pi_user_id` 與 `study_director_user_id`
    // 可能在指派之後才變成同一人（存量資料、或日後放寬規則）。
    // 雙簽的意義是兩個人各自具結，同一人簽兩次不構成雙簽。
    Ok(pi_signer != sd_signer)
}

/// 允許開始簽結案的狀態（設計文件 §5.2）。
///
/// ⚠️ 不含 `SUSPENDED` 等其他狀態機允許轉 `CLOSED` 的來源——那些仍走完整
/// `change_status` 權限，不開放給雙簽這條窄路。
pub fn can_start_closure_signing(status: ProtocolStatus) -> bool {
    matches!(
        status,
        ProtocolStatus::Approved | ProtocolStatus::ApprovedWithConditions
    )
}

/// 簽結案前的狀態與補登檢查（裁定 8）。
///
/// ⚠️ **`import_pending` 期間一律不得簽結案**（裁定 8，2026-08-25）。
/// 理由是「要正式開始才能結案」。
///
/// 這條同時解掉設計文件 §5.2b 指出的脫節問題：`APPROVED + import_pending`
/// 的計畫可以改 `working_content` 也可以改 SD，若允許此時簽結案，
/// 會出現「PI 簽的是改之前那份內容、SD 簽的是改之後」而 gate 照樣全過。
///
/// ⚠️ 影響面很大而且要講清楚：2026-08-26 實測正式庫 33 份已核准計畫中
/// **32 份仍 `import_pending`**，所以結案功能上線後對絕大多數計畫暫時不可用，
/// 要等補登完成。這是裁定 8 已知並接受的代價。
pub fn ensure_closure_signable(status: ProtocolStatus, import_pending: bool) -> Result<()> {
    if !can_start_closure_signing(status) {
        return Err(AppError::BusinessRule(format!(
            "只有已核准（含附條件）的計畫可以簽結案，目前狀態為「{}」",
            status.display_name()
        )));
    }
    if import_pending {
        return Err(AppError::BusinessRule(
            "補登匯入尚未完成的計畫不得簽結案。請先完成補登，計畫正式開始後才能結案。".into(),
        ));
    }
    Ok(())
}

/// 簽一次結案，並在雙簽齊備時**自動轉 `CLOSED`**（裁定 9）。
///
/// # 為什麼是「自動轉」而不是「簽完再按結案鈕」
///
/// 裁定 9（2026-08-25）採設計文件 §5.6a 的做法 (i)。理由是那份文件指出的
/// R98-1 復活問題：SD 是 `EXPERIMENT_STAFF`，而 `handlers/protocol/crud.rs` 與
/// `status.rs` 兩處都要求 `close_own` 才按得下結案——**SD 簽完名之後仍然按不下**。
/// 做法 (ii)「只讓 PI 按」則把死鎖從 SD 側搬到 PI 側。
///
/// 讓第二簽落地時 service 自動轉，就不存在「誰按下結案」這個動作，
/// 語意也最貼近雙簽的意義：**兩簽齊備＝結案成立**。
///
/// # 原子性
///
/// 建簽章、寫欄位、（必要時）轉狀態全部在同一個 tx 內。中途失敗整個 rollback，
/// 不會留下「簽了但沒記錄」或「記錄了但沒轉狀態」的半套。
///
/// # ⚠️ 呼叫端責任
///
/// **權責檢查不在這裡。** `sign_record_tx` 不驗權責（設計文件 §3.3），本函式也不驗——
/// 呼叫端（兩支 handler）各自檢查 `actor.id == pi_user_id` 或
/// `actor.id == study_director_user_id`（或 PI 那一支另外接受生效中的
/// `protocol_pi_delegates` 代理人，見 `delegation_id` 參數）。這是刻意的：權責屬
/// HTTP 端點的語意，而這裡是共用流程。**新增呼叫端時必須自己補權責檢查。**
#[allow(clippy::too_many_arguments)]
pub async fn sign_closure(
    pool: &PgPool,
    actor: &ActorContext,
    protocol_id: Uuid,
    signer: ClosureSigner,
    signer_id: Uuid,
    // 非 NULL = `signer_id` 是依此筆 `protocol_pi_delegates` 授權代簽（僅
    // `ClosureSigner::Pi` 有意義）。呼叫端必須已驗證這筆授權生效中、屬於本計畫、
    // `delegate_user_id == signer_id`——本函式只負責把它綁進簽章紀錄。
    delegation_id: Option<Uuid>,
    password: Option<&str>,
    handwriting_svg: Option<&str>,
    stroke_data: Option<&JsonValue>,
) -> Result<Protocol> {
    let mut tx = pool.begin().await?;

    // FOR UPDATE：從這裡到 commit，這份 protocol 的兩個簽章欄與狀態都不會被別人改。
    // 少了它會有「兩人同時簽第二簽 → 兩邊都判定齊備 → 重複轉狀態」。
    let before = sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
        .bind(protocol_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到計劃書".into()))?;

    ensure_closure_signable(before.status, before.import_pending)?;

    // SD 不得以 PI 代理人身分簽 PI 那一欄（CodeRabbit #53）。
    //
    // `authorize_pi_delegate` 刻意允許「SD 自任代理人」這個組合（改由執秘/admin 核准，
    // 避免自簽自證），那條決策對安樂死核准、修正案寫入等用途仍然成立——那些動作
    // 只需要一個有權責的人，不要求兩人。**但結案雙簽要求兩人各自具結**
    // （`dual_signature_ready` 條件 7：`pi_signer != sd_signer`），SD 若先以代理人
    // 身分簽 PI 欄、再簽 SD 欄，兩張的 signer 是同一人，gate 永遠回 false，
    // 計畫**再也進不了 `CLOSED`**。
    //
    // 擋在這裡而不是擋在核准端（CodeRabbit 的原始建議）：核准端一擋，SD 自任代理人
    // 這個組合就整個消失，連用不到雙簽的那些用途也一併沒了。擋在簽署端只否決
    // 真正衝突的那一個動作，其餘照舊。
    //
    // ⚠️ 必須擋在建立簽章**之前**：PI 欄的簽章一旦寫下去就佔住欄位，脫困要撤銷授權
    // 加作廢簽章，是人工修資料等級的成本。
    if matches!(signer, ClosureSigner::Pi)
        && delegation_id.is_some()
        && before.study_director_user_id == Some(signer_id)
    {
        return Err(AppError::BusinessRule(
            "計劃負責人（SD）不可以 PI 代理人身分簽結案：結案雙簽要求兩人各自具結，\
             同一人簽兩欄不構成雙簽。請改由 SD 以外的代理人簽 PI 那一欄。"
                .into(),
        ));
    }

    // 動物守門前置（CodeRabbit #38）：與 `change_status_tx` 的 CLOSED 分支同一道檢查，
    // 但在這裡先擋一次，讓簽署人在**簽之前**就知道還有動物在場——而不是簽完才發現
    // 因為動物守門把整個 tx（含剛建立的簽章）一起 rollback 掉。
    // `change_status_tx` 仍會在雙簽齊備轉狀態時再驗一次，真正的不變式保證在那裡。
    ensure_no_live_animals_for_closure(&mut tx, before.id, before.iacuc_no.as_deref()).await?;

    // 各自擋重複簽（設計文件 §5.2，同 disposal.rs 的形狀）。
    //
    // ⚠️ 只擋「仍然有效」的既有簽章（CodeRabbit #38）：`SignatureService::invalidate_tx`
    // 只改 `electronic_signatures.is_valid`，不會清空這裡的 `close_pi_signature_id` /
    // `close_sd_signature_id`。若只看欄位是否非 NULL，一旦某張結案簽章被作廢
    // （簽錯人、行政撤銷），這個位置就永久卡死、連本人都補不回來，只能手動修資料。
    let already = match signer {
        ClosureSigner::Pi => before.close_pi_signature_id,
        ClosureSigner::StudyDirector => before.close_sd_signature_id,
    };
    if let Some(existing_sig_id) = already {
        let existing_is_valid: bool =
            sqlx::query_scalar("SELECT is_valid FROM electronic_signatures WHERE id = $1")
                .bind(existing_sig_id)
                .fetch_one(&mut *tx)
                .await?;
        if existing_is_valid {
            return Err(AppError::BusinessRule(format!(
                "{}已經簽過結案，不可重複簽署。",
                signer.label()
            )));
        }
        // 既有簽章已作廢：視為未簽，讓下面的寫入覆蓋掉那個欄位。
    }

    // 簽章內容綁計畫識別 + 標題 + 狀態，與既有 `fetch_protocol_content` 同構，
    // 但**加上 closure 前綴**——同一份計畫的核准簽章與結案簽章不該產生相同的
    // content_hash，否則稽核比對時分不出這個雜湊對應哪一件事。
    let content = format!(
        "protocol_closure:{},title:{},status:{}",
        before.id,
        before.title,
        before.status.as_str()
    );

    // ⚠️ `Confirm` 不是 `Approve`（設計文件 §5.2）：結案雙簽的語意是雙方確認
    // 試驗已完成（§11.50 "responsibility"），不是審查方核准（"approval"）。
    // 用錯 meaning 會讓稽核報表把結案簽章與 IACUC 核准簽章混為一談。
    let signature = if let Some(delegation_id) = delegation_id {
        SignatureService::sign_record_delegated_tx(
            &mut tx,
            pool,
            actor,
            CLOSURE_ENTITY_TYPE,
            &protocol_id.to_string(),
            signer_id,
            delegation_id,
            SignatureType::Confirm,
            &content,
            password,
            handwriting_svg,
            stroke_data,
        )
        .await?
    } else {
        SignatureService::sign_record_tx(
            &mut tx,
            pool,
            actor,
            CLOSURE_ENTITY_TYPE,
            &protocol_id.to_string(),
            signer_id,
            SignatureType::Confirm,
            &content,
            password,
            handwriting_svg,
            stroke_data,
        )
        .await?
    };

    let (pi_sig, sd_sig) = match signer {
        ClosureSigner::Pi => (Some(signature.id), before.close_sd_signature_id),
        ClosureSigner::StudyDirector => (before.close_pi_signature_id, Some(signature.id)),
    };

    // 先寫簽章欄。狀態轉移**不在這裡做**——見下面。
    let updated = sqlx::query_as::<_, Protocol>(
        r#"UPDATE protocols
           SET close_pi_signature_id = $2,
               close_sd_signature_id = $3,
               updated_at = NOW()
           WHERE id = $1
           RETURNING *"#,
    )
    .bind(protocol_id)
    .bind(pi_sig)
    .bind(sd_sig)
    .fetch_one(&mut *tx)
    .await?;

    let ready = dual_signature_ready(
        &mut tx,
        updated.id,
        updated.pi_user_id,
        updated.study_director_user_id,
        updated.close_pi_signature_id,
        updated.close_sd_signature_id,
    )
    .await?;

    let result = if ready {
        // 🔴 **走 `change_status_tx`，不要自己下 UPDATE 轉狀態。**
        //
        // 第一版我寫成一條 UPDATE 同時寫簽章欄與 `status = CASE WHEN ready THEN 'CLOSED'`。
        // 那樣是原子的沒錯，但**跳過了狀態機驗證（`can_change_status_to`）、
        // 活動紀錄、稽核鏈與通知**——等於在 `change_status` 之外開了第二條轉 CLOSED
        // 的路徑，正是設計文件 §5.6a 警告的那件事（「所有直接呼叫路徑都必須套用
        // 同一套規則」）。
        //
        // 走這裡之後，授權來源只有一個：`change_status_tx` 內的雙簽 gate。
        // 簽章欄已在同一個 tx 內寫好，所以那個 gate 會讀到齊備的狀態而放行——
        // **不需要也不應該傳任何「我已經驗過了」的旗標進去**，那種旗標就是繞過本身。
        // ⚠️ 四個欄位全部明確列出，不用 `..Default::default()`——
        // `ChangeStatusRequest` 沒有 derive `Default`，而**刻意不幫它加**：
        // 那是 production model，加了之後別處會出現「忘了填 to_status 也能編譯」
        // 的請求物件，而 `to_status` 是這個型別唯一沒有合理預設值的欄位。
        let req = ChangeStatusRequest {
            to_status: ProtocolStatus::Closed,
            remark: Some(format!("結案雙簽齊備（{}完成第二簽）", signer.label())),
            reviewer_ids: None,
            vet_id: None,
        };
        ProtocolService::change_status_tx(&mut tx, actor, protocol_id, &req).await?
    } else {
        updated
    };

    tx.commit().await?;
    Ok(result)
}
