//! `google_oidc` 的純函式單元測試（ID token claims、flow cookie、PKCE）。
//! 需要資料庫的綁定邏輯測試在 `backend/tests/google_identity_link.rs`。

use super::*;

const SECRET: &str = "test-csrf-secret-test-csrf-secret-test-csrf-secret";
const CLIENT_ID: &str = "client-123.apps.googleusercontent.com";
const NONCE: &str = "nonce-abc";
const NOW: i64 = 1_800_000_000;

fn google_config() -> GoogleOAuthConfig {
    GoogleOAuthConfig {
        client_id: CLIENT_ID.into(),
        client_secret: "secret".into(),
        redirect_uri: "https://ipig.example.org/api/v1/auth/google/callback".into(),
    }
}

/// 組一個未簽章的 ID token（本模組本來就不驗簽章，只驗 claims）。
fn id_token(claims: serde_json::Value) -> String {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
    let payload = URL_SAFE_NO_PAD.encode(claims.to_string());
    format!("{header}.{payload}.sig")
}

fn valid_claims() -> serde_json::Value {
    serde_json::json!({
        "iss": "https://accounts.google.com",
        "aud": CLIENT_ID,
        "sub": "109876543210987654321",
        "exp": NOW + 3600,
        "iat": NOW,
        "nonce": NONCE,
        "email": "someone@example.com",
        "email_verified": true,
    })
}

fn with(mut claims: serde_json::Value, key: &str, value: serde_json::Value) -> serde_json::Value {
    claims[key] = value;
    claims
}

fn without(mut claims: serde_json::Value, key: &str) -> serde_json::Value {
    claims
        .as_object_mut()
        .expect("claims 是 JSON 物件")
        .remove(key);
    claims
}

fn verify(
    claims: serde_json::Value,
) -> std::result::Result<VerifiedGoogleIdentity, GoogleLoginFailure> {
    AuthService::verify_google_id_token(&id_token(claims), CLIENT_ID, NONCE, NOW)
}

#[test]
fn valid_id_token_yields_identity() {
    let identity = verify(valid_claims()).expect("合格的 claims 應通過");
    assert_eq!(identity.subject, "109876543210987654321");
    assert_eq!(identity.email, "someone@example.com");
}

#[test]
fn issuer_without_scheme_is_accepted() {
    verify(with(valid_claims(), "iss", "accounts.google.com".into()))
        .expect("Google 文件列出的兩種 iss 寫法都要接受");
}

#[test]
fn foreign_issuer_is_rejected() {
    let err = verify(with(
        valid_claims(),
        "iss",
        "https://evil.example.com".into(),
    ))
    .expect_err("非 Google 的 iss 必須拒絕");
    assert_eq!(err, GoogleLoginFailure::ProviderError);
}

#[test]
fn token_for_another_client_is_rejected() {
    verify(with(valid_claims(), "aud", "other-client".into()))
        .expect_err("發給別的 client 的 token 必須拒絕");
}

#[test]
fn multi_audience_requires_matching_azp() {
    let multi = with(
        valid_claims(),
        "aud",
        serde_json::json!([CLIENT_ID, "other"]),
    );
    verify(multi.clone()).expect_err("多個 aud 且缺 azp 必須拒絕");
    verify(with(multi, "azp", CLIENT_ID.into())).expect("azp 為自己時應通過");
}

#[test]
fn expired_token_is_rejected() {
    verify(with(
        valid_claims(),
        "exp",
        (NOW - CLOCK_SKEW_SECS - 1).into(),
    ))
    .expect_err("過期（超出容許誤差）必須拒絕");
}

#[test]
fn future_issued_token_is_rejected() {
    verify(with(
        valid_claims(),
        "iat",
        (NOW + CLOCK_SKEW_SECS + 1).into(),
    ))
    .expect_err("iat 在未來必須拒絕");
}

#[test]
fn nonce_mismatch_or_missing_is_rejected() {
    verify(with(valid_claims(), "nonce", "other".into())).expect_err("nonce 不符必須拒絕");
    verify(without(valid_claims(), "nonce")).expect_err("缺 nonce 必須拒絕（防重放）");
}

#[test]
fn unverified_email_is_rejected_with_specific_reason() {
    let err = verify(with(valid_claims(), "email_verified", false.into()))
        .expect_err("email 未驗證必須拒絕");
    assert_eq!(err, GoogleLoginFailure::EmailNotVerified);
    assert_eq!(
        verify(without(valid_claims(), "email_verified"))
            .expect_err("缺 email_verified 視同未驗證"),
        GoogleLoginFailure::EmailNotVerified
    );
}

#[test]
fn string_email_verified_is_accepted() {
    verify(with(valid_claims(), "email_verified", "true".into()))
        .expect("字串形式的 \"true\" 也要接受");
}

#[test]
fn missing_email_or_blank_subject_is_rejected() {
    verify(without(valid_claims(), "email")).expect_err("缺 email 必須拒絕");
    verify(with(valid_claims(), "sub", "   ".into())).expect_err("空白 sub 必須拒絕");
}

#[test]
fn malformed_token_is_rejected() {
    for bad in ["", "a.b", "a.b.c.d", "a.!!!.c"] {
        AuthService::verify_google_id_token(bad, CLIENT_ID, NONCE, NOW)
            .expect_err(&format!("格式錯誤必須拒絕：{bad:?}"));
    }
}

fn sample_flow() -> GoogleFlowState {
    GoogleFlowState {
        state: "state-1".into(),
        nonce: NONCE.into(),
        code_verifier: "verifier-1".into(),
        exp: NOW + 600,
    }
}

#[test]
fn flow_state_roundtrips() {
    let flow = sample_flow();
    let sealed = AuthService::seal_google_flow_state(SECRET, &flow).expect("封裝應成功");
    let opened = AuthService::open_google_flow_state(SECRET, &sealed, NOW).expect("同金鑰應可解開");
    assert_eq!(opened, flow);
}

#[test]
fn flow_state_rejects_tampering_wrong_key_and_expiry() {
    let sealed = AuthService::seal_google_flow_state(SECRET, &sample_flow()).expect("封裝應成功");

    assert!(
        AuthService::open_google_flow_state("another-secret", &sealed, NOW).is_none(),
        "不同金鑰不可解開"
    );

    let (payload, tag) = sealed.split_once('.').expect("格式為 payload.tag");
    let forged_flow = GoogleFlowState {
        state: "attacker-state".into(),
        ..sample_flow()
    };
    let forged_payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&forged_flow).expect("序列化"));
    assert_ne!(forged_payload, payload);
    assert!(
        AuthService::open_google_flow_state(SECRET, &format!("{forged_payload}.{tag}"), NOW)
            .is_none(),
        "換掉 payload 但沿用舊 tag 必須被擋"
    );

    assert!(
        AuthService::open_google_flow_state(SECRET, &sealed, NOW + 600).is_none(),
        "到期當下即失效"
    );
    assert!(AuthService::open_google_flow_state(SECRET, "garbage", NOW).is_none());
}

#[test]
fn begin_login_builds_pkce_url_and_matching_cookie() {
    let start =
        AuthService::begin_google_login(&google_config(), SECRET, NOW).expect("應產生授權網址");
    let flow = AuthService::open_google_flow_state(SECRET, &start.flow_cookie_value, NOW)
        .expect("cookie 應可解開");
    assert_eq!(flow.exp, NOW + GOOGLE_FLOW_TTL_SECS);
    assert_eq!(
        flow.code_verifier.len(),
        43,
        "RFC 7636 verifier 長度 43～128"
    );

    let url = url::Url::parse(&start.authorization_url).expect("授權網址應可解析");
    assert!(start
        .authorization_url
        .starts_with(GOOGLE_AUTHORIZATION_ENDPOINT));
    let params: std::collections::HashMap<String, String> =
        url.query_pairs().into_owned().collect();
    assert_eq!(params.get("client_id").map(String::as_str), Some(CLIENT_ID));
    assert_eq!(
        params.get("response_type").map(String::as_str),
        Some("code")
    );
    assert_eq!(params.get("scope").map(String::as_str), Some(GOOGLE_SCOPES));
    assert_eq!(params.get("state"), Some(&flow.state));
    assert_eq!(params.get("nonce"), Some(&flow.nonce));
    assert_eq!(
        params.get("code_challenge_method").map(String::as_str),
        Some("S256")
    );
    assert_eq!(
        params.get("code_challenge"),
        Some(&pkce_challenge(&flow.code_verifier)),
        "challenge 必須由 cookie 裡的 verifier 算出，否則 token 交換會失敗"
    );
    assert_ne!(flow.state, flow.nonce, "state 與 nonce 必須各自獨立產生");
}

#[test]
fn pkce_challenge_matches_rfc7636_example() {
    // RFC 7636 Appendix B 的官方範例
    assert_eq!(
        pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
}

#[test]
fn state_comparison_is_exact() {
    let flow = sample_flow();
    assert!(AuthService::google_state_matches(&flow, "state-1"));
    assert!(!AuthService::google_state_matches(&flow, "state-2"));
    assert!(!AuthService::google_state_matches(&flow, "state-"));
    assert!(!AuthService::google_state_matches(&flow, ""));
}

#[test]
fn failure_codes_are_distinct() {
    let all = [
        GoogleLoginFailure::NotConfigured,
        GoogleLoginFailure::Cancelled,
        GoogleLoginFailure::InvalidState,
        GoogleLoginFailure::ProviderError,
        GoogleLoginFailure::EmailNotVerified,
        GoogleLoginFailure::NoAccount,
        GoogleLoginFailure::AmbiguousAccount,
        GoogleLoginFailure::AccountUnavailable,
        GoogleLoginFailure::IdentityConflict,
    ];
    let codes: std::collections::HashSet<_> = all.iter().map(|f| f.code()).collect();
    assert_eq!(codes.len(), all.len(), "前端依 code 顯示訊息，不可重複");
}

#[test]
fn only_security_relevant_failures_are_audited() {
    use GoogleLoginFailure as F;
    // 使用者自己取消、伺服器未設定：不是安全事件，不進稽核鏈
    assert!(!F::Cancelled.is_security_relevant());
    assert!(!F::NotConfigured.is_security_relevant());
    // 其餘（含 state 不符這種 CSRF／重放訊號）都要進稽核鏈
    for f in [
        F::InvalidState,
        F::ProviderError,
        F::EmailNotVerified,
        F::NoAccount,
        F::AmbiguousAccount,
        F::AccountUnavailable,
        F::IdentityConflict,
    ] {
        assert!(f.is_security_relevant(), "{} 應進稽核", f.code());
    }
}
