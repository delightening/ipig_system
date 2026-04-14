#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::Arc;

use reqwest::{Client, Response};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::PgPool;
use tokio::net::TcpListener;

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

impl TestApp {
    /// Spawn the full application on a random OS-assigned port.
    ///
    /// Requires `TEST_DATABASE_URL` env var (or `DATABASE_URL`) pointing to a
    /// test-ready PostgreSQL instance with migrations already applied.
    pub async fn spawn() -> Self {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .expect("TEST_DATABASE_URL or DATABASE_URL must be set for integration tests");

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

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

        let config = erp_backend::config::Config::from_env()
            .expect("Failed to build Config from env");
        let config = Arc::new(config);

        let geoip = erp_backend::services::GeoIpService::new("/nonexistent");
        let jwt_blacklist = erp_backend::middleware::JwtBlacklist::new();

        // Startup: ensure schema, admin, permissions
        // 測試環境使用 ensure_admin_user_after_import，強制將 admin 密碼重設為
        // 目前 ADMIN_INITIAL_PASSWORD 的值，避免 DB 中殘留舊密碼導致登入失敗
        let _ = erp_backend::startup::ensure_schema(&pool).await;
        let _ = erp_backend::startup::ensure_admin_user_after_import(&pool, &config).await;
        let _ = erp_backend::startup::ensure_required_permissions(&pool).await;
        let _ = erp_backend::startup::ensure_all_role_permissions(&pool).await;

        let gotenberg = erp_backend::GotenbergClient::new("http://localhost:3000");
        let image_processor = erp_backend::ImageProcessorClient::new("http://localhost:3100");
        let templates = erp_backend::TemplateService::new()
            .unwrap_or_else(|_| {
                // 測試環境中模板目錄可能不存在，使用空模板
                erp_backend::TemplateService::empty()
            });

        let state = erp_backend::AppState {
            db: pool.clone(),
            config,
            geoip,
            jwt_blacklist,
            metrics_handle: None,
            gotenberg,
            image_processor,
            templates,
            permission_cache: std::sync::Arc::new(dashmap::DashMap::new()),
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
}
