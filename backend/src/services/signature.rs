// 電子簽章服務 - GLP 合規
// 用於犧牲記錄確認、計畫核准等需要簽章的操作

use crate::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// 簽章類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignatureType {
    Approve, // 核准
    Confirm, // 確認
    Witness, // 見證
}

impl SignatureType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureType::Approve => "APPROVE",
            SignatureType::Confirm => "CONFIRM",
            SignatureType::Witness => "WITNESS",
        }
    }
}

/// 電子簽章記錄
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ElectronicSignature {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub signer_id: Uuid,
    pub signature_type: String,
    pub content_hash: String,
    pub signature_data: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub signed_at: DateTime<Utc>,
    pub is_valid: bool,
    pub invalidated_reason: Option<String>,
    pub invalidated_at: Option<DateTime<Utc>>,
    pub invalidated_by: Option<Uuid>,
    pub handwriting_svg: Option<String>,
    pub stroke_data: Option<JsonValue>,
    pub signature_method: Option<String>,
}

/// 簽章驗證結果
#[derive(Debug, Serialize)]
pub struct VerifyResult {
    pub is_valid: bool,
    pub signer_name: Option<String>,
    pub signed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}

/// 簽章建立參數
pub struct SignParams<'a> {
    pub entity_type: &'a str,
    pub entity_id: &'a str,
    pub signer_id: Uuid,
    pub signature_type: SignatureType,
    pub content: &'a str,
    pub ip_address: Option<&'a str>,
    pub user_agent: Option<&'a str>,
    pub password_hash: Option<&'a str>,
    pub handwriting_svg: Option<&'a str>,
    pub stroke_data: Option<&'a JsonValue>,
    pub signature_method: &'a str,
}

pub struct SignatureService;

impl SignatureService {
    /// 計算內容雜湊 (SHA-256)
    pub fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 建立電子簽章（密碼驗證方式）
    #[allow(clippy::too_many_arguments)]
    pub async fn sign(
        pool: &PgPool,
        entity_type: &str,
        entity_id: &str,
        signer_id: Uuid,
        password_hash: &str,
        signature_type: SignatureType,
        content: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<ElectronicSignature> {
        Self::sign_with_params(pool, SignParams {
            entity_type,
            entity_id,
            signer_id,
            signature_type,
            content,
            ip_address,
            user_agent,
            password_hash: Some(password_hash),
            handwriting_svg: None,
            stroke_data: None,
            signature_method: "password",
        })
        .await
    }

    /// 建立電子簽章（手寫簽名方式）
    #[allow(clippy::too_many_arguments)]
    pub async fn sign_with_handwriting(
        pool: &PgPool,
        entity_type: &str,
        entity_id: &str,
        signer_id: Uuid,
        signature_type: SignatureType,
        content: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        handwriting_svg: &str,
        stroke_data: Option<&JsonValue>,
    ) -> Result<ElectronicSignature> {
        Self::sign_with_params(pool, SignParams {
            entity_type,
            entity_id,
            signer_id,
            signature_type,
            content,
            ip_address,
            user_agent,
            password_hash: None,
            handwriting_svg: Some(handwriting_svg),
            stroke_data,
            signature_method: "handwriting",
        })
        .await
    }

    /// 建立電子簽章（使用參數結構體）
    async fn sign_with_params(
        pool: &PgPool,
        params: SignParams<'_>,
    ) -> Result<ElectronicSignature> {
        let content_hash = Self::compute_hash(params.content);

        let timestamp = Utc::now();
        let hash_input = params.password_hash.unwrap_or("handwriting");
        let signature_input = format!(
            "{}:{}:{}:{}",
            params.signer_id,
            content_hash,
            timestamp.timestamp(),
            hash_input
        );
        let signature_data = Self::compute_hash(&signature_input);

        let signature = sqlx::query_as::<_, ElectronicSignature>(
            r#"
            INSERT INTO electronic_signatures (
                entity_type, entity_id, signer_id, signature_type,
                content_hash, signature_data, ip_address, user_agent,
                handwriting_svg, stroke_data, signature_method
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(params.entity_type)
        .bind(params.entity_id)
        .bind(params.signer_id)
        .bind(params.signature_type.as_str())
        .bind(&content_hash)
        .bind(&signature_data)
        .bind(params.ip_address)
        .bind(params.user_agent)
        .bind(params.handwriting_svg)
        .bind(params.stroke_data)
        .bind(params.signature_method)
        .fetch_one(pool)
        .await?;

        Ok(signature)
    }

    /// 取得實體的所有有效簽章
    pub async fn get_signatures(
        pool: &PgPool,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<ElectronicSignature>> {
        let signatures = sqlx::query_as::<_, ElectronicSignature>(
            r#"
            SELECT * FROM electronic_signatures
            WHERE entity_type = $1 AND entity_id = $2 AND is_valid = true
            ORDER BY signed_at DESC
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(pool)
        .await?;

        Ok(signatures)
    }

    /// 驗證簽章
    pub async fn verify(
        pool: &PgPool,
        signature_id: Uuid,
        current_content: &str,
    ) -> Result<VerifyResult> {
        let signature = sqlx::query_as::<_, ElectronicSignature>(
            "SELECT * FROM electronic_signatures WHERE id = $1",
        )
        .bind(signature_id)
        .fetch_optional(pool)
        .await?;

        match signature {
            Some(sig) => {
                if !sig.is_valid {
                    return Ok(VerifyResult {
                        is_valid: false,
                        signer_name: None,
                        signed_at: Some(sig.signed_at),
                        failure_reason: Some(
                            sig.invalidated_reason.unwrap_or("簽章已失效".to_string()),
                        ),
                    });
                }

                // 驗證內容是否被竄改
                let current_hash = Self::compute_hash(current_content);
                if current_hash != sig.content_hash {
                    return Ok(VerifyResult {
                        is_valid: false,
                        signer_name: None,
                        signed_at: Some(sig.signed_at),
                        failure_reason: Some("內容已被修改，簽章失效".to_string()),
                    });
                }

                // 取得簽章者名稱
                let signer_name: Option<String> =
                    sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
                        .bind(sig.signer_id)
                        .fetch_optional(pool)
                        .await?;

                Ok(VerifyResult {
                    is_valid: true,
                    signer_name,
                    signed_at: Some(sig.signed_at),
                    failure_reason: None,
                })
            }
            None => Ok(VerifyResult {
                is_valid: false,
                signer_name: None,
                signed_at: None,
                failure_reason: Some("找不到簽章".to_string()),
            }),
        }
    }

    /// 使簽章失效
    pub async fn invalidate(
        pool: &PgPool,
        signature_id: Uuid,
        reason: &str,
        invalidated_by: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE electronic_signatures SET
                is_valid = false,
                invalidated_reason = $2,
                invalidated_at = NOW(),
                invalidated_by = $3
            WHERE id = $1
            "#,
        )
        .bind(signature_id)
        .bind(reason)
        .bind(invalidated_by)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 檢查實體是否已簽章
    pub async fn is_signed(pool: &PgPool, entity_type: &str, entity_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM electronic_signatures
            WHERE entity_type = $1 AND entity_id = $2 AND is_valid = true
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }

    /// 將記錄類型字串解析為資料表名稱
    fn resolve_table_name(record_type: &str) -> Result<&'static str> {
        match record_type {
            "observation" => Ok("animal_observations"),
            "surgery" => Ok("animal_surgeries"),
            "sacrifice" => Ok("animal_sacrifices"),
            _ => Err(AppError::Validation(format!(
                "不支援的記錄類型: {}",
                record_type
            ))),
        }
    }

    /// 鎖定記錄（簽章後自動鎖定，記錄 ID 為 i32）
    pub async fn lock_record(
        pool: &PgPool,
        record_type: &str,
        record_id: i32,
        locked_by: Uuid,
    ) -> Result<()> {
        let table_name = Self::resolve_table_name(record_type)?;
        let query = format!(
            "UPDATE {} SET is_locked = true, locked_at = NOW(), locked_by = $2 WHERE id = $1",
            table_name
        );
        sqlx::query(&query)
            .bind(record_id)
            .bind(locked_by)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 鎖定記錄（記錄 ID 為 UUID，用於 animal_sacrifices）
    pub async fn lock_record_uuid(
        pool: &PgPool,
        record_type: &str,
        record_id: Uuid,
        locked_by: Uuid,
    ) -> Result<()> {
        let table_name = Self::resolve_table_name(record_type)?;
        let query = format!(
            "UPDATE {} SET is_locked = true, locked_at = NOW(), locked_by = $2 WHERE id = $1",
            table_name
        );
        sqlx::query(&query)
            .bind(record_id)
            .bind(locked_by)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 檢查記錄是否已鎖定（記錄 ID 為 i32）
    pub async fn is_locked(pool: &PgPool, record_type: &str, record_id: i32) -> Result<bool> {
        let table_name = Self::resolve_table_name(record_type)?;
        let query = format!(
            "SELECT COALESCE(is_locked, false) FROM {} WHERE id = $1",
            table_name
        );
        let locked: bool = sqlx::query_scalar(&query)
            .bind(record_id)
            .fetch_optional(pool)
            .await?
            .unwrap_or(false);
        Ok(locked)
    }

    /// 檢查記錄是否已鎖定（記錄 ID 為 UUID）
    pub async fn is_locked_uuid(pool: &PgPool, record_type: &str, record_id: Uuid) -> Result<bool> {
        let table_name = Self::resolve_table_name(record_type)?;
        let query = format!(
            "SELECT COALESCE(is_locked, false) FROM {} WHERE id = $1",
            table_name
        );
        let locked: bool = sqlx::query_scalar(&query)
            .bind(record_id)
            .fetch_optional(pool)
            .await?
            .unwrap_or(false);
        Ok(locked)
    }
}

// ============================================
// 記錄附註服務
// ============================================

/// 附註類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnnotationType {
    Note,       // 一般附註
    Correction, // 更正（需簽章）
    Addendum,   // 補充說明
}

impl AnnotationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnnotationType::Note => "NOTE",
            AnnotationType::Correction => "CORRECTION",
            AnnotationType::Addendum => "ADDENDUM",
        }
    }
}

/// 記錄附註
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RecordAnnotation {
    pub id: Uuid,
    pub record_type: String,
    pub record_id: i32,
    pub annotation_type: String,
    pub content: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub signature_id: Option<Uuid>,
}

pub struct AnnotationService;

impl AnnotationService {
    /// 新增附註
    pub async fn create(
        pool: &PgPool,
        record_type: &str,
        record_id: i32,
        annotation_type: AnnotationType,
        content: &str,
        created_by: Uuid,
        signature_id: Option<Uuid>,
    ) -> Result<RecordAnnotation> {
        // 如果是 CORRECTION 類型，必須有簽章
        if annotation_type == AnnotationType::Correction && signature_id.is_none() {
            return Err(AppError::Validation("更正附註需要電子簽章".to_string()));
        }

        let annotation = sqlx::query_as::<_, RecordAnnotation>(
            r#"
            INSERT INTO record_annotations (
                record_type, record_id, annotation_type, content, created_by, signature_id
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(record_type)
        .bind(record_id)
        .bind(annotation_type.as_str())
        .bind(content)
        .bind(created_by)
        .bind(signature_id)
        .fetch_one(pool)
        .await?;

        Ok(annotation)
    }

    /// 取得記錄的所有附註
    pub async fn get_by_record(
        pool: &PgPool,
        record_type: &str,
        record_id: i32,
    ) -> Result<Vec<RecordAnnotation>> {
        let annotations = sqlx::query_as::<_, RecordAnnotation>(
            r#"
            SELECT * FROM record_annotations
            WHERE record_type = $1 AND record_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(record_type)
        .bind(record_id)
        .fetch_all(pool)
        .await?;

        Ok(annotations)
    }
}
