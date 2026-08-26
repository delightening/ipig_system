//! 關卡待辦：把「這筆現在卡在誰手上」算出來，並把待處理清單**同步**到那個結果。
//!
//! # 為什麼是「同步」而不是「建立 / 解除」配對
//!
//! 既有的置頂待辦全部走「送出時建立、完成時解除」的配對寫法。2026-08-07 的巡場事故
//! 證明了那個形狀的代價：解除只掛在 happy path 上，撤回 / 刪除 / 作廢 / 轉單每多一條
//! 就多一次漏接機會，而漏接的後果是使用者的待辦永久卡死且**依設計不可手動清除**
//! （見 `reconcile.rs` 模組註解）。
//!
//! 本模組改成冪等的一次同步：**「這筆現在該在誰的待辦清單裡」是狀態的函數**，
//! 呼叫 [`NotificationService::sync_stage_todos_tx`] 就把清單對齊到那個答案——
//! 多的解除、缺的補上。呼叫端不必知道自己是哪一種轉換，也就沒有「這條路徑要不要
//! 記得解除」這個問題。同一筆重複呼叫不會產生第二則待辦。
//!
//! # 硬規則：收件人必須與該關卡的授權判準同源
//!
//! 每一關的權限碼 / 角色碼 / SoD 排除對象都標了抄自哪一行。分岔的後果是使用者收到
//! 一則點下去拿 403 的待辦——**比不通知更糟**，因為待辦不能手動清掉。
//!
//! ⚠️ 候選人查詢一律用 `repositories::pending_owner` 的三支
//! （它們處理了 `has_permission()` 對管理員短路這件事，見該檔註解），
//! 不要自己接 `user_roles` / `role_permissions`。
//!
//! # 與 `services/pending_owner.rs` 的分工
//!
//! 那邊算「卡在誰」給**列表頁顯示**（回人名與人數），這邊算「發給誰」給**待辦清單**。
//! 單據三關的判準已在那邊，故本模組直接呼叫它的
//! [`crate::services::pending_owner::stage_recipients_for_document`]，不重寫一份。
//! 其餘關卡的判準目前只有本模組有；`services/pending_owner` 日後展開到同樣的關卡時，
//! **兩邊必須收斂成一份**（TODO.md R112-7）。

use std::collections::HashSet;

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::constants::{ROLE_ADMIN_LEGACY, ROLE_ADMIN_STAFF, ROLE_SYSTEM_ADMIN};
use crate::error::AppError;
use crate::models::{
    CreateNotificationRequest, NotificationType, PRIORITY_NORMAL, PRIORITY_PINNED,
    RECIPIENT_ROLE_APPROVER,
};
use crate::repositories::pending_owner as candidates;

use super::dispatcher::{EventContext, NotificationPayload};
use super::NotificationService;

/// `is_admin()` 認的兩個角色代碼（`middleware/auth.rs`）。
fn admin_roles() -> Vec<String> {
    vec![ROLE_SYSTEM_ADMIN.to_string(), ROLE_ADMIN_LEGACY.to_string()]
}

/// 會產生「等某人動作」關卡的業務實體。
///
/// 變體對應 `notifications.related_entity_type`，**新增變體要同步三處**：
/// [`StageEntity::entity_type`]、`services/notification/reconcile.rs` 的
/// `KNOWN_ENTITY_TYPES` 與其 UNION 分支、`frontend/src/lib/notificationRoute.ts` 的 case。
/// 漏掉第二處的後果是卡死的待辦沒有安全網；漏掉第三處是點了待辦沒有落點。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageEntity {
    /// ERP 單據：倉管核准 / 大額終審 / 沖銷核准三關。
    Document(Uuid),
    /// 加班申請：行政人員關 / 管理員關。
    Overtime(Uuid),
    /// 設備報廢申請：待核准。
    EquipmentDisposal(Uuid),
    /// 設備閒置 / 復用申請：待核准。
    EquipmentIdle(Uuid),
    /// 設備維修 / 保養紀錄：待驗收。
    MaintenanceRecord(Uuid),
    /// 安樂死單：待 PI 決定（**有 24 小時自動核准時鐘**）。
    EuthanasiaOrder(Uuid),
    /// 變更申請：待分類 / 已分類待送審。
    Amendment(Uuid),
    /// AUP 計畫：獸醫審查 / 委員審查（皆為「被指派的那些人」）。
    Protocol(Uuid),
}

impl StageEntity {
    /// 寫進 `notifications.related_entity_type` 的值。
    pub fn entity_type(self) -> &'static str {
        match self {
            Self::Document(_) => "document",
            Self::Overtime(_) => "overtime_record",
            Self::EquipmentDisposal(_) => "equipment_disposal",
            Self::EquipmentIdle(_) => "equipment_idle_request",
            Self::MaintenanceRecord(_) => "maintenance_record",
            Self::EuthanasiaOrder(_) => "euthanasia_order",
            Self::Amendment(_) => "amendment",
            Self::Protocol(_) => "protocol",
        }
    }

    pub fn id(self) -> Uuid {
        match self {
            Self::Document(id)
            | Self::Overtime(id)
            | Self::EquipmentDisposal(id)
            | Self::EquipmentIdle(id)
            | Self::MaintenanceRecord(id)
            | Self::EuthanasiaOrder(id)
            | Self::Amendment(id)
            | Self::Protocol(id) => id,
        }
    }

    /// 對應的 `notification_routing` 事件碼，**只用來決定 email 管道**，不決定收件人。
    ///
    /// `None` ＝ 該關卡沒有對應的路由事件（例：安樂死是固定通知），只發站內。
    fn channel_event(self) -> Option<&'static str> {
        match self {
            Self::Document(_) => Some("document_submitted"),
            Self::Overtime(_) => Some("overtime_submitted"),
            Self::EquipmentDisposal(_) => Some("equipment_disposal"),
            Self::EquipmentIdle(_) => Some("equipment_idle_request"),
            Self::MaintenanceRecord(_) => Some("equipment_maintenance_review"),
            Self::Amendment(_) => Some("amendment_submitted"),
            Self::EuthanasiaOrder(_) | Self::Protocol(_) => None,
        }
    }
}

/// 本次同步**新建**的待辦中、依 routing 該收 email 的那批，供 commit 後寄出。
///
/// 由 [`NotificationService::sync_stage_todos_tx`] 回傳、
/// [`NotificationService::send_stage_emails`] 消費。email 不進 tx：rollback 收不回已寄出的信。
pub struct StageEmailBatch {
    event_type: &'static str,
    payload: NotificationPayload,
    recipients: Vec<Uuid>,
}

/// 一個關卡的待辦內容與收件人。
pub struct StageTodo {
    /// 這一關的合法處理人（**已扣除 SoD**）。空集合是合法結果，代表沒有人能處理。
    pub recipients: Vec<Uuid>,
    pub title: String,
    pub content: Option<String>,
    pub notification_type: NotificationType,
}

/// 一筆實體「現在卡在誰手上」。`None` ＝ 不在等任何人（草稿、終態、已刪）。
///
/// 讀取一律走 `pool` 而非呼叫端的 tx：這些是唯讀查詢，不需要與業務 tx 同一連線，
/// 而放進 tx 會讓候選名單查詢也被業務 tx 的列鎖拖住。
/// ⚠️ **例外**：實體本身的狀態必須讀得到呼叫端 tx 內的最新值，否則同步的是舊狀態——
/// 故呼叫端必須在**狀態已寫入該 tx 之後**才呼叫 [`NotificationService::sync_stage_todos_tx`]，
/// 且該函式改用 tx 讀實體狀態（見其實作）。
async fn resolve_stage(
    pool: &PgPool,
    tx: &mut Transaction<'_, Postgres>,
    entity: StageEntity,
) -> Result<Option<StageTodo>, AppError> {
    match entity {
        StageEntity::Document(id) => document_stage(pool, id).await,
        StageEntity::Overtime(id) => overtime_stage(pool, tx, id).await,
        StageEntity::EquipmentDisposal(id) => disposal_stage(pool, tx, id).await,
        StageEntity::EquipmentIdle(id) => idle_stage(pool, tx, id).await,
        StageEntity::MaintenanceRecord(id) => maintenance_stage(pool, tx, id).await,
        StageEntity::EuthanasiaOrder(id) => euthanasia_stage(tx, id).await,
        StageEntity::Amendment(id) => amendment_stage(pool, tx, id).await,
        StageEntity::Protocol(id) => protocol_stage(tx, id).await,
    }
}

// ── 各關卡的判準 ─────────────────────────────────────────────────────────

/// ERP 單據三關。判準完全委派給 `services/pending_owner.rs`（列表頁「卡在誰」的同一份）。
async fn document_stage(pool: &PgPool, id: Uuid) -> Result<Option<StageTodo>, AppError> {
    let Some((stage_key, recipients)) =
        crate::services::pending_owner::stage_recipients_for_document(pool, id).await?
    else {
        return Ok(None);
    };

    let doc_no: Option<String> = sqlx::query_scalar("SELECT doc_no FROM documents WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    let doc_no = doc_no.unwrap_or_else(|| "-".to_string());

    let stage_text = match stage_key {
        "doc_wm_approve" => "倉管核准",
        "doc_final_approve" => "負責人終審",
        _ => "沖銷核准",
    };

    Ok(Some(StageTodo {
        recipients,
        title: format!("[iPig] 單據待{stage_text} - {doc_no}"),
        content: Some(format!("單據 {doc_no} 已送審，待您{stage_text}。")),
        notification_type: NotificationType::DocumentApproval,
    }))
}

/// 加班兩關。判準抄自 `services/hr/overtime.rs:247-251` 的 `can_approve`：
/// `pending_admin_staff` → `is_admin || ADMIN_STAFF`、`pending_admin` → 僅 `is_admin`，
/// 其餘狀態一律 false（**legacy 的 `pending` 沒有任何人能審**，故不產生待辦）。
/// SoD：申請人不得審自己的申請（同檔 `:241`）。
async fn overtime_stage(
    pool: &PgPool,
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<StageTodo>, AppError> {
    let row: Option<(String, Uuid, Option<chrono::NaiveDate>)> =
        sqlx::query_as("SELECT status, user_id, overtime_date FROM overtime_records WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut **tx)
            .await?;
    let Some((status, applicant, date)) = row else {
        return Ok(None);
    };

    let roles: Vec<String> = match status.as_str() {
        "pending_admin_staff" => vec![
            ROLE_SYSTEM_ADMIN.to_string(),
            ROLE_ADMIN_LEGACY.to_string(),
            ROLE_ADMIN_STAFF.to_string(),
        ],
        "pending_admin" => admin_roles(),
        _ => return Ok(None),
    };

    let recipients = candidates::list_users_with_any_role(pool, &roles)
        .await?
        .into_iter()
        .map(|(uid, _)| uid)
        .filter(|uid| *uid != applicant)
        .collect();

    let date_text = date
        .map(|d| d.to_string())
        .unwrap_or_else(|| "-".to_string());
    Ok(Some(StageTodo {
        recipients,
        title: format!("[iPig] 加班申請待審核 - {date_text}"),
        content: Some(format!("{date_text} 的加班申請待您審核。")),
        notification_type: NotificationType::OvertimeApproval,
    }))
}

/// 設備報廢待核准。判準：`equipment.disposal.approve`（`services/equipment/disposal.rs:325`）、
/// 狀態必須是 `pending`（`:334`）、申請人不得自核（`:340`）。
async fn disposal_stage(
    pool: &PgPool,
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<StageTodo>, AppError> {
    let row: Option<(String, Uuid, Uuid)> = sqlx::query_as(
        "SELECT status::text, applied_by, equipment_id FROM equipment_disposals WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?;
    applied_stage(pool, row, "equipment.disposal.approve", "報廢").await
}

/// 設備閒置 / 復用待核准。判準：`equipment.idle.approve`（`services/equipment/idle.rs:162`）、
/// 狀態必須是 `pending`（`:191`）、申請人不得自核（`:196`）。
async fn idle_stage(
    pool: &PgPool,
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<StageTodo>, AppError> {
    let row: Option<(String, Uuid, Uuid)> = sqlx::query_as(
        "SELECT status::text, applied_by, equipment_id FROM equipment_idle_requests WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?;
    applied_stage(pool, row, "equipment.idle.approve", "閒置/復用").await
}

/// 報廢與閒置的形狀完全相同（`status` / `applied_by` / `equipment_id`），只差權限碼與字樣，
/// 故查詢在各自的函式內（**SQL 必須是字面值**——CI 有守衛擋 `format!` 拼 SQL，
/// 見 `.github/workflows/ci.yml`），共用的只有查完之後的判斷。
async fn applied_stage(
    pool: &PgPool,
    row: Option<(String, Uuid, Uuid)>,
    permission: &'static str,
    label: &str,
) -> Result<Option<StageTodo>, AppError> {
    let Some((status, applied_by, equipment_id)) = row else {
        return Ok(None);
    };
    if status != "pending" {
        return Ok(None);
    }

    let recipients = candidates::list_users_with_permission(pool, permission)
        .await?
        .into_iter()
        .map(|(uid, _)| uid)
        .filter(|uid| *uid != applied_by)
        .collect();

    let name = equipment_name(pool, equipment_id).await?;
    Ok(Some(StageTodo {
        recipients,
        title: format!("[iPig] 設備{label}申請待核准 - {name}"),
        content: Some(format!("設備「{name}」的{label}申請待您核准。")),
        notification_type: NotificationType::SystemAlert,
    }))
}

/// 維修 / 保養待驗收。判準：`equipment.maintenance.review` **或** `equipment.manage`
/// （`services/equipment/maintenance.rs` 的 `review_maintenance_record`）、
/// 狀態必須是 `pending_review`、登錄者不得自驗（同檔 `assert_not_self_approval`）。
async fn maintenance_stage(
    pool: &PgPool,
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<StageTodo>, AppError> {
    let row: Option<(String, Uuid, Uuid, String)> = sqlx::query_as(
        "SELECT status::text, created_by, equipment_id, maintenance_type::text \
         FROM equipment_maintenance_records WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?;
    let Some((status, created_by, equipment_id, maintenance_type)) = row else {
        return Ok(None);
    };
    if status != "pending_review" {
        return Ok(None);
    }

    // 兩個權限任一即可 → 取聯集後去重。
    let mut seen: HashSet<Uuid> = HashSet::new();
    let mut recipients = Vec::new();
    for permission in ["equipment.maintenance.review", "equipment.manage"] {
        for (uid, _) in candidates::list_users_with_permission(pool, permission).await? {
            if uid != created_by && seen.insert(uid) {
                recipients.push(uid);
            }
        }
    }

    let name = equipment_name(pool, equipment_id).await?;
    let type_text = if maintenance_type == "repair" {
        "維修"
    } else {
        "保養"
    };
    Ok(Some(StageTodo {
        recipients,
        title: format!("[iPig] 設備{type_text}待驗收 - {name}"),
        content: Some(format!("「{name}」的{type_text}紀錄已標記完修，待您驗收。")),
        notification_type: NotificationType::SystemAlert,
    }))
}

/// 安樂死待 PI 決定。收件人是該單的 `pi_user_id`（單一特定人，不經路由）。
///
/// ⚠️ **這一關有 24 小時自動核准時鐘**（`scheduler.rs` 的 `euthanasia_timeout` job）：
/// PI 漏看等同預設同意，是 IACUC 合規路徑而非一般提醒——這正是它必須進待處理清單、
/// 不能只留在鈴鐺的理由。`pending_pi` 以外的狀態（appealed / chair_arbitration /
/// approved / rejected / executed / cancelled）都已離開「等 PI」，一律不產生待辦。
async fn euthanasia_stage(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<StageTodo>, AppError> {
    let row: Option<(String, Uuid, String)> = sqlx::query_as(
        "SELECT e.status::text, e.pi_user_id, COALESCE(a.ear_tag, '-') \
         FROM euthanasia_orders e LEFT JOIN animals a ON a.id = e.animal_id WHERE e.id = $1",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?;
    let Some((status, pi_user_id, ear_tag)) = row else {
        return Ok(None);
    };
    if status != "pending_pi" {
        return Ok(None);
    }

    Ok(Some(StageTodo {
        recipients: vec![pi_user_id],
        title: format!("[iPig] 安樂死單待您決定 - 耳號 {ear_tag}"),
        content: Some(format!(
            "動物 #{ear_tag} 的安樂死單待您選擇「同意執行」或「申請暫緩」。\n\n\
             逾 24 小時未回應，系統將自動解鎖執行權限。"
        )),
        notification_type: NotificationType::SystemAlert,
    }))
}

/// 變更申請兩關：
/// - `SUBMITTED` / `RESUBMITTED` → 待執秘分類（`aup.amendment.classify`，`handlers/amendment.rs:235`）
/// - `CLASSIFIED` → 待送審（`aup.protocol.change_status`）
///
/// `UNDER_REVIEW` 由委員審查，收件人是被指派的委員——那一關綁在
/// `amendment_review_assignments`，形狀與計畫委員審查相同，**本輪不收**（見 R112 說明）。
async fn amendment_stage(
    pool: &PgPool,
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<StageTodo>, AppError> {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT status::text, amendment_no FROM amendments WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut **tx)
            .await?;
    let Some((status, amendment_no)) = row else {
        return Ok(None);
    };

    let (permission, stage_text) = match status.as_str() {
        "SUBMITTED" | "RESUBMITTED" => ("aup.amendment.classify", "分類"),
        "CLASSIFIED" => ("aup.protocol.change_status", "送審"),
        _ => return Ok(None),
    };

    let recipients = candidates::list_users_with_permission(pool, permission)
        .await?
        .into_iter()
        .map(|(uid, _)| uid)
        .collect();

    Ok(Some(StageTodo {
        recipients,
        title: format!("[iPig] 變更申請待{stage_text} - {amendment_no}"),
        content: Some(format!("變更申請 {amendment_no} 待您{stage_text}。")),
        notification_type: NotificationType::ProtocolStatus,
    }))
}

/// AUP 計畫的兩個「等被指派的人」關卡：
/// - `VET_REVIEW` → `vet_review_assignments` 中 `completed_at IS NULL` 的獸醫
/// - `UNDER_REVIEW` → `review_assignments` 中 `completed_at IS NULL` 的委員
///
/// ⚠️ 這兩關**不能用角色算**：routing 現行規則 `protocol_vet_review → role VET` 會發給
/// 全部獸醫，而真正能審的只有被指派的那一位（授權判準見 `services/access.rs` 對
/// `vet_review_assignments` / `review_assignments` 的比對）。發給沒被指派的人＝
/// 給一則他點下去進不了的待辦。
///
/// 行政受理關（`SUBMITTED` / `PRE_REVIEW` / `RESUBMITTED`）不在此：那關的既有 routing
/// （→ `IACUC_STAFF`）與授權碼 `aup.protocol.change_status` 實務上同一群人，落差小，
/// 本輪不動（R112 說明有記）。
async fn protocol_stage(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<StageTodo>, AppError> {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT status::text, iacuc_no FROM protocols WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut **tx)
            .await?;
    let Some((status, iacuc_no)) = row else {
        return Ok(None);
    };

    let (sql, stage_text) = match status.as_str() {
        "VET_REVIEW" => (
            "SELECT vra.vet_id FROM vet_review_assignments vra \
             JOIN users u ON u.id = vra.vet_id \
             WHERE vra.protocol_id = $1 AND vra.completed_at IS NULL \
               AND u.is_active = true AND u.deleted_at IS NULL",
            "獸醫審查",
        ),
        // ⚠️ 委員這一關同時看 `completed_at` 與 `decision`：2026-08-26 實查全 codebase
        // **沒有任何地方 UPDATE `review_assignments.completed_at`**（只有
        // `vet_review_assignments` 有，`my_protocols.rs::save_vet_review_form`），
        // 只靠 completed_at 判斷的話委員的待辦要等整個計畫離開 UNDER_REVIEW 才會消失。
        // 加上 `decision IS NULL` 讓已表態的委員先行消失；兩者都沒被寫入時，
        // 仍有「計畫離開審查狀態」這層（本函式開頭的 status 判斷）兜底。
        "UNDER_REVIEW" => (
            "SELECT ra.reviewer_id FROM review_assignments ra \
             JOIN users u ON u.id = ra.reviewer_id \
             WHERE ra.protocol_id = $1 AND ra.completed_at IS NULL AND ra.decision IS NULL \
               AND u.is_active = true AND u.deleted_at IS NULL",
            "審查",
        ),
        _ => return Ok(None),
    };

    let recipients: Vec<Uuid> = sqlx::query_scalar(sql)
        .bind(id)
        .fetch_all(&mut **tx)
        .await?;

    Ok(Some(StageTodo {
        recipients,
        title: format!("[iPig] 計畫待您{stage_text} - {iacuc_no}"),
        content: Some(format!("計畫 {iacuc_no} 已指派給您{stage_text}。")),
        notification_type: NotificationType::ProtocolStatus,
    }))
}

async fn equipment_name(pool: &PgPool, equipment_id: Uuid) -> Result<String, AppError> {
    let name: Option<String> = sqlx::query_scalar("SELECT name FROM equipment WHERE id = $1")
        .bind(equipment_id)
        .fetch_optional(pool)
        .await?;
    Ok(name.unwrap_or_else(|| "設備".to_string()))
}

// ── 同步 ────────────────────────────────────────────────────────────────

impl NotificationService {
    /// 把某筆實體的待處理清單**同步**到它目前關卡該有的樣子。
    ///
    /// 呼叫時機：任何可能改變「誰在等這筆」的狀態寫入之後、**同一個 tx 內、commit 之前**。
    /// 送出 / 核准 / 駁回 / 撤回 / 作廢 / 轉關 / 刪除都是同一句呼叫，不必分辨。
    ///
    /// 冪等：重複呼叫不會產生第二則待辦，也不會把已解除的重新點亮。
    ///
    /// 回傳「本次**新建**的待辦中、依 routing 該收 email 的收件人」，供呼叫端在
    /// **commit 之後**呼叫 [`Self::send_routed_emails`]。只對新建的寄——否則每次同步
    /// 都會對同一批人重寄一次。
    pub async fn sync_stage_todos_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        entity: StageEntity,
        actor_id: Option<Uuid>,
    ) -> Result<Option<StageEmailBatch>, AppError> {
        let entity_type = entity.entity_type();
        let entity_id = entity.id();
        let todo = resolve_stage(&self.db, tx, entity).await?;

        let desired: Vec<Uuid> = todo
            .as_ref()
            .map(|t| {
                t.recipients
                    .iter()
                    .copied()
                    // 觸發這次轉換的人不必收到自己造成的待辦。與 dispatcher 的
                    // 「不通知觸發者本人」同一條規則，但這裡是每次同步都要套用，
                    // 否則自己送出的單會在自己的清單裡。
                    .filter(|uid| Some(*uid) != actor_id)
                    .collect()
            })
            .unwrap_or_default();

        // ① 解除「還置頂、但已不在收件人名單」的——涵蓋轉關、終態、撤回、SoD 變動。
        //    只降 priority 不動 is_read/read_at（理由見 reconcile.rs：會踩到 GC 條件）。
        //
        // ⚠️ `recipient_role` 這個條件不可省：同一個 `document` id 上還掛著採購單
        //    未入庫提醒（`recipient_role` 為 NULL），少了它會在單據轉關時把倉管的
        //    未入庫待辦一起清掉——而那筆的完成條件是入庫，與核准關卡無關。
        sqlx::query(
            r#"
            UPDATE notifications
            SET priority = $4
            WHERE related_entity_type = $1
              AND related_entity_id = $2
              AND priority > $4
              AND recipient_role = $5
              AND NOT (user_id = ANY($3))
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .bind(&desired)
        .bind(PRIORITY_NORMAL)
        .bind(RECIPIENT_ROLE_APPROVER)
        .execute(&mut **tx)
        .await?;

        let Some(todo) = todo else {
            return Ok(None);
        };
        if desired.is_empty() {
            return Ok(None);
        }

        // ② 補上缺的。已有未完成待辦者跳過——這就是冪等的來源。
        let existing: Vec<Uuid> = sqlx::query_scalar(
            r#"
            SELECT user_id FROM notifications
            WHERE related_entity_type = $1 AND related_entity_id = $2 AND priority > $3
              AND recipient_role = $4
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .bind(PRIORITY_NORMAL)
        .bind(RECIPIENT_ROLE_APPROVER)
        .fetch_all(&mut **tx)
        .await?;
        let existing: HashSet<Uuid> = existing.into_iter().collect();

        let mut created = Vec::new();
        for uid in desired {
            if existing.contains(&uid) {
                continue;
            }
            Self::create_notification_tx_with_priority(
                tx,
                CreateNotificationRequest {
                    user_id: uid,
                    notification_type: todo.notification_type.clone(),
                    title: todo.title.clone(),
                    content: todo.content.clone(),
                    related_entity_type: Some(entity_type.to_string()),
                    related_entity_id: Some(entity_id),
                },
                PRIORITY_PINNED,
                // 一律標成「本關的處理人」。**這不是裝飾**：`document` 這個
                // entity_type 同時掛著採購單未入庫提醒（`erp.rs::notify_po_pending_receipt`，
                // `recipient_role` 為 NULL）與本模組的三關核准待辦，對帳只看
                // (type, id) 分不出兩者——未入庫提醒綁的是**已核准**的 PO，
                // 若用「status 不是 submitted 就降級」的規則會把它們全部誤清。
                // 請假當初加這個欄位（R92-1）解的是同一類問題。
                Some(RECIPIENT_ROLE_APPROVER),
            )
            .await?;
            created.push(uid);
        }

        if created.is_empty() {
            return Ok(None);
        }

        // **routing 在這裡只決定管道、不決定收件人**——R112-1 裁定的分工：
        // 「誰該處理」抄授權碼（上面各 `*_stage`），「要不要寄信」留給 admin 在路由頁調。
        let Some(event_type) = entity.channel_event() else {
            return Ok(None);
        };
        let ctx = EventContext {
            actor_id,
            ..Default::default()
        };
        // 查不到 / 查詢失敗 → 只發站內。email 是加值不是必要條件，不值得為它讓業務 tx 失敗。
        let email_users = match self.email_channel_users(event_type, &ctx).await {
            Ok(users) => users,
            Err(e) => {
                tracing::warn!(event_type, "查詢 email 管道設定失敗，本次只發站內: {e}");
                return Ok(None);
            }
        };
        let recipients: Vec<Uuid> = created
            .into_iter()
            .filter(|uid| email_users.contains(uid))
            .collect();
        if recipients.is_empty() {
            return Ok(None);
        }

        Ok(Some(StageEmailBatch {
            event_type,
            payload: NotificationPayload {
                notification_type: todo.notification_type,
                title: todo.title,
                content: todo.content,
                related_entity_type: Some(entity_type.to_string()),
                related_entity_id: Some(entity_id),
            },
            recipients,
        }))
    }

    /// [`Self::sync_stage_todos_tx`] 的 pool 版：**只給沒有 tx 的既有呼叫端用**。
    ///
    /// 自己開一個短 tx 做同步。與 tx 版的差別是它不與業務狀態寫入同一個 tx，因此留有
    /// 「狀態已 commit → 併發的下一個轉換也同步完 → 本次同步才跑」這條時序，
    /// 可能補出一則已經不該存在的待辦。冪等設計讓它不會累積（下一次同步就清掉），
    /// 每日對帳（`reconcile.rs`）也接得住，最長 24 小時。
    ///
    /// ⚠️ **新程式碼一律用 tx 版。** 這支存在的唯一理由是不把 AUP 既有的非 tx 流程
    /// （`services/amendment/workflow.rs` 的 `submit` / `start_review`、
    /// `services/protocol/` 的審查指派）為了掛通知而改寫成 tx——那是獨立的一件事，
    /// 風險不該混進通知這個 PR 裡（TODO.md R112-7 追蹤）。
    pub async fn sync_stage_todos(
        &self,
        entity: StageEntity,
        actor_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let mut tx = self.db.begin().await?;
        let emails = self.sync_stage_todos_tx(&mut tx, entity, actor_id).await?;
        tx.commit().await?;
        self.send_stage_emails(emails).await;
        Ok(())
    }

    /// [`Self::sync_stage_todos_tx`] 的 commit 後配套：把該批 email 寄出去。
    /// 傳 `None`（沒有人要收信）是常態，直接略過。
    pub async fn send_stage_emails(&self, batch: Option<StageEmailBatch>) {
        let Some(batch) = batch else { return };
        self.send_routed_emails(batch.event_type, &batch.payload, &batch.recipients)
            .await;
    }
}
