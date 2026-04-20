use crate::error::AppError;

const MAX_EMAIL_LENGTH: usize = 254;
const MAX_FILENAME_LENGTH: usize = 255;

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
        ".exe", ".sh", ".bat", ".cmd", ".ps1", ".php", ".py",
        ".rb", ".pl", ".cgi", ".js", ".ts", ".jar",
    ];
    let lower = name.to_lowercase();
    for ext in &blocked_exts {
        if lower.ends_with(ext) {
            return Err(AppError::BadRequest(
                format!("不允許 {} 類型的檔案", ext).into(),
            ));
        }
    }
    Ok(())
}

/// 確認 `path` 位於 `base_dir` 之內，防止路徑穿越（Path Traversal）
pub fn validate_path_within_base(path: &std::path::Path, base_dir: &std::path::Path) -> Result<(), AppError> {
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
