//! R32-A6: PDF artifact 永久存證 model。
//!
//! 對應 migration 053 `pdf_artifacts` 表。每次「下載 PDF」endpoint 觸發時，
//! 由 `services/pdf_artifact.rs` 寫入一筆，與業務操作同 tx 確保 atomic。
//!
//! GLP / 21 CFR §11 對齊：
//! - §11.50 (signature meanings)：`electronic_signature_id` 連結
//! - §11.10(c) (record integrity)：`pdf_blob_hash` 用 SHA-256 防外部竄改
//! - GLP §58 audit trail：`hmac_chain_link` 連 R26 HMAC chain

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PdfArtifact {
    pub id: Uuid,

    /// 業務 entity 類型，e.g. `"protocol"` / `"animal_medical"` / `"surgery_summary"` / `"audit_log"`
    pub resource_type: String,
    /// entity 的 PK（TEXT 而非 UUID — audit_log 用複合 key 字串化）
    pub resource_id: String,

    /// SHA-256 hex of PDF binary
    pub pdf_blob_hash: String,
    pub pdf_size_bytes: i64,
    /// pdf-service `DOCX_REGISTRY` 的 key
    pub doc_type: String,

    pub generated_by: Uuid,
    pub generated_at: DateTime<Utc>,
    /// 對齊 codebase 既有 user_activity_logs.ip_address 慣例：DB 存 inet，
    /// Rust 端用 String + SELECT `request_ip::text as request_ip` 取出，
    /// INSERT 用 `$N::inet` cast 寫入。
    pub request_ip: Option<String>,
    pub user_agent: Option<String>,

    /// 21 CFR §11.50 signature 連結；NULL = 純檢視匯出
    pub electronic_signature_id: Option<Uuid>,
    /// 對應 `user_activity_logs.id` (PDF_EXPORT_REQUESTED 事件)
    pub hmac_chain_link: Option<String>,

    pub attachment_id: Option<Uuid>,
    /// `"attachment"` / `"transient"` / `"s3"`
    pub storage_strategy: String,

    pub created_at: DateTime<Utc>,
}

/// 寫入 pdf_artifacts 用的 request struct。
///
/// caller 通常在「PDF 已產出 + audit log 已寫」之後組此 struct + 呼
/// `services::pdf_artifact::insert_artifact_tx`。
#[derive(Debug, Clone)]
pub struct CreatePdfArtifact<'a> {
    pub resource_type: &'a str,
    pub resource_id: &'a str,
    pub pdf_blob_hash: &'a str,
    pub pdf_size_bytes: i64,
    pub doc_type: &'a str,
    pub generated_by: Uuid,
    pub request_ip: Option<&'a str>,
    pub user_agent: Option<&'a str>,
    pub electronic_signature_id: Option<Uuid>,
    pub hmac_chain_link: Option<&'a str>,
    pub attachment_id: Option<Uuid>,
    pub storage_strategy: PdfStorageStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfStorageStrategy {
    /// 透過既有 `attachments` 表存實體檔案（預設）
    Attachment,
    /// 不存盤（即時下載即丟）— 純稽核紀錄
    Transient,
    /// 預留未來雲端儲存路徑
    S3,
}

impl PdfStorageStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Attachment => "attachment",
            Self::Transient => "transient",
            Self::S3 => "s3",
        }
    }
}
