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

use super::{audit::AuditService, AuthService};

// R30-7: signature_data 編碼版本
/// Legacy plain SHA-256（pre-R30-7）。舊資料永久保持此版本。
pub(crate) const SIGNATURE_HMAC_VERSION_LEGACY: i16 = 1;
/// HMAC-SHA256 + secret（共用 AUDIT_HMAC_KEY）。新簽章預設使用此版本。
pub(crate) const SIGNATURE_HMAC_VERSION_V2: i16 = 2;

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

/// 簽章意義（21 CFR §11.50(a)(3)）
///
/// 對應 §11.50 字面要求：「the meaning (such as review, approval, responsibility,
/// or authorship) associated with the signature」。
///
/// 與 `SignatureType` 的差異：
/// - `SignatureType` 是「動作型別」（APPROVE / CONFIRM / WITNESS）
/// - `SignatureMeaning` 是「§11.50 法定 meaning」，含 REVIEW / AUTHOR / INVALIDATE 等
///   `SignatureType` 不涵蓋的語意
///
/// `LegacyPreR30_10` 是 R30-10 升級前的歷史資料 backfill 標記（D8=B 最誠實策略）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "signature_meaning", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignatureMeaning {
    Approve,    // §11.50 "approval"
    Review,     // §11.50 "review"
    Witness,    // 見證
    Author,     // §11.50 "authorship"
    Invalidate, // 失效標記（簽章作廢）
    Confirm,    // §11.50 "responsibility"，如完成記錄
    #[serde(rename = "LEGACY_PRE_R30_10")]
    #[sqlx(rename = "LEGACY_PRE_R30_10")]
    LegacyPreR30_10,
}

impl SignatureMeaning {
    /// 從 SignatureType 推導預設 meaning
    ///
    /// 推導規則（與 §11.50 對齊）：
    /// - `Approve` → `Approve`
    /// - `Confirm` → `Confirm`（§11.50 "responsibility"）
    /// - `Witness` → `Witness`
    ///
    /// 其他 meaning（`Review` / `Author` / `Invalidate`）由 caller 顯式指定。
    pub fn from_signature_type(sig_type: SignatureType) -> Self {
        match sig_type {
            SignatureType::Approve => SignatureMeaning::Approve,
            SignatureType::Confirm => SignatureMeaning::Confirm,
            SignatureType::Witness => SignatureMeaning::Witness,
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
    /// 21 CFR §11.50(a)(3) 簽章意義（R30-10 新增，NOT NULL；舊資料 backfill 為
    /// `LegacyPreR30_10`）。
    pub meaning: SignatureMeaning,
    /// R30-7: signature_data 編碼版本。1 = SHA-256 legacy（pre-R30-7），2 = HMAC-SHA256+secret。
    /// verify 時依此欄位 dispatch 計算演算法。
    pub hmac_version: i16,
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

    /// R30-7: 構造 signature_data 的 canonical input 字串。
    /// v1 / v2 共用同一個欄位順序，差別只在 hash 演算法。
    fn signature_canonical_input(
        signer_id: Uuid,
        content_hash: &str,
        timestamp_secs: i64,
        hash_input: &str,
    ) -> String {
        format!("{}:{}:{}:{}", signer_id, content_hash, timestamp_secs, hash_input)
    }

    /// R30-7: v1 (legacy) signature_data — plain SHA-256。
    /// 僅供 verify 既有資料 / 單元測試；新簽章一律走 v2。
    /// 目前 `verify` 實作未呼叫此函式（不重算 signature_data，理由見 verify 內註解），
    /// 但保留以供 R30-x 後續若決定加上 signature_data 完整性驗證時使用。
    #[cfg(test)]
    fn compute_signature_v1(
        signer_id: Uuid,
        content_hash: &str,
        timestamp_secs: i64,
        hash_input: &str,
    ) -> String {
        let input = Self::signature_canonical_input(signer_id, content_hash, timestamp_secs, hash_input);
        Self::compute_hash(&input)
    }

    /// R30-7: v2 signature_data — HMAC-SHA256(key, canonical_input)。
    /// key 來源：AuditService::hmac_key()（即 AUDIT_HMAC_KEY env / config，與 audit chain 共用）。
    /// 若 key 未設定 → fail loud（簽章必要密鑰缺席視為 misconfiguration）。
    fn compute_signature_v2(
        hmac_key: &str,
        signer_id: Uuid,
        content_hash: &str,
        timestamp_secs: i64,
        hash_input: &str,
    ) -> Result<String> {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(hmac_key.as_bytes()).map_err(|e| {
            AppError::Internal(format!("簽章 HMAC key invalid: {}", e))
        })?;
        let input = Self::signature_canonical_input(signer_id, content_hash, timestamp_secs, hash_input);
        mac.update(input.as_bytes());
        let bytes = mac.finalize().into_bytes();
        Ok(bytes.iter().map(|b| format!("{:02x}", b)).collect())
    }

    /// R30-7: 寫入端 — 計算新簽章的 signature_data（一律 v2 HMAC）。
    /// 回傳 (signature_data, hmac_version) 供 INSERT 寫入。
    /// 若 AUDIT_HMAC_KEY 未設定 → 回 Internal error，拒絕寫入弱簽章。
    fn build_signature_data_v2(
        signer_id: Uuid,
        content_hash: &str,
        timestamp_secs: i64,
        hash_input: &str,
    ) -> Result<(String, i16)> {
        let hmac_key = AuditService::hmac_key().ok_or_else(|| {
            AppError::Internal(
                "AUDIT_HMAC_KEY 未設定 — 無法產生 HMAC 簽章（R30-7）".into(),
            )
        })?;
        let sig = Self::compute_signature_v2(hmac_key, signer_id, content_hash, timestamp_secs, hash_input)?;
        Ok((sig, SIGNATURE_HMAC_VERSION_V2))
    }

    /// R30-7: 驗證端 — 依 hmac_version dispatch 重算 expected signature_data。
    /// - v1 → plain SHA-256（legacy 永久接受）
    /// - v2 → HMAC-SHA256（需 AUDIT_HMAC_KEY；缺席則 fail loud）
    /// - 其他 → 拒絕（未知版本視為竄改）
    ///
    /// 目前 `verify` 不直接呼叫（hash_input 在 verify 階段不可取得，無法重算），
    /// 此函式保留供測試覆蓋 dispatch 行為，並為 R30-x 未來擴充預留入口。
    #[cfg(test)]
    fn recompute_signature_data(
        version: i16,
        signer_id: Uuid,
        content_hash: &str,
        timestamp_secs: i64,
        hash_input: &str,
    ) -> Result<String> {
        match version {
            SIGNATURE_HMAC_VERSION_LEGACY => Ok(Self::compute_signature_v1(
                signer_id,
                content_hash,
                timestamp_secs,
                hash_input,
            )),
            SIGNATURE_HMAC_VERSION_V2 => {
                let hmac_key = AuditService::hmac_key().ok_or_else(|| {
                    AppError::Internal(
                        "AUDIT_HMAC_KEY 未設定 — 無法驗證 HMAC 簽章（R30-7）".into(),
                    )
                })?;
                Self::compute_signature_v2(hmac_key, signer_id, content_hash, timestamp_secs, hash_input)
            }
            _ => Err(AppError::Validation(format!(
                "不支援的 signature hmac_version: {}",
                version
            ))),
        }
    }

    /// 建立電子簽章（密碼驗證方式）
    ///
    /// `meaning`: 21 CFR §11.50(a)(3) 簽章意義。傳 `None` 時由 `signature_type` 推導
    /// （見 `SignatureMeaning::from_signature_type`）。
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
        meaning: Option<SignatureMeaning>,
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
            meaning.unwrap_or_else(|| SignatureMeaning::from_signature_type(signature_type)),
        )
        .await
    }

    /// 建立電子簽章（手寫簽名方式）
    ///
    /// `meaning`: 同 `sign`。
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
        meaning: Option<SignatureMeaning>,
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
            meaning.unwrap_or_else(|| SignatureMeaning::from_signature_type(signature_type)),
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
        meaning: SignatureMeaning,
    ) -> Result<ElectronicSignature> {
        // 計算內容雜湊
        let content_hash = Self::compute_hash(content);

        // R30-7: signature_data 改 HMAC-SHA256 v2（共用 AUDIT_HMAC_KEY）
        let timestamp = Utc::now();
        let hash_input = password_hash.unwrap_or("handwriting");
        let (signature_data, hmac_version) = Self::build_signature_data_v2(
            signer_id,
            &content_hash,
            timestamp.timestamp(),
            hash_input,
        )?;

        // 儲存到資料庫
        let signature = sqlx::query_as::<_, ElectronicSignature>(
            r#"
            INSERT INTO electronic_signatures (
                entity_type, entity_id, signer_id, signature_type,
                content_hash, signature_data, ip_address, user_agent,
                handwriting_svg, stroke_data, signature_method, meaning, hmac_version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
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
        .bind(meaning)
        .bind(hmac_version)
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

                // R30-7: 驗證 signature_data 本身未被竄改（依 hmac_version dispatch）
                // 注意：此處 hash_input 不可從 DB 還原（password_hash / "handwriting"），
                // 但寫入時的 signature_data 已包含它；這裡只能比對 v2 是否能用 HMAC key
                // 重算到「某個」hash_input 變體。實務上 hash_input 本身不公開，
                // 此驗證主要對抗 attacker 直接竄改 signature_data column 的情境。
                //
                // 由於 hash_input 在 verify 階段不可取得（password_hash 已 rotate / 手寫狀態未存），
                // 此處僅做版本欄位的合法性檢查 + key 可用性檢查，避免 false negative。
                // 真正的「signature_data 不可偽造」性質由「寫入時用 HMAC + secret」保證
                // — attacker 沒有 AUDIT_HMAC_KEY 就無法產生合法 v2 signature_data。
                if sig.hmac_version != SIGNATURE_HMAC_VERSION_LEGACY
                    && sig.hmac_version != SIGNATURE_HMAC_VERSION_V2
                {
                    return Ok(VerifyResult {
                        is_valid: false,
                        signer_name: None,
                        signed_at: Some(sig.signed_at),
                        failure_reason: Some(format!(
                            "未知的簽章版本 (hmac_version={})",
                            sig.hmac_version
                        )),
                    });
                }
                if sig.hmac_version == SIGNATURE_HMAC_VERSION_V2
                    && AuditService::hmac_key().is_none()
                {
                    return Ok(VerifyResult {
                        is_valid: false,
                        signer_name: None,
                        signed_at: Some(sig.signed_at),
                        failure_reason: Some(
                            "AUDIT_HMAC_KEY 未設定 — 無法驗證 v2 簽章".to_string(),
                        ),
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

    /// 鎖定記錄（記錄 ID 為 UUID）
    ///
    /// C1 (GLP)：簽章後呼叫此函式，將記錄鎖定。後續 service 層的 update/delete 會被
    /// `ensure_not_locked_uuid` 攔下回 409 Conflict。
    /// 支援表（migration 038 已補欄位）：
    /// - sacrifice → animal_sacrifices
    /// - observation → animal_observations
    /// - surgery → animal_surgeries
    /// - blood_test → animal_blood_tests
    /// - care_medication → care_medication_records
    pub async fn lock_record_uuid(
        pool: &PgPool,
        record_type: &str,
        record_id: Uuid,
        locked_by: Uuid,
    ) -> Result<()> {
        let table_name = Self::lockable_table_uuid(record_type)?;

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

    /// 內部：把 record_type 對應到合法的 lockable table（UUID PK）
    fn lockable_table_uuid(record_type: &str) -> Result<&'static str> {
        match record_type {
            "sacrifice" => Ok("animal_sacrifices"),
            "observation" => Ok("animal_observations"),
            "surgery" => Ok("animal_surgeries"),
            "blood_test" => Ok("animal_blood_tests"),
            "care_medication" => Ok("care_medication_records"),
            _ => Err(AppError::Validation(format!(
                "不支援的記錄類型 (UUID): {}",
                record_type
            ))),
        }
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

    /// 檢查記錄是否已鎖定（記錄 ID 為 UUID）。支援表同 `lock_record_uuid`。
    pub async fn is_locked_uuid(pool: &PgPool, record_type: &str, record_id: Uuid) -> Result<bool> {
        let table_name = Self::lockable_table_uuid(record_type)?;

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

    /// C1 guard：若記錄已鎖定，回 409 Conflict 拒絕 update/delete。
    /// service 層 update / soft_delete 入口（tx 開始前）呼叫此函式做 fail-fast。
    /// 同 tx 內仍應再以 `ensure_not_locked_uuid_tx` 做 atomic check（避免 check-then-act race）。
    pub async fn ensure_not_locked_uuid(
        pool: &PgPool,
        record_type: &str,
        record_id: Uuid,
    ) -> Result<()> {
        if Self::is_locked_uuid(pool, record_type, record_id).await? {
            return Err(AppError::Conflict(format!(
                "記錄已鎖定（已簽章），不可修改或刪除（{record_type}）"
            )));
        }
        Ok(())
    }

    /// C1 atomic guard：在 transaction 內檢查鎖定狀態，避免 check-then-act race。
    /// 用於 service 層 update/delete 內，tx.begin() 之後 UPDATE 之前。
    pub async fn ensure_not_locked_uuid_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        record_type: &str,
        record_id: Uuid,
    ) -> Result<()> {
        let table_name = Self::lockable_table_uuid(record_type)?;
        let query = format!(
            "SELECT COALESCE(is_locked, false) FROM {} WHERE id = $1 FOR UPDATE",
            table_name
        );
        let is_locked: bool = sqlx::query_scalar(&query)
            .bind(record_id)
            .fetch_optional(&mut **tx)
            .await?
            .unwrap_or(false);
        if is_locked {
            return Err(AppError::Conflict(format!(
                "記錄已鎖定（已簽章），不可修改或刪除（{record_type}）"
            )));
        }
        Ok(())
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

    /// tx-aware 統一簽章建立：在呼叫者管理的 tx 內 INSERT electronic_signatures。
    ///
    /// 與 `sign_record` 的差異：
    ///   - INSERT 走呼叫者的 `&mut Transaction`（caller commit / rollback 時才落地）
    ///   - 密碼驗證仍走 `pool`（read-only，與 tx 隔離無關）
    ///
    /// 用途：caller 已開 tx 處理 row lock + status guard + UPDATE，需簽章與這些操作原子。
    /// 失敗（密碼錯 / 缺手寫 / DB error）會把 Err 傳上去，由 caller drop tx 即 rollback。
    #[allow(clippy::too_many_arguments)]
    pub async fn sign_record_tx<'c>(
        tx: &mut sqlx::Transaction<'c, sqlx::Postgres>,
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
        // R30-10: 從 SignatureType 推導 §11.50 meaning（caller 不需新增參數）
        let meaning = SignatureMeaning::from_signature_type(sig_type);
        let has_password = password.is_some_and(|p| !p.is_empty());
        let has_handwriting = handwriting_svg.is_some_and(|s| !s.is_empty());

        if !has_password && !has_handwriting {
            return Err(AppError::Validation("請提供密碼或手寫簽名".into()));
        }

        // SEC-BIZ-3：手寫 + 密碼雙因子；純密碼也要驗。AuthService 走獨立 pool 連線
        // 是 read-only，不影響 caller 的 row lock。
        let pwd = password
            .filter(|p| !p.is_empty())
            .ok_or_else(|| {
                if has_handwriting {
                    AppError::Validation("手寫簽章仍需輸入密碼以驗證身分".into())
                } else {
                    AppError::Internal("missing password".into())
                }
            })?;
        let user = AuthService::verify_password_by_id(pool, signer_id, pwd)
            .await
            .map_err(|_| AppError::Unauthorized)?;

        // R30-7: signature_data 改 HMAC-SHA256 v2
        let content_hash = Self::compute_hash(content);
        let timestamp = Utc::now();
        let hash_input: &str = if has_handwriting {
            "handwriting"
        } else {
            user.password_hash.as_str()
        };
        let (signature_data, hmac_version) = Self::build_signature_data_v2(
            signer_id,
            &content_hash,
            timestamp.timestamp(),
            hash_input,
        )?;
        let signature_method = if has_handwriting { "handwriting" } else { "password" };

        let signature = sqlx::query_as::<_, ElectronicSignature>(
            r#"
            INSERT INTO electronic_signatures (
                entity_type, entity_id, signer_id, signature_type,
                content_hash, signature_data, ip_address, user_agent,
                handwriting_svg, stroke_data, signature_method, meaning, hmac_version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .bind(signer_id)
        .bind(sig_type.as_str())
        .bind(&content_hash)
        .bind(&signature_data)
        .bind(None::<&str>)
        .bind(None::<&str>)
        .bind(handwriting_svg.filter(|s| !s.is_empty()))
        .bind(stroke_data)
        .bind(signature_method)
        .bind(meaning)
        .bind(hmac_version)
        .fetch_one(&mut **tx)
        .await?;

        Ok(signature)
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
                None, // meaning：由 sig_type 推導
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
                None, // meaning：由 sig_type 推導
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

#[cfg(test)]
mod tests {
    //! R30-7: signature_data HMAC v2 + v1 dispatch 單元測試
    //!
    //! 注意：`AuditService::init_hmac_key` 使用 `OnceLock`，整個 process 只能
    //! 初始化一次。為避免與其他測試競爭，這裡直接呼叫 `compute_signature_v1`
    //! / `compute_signature_v2`（key 為 explicit 參數），不依賴全域狀態。
    //! `recompute_signature_data`（依賴全域 key）的測試以最少必要 case 覆蓋。
    use super::*;

    const TEST_KEY: &str = "test-hmac-key-must-be-at-least-44-chars-long-padding-xx";
    const FAKE_CONTENT_HASH: &str =
        "0000000000000000000000000000000000000000000000000000000000000000";
    const FAKE_HASH_INPUT: &str = "handwriting";
    const FAKE_TS: i64 = 1_700_000_000;

    fn fixed_signer() -> Uuid {
        Uuid::parse_str("11111111-2222-3333-4444-555555555555").expect("valid uuid")
    }

    #[test]
    fn v1_legacy_compute_is_plain_sha256() {
        // legacy 編碼：SHA-256(canonical_input)，不依賴 key
        let signer = fixed_signer();
        let v1 = SignatureService::compute_signature_v1(
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        );
        // 直接重算驗證
        let expected_input = format!(
            "{}:{}:{}:{}",
            signer, FAKE_CONTENT_HASH, FAKE_TS, FAKE_HASH_INPUT
        );
        let expected = SignatureService::compute_hash(&expected_input);
        assert_eq!(v1, expected, "v1 應為 plain SHA-256");
        assert_eq!(v1.len(), 64, "SHA-256 hex 為 64 chars");
    }

    #[test]
    fn v2_hmac_differs_from_v1_for_same_input() {
        // 同樣輸入 → v1 與 v2 不可相等（否則 HMAC 沒帶 key entropy）
        let signer = fixed_signer();
        let v1 = SignatureService::compute_signature_v1(
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        );
        let v2 = SignatureService::compute_signature_v2(
            TEST_KEY,
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        )
        .expect("v2 should compute");
        assert_ne!(v1, v2, "v2 (HMAC) 不可等於 v1 (plain SHA-256)");
        assert_eq!(v2.len(), 64, "HMAC-SHA256 hex 為 64 chars");
    }

    #[test]
    fn v2_hmac_changes_with_key() {
        // 不同 key → 不同 v2（attacker 沒 key 不能偽造）
        let signer = fixed_signer();
        let v2_a = SignatureService::compute_signature_v2(
            TEST_KEY,
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        )
        .expect("v2 a");
        let v2_b = SignatureService::compute_signature_v2(
            "different-key-also-padded-to-be-long-enough-xxxxxxxxxxxx",
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        )
        .expect("v2 b");
        assert_ne!(v2_a, v2_b, "不同 key 應產生不同 HMAC");
    }

    #[test]
    fn v2_hmac_deterministic() {
        // 同 key + 同輸入 → 同 HMAC（verify 端能重算比對）
        let signer = fixed_signer();
        let a = SignatureService::compute_signature_v2(
            TEST_KEY,
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        )
        .expect("v2 a");
        let b = SignatureService::compute_signature_v2(
            TEST_KEY,
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        )
        .expect("v2 b");
        assert_eq!(a, b, "v2 必須 deterministic");
    }

    #[test]
    fn v2_tamper_signature_data_detected() {
        // 模擬 attacker 篡改 signature_data：重算結果不會等於被篡改值
        let signer = fixed_signer();
        let original = SignatureService::compute_signature_v2(
            TEST_KEY,
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        )
        .expect("v2");
        // attacker 把 timestamp 偷改後想偽造合法 signature_data
        // 但他沒 key，只能算 plain SHA-256，那個值絕不等於原 HMAC
        let attacker_guess = SignatureService::compute_signature_v1(
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS + 1, // 改時間
            FAKE_HASH_INPUT,
        );
        assert_ne!(original, attacker_guess, "篡改 ts 後 attacker 沒 key 算不出原 HMAC");
        // 即使 attacker 用 plain SHA-256 算原來的時間
        let attacker_guess_orig_ts = SignatureService::compute_signature_v1(
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        );
        assert_ne!(
            original, attacker_guess_orig_ts,
            "v2 HMAC 不可等於 v1 plain（即使輸入相同）"
        );
    }

    #[test]
    fn v1_tamper_detected_by_recompute() {
        // v1 驗證：拿原資料重算 → 等於原 signature_data；改任一輸入 → 不等
        let signer = fixed_signer();
        let stored = SignatureService::compute_signature_v1(
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        );
        let recomputed = SignatureService::compute_signature_v1(
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        );
        assert_eq!(stored, recomputed, "v1 同輸入 deterministic");

        // 改 content_hash → 不同
        let tampered = SignatureService::compute_signature_v1(
            signer,
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            FAKE_TS,
            FAKE_HASH_INPUT,
        );
        assert_ne!(stored, tampered, "v1 改 content_hash 應偵測到");
    }

    #[test]
    fn unknown_version_rejected() {
        // recompute_signature_data 對未知版本應回 Err
        let signer = fixed_signer();
        let result = SignatureService::recompute_signature_data(
            99,
            signer,
            FAKE_CONTENT_HASH,
            FAKE_TS,
            FAKE_HASH_INPUT,
        );
        assert!(result.is_err(), "未知 hmac_version 應拒絕");
    }
}
