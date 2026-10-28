use super::AuthService;

// ==========================================
// validate_password_strength 測試
// ==========================================

#[test]
fn test_password_valid() {
    // 符合所有條件：10 字元以上 + 大寫 + 小寫 + 數字 + 非弱密碼
    assert!(AuthService::validate_password_strength("Abcdef1234").is_ok());
}

#[test]
fn test_password_too_short() {
    assert!(AuthService::validate_password_strength("Ab1").is_err());
    assert!(AuthService::validate_password_strength("Abcd12345").is_err()); // 9 字元
}

#[test]
fn test_password_exact_10_chars() {
    // 恰好 10 字元
    assert!(AuthService::validate_password_strength("Abcdef123!").is_ok());
}

#[test]
fn test_password_no_uppercase() {
    assert!(AuthService::validate_password_strength("abcdef12345").is_err());
}

#[test]
fn test_password_no_lowercase() {
    assert!(AuthService::validate_password_strength("ABCDEF12345").is_err());
}

#[test]
fn test_password_no_digit() {
    assert!(AuthService::validate_password_strength("Abcdefghijk").is_err());
}

#[test]
fn test_password_empty() {
    assert!(AuthService::validate_password_strength("").is_err());
}

#[test]
fn test_password_with_special_chars() {
    // 特殊字元不影響通過
    assert!(AuthService::validate_password_strength("Abcd123!@#x").is_ok());
}

#[test]
fn test_password_unicode() {
    // 含中文字元仍需滿足大小寫與數字
    assert!(AuthService::validate_password_strength("Ab12345中文extra").is_ok());
}

#[test]
fn test_password_common_weak_rejected() {
    // 常見弱密碼即使滿足長度與複雜度也應被拒絕
    // 注意：大部分弱密碼不滿足複雜度，但 Changeme123 同時滿足長度與複雜度
    assert!(AuthService::validate_password_strength("Changeme123").is_err());
    assert!(AuthService::validate_password_strength("Password123").is_err());
}

#[test]
fn test_password_weak_case_insensitive() {
    // 弱密碼黑名單應大小寫不敏感
    assert!(AuthService::validate_password_strength("CHANGEME123").is_err());
}

// ==========================================
// hash_token 測試
// ==========================================

#[test]
fn test_hash_token_deterministic() {
    let hash1 = AuthService::hash_token("test_token");
    let hash2 = AuthService::hash_token("test_token");
    assert_eq!(hash1, hash2, "相同輸入應產生相同 hash");
}

#[test]
fn test_hash_token_different_inputs() {
    let hash1 = AuthService::hash_token("token_a");
    let hash2 = AuthService::hash_token("token_b");
    assert_ne!(hash1, hash2, "不同輸入應產生不同 hash");
}

#[test]
fn test_hash_token_length() {
    // SHA-256 hex 輸出固定 64 字元
    let hash = AuthService::hash_token("any_token");
    assert_eq!(hash.len(), 64, "SHA-256 hex 應為 64 字元");
}

// ==========================================
// hash_password / verify_password 測試
// ==========================================

#[test]
fn test_hash_and_verify_password() {
    let password = "TestPass123";
    let hash = AuthService::hash_password(password).expect("hash 應成功");
    let is_valid = AuthService::verify_password(password, &hash).expect("verify 應成功");
    assert!(is_valid, "正確密碼應通過驗證");
}

#[test]
fn test_verify_wrong_password() {
    let password = "TestPass123";
    let hash = AuthService::hash_password(password).expect("hash 應成功");
    let is_valid = AuthService::verify_password("WrongPass123", &hash).expect("verify 應成功");
    assert!(!is_valid, "錯誤密碼不應通過驗證");
}

#[test]
fn test_hash_password_unique_salts() {
    let password = "SamePassword1";
    let hash1 = AuthService::hash_password(password).expect("hash 應成功");
    let hash2 = AuthService::hash_password(password).expect("hash 應成功");
    assert_ne!(hash1, hash2, "不同 salt 應產生不同 hash");
}

// ==========================================
// PHC fixture：跨版本相容（R113-1）
//
// 上面的來回測試是「新版 hash → 新版 verify」，自己跟自己相容，抓不到
// argon2／password-hash 升版後「舊版產生的雜湊還能不能驗」。
// 下列字串由 argon2 0.5.3（Cargo.lock 實際鎖定版本）以 Argon2::default() 產生，
// 為測試專用字串，非真實資料。
// 升版時**不得修改這些常數**來求綠：任何一個變紅，代表既有使用者的密碼
// 會無法登入，要回報、不要調整 fixture。
// ==========================================

const FIXTURE_PASSWORD_ASCII: &str = "Fixture-Pass-2026!";
const FIXTURE_HASH_ASCII: &str = "$argon2id$v=19$m=19456,t=2,p=1$TZy1aCNo0Jh1hxeapcdrFA$W+U04MOXzWadhIgNDlvZrBGrE0diURbnO+ufkfZCDIw";

const FIXTURE_PASSWORD_UNICODE: &str = "密碼Fixture測試9a";
const FIXTURE_HASH_UNICODE: &str = "$argon2id$v=19$m=19456,t=2,p=1$S/5w0keO/ZCZHAH//+Fahg$nKZhGErkwM2x+22BaaBpO7Mc1hP1dwIe3MGz0R0TI0k";

/// login.rs「使用者不存在」分支用的虛擬雜湊（SEC-AUDIT-002）。
/// ⚠️ 這是 login.rs 內寫死字串的副本；兩處必須同步。
const LOGIN_DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$dGhpc2lzYWR1bW15c2FsdA$ZKhM8GZ8MJD3E5VNrp0MWuVIL+rCBjKNHdXMaGW4+A8";

#[test]
fn test_fixture_hashes_verify_with_correct_password() {
    for (password, hash) in [
        (FIXTURE_PASSWORD_ASCII, FIXTURE_HASH_ASCII),
        (FIXTURE_PASSWORD_UNICODE, FIXTURE_HASH_UNICODE),
    ] {
        let ok =
            AuthService::verify_password(password, hash).expect("舊版產生的 PHC 字串必須能被解析");
        assert!(ok, "舊版雜湊對正確密碼必須驗證通過：{hash}");
    }
}

#[test]
fn test_fixture_hashes_reject_wrong_password() {
    for hash in [FIXTURE_HASH_ASCII, FIXTURE_HASH_UNICODE] {
        let ok = AuthService::verify_password("Wrong-Pass-0000!", hash)
            .expect("舊版產生的 PHC 字串必須能被解析");
        assert!(!ok, "舊版雜湊對錯誤密碼不得驗證通過：{hash}");
    }
}

#[test]
fn test_fixture_hash_is_not_valid_for_the_other_password() {
    let ok =
        AuthService::verify_password(FIXTURE_PASSWORD_ASCII, FIXTURE_HASH_UNICODE).expect("可解析");
    assert!(!ok, "雜湊與密碼必須一一對應，不能互相通過");
}

#[test]
fn test_new_hash_keeps_expected_phc_prefix_and_params() {
    // 鎖住 Argon2::default() 的演算法／版本／參數。升版若改變預設參數，這裡會紅，
    // 需要人工決定「明確指定舊參數」或「接受新參數並同步 LOGIN_DUMMY_HASH」。
    let hash = AuthService::hash_password("Prefix-Check-1!").expect("hash 應成功");
    assert!(
        hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"),
        "預設參數改變：{hash}"
    );
}

#[test]
fn test_login_dummy_hash_is_parseable() {
    // login.rs 以 `if let Ok(parsed) = PasswordHash::new(dummy)` 無聲略過解析失敗，
    // 一旦解析失敗，「使用者不存在」與「密碼錯誤」的耗時差異就回來且沒有任何錯誤。
    // 這裡確認它是可解析的 PHC 字串（結果是 true 或 false 皆可，不能是 Err）。
    AuthService::verify_password("any-password", LOGIN_DUMMY_HASH)
        .expect("login 虛擬雜湊必須能被解析，否則常數時間防護會無聲失效");
}

// ==========================================
// SEC-33: reauth token 測試
// ==========================================

fn test_config() -> crate::config::Config {
    use crate::config::{Config, JwtKeys};
    Config {
        host: "0.0.0.0".to_string(),
        port: 3000,
        database_url: "postgres://test:test@localhost/test".to_string(),
        database_max_connections: 10,
        database_min_connections: 2,
        database_acquire_timeout_seconds: 30,
        database_retry_attempts: 5,
        database_retry_delay_seconds: 5,
        database_statement_timeout_ms: 30000,
        jwt_keys: JwtKeys::for_testing(),
        csrf_secret: "b".repeat(64),
        jwt_expiration_seconds: 900,
        jwt_refresh_expiration_days: 7,
        max_sessions_per_user: 5,
        auth_idle_timeout_minutes: 30,
        smtp_host: None,
        smtp_port: 587,
        smtp_username: None,
        smtp_password: None,
        smtp_from_email: "noreply@test.local".to_string(),
        smtp_from_name: "Test".to_string(),
        line_notify_token: None,
        app_url: "http://localhost".to_string(),
        cookie_secure: false,
        cookie_domain: None,
        seed_dev_users: false,
        allowed_clock_ip_ranges: vec![],
        clock_office_latitude: None,
        clock_office_longitude: None,
        clock_gps_radius_meters: 200.0,
        trust_proxy_headers: true,
        cors_allowed_origins: vec![],
        audit_hmac_key: None,
        encryption_key: None,
        audit_chain_verify_active: false,
        role_signature_required: false,
        retention_enforcer_enabled: false,
        jwt_ec_private_key_file: None,
        disable_csrf_for_tests: false,
        disable_account_lockout: false,
        account_lockout_max_attempts: 5,
        account_lockout_duration_minutes: 15,
        upload_dir: "./uploads".to_string(),
        geoip_db_path: "/app/geoip/GeoLite2-City.mmdb".to_string(),
        skip_migration_check: false,
        admin_initial_password: None,
        test_user_password: None,
        dev_user_password: None,
        is_ci: false,
        pdf_service_url: "http://localhost:9210".to_string(),
        pdf_service_token: "test-token".to_string(),
        metrics_token: None,
        alertmanager_webhook_token: None,
        anthropic_api_key: None,
        ai_review_model: "claude-haiku-4-5".to_string(),
        ai_review_enabled: false,
        ai_review_timeout_secs: 30,
        google_oauth: None,
    }
}

#[test]
fn test_reauth_token_roundtrip() {
    let config = test_config();
    let user_id = uuid::Uuid::new_v4();
    let (token, expires_in) =
        AuthService::generate_reauth_token(&config, user_id).expect("should generate token");
    assert!(expires_in > 0);
    assert!(AuthService::verify_reauth_token(&config, &token, user_id).is_ok());
}

#[test]
fn test_reauth_token_wrong_user_rejected() {
    let config = test_config();
    let user_id = uuid::Uuid::new_v4();
    let other_user = uuid::Uuid::new_v4();
    let (token, _) =
        AuthService::generate_reauth_token(&config, user_id).expect("should generate token");
    assert!(
        AuthService::verify_reauth_token(&config, &token, other_user).is_err(),
        "Token for user A should be rejected when verifying as user B"
    );
}

#[test]
fn test_reauth_token_tampered_rejected() {
    let config = test_config();
    let user_id = uuid::Uuid::new_v4();
    let (token, _) =
        AuthService::generate_reauth_token(&config, user_id).expect("should generate token");
    let tampered = format!("{}x", token);
    assert!(
        AuthService::verify_reauth_token(&config, &tampered, user_id).is_err(),
        "Tampered token should be rejected"
    );
}

#[test]
fn test_reauth_token_empty_rejected() {
    let config = test_config();
    let user_id = uuid::Uuid::new_v4();
    assert!(AuthService::verify_reauth_token(&config, "", user_id).is_err());
}

#[test]
fn test_reauth_token_garbage_rejected() {
    let config = test_config();
    let user_id = uuid::Uuid::new_v4();
    assert!(AuthService::verify_reauth_token(&config, "not.a.jwt", user_id).is_err());
}
