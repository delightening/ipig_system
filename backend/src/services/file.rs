//! 檔案服務：上傳、驗證（僅 magic number）、儲存與下載。
//!
//! **架構原則（方案 D，見 docs/security-compliance/security.md）**：本服務不解析／解碼圖片內容（無 libpng 等）。
//! 僅以檔案簽名（magic number）驗證格式；若有縮圖、轉檔等圖片處理需求，應由獨立可升級之服務負責。

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use zip::read::ZipArchive;

/// 上傳目錄（由 Config 初始化一次，避免每次讀取 env var）
static UPLOAD_DIR: OnceLock<String> = OnceLock::new();

use crate::constants::{
    FILE_MAX_ANIMAL_PHOTO, FILE_MAX_LEAVE_ATTACHMENT, FILE_MAX_OBSERVATION_ATTACHMENT,
    FILE_MAX_PATHOLOGY_REPORT, FILE_MAX_PROTOCOL_ATTACHMENT, FILE_MAX_SOP_DOCUMENT,
    FILE_MAX_VET_RECOMMENDATION,
};
use crate::error::AppError;
use crate::time;

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
    /// 動物照片
    AnimalPhoto,
    /// 病理報告
    PathologyReport,
    /// 獸醫師建議附件
    VetRecommendation,
    /// 請假附件
    LeaveAttachment,
    /// 觀察紀錄附件
    ObservationAttachment,
    /// SOP 文件
    SopDocument,
}

impl FileCategory {
    /// 取得儲存子目錄
    pub fn subdirectory(&self) -> &'static str {
        match self {
            FileCategory::ProtocolAttachment => "protocols",
            FileCategory::AnimalPhoto => "animals",
            FileCategory::PathologyReport => "pathology",
            FileCategory::VetRecommendation => "vet-recommendations",
            FileCategory::LeaveAttachment => "leave-attachments",
            FileCategory::ObservationAttachment => "observations",
            FileCategory::SopDocument => "sop-documents",
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
            FileCategory::AnimalPhoto | FileCategory::LeaveAttachment => vec![
                "image/jpeg",
                "image/png",
                "image/gif",
                "image/webp",
            ],
            FileCategory::VetRecommendation | FileCategory::ObservationAttachment => vec![
                "image/jpeg",
                "image/png",
                "image/gif",
                "image/webp",
                "application/pdf",
                "application/msword",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ],
            FileCategory::PathologyReport => vec![
                "application/pdf",
                "image/jpeg",
                "image/png",
                "application/msword",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ],
            FileCategory::SopDocument => vec![
                "application/pdf",
                "application/msword",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ],
        }
    }

    /// 取得最大檔案大小（bytes）
    pub fn max_file_size(&self) -> usize {
        match self {
            FileCategory::ProtocolAttachment => FILE_MAX_PROTOCOL_ATTACHMENT,
            FileCategory::AnimalPhoto => FILE_MAX_ANIMAL_PHOTO,
            FileCategory::PathologyReport => FILE_MAX_PATHOLOGY_REPORT,
            FileCategory::VetRecommendation => FILE_MAX_VET_RECOMMENDATION,
            FileCategory::LeaveAttachment => FILE_MAX_LEAVE_ATTACHMENT,
            FileCategory::ObservationAttachment => FILE_MAX_OBSERVATION_ATTACHMENT,
            FileCategory::SopDocument => FILE_MAX_SOP_DOCUMENT,
        }
    }
}

impl FileService {
    /// 驗證檔案 Magic Number（檔案簽名）是否與宣告的 MIME 類型一致
    /// SEC-14：防止攻擊者偽裝 MIME 類型上傳惡意檔案
    pub fn validate_magic_number(data: &[u8], declared_mime: &str) -> Result<(), AppError> {
        // 檔案太小無法驗證時跳過（空檔或極小檔）
        if data.len() < 4 {
            return Ok(());
        }

        let is_valid = match declared_mime {
            // PDF: %PDF (25 50 44 46)
            "application/pdf" => data.starts_with(b"%PDF"),

            // JPEG: FF D8 FF
            "image/jpeg" => data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF,

            // PNG: 89 50 4E 47 0D 0A 1A 0A
            "image/png" => data.len() >= 4 && data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47,

            // GIF: GIF87a 或 GIF89a
            "image/gif" => data.len() >= 6 && (data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a")),

            // WebP: RIFF....WEBP
            "image/webp" => data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP",

            // DOCX/XLSX (ZIP PK header): 50 4B 03 04
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
                data.starts_with(&[0x50, 0x4B, 0x03, 0x04])
            }

            // DOC/XLS (OLE2): D0 CF 11 E0
            "application/msword"
            | "application/vnd.ms-excel" => {
                data.starts_with(&[0xD0, 0xCF, 0x11, 0xE0])
            }

            // text/plain: 檢查不含二進位控制字元（NUL、BEL 等），防止偽裝二進位檔案
            "text/plain" => {
                !data.iter().any(|&b| b < 0x09 || (b > 0x0D && b < 0x20 && b != 0x1B))
            }

            // 其他未知格式，不驗證（允許通過）
            _ => true,
        };

        if !is_valid {
            return Err(AppError::Validation(format!(
                "檔案內容與宣告的類型 '{}' 不符，可能為偽裝檔案",
                declared_mime
            )));
        }

        Ok(())
    }

    /// High 3: 驗證 ZIP 檔內無路徑穿越或絕對路徑，防止惡意壓縮檔
    fn validate_zip_entries_safe(data: &[u8]) -> Result<(), AppError> {
        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor).map_err(|e| {
            AppError::Validation(format!("ZIP 格式無效或損壞: {}", e))
        })?;
        for i in 0..archive.len() {
            let entry = archive.by_index(i).map_err(|e| {
                AppError::Validation(format!("ZIP 項目讀取失敗: {}", e))
            })?;
            let name = entry.name();
            if name.contains("..") || name.starts_with('/') || name.contains('\\') {
                return Err(AppError::Validation(
                    "ZIP 內含不允許的路徑，拒絕上傳".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// 從 Config 初始化上傳目錄（應在啟動時呼叫一次）
    pub fn init_upload_dir(upload_dir: &str) {
        let _ = UPLOAD_DIR.set(upload_dir.to_string());
    }

    /// 取得上傳目錄
    fn get_upload_dir() -> PathBuf {
        PathBuf::from(UPLOAD_DIR.get().map(|s| s.as_str()).unwrap_or("./uploads"))
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
        let date_prefix = time::now_taiwan().format("%Y%m%d").to_string();
        
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
        // SEC-35: 檔名清理 — 防止路徑穿越攻擊
        if original_filename.contains('/') || original_filename.contains('\\') || original_filename.contains("..") {
            return Err(AppError::Validation(
                "Filename contains invalid characters".to_string(),
            ));
        }
        if let Some(id) = entity_id {
            if id.contains('/') || id.contains('\\') || id.contains("..") {
                return Err(AppError::Validation(
                    "Entity ID contains invalid characters".to_string(),
                ));
            }
        }

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

        // SEC-14: 驗證 Magic Number（檔案簽名）
        Self::validate_magic_number(data, mime_type)?;

        // High 3: ZIP 型檔案（DOCX/XLSX）驗證壓縮檔內無路徑穿越
        if matches!(
            mime_type,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        ) {
            Self::validate_zip_entries_safe(data)?;
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

    /// 刪除某 entity 的所有附件（檔案 + DB 記錄）
    pub async fn delete_by_entity(
        pool: &sqlx::PgPool,
        entity_type: &str,
        entity_id: &uuid::Uuid,
    ) -> Result<(), AppError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT file_path FROM attachments WHERE entity_type = $1 AND entity_id = $2"
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e))?;

        for (path,) in &rows {
            let _ = Self::delete(path).await;
        }

        sqlx::query("DELETE FROM attachments WHERE entity_type = $1 AND entity_id = $2")
            .bind(entity_type)
            .bind(entity_id)
            .execute(pool)
            .await
            .map_err(|e: sqlx::Error| AppError::Database(e))?;

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

    // ==========================================
    // generate_unique_filename 測試
    // ==========================================

    #[test]
    fn test_generate_unique_filename_pdf() {
        let filename = FileService::generate_unique_filename("test.pdf", "application/pdf");
        assert!(filename.ends_with(".pdf"));
        assert!(filename.len() > 20); // 日期前綴 + UUID
    }

    #[test]
    fn test_generate_unique_filename_uses_original_extension() {
        let filename = FileService::generate_unique_filename("photo.jpg", "image/jpeg");
        assert!(filename.ends_with(".jpg"));
    }

    #[test]
    fn test_generate_unique_filename_fallback_to_mime() {
        // 原始檔名無副檔名，使用 MIME 類型推斷
        let filename = FileService::generate_unique_filename("noext", "image/png");
        assert!(filename.ends_with(".png"));
    }

    #[test]
    fn test_generate_unique_filename_unknown_mime() {
        // 未知 MIME 類型，fallback 為 .bin
        let filename = FileService::generate_unique_filename("noext", "application/unknown");
        assert!(filename.ends_with(".bin"));
    }

    #[test]
    fn test_generate_unique_filename_uniqueness() {
        let f1 = FileService::generate_unique_filename("a.pdf", "application/pdf");
        let f2 = FileService::generate_unique_filename("a.pdf", "application/pdf");
        assert_ne!(f1, f2, "每次呼叫應產生不同的檔名");
    }

    // ==========================================
    // get_extension_from_mime 測試
    // ==========================================

    #[test]
    fn test_extension_from_mime_known_types() {
        assert_eq!(FileService::get_extension_from_mime("application/pdf"), Some("pdf"));
        assert_eq!(FileService::get_extension_from_mime("image/jpeg"), Some("jpg"));
        assert_eq!(FileService::get_extension_from_mime("image/png"), Some("png"));
        assert_eq!(FileService::get_extension_from_mime("image/gif"), Some("gif"));
        assert_eq!(FileService::get_extension_from_mime("image/webp"), Some("webp"));
        assert_eq!(FileService::get_extension_from_mime("text/plain"), Some("txt"));
        assert_eq!(FileService::get_extension_from_mime("application/msword"), Some("doc"));
        assert_eq!(
            FileService::get_extension_from_mime("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
            Some("docx")
        );
    }

    #[test]
    fn test_extension_from_mime_unknown() {
        assert_eq!(FileService::get_extension_from_mime("application/octet-stream"), None);
    }

    // ==========================================
    // get_extension_from_filename 測試
    // ==========================================

    #[test]
    fn test_extension_from_filename() {
        assert_eq!(FileService::get_extension_from_filename("report.PDF"), Some("pdf".to_string()));
        assert_eq!(FileService::get_extension_from_filename("no_extension"), None);
        // Rust 的 Path::extension() 對 ".hidden" 回傳 None（視為檔名非副檔名）
        assert_eq!(FileService::get_extension_from_filename(".hidden"), None);
    }

    // ==========================================
    // guess_mime_type 測試
    // ==========================================

    #[test]
    fn test_guess_mime_type() {
        assert_eq!(FileService::guess_mime_type(Path::new("test.pdf")), "application/pdf");
        assert_eq!(FileService::guess_mime_type(Path::new("photo.JPG")), "image/jpeg");
        assert_eq!(FileService::guess_mime_type(Path::new("photo.jpeg")), "image/jpeg");
        assert_eq!(FileService::guess_mime_type(Path::new("file.xyz")), "application/octet-stream");
    }

    // ==========================================
    // FileCategory 測試
    // ==========================================

    #[test]
    fn test_allowed_mime_types() {
        assert!(FileCategory::ProtocolAttachment.allowed_mime_types().contains(&"application/pdf"));
        assert!(FileCategory::AnimalPhoto.allowed_mime_types().contains(&"image/jpeg"));
        assert!(FileCategory::AnimalPhoto.allowed_mime_types().contains(&"image/webp"));
        // 動物照片不允許 PDF
        assert!(!FileCategory::AnimalPhoto.allowed_mime_types().contains(&"application/pdf"));
    }

    #[test]
    fn test_file_category_subdirectory() {
        assert_eq!(FileCategory::ProtocolAttachment.subdirectory(), "protocols");
        assert_eq!(FileCategory::AnimalPhoto.subdirectory(), "animals");
        assert_eq!(FileCategory::PathologyReport.subdirectory(), "pathology");
        assert_eq!(FileCategory::VetRecommendation.subdirectory(), "vet-recommendations");
        assert_eq!(FileCategory::LeaveAttachment.subdirectory(), "leave-attachments");
    }

    #[test]
    fn test_file_category_max_size() {
        assert_eq!(FileCategory::ProtocolAttachment.max_file_size(), 30 * 1024 * 1024);
        assert_eq!(FileCategory::AnimalPhoto.max_file_size(), 10 * 1024 * 1024);
        assert_eq!(FileCategory::PathologyReport.max_file_size(), 30 * 1024 * 1024);
    }

    // ==========================================
    // SEC-14: Magic Number 驗證測試
    // ==========================================

    #[test]
    fn test_magic_number_pdf_valid() {
        let data = b"%PDF-1.4 fake pdf content";
        assert!(FileService::validate_magic_number(data, "application/pdf").is_ok());
    }

    #[test]
    fn test_magic_number_pdf_invalid() {
        let data = b"This is not a PDF file";
        assert!(FileService::validate_magic_number(data, "application/pdf").is_err());
    }

    #[test]
    fn test_magic_number_jpeg_valid() {
        let data = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert!(FileService::validate_magic_number(data, "image/jpeg").is_ok());
    }

    #[test]
    fn test_magic_number_jpeg_invalid() {
        let data = b"not a jpeg";
        assert!(FileService::validate_magic_number(data, "image/jpeg").is_err());
    }

    #[test]
    fn test_magic_number_png_valid() {
        let data = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(FileService::validate_magic_number(data, "image/png").is_ok());
    }

    #[test]
    fn test_magic_number_png_invalid() {
        let data = b"not a png file";
        assert!(FileService::validate_magic_number(data, "image/png").is_err());
    }

    #[test]
    fn test_magic_number_gif_valid() {
        assert!(FileService::validate_magic_number(b"GIF89a content", "image/gif").is_ok());
        assert!(FileService::validate_magic_number(b"GIF87a content", "image/gif").is_ok());
    }

    #[test]
    fn test_magic_number_gif_invalid() {
        assert!(FileService::validate_magic_number(b"GIF86x nope", "image/gif").is_err());
    }

    #[test]
    fn test_magic_number_webp_valid() {
        let mut data = Vec::from(b"RIFF" as &[u8]);
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // size placeholder
        data.extend_from_slice(b"WEBP");
        assert!(FileService::validate_magic_number(&data, "image/webp").is_ok());
    }

    #[test]
    fn test_magic_number_docx_valid() {
        let data = &[0x50, 0x4B, 0x03, 0x04, 0x14, 0x00];
        assert!(FileService::validate_magic_number(
            data,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        ).is_ok());
    }

    #[test]
    fn test_magic_number_doc_valid() {
        let data = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1];
        assert!(FileService::validate_magic_number(data, "application/msword").is_ok());
    }

    #[test]
    fn test_magic_number_unknown_mime_passes() {
        // 未知 MIME 類型不驗證，直接通過
        assert!(FileService::validate_magic_number(b"anything", "text/plain").is_ok());
    }

    #[test]
    fn test_magic_number_small_file_passes() {
        // 小於 4 bytes 的檔案跳過驗證
        assert!(FileService::validate_magic_number(b"ab", "application/pdf").is_ok());
    }
}
