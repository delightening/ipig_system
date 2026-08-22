//! R32-A6: PDF artifact service — GLP 永久存證寫入。
//!
//! 設計原則：
//! - **Insert-only**：artifact 寫入後不允許 UPDATE / DELETE（GLP §58 audit trail
//!   永久保留要求；類似 user_activity_logs 的 immutability）
//! - **同 tx 寫入**：與「PDF 已產出 + audit log 已寫」放同一個 Transaction，
//!   失敗任一步整批 rollback，避免 partial state
//! - **caller 提供 hash**：本 service 不算 SHA-256，由 caller 在拿到 PDF binary
//!   後直接算（caller 對 binary lifecycle 最清楚）

use sha2::{Digest, Sha256};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{
    models::pdf_artifact::{CreatePdfArtifact, PdfArtifact, PdfStorageStrategy},
    AppError, Result,
};

/// 在給定 tx 內 INSERT 一筆 pdf_artifact，回傳新 row 的 id。
///
/// 失敗（FK violation / CHECK constraint 等）→ caller drop tx 即 rollback。
pub async fn insert_artifact_tx<'c>(
    tx: &mut Transaction<'c, Postgres>,
    req: CreatePdfArtifact<'_>,
) -> Result<Uuid> {
    if req.pdf_size_bytes < 0 {
        return Err(AppError::Validation("pdf_size_bytes 不可為負".into()));
    }
    // hex 驗證限定 lowercase — 與 compute_pdf_hash 產出一致 + 對齊 migration 053
    // CHECK constraint。任何 caller 自己 hash 應走 compute_pdf_hash，避免大寫變體。
    if req.pdf_blob_hash.len() != 64
        || !req
            .pdf_blob_hash
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    {
        return Err(AppError::Validation(format!(
            "pdf_blob_hash 應為 64 字元 lowercase SHA-256 hex（收到 {} 字元，必須 [0-9a-f]）",
            req.pdf_blob_hash.len()
        )));
    }
    // service 層先驗 storage_strategy / attachment_id 一致性 → 提供清晰錯誤訊息
    // （優於 DB CHECK constraint 拋的 generic constraint error）
    match (req.storage_strategy, req.attachment_id) {
        (PdfStorageStrategy::Attachment, None) => {
            return Err(AppError::Validation(
                "storage_strategy=attachment 必須提供 attachment_id".into(),
            ));
        }
        (PdfStorageStrategy::Transient, Some(_)) => {
            return Err(AppError::Validation(
                "storage_strategy=transient 不可帶 attachment_id".into(),
            ));
        }
        _ => {}
    }

    let storage = req.storage_strategy.as_str();
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO pdf_artifacts (
            resource_type, resource_id, pdf_blob_hash, pdf_size_bytes, doc_type,
            generated_by, request_ip, user_agent,
            electronic_signature_id, hmac_chain_link,
            attachment_id, storage_strategy
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7::inet, $8, $9, $10, $11, $12)
        RETURNING id
        "#,
    )
    .bind(req.resource_type)
    .bind(req.resource_id)
    .bind(req.pdf_blob_hash)
    .bind(req.pdf_size_bytes)
    .bind(req.doc_type)
    .bind(req.generated_by)
    .bind(req.request_ip)
    .bind(req.user_agent)
    .bind(req.electronic_signature_id)
    .bind(req.hmac_chain_link)
    .bind(req.attachment_id)
    .bind(storage)
    .fetch_one(&mut **tx)
    .await?;

    Ok(row.0)
}

/// 計算 PDF binary 的 SHA-256，回傳 64 字元 hex string。
///
/// 抽出 helper 是因為 caller 拿到 PDF 後通常立即算 hash + 寫 artifact，
/// 集中於此確保所有 caller 用同一演算法（避免日後 SHA-512 / Blake3 不一致）。
pub fn compute_pdf_hash(pdf_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pdf_bytes);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// 用 id 取單筆 artifact。供「重新下載已存證 PDF」場景使用（從 attachments
/// 取 binary + 比對 pdf_blob_hash 確認未被竄改）。R32-A4 endpoint 整合時呼叫。
pub async fn find_artifact_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<PdfArtifact>> {
    let row = sqlx::query_as::<_, PdfArtifact>(
        r#"SELECT id, resource_type, resource_id, pdf_blob_hash, pdf_size_bytes,
                  doc_type, generated_by, generated_at,
                  request_ip::text AS request_ip, user_agent,
                  electronic_signature_id, hmac_chain_link,
                  attachment_id, storage_strategy, created_at
           FROM pdf_artifacts WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// 列出某資源的所有 artifact（DESC by generated_at）。
///
/// 給 GLP 稽核 UI 用：「此 protocol 歷年來被誰下載過幾次 PDF」。後續 admin UI 呼叫。
pub async fn list_artifacts_by_resource(
    pool: &sqlx::PgPool,
    resource_type: &str,
    resource_id: &str,
) -> Result<Vec<PdfArtifact>> {
    let rows = sqlx::query_as::<_, PdfArtifact>(
        r#"
        SELECT id, resource_type, resource_id, pdf_blob_hash, pdf_size_bytes,
               doc_type, generated_by, generated_at,
               request_ip::text AS request_ip, user_agent,
               electronic_signature_id, hmac_chain_link,
               attachment_id, storage_strategy, created_at
        FROM pdf_artifacts
        WHERE resource_type = $1 AND resource_id = $2
        ORDER BY generated_at DESC
        "#,
    )
    .bind(resource_type)
    .bind(resource_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_pdf_hash_returns_64_hex_chars() {
        let hash = compute_pdf_hash(b"test pdf content");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn compute_pdf_hash_deterministic() {
        let h1 = compute_pdf_hash(b"hello");
        let h2 = compute_pdf_hash(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn compute_pdf_hash_differs_for_different_input() {
        let h1 = compute_pdf_hash(b"hello");
        let h2 = compute_pdf_hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn compute_pdf_hash_only_produces_lowercase_hex() {
        // 對齊 insert_artifact_tx 的 hex 格式驗證 — 大寫 / 非 hex 必拒
        let hash = compute_pdf_hash(b"verify case");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hash, hash.to_lowercase(), "compute_pdf_hash 應產 lowercase");
    }

    #[test]
    fn storage_strategy_as_str_matches_db_constraint() {
        // 對齊 migration 053 CHECK 約束
        assert_eq!(PdfStorageStrategy::Attachment.as_str(), "attachment");
        assert_eq!(PdfStorageStrategy::Transient.as_str(), "transient");
        assert_eq!(PdfStorageStrategy::S3.as_str(), "s3");
    }

    /// 純驗證 hex 規則（不打 DB）— 與 insert_artifact_tx 內判斷邏輯同步。
    /// 若改 hex 規則時，重複此邏輯維護兩處的成本被測試壓住。
    fn is_valid_lowercase_sha256_hex(s: &str) -> bool {
        s.len() == 64
            && s.chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    }

    #[test]
    fn hex_validation_rejects_uppercase() {
        // 對齊 PR #318 round 2 review：大寫 hex 不該被接受（compute_pdf_hash 產
        // lowercase + DB CHECK 限定 [0-9a-f]）
        let upper = "DEADBEEF".repeat(8); // 64 chars 大寫
        assert!(!is_valid_lowercase_sha256_hex(&upper));
        let lower = "deadbeef".repeat(8);
        assert!(is_valid_lowercase_sha256_hex(&lower));
    }

    #[test]
    fn hex_validation_rejects_wrong_length_or_non_hex() {
        assert!(!is_valid_lowercase_sha256_hex("a")); // too short
        assert!(!is_valid_lowercase_sha256_hex(&"a".repeat(63))); // 63
        assert!(!is_valid_lowercase_sha256_hex(&"g".repeat(64))); // not hex
        assert!(is_valid_lowercase_sha256_hex(&"0".repeat(64))); // valid edge
    }
}
