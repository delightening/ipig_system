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
struct HmacInput<'a> {
    event_category: &'a str,
    event_type: &'a str,
    actor_user_id: Uuid,
    before_data: &'a Option<serde_json::Value>,
    after_data: &'a Option<serde_json::Value>,
    impersonated_by: Option<Uuid>,
    changed_fields: &'a [String],
}

impl HmacInput<'_> {
    /// 將所有欄位以 length-prefix canonical form 寫入 buffer。
    ///
    /// 每個欄位：`8-byte BE length` + `UTF-8 bytes`
    /// changed_fields 以 `array length (u64 BE)` 開頭，再逐欄位 length-prefix。
    ///
    /// prev_hash 由呼叫端（從 DB 讀出）提供；這設計讓 struct 只承載「內容」，
    /// chain 連結資料由 DB 提供。
    fn canonical_bytes(&self, prev_hash: Option<&str>) -> Vec<u8> {
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
    /// 取代舊版 `log_activity(pool, ...)`（已標 `#[deprecated]`）。
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
    /// 對給定欄位集合計算 HMAC（不接觸 DB，純運算）。
    ///
    /// 供 [`verify_chain_range`](Self::verify_chain_range) 使用：針對每筆 audit row
    /// 重算預期 HMAC，和儲存值比對即可判定 chain 完整性。
    pub(crate) fn compute_hmac_for_fields(
        hmac_key: &str,
        prev_hash: Option<&str>,
        event_category: &str,
        event_type: &str,
        actor_user_id: Uuid,
        before_data: &Option<serde_json::Value>,
        after_data: &Option<serde_json::Value>,
        impersonated_by: Option<Uuid>,
        changed_fields: &[String],
    ) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let input = HmacInput {
            event_category,
            event_type,
            actor_user_id,
            before_data,
            after_data,
            impersonated_by,
            changed_fields,
        };
        let message = input.canonical_bytes(prev_hash);

        let mut mac = HmacSha256::new_from_slice(hmac_key.as_bytes())
            .expect("HMAC key invalid — 啟動時應已驗證"); // OnceLock 存的 key 必有效
        mac.update(&message);
        let hash_bytes = mac.finalize().into_bytes();
        hash_bytes.iter().map(|b| format!("{:02x}", b)).collect()
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
    /// 流程：
    /// 1. 取範圍內所有 row，ORDER BY (created_at, id) ASC
    /// 2. 取範圍**前一筆** row 的 integrity_hash 作為初始 prev_hash
    /// 3. 逐行：重算 HMAC → 與 row.integrity_hash 比對
    ///    - 不等 → 加入 broken_links
    ///    - 推進 prev_hash = row.integrity_hash（即使 broken 也繼續，方便定位
    ///      連鎖破壞）
    ///
    /// **效能考量**：此方法會把範圍內所有 row 拉進記憶體。對每日驗證（~1000-10000 rows）
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

        #[derive(sqlx::FromRow)]
        struct ChainRow {
            id: Uuid,
            created_at: chrono::DateTime<chrono::Utc>,
            event_category: String,
            event_type: String,
            actor_user_id: Uuid,
            before_data: Option<serde_json::Value>,
            after_data: Option<serde_json::Value>,
            impersonated_by_user_id: Option<Uuid>,
            changed_fields: Option<Vec<String>>,
            integrity_hash: Option<String>,
            previous_hash: Option<String>,
        }

        let rows: Vec<ChainRow> = sqlx::query_as::<_, ChainRow>(
            r#"
            SELECT id, created_at, event_category, event_type, actor_user_id,
                   before_data, after_data, impersonated_by_user_id,
                   changed_fields, integrity_hash, previous_hash
            FROM user_activity_logs
            WHERE created_at >= $1 AND created_at < $2
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(pool)
        .await?;

        let total_rows = rows.len();
        if total_rows == 0 {
            return Ok(ChainVerificationReport {
                range_from: from,
                range_to: to,
                total_rows: 0,
                verified_rows: 0,
                skipped_no_hash: 0,
                broken_links: Vec::new(),
            });
        }

        // 取範圍前一筆 row 的 hash 作為起始 prev_hash
        let first = &rows[0];
        let mut prev_hash: Option<String> = sqlx::query_scalar(
            r#"
            SELECT integrity_hash FROM user_activity_logs
            WHERE (created_at, id) < ($1, $2) AND integrity_hash IS NOT NULL
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(first.created_at)
        .bind(first.id)
        .fetch_optional(pool)
        .await?;

        let mut broken_links = Vec::new();
        let mut verified_rows = 0_usize;
        let mut skipped_no_hash = 0_usize;

        for row in &rows {
            // 有些安全事件（log_security_event）不走 HMAC chain；這類 row 略過驗證
            let Some(stored_hash) = row.integrity_hash.as_deref() else {
                skipped_no_hash += 1;
                continue;
            };

            let changed_fields = row.changed_fields.as_deref().unwrap_or(&[]);
            let expected = Self::compute_hmac_for_fields(
                hmac_key,
                prev_hash.as_deref(),
                &row.event_category,
                &row.event_type,
                row.actor_user_id,
                &row.before_data,
                &row.after_data,
                row.impersonated_by_user_id,
                changed_fields,
            );

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

        Ok(ChainVerificationReport {
            range_from: from,
            range_to: to,
            total_rows,
            verified_rows,
            skipped_no_hash,
            broken_links,
        })
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
        let prev_hash: Option<String> = sqlx::query_scalar(
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

        let message = input.canonical_bytes(prev_hash.as_deref());

        let mut mac = HmacSha256::new_from_slice(hmac_key.as_bytes())
            .map_err(|e| crate::error::AppError::Internal(format!("HMAC key error: {}", e)))?;
        mac.update(&message);
        let hash_bytes = mac.finalize().into_bytes();
        let hash_result: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        sqlx::query(
            "UPDATE user_activity_logs SET integrity_hash = $1, previous_hash = $2 WHERE id = $3",
        )
        .bind(&hash_result)
        .bind(prev_hash.as_deref())
        .bind(log_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 記錄詳細活動日誌（舊版 API，保留向後相容）。
    ///
    /// ⚠️ **此函式不保證 audit 與資料變更在同一 transaction 內**。若呼叫端
    /// 在資料寫入與本函式之間 crash，會造成資料變更已 commit 但 audit log
    /// 未寫入（GLP 合規風險）。
    ///
    /// 新程式碼請改用 [`AuditService::log_activity_tx`]，搭配 `pool.begin()`。
    #[deprecated(
        since = "0.2.0",
        note = "改用 log_activity_tx 保證 audit 與資料變更同一 tx；舊版在分批遷移後移除"
    )]
    #[allow(clippy::too_many_arguments)]
    pub async fn log_activity(
        pool: &PgPool,
        actor_user_id: Uuid,
        event_category: &str,
        event_type: &str,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
        entity_display_name: Option<&str>,
        before_data: Option<serde_json::Value>,
        after_data: Option<serde_json::Value>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<Uuid> {
        let result: (Uuid,) =
            sqlx::query_as("SELECT log_activity($1, $2, $3, $4, $5, $6, $7, $8, $9::inet, $10)")
                .bind(actor_user_id)
                .bind(event_category)
                .bind(event_type)
                .bind(entity_type)
                .bind(entity_id)
                .bind(entity_display_name)
                .bind(before_data.clone())
                .bind(after_data.clone())
                .bind(ip_address)
                .bind(user_agent)
                .fetch_one(pool)
                .await?;

        let log_id = result.0;

        // SEC-34: HMAC 雜湊鏈（有 key 時才啟用）。
        // 錯誤不再靜默吞（let _ = ...），改記 tracing::error! 讓 Prometheus
        // 可 alert；但仍不中斷回傳 — audit 已寫入、HMAC 失敗只影響鏈結
        // 完整性，呼叫端已拿到 log_id。R26-4 移除舊版時此區塊一併刪除。
        if let Some(Some(hmac_key)) = AUDIT_HMAC_KEY.get() {
            if let Err(e) = Self::compute_and_store_hmac(
                pool,
                log_id,
                hmac_key,
                event_category,
                event_type,
                actor_user_id,
                &before_data,
                &after_data,
            )
            .await
            {
                tracing::error!(
                    log_id = %log_id,
                    error = %e,
                    "[audit] HMAC chain write failed for deprecated log_activity; chain integrity degraded"
                );
            }
        }

        Ok(log_id)
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
        .execute(pool)
        .await?;

        Ok(id)
    }

    /// 單據審計專用函式（減少重複程式碼）
    pub async fn audit_document(
        pool: &PgPool,
        user_id: Uuid,
        event_type: &str, // DOC_CREATE, DOC_SUBMIT, etc.
        doc_id: Uuid,
        doc_no: &str,
        doc_type: Option<&str>,
        extra_data: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut after_data = serde_json::json!({
            "doc_no": doc_no,
        });

        if let Some(dt) = doc_type {
            after_data["doc_type"] = serde_json::Value::String(dt.to_string());
        }

        if let Some(extra) = extra_data {
            after_data["extra"] = extra;
        }

        Self::log_activity(
            pool,
            user_id,
            "ERP",
            event_type,
            Some("document"),
            Some(doc_id),
            Some(doc_no),
            None,
            Some(after_data),
            None,
            None,
        )
        .await?;

        Ok(())
    }

    /// SEC-34: 計算 HMAC-SHA256 並更新日誌記錄
    #[allow(clippy::too_many_arguments)]
    async fn compute_and_store_hmac(
        pool: &PgPool,
        log_id: Uuid,
        hmac_key: &str,
        event_category: &str,
        event_type: &str,
        actor_user_id: Uuid,
        before_data: &Option<serde_json::Value>,
        after_data: &Option<serde_json::Value>,
    ) -> Result<()> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        // 取得前一筆日誌的 integrity_hash（若有）
        let prev_hash: Option<String> = sqlx::query_scalar(
            r#"
            SELECT integrity_hash FROM user_activity_logs
            WHERE created_at < (SELECT created_at FROM user_activity_logs WHERE id = $1)
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(log_id)
        .fetch_optional(pool)
        .await?;

        // 組合雜湊內容: prev_hash + event_category + event_type + actor_id + data
        let mut message = String::new();
        if let Some(ref ph) = prev_hash {
            message.push_str(ph);
        }
        message.push_str(event_category);
        message.push_str(event_type);
        message.push_str(&actor_user_id.to_string());
        if let Some(ref bd) = before_data {
            message.push_str(&bd.to_string());
        }
        if let Some(ref ad) = after_data {
            message.push_str(&ad.to_string());
        }

        // 計算 HMAC-SHA256
        let mut mac = HmacSha256::new_from_slice(hmac_key.as_bytes())
            .map_err(|e| crate::error::AppError::Internal(format!("HMAC key error: {}", e)))?;
        mac.update(message.as_bytes());
        let hash_bytes = mac.finalize().into_bytes();
        let hash_result: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        // 更新記錄
        sqlx::query(
            "UPDATE user_activity_logs SET integrity_hash = $1, previous_hash = $2 WHERE id = $3",
        )
        .bind(&hash_result)
        .bind(prev_hash.as_deref())
        .bind(log_id)
        .execute(pool)
        .await?;

        Ok(())
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
