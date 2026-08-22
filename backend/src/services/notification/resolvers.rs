//! 關係型收件人解析器註冊表。
//!
//! 「零寫死」的關鍵：收件人**來源**在 `notification_routing.target_value` 以 key 宣告，
//! 解析**邏輯**集中在此（關係查詢無法純資料庫表達，故落在程式，但仍是宣告式選用）。
//!
//! 新增 resolver 步驟：
//! 1. 在此 `match` 增一個 key → 對應收件人查詢（盡量複用 `helpers.rs` 既有方法）。
//! 2. 在 `EventContext` 補上該 resolver 需要的實體欄位（若尚無）。
//! 3. 在 seed / 路由設定以該 key 建立 `target_kind='resolver'` 的列。
//!
//! P0 先註冊計畫關係兩個 resolver；其餘（申請人 / 經手人 / 安樂死當事人等）於後續 Phase
//! 隨對應事件遷移時補上。

use uuid::Uuid;

use crate::error::AppError;

use super::dispatcher::EventContext;
use super::NotificationService;

/// 依 resolver key 解析關係型收件人，回傳 user_id 清單。
/// 未知 / 上下文不足 → 回空集合（記 warn，不阻斷派送）。
pub(crate) async fn resolve(
    svc: &NotificationService,
    key: &str,
    ctx: &EventContext,
) -> Result<Vec<Uuid>, AppError> {
    match key {
        // 該計畫的 PI 成員 + 計劃負責人（SD）。
        "protocol_pi_sd" => match ctx.protocol_id {
            Some(pid) => Ok(svc
                .get_protocol_pi_and_sd(pid)
                .await?
                .into_iter()
                .map(|(id, _, _)| id)
                .collect()),
            None => {
                tracing::warn!("[resolver] protocol_pi_sd 缺 protocol_id，回空");
                Ok(vec![])
            }
        },
        // 該計畫的 PI（user_protocols.role_in_protocol='PI'）。
        "protocol_pi" => match ctx.protocol_id {
            Some(pid) => {
                let rows: Vec<(Uuid,)> = sqlx::query_as(
                    "SELECT DISTINCT u.id FROM users u \
                     JOIN user_protocols up ON u.id = up.user_id \
                     WHERE up.protocol_id = $1 AND up.role_in_protocol = 'PI' \
                       AND u.is_active = true AND u.deleted_at IS NULL",
                )
                .bind(pid)
                .fetch_all(&svc.db)
                .await?;
                Ok(rows.into_iter().map(|(id,)| id).collect())
            }
            None => {
                tracing::warn!("[resolver] protocol_pi 缺 protocol_id，回空");
                Ok(vec![])
            }
        },
        // 被指派審查該計畫的審查委員。
        "assigned_reviewers" => match ctx.protocol_id {
            Some(pid) => Ok(svc
                .get_assigned_reviewers(pid)
                .await?
                .into_iter()
                .map(|(id, _, _)| id)
                .collect()),
            None => {
                tracing::warn!("[resolver] assigned_reviewers 缺 protocol_id，回空");
                Ok(vec![])
            }
        },
        // 直接指定的「當事人」本人（如假單 / 加班申請人）；驗證 active 且未軟刪除後回傳
        // （對齊原 notify_leave_approved 的 GDPR 檢查：停用 / 已刪除帳號不通知）。
        "event_subject" => match ctx.subject_user_id {
            Some(uid) => {
                let rows: Vec<(Uuid,)> = sqlx::query_as(
                    "SELECT id FROM users WHERE id = $1 AND is_active = true AND deleted_at IS NULL",
                )
                .bind(uid)
                .fetch_all(&svc.db)
                .await?;
                Ok(rows.into_iter().map(|(id,)| id).collect())
            }
            None => {
                tracing::warn!("[resolver] event_subject 缺 subject_user_id，回空");
                Ok(vec![])
            }
        },
        // 這則假單「現在這一關」該通知誰（單位主管 / 負責人 / 卡關代批的管理員，
        // 依 leave_requests.status 動態決定）。取代原本 leave_submitted 事件的角色廣播
        // （admin + DIRECTOR，不分審核關卡——會讓還輪不到的角色也收到「待處理」，
        // 點下去卻 403）。權威邏輯在 HrService::current_stage_approvers，與
        // can_user_approve_leave 的授權判準共用同一組查詢，保證兩邊一致。
        "leave_current_stage_approvers" => match ctx.entity_id {
            Some(lid) => crate::services::HrService::current_stage_approvers(&svc.db, lid).await,
            None => {
                tracing::warn!("[resolver] leave_current_stage_approvers 缺 entity_id，回空");
                Ok(vec![])
            }
        },
        // 曾核准此假單的經手人（leave_approvals action='APPROVE'）。
        "leave_request_approvers" => match ctx.entity_id {
            Some(lid) => {
                // 與 event_subject 一致：僅回 active 且未軟刪除的經手人（離職/停用不再收 HR 通知）。
                let rows: Vec<(Uuid,)> = sqlx::query_as(
                    "SELECT DISTINCT u.id FROM leave_approvals la \
                     JOIN users u ON u.id = la.approver_id \
                     WHERE la.leave_request_id = $1 AND la.action = 'APPROVE' \
                       AND u.is_active = true AND u.deleted_at IS NULL",
                )
                .bind(lid)
                .fetch_all(&svc.db)
                .await?;
                Ok(rows.into_iter().map(|(id,)| id).collect())
            }
            None => {
                tracing::warn!("[resolver] leave_request_approvers 缺 entity_id，回空");
                Ok(vec![])
            }
        },
        other => {
            tracing::warn!(
                resolver = other,
                "[resolver] 未知或尚未實作的收件人解析器，回空"
            );
            Ok(vec![])
        }
    }
}

/// resolver key → (人類可讀標籤, 說明, 是否「固定收件人、不可調整」)。
/// 供 admin 路由頁顯示 resolver 型規則與標註不可調。
/// 關係型收件人由事件實體動態決定，故 `fixed=true`（admin 無法重導對象）。
pub(crate) fn resolver_meta(key: &str) -> (&'static str, &'static str, bool) {
    match key {
        "protocol_pi_sd" => ("計畫 PI / 計劃負責人(SD)", "依事件所屬計畫動態決定", true),
        "protocol_pi" => ("計畫 PI", "依事件所屬計畫動態決定", true),
        "assigned_reviewers" => ("被指派審查委員", "依計畫審查指派動態決定", true),
        "event_subject" => ("當事人本人", "如申請人本人，由事件決定", true),
        "leave_current_stage_approvers" => (
            "本關合法審核人",
            "依假單目前狀態動態決定（單位主管／負責人／卡關代批管理員）",
            true,
        ),
        "leave_request_approvers" => ("核准經手人", "曾核准此假單者，由紀錄決定", true),
        _ => ("（未知解析器）", "未註冊的解析器 key", true),
    }
}
