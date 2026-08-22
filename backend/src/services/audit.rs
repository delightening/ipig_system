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
// R53-6: Audit entity_type blacklist
// ============================================
//
// 廢棄物再利用紀錄（byproduct reuse 框架）的 entity_type 不應出現在任何
// audit list endpoint 的 response。對應 R53 規格決策：
//   "PI audit log 範圍限縮為「研究內容相關事件」；byproduct_samples entity_type
//    對 PI viewer 整類不出現；PDF / 出貨文件不洩漏欄位"
//
// 採全 viewer 一致過濾策略（admin / VET / QAU 也看不到），避免「忘記檢查
// viewer role」的 bug。Admin 需稽核時走 byproduct_sample API（其自有
// `animal.byproduct_sample.view` permission gate）。
//
// Audit row 本身仍寫入 user_activity_logs（HMAC chain 不破），只是 list /
// export endpoint 過濾掉。內部 SOP 稽核需要時 DBA 可直接 SQL 查。
pub const AUDIT_ENTITY_BLACKLIST: &[&str] = &["byproduct_sample"];

/// 將 const `&[&str]` 轉成 sqlx 可 bind 的 `&[String]`，OnceLock 快取以避免
/// 每次 audit list / export 都重 alloc（Gemini review）。
fn audit_entity_blacklist() -> &'static [String] {
    static BLACKLIST: OnceLock<Vec<String>> = OnceLock::new();
    BLACKLIST.get_or_init(|| {
        AUDIT_ENTITY_BLACKLIST
            .iter()
            .map(|s| s.to_string())
            .collect()
    })
}

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
pub struct HmacInput<'a> {
    pub event_category: &'a str,
    pub event_type: &'a str,
    pub actor_user_id: Uuid,
    pub before_data: &'a Option<serde_json::Value>,
    pub after_data: &'a Option<serde_json::Value>,
    pub impersonated_by: Option<Uuid>,
    pub changed_fields: &'a [String],
    /// R30-9a: v3 only — entity_type 進 hash 修補 v2 entity gap。v2 path 不使用此欄位。
    pub entity_type: Option<&'a str>,
    /// R30-9a: v3 only — entity_id 進 hash。
    pub entity_id: Option<&'a str>,
    /// R30-9a: v3 only — 額外綁定（SIGNATURE_CREATE: sig_id:content_hash;
    /// SIGNATURE_INVALIDATED: sig_id:reason_hash）。
    pub extra_input: Option<&'a str>,
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

/// R30-9a: v3 編碼 — 在 v2 後接續 length-prefix entity_type / entity_id / extra_input。
/// 修補 v2 entity gap 並讓特定事件能綁額外 fingerprint。
/// 由 SIGNATURE_CREATE / SIGNATURE_INVALIDATED 強制使用；其他 event_type 仍走 v2。
pub(crate) const HMAC_VERSION_V3: i16 = 3;

/// R30-9a: 簽章建立事件（`AuditService::log_signature_event_tx` action 參數）。
pub const SIGNATURE_EVENT_CREATE: &str = "SIGNATURE_CREATE";
/// R30-9a: 簽章作廢事件。
pub const SIGNATURE_EVENT_INVALIDATED: &str = "SIGNATURE_INVALIDATED";

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

    /// R30-9a: v3 編碼 — 在 v2 後接續 length-prefix entity_type / entity_id / extra_input。
    /// 三個欄位用 `Option::unwrap_or("")` 處理（v2 row 升 v3 時這 3 欄為空字串，
    /// 仍維持 length-prefix 規則寫入 length=0；防 hash 碰撞）。
    pub(crate) fn canonical_bytes_v3(&self, prev_hash: Option<&str>) -> Vec<u8> {
        let mut buf = self.canonical_bytes(prev_hash); // 重用 v2 編碼（與 v2 row 共享尾部相容性）
        write_field(&mut buf, self.entity_type.unwrap_or("").as_bytes());
        write_field(&mut buf, self.entity_id.unwrap_or("").as_bytes());
        write_field(&mut buf, self.extra_input.unwrap_or("").as_bytes());
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
    /// HMAC 不一致**且未列入已知斷鏈白名單**的 row（觸發告警）
    pub broken_links: Vec<BrokenChainLink>,
    /// HMAC 不一致但已登記於 `audit_chain_known_breaks`（歷史寫入 bug 產物、非竄改）→
    /// 不觸發告警；保留計數供報表呈現。
    pub acknowledged_breaks: usize,
}

impl ChainVerificationReport {
    /// chain 完整 = 無「未確認」斷鏈（已知斷鏈白名單不計入）。
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
    /// R30-9a: v3 only — 額外綁定（SIGNATURE_*）；v2 row 為 NULL。
    /// verifier 重算時用此欄位重建 HmacInput::extra_input。
    extra_input: Option<String>,
    /// audit log entity_type；R30-9a v3 進 hash，v2 row hash 不依賴此欄位。
    entity_type: Option<String>,
    /// audit log entity_id；R30-9a v3 進 hash。
    entity_id: Option<String>,
}

pub struct AuditService;

/// 跳脫 ILIKE 萬用字元（`\` `%` `_`），使其在 `... ILIKE pattern ESCAPE '\'` 下被當字面
/// 字元而非萬用字元。供 audit 自由文字搜尋的 list / export 兩處共用（Low-2 #265 / bot review #630）。
/// 反斜線需先跳脫，否則會吃掉後續的 `%` / `_` 跳脫。
fn escape_ilike_wildcards(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

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
        // 一般路徑：version + extra_input 由 derive_v3_routing 自動推導
        // （SIGNATURE_* 走 v3、其餘走 v2）。需要顯式控制者改用 log_signature_event_tx。
        Self::log_activity_tx_inner(tx, actor, entry, None, None, false).await
    }

    /// R28-5 follow-up：可疑安全事件專屬 tx 寫入路徑（走 HMAC chain）。
    ///
    /// 與 [`log_activity_tx`] 唯一差異：傳 `is_suspicious=true`，讓 stored proc
    /// 標 `is_suspicious=true` + `event_severity='warning'` + `suspicious_reason`，
    /// 還原 R28-5 前直接 INSERT 的安全事件語意。**僅供 `log_security_event*` 使用** —
    /// 一般 SECURITY 分類事件（改密碼 / 開權限等正常操作）不得走此路徑。
    async fn log_security_activity_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        entry: ActivityLogEntry<'_>,
    ) -> Result<Uuid> {
        Self::log_activity_tx_inner(tx, actor, entry, None, None, true).await
    }

    /// R28-5 follow-up：[`log_security_activity_tx`] 的 pool 版（自管 tx）。
    async fn log_security_activity_oneshot(
        pool: &PgPool,
        actor: &ActorContext,
        entry: ActivityLogEntry<'_>,
    ) -> Result<Uuid> {
        let mut tx = pool.begin().await?;
        let log_id = Self::log_security_activity_tx(&mut tx, actor, entry).await?;
        tx.commit().await?;
        Ok(log_id)
    }

    /// R30-9a: 簽章事件專屬 audit 寫入路徑。
    ///
    /// 與 [`log_activity_tx`] 的差異：caller 顯式提供 `sig_id` + `binding`
    /// （CREATE: content_hash；INVALIDATED: invalidated_reason），由本函式
    /// 組成 `<sig_id>:<binding>` 並**強制**走 v3 編碼，不依賴 after_data 的
    /// JSON shape — 簽章 model 欄位日後改名也不會悄悄退回 `:""`。
    ///
    /// `action` 必為 [`SIGNATURE_EVENT_CREATE`] 或 [`SIGNATURE_EVENT_INVALIDATED`]
    /// 兩個常數之一（保留給未來擴充其他 SIGNATURE_* 事件）。
    pub async fn log_signature_event_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        action: &'static str,
        sig_id: Uuid,
        binding: &str,
        entry: ActivityLogEntry<'_>,
    ) -> Result<Uuid> {
        // Runtime 驗證（非 debug_assert！release build 也須生效，否則 caller 可
        // 寫出 v3 row 但 action 不是 SIGNATURE_*，破壞 chain 規則）。
        if action != SIGNATURE_EVENT_CREATE && action != SIGNATURE_EVENT_INVALIDATED {
            return Err(crate::error::AppError::Validation(format!(
                "log_signature_event_tx action 必為 SIGNATURE_EVENT_CREATE/INVALIDATED，收到 {action}"
            )));
        }
        if entry.event_type != action {
            return Err(crate::error::AppError::Validation(format!(
                "ActivityLogEntry.event_type ({}) 必須與 action ({action}) 一致",
                entry.event_type
            )));
        }
        if binding.is_empty() {
            return Err(crate::error::AppError::Validation(
                "log_signature_event_tx binding 不可為空（會弱化 v3 fingerprint 保護）".into(),
            ));
        }
        let extra = format!("{sig_id}:{binding}");
        Self::log_activity_tx_inner(tx, actor, entry, Some(HMAC_VERSION_V3), Some(extra), false)
            .await
    }

    async fn log_activity_tx_inner(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        entry: ActivityLogEntry<'_>,
        version_override: Option<i16>,
        extra_input_override: Option<String>,
        is_suspicious: bool,
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
        // R28-M3：lock key 集中於 crate::constants
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
            .bind(crate::constants::AUDIT_LOG_CHAIN_LOCK_KEY)
            .execute(&mut **tx)
            .await?;

        // R26-3: 一次 INSERT 完整寫入（含 impersonated_by + changed_fields），
        // 不再做事後 UPDATE — 這樣 HMAC 計算能涵蓋所有欄位，tamper-resistance 完整。
        let result: (Uuid,) = sqlx::query_as(
            "SELECT log_activity($1, $2, $3, $4, $5, $6, $7, $8, $9::inet, $10, $11, $12, $13)",
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
        .bind(is_suspicious)
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
            // R30-9a: caller 顯式 override（log_signature_event_tx 走此路徑）優先，
            // 否則由 derive_v3_routing 從 event_type + after_data 自動推導。
            let (version, extra_input_owned) = match (version_override, extra_input_override) {
                (Some(v), xi) => (v, xi),
                (None, _) => Self::derive_v3_routing(entry.event_type, &after_data)?,
            };
            let entity_id_str = entry.entity.as_ref().map(|e| e.entity_id.to_string());
            let hmac_input = HmacInput {
                event_category: entry.event_category,
                event_type: entry.event_type,
                actor_user_id: hash_actor,
                before_data: &before_data,
                after_data: &after_data,
                impersonated_by,
                changed_fields: &changed_fields,
                entity_type: entry.entity.as_ref().map(|e| e.entity_type),
                entity_id: entity_id_str.as_deref(),
                extra_input: extra_input_owned.as_deref(),
            };
            Self::compute_and_store_hmac_tx(
                tx,
                log_id,
                hmac_key,
                hmac_input,
                version,
                extra_input_owned.as_deref(),
            )
            .await?;
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
    pub fn compute_hmac_for_fields_versioned(
        hmac_key: &str,
        prev_hash: Option<&str>,
        input: HmacInput<'_>,
        version: i16,
    ) -> Result<String> {
        use hmac::{Hmac, KeyInit, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(hmac_key.as_bytes())
            .map_err(|e| crate::error::AppError::Internal(format!("HMAC key invalid: {}", e)))?;

        // R30-9a: 三版本分流。未知版本（未來擴充）fallback v2 canonical。
        match version {
            HMAC_VERSION_LEGACY => {
                mac.update(input.legacy_concat_message(prev_hash).as_bytes());
            }
            HMAC_VERSION_V3 => {
                mac.update(&input.canonical_bytes_v3(prev_hash));
            }
            _ => {
                // HMAC_VERSION_CANONICAL (2) 與未知版本均走 v2
                mac.update(&input.canonical_bytes(prev_hash));
            }
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
            crate::error::AppError::Internal("AUDIT_HMAC_KEY 未設定 — 無法驗證 chain 完整性".into())
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
                acknowledged_breaks: 0,
            });
        }

        let first = &rows[0];
        let prev_hash = Self::load_initial_prev_hash(pool, first.created_at, first.id).await?;

        let (verified_rows, skipped_no_hash, all_broken) =
            Self::verify_chain_rows(hmac_key, prev_hash, &rows)?;

        // C（migration 097）：把已登記於 audit_chain_known_breaks 的歷史斷鏈分流為
        // acknowledged，不計入觸發告警的 broken_links；未登記者仍為真正斷鏈。
        let known = Self::load_known_break_ids(pool).await?;
        let (acknowledged, broken_links): (Vec<_>, Vec<_>) =
            all_broken.into_iter().partition(|b| known.contains(&b.id));

        Ok(ChainVerificationReport {
            range_from: from,
            range_to: to,
            total_rows: rows.len(),
            verified_rows,
            skipped_no_hash,
            broken_links,
            acknowledged_breaks: acknowledged.len(),
        })
    }

    /// 載入 `audit_chain_known_breaks` 全部 log_id（已知歷史斷鏈白名單）。
    async fn load_known_break_ids(pool: &PgPool) -> Result<std::collections::HashSet<Uuid>> {
        let ids: Vec<Uuid> = sqlx::query_scalar("SELECT log_id FROM audit_chain_known_breaks")
            .fetch_all(pool)
            .await?;
        Ok(ids.into_iter().collect())
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
                   changed_fields, integrity_hash, previous_hash, hmac_version,
                   extra_input, entity_type, entity_id::text AS entity_id
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
                        // R30-9a v3 欄位 — v2 row 的 entity_*/extra_input 雖在 DB 有值
                        // 但 v2 編碼不會用到（hash 公式不依賴），所以 v2 path 拿到也無害；
                        // v3 path 一定要拿到才能重算 hash 正確。
                        entity_type: row.entity_type.as_deref(),
                        entity_id: row.entity_id.as_deref(),
                        extra_input: row.extra_input.as_deref(),
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

    /// R30-9a: 依 event_type + after_data 決定 HMAC 編碼版本與 extra_input。
    ///
    /// `SIGNATURE_CREATE` / `SIGNATURE_INVALIDATED` → v3，從 after_data
    /// （ElectronicSignature row serialized via DataDiff）抽：
    /// - `id`（簽章 UUID）
    /// - `content_hash`（CREATE）或 `invalidated_reason`（INVALIDATED）
    ///
    /// 組成 `<sig_id>:<binding>` 進入 HMAC chain，讓事後竄改簽章內容或
    /// 偽造 INVALIDATED 都會破壞 chain hash。
    ///
    /// 其他 event_type → v2（保留現狀），extra_input = None。
    fn derive_v3_routing(
        event_type: &str,
        after_data: &Option<serde_json::Value>,
    ) -> Result<(i16, Option<String>)> {
        // 安全網：若 caller 沒走 log_signature_event_tx 而走一般 log_activity_tx
        // 寫 SIGNATURE_*，仍嘗試自動升 v3（從 after_data 抽 id + content_hash/reason）。
        // 缺欄位則 fail loud — 不能靜默退成「v3 但無 binding」的弱化 row，否則就
        // 繞過 R30-9a 要補的保護。主要路徑請走 log_signature_event_tx。
        const SIGNATURE_EVENTS: &[&str] = &[SIGNATURE_EVENT_CREATE, SIGNATURE_EVENT_INVALIDATED];
        if !SIGNATURE_EVENTS.contains(&event_type) {
            return Ok((HMAC_VERSION_CANONICAL, None));
        }
        let obj = after_data
            .as_ref()
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                crate::error::AppError::Internal(format!(
                    "{event_type} 寫 audit 缺 after_data object — 改用 log_signature_event_tx \
                     顯式傳 sig_id + binding"
                ))
            })?;
        let sig_id = obj.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::AppError::Internal(format!(
                "{event_type} after_data 缺 id 欄位（v3 fingerprint binding 需要 sig_id）"
            ))
        })?;
        let binding = obj
            .get("content_hash")
            .or_else(|| obj.get("invalidated_reason"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                crate::error::AppError::Internal(format!(
                    "{event_type} after_data 缺 content_hash/invalidated_reason \
                     （v3 fingerprint binding 需要 binding 值）"
                ))
            })?;
        Ok((HMAC_VERSION_V3, Some(format!("{sig_id}:{binding}"))))
    }

    async fn compute_and_store_hmac_tx(
        tx: &mut Transaction<'_, Postgres>,
        log_id: Uuid,
        hmac_key: &str,
        input: HmacInput<'_>,
        version: i16,
        extra_input: Option<&str>,
    ) -> Result<()> {
        use hmac::{Hmac, KeyInit, Mac};
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

        // R30-9a: 依 version 選編碼路徑（v3 多三個 length-prefix 欄位）。
        let message = match version {
            HMAC_VERSION_V3 => input.canonical_bytes_v3(prev_hash.as_deref()),
            _ => input.canonical_bytes(prev_hash.as_deref()),
        };

        let mut mac = HmacSha256::new_from_slice(hmac_key.as_bytes())
            .map_err(|e| crate::error::AppError::Internal(format!("HMAC key error: {}", e)))?;
        mac.update(&message);
        let hash_bytes = mac.finalize().into_bytes();
        let hash_result: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        // R30-9a: extra_input 隨 version 一起寫入，verifier 重算需此欄位。
        sqlx::query(
            "UPDATE user_activity_logs SET integrity_hash = $1, previous_hash = $2, \
             hmac_version = $3, extra_input = $4 WHERE id = $5",
        )
        .bind(&hash_result)
        .bind(prev_hash.as_deref())
        .bind(version)
        .bind(extra_input)
        .bind(log_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// R22: 記錄安全事件（rate limit / 403 / lockout 等）
    ///
    /// Gemini #5: 支援可選 actor_user_id（403 事件記錄使用者，方便 IDOR 查詢用索引欄位）
    ///
    /// R28-5: 改走 log_security_activity_oneshot（含 HMAC chain），不再產生 NULL
    /// hmac_version rows，並標 is_suspicious=true + event_severity='warning'。
    /// Actor 由 actor_user_id 決定：Some(uid) → User（保留使用者身份），None → Anonymous。
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
        use crate::middleware::{ActorContext, CurrentUser};

        let actor = match actor_user_id {
            Some(uid) => ActorContext::User(CurrentUser {
                id: uid,
                email: String::new(),
                roles: vec![],
                permissions: vec![],
                jti: String::new(),
                exp: 0,
                impersonated_by: None,
            }),
            None => ActorContext::Anonymous,
        };

        let mut detail = context.clone();
        if let Some(obj) = detail.as_object_mut() {
            if let Some(p) = request_path {
                obj.insert("request_path".into(), serde_json::Value::String(p.into()));
            }
            if let Some(m) = request_method {
                obj.insert("request_method".into(), serde_json::Value::String(m.into()));
            }
        }

        Self::log_security_activity_oneshot(
            pool,
            &actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type,
                entity: actor_user_id.map(|uid| AuditEntity::new("user", uid, "")),
                data_diff: {
                    let mut dd = crate::models::audit_diff::DataDiff::empty();
                    dd.after = Some(detail);
                    Some(dd)
                },
                request_context: Some(RequestContext {
                    ip_address,
                    user_agent,
                }),
            },
        )
        .await
    }

    /// H8：log_security_event 的 tx 版（接受 &mut Transaction），讓帳號鎖定等
    /// 安全事件可與業務 SQL 在同一原子事務內寫入，避免 tokio::spawn 的火忘式
    /// 寫入在進程崩潰時遺失稽核紀錄。
    ///
    /// R28-5：改走 log_security_activity_tx（含 HMAC chain），不再產生 NULL
    /// hmac_version rows，並標 is_suspicious=true + event_severity='warning'。
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
        use crate::middleware::{ActorContext, CurrentUser};

        let actor = match actor_user_id {
            Some(uid) => ActorContext::User(CurrentUser {
                id: uid,
                email: String::new(),
                roles: vec![],
                permissions: vec![],
                jti: String::new(),
                exp: 0,
                impersonated_by: None,
            }),
            None => ActorContext::Anonymous,
        };

        let mut detail = context.clone();
        if let Some(obj) = detail.as_object_mut() {
            if let Some(p) = request_path {
                obj.insert("request_path".into(), serde_json::Value::String(p.into()));
            }
            if let Some(m) = request_method {
                obj.insert("request_method".into(), serde_json::Value::String(m.into()));
            }
        }

        Self::log_security_activity_tx(
            tx,
            &actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type,
                entity: actor_user_id.map(|uid| AuditEntity::new("user", uid, "")),
                data_diff: {
                    let mut dd = crate::models::audit_diff::DataDiff::empty();
                    dd.after = Some(detail);
                    Some(dd)
                },
                request_context: Some(RequestContext {
                    ip_address,
                    user_agent,
                }),
            },
        )
        .await
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

        // R30-14: 自由文字搜尋，限制最多 100 字防 DoS
        let q_text: Option<String> = query
            .query
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            // Low-2 (#265): 截斷 100 字後跳脫 ILIKE 萬用字元（見 escape_ilike_wildcards），
            // 搭配 SQL `ESCAPE '\'` 使 q_text 內的 % / _ 被當字面字元而非萬用字元。
            .map(|s| escape_ilike_wildcards(&s.chars().take(100).collect::<String>()));

        // R53-6: blacklist 過濾（byproduct_sample entity_type 整類不出現）
        let blacklist: &[String] = audit_entity_blacklist();

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
              AND ($9::text IS NULL OR (
                    entity_display_name ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR actor_display_name  ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR actor_email         ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR event_type          ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR ip_address::text    ILIKE '%' || $9 || '%' ESCAPE '\'
              ))
              AND (entity_type IS NULL OR entity_type <> ALL($10::text[]))
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
        .bind(&q_text)
        .bind(blacklist)
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
              AND ($9::text IS NULL OR (
                    entity_display_name ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR actor_display_name  ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR actor_email         ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR event_type          ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR ip_address::text    ILIKE '%' || $9 || '%' ESCAPE '\'
              ))
              AND (entity_type IS NULL OR entity_type <> ALL($10::text[]))
            ORDER BY created_at DESC
            LIMIT $11 OFFSET $12
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
        .bind(&q_text)
        .bind(blacklist)
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
        // R30-14: 自由文字搜尋，限制最多 100 字防 DoS
        let q_text: Option<String> = query
            .query
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            // Low-2 (#265): 截斷 100 字後跳脫 ILIKE 萬用字元（見 escape_ilike_wildcards），
            // 搭配 SQL `ESCAPE '\'` 使 q_text 內的 % / _ 被當字面字元而非萬用字元。
            .map(|s| escape_ilike_wildcards(&s.chars().take(100).collect::<String>()));

        // R53-6: blacklist 過濾
        let blacklist: &[String] = audit_entity_blacklist();

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
              AND ($9::text IS NULL OR (
                    entity_display_name ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR actor_display_name  ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR actor_email         ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR event_type          ILIKE '%' || $9 || '%' ESCAPE '\'
                 OR ip_address::text    ILIKE '%' || $9 || '%' ESCAPE '\'
              ))
              AND (entity_type IS NULL OR entity_type <> ALL($10::text[]))
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
        .bind(&q_text)
        .bind(blacklist)
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
        // 安全關鍵三步（停用 session + 斷既發 access token + 撤 refresh token）須原子完成，
        // 否則部分失敗會留下「session 停用但 token 仍有效」的漏洞（重現本次修復的問題）。
        let mut tx = pool.begin().await?;

        // 停用該 session 並取回所屬 user_id（供撤 token 用）
        let user_id: Option<Uuid> = sqlx::query_scalar(
            r#"
            UPDATE user_sessions
            SET is_active = false,
                ended_at = NOW(),
                ended_reason = 'forced_logout'
            WHERE id = $1
            RETURNING user_id
            "#,
        )
        .bind(session_id)
        .fetch_optional(&mut *tx)
        .await?;

        // session 不存在 → 回 Forbidden（寫入端點統一以 403 遮蔽存在性，避免 existence oracle；
        // 亦避免留下「強制登出成功」的誤導 audit）。tx 隨錯誤 drop 自動 rollback。
        let uid =
            user_id.ok_or_else(|| crate::error::AppError::Forbidden("強制登出失敗".into()))?;

        // R82-5 安全修補：auth 中介層不查 `user_sessions.is_active`（只查 `users.is_active`
        // + `tokens_valid_after`），光把 session 標 inactive 踢不掉既有 access/refresh token
        // ——「強制登出」會失效（token 最長 7 天仍可用）。故連帶撤該 user 的 token 才真斷線：
        // 設 `tokens_valid_after=NOW()`（斷所有既發 access token）+ 撤未撤銷 refresh token。
        // 註：token 撤銷為 per-user（與 session_id 無對應欄位），會登出該使用者「所有」裝置
        // ——對「強制登出 / 疑遭盜用」屬安全且可接受的行為。
        sqlx::query("UPDATE users SET tokens_valid_after = NOW() WHERE id = $1")
            .bind(uid)
            .execute(&mut *tx)
            .await?;
        crate::services::AuthService::revoke_all_user_tokens_tx(&mut tx, uid).await?;

        tx.commit().await?;

        // 記錄審計日誌（legacy audit_logs 供舊 dashboard；鏈上 FORCE_LOGOUT 由 handler 的
        // log_activity_oneshot 寫入，本處不重複）。commit 後寫，非安全關鍵。
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
                .map_err(|e| {
                    tracing::error!("audit stats query failed: {e}");
                    e
                })?;

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
            entity_type: None,
            entity_id: None,
            extra_input: None,
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
                entity_type: input.entity_type,
                entity_id: input.entity_id,
                extra_input: input.extra_input,
            },
            HMAC_VERSION_LEGACY,
        )
        .expect("test key valid");
        let v2 = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            prev,
            input,
            HMAC_VERSION_CANONICAL,
        )
        .expect("test key valid");

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
                entity_type: case_a.entity_type,
                entity_id: case_a.entity_id,
                extra_input: case_a.extra_input,
            },
            HMAC_VERSION_LEGACY,
        )
        .expect("test key valid");
        let legacy_b = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            None,
            case_b,
            HMAC_VERSION_LEGACY,
        )
        .expect("test key valid");
        assert_eq!(
            legacy_a, legacy_b,
            "legacy 確實會碰撞（合預期，說明為何需 v2）"
        );

        // v2 canonical：不會碰撞
        let v2_a = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            None,
            case_a,
            HMAC_VERSION_CANONICAL,
        )
        .expect("test key valid");
        let input_b_v2 = sample_input("abc", "d", actor, &before, &after, &fields);
        let v2_b = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            None,
            input_b_v2,
            HMAC_VERSION_CANONICAL,
        )
        .expect("test key valid");
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
        )
        .expect("test key valid");
        let unknown = AuditService::compute_hmac_for_fields_versioned(TEST_KEY, None, input2, 99)
            .expect("test key valid");

        assert_eq!(v2, unknown, "未知版本應 fallback canonical，不應 panic");
    }

    #[test]
    fn v3_differs_from_v2_when_extra_fields_set() {
        // v3 編碼比 v2 多 length-prefix entity_type/entity_id/extra_input；
        // 同樣 input 用兩種版本算出的 HMAC 必須不同。
        use super::HMAC_VERSION_V3;
        let actor = Uuid::nil();
        let before = None::<serde_json::Value>;
        let after = None::<serde_json::Value>;
        let fields: Vec<String> = vec![];
        let mk = || HmacInput {
            event_category: "AUDIT",
            event_type: "SIGNATURE_CREATE",
            actor_user_id: actor,
            before_data: &before,
            after_data: &after,
            impersonated_by: None,
            changed_fields: &fields,
            entity_type: Some("electronic_signature"),
            entity_id: Some("00000000-0000-0000-0000-0000000000aa"),
            extra_input: Some("aa:deadbeef"),
        };
        let v2 = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            None,
            mk(),
            HMAC_VERSION_CANONICAL,
        )
        .expect("test key valid");
        let v3 =
            AuditService::compute_hmac_for_fields_versioned(TEST_KEY, None, mk(), HMAC_VERSION_V3)
                .expect("test key valid");
        assert_ne!(v2, v3, "v3 必須因納入 entity/extra_input 而與 v2 不同");
    }

    #[test]
    fn v3_extra_input_change_changes_hash() {
        // 同 sig_id 不同 binding（content_hash）→ chain hash 不同，
        // 證明 SIGNATURE_* 事件確實綁住簽章內容。
        use super::HMAC_VERSION_V3;
        let actor = Uuid::nil();
        let before = None::<serde_json::Value>;
        let after = None::<serde_json::Value>;
        let fields: Vec<String> = vec![];
        let mk = |xi: &'static str| HmacInput {
            event_category: "AUDIT",
            event_type: "SIGNATURE_CREATE",
            actor_user_id: actor,
            before_data: &before,
            after_data: &after,
            impersonated_by: None,
            changed_fields: &fields,
            entity_type: Some("electronic_signature"),
            entity_id: Some("00000000-0000-0000-0000-0000000000aa"),
            extra_input: Some(xi),
        };
        let h1 = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            None,
            mk("aa:111"),
            HMAC_VERSION_V3,
        )
        .expect("test key valid");
        let h2 = AuditService::compute_hmac_for_fields_versioned(
            TEST_KEY,
            None,
            mk("aa:222"),
            HMAC_VERSION_V3,
        )
        .expect("test key valid");
        assert_ne!(h1, h2, "extra_input 不同必須改變 v3 hash");
    }

    #[test]
    fn derive_v3_routing_picks_v3_only_for_signature_events() {
        let after = Some(serde_json::json!({
            "id": "00000000-0000-0000-0000-0000000000aa",
            "content_hash": "deadbeef"
        }));
        let (v, xi) =
            AuditService::derive_v3_routing("SIGNATURE_CREATE", &after).expect("complete payload");
        assert_eq!(v, super::HMAC_VERSION_V3);
        assert_eq!(
            xi.as_deref(),
            Some("00000000-0000-0000-0000-0000000000aa:deadbeef")
        );

        let (v2, xi2) = AuditService::derive_v3_routing("UPDATE", &after)
            .expect("non-signature event always Ok");
        assert_eq!(v2, super::HMAC_VERSION_CANONICAL);
        assert!(xi2.is_none());
    }

    #[test]
    fn derive_v3_routing_fails_loud_when_signature_missing_binding() {
        // 缺 binding 必須 fail loud — 不能寫出「v3 但無 fingerprint」的弱化 row。
        let after_no_id = Some(serde_json::json!({"content_hash": "deadbeef"}));
        AuditService::derive_v3_routing("SIGNATURE_CREATE", &after_no_id)
            .expect_err("缺 id 必須 Err");

        let after_no_binding = Some(serde_json::json!({
            "id": "00000000-0000-0000-0000-0000000000aa"
        }));
        AuditService::derive_v3_routing("SIGNATURE_CREATE", &after_no_binding)
            .expect_err("缺 content_hash/invalidated_reason 必須 Err");

        let after_none: Option<serde_json::Value> = None;
        AuditService::derive_v3_routing("SIGNATURE_INVALIDATED", &after_none)
            .expect_err("after_data 為 None 必須 Err");
    }
}
