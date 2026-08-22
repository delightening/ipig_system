//! 共用驗證函式。
//!
//! # 慣例：validate-after-trim
//!
//! 本模組所有 string 驗證函式於規則計算前統一 `.trim()`，呼叫端**不需**事前 trim：
//!
//! - `validate_reason_text` / `validate_pen_location` — 內部 `.trim()` 後判定空與長度
//! - `validate_filename` — **不 trim**（檔名前後空白本身可疑，視為非法）
//! - `validate_ear_tag` — 純數字格式化檢查，空白會自然 fail（`parse::<u32>()` 拒絕含空白）
//!
//! DB 寫入由呼叫端決定存原始或 trimmed 字串，建議存 trimmed 避免 audit 受 leading/trailing
//! 空白污染（`validate_reason_text` 文件已說明）。

use crate::error::AppError;

/// RFC 5321 §4.5.3.1.3 local-part + "@" + domain 上限
const MAX_EMAIL_LENGTH: usize = 254;
/// NTFS / ext4 / Linux NAME_MAX 等主流檔案系統的單一檔名長度上限
const MAX_FILENAME_LENGTH: usize = 255;

/// R30-9b: 自由文字「理由」欄位上限（HMAC chain v3 binding 一行 row payload，避免暴漲）。
pub const REASON_TEXT_MAX_CHARS: usize = 1000;

/// 驗證自由文字「理由」欄位（撤銷簽章 reason / 變更申請 reason 等）。
///
/// 規則：
/// 1. trim 後不可為空（防只送空白繞過 required 檢查）
/// 2. trim 後字元數 ≤ [`REASON_TEXT_MAX_CHARS`]（用 chars().count() 而非 byte len，
///    支援中文：1000 中文字 = 3000 bytes，bytes 限制會誤拒）
///
/// 設計：以 trim 後規則計算，handler 與 service 規則一致；DB 層另存原始或 trimmed
/// 由呼叫端決定（建議存 trimmed，避免 leading/trailing 空白污染 audit）。
///
/// 給 `#[derive(Validate)]` 用：`#[validate(custom(function = "validate_reason_text"))]`
pub fn validate_reason_text(reason: &str) -> Result<(), validator::ValidationError> {
    let trimmed = reason.trim();
    if trimmed.is_empty() {
        let mut e = validator::ValidationError::new("reason_required");
        e.message = Some(std::borrow::Cow::Borrowed("理由不可為空"));
        return Err(e);
    }
    if trimmed.chars().count() > REASON_TEXT_MAX_CHARS {
        let mut e = validator::ValidationError::new("reason_too_long");
        e.message = Some(std::borrow::Cow::Owned(format!(
            "理由不可超過 {REASON_TEXT_MAX_CHARS} 字"
        )));
        return Err(e);
    }
    Ok(())
}

/// 驗證附件路徑/URL：僅允許 same-origin 相對路徑（/...）或明確的 http(s) URL。
///
/// 防 `javascript:` / `data:` 等危險 scheme 注入（前端會把此值放入 `<a href>`，
/// React 不會自動過濾 href，因此後端在系統邊界先擋一層 defense-in-depth）。
///
/// 給 `#[derive(Validate)]` 用：`#[validate(custom(function = "validate_attachment_path"))]`
pub fn validate_attachment_path(path: &str) -> Result<(), validator::ValidationError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let is_relative = trimmed.starts_with('/') && !trimmed.starts_with("//");
    let is_http = {
        let lower = trimmed.to_ascii_lowercase();
        lower.starts_with("http://") || lower.starts_with("https://")
    };
    if !is_relative && !is_http {
        let mut e = validator::ValidationError::new("attachment_path_scheme");
        e.message = Some(std::borrow::Cow::Borrowed(
            "附件路徑僅允許相對路徑（/...）或 http(s) URL",
        ));
        return Err(e);
    }
    Ok(())
}

/// 驗證 email 格式與長度（RFC 5321 上限 254 字元）
pub fn validate_email(email: &str) -> Result<(), AppError> {
    if email.len() > MAX_EMAIL_LENGTH {
        return Err(AppError::BadRequest("Email 過長".into()));
    }
    let parts: Vec<&str> = email.splitn(2, '@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(AppError::BadRequest("Email 格式無效".into()));
    }
    if !parts[1].contains('.') {
        return Err(AppError::BadRequest("Email 網域格式無效".into()));
    }
    Ok(())
}

/// 驗證上傳檔名：禁止路徑穿越、空名稱、過長名稱、及危險副檔名
pub fn validate_filename(name: &str) -> Result<(), AppError> {
    if name.is_empty() {
        return Err(AppError::BadRequest("檔名不得為空".into()));
    }
    if name.len() > MAX_FILENAME_LENGTH {
        return Err(AppError::BadRequest("檔名過長".into()));
    }
    // 防止路徑穿越
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(AppError::BadRequest("檔名含有非法字元".into()));
    }
    // 禁止以 . 開頭的隱藏檔
    if name.starts_with('.') {
        return Err(AppError::BadRequest("不允許隱藏檔名".into()));
    }
    // 拒絕可執行副檔名
    let blocked_exts = [
        ".exe", ".sh", ".bat", ".cmd", ".ps1", ".php", ".py", ".rb", ".pl", ".cgi", ".js", ".ts",
        ".jar",
    ];
    let lower = name.to_lowercase();
    for ext in &blocked_exts {
        if lower.ends_with(ext) {
            return Err(AppError::BadRequest(format!("不允許 {} 類型的檔案", ext)));
        }
    }
    Ok(())
}

/// 確認 `path` 位於 `base_dir` 之內，防止路徑穿越（Path Traversal）
pub fn validate_path_within_base(
    path: &std::path::Path,
    base_dir: &std::path::Path,
) -> Result<(), AppError> {
    let canonical_base = base_dir
        .canonicalize()
        .map_err(|_| AppError::Internal("無法解析基礎路徑".into()))?;
    let canonical_path = path
        .canonicalize()
        .map_err(|_| AppError::BadRequest("無效的檔案路徑".into()))?;
    if !canonical_path.starts_with(&canonical_base) {
        return Err(AppError::Forbidden("存取路徑超出允許範圍".into()));
    }
    Ok(())
}

/// 驗證分頁參數，避免超大偏移量造成資料庫壓力
pub fn validate_pagination(page: i64, page_size: i64) -> Result<(), AppError> {
    if page < 1 {
        return Err(AppError::BadRequest("頁碼必須 ≥ 1".into()));
    }
    if !(1..=200).contains(&page_size) {
        return Err(AppError::BadRequest("每頁筆數須介於 1 到 200 之間".into()));
    }
    Ok(())
}
