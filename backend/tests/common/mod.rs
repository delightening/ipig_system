#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::Arc;

use reqwest::{Client, Response};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::PgPool;
use tokio::net::TcpListener;

pub mod test_db;

/// Reusable test application that spawns a real Axum server on a random port.
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub client: Client,
}

#[derive(Debug, serde::Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// 配一個**目前沒有任何「非終態」動物在用**的三位數耳號。
///
/// # 為什麼不能用亂數
///
/// 耳號受 `validate_ear_tag` 限制**只能是三位數**（`models/animal/requests.rs`），
/// 即整個系統只有 100–999 共 900 個值——這是真實業務規則，測試不能繞過。它是
/// **有限資源，該用「配置」而非「抽籤」**：多個測試檔在同一顆共用測試 DB 上抽
/// 三位數亂號，累積後必然發生生日碰撞（本函數取代的舊作法就是這樣間歇紅）。
///
/// 碰撞的後果是 409：`guard_duplicate_ear_tag`（`services/animal/core/write.rs`）
/// 擋「同耳號**且**同出生日期的存活動物」，且其 doc comment 明寫 `force_create`
/// 跳不過這條。舊測試不帶 `birth_date` → 全為 `NULL` → `None == None` 恆真 →
/// 複合鍵退化成單鍵，耳號一撞就失敗。
///
/// # 為什麼這樣就保證不撞
///
/// 本函數回傳的耳號在查詢當下**沒有任何非終態動物使用**，故守衛的 `existing`
/// 為空、直接放行——與 `birth_date` 給什麼無關。查詢條件與守衛本身完全一致
/// （`deleted_at IS NULL AND status NOT IN ('euthanized','sudden_death')`）。
///
/// `OFFSET` 帶 process 內遞增序號：同一個測試 binary 內平行執行的測試各自取到
/// **不同**的空號，不會在雙方都還沒 INSERT 前拿到同一個。測試 binary 之間 cargo
/// 是循序執行的，前一支的資料已 commit，故跨 binary 也安全。
pub async fn free_ear_tag(pool: &PgPool) -> String {
    use std::sync::atomic::{AtomicI64, Ordering};

    static SEQ: AtomicI64 = AtomicI64::new(0);
    let skip = SEQ.fetch_add(1, Ordering::Relaxed);

    let tag: Option<String> = sqlx::query_scalar(
        r#"
        SELECT lpad(g::text, 3, '0')
        FROM generate_series(100, 999) g
        WHERE NOT EXISTS (
            SELECT 1 FROM animals a
            WHERE a.ear_tag = lpad(g::text, 3, '0')
              AND a.deleted_at IS NULL
              AND a.status NOT IN ('euthanized', 'sudden_death')
        )
        ORDER BY g
        OFFSET $1
        LIMIT 1
        "#,
    )
    .bind(skip)
    .fetch_optional(pool)
    .await
    .expect("query free ear tag");

    // 用盡時明確報錯，好過讓呼叫端拿到神秘的 409。
    tag.expect("測試 DB 已無可用的三位數耳號（100–999 全被非終態動物佔用），請重建測試 DB")
}

impl TestApp {
    /// Spawn the full application on a random OS-assigned port.
    ///
    /// **Requires `TEST_DATABASE_URL`**（不 fallback 到 `DATABASE_URL`）指向一個
    /// 獨立的丟棄用 PostgreSQL；migration 由本函式自行執行。
    ///
    /// 開發機的 `DATABASE_URL` 指向 prod，fallback 會讓整合測試寫進正式庫，
    /// 見 CLAUDE.md「禁止在 prod 跑 backend 整合測試」。
    ///
    /// 光要求變數存在還不夠——它不檢查變數指向哪裡。實際的隔離判斷在
    /// [`test_db::connect_disposable`]：在 migration 與任何寫入之前驗證目標資料庫
    /// **本身**是丟棄用測試庫（標記表 + 核心業務表為空），不看它的名字。
    pub async fn spawn() -> Self {
        // R28-2：擴大 conn pool 至 15，讓 concurrent audit write 整合測試
        // 可跑 10 並發（原 max_connections=5 限制 → 只能跑 3 並發）。
        // 5 仍足以支援單請求 backend test，15 換取 concurrent test 強度。
        let pool = test_db::connect_disposable(15).await;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations on test database");

        // Minimal config for testing
        std::env::set_var("HOST", "127.0.0.1");
        std::env::set_var("PORT", "0"); // will be overridden by listener
        std::env::set_var("COOKIE_SECURE", "false");
        std::env::set_var("SEED_DEV_USERS", "false");
        std::env::set_var("DISABLE_CSRF_FOR_TESTS", "true");
        // R66-B2：固定測試用 AEAD 金鑰（base64 of 32 bytes，非機密）— 讓 2FA 加密路徑可跑。
        if std::env::var("ENCRYPTION_KEY").is_err() {
            std::env::set_var("ENCRYPTION_KEY", format!("{}=", "A".repeat(43)));
        }
        if std::env::var("JWT_EC_PRIVATE_KEY").is_err() {
            // 固定測試用 EC P-256 金鑰（來自 jsonwebtoken crate 官方測試金鑰，非機密）
            std::env::set_var(
                "JWT_EC_PRIVATE_KEY",
                "-----BEGIN PRIVATE KEY-----\n\
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgWTFfCGljY6aw3Hrt\n\
kHmPRiazukxPLb6ilpRAewjW8nihRANCAATDskChT+Altkm9X7MI69T3IUmrQU0L\n\
950IxEzvw/x5BMEINRMrXLBJhqzO9Bm+d6JbqA21YQmd1Kt4RzLJR1W+\n\
-----END PRIVATE KEY-----\n",
            );
            std::env::set_var(
                "JWT_EC_PUBLIC_KEY",
                "-----BEGIN PUBLIC KEY-----\n\
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEw7JAoU/gJbZJvV+zCOvU9yFJq0FN\n\
C/edCMRM78P8eQTBCDUTK1ywSYaszvQZvneiW6gNtWEJndSreEcyyUdVvg==\n\
-----END PUBLIC KEY-----\n",
            );
        }
        // 確保測試用 admin 密碼與 login_as_admin() 回退值一致
        if std::env::var("ADMIN_INITIAL_PASSWORD")
            .ok()
            .filter(|s| !s.is_empty())
            .is_none()
        {
            std::env::set_var("ADMIN_INITIAL_PASSWORD", "iPig$ecure1");
        }
        // 測試環境標記為 CI，避免 must_change_password = true 影響登入
        if std::env::var("CI").is_err() {
            std::env::set_var("CI", "1");
        }

        // 確保 uploads 目錄存在（health check 需要）
        let uploads_dir = std::path::Path::new("./uploads");
        if !uploads_dir.exists() {
            std::fs::create_dir_all(uploads_dir).expect("Failed to create uploads dir");
        }

        let config =
            erp_backend::config::Config::from_env().expect("Failed to build Config from env");
        let config = Arc::new(config);

        let geoip = erp_backend::services::GeoIpService::new("/nonexistent");
        let jwt_blacklist = erp_backend::middleware::JwtBlacklist::new();

        // R26-2: 初始化 HMAC key（audit chain 驗證測試需要）。
        // 優先順序：AUDIT_HMAC_KEY env > config.audit_hmac_key > 測試 fallback。
        // init_hmac_key 內部用 OnceLock::set，多次呼叫除第一次外都 no-op，
        // 並發安全。
        let test_hmac_key = std::env::var("AUDIT_HMAC_KEY")
            .ok()
            .filter(|s| s.len() >= 44) // 同步 config.rs::audit_hmac_key 最小長度
            .or_else(|| config.audit_hmac_key.clone())
            .unwrap_or_else(|| "test-hmac-key-do-not-use-in-prod-base64-padding".to_string());
        erp_backend::services::AuditService::init_hmac_key(Some(test_hmac_key));

        // Startup: ensure schema, admin, permissions
        // 測試環境使用 ensure_admin_user_after_import，強制將 admin 密碼重設為
        // 目前 ADMIN_INITIAL_PASSWORD 的值，避免 DB 中殘留舊密碼導致登入失敗
        let _ = erp_backend::startup::ensure_schema(&pool).await;
        let _ = erp_backend::startup::ensure_admin_user_after_import(&pool, &config).await;
        let _ = erp_backend::startup::ensure_required_permissions(&pool).await;
        let _ = erp_backend::startup::ensure_all_role_permissions(&pool).await;

        let pdf_service = erp_backend::PdfServiceClient::new("http://localhost:9210", "test-token");

        // R28-M5：TestApp 也提供 PrometheusHandle 讓 /api/health metrics
        // 健檢回 "up"。install_recorder 是 process-global 一次性，跨多個
        // TestApp::spawn() 呼叫須用 OnceLock 共享。
        use std::sync::OnceLock;
        static METRICS_HANDLE: OnceLock<metrics_exporter_prometheus::PrometheusHandle> =
            OnceLock::new();
        let metrics_handle = METRICS_HANDLE
            .get_or_init(|| {
                metrics_exporter_prometheus::PrometheusBuilder::new()
                    .install_recorder()
                    .expect("install Prometheus recorder for tests")
            })
            .clone();

        let state = erp_backend::AppState {
            db: pool.clone(),
            config,
            geoip,
            jwt_blacklist,
            metrics_handle: Some(metrics_handle),
            pdf_service,
            permission_cache: erp_backend::build_permission_cache(),
            shutdown_token: tokio_util::sync::CancellationToken::new(),
        };

        let app = erp_backend::routes::api_routes(state);

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind random port");
        let addr: SocketAddr = listener
            .local_addr()
            .expect("Failed to get listener address");

        tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await
            .expect("Failed to start Axum server");
        });

        let client = Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Failed to build HTTP client");

        TestApp {
            address: format!("http://127.0.0.1:{}", addr.port()),
            db_pool: pool,
            client,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }

    /// Login with email/password and return the access token.
    pub async fn login(&self, email: &str, password: &str) -> Option<String> {
        let res = self
            .client
            .post(self.url("/api/v1/auth/login"))
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await
            .ok()?;

        if !res.status().is_success() {
            return None;
        }

        let body: LoginResponse = res.json().await.ok()?;
        Some(body.access_token)
    }

    /// Login as admin (uses ADMIN_EMAIL / ADMIN_INITIAL_PASSWORD from env or defaults).
    pub async fn login_as_admin(&self) -> String {
        let email = std::env::var("ADMIN_EMAIL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "admin@ipigsystem.asia".to_string());
        let password = std::env::var("ADMIN_INITIAL_PASSWORD")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "iPig$ecure1".to_string());

        self.login(&email, &password)
            .await
            .expect("Admin login failed — check ADMIN_EMAIL and ADMIN_INITIAL_PASSWORD env vars")
    }

    /// Authenticated GET request.
    pub async fn auth_get(&self, path: &str, token: &str) -> Response {
        self.client
            .get(self.url(path))
            .bearer_auth(token)
            .send()
            .await
            .expect("Request failed")
    }

    /// Authenticated POST with JSON body.
    pub async fn auth_post<T: Serialize>(&self, path: &str, body: &T, token: &str) -> Response {
        self.client
            .post(self.url(path))
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .expect("Request failed")
    }

    /// Authenticated PUT with JSON body.
    pub async fn auth_put<T: Serialize>(&self, path: &str, body: &T, token: &str) -> Response {
        self.client
            .put(self.url(path))
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .expect("Request failed")
    }

    /// Authenticated DELETE request.
    pub async fn auth_delete(&self, path: &str, token: &str) -> Response {
        self.client
            .delete(self.url(path))
            .bearer_auth(token)
            .send()
            .await
            .expect("Request failed")
    }

    /// Parse JSON response body.
    pub async fn json<T: DeserializeOwned>(response: Response) -> T {
        response.json::<T>().await.expect("Failed to parse JSON")
    }

    /// 取 PI role 的 UUID — `CreateInvitationRequest.role_ids` required 後測試 fixture 必填。
    /// 假設 `ensure_required_permissions` 已建立 PI role（從 spawn 路徑保證）。
    pub async fn pi_role_id(&self) -> uuid::Uuid {
        sqlx::query_scalar::<_, uuid::Uuid>(
            "SELECT id FROM roles WHERE code = 'PI' AND is_active = true LIMIT 1",
        )
        .fetch_one(&self.db_pool)
        .await
        .expect("PI role must exist after startup migrations")
    }
}
