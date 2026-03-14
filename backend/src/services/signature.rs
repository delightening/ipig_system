// 電子簽章服務 - GLP 合規
// 用於犧牲記錄確認、計畫核准等需要簽章的操作

use crate::{middleware::CurrentUser, repositories, AppError, Result};
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

    // ============================================
    // 存取權限檢查（IDOR 防護）
    // ============================================

    /// 檢查使用者是否有權存取安樂死單據（PI、VET、CHAIR 或管理員）
    pub async fn check_euthanasia_access(
        pool: &PgPool,
        order_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if current_user.has_permission("animal.euthanasia.arbitrate") || current_user.is_admin() {
            return Ok(());
        }
        let related: Option<(Uuid, Uuid)> = sqlx::query_as(
            "SELECT pi_user_id, vet_user_id FROM euthanasia_orders WHERE id = $1",
        )
        .bind(order_id)
        .fetch_optional(pool)
        .await?;

        match related {
            Some((pi_id, vet_id)) if pi_id == current_user.id || vet_id == current_user.id => {
                Ok(())
            }
            Some(_) => Err(AppError::Forbidden("無權存取此安樂死單據".into())),
            None => Err(AppError::NotFound("找不到安樂死單據".into())),
        }
    }

    /// 檢查使用者是否有權存取轉讓記錄（透過動物所屬計畫關聯）
    pub async fn check_transfer_access(
        pool: &PgPool,
        transfer_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
            return Ok(());
        }
        let has_access: Option<(i64,)> = sqlx::query_as(
            r#"SELECT 1 FROM animal_transfers t
               JOIN animals a ON t.animal_id = a.id
               LEFT JOIN user_protocols up ON up.protocol_id = a.protocol_id
               WHERE t.id = $1 AND up.user_id = $2"#,
        )
        .bind(transfer_id)
        .bind(current_user.id)
        .fetch_optional(pool)
        .await?;

        if has_access.is_some() {
            Ok(())
        } else {
            Err(AppError::Forbidden("無權存取此轉讓記錄".into()))
        }
    }

    /// 檢查使用者是否有權存取計畫書（PI、共同編輯者、審查委員或管理員）
    pub async fn check_protocol_access(
        pool: &PgPool,
        protocol_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
            return Ok(());
        }
        let has_access: Option<(i64,)> = sqlx::query_as(
            r#"SELECT 1 FROM user_protocols
               WHERE protocol_id = $1 AND user_id = $2"#,
        )
        .bind(protocol_id)
        .bind(current_user.id)
        .fetch_optional(pool)
        .await?;

        if has_access.is_some() {
            Ok(())
        } else {
            Err(AppError::Forbidden("無權存取此計畫書".into()))
        }
    }

    /// 檢查使用者是否有權存取動物記錄（透過動物所屬計畫關聯，記錄 ID 為 i32）
    pub async fn check_animal_record_access(
        pool: &PgPool,
        table: &str,
        record_id: i32,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
            return Ok(());
        }
        let query = format!(
            r#"SELECT 1 FROM {} r
               JOIN animals a ON r.animal_id = a.id
               LEFT JOIN user_protocols up ON up.protocol_id = a.protocol_id
               WHERE r.id = $1 AND up.user_id = $2"#,
            table
        );
        let has_access: Option<(i64,)> = sqlx::query_as(&query)
            .bind(record_id)
            .bind(current_user.id)
            .fetch_optional(pool)
            .await?;

        if has_access.is_some() {
            Ok(())
        } else {
            Err(AppError::Forbidden("無權存取此記錄".into()))
        }
    }

    /// 檢查使用者是否有權存取動物記錄（記錄 ID 為 UUID）
    pub async fn check_animal_record_access_uuid(
        pool: &PgPool,
        table: &str,
        record_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<()> {
        if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
            return Ok(());
        }
        let query = format!(
            r#"SELECT 1 FROM {} r
               JOIN animals a ON r.animal_id = a.id
               LEFT JOIN user_protocols up ON up.protocol_id = a.protocol_id
               WHERE r.id = $1 AND up.user_id = $2"#,
            table
        );
        let has_access: Option<(i64,)> = sqlx::query_as(&query)
            .bind(record_id)
            .bind(current_user.id)
            .fetch_optional(pool)
            .await?;

        if has_access.is_some() {
            Ok(())
        } else {
            Err(AppError::Forbidden("無權存取此記錄".into()))
        }
    }

    // ============================================
    // 記錄內容查詢（用於簽章雜湊計算）
    // ============================================

    /// 取得犧牲記錄內容（用於生成簽章雜湊）
    pub async fn fetch_sacrifice_content(pool: &PgPool, id: Uuid) -> Result<String> {
        sqlx::query_scalar(
            r#"SELECT CONCAT(
                'sacrifice_id:', id::text,
                ',animal_id:', animal_id::text,
                ',date:', COALESCE(sacrifice_date::text, ''),
                ',confirmed:', confirmed_sacrifice::text
            ) FROM animal_sacrifices WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到犧牲記錄".into()))
    }

    /// 取得觀察記錄內容（用於生成簽章雜湊）
    pub async fn fetch_observation_content(pool: &PgPool, id: i32) -> Result<String> {
        sqlx::query_scalar(
            r#"SELECT CONCAT(
                'observation_id:', id::text,
                ',animal_id:', animal_id::text,
                ',date:', event_date::text,
                ',content:', content
            ) FROM animal_observations WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到觀察記錄".into()))
    }

    /// 取得安樂死單據內容（用於生成簽章雜湊）
    pub async fn fetch_euthanasia_content(pool: &PgPool, id: Uuid) -> Result<String> {
        sqlx::query_scalar(
            r#"SELECT CONCAT(
                'euthanasia_id:', id::text,
                ',animal_id:', animal_id::text,
                ',reason:', reason,
                ',status:', status
            ) FROM euthanasia_orders WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到安樂死單據".into()))
    }

    /// 取得轉讓記錄內容（用於生成簽章雜湊）
    pub async fn fetch_transfer_content(pool: &PgPool, id: Uuid) -> Result<String> {
        sqlx::query_scalar(
            r#"SELECT CONCAT(
                'transfer_id:', id::text,
                ',animal_id:', animal_id::text,
                ',from_iacuc:', from_iacuc_no,
                ',status:', status
            ) FROM animal_transfers WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到轉讓記錄".into()))
    }

    /// 取得計畫書內容（用於生成簽章雜湊）
    pub async fn fetch_protocol_content(pool: &PgPool, id: Uuid) -> Result<String> {
        sqlx::query_scalar(
            r#"SELECT CONCAT(
                'protocol_id:', id::text,
                ',title:', title,
                ',status:', status
            ) FROM protocols WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到計劃書".into()))
    }

    // ============================================
    // 統一簽章流程
    // ============================================

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

    // ============================================
    // 簽章狀態查詢（含簽章者姓名）
    // ============================================

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

/// 建立附註請求
#[derive(Debug, Deserialize)]
pub struct CreateAnnotationRequest {
    pub content: String,
    pub annotation_type: String,
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

    /// 取得附註清單並附加建立者姓名
    pub async fn enrich_with_names(
        pool: &PgPool,
        annotations: Vec<RecordAnnotation>,
    ) -> Result<Vec<(RecordAnnotation, Option<String>)>> {
        let mut result = Vec::with_capacity(annotations.len());
        for ann in annotations {
            let name =
                repositories::user::find_user_display_name_by_id(pool, ann.created_by).await?;
            result.push((ann, name));
        }
        Ok(result)
    }
}
