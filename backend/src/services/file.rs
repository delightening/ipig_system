use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use chrono::Utc;

use crate::error::AppError;

/// 檔案服務 - 處理檔案上傳、下載與管理
pub struct FileService;

/// 上傳結果
#[derive(Debug, Clone)]
pub struct UploadResult {
    pub file_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
}

/// 檔案類別
#[derive(Debug, Clone, Copy)]
pub enum FileCategory {
    /// AUP 計畫附件
    ProtocolAttachment,
    /// 豬隻照片
    PigPhoto,
    /// 病理報告
    PathologyReport,
    /// 獸醫師建議附件
    VetRecommendation,
    /// 請假附件
    LeaveAttachment,
}

impl FileCategory {
    /// 取得儲存子目錄
    pub fn subdirectory(&self) -> &'static str {
        match self {
            FileCategory::ProtocolAttachment => "protocols",
            FileCategory::PigPhoto => "pigs",
            FileCategory::PathologyReport => "pathology",
            FileCategory::VetRecommendation => "vet-recommendations",
            FileCategory::LeaveAttachment => "leave-attachments",
        }
    }

    /// 取得允許的 MIME 類型
    pub fn allowed_mime_types(&self) -> Vec<&'static str> {
        match self {
            FileCategory::ProtocolAttachment => vec![
                "application/pdf",
                "application/msword",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "application/vnd.ms-excel",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "image/jpeg",
                "image/png",
                "image/gif",
                "text/plain",
            ],
            FileCategory::PigPhoto | FileCategory::VetRecommendation | FileCategory::LeaveAttachment => vec![
                "image/jpeg",
                "image/png",
                "image/gif",
                "image/webp",
            ],
            FileCategory::PathologyReport => vec![
                "application/pdf",
                "image/jpeg",
                "image/png",
                "application/msword",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ],
        }
    }

    /// 取得最大檔案大小（bytes）
    pub fn max_file_size(&self) -> usize {
        match self {
            FileCategory::ProtocolAttachment => 50 * 1024 * 1024, // 50 MB
            FileCategory::PigPhoto => 10 * 1024 * 1024,           // 10 MB
            FileCategory::PathologyReport => 30 * 1024 * 1024,    // 30 MB
            FileCategory::VetRecommendation => 10 * 1024 * 1024,  // 10 MB
            FileCategory::LeaveAttachment => 10 * 1024 * 1024,    // 10 MB
        }
    }
}

impl FileService {
    /// 取得上傳目錄
    fn get_upload_dir() -> PathBuf {
        std::env::var("UPLOAD_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./uploads"))
    }

    /// 確保目錄存在
    async fn ensure_dir_exists(dir: &Path) -> Result<(), AppError> {
        if !dir.exists() {
            fs::create_dir_all(dir).await.map_err(|e| {
                AppError::Internal(format!("Failed to create directory: {}", e))
            })?;
        }
        Ok(())
    }

    /// 從 MIME 類型推斷副檔名
    fn get_extension_from_mime(mime_type: &str) -> Option<&'static str> {
        match mime_type {
            "application/pdf" => Some("pdf"),
            "application/msword" => Some("doc"),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => Some("docx"),
            "application/vnd.ms-excel" => Some("xls"),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Some("xlsx"),
            "image/jpeg" => Some("jpg"),
            "image/png" => Some("png"),
            "image/gif" => Some("gif"),
            "image/webp" => Some("webp"),
            "text/plain" => Some("txt"),
            _ => None,
        }
    }

    /// 從檔名取得副檔名
    fn get_extension_from_filename(filename: &str) -> Option<String> {
        Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    /// 產生唯一檔名
    fn generate_unique_filename(original_filename: &str, mime_type: &str) -> String {
        let file_id = Uuid::new_v4().to_string();
        let date_prefix = Utc::now().format("%Y%m%d").to_string();
        
        // 優先使用原始檔名的副檔名
        let extension = Self::get_extension_from_filename(original_filename)
            .or_else(|| Self::get_extension_from_mime(mime_type).map(String::from))
            .unwrap_or_else(|| "bin".to_string());
        
        format!("{}_{}.{}", date_prefix, file_id, extension)
    }

    /// 上傳檔案
    pub async fn upload(
        category: FileCategory,
        original_filename: &str,
        mime_type: &str,
        data: &[u8],
        entity_id: Option<&str>,
    ) -> Result<UploadResult, AppError> {
        // 驗證 MIME 類型
        if !category.allowed_mime_types().contains(&mime_type) {
            return Err(AppError::Validation(format!(
                "File type '{}' is not allowed for this category",
                mime_type
            )));
        }

        // 驗證檔案大小
        if data.len() > category.max_file_size() {
            return Err(AppError::Validation(format!(
                "File size exceeds maximum allowed size of {} MB",
                category.max_file_size() / 1024 / 1024
            )));
        }

        // 建立目錄結構
        let base_dir = Self::get_upload_dir();
        let category_dir = base_dir.join(category.subdirectory());
        
        // 如果有實體 ID，建立子目錄
        let target_dir = if let Some(id) = entity_id {
            category_dir.join(id)
        } else {
            category_dir
        };
        
        Self::ensure_dir_exists(&target_dir).await?;

        // 產生唯一檔名
        let unique_filename = Self::generate_unique_filename(original_filename, mime_type);
        let file_path = target_dir.join(&unique_filename);

        // 寫入檔案
        let mut file = fs::File::create(&file_path).await.map_err(|e| {
            AppError::Internal(format!("Failed to create file: {}", e))
        })?;
        
        file.write_all(data).await.map_err(|e| {
            AppError::Internal(format!("Failed to write file: {}", e))
        })?;

        // 計算相對路徑
        let relative_path = file_path
            .strip_prefix(&base_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| unique_filename.clone());

        Ok(UploadResult {
            file_id: Uuid::new_v4().to_string(),
            file_name: original_filename.to_string(),
            file_path: relative_path,
            file_size: data.len() as i64,
            mime_type: mime_type.to_string(),
        })
    }

    /// 讀取檔案
    pub async fn read(relative_path: &str) -> Result<(Vec<u8>, String), AppError> {
        let base_dir = Self::get_upload_dir();
        let file_path = base_dir.join(relative_path);

        // 安全檢查：確保路徑在上傳目錄內
        let canonical_base = base_dir.canonicalize().unwrap_or(base_dir.clone());
        let canonical_file = file_path.canonicalize().map_err(|_| {
            AppError::NotFound("File not found".to_string())
        })?;
        
        if !canonical_file.starts_with(&canonical_base) {
            return Err(AppError::Forbidden("Invalid file path".to_string()));
        }

        let data = fs::read(&canonical_file).await.map_err(|_| {
            AppError::NotFound("File not found".to_string())
        })?;

        // 推斷 MIME 類型
        let mime_type = Self::guess_mime_type(&canonical_file);

        Ok((data, mime_type))
    }

    /// 刪除檔案
    pub async fn delete(relative_path: &str) -> Result<(), AppError> {
        let base_dir = Self::get_upload_dir();
        let file_path = base_dir.join(relative_path);

        // 安全檢查
        let canonical_base = base_dir.canonicalize().unwrap_or(base_dir.clone());
        if let Ok(canonical_file) = file_path.canonicalize() {
            if !canonical_file.starts_with(&canonical_base) {
                return Err(AppError::Forbidden("Invalid file path".to_string()));
            }
            
            fs::remove_file(canonical_file).await.map_err(|e| {
                AppError::Internal(format!("Failed to delete file: {}", e))
            })?;
        }
        // 如果檔案不存在，靜默成功

        Ok(())
    }

    /// 推斷 MIME 類型
    fn guess_mime_type(path: &Path) -> String {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        match extension.as_deref() {
            Some("pdf") => "application/pdf".to_string(),
            Some("doc") => "application/msword".to_string(),
            Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            Some("xls") => "application/vnd.ms-excel".to_string(),
            Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
            Some("png") => "image/png".to_string(),
            Some("gif") => "image/gif".to_string(),
            Some("webp") => "image/webp".to_string(),
            Some("txt") => "text/plain".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }

    /// 檢查檔案是否存在
    pub async fn exists(relative_path: &str) -> bool {
        let base_dir = Self::get_upload_dir();
        let file_path = base_dir.join(relative_path);
        file_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_unique_filename() {
        let filename = FileService::generate_unique_filename("test.pdf", "application/pdf");
        assert!(filename.ends_with(".pdf"));
        assert!(filename.len() > 20); // 確保有日期前綴和 UUID
    }

    #[test]
    fn test_allowed_mime_types() {
        assert!(FileCategory::ProtocolAttachment.allowed_mime_types().contains(&"application/pdf"));
        assert!(FileCategory::PigPhoto.allowed_mime_types().contains(&"image/jpeg"));
    }
}
