// 擴展的審計 Service

use std::sync::OnceLock;

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, ActivityLogQuery, AuditAction, AuditDashboardStats, AuditLog,
        AuditLogQuery, AuditLogWithActor, LoginEventQuery, LoginEventWithUser, PaginatedResponse,
        SecurityAlert, SecurityAlertQuery, SessionQuery, SessionWithUser, UserActivityLog,
    },
    Result,
};

// ============================================
// Service-driven audit 重構新型別
// ============================================

/// audit log 的參數封裝（取代原本 11 個位置參數）。
///
/// # Example
/// ```ignore
/// AuditService::log_activity_tx(&mut tx, &actor, ActivityLogEntry {
///     event_category: "ANIMAL",
///     event_type: "UPDATE",
///     entity: Some(AuditEntity::new("animal", animal.id, &animal.ear_tag)),
///     data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
///     request_context: None,
/// }).await?;
/// ```
pub struct ActivityLogEntry<'a> {
    /// 事件大類：ANIMAL / AUP / HR / ERP / SECURITY 等
    pub event_category: &'a str,
    /// 事件類型：CREATE / UPDATE / DELETE / SUBMIT / APPROVE 等
    pub event_type: &'a str,
    /// 變更對象；Some 時 data_diff 通常也會 Some
    pub entity: Option<AuditEntity<'a>>,
    /// before/after diff；用 DataDiff::compute 產生
    pub data_diff: Option<DataDiff>,
    /// HTTP 請求脈絡（IP / UA）；scheduler / bin 觸發時為 None
    pub request_context: Option<RequestContext<'a>>,
}

/// 變更對象的描述（供 user_activity_logs 的 entity_* 欄位）
pub struct AuditEntity<'a> {
    pub entity_type: &'a str,
    pub entity_id: Uuid,
    pub entity_display_name: &'a str,
}

impl<'a> AuditEntity<'a> {
    pub fn new(entity_type: &'a str, id: Uuid, display: &'a str) -> Self {
        Self {
            entity_type,
            entity_id: id,
            entity_display_name: display,
        }
    }
}

/// HTTP 請求脈絡
pub struct RequestContext<'a> {
    pub ip_address: Option<&'a str>,
    pub user_agent: Option<&'a str>,
}

/// HMAC 雜湊鏈的結構化輸入（length-prefixed canonical encoding）。
///
/// 取代 `compute_and_store_hmac_tx` 的 10 個散落參數；同時讓編碼免於
/// 字串串接碰撞（見 `canonical_bytes` 說明）。
///
/// `pub(crate)` 是因 [`AuditService::compute_hmac_for_fields`] 接此型別
/// 作為單一參數，呼叫端（`audit_chain_verify` / `verify_chain_range`）
/// 需在 crate 內直接構造。
pub(crate) struct HmacInput<'a> {
    pub(crate) event_category: &'a str,
    pub(crate) event_type: &'a str,
    pub(crate) actor_user_id: Uuid,
    pub(crate) before_data: &'a Option<serde_json::Value>,
    pub(crate) after_data: &'a Option<serde_json::Value>,
    pub(crate) impersonated_by: Option<Uuid>,
    pub(crate) changed_fields: &'a [String],
}

// ============================================
// R26-6: HMAC 編碼版本
// ============================================

/// Legacy string-concat 編碼（pre-R26-6）。
/// 由 R26 前的 `AuditService::log_activity` / `compute_and_store_hmac` 舊版
/// 寫入路徑使用；此二函式已於 R26-4 移除，verifier 為相容既有 legacy row
/// 保留此編碼實作（見 `canonical_bytes` 的 v1 fallback 路徑）。
pub(crate) const HMAC_VERSION_LEGACY: i16 = 1;

/// Length-prefix canonical 編碼（R26 SDD 新版）。
/// 由 [`AuditService::log_activity_tx`] / [`AuditService::compute_and_store_hmac_tx`] 使用。
pub(crate) const HMAC_VERSION_CANONICAL: i16 = 2;

impl HmacInput<'_> {
    /// v2 (R26 canonical) 編碼：將所有欄位以 length-prefix 寫入 buffer。
    ///
    /// 每個欄位：`8-byte BE length` + `UTF-8 bytes`
    /// changed_fields 以 `array length (u64 BE)` 開頭，再逐欄位 length-prefix。
    ///
    /// prev_hash 由呼叫端（從 DB 讀出）提供；這設計讓 struct 只承載「內容」，
    /// chain 連結資料由 DB 提供。
    pub(crate) fn canonical_bytes(&self, prev_hash: Option<&str>) -> Vec<u8> {
        let mut buf = Vec::with_capacity(512);
        write_field(&mut buf, prev_hash.unwrap_or("").as_bytes());
        write_field(&mut buf, self.event_category.as_bytes());
        write_field(&mut buf, self.event_type.as_bytes());
        // Uuid::to_string 永遠 36 char（hyphenated），長度穩定
        write_field(&mut buf, self.actor_user_id.to_string().as_bytes());
        let before_str = self
            .before_data
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();
        write_field(&mut buf, before_str.as_bytes());
        let after_str = self
            .after_data
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();
        write_field(&mut buf, after_str.as_bytes());
        let imp = self
            .impersonated_by
            .map(|u| u.to_string())
            .unwrap_or_default();
        write_field(&mut buf, imp.as_bytes());
        buf.extend_from_slice(&(self.changed_fields.len() as u64).to_be_bytes());
        for f in self.changed_fields {
            write_field(&mut buf, f.as_bytes());
        }
        buf
    }

    /// v1 (legacy) 編碼：pre-R26-6 的字串串接方式（已隨 R26-4 移除的
    /// `compute_and_store_hmac` 舊版所用）。
    ///
    /// ⚠️ 此編碼有碰撞風險（`"ab"+"cd"` 與 `"abc"+"d"` 產生相同 byte stream），
    /// 也未包含 `impersonated_by` / `changed_fields` 兩欄位。僅供 verifier 對
    /// `hmac_version=1` 的 legacy row 重算 HMAC 使用；**新程式碼禁止使用此編碼**。
    ///
    /// 與舊 `compute_and_store_hmac` 保持 byte-for-byte 一致以避免 false positive。
    pub(crate) fn legacy_concat_message(&self, prev_hash: Option<&str>) -> String {
        let mut message = String::new();
        if let Some(ph) = prev_hash {
            message.push_str(ph);
        }
        message.push_str(self.event_category);
        message.push_str(self.event_type);
        message.push_str(&self.actor_user_id.to_string());
        if let Some(bd) = self.before_data {
            message.push_str(&bd.to_string());
        }
        if let Some(ad) = self.after_data {
            message.push_str(&ad.to_string());
        }
        message
    }
}

/// 寫入單一欄位：8-byte big-endian 長度 + bytes
fn write_field(buf: &mut Vec<u8>, bytes: &[u8]) {
    buf.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    buf.extend_from_slice(bytes);
}

/// HMAC 密鑰（由 Config 初始化一次，避免每次讀取 env var）
static AUDIT_HMAC_KEY: OnceLock<Option<String>> = OnceLock::new();

/// R26-2：HMAC chain 範圍驗證結果。
///
/// 由 [`AuditService::verify_chain_range`] 產生；每日驗證 cron 根據 broken_links 是否
/// 為空決定是否觸發 SecurityNotifier 告警。
#[derive(Debug, Clone)]
pub struct ChainVerificationReport {
    pub range_from: chrono::DateTime<chrono::Utc>,
    pub range_to: chrono::DateTime<chrono::Utc>,
    /// 範圍內的 row 總數（含已略過的 security_event）
    pub total_rows: usize,
    /// 真正跑完 HMAC 比對的 row 數（= total_rows - skipped_no_hash）
    pub verified_rows: usize,
    /// 無 integrity_hash（security_event 等不入鏈的紀錄）
    pub skipped_no_hash: usize,
    /// HMAC 不一致的 row（斷鏈偵測）
    pub broken_links: Vec<BrokenChainLink>,
}

impl ChainVerificationReport {
    pub fn is_intact(&self) -> bool {
        self.broken_links.is_empty()
    }
}

/// 單一 HMAC 驗證失敗的 row 紀錄。
#[derive(Debug, Clone)]
pub struct BrokenChainLink {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expected_hash: String,
    pub stored_hash: String,
    pub stored_previous_hash: Option<String>,
}

/// audit chain 驗證時讀取的 row（內部用，不對外公開）。
///
/// 注意 `actor_user_id: Option<Uuid>` — `user_activity_logs.actor_user_id`
/// schema 允許 NULL（匿名事件）；改用 `Uuid` 會在 fetch 時 panic。
#[derive(sqlx::FromRow)]
struct ChainRow {
    id: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    event_category: String,
    event_type: String,
    actor_user_id: Option<Uuid>,
    before_data: Option<serde_json::Value>,
    after_data: Option<serde_json::Value>,
    impersonated_by_user_id: Option<Uuid>,
    changed_fields: Option<Vec<String>>,
    integrity_hash: Option<String>,
    previous_hash: Option<String>,
    /// R26-6：HMAC 編碼版本。
    /// - `Some(1)` = legacy string-concat；`Some(2)` = length-prefix canonical。
    /// - `None`（pre-R26-6 row，尚未 backfill）— verifier 採 **try-both** 策略：
    ///   先試 canonical (v=2)，不符再 fallback legacy (v=1)。原因是 migration 037
    ///   前的 `log_activity_tx` 已使用 v2 編碼寫入但尚無 column 可標記，單純預設
    ///   v=1 會對這批 row 產生 false positive。Backfill 目的僅為消除 try-both
    ///   成本並讓 SQL 報表可直接用 `hmac_version = 1` 篩選 legacy row。
    /// - 維護警告：撰寫 backfill 腳本時**不可假設所有 NULL 都是 v=1**。
    hmac_version: Option<i16>,
}

pub struct AuditService;

impl AuditService {
    /// 從 Config 初始化 HMAC 密鑰（應在啟動時呼叫一次）
    pub fn init_hmac_key(key: Option<String>) {
        let _ = AUDIT_HMAC_KEY.set(key);
    }

    /// 記錄稽核日誌（原有）
    pub async fn log(
        pool: &PgPool,
        actor_user_id: Uuid,
        action: AuditAction,
        entity_type: &str,
        entity_id: Uuid,
        before: Option<serde_json::Value>,
        after: Option<serde_json::Value>,
    ) -> Result<AuditLog> {
        let log = sqlx::query_as::<_, AuditLog>(
            r#"
            INSERT INTO audit_logs (id, actor_user_id, action, entity_type, entity_id, before_data, after_data, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(actor_user_id)
        .bind(action.as_str())
        .bind(entity_type)
        .bind(entity_id)
        .bind(before)
        .bind(after)
        .fetch_one(pool)
        .await?;

        Ok(log)
    }

    /// 查詢稽核日誌（原有）
    pub async fn list(pool: &PgPool, query: &AuditLogQuery) -> Result<Vec<AuditLogWithActor>> {
        let logs = if let Some(ref entity_type) = query.entity_type {
            if let Some(ref action) = query.action {
                sqlx::query_as::<_, AuditLogWithActor>(
                    r#"
                    SELECT 
                        al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                        al.action, al.entity_type, al.entity_id,
                        eu.email as entity_email, eu.display_name as entity_name,
                        al.before_data, al.after_data, al.ip_address, al.user_agent, al.created_at
                    FROM audit_logs al
                    INNER JOIN users u ON al.actor_user_id = u.id
                    LEFT JOIN users eu ON al.entity_type = 'user' AND al.entity_id = eu.id
                    WHERE al.entity_type = $1 AND al.action = $2
                    ORDER BY al.created_at DESC
                    LIMIT 200
                    "#
                )
                .bind(entity_type)
                .bind(action)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, AuditLogWithActor>(
                    r#"
                    SELECT 
                        al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                        al.action, al.entity_type, al.entity_id,
                        eu.email as entity_email, eu.display_name as entity_name,
                        al.before_data, al.after_data, al.ip_address, al.user_agent, al.created_at
                    FROM audit_logs al
                    INNER JOIN users u ON al.actor_user_id = u.id
                    LEFT JOIN users eu ON al.entity_type = 'user' AND al.entity_id = eu.id
                    WHERE al.entity_type = $1
                    ORDER BY al.created_at DESC
                    LIMIT 200
                    "#
                )
                .bind(entity_type)
                .fetch_all(pool)
                .await?
            }
        } else if let Some(ref action) = query.action {
            sqlx::query_as::<_, AuditLogWithActor>(
                r#"
                SELECT 
                    al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                    al.action, al.entity_type, al.entity_id,
                    eu.email as entity_email, eu.display_name as entity_name,
                    al.before_data, al.after_data, al.ip_address, al.user_agent, al.created_at
                FROM audit_logs al
                INNER JOIN users u ON al.actor_user_id = u.id
                LEFT JOIN users eu ON al.entity_type = 'user' AND al.entity_id = eu.id
                WHERE al.action = $1
                ORDER BY al.created_at DESC
                LIMIT 200
                "#,
            )
            .bind(action)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, AuditLogWithActor>(
                r#"
                SELECT 
                    al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                    al.action, al.entity_type, al.entity_id,
                    eu.email as entity_email, eu.display_name as entity_name,
                    al.before_data, al.after_data, al.ip_address, al.user_agent, al.created_at
                FROM audit_logs al
                INNER JOIN users u ON al.actor_user_id = u.id
                LEFT JOIN users eu ON al.entity_type = 'user' AND al.entity_id = eu.id
                ORDER BY al.created_at DESC
                LIMIT 200
                "#,
            )
            .fetch_all(pool)
            .await?
        };

        Ok(logs)
    }

    /// 取得特定實體的稽核歷史
    pub async fn get_entity_history(
        pool: &PgPool,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AuditLogWithActor>> {
        let logs = sqlx::query_as::<_, AuditLogWithActor>(
            r#"
            SELECT 
                al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                al.action, al.entity_type, al.entity_id,
                eu.email as entity_email, eu.display_name as entity_name,
                al.before_data, al.after_data, al.ip_address, al.user_agent, al.created_at
            FROM audit_logs al
            INNER JOIN users u ON al.actor_user_id = u.id
            LEFT JOIN users eu ON al.entity_type = 'user' AND al.entity_id = eu.id
            WHERE al.entity_type = $1 AND al.entity_id = $2
            ORDER BY al.created_at DESC
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    // ============================================
    // Service-driven audit 重構：transaction 版本
    // ============================================

    /// Transaction 版本的 activity log。Service-driven 重構模式使用此函式，
    /// 保證 audit 與資料變更在同一 tx 內 commit 或 rollback。
    ///
    /// 取代已於 R26-4 移除的 `log_activity(pool, ...)` 舊版 API。
    pub async fn log_activity_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        entry: ActivityLogEntry<'_>,
    ) -> Result<Uuid> {
        let (before_data, after_data, changed_fields) = match entry.data_diff {
            Some(diff) => diff.into_parts(),
            None => (None, None, Vec::new()),
        };
        let (ip, ua) = match entry.request_context {
            Some(ref r) => (r.ip_address, r.user_agent),
            None => (None, None),
        };
        let (entity_type, entity_id, entity_name) = match entry.entity {
            Some(ref e) => (
                Some(e.entity_type),
                Some(e.entity_id),
                Some(e.entity_display_name),
            ),
            None => (None, None, None),
        };
        let impersonated_by = actor.impersonated_by();
        // app 層若提供 changed_fields（Vec 非空），傳給 stored proc；
        // 若空 Vec，傳 NULL 讓 stored proc 自己用 JSONB EXCEPT 算。
        let changed_fields_param: Option<&[String]> = if changed_fields.is_empty() {
            None
        } else {
            Some(&changed_fields)
        };

        // R26-4 疑慮 1+2: advisory lock 序列化 audit 寫入，保證 HMAC chain
        // 不會在並發下跳 row 或指向 rollback 的死連結。
        // Lock 綁在 tx 上，tx commit/rollback 時自動釋放。
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind("audit_log_chain")
            .execute(&mut **tx)
            .await?;

        // R26-3: 一次 INSERT 完整寫入（含 impersonated_by + changed_fields），
        // 不再做事後 UPDATE — 這樣 HMAC 計算能涵蓋所有欄位，tamper-resistance 完整。
        let result: (Uuid,) = sqlx::query_as(
            "SELECT log_activity($1, $2, $3, $4, $5, $6, $7, $8, $9::inet, $10, $11, $12)",
        )
        .bind(actor.actor_user_id())
        .bind(entry.event_category)
        .bind(entry.event_type)
        .bind(entity_type)
        .bind(entity_id)
        .bind(entity_name)
        .bind(before_data.clone())
        .bind(after_data.clone())
        .bind(ip)
        .bind(ua)
        .bind(impersonated_by)
        .bind(changed_fields_param)
        .fetch_one(&mut **tx)
        .await?;

        let log_id = result.0;

        // SEC-34: HMAC 雜湊鏈（在同一 tx 內；rollback 時 HMAC 也會退）
        if let Some(Some(hmac_key)) = AUDIT_HMAC_KEY.get() {
            // actor_user_id 若為 None（Anonymous），使用 SYSTEM UUID 參與雜湊，
            // 避免鏈斷（匿名事件仍計入 chain，但 DB 欄位存 NULL）
            let hash_actor = actor
                .actor_user_id()
                .unwrap_or(crate::middleware::SYSTEM_USER_ID);
            let hmac_input = HmacInput {
                event_category: entry.event_category,
                event_type: entry.event_type,
                actor_user_id: hash_actor,
                before_data: &before_data,
                after_data: &after_data,
                impersonated_by,
                changed_fields: &changed_fields,
            };
            Self::compute_and_store_hmac_tx(tx, log_id, hmac_key, hmac_input).await?;
        }

        Ok(log_id)
    }

    /// 單次 audit log 便利函式：自行開一個 tx 寫入後 commit。
    ///
    /// 適用情境：
    /// 1. **外部服務後事件**（PDF export / import 完成後記錄）— 操作本身不屬於 tx 範疇，
    ///    audit 只是「事件發生」的紀錄。
    /// 2. **`tokio::spawn` 中 audit**（Step 6）— 背景任務無法共用 request tx。
    /// 3. **純 audit 事件**（無 entity 變更）— 不需與資料變更同 tx。
    ///
    /// 相對於舊版 `log_activity(pool, ...)` 的優點：
    /// - 使用新版 `ActivityLogEntry` struct（支援 DataDiff、impersonated_by、
    ///   changed_fields）
    /// - 透過 `log_activity_tx` 統一寫入路徑，HMAC chain 保持一致
    pub async fn log_activity_oneshot(
        pool: &PgPool,
        actor: &ActorContext,
        entry: ActivityLogEntry<'_>,
    ) -> Result<Uuid> {
        let mut tx = pool.begin().await?;
        let log_id = Self::log_activity_tx(&mut tx, actor, entry).await?;
        tx.commit().await?;
        Ok(log_id)
    }

    /// Transaction 版本的 HMAC 計算。
    ///
    /// HMAC 輸入編碼（length-prefixed canonical form）：
    ///   每個欄位以「8-byte big-endian 長度 + UTF-8 bytes」寫入 HMAC buffer。
    ///   欄位順序固定：prev_hash → event_category → event_type → actor_user_id
    ///                → before_data → after_data → impersonated_by → changed_fields
    ///   changed_fields 先寫 array 長度（u64 BE），再逐欄位 length-prefix。
    ///
    /// 為何用 length-prefix 而非字串串接：
    ///   `"ab" + "cd"` 和 `"abc" + "d"` 的 byte stream 相同，字串串接會碰撞。
    ///   加 length prefix 後 `(2)"ab"(2)"cd"` vs `(3)"abc"(1)"d"` 不同。
    ///
    /// 納入 impersonated_by 與 changed_fields 的目的：
    ///   若 DB 被竄改（例如清空這兩欄），HMAC 驗證會失敗。
    /// 對給定 [`HmacInput`] 計算 HMAC（不接觸 DB，純運算）。
    ///
    /// 供 [`verify_chain_range`](Self::verify_chain_range) 使用：針對每筆 audit row
    /// 重算預期 HMAC，和儲存值比對即可判定 chain 完整性。
    ///
    /// R26-6：依 HMAC 編碼版本分流計算 expected hash。
    ///
    /// - `HMAC_VERSION_LEGACY` (1) → 字串串接（`legacy_concat_message`）
    /// - `HMAC_VERSION_CANONICAL` (2) → length-prefix canonical（`canonical_bytes`）
    ///
    /// 未知版本（未來擴充）fallback canonical — 這是刻意選擇：新版 writer 應
    /// 先寫 migration 再 deploy code，若 verifier 在版本過渡期遇未知值會對
    /// canonical 版本做比對，至少能偵測 canonical row 的竄改。
    ///
    /// 返回 `Result` 以符合 coding rules「執行期禁用 `expect()`」（見 CLAUDE.md）。
    /// 實務上 `HmacSha256::new_from_slice` 僅在 key 長度 0 時失敗，
    /// `config.rs` 已強制 AUDIT_HMAC_KEY 長度 ≥ 44 chars，此 error 路徑不應觸發；
    /// 但仍保留 fallible 簽名以避免未來 key source 改動時引入 panic。
    pub(crate) fn compute_hmac_for_fields_versioned(
        hmac_key: &str,
        prev_hash: Option<&str>,
        input: HmacInput<'_>,
        version: i16,
    ) -> Result<String> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(hmac_key.as_bytes()).map_err(|e| {
            crate::error::AppError::Internal(format!("HMAC key invalid: {}", e))
        })?;

        if version == HMAC_VERSION_LEGACY {
            mac.update(input.legacy_concat_message(prev_hash).as_bytes());
        } else {
            mac.update(&input.canonical_bytes(prev_hash));
        }

        let hash_bytes = mac.finalize().into_bytes();
        Ok(hash_bytes.iter().map(|b| format!("{:02x}", b)).collect())
    }

    /// 取得目前 HMAC 密鑰（未初始化或未設定則 None）。
    ///
    /// 主要給 `audit_chain_verify` job 用：key 缺席時應 fail loud（HMAC chain
    /// 驗證無意義）而非靜默略過。
    pub fn hmac_key() -> Option<&'static str> {
        AUDIT_HMAC_KEY.get().and_then(|k| k.as_deref())
    }

    /// 驗證指定時間範圍內 `user_activity_logs` HMAC 鏈的完整性（R26-2）。
    ///
    /// 流程（拆為 3 個 helper 以滿足 50-line function 上限）：
    /// 1. [`load_chain_rows`](Self::load_chain_rows)：取範圍內所有 row（ORDER BY (created_at, id) ASC）
    /// 2. [`load_initial_prev_hash`](Self::load_initial_prev_hash)：取範圍前一筆 row 的 integrity_hash
    /// 3. [`verify_chain_rows`](Self::verify_chain_rows)：逐行重算 HMAC + 比對
    ///
    /// **prev_hash 推進規則**（與 [`compute_and_store_hmac_tx`](Self::compute_and_store_hmac_tx) 寫入端一致）：
    /// - 寫入端取「立即前一筆 row」的 integrity_hash，**不**過濾 NULL
    /// - 因此 verifier 也不過濾 NULL；security_event 類 row（NULL hash）會把 prev_hash 重置為 None
    /// - 不一致會造成 false positive：security_event 後第一筆業務 row 預期 hash 對不上
    ///
    /// **效能考量**：把範圍內所有 row 拉進記憶體。對每日驗證（~1000-10000 rows）
    /// 約耗時 < 1 秒；若要驗證大範圍（>100k rows）需改為 streaming 版本。
    pub async fn verify_chain_range(
        pool: &PgPool,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<ChainVerificationReport> {
        let hmac_key = Self::hmac_key().ok_or_else(|| {
            crate::error::AppError::Internal(
                "AUDIT_HMAC_KEY 未設定 — 無法驗證 chain 完整性".into(),
            )
        })?;

        let rows = Self::load_chain_rows(pool, from, to).await?;

        if rows.is_empty() {
            return Ok(ChainVerificationReport {
                range_from: from,
                range_to: to,
                total_rows: 0,
                verified_rows: 0,
                skipped_no_hash: 0,
                broken_links: Vec::new(),
            });
        }

        let first = &rows[0];
        let prev_hash = Self::load_initial_prev_hash(pool, first.created_at, first.id).await?;

        let (verified_rows, skipped_no_hash, broken_links) =
            Self::verify_chain_rows(hmac_key, prev_hash, &rows)?;

        Ok(ChainVerificationReport {
            range_from: from,
            range_to: to,
            total_rows: rows.len(),
            verified_rows,
            skipped_no_hash,
            broken_links,
        })
    }

    /// 載入指定時間範圍的 audit row（含 partition_date filter 啟用 PostgreSQL partition pruning）。
    ///
    /// `actor_user_id` 為 `Option<Uuid>` — `user_activity_logs.actor_user_id`
    /// schema 允許 NULL（匿名事件、CSP report 等）；若 decode 為 `Uuid` 會在
    /// `fetch_all` panic（CodeRabbit PR #158 🔴 Critical）。
    async fn load_chain_rows(
        pool: &PgPool,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<ChainRow>> {
        let rows: Vec<ChainRow> = sqlx::query_as::<_, ChainRow>(
            r#"
            SELECT id, created_at, event_category, event_type, actor_user_id,
                   before_data, after_data, impersonated_by_user_id,
                   changed_fields, integrity_hash, previous_hash, hmac_version
            FROM user_activity_logs
            WHERE created_at >= $1 AND created_at < $2
              AND partition_date >= $1::date AND partition_date <= $2::date
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    /// 取範圍前一筆 row 的 integrity_hash 作為初始 prev_hash。
    ///
    /// **不過濾 NULL** — 必須與寫入端 `compute_and_store_hmac_tx` 取 prev_hash
    /// 的規則完全一致（CodeRabbit PR #158 🔴 Critical）。
    async fn load_initial_prev_hash(
        pool: &PgPool,
        first_created_at: chrono::DateTime<chrono::Utc>,
        first_id: Uuid,
    ) -> Result<Option<String>> {
        let prev: Option<Option<String>> = sqlx::query_scalar(
            r#"
            SELECT integrity_hash FROM user_activity_logs
            WHERE (created_at, id) < ($1, $2)
              AND partition_date <= $1::date
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(first_created_at)
        .bind(first_id)
        .fetch_optional(pool)
        .await?;
        // fetch_optional 回 Option<T>；T = Option<String>（NULL hash row）
        // 攤平兩層 Option：無 row → None；有 row 但 hash NULL → None；有 row + hash → Some(s)
        Ok(prev.flatten())
    }

    /// 逐行 verify chain；返回 (verified_count, skipped_count, broken_links)。
    ///
    /// 規則：
    /// - SECURITY 類事件（`event_category == "SECURITY"`）：寫入時不走 HMAC chain，
    ///   `integrity_hash` 為 NULL；verifier 略過（skipped_no_hash++）但**仍把
    ///   prev_hash 重置為 None**（與寫入端一致）。
    /// - 非 SECURITY 但 `integrity_hash` 為 NULL：**異常**（不應發生），
    ///   記入 broken_links（Gemini 🟠 high：避免攻擊者 nullify hash 規避偵測）；
    ///   prev_hash 同樣重置為 None。
    /// - 有 `integrity_hash`：重算 HMAC 比對；不一致記入 broken_links；
    ///   prev_hash 推進為 stored_hash（即使 broken 也推進，方便定位連鎖破壞）。
    fn verify_chain_rows(
        hmac_key: &str,
        initial_prev_hash: Option<String>,
        rows: &[ChainRow],
    ) -> Result<(usize, usize, Vec<BrokenChainLink>)> {
        const SECURITY_CATEGORY: &str = "SECURITY";

        let mut prev_hash = initial_prev_hash;
        let mut broken_links: Vec<BrokenChainLink> = Vec::new();
        let mut verified_rows = 0_usize;
        let mut skipped_no_hash = 0_usize;

        for row in rows {
            match row.integrity_hash.as_deref() {
                None if row.event_category == SECURITY_CATEGORY => {
                    // SECURITY 事件不入鏈：合法 NULL；prev_hash 重置與寫入端對齊
                    skipped_no_hash += 1;
                    prev_hash = None;
                }
                None => {
                    // 非 SECURITY 卻 NULL → 視為斷鏈（防止 nullify-bypass attack）
                    broken_links.push(BrokenChainLink {
                        id: row.id,
                        created_at: row.created_at,
                        expected_hash: "<missing integrity_hash for non-SECURITY row>".into(),
                        stored_hash: String::new(),
                        stored_previous_hash: row.previous_hash.clone(),
                    });
                    prev_hash = None;
                }
                Some(stored_hash) => {
                    let changed_fields = row.changed_fields.as_deref().unwrap_or(&[]);
                    let build_input = || HmacInput {
                        event_category: &row.event_category,
                        event_type: &row.event_type,
                        // 匿名 actor 寫入時用 SYSTEM_USER_ID（與 ActorContext::Anonymous
                        // 寫入端 fallback 一致）
                        actor_user_id: row
                            .actor_user_id
                            .unwrap_or(crate::middleware::SYSTEM_USER_ID),
                        before_data: &row.before_data,
                        after_data: &row.after_data,
                        impersonated_by: row.impersonated_by_user_id,
                        changed_fields,
                    };
                    // R26-6：hmac_version 分流。
                    // - Some(version) → 依版本編碼比對一次
                    // - None（pre-R26-6 row，尚未 backfill）→ try-both 策略：先試
                    //   canonical（因 migration 037 前 log_activity_tx 已使用 v2 編碼
                    //   但無 column 可標記），再試 legacy；避免任一方向的 false positive。
                    let expected = match row.hmac_version {
                        Some(v) => Self::compute_hmac_for_fields_versioned(
                            hmac_key,
                            prev_hash.as_deref(),
                            build_input(),
                            v,
                        )?,
                        None => {
                            let v2 = Self::compute_hmac_for_fields_versioned(
                                hmac_key,
                                prev_hash.as_deref(),
                                build_input(),
                                HMAC_VERSION_CANONICAL,
                            )?;
                            if v2 == stored_hash {
                                v2
                            } else {
                                Self::compute_hmac_for_fields_versioned(
                                    hmac_key,
                                    prev_hash.as_deref(),
                                    build_input(),
                                    HMAC_VERSION_LEGACY,
                                )?
                            }
                        }
                    };

                    if expected != stored_hash {
                        broken_links.push(BrokenChainLink {
                            id: row.id,
                            created_at: row.created_at,
                            expected_hash: expected,
                            stored_hash: stored_hash.to_string(),
                            stored_previous_hash: row.previous_hash.clone(),
                        });
                    }

                    verified_rows += 1;
                    prev_hash = Some(stored_hash.to_string());
                }
            }
        }

        Ok((verified_rows, skipped_no_hash, broken_links))
    }

    /// 寫入 `security_alerts` 表（給 audit_chain_verify cron + 其他需要產生
    /// 安全告警的 service 使用，避免 SQL 散落）。
    ///
    /// 移自 `audit_chain_verify.rs` 的 inline INSERT（CodeRabbit PR #158 🟠 Major）。
    pub async fn create_security_alert(
        pool: &PgPool,
        alert_type: &str,
        severity: &str,
        title: &str,
        description: &str,
        context_data: &serde_json::Value,
    ) -> Result<Uuid> {
        let id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO security_alerts (alert_type, severity, title, description, context_data, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING id
            "#,
        )
        .bind(alert_type)
        .bind(severity)
        .bind(title)
        .bind(description)
        .bind(context_data)
        .fetch_one(pool)
        .await?;
        Ok(id)
    }

    async fn compute_and_store_hmac_tx(
        tx: &mut Transaction<'_, Postgres>,
        log_id: Uuid,
        hmac_key: &str,
        input: HmacInput<'_>,
    ) -> Result<()> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        // R26-4 疑慮 4：(created_at, id) tuple 比較 + 雙欄 DESC 排序
        // 讓同微秒寫入時仍能得到穩定的 prev_hash。
        //
        // Bug fix (R26-19): 使用 `Option<Option<String>>` + `.flatten()` 與
        // `load_initial_prev_hash` 一致處理 NULL integrity_hash（例如 SECURITY
        // 類 row 或 legacy row）。原本 `Option<String>` 直接解碼，當前一筆
        // row 的 integrity_hash IS NULL 時會觸發 sqlx `UnexpectedNullError`，
        // 導致 tx rollback、呼叫端（如 create_user）收到 500。
        //
        // 語意一致性：verifier `load_initial_prev_hash` 同樣 flatten，
        // 有 row 但 hash NULL → None，與 write 端重算 hash 結果一致。
        let raw: Option<Option<String>> = sqlx::query_scalar(
            r#"
            SELECT integrity_hash FROM user_activity_logs
            WHERE (created_at, id) < (
                SELECT created_at, id FROM user_activity_logs WHERE id = $1
            )
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(log_id)
        .fetch_optional(&mut **tx)
        .await?;
        // 攤平兩層 Option：無 row → None；有 row 但 hash NULL → None；有 row + hash → Some(s)
        let prev_hash: Option<String> = raw.flatten();

        let message = input.canonical_bytes(prev_hash.as_deref());

        let mut mac = HmacSha256::new_from_slice(hmac_key.as_bytes())
            .map_err(|e| crate::error::AppError::Internal(format!("HMAC key error: {}", e)))?;
        mac.update(&message);
        let hash_bytes = mac.finalize().into_bytes();
        let hash_result: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        // R26-6：寫入 hmac_version=2 標示為 length-prefix canonical 編碼。
        sqlx::query(
            "UPDATE user_activity_logs SET integrity_hash = $1, previous_hash = $2, hmac_version = $3 WHERE id = $4",
        )
        .bind(&hash_result)
        .bind(prev_hash.as_deref())
        .bind(HMAC_VERSION_CANONICAL)
        .bind(log_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// R22: 記錄安全事件（rate limit / 403 / lockout 等）
    ///
    /// 直接 INSERT user_activity_logs（繞過 log_activity stored proc），
    /// 因為安全事件可能無 actor_user_id（匿名），且不需 HMAC chain。
    /// Gemini #5: 支援可選 actor_user_id（403 事件記錄使用者，方便 IDOR 查詢用索引欄位）
    #[allow(clippy::too_many_arguments)]
    pub async fn log_security_event(
        pool: &PgPool,
        event_type: &str,
        actor_user_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        request_path: Option<&str>,
        request_method: Option<&str>,
        context: serde_json::Value,
    ) -> Result<Uuid> {
        Self::log_security_event_with_executor(
            pool,
            event_type,
            actor_user_id,
            ip_address,
            user_agent,
            request_path,
            request_method,
            context,
        )
        .await
    }

    /// H8：log_security_event 的 tx 版（接受 &mut Transaction），讓帳號鎖定等
    /// 安全事件可與業務 SQL 在同一原子事務內寫入，避免 tokio::spawn 的火忘式
    /// 寫入在進程崩潰時遺失稽核紀錄。
    #[allow(clippy::too_many_arguments)]
    pub async fn log_security_event_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        event_type: &str,
        actor_user_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        request_path: Option<&str>,
        request_method: Option<&str>,
        context: serde_json::Value,
    ) -> Result<Uuid> {
        Self::log_security_event_with_executor(
            &mut **tx,
            event_type,
            actor_user_id,
            ip_address,
            user_agent,
            request_path,
            request_method,
            context,
        )
        .await
    }

    /// H8：實際 INSERT — 接受任意 sqlx Executor，pool 與 tx 共用實作（DRY）。
    #[allow(clippy::too_many_arguments)]
    async fn log_security_event_with_executor<'e, E>(
        executor: E,
        event_type: &str,
        actor_user_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        request_path: Option<&str>,
        request_method: Option<&str>,
        context: serde_json::Value,
    ) -> Result<Uuid>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let id = Uuid::new_v4();
        // Gemini #3: partition_date 統一使用台灣時間（與其他 activity logs 一致）
        let partition_date = crate::time::today_taiwan_naive();

        sqlx::query(
            r#"
            INSERT INTO user_activity_logs (
                id, partition_date, actor_user_id, event_category, event_type, event_severity,
                ip_address, user_agent, request_path, request_method,
                after_data, is_suspicious, suspicious_reason, created_at
            ) VALUES (
                $1, $2, $3, 'SECURITY', $4, 'warning',
                $5::inet, $6, $7, $8,
                $9, true, $10, NOW()
            )
            "#,
        )
        .bind(id)
        .bind(partition_date)
        .bind(actor_user_id)
        .bind(event_type)
        .bind(ip_address)
        .bind(user_agent)
        .bind(request_path)
        .bind(request_method)
        .bind(&context)
        .bind(format!("Security event: {event_type}"))
        .execute(executor)
        .await?;

        Ok(id)
    }

    // ============================================
    // 新增的 Activity Logs 方法
    // ============================================

    /// 列出使用者活動記錄
    pub async fn list_activities(
        pool: &PgPool,
        query: &ActivityLogQuery,
    ) -> Result<PaginatedResponse<UserActivityLog>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        // 計算總數
        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM user_activity_logs
            WHERE ($1::uuid IS NULL OR actor_user_id = $1)
              AND ($2::text IS NULL OR event_category = $2)
              AND ($3::text IS NULL OR event_type = $3)
              AND ($4::text IS NULL OR entity_type = $4)
              AND ($5::uuid IS NULL OR entity_id = $5)
              AND ($6::bool IS NULL OR is_suspicious = $6)
              AND ($7::date IS NULL OR partition_date >= $7)
              AND ($8::date IS NULL OR partition_date <= $8)
            "#,
        )
        .bind(query.user_id)
        .bind(&query.event_category)
        .bind(&query.event_type)
        .bind(&query.entity_type)
        .bind(query.entity_id)
        .bind(query.is_suspicious)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, UserActivityLog>(
            r#"
            SELECT id, actor_user_id, actor_email, actor_display_name, actor_roles, session_id,
                   event_category, event_type, event_severity, entity_type, entity_id, entity_display_name,
                   before_data, after_data, changed_fields, ip_address::text as ip_address,
                   user_agent, request_path, request_method, response_status,
                   is_suspicious, suspicious_reason, created_at, partition_date
            FROM user_activity_logs
            WHERE ($1::uuid IS NULL OR actor_user_id = $1)
              AND ($2::text IS NULL OR event_category = $2)
              AND ($3::text IS NULL OR event_type = $3)
              AND ($4::text IS NULL OR entity_type = $4)
              AND ($5::uuid IS NULL OR entity_id = $5)
              AND ($6::bool IS NULL OR is_suspicious = $6)
              AND ($7::date IS NULL OR partition_date >= $7)
              AND ($8::date IS NULL OR partition_date <= $8)
            ORDER BY created_at DESC
            LIMIT $9 OFFSET $10
            "#,
        )
        .bind(query.user_id)
        .bind(&query.event_category)
        .bind(&query.event_type)
        .bind(&query.entity_type)
        .bind(query.entity_id)
        .bind(query.is_suspicious)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    /// 匯出使用者活動記錄（不分頁，上限 10000 筆）
    pub async fn export_activities(
        pool: &PgPool,
        query: &ActivityLogQuery,
    ) -> Result<Vec<UserActivityLog>> {
        let data = sqlx::query_as::<_, UserActivityLog>(
            r#"
            SELECT id, actor_user_id, actor_email, actor_display_name, actor_roles, session_id,
                   event_category, event_type, event_severity, entity_type, entity_id, entity_display_name,
                   before_data, after_data, changed_fields, ip_address::text as ip_address,
                   user_agent, request_path, request_method, response_status,
                   is_suspicious, suspicious_reason, created_at, partition_date
            FROM user_activity_logs
            WHERE ($1::uuid IS NULL OR actor_user_id = $1)
              AND ($2::text IS NULL OR event_category = $2)
              AND ($3::text IS NULL OR event_type = $3)
              AND ($4::text IS NULL OR entity_type = $4)
              AND ($5::uuid IS NULL OR entity_id = $5)
              AND ($6::bool IS NULL OR is_suspicious = $6)
              AND ($7::date IS NULL OR partition_date >= $7)
              AND ($8::date IS NULL OR partition_date <= $8)
            ORDER BY created_at DESC
            LIMIT 10000
            "#,
        )
        .bind(query.user_id)
        .bind(&query.event_category)
        .bind(&query.event_type)
        .bind(&query.entity_type)
        .bind(query.entity_id)
        .bind(query.is_suspicious)
        .bind(query.from)
        .bind(query.to)
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    // ============================================
    // Login Events
    // ============================================

    /// 列出登入事件
    pub async fn list_login_events(
        pool: &PgPool,
        query: &LoginEventQuery,
    ) -> Result<PaginatedResponse<LoginEventWithUser>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM login_events
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::text IS NULL OR event_type = $2)
              AND ($3::bool IS NULL OR (is_unusual_time OR is_unusual_location OR is_new_device) = $3)
              AND ($4::date IS NULL OR created_at::date >= $4)
              AND ($5::date IS NULL OR created_at::date <= $5)
            "#,
        )
        .bind(query.user_id)
        .bind(&query.event_type)
        .bind(query.is_unusual)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, LoginEventWithUser>(
            r#"
            SELECT 
                le.id, le.user_id, le.email, u.display_name as user_name,
                le.event_type, le.ip_address::TEXT, le.user_agent,
                le.device_type, le.browser, le.os,
                le.is_unusual_time, le.is_unusual_location, le.is_new_device, le.is_mass_login,
                le.failure_reason, le.created_at
            FROM login_events le
            LEFT JOIN users u ON le.user_id = u.id
            WHERE ($1::uuid IS NULL OR le.user_id = $1)
              AND ($2::text IS NULL OR le.event_type = $2)
              AND ($3::bool IS NULL OR (le.is_unusual_time OR le.is_unusual_location OR le.is_new_device OR le.is_mass_login) = $3)
              AND ($4::date IS NULL OR le.created_at::date >= $4)
              AND ($5::date IS NULL OR le.created_at::date <= $5)
            ORDER BY le.created_at DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(query.user_id)
        .bind(&query.event_type)
        .bind(query.is_unusual)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    // ============================================
    // Sessions
    // ============================================

    /// 列出 Sessions
    pub async fn list_sessions(
        pool: &PgPool,
        query: &SessionQuery,
    ) -> Result<PaginatedResponse<SessionWithUser>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM user_sessions
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::bool IS NULL OR is_active = $2)
              AND ($3::date IS NULL OR started_at::date >= $3)
              AND ($4::date IS NULL OR started_at::date <= $4)
            "#,
        )
        .bind(query.user_id)
        .bind(query.is_active)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, SessionWithUser>(
            r#"
            SELECT 
                s.id, s.user_id, u.email as user_email, u.display_name as user_name,
                s.started_at, s.ended_at, s.last_activity_at,
                s.ip_address::TEXT, s.user_agent,
                s.page_view_count, s.action_count,
                s.is_active, s.ended_reason
            FROM user_sessions s
            INNER JOIN users u ON s.user_id = u.id
            WHERE ($1::uuid IS NULL OR s.user_id = $1)
              AND ($2::bool IS NULL OR s.is_active = $2)
              AND ($3::date IS NULL OR s.started_at::date >= $3)
              AND ($4::date IS NULL OR s.started_at::date <= $4)
            ORDER BY s.started_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(query.user_id)
        .bind(query.is_active)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    /// 強制登出 Session
    pub async fn force_logout_session(
        pool: &PgPool,
        session_id: Uuid,
        admin_id: Uuid,
        reason: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_active = false,
                ended_at = NOW(),
                ended_reason = 'forced_logout'
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .execute(pool)
        .await?;

        // 記錄審計日誌
        Self::log(
            pool,
            admin_id,
            AuditAction::Logout,
            "session",
            session_id,
            None,
            Some(serde_json::json!({ "reason": reason.unwrap_or("admin_forced") })),
        )
        .await?;

        Ok(())
    }

    // ============================================
    // Security Alerts
    // ============================================

    /// 列出安全警報
    pub async fn list_security_alerts(
        pool: &PgPool,
        query: &SecurityAlertQuery,
    ) -> Result<PaginatedResponse<SecurityAlert>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM security_alerts
            WHERE ($1::text IS NULL OR status = $1)
              AND ($2::text IS NULL OR severity = $2)
              AND ($3::uuid IS NULL OR user_id = $3)
              AND ($4::date IS NULL OR created_at::date >= $4)
              AND ($5::date IS NULL OR created_at::date <= $5)
              AND ($6::text IS NULL OR (title ILIKE '%' || $6 || '%' OR description ILIKE '%' || $6 || '%'))
            "#,
        )
        .bind(&query.status)
        .bind(&query.severity)
        .bind(query.user_id)
        .bind(query.from)
        .bind(query.to)
        .bind(&query.query)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, SecurityAlert>(
            r#"
            SELECT * FROM security_alerts
            WHERE ($1::text IS NULL OR status = $1)
              AND ($2::text IS NULL OR severity = $2)
              AND ($3::uuid IS NULL OR user_id = $3)
              AND ($4::date IS NULL OR created_at::date >= $4)
              AND ($5::date IS NULL OR created_at::date <= $5)
              AND ($8::text IS NULL OR (title ILIKE '%' || $8 || '%' OR description ILIKE '%' || $8 || '%'))
            ORDER BY created_at DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(&query.status)
        .bind(&query.severity)
        .bind(query.user_id)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .bind(&query.query)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    /// 查詢指定時間之後的新安全警報（供前端 polling 使用）
    pub async fn find_recent_alerts(
        pool: &PgPool,
        after: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<SecurityAlert>> {
        let alerts = sqlx::query_as::<_, SecurityAlert>(
            r#"
            SELECT * FROM security_alerts
            WHERE created_at > $1
              AND status IN ('open', 'acknowledged')
            ORDER BY created_at DESC
            LIMIT 20
            "#,
        )
        .bind(after)
        .fetch_all(pool)
        .await?;

        Ok(alerts)
    }

    /// 解決安全警報
    pub async fn resolve_alert(
        pool: &PgPool,
        alert_id: Uuid,
        resolver_id: Uuid,
        notes: Option<&str>,
    ) -> Result<SecurityAlert> {
        let alert = sqlx::query_as::<_, SecurityAlert>(
            r#"
            UPDATE security_alerts
            SET status = 'resolved',
                resolved_by = $2,
                resolved_at = NOW(),
                resolution_notes = $3,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(alert_id)
        .bind(resolver_id)
        .bind(notes)
        .fetch_one(pool)
        .await?;

        Ok(alert)
    }

    /// 批次解決安全警報，回傳實際更新筆數
    pub async fn bulk_resolve_alerts(
        pool: &PgPool,
        ids: &[Uuid],
        resolver_id: Uuid,
        notes: Option<&str>,
    ) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE security_alerts
            SET status = 'resolved',
                resolved_by = $2,
                resolved_at = NOW(),
                resolution_notes = $3,
                updated_at = NOW()
            WHERE id = ANY($1)
              AND status != 'resolved'
            "#,
        )
        .bind(ids)
        .bind(resolver_id)
        .bind(notes)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    // ============================================
    // Dashboard
    // ============================================

    /// 取得審計儀表板統計
    pub async fn get_dashboard_stats(pool: &PgPool) -> Result<AuditDashboardStats> {
        let today = crate::time::today_taiwan_naive();
        let week_ago = today - chrono::Duration::days(7);
        let month_ago = today - chrono::Duration::days(30);

        // 活躍用戶數
        let active_today: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT actor_user_id) FROM user_activity_logs WHERE partition_date = $1",
        )
        .bind(today)
        .fetch_one(pool)
        .await
        .map_err(|e| { tracing::error!("audit stats query failed: {e}"); e })?;

        let active_week: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT actor_user_id) FROM user_activity_logs WHERE partition_date >= $1",
        )
        .bind(week_ago)
        .fetch_one(pool)
        .await
        .map_err(|e| { tracing::error!("audit stats query failed: {e}"); e })?;

        let active_month: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT actor_user_id) FROM user_activity_logs WHERE partition_date >= $1",
        )
        .bind(month_ago)
        .fetch_one(pool)
        .await
        .map_err(|e| { tracing::error!("audit stats query failed: {e}"); e })?;

        // 登入統計
        let logins_today: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM login_events WHERE created_at::date = $1 AND event_type = 'login_success'",
        )
        .bind(today)
        .fetch_one(pool)
        .await
        .map_err(|e| { tracing::error!("audit stats query failed: {e}"); e })?;

        let failed_logins: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM login_events WHERE created_at::date = $1 AND event_type = 'login_failure'",
        )
        .bind(today)
        .fetch_one(pool)
        .await
        .map_err(|e| { tracing::error!("audit stats query failed: {e}"); e })?;

        // 活躍 Sessions
        let active_sessions: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM user_sessions WHERE is_active = true")
                .fetch_one(pool)
                .await
                .map_err(|e| { tracing::error!("audit stats query failed: {e}"); e })?;

        // 警報統計
        let open_alerts: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM security_alerts WHERE status IN ('open', 'acknowledged', 'investigating')",
        )
        .fetch_one(pool)
        .await
        .map_err(|e| { tracing::error!("audit stats query failed: {e}"); e })?;

        let critical_alerts: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM security_alerts WHERE status IN ('open', 'acknowledged') AND severity = 'critical'",
        )
        .fetch_one(pool)
        .await
        .map_err(|e| { tracing::error!("audit stats query failed: {e}"); e })?;

        Ok(AuditDashboardStats {
            active_users_today: active_today.0,
            active_users_week: active_week.0,
            active_users_month: active_month.0,
            total_logins_today: logins_today.0,
            failed_logins_today: failed_logins.0,
            active_sessions: active_sessions.0,
            open_alerts: open_alerts.0,
            critical_alerts: critical_alerts.0,
        })
    }
}

#[cfg(test)]
mod hmac_versioning_tests {
    //! R26-6：HMAC 編碼版本化單元測試。
    //!
    //! 驗證 v1（legacy string-concat）與 v2（length-prefix canonical）編碼
    //! 產出的 HMAC **不同**，確保 verifier 分流正確（用錯版本會偵測為斷鏈）。
    use super::{AuditService, HmacInput, HMAC_VERSION_CANONICAL, HMAC_VERSION_LEGACY};
    use uuid::Uuid;

    const TEST_KEY: &str = "test-hmac-key-for-unit-tests-only";

    fn sample_input<'a>(
        category: &'a str,
        event_type: &'a str,
        actor_id: Uuid,
        before: &'a Option<serde_json::Value>,
        after: &'a Option<serde_json::Value>,
        changed_fields: &'a [String],
    ) -> HmacInput<'a> {
        HmacInput {
            event_category: category,
            event_type,
            actor_user_id: actor_id,
            before_data: before,
            after_data: after,
            impersonated_by: None,
            changed_fields,
        }
    }

    #[test]
    fn legacy_and_canonical_encodings_produce_different_hashes() {
        let actor = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
            .expect("hard-coded test UUID must parse");
        let before = Some(serde_json::json!({"a": 1}));
        let after = Some(serde_json::json!({"a": 2}));
        let fields = vec!["a".to_string()];

        let input = sample_input("ANIMAL", "UPDATE", actor, &before, &after, &fields);
        let prev = Some("abcd1234");

        let v1 = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            prev,
            HmacInput {
                event_category: input.event_category,
                event_type: input.event_type,
                actor_user_id: input.actor_user_id,
                before_data: input.before_data,
                after_data: input.after_data,
                impersonated_by: input.impersonated_by,
                changed_fields: input.changed_fields,
            },
            HMAC_VERSION_LEGACY,
        ).expect("test key valid");
        let v2 = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            prev,
            input,
            HMAC_VERSION_CANONICAL,
        ).expect("test key valid");

        assert_ne!(v1, v2, "v1 vs v2 不同編碼 HMAC 應有差異");
        assert_eq!(v1.len(), 64, "HMAC-SHA256 hex 應為 64 字元");
        assert_eq!(v2.len(), 64);
    }

    #[test]
    fn canonical_encoding_detects_string_concat_collision() {
        // 經典碰撞：("ab","cd") 與 ("abc","d") 在字串串接下產生同一 message
        let actor = Uuid::nil();
        let before = None::<serde_json::Value>;
        let after = None::<serde_json::Value>;
        let fields: Vec<String> = vec![];

        let case_a = sample_input("ab", "cd", actor, &before, &after, &fields);
        let case_b = sample_input("abc", "d", actor, &before, &after, &fields);

        // v1 legacy：會碰撞（這就是為什麼需要 v2）
        let legacy_a = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            None,
            HmacInput {
                event_category: case_a.event_category,
                event_type: case_a.event_type,
                actor_user_id: case_a.actor_user_id,
                before_data: case_a.before_data,
                after_data: case_a.after_data,
                impersonated_by: case_a.impersonated_by,
                changed_fields: case_a.changed_fields,
            },
            HMAC_VERSION_LEGACY,
        ).expect("test key valid");
        let legacy_b = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            None,
            case_b,
            HMAC_VERSION_LEGACY,
        ).expect("test key valid");
        assert_eq!(legacy_a, legacy_b, "legacy 確實會碰撞（合預期，說明為何需 v2）");

        // v2 canonical：不會碰撞
        let v2_a = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            None,
            case_a,
            HMAC_VERSION_CANONICAL,
        ).expect("test key valid");
        let input_b_v2 = sample_input("abc", "d", actor, &before, &after, &fields);
        let v2_b = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            None,
            input_b_v2,
            HMAC_VERSION_CANONICAL,
        ).expect("test key valid");
        assert_ne!(v2_a, v2_b, "canonical v2 不應碰撞");
    }

    #[test]
    fn unknown_version_falls_back_to_canonical() {
        let actor = Uuid::nil();
        let before = None::<serde_json::Value>;
        let after = None::<serde_json::Value>;
        let fields: Vec<String> = vec![];

        let input = sample_input("X", "Y", actor, &before, &after, &fields);
        let input2 = sample_input("X", "Y", actor, &before, &after, &fields);

        let v2 = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            None,
            input,
            HMAC_VERSION_CANONICAL,
        ).expect("test key valid");
        let unknown = AuditService::compute_hmac_for_fields_versioned(TEST_KEY, None, input2, 99).expect("test key valid");

        assert_eq!(
            v2, unknown,
            "未知版本應 fallback canonical，不應 panic"
        );
    }
}
