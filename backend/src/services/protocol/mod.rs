// AUP 計畫管理服務模組
//
// 將原 protocol.rs 拆分為以下子模組：
// - core: 計畫 CRUD、列表
// - comment: 審查評論
// - history: 活動紀錄與狀態歷程
// - my_protocols: 使用者相關計畫查詢
// - numbering: 計畫與 IACUC 編號生成
// - review: 審查流程
// - status: 計畫狀態管理

pub mod ai_review;
mod comment;
mod core;
mod history;
mod import_review;
mod my_protocols;
mod notice;
mod numbering;
mod pi_provision;
mod review;
mod status;
pub mod validation;

use crate::models::{ProtocolQuery, ProtocolStatus};

pub struct ProtocolService;

/// 「可指派計畫」SQL 過濾片段（動物 IACUC No. 下拉用）：已核准（APPROVED /
/// APPROVED_WITH_CONDITIONS）且具 iacuc_no。`list` 與 `get_my_protocols` 共用，
/// 兩者皆以別名 `p` 指 protocols（DRY，狀態字串集中一處）。
pub(super) const ASSIGNABLE_PROTOCOL_FILTER: &str =
    " AND p.status IN ('APPROVED', 'APPROVED_WITH_CONDITIONS') AND p.iacuc_no IS NOT NULL";

/// 依 `query` 附加「可選過濾」SQL 片段到 `sql`（狀態 / 關鍵字 / PI / 起訖日期），
/// placeholder 由 `$start_param` 起遞增，回傳下一個未使用的參數編號。
/// `list`（view_all 路徑，$1-$3 固定，故 `start_param = 4`）與 `get_my_protocols`
/// （成員路徑，僅 $1=user_id，故 `start_param = 2`）共用同一組裝器（DRY），確保兩路徑
/// 過濾語義一致、佔位符不再各寫一份而漂移。別名 `p` 指 protocols。
///
/// ⚠️ 綁定端（呼叫者的 `.bind(...)`）順序**必須**與此處佔位符產生順序完全一致：
/// status → keyword → pi_user_id → start_date → end_date。新增過濾欄位時兩端同步改。
pub(super) fn push_optional_protocol_filters(
    sql: &mut String,
    query: &ProtocolQuery,
    start_param: u32,
) -> u32 {
    let mut p = start_param;
    if let Some(status) = query.status {
        if status != ProtocolStatus::Deleted {
            sql.push_str(&format!(" AND p.status = ${p}"));
            p += 1;
        }
    }
    if query.keyword.is_some() {
        sql.push_str(&format!(
            " AND (p.title ILIKE ${p} OR p.protocol_no ILIKE ${p} OR p.iacuc_no ILIKE ${p})"
        ));
        p += 1;
    }
    if query.pi_user_id.is_some() {
        sql.push_str(&format!(" AND p.pi_user_id = ${p}"));
        p += 1;
    }
    if query.start_date.is_some() {
        sql.push_str(&format!(" AND p.start_date >= ${p}"));
        p += 1;
    }
    if query.end_date.is_some() {
        sql.push_str(&format!(" AND p.end_date <= ${p}"));
        p += 1;
    }
    p
}
