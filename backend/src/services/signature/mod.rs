// 電子簽章服務 - GLP 合規
// 用於犧牲記錄確認、計畫核准等需要簽章的操作

mod access;
mod content;
mod annotation;

pub use annotation::{AnnotationService, AnnotationType};

use crate::{repositories, AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use super::AuthService;

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

/// 簽章請求
#[derive(Debug, Deserialize)]
pub struct SignRequest {
    pub password: String,
    pub signature_type: String,
}

/// 簽章回應
#[derive(Debug, Serialize)]
pub struct SignResponse {
    pub signature_id: Uuid,
    pub signed_at: DateTime<Utc>,
    pub content_hash: String,
}

/// 簽章驗證結果
#[derive(Debug, Serialize)]
pub struct VerifyResult {
    pub is_valid: bool,
    pub signer_name: Option<String>,
    pub signed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}

/// 簽章詳細資訊 DTO（含簽章者姓名）
#[derive(Debug, Serialize)]
pub struct SignatureInfoDto {
    pub id: Uuid,
    pub signature_type: String,
    pub signer_name: Option<String>,
    pub signed_at: DateTime<Utc>,
    pub signature_method: Option<String>,
    pub handwriting_svg: Option<String>,
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
        password_hash: &str, // 預先驗證過的密碼雜湊
        signature_type: SignatureType,
        content: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<ElectronicSignature> {
        Self::sign_internal(
            pool,
            entity_type,
            entity_id,
            signer_id,
            Some(password_hash),
            signature_type,
            content,
            ip_address,
            user_agent,
            None,
            None,
            "password",
        )
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
        Self::sign_internal(
            pool,
            entity_type,
            entity_id,
            signer_id,
            None,
            signature_type,
            content,
            ip_address,
            user_agent,
            Some(handwriting_svg),
            stroke_data,
            "handwriting",
        )
        .await
    }

    /// 內部簽章建立邏輯（統一密碼 / 手寫兩種方式）
    #[allow(clippy::too_many_arguments)]
    async fn sign_internal(
        pool: &PgPool,
        entity_type: &str,
        entity_id: &str,
        signer_id: Uuid,
        password_hash: Option<&str>,
        signature_type: SignatureType,
        content: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        handwriting_svg: Option<&str>,
        stroke_data: Option<&JsonValue>,
        signature_method: &str,
    ) -> Result<ElectronicSignature> {
        // 計算內容雜湊
        let content_hash = Self::compute_hash(content);

        // 建立簽章資料（簽章 = 使用者ID + 內容雜湊 + 時間戳記 的雜湊）
        let timestamp = Utc::now();
        let hash_input = password_hash.unwrap_or("handwriting");
        let signature_input = format!(
            "{}:{}:{}:{}",
            signer_id,
            content_hash,
            timestamp.timestamp(),
            hash_input
        );
        let signature_data = Self::compute_hash(&signature_input);

        // 儲存到資料庫
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
        .bind(entity_type)
        .bind(entity_id)
        .bind(signer_id)
        .bind(signature_type.as_str())
        .bind(&content_hash)
        .bind(&signature_data)
        .bind(ip_address)
        .bind(user_agent)
        .bind(handwriting_svg)
        .bind(stroke_data)
        .bind(signature_method)
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
                let signer_name =
                    repositories::user::find_user_display_name_by_id(pool, sig.signer_id)
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

    /// 鎖定記錄（簽章後自動鎖定，記錄 ID 為 i32）
    pub async fn lock_record(
        pool: &PgPool,
        record_type: &str, // "observation", "surgery", "sacrifice"
        record_id: i32,
        locked_by: Uuid,
    ) -> Result<()> {
        let table_name = match record_type {
            "observation" => "animal_observations",
            "surgery" => "animal_surgeries",
            "sacrifice" => "animal_sacrifices",
            _ => {
                return Err(AppError::Validation(format!(
                    "不支援的記錄類型: {}",
                    record_type
                )))
            }
        };

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
        let table_name = match record_type {
            "sacrifice" => "animal_sacrifices",
            _ => {
                return Err(AppError::Validation(format!(
                    "不支援的記錄類型 (UUID): {}",
                    record_type
                )))
            }
        };

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
        let table_name = match record_type {
            "observation" => "animal_observations",
            "surgery" => "animal_surgeries",
            "sacrifice" => "animal_sacrifices",
            _ => {
                return Err(AppError::Validation(format!(
                    "不支援的記錄類型: {}",
                    record_type
                )))
            }
        };

        let query = format!(
            "SELECT COALESCE(is_locked, false) FROM {} WHERE id = $1",
            table_name
        );

        let is_locked: bool = sqlx::query_scalar(&query)
            .bind(record_id)
            .fetch_optional(pool)
            .await?
            .unwrap_or(false);

        Ok(is_locked)
    }

    /// 檢查記錄是否已鎖定（記錄 ID 為 UUID，用於 animal_sacrifices）
    pub async fn is_locked_uuid(pool: &PgPool, record_type: &str, record_id: Uuid) -> Result<bool> {
        let table_name = match record_type {
            "sacrifice" => "animal_sacrifices",
            _ => {
                return Err(AppError::Validation(format!(
                    "不支援的記錄類型 (UUID): {}",
                    record_type
                )))
            }
        };

        let query = format!(
            "SELECT COALESCE(is_locked, false) FROM {} WHERE id = $1",
            table_name
        );

        let is_locked: bool = sqlx::query_scalar(&query)
            .bind(record_id)
            .fetch_optional(pool)
            .await?
            .unwrap_or(false);

        Ok(is_locked)
    }

    /// 解析簽章類型字串，預設使用指定的 default
    pub fn parse_signature_type(s: Option<&str>, default: SignatureType) -> SignatureType {
        match s {
            Some("APPROVE") => SignatureType::Approve,
            Some("CONFIRM") => SignatureType::Confirm,
            Some("WITNESS") => SignatureType::Witness,
            _ => default,
        }
    }

    /// 統一簽章請求處理：驗證輸入 → 依模式建立簽章
    #[allow(clippy::too_many_arguments)]
    pub async fn sign_record(
        pool: &PgPool,
        entity_type: &str,
        entity_id: &str,
        signer_id: Uuid,
        sig_type: SignatureType,
        content: &str,
        password: Option<&str>,
        handwriting_svg: Option<&str>,
        stroke_data: Option<&JsonValue>,
    ) -> Result<ElectronicSignature> {
        let has_password = password.is_some_and(|p| !p.is_empty());
        let has_handwriting = handwriting_svg.is_some_and(|s| !s.is_empty());

        if !has_password && !has_handwriting {
            return Err(AppError::Validation("請提供密碼或手寫簽名".into()));
        }

        if has_handwriting {
            // SEC-BIZ-3: 手寫簽章仍需密碼驗證，確保身分不可否認性（GLP 21 CFR 11 合規）
            // 若同時提供密碼則驗證，否則強制要求密碼
            let pwd = password
                .filter(|p| !p.is_empty())
                .ok_or_else(|| AppError::Validation(
                    "手寫簽章仍需輸入密碼以驗證身分".into(),
                ))?;
            let _user = AuthService::verify_password_by_id(pool, signer_id, pwd)
                .await
                .map_err(|_| AppError::Unauthorized)?;

            let svg = handwriting_svg
                .ok_or_else(|| AppError::Internal("missing handwriting SVG".into()))?;
            Self::sign_with_handwriting(
                pool,
                entity_type,
                entity_id,
                signer_id,
                sig_type,
                content,
                None,
                None,
                svg,
                stroke_data,
            )
            .await
        } else {
            let pwd = password
                .ok_or_else(|| AppError::Internal("missing password".into()))?;
            let user = AuthService::verify_password_by_id(pool, signer_id, pwd)
                .await
                .map_err(|_| AppError::Unauthorized)?;
            Self::sign(
                pool,
                entity_type,
                entity_id,
                signer_id,
                &user.password_hash,
                sig_type,
                content,
                None,
                None,
            )
            .await
        }
    }

    /// 簽章詳細資訊（含簽章者姓名）
    pub async fn get_signature_infos(
        pool: &PgPool,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<SignatureInfoDto>> {
        let signatures = Self::get_signatures(pool, entity_type, entity_id).await?;
        let mut infos = Vec::with_capacity(signatures.len());
        for sig in signatures {
            let signer_name =
                repositories::user::find_user_display_name_by_id(pool, sig.signer_id).await?;
            infos.push(SignatureInfoDto {
                id: sig.id,
                signature_type: sig.signature_type,
                signer_name,
                signed_at: sig.signed_at,
                signature_method: sig.signature_method,
                handwriting_svg: sig.handwriting_svg,
            });
        }
        Ok(infos)
    }
}
