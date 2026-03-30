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
mod my_protocols;
mod numbering;
mod review;
mod status;
pub mod validation;

pub struct ProtocolService;
