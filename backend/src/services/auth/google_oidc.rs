//! R121：Google 帳號登入（OpenID Connect，Authorization Code ＋ PKCE）。
//!
//! 流程（HTTP 端在 `handlers/auth/google.rs`）：
//! 1. `GET /auth/google/start`：[`AuthService::begin_google_login`] 產生 state／nonce／
//!    PKCE verifier，封進有簽章的短效 cookie，並回傳要 302 過去的 Google 授權網址。
//! 2. `GET /auth/google/callback`：[`AuthService::open_google_flow_state`] 取回 cookie 內容並比對
//!    state → [`AuthService::exchange_google_code`] 換 ID token →
//!    [`AuthService::verify_google_id_token`] 驗 claims →
//!    [`AuthService::resolve_google_user`] 找出（必要時自動綁定）系統帳號。
//!
//! 綁定規則（使用者 2026-09-23 裁定，見 `docs/TODO.md` R121）：
//! - 只開放既有帳號，不會因 Google 登入而新增帳號。
//! - 已綁定者以 Google `sub` 查找；未綁定者若 `email_verified` 且 email 與恰好一個在職帳號
//!   相同（不分大小寫）就自動綁定。理由：帳號為邀請制，且系統本來就能以「忘記密碼」寄信
//!   重設密碼——能控制該信箱的人本來就能接管帳號，以 email 綁定沒有擴大攻擊面。
//! - 密碼登入保留，本模組不碰 `password_hash`，也不寫 `login_failure`（那是密碼暴力破解的
//!   計數來源，Google 端的拒絕不應讓使用者的密碼登入被鎖）。
//!
//! 為什麼不驗 ID token 的簽章：ID token 是後端帶著 client secret、直接經 TLS 向 Google token
//! endpoint 換來的，不經過瀏覽器。OIDC Core §3.1.3.7 第 6 點允許此情境以 TLS 伺服器驗證
//! 取代簽章驗證；`iss`／`aud`／`exp`／`nonce` 仍逐一檢查。這讓本功能不必抓 JWKS、也不需要
//! 新增任何相依套件。
//! ⚠️ 若日後改成讓瀏覽器把 ID token 送上來（例如改用 Google 的 JS 登入按鈕），
//! 這個前提就不成立，**必須**補上簽章驗證。

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use hmac::{Hmac, KeyInit, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    config::GoogleOAuthConfig,
    constants::GUEST_DEMO_EMAIL,
    middleware::{ActorContext, CurrentUser, SYSTEM_USER_ID},
    models::{
        audit_diff::{AuditRedact, DataDiff},
        User,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity, RequestContext},
        AuditService,
    },
    AppError, Result,
};

use super::AuthService;

type HmacSha256 = Hmac<Sha256>;

/// `user_identities.provider` 的值（migration 016 的 CHECK 約束同步只允許這個值）。
pub const GOOGLE_PROVIDER: &str = "google";
/// 存放 state／nonce／PKCE verifier 的 cookie 名稱。
pub const GOOGLE_FLOW_COOKIE: &str = "google_oidc_flow";
/// flow cookie 只需要在 start → callback 之間送出，Path 限縮到這一段。
pub const GOOGLE_FLOW_COOKIE_PATH: &str = "/api/v1/auth/google";
/// 從按下按鈕到 Google 導回來的時限。含 Google 帳號選擇與可能的 Google 端 2FA，給 10 分鐘。
pub const GOOGLE_FLOW_TTL_SECS: i64 = 600;

// 授權頁／token endpoint／合法 `iss` 由 `GoogleOAuthConfig::endpoints` 提供
// （正式一律是真 Google；本機模擬才會指向假 Google，限制見 `config/google_oauth.rs`）。
/// 只要身分與 email，不要任何 Google API 權限。
const GOOGLE_SCOPES: &str = "openid email";
/// flow cookie 的簽章金鑰由 CSRF_SECRET 以此標籤派生，與 CSRF token 本身的用途隔開。
const FLOW_KEY_LABEL: &[u8] = b"ipig/google-oidc-flow/v1";
/// ID token 時間欄位容許的時鐘誤差。
const CLOCK_SKEW_SECS: i64 = 60;
const TOKEN_ENDPOINT_TIMEOUT_SECS: u64 = 10;
/// 與 `user_identities.subject` 欄寬一致。
const MAX_SUBJECT_LEN: usize = 255;

/// Google 登入被拒的原因。[`Self::code`] 會出現在導回登入頁的網址上，前端依此顯示訊息，
/// 所以措辭只到「哪一類問題」，不帶任何帳號細節。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoogleLoginFailure {
    /// 伺服器沒有設定 Google OAuth（`Config::google_oauth` 為 None）。
    NotConfigured,
    /// 使用者在 Google 畫面按了取消。
    Cancelled,
    /// flow cookie 遺失／過期／被竄改，或 state 對不上（可能是 CSRF，也可能只是逾時）。
    InvalidState,
    /// 與 Google 交換 token 失敗，或 ID token 內容不合格。
    ProviderError,
    /// Google 回報 email 未驗證——不可據此綁定帳號。
    EmailNotVerified,
    /// 沒有對應的在職帳號（系統只開放既有帳號）。
    NoAccount,
    /// 同一個 email（不分大小寫）對到不只一個帳號，不猜，交給管理員處理。
    AmbiguousAccount,
    /// 帳號已停用或已過期。
    AccountUnavailable,
    /// 該帳號已綁定另一個 Google 帳號。
    IdentityConflict,
}

impl GoogleLoginFailure {
    pub fn code(self) -> &'static str {
        match self {
            Self::NotConfigured => "not_configured",
            Self::Cancelled => "cancelled",
            Self::InvalidState => "invalid_state",
            Self::ProviderError => "provider_error",
            Self::EmailNotVerified => "email_not_verified",
            Self::NoAccount => "no_account",
            Self::AmbiguousAccount => "ambiguous_account",
            Self::AccountUnavailable => "account_unavailable",
            Self::IdentityConflict => "identity_conflict",
        }
    }

    /// 是否寫進 `LOGIN_FAILED` 安全稽核（HMAC 鏈）。
    ///
    /// 與密碼登入一致：驗證失敗都要留稽核——`InvalidState` 尤其是 CSRF／重放的訊號，
    /// 只留在應用程式 log 等於稽核查詢看不到。例外只有兩種不是安全事件的情況：
    /// 使用者自己在 Google 畫面按取消、伺服器根本沒開這個功能。
    pub fn is_security_relevant(self) -> bool {
        !matches!(self, Self::Cancelled | Self::NotConfigured)
    }
}

/// [`AuthService::resolve_google_user`] 的結果。外層 `Result` 是基礎設施錯誤（DB），
/// 這裡的 `Rejected` 是業務上的拒絕，兩者分開，handler 才能對後者導回登入頁顯示訊息、
/// 對前者照常回 500。
pub enum GoogleAuthResult {
    Authenticated(Box<User>),
    Rejected(GoogleLoginFailure),
}

/// 封進 flow cookie 的內容。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoogleFlowState {
    pub state: String,
    pub nonce: String,
    pub code_verifier: String,
    /// Unix 秒。
    pub exp: i64,
}

/// [`AuthService::begin_google_login`] 的輸出。
pub struct GoogleLoginStart {
    pub authorization_url: String,
    pub flow_cookie_value: String,
}

/// 通過 claims 驗證的 Google 身分。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedGoogleIdentity {
    /// Google 的永久帳號 ID（`sub`）。
    pub subject: String,
    /// 已驗證的 email。
    pub email: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Audience {
    One(String),
    Many(Vec<String>),
}

/// Google 的 ID token 用布林，但部分端點（tokeninfo）用字串 "true"，兩種都接受。
#[derive(Deserialize)]
#[serde(untagged)]
enum LenientBool {
    Bool(bool),
    Text(String),
}

impl LenientBool {
    fn is_true(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Text(s) => s.eq_ignore_ascii_case("true"),
        }
    }
}

#[derive(Deserialize)]
struct GoogleIdTokenClaims {
    iss: String,
    aud: Audience,
    azp: Option<String>,
    sub: String,
    exp: i64,
    iat: Option<i64>,
    nonce: Option<String>,
    email: Option<String>,
    email_verified: Option<LenientBool>,
}

#[derive(Deserialize)]
struct TokenEndpointResponse {
    id_token: Option<String>,
}

#[derive(Deserialize)]
struct TokenEndpointError {
    error: Option<String>,
}

/// 建立綁定時寫進稽核的內容。
#[derive(Serialize)]
struct IdentityLinkAudit<'a> {
    provider: &'a str,
    subject: &'a str,
    email_at_link: &'a str,
}

impl AuditRedact for IdentityLinkAudit<'_> {}

/// 登入成功／失敗稽核附帶的資訊（`LOGIN_SUCCESS` 與密碼登入共用事件名，靠 `method` 區分）。
#[derive(Serialize)]
struct GoogleLoginAudit<'a> {
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'a str>,
}

impl AuditRedact for GoogleLoginAudit<'_> {}

/// 32 bytes 隨機值的 base64url（43 字元），同時符合 RFC 7636 對 PKCE verifier 的長度與字元集要求。
fn random_url_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// RFC 7636 S256：`BASE64URL(SHA256(verifier))`。
fn pkce_challenge(code_verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

/// 固定時間比較，避免以回應時間逐字元猜出 state／nonce。
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn flow_mac(csrf_secret: &str) -> Result<HmacSha256> {
    let mut deriver = HmacSha256::new_from_slice(csrf_secret.as_bytes())
        .map_err(|e| AppError::Internal(format!("HMAC 初始化失敗: {e}")))?;
    deriver.update(FLOW_KEY_LABEL);
    let flow_key = deriver.finalize().into_bytes();
    HmacSha256::new_from_slice(&flow_key)
        .map_err(|e| AppError::Internal(format!("HMAC 初始化失敗: {e}")))
}

fn http_client() -> Result<&'static reqwest::Client> {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    if let Some(client) = CLIENT.get() {
        return Ok(client);
    }
    let built = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(TOKEN_ENDPOINT_TIMEOUT_SECS))
        // token endpoint 不該轉址；照著轉址等於把 client secret 送去別處
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| AppError::Internal(format!("建立 Google HTTP client 失敗: {e}")))?;
    Ok(CLIENT.get_or_init(|| built))
}

/// 帳號狀態是否允許登入；回傳 `Some` 即拒絕原因。
fn account_rejection(user: &User, now: chrono::DateTime<Utc>) -> Option<GoogleLoginFailure> {
    if !user.is_active {
        return Some(GoogleLoginFailure::AccountUnavailable);
    }
    if user.expires_at.is_some_and(|expires_at| expires_at < now) {
        return Some(GoogleLoginFailure::AccountUnavailable);
    }
    None
}

/// 不開放 Google 登入的特殊帳號：系統 actor 與公開試用帳號。
fn is_reserved_account(user: &User) -> bool {
    user.id == SYSTEM_USER_ID || user.email.eq_ignore_ascii_case(GUEST_DEMO_EMAIL)
}

impl AuthService {
    /// 產生一次 Google 登入所需的 state／nonce／PKCE，回傳授權網址與要設進 cookie 的值。
    pub fn begin_google_login(
        google: &GoogleOAuthConfig,
        csrf_secret: &str,
        now: i64,
    ) -> Result<GoogleLoginStart> {
        let flow = GoogleFlowState {
            state: random_url_token(),
            nonce: random_url_token(),
            code_verifier: random_url_token(),
            exp: now + GOOGLE_FLOW_TTL_SECS,
        };

        let mut url = url::Url::parse(&google.endpoints.authorization)
            .map_err(|e| AppError::Internal(format!("Google 授權端點網址無效: {e}")))?;
        url.query_pairs_mut()
            .append_pair("client_id", &google.client_id)
            .append_pair("redirect_uri", &google.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", GOOGLE_SCOPES)
            .append_pair("state", &flow.state)
            .append_pair("nonce", &flow.nonce)
            .append_pair("code_challenge", &pkce_challenge(&flow.code_verifier))
            .append_pair("code_challenge_method", "S256")
            // 每次都讓使用者選帳號，避免瀏覽器登著別人的 Google 帳號時靜默帶入
            .append_pair("prompt", "select_account");

        Ok(GoogleLoginStart {
            authorization_url: url.into(),
            flow_cookie_value: Self::seal_google_flow_state(csrf_secret, &flow)?,
        })
    }

    /// 把 flow state 編成 `{payload}.{tag}`（皆 base64url），tag 為 HMAC-SHA256。
    pub fn seal_google_flow_state(csrf_secret: &str, flow: &GoogleFlowState) -> Result<String> {
        let payload = serde_json::to_vec(flow)
            .map_err(|e| AppError::Internal(format!("flow state 序列化失敗: {e}")))?;
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload);
        let mut mac = flow_mac(csrf_secret)?;
        mac.update(payload_b64.as_bytes());
        let tag_b64 = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        Ok(format!("{payload_b64}.{tag_b64}"))
    }

    /// 驗簽並解出 flow state；簽章不符、格式錯誤或已過期一律回 `None`
    /// （呼叫端不需要區分，全部都是「請重新按一次登入」）。
    pub fn open_google_flow_state(
        csrf_secret: &str,
        raw: &str,
        now: i64,
    ) -> Option<GoogleFlowState> {
        let (payload_b64, tag_b64) = raw.split_once('.')?;
        let tag = URL_SAFE_NO_PAD.decode(tag_b64).ok()?;
        let mut mac = flow_mac(csrf_secret).ok()?;
        mac.update(payload_b64.as_bytes());
        mac.verify_slice(&tag).ok()?;

        let payload = URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
        let flow: GoogleFlowState = serde_json::from_slice(&payload).ok()?;
        (flow.exp > now).then_some(flow)
    }

    /// callback 帶回的 state 是否與 cookie 內的相符。
    pub fn google_state_matches(flow: &GoogleFlowState, returned_state: &str) -> bool {
        constant_time_eq(flow.state.as_bytes(), returned_state.as_bytes())
    }

    /// 以授權碼 ＋ PKCE verifier 向 Google 換 ID token。
    ///
    /// 任何失敗都歸為 [`GoogleLoginFailure::ProviderError`]；細節只寫 log，不回給瀏覽器。
    /// Google 的錯誤回應只記 `error` 代碼欄位，不記整段 body。
    pub async fn exchange_google_code(
        google: &GoogleOAuthConfig,
        code: &str,
        code_verifier: &str,
    ) -> std::result::Result<String, GoogleLoginFailure> {
        let body = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("grant_type", "authorization_code")
            .append_pair("code", code)
            .append_pair("redirect_uri", &google.redirect_uri)
            .append_pair("client_id", &google.client_id)
            .append_pair("client_secret", &google.client_secret)
            .append_pair("code_verifier", code_verifier)
            .finish();

        let client = http_client().map_err(|e| {
            tracing::error!("[Google OIDC] {e}");
            GoogleLoginFailure::ProviderError
        })?;
        let response = client
            .post(&google.endpoints.token)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body)
            .send()
            .await
            .map_err(|e| {
                tracing::warn!("[Google OIDC] 呼叫 token endpoint 失敗: {e}");
                GoogleLoginFailure::ProviderError
            })?;

        let status = response.status();
        let text = response.text().await.map_err(|e| {
            tracing::warn!("[Google OIDC] 讀取 token endpoint 回應失敗: {e}");
            GoogleLoginFailure::ProviderError
        })?;

        if !status.is_success() {
            let error_code = serde_json::from_str::<TokenEndpointError>(&text)
                .ok()
                .and_then(|e| e.error);
            tracing::warn!(
                status = %status,
                error = error_code.as_deref().unwrap_or("<無>"),
                "[Google OIDC] token endpoint 拒絕交換授權碼"
            );
            return Err(GoogleLoginFailure::ProviderError);
        }

        serde_json::from_str::<TokenEndpointResponse>(&text)
            .ok()
            .and_then(|r| r.id_token)
            .filter(|t| !t.is_empty())
            .ok_or_else(|| {
                tracing::warn!("[Google OIDC] token endpoint 回應缺少 id_token");
                GoogleLoginFailure::ProviderError
            })
    }

    /// 驗證 ID token 的 claims（簽章不驗的理由見模組文件）。
    pub fn verify_google_id_token(
        id_token: &str,
        google: &GoogleOAuthConfig,
        expected_nonce: &str,
        now: i64,
    ) -> std::result::Result<VerifiedGoogleIdentity, GoogleLoginFailure> {
        let client_id = google.client_id.as_str();
        let reject = |why: &str| {
            tracing::warn!("[Google OIDC] ID token 不合格：{why}");
            GoogleLoginFailure::ProviderError
        };

        let mut parts = id_token.split('.');
        let (Some(_header), Some(payload_b64), Some(_signature), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(reject("不是三段式 JWT"));
        };
        let payload = URL_SAFE_NO_PAD
            .decode(payload_b64.trim_end_matches('='))
            .map_err(|_| reject("payload 不是 base64url"))?;
        let claims: GoogleIdTokenClaims =
            serde_json::from_slice(&payload).map_err(|_| reject("payload 欄位不符"))?;

        if !google.endpoints.issuers.contains(&claims.iss) {
            return Err(reject("iss 不在允許清單"));
        }
        let audience_ok = match &claims.aud {
            Audience::One(aud) => aud == client_id,
            // 多個 audience 時，OIDC 要求 azp 必須是自己
            Audience::Many(auds) => {
                auds.iter().any(|a| a == client_id) && claims.azp.as_deref() == Some(client_id)
            }
        };
        if !audience_ok {
            return Err(reject("aud 不是本系統的 client_id"));
        }
        if claims.exp + CLOCK_SKEW_SECS <= now {
            return Err(reject("已過期"));
        }
        if claims.iat.is_some_and(|iat| iat > now + CLOCK_SKEW_SECS) {
            return Err(reject("iat 在未來"));
        }
        let nonce_ok = claims
            .nonce
            .as_deref()
            .is_some_and(|n| constant_time_eq(n.as_bytes(), expected_nonce.as_bytes()));
        if !nonce_ok {
            return Err(reject("nonce 不符"));
        }
        let subject = claims.sub.trim();
        if subject.is_empty() || subject.len() > MAX_SUBJECT_LEN {
            return Err(reject("sub 空白或過長"));
        }

        let email = claims
            .email
            .as_deref()
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .ok_or_else(|| reject("缺少 email（scope 應含 email）"))?;
        if !claims
            .email_verified
            .as_ref()
            .is_some_and(LenientBool::is_true)
        {
            return Err(GoogleLoginFailure::EmailNotVerified);
        }

        Ok(VerifiedGoogleIdentity {
            subject: subject.to_string(),
            email: email.to_string(),
        })
    }

    /// 由已驗證的 Google 身分找出系統帳號；尚未綁定時依 email 自動綁定。
    pub async fn resolve_google_user(
        pool: &PgPool,
        identity: &VerifiedGoogleIdentity,
        request_context: RequestContext<'_>,
    ) -> Result<GoogleAuthResult> {
        let now = Utc::now();
        let mut tx = pool.begin().await?;

        if let Some(user) = Self::find_user_by_google_subject(&mut tx, &identity.subject).await? {
            if let Some(failure) = account_rejection(&user, now) {
                return Ok(GoogleAuthResult::Rejected(failure));
            }
            sqlx::query(
                "UPDATE user_identities SET last_login_at = NOW() \
                 WHERE provider = $1 AND subject = $2",
            )
            .bind(GOOGLE_PROVIDER)
            .bind(&identity.subject)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            return Ok(GoogleAuthResult::Authenticated(Box::new(user)));
        }

        let user = match Self::find_link_candidate(&mut tx, &identity.email, now).await? {
            Ok(user) => user,
            Err(failure) => return Ok(GoogleAuthResult::Rejected(failure)),
        };

        let inserted: Option<uuid::Uuid> = sqlx::query_scalar(
            r#"INSERT INTO user_identities (user_id, provider, subject, email_at_link, last_login_at)
               VALUES ($1, $2, $3, $4, NOW())
               ON CONFLICT DO NOTHING
               RETURNING id"#,
        )
        .bind(user.id)
        .bind(GOOGLE_PROVIDER)
        .bind(&identity.subject)
        .bind(&identity.email)
        .fetch_optional(&mut *tx)
        .await?;

        if inserted.is_none() {
            // 兩個 callback 並行（例如使用者連點）時，另一邊已先寫入。
            // 先放掉本交易，再以 sub 重查一次：寫入的正是同一個帳號就放行，否則視為衝突。
            tx.rollback().await?;
            return Self::resolve_after_link_race(pool, identity, user.id, now).await;
        }

        let actor = ActorContext::User(CurrentUser {
            id: user.id,
            email: user.email.clone(),
            roles: vec![],
            permissions: vec![],
            jti: String::new(),
            exp: 0,
            impersonated_by: None,
        });
        AuditService::log_activity_tx(
            &mut tx,
            &actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "EXTERNAL_IDENTITY_LINKED",
                entity: Some(AuditEntity::new("user", user.id, &user.display_name)),
                data_diff: Some(DataDiff::create_only(&IdentityLinkAudit {
                    provider: GOOGLE_PROVIDER,
                    subject: &identity.subject,
                    email_at_link: &identity.email,
                })),
                request_context: Some(request_context),
            },
        )
        .await?;
        tx.commit().await?;

        tracing::info!(
            user_id = %user.id,
            "[Google OIDC] 依 email 自動綁定 Google 帳號"
        );
        Ok(GoogleAuthResult::Authenticated(Box::new(user)))
    }

    /// `LOGIN_SUCCESS` 稽核附帶的 diff，標明登入方式是 Google。
    pub fn google_login_success_audit() -> DataDiff {
        DataDiff::create_only(&GoogleLoginAudit {
            method: GOOGLE_PROVIDER,
            reason: None,
        })
    }

    /// `LOGIN_FAILED` 稽核附帶的 diff，標明登入方式與被拒原因代碼。
    pub fn google_login_failure_audit(failure: GoogleLoginFailure) -> DataDiff {
        DataDiff::create_only(&GoogleLoginAudit {
            method: GOOGLE_PROVIDER,
            reason: Some(failure.code()),
        })
    }

    async fn find_user_by_google_subject(
        tx: &mut Transaction<'_, Postgres>,
        subject: &str,
    ) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT u.* FROM user_identities ui
               JOIN users u ON u.id = ui.user_id
               WHERE ui.provider = $1 AND ui.subject = $2 AND u.deleted_at IS NULL"#,
        )
        .bind(GOOGLE_PROVIDER)
        .bind(subject)
        .fetch_optional(&mut **tx)
        .await?;
        Ok(user)
    }

    /// 找出可以自動綁定的帳號；內層 `Err` 是業務上的拒絕原因。
    async fn find_link_candidate(
        tx: &mut Transaction<'_, Postgres>,
        email: &str,
        now: chrono::DateTime<Utc>,
    ) -> Result<std::result::Result<User, GoogleLoginFailure>> {
        // users.email 的唯一約束分大小寫，而 Google 回的 email 大小寫不一定與建帳時相同，
        // 所以比對用 lower()。代價是理論上可能對到多筆，那種情況交給管理員，不猜。
        let mut candidates = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE lower(email) = lower($1) AND deleted_at IS NULL",
        )
        .bind(email)
        .fetch_all(&mut **tx)
        .await?;

        if candidates.len() > 1 {
            tracing::error!(
                count = candidates.len(),
                "[Google OIDC] 同一 email（不分大小寫）對到多個帳號，拒絕自動綁定"
            );
            return Ok(Err(GoogleLoginFailure::AmbiguousAccount));
        }
        let Some(user) = candidates.pop() else {
            return Ok(Err(GoogleLoginFailure::NoAccount));
        };
        if is_reserved_account(&user) {
            return Ok(Err(GoogleLoginFailure::NoAccount));
        }
        if let Some(failure) = account_rejection(&user, now) {
            return Ok(Err(failure));
        }

        let already_linked: Option<String> = sqlx::query_scalar(
            "SELECT subject FROM user_identities WHERE user_id = $1 AND provider = $2",
        )
        .bind(user.id)
        .bind(GOOGLE_PROVIDER)
        .fetch_optional(&mut **tx)
        .await?;
        if already_linked.is_some() {
            // 帳號已綁了另一個 Google 帳號：不覆寫、不疊加，要換綁須由管理員處理。
            tracing::warn!(
                user_id = %user.id,
                "[Google OIDC] 帳號已綁定其他 Google 帳號，拒絕以 email 再綁一個"
            );
            return Ok(Err(GoogleLoginFailure::IdentityConflict));
        }

        Ok(Ok(user))
    }

    async fn resolve_after_link_race(
        pool: &PgPool,
        identity: &VerifiedGoogleIdentity,
        expected_user_id: uuid::Uuid,
        now: chrono::DateTime<Utc>,
    ) -> Result<GoogleAuthResult> {
        let mut tx = pool.begin().await?;
        let found = Self::find_user_by_google_subject(&mut tx, &identity.subject).await?;
        tx.commit().await?;

        match found {
            Some(user) if user.id == expected_user_id => match account_rejection(&user, now) {
                Some(failure) => Ok(GoogleAuthResult::Rejected(failure)),
                None => Ok(GoogleAuthResult::Authenticated(Box::new(user))),
            },
            _ => Ok(GoogleAuthResult::Rejected(
                GoogleLoginFailure::IdentityConflict,
            )),
        }
    }
}

#[cfg(test)]
mod tests;
