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
