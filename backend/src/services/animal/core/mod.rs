mod delete;
mod pen_validation;
mod query;
mod update;
mod write;

/// IACUC No. 變更資訊（供 handler 記錄審計日誌用）
pub struct IacucChangeInfo {
    pub old_iacuc_no: Option<String>,
    pub new_iacuc_no: String,
}
