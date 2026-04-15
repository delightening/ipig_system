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
        jwt_keys: JwtKeys::for_testing(),
        csrf_secret: "b".repeat(64),
        jwt_expiration_seconds: 21600,
        jwt_refresh_expiration_days: 7,
        max_sessions_per_user: 5,
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
        gotenberg_url: "http://localhost:3000".to_string(),
        image_processor_url: "http://localhost:3100".to_string(),
        metrics_token: None,
        anthropic_api_key: None,
        ai_review_model: "claude-haiku-4-5".to_string(),
        ai_review_enabled: false,
        ai_review_timeout_secs: 30,
    }
}

#[test]
fn test_reauth_token_roundtrip() {
    let config = test_config();
    let user_id = uuid::Uuid::new_v4();
    let (token, expires_in) = AuthService::generate_reauth_token(&config, user_id)
        .expect("should generate token");
    assert!(expires_in > 0);
    assert!(AuthService::verify_reauth_token(&config, &token, user_id).is_ok());
}

#[test]
fn test_reauth_token_wrong_user_rejected() {
    let config = test_config();
    let user_id = uuid::Uuid::new_v4();
    let other_user = uuid::Uuid::new_v4();
    let (token, _) = AuthService::generate_reauth_token(&config, user_id)
        .expect("should generate token");
    assert!(
        AuthService::verify_reauth_token(&config, &token, other_user).is_err(),
        "Token for user A should be rejected when verifying as user B"
    );
}

#[test]
fn test_reauth_token_tampered_rejected() {
    let config = test_config();
    let user_id = uuid::Uuid::new_v4();
    let (token, _) = AuthService::generate_reauth_token(&config, user_id)
        .expect("should generate token");
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
