//! R121：Google 帳號登入（OIDC）的設定。
//!
//! 環境變數：
//! - `GOOGLE_OAUTH_CLIENT_ID`／`GOOGLE_OAUTH_CLIENT_SECRET`（或 `_FILE`）／`GOOGLE_OAUTH_REDIRECT_URI`：
//!   三者全設＝開啟，全不設＝關閉，只設一部分＝啟動失敗。
//! - `GOOGLE_OAUTH_DEV_MOCK_BASE_URL`（僅供本機模擬）：把授權頁與 token endpoint 改指向本機的
//!   假 Google（`scripts/dev/mock-google-oidc.mjs`），在沒有真 Google 憑證時跑完整登入流程。
//!
//! 🔴 假 Google 開關的三道限制（任一不符即**啟動失敗**，不是忽略）：
//! 1. `COOKIE_SECURE=true`（正式環境）時不准設——token 交換會把 client secret 送過去，
//!    正式機被誤設等於把 secret 交給別人；
//! 2. 位址只能是 `localhost`／`127.0.0.1`——假服務必須在同一台機器上；
//! 3. 設了就會在啟動 log 印出醒目警告。

use anyhow::Context;

/// Google 真正的授權頁與 token endpoint。
const GOOGLE_AUTHORIZATION_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
/// Google 文件載明 ID token 的 `iss` 可能是這兩種寫法之一。
const GOOGLE_ISSUERS: [&str; 2] = ["https://accounts.google.com", "accounts.google.com"];

/// 假 Google 的固定路徑（與 `scripts/dev/mock-google-oidc.mjs` 一致）。
const MOCK_AUTHORIZATION_PATH: &str = "/o/oauth2/v2/auth";
const MOCK_TOKEN_PATH: &str = "/token";

/// 身分提供者的端點。正式一律是 [`GoogleEndpoints::google`]；
/// 只有本機模擬（見模組文件）才會是假 Google。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoogleEndpoints {
    /// 瀏覽器被導去的授權頁
    pub authorization: String,
    /// 後端換 token 的位址（會帶 client secret）
    pub token: String,
    /// ID token `iss` 的合法值
    pub issuers: Vec<String>,
}

impl GoogleEndpoints {
    pub fn google() -> Self {
        Self {
            authorization: GOOGLE_AUTHORIZATION_ENDPOINT.to_string(),
            token: GOOGLE_TOKEN_ENDPOINT.to_string(),
            issuers: GOOGLE_ISSUERS.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn dev_mock(base: &str) -> Self {
        let base = base.trim_end_matches('/');
        Self {
            authorization: format!("{base}{MOCK_AUTHORIZATION_PATH}"),
            token: format!("{base}{MOCK_TOKEN_PATH}"),
            issuers: vec![base.to_string()],
        }
    }

    pub fn is_dev_mock(&self) -> bool {
        self.token != GOOGLE_TOKEN_ENDPOINT
    }
}

/// Google 登入設定。
///
/// `client_secret` 不可出現在 log，故手動實作 `Debug`（`Config` 整體 derive `Debug`）。
#[derive(Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    /// 在 Google Cloud Console 登記的 Authorized redirect URI，須與實際 callback 路徑逐字相同。
    pub redirect_uri: String,
    pub endpoints: GoogleEndpoints,
}

impl std::fmt::Debug for GoogleOAuthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleOAuthConfig")
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("redirect_uri", &self.redirect_uri)
            .field("endpoints", &self.endpoints)
            .finish()
    }
}

/// 從環境變數讀出的原始值（全部已 trim、空字串視為未設定）。
#[derive(Default)]
pub(super) struct GoogleOAuthEnv {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub dev_mock_base_url: Option<String>,
}

fn is_local_host(url: &url::Url) -> bool {
    matches!(url.host_str(), Some("localhost") | Some("127.0.0.1"))
}

/// 由原始值組出設定；規則見模組文件。
pub(super) fn google_oauth_config_from_env(
    env: GoogleOAuthEnv,
    cookie_secure: bool,
) -> anyhow::Result<Option<GoogleOAuthConfig>> {
    let (client_id, client_secret, redirect_uri) =
        match (env.client_id, env.client_secret, env.redirect_uri) {
            (None, None, None) => {
                anyhow::ensure!(
                    env.dev_mock_base_url.is_none(),
                    "設了 GOOGLE_OAUTH_DEV_MOCK_BASE_URL 卻沒有設 GOOGLE_OAUTH_CLIENT_ID 等三個變數"
                );
                return Ok(None);
            }
            (Some(id), Some(secret), Some(uri)) => (id, secret, uri),
            _ => anyhow::bail!(
                "Google 登入設定不完整：GOOGLE_OAUTH_CLIENT_ID、GOOGLE_OAUTH_CLIENT_SECRET（或 _FILE）、\
                 GOOGLE_OAUTH_REDIRECT_URI 必須三個都設，或三個都不設（= 關閉 Google 登入）"
            ),
        };

    let parsed = url::Url::parse(&redirect_uri)
        .with_context(|| format!("GOOGLE_OAUTH_REDIRECT_URI 不是合法網址：{redirect_uri}"))?;
    // Google 只接受 https，或 http://localhost（本機開發）。在這裡先擋，
    // 否則錯誤要等到使用者按下登入、被 Google 拒絕時才看得到。
    anyhow::ensure!(
        parsed.scheme() == "https" || (parsed.scheme() == "http" && is_local_host(&parsed)),
        "GOOGLE_OAUTH_REDIRECT_URI 必須是 https（本機開發可用 http://localhost）：{redirect_uri}"
    );
    anyhow::ensure!(
        parsed.path().ends_with("/api/v1/auth/google/callback"),
        "GOOGLE_OAUTH_REDIRECT_URI 的路徑必須是 /api/v1/auth/google/callback：{redirect_uri}"
    );

    let endpoints = match env.dev_mock_base_url {
        None => GoogleEndpoints::google(),
        Some(base) => {
            anyhow::ensure!(
                !cookie_secure,
                "GOOGLE_OAUTH_DEV_MOCK_BASE_URL 只能用於本機模擬，COOKIE_SECURE=true（正式環境）時不准設定"
            );
            let mock = url::Url::parse(&base)
                .with_context(|| format!("GOOGLE_OAUTH_DEV_MOCK_BASE_URL 不是合法網址：{base}"))?;
            anyhow::ensure!(
                matches!(mock.scheme(), "http" | "https") && is_local_host(&mock),
                "GOOGLE_OAUTH_DEV_MOCK_BASE_URL 只能指向本機（localhost / 127.0.0.1）：{base}"
            );
            tracing::warn!(
                mock_base = %base,
                "⚠️ Google 登入目前指向本機假 Google（GOOGLE_OAUTH_DEV_MOCK_BASE_URL），僅供模擬，不是真的 Google"
            );
            GoogleEndpoints::dev_mock(&base)
        }
    };

    Ok(Some(GoogleOAuthConfig {
        client_id,
        client_secret,
        redirect_uri,
        endpoints,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CALLBACK: &str = "https://ipig.example.org/api/v1/auth/google/callback";
    const LOCAL_CALLBACK: &str = "http://localhost:8080/api/v1/auth/google/callback";

    fn env(id: Option<&str>, secret: Option<&str>, uri: Option<&str>) -> GoogleOAuthEnv {
        GoogleOAuthEnv {
            client_id: id.map(str::to_string),
            client_secret: secret.map(str::to_string),
            redirect_uri: uri.map(str::to_string),
            dev_mock_base_url: None,
        }
    }

    fn full(uri: &str) -> GoogleOAuthEnv {
        env(Some("id"), Some("secret"), Some(uri))
    }

    fn with_mock(base: &str) -> GoogleOAuthEnv {
        GoogleOAuthEnv {
            dev_mock_base_url: Some(base.to_string()),
            ..full(LOCAL_CALLBACK)
        }
    }

    #[test]
    fn all_unset_means_disabled() {
        let cfg =
            google_oauth_config_from_env(GoogleOAuthEnv::default(), true).expect("全未設不應報錯");
        assert!(cfg.is_none());
    }

    #[test]
    fn all_set_uses_real_google() {
        let cfg = google_oauth_config_from_env(full(CALLBACK), true)
            .expect("三者齊全應成功")
            .expect("三者齊全應為 Some");
        assert_eq!(cfg.client_id, "id");
        assert_eq!(cfg.redirect_uri, CALLBACK);
        assert_eq!(cfg.endpoints, GoogleEndpoints::google());
        assert!(!cfg.endpoints.is_dev_mock());
    }

    #[test]
    fn partial_config_fails_fast() {
        // 每一種「缺一或缺二」的組合都必須報錯，不可靜默當成關閉
        let cases = [
            env(Some("id"), None, None),
            env(None, Some("secret"), None),
            env(None, None, Some(CALLBACK)),
            env(Some("id"), Some("secret"), None),
            env(Some("id"), None, Some(CALLBACK)),
            env(None, Some("secret"), Some(CALLBACK)),
        ];
        for (i, case) in cases.into_iter().enumerate() {
            google_oauth_config_from_env(case, false)
                .expect_err(&format!("半套設定必須報錯：第 {i} 組"));
        }
    }

    #[test]
    fn rejects_non_https_redirect() {
        google_oauth_config_from_env(
            full("http://ipig.example.org/api/v1/auth/google/callback"),
            false,
        )
        .expect_err("非 localhost 的 http 必須拒絕");
    }

    #[test]
    fn allows_localhost_http_redirect() {
        let cfg = google_oauth_config_from_env(full(LOCAL_CALLBACK), false)
            .expect("localhost http 應允許");
        assert!(cfg.is_some());
    }

    #[test]
    fn rejects_wrong_callback_path() {
        google_oauth_config_from_env(full("https://ipig.example.org/login"), false)
            .expect_err("路徑不是 callback 必須拒絕");
    }

    #[test]
    fn dev_mock_points_all_endpoints_at_local_service() {
        let cfg = google_oauth_config_from_env(with_mock("http://127.0.0.1:9099/"), false)
            .expect("本機模擬應允許")
            .expect("應為 Some");
        assert!(cfg.endpoints.is_dev_mock());
        assert_eq!(
            cfg.endpoints.authorization,
            "http://127.0.0.1:9099/o/oauth2/v2/auth"
        );
        assert_eq!(cfg.endpoints.token, "http://127.0.0.1:9099/token");
        assert_eq!(
            cfg.endpoints.issuers,
            vec!["http://127.0.0.1:9099".to_string()]
        );
    }

    #[test]
    fn dev_mock_is_refused_in_production_mode() {
        google_oauth_config_from_env(with_mock("http://127.0.0.1:9099"), true)
            .expect_err("COOKIE_SECURE=true 時設假 Google 必須啟動失敗");
    }

    #[test]
    fn dev_mock_must_be_local() {
        for base in [
            "http://evil.example.com",
            "https://10.0.0.5:9099",
            "ftp://localhost",
        ] {
            google_oauth_config_from_env(with_mock(base), false)
                .expect_err(&format!("非本機或非 http(s) 必須拒絕：{base}"));
        }
    }

    #[test]
    fn dev_mock_without_client_config_fails() {
        let env = GoogleOAuthEnv {
            dev_mock_base_url: Some("http://127.0.0.1:9099".to_string()),
            ..GoogleOAuthEnv::default()
        };
        google_oauth_config_from_env(env, false).expect_err("只設假 Google 不設 client 必須報錯");
    }

    #[test]
    fn debug_does_not_leak_secret() {
        let cfg = GoogleOAuthConfig {
            client_id: "id".into(),
            client_secret: "super-secret-value".into(),
            redirect_uri: CALLBACK.into(),
            endpoints: GoogleEndpoints::google(),
        };
        let printed = format!("{cfg:?}");
        assert!(
            !printed.contains("super-secret-value"),
            "Debug 輸出洩漏 secret：{printed}"
        );
    }
}
