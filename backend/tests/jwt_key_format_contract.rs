//! JWT EC 私鑰格式契約：**只接受 PKCS8，不接受 SEC1**。
//!
//! 為什麼需要這支測試：`Cargo.toml` 選了 `jsonwebtoken` 的 `rust_crypto` feature，
//! 其底層 `p256` 只提供 `from_pkcs8_pem()`。餵 SEC1（`-----BEGIN EC PRIVATE KEY-----`）
//! 會回 `InvalidKeyFormat`，而 api 在啟動時解析金鑰失敗會直接離開 → 無限重啟。
//!
//! 2026-08-20 實際踩過：`scripts/newprod/gen-secrets.sh` 原本用 `openssl ecparam -genkey`
//! 產出 SEC1，新環境的 api 因此起不來；當時 `config.rs` 的錯誤訊息還寫著
//! 「SEC1 或 PKCS8 格式」，把排查方向帶偏。
//!
//! 這支測試把「只吃 PKCS8」這件事變成可執行的契約——若日後有人改掉密碼學後端
//! （例如換回預設的 aws_lc_rs），SEC1 那個 assertion 會失敗，提醒他一併更新
//! `config.rs` 的錯誤訊息與 `gen-secrets.sh` 的產生方式。

use jsonwebtoken::EncodingKey;

/// 同一把 P-256 金鑰的兩種編碼（用 openssl 產生後固定寫進測試，避免測試依賴外部指令）。
/// 產生方式：
///   openssl ecparam -name prime256v1 -genkey -noout -out sec1.pem
///   openssl pkcs8 -topk8 -nocrypt -in sec1.pem -out pkcs8.pem
/// ⚠️ 這是一把**真實有效**的 P-256 SEC1 私鑰（僅供測試，從未用於任何環境）。
/// 必須是有效金鑰才有鑑別力——若隨手打一段無效 base64，`from_ec_pem` 會因為
/// 「解不開」而失敗，測試看似通過，實際上完全沒驗到「格式不受支援」這件事。
const SEC1_PEM: &str = "-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIL1sqHAtgk5dxe5aFSIJCq4HNhEwsGE4a378VQnDuX7JoAoGCCqGSM49
AwEHoUQDQgAETqL4tS/vo6LZkH0uKIMk3BQhrKpuVS93z2T/4+Tph2ly+UrqazRu
zCOvs0exeC2URSM4EMZOXY8yYwh4WJ7Tag==
-----END EC PRIVATE KEY-----
";

/// 上面那把 SEC1 金鑰的 PKCS8 編碼——**同一把金鑰材料**，只是外層編碼不同。
/// 有這個對照組，`sec1_rejected_but_same_key_as_pkcs8_accepted` 才能證明
/// 「被拒的原因是格式，不是金鑰本身有問題」。
const SAME_KEY_AS_PKCS8_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgvWyocC2CTl3F7loV
IgkKrgc2ETCwYThrfvxVCcO5fsmhRANCAAROovi1L++jotmQfS4ogyTcFCGsqm5V
L3fPZP/j5OmHaXL5SuprNG7MI6+zR7F4LZRFIzgQxk5djzJjCHhYntNq
-----END PRIVATE KEY-----
";

/// 核心契約：同一把金鑰，PKCS8 可用、SEC1 不可用。
///
/// 這兩個 assertion 必須成對看——只斷言「SEC1 失敗」證明不了什麼
/// （無效金鑰也會失敗）；加上「同一把金鑰的 PKCS8 成功」，才鎖定
/// 差異確實來自編碼格式。
#[test]
fn sec1_rejected_but_same_key_as_pkcs8_accepted() {
    assert!(
        EncodingKey::from_ec_pem(SAME_KEY_AS_PKCS8_PEM.as_bytes()).is_ok(),
        "對照組：同一把金鑰的 PKCS8 編碼必須被接受（若這裡失敗，代表測試金鑰本身壞了）"
    );
    assert!(
        EncodingKey::from_ec_pem(SEC1_PEM.as_bytes()).is_err(),
        "同一把金鑰的 SEC1 編碼必須被拒絕——證明限制來自「格式」而非金鑰材料"
    );
}

#[test]
fn pkcs8_header_is_the_distinguishing_marker() {
    // 兩種格式的差別在 PEM 標頭，而 gen-secrets.sh 正是靠這一行做防呆檢查。
    // 這裡把那個檢查條件本身鎖住，避免日後有人改了腳本的判斷字串卻沒對齊實際格式。
    assert!(
        SAME_KEY_AS_PKCS8_PEM.starts_with("-----BEGIN PRIVATE KEY-----"),
        "PKCS8 的標頭必須是 BEGIN PRIVATE KEY"
    );
    assert!(
        SEC1_PEM.starts_with("-----BEGIN EC PRIVATE KEY-----"),
        "SEC1 的標頭必須是 BEGIN EC PRIVATE KEY"
    );
}

#[test]
fn sec1_private_key_is_rejected() {
    // 這是契約的重點：SEC1 必須被拒絕，且失敗發生在解析階段（而非稍後簽章時）。
    // 若這個 assertion 失敗，代表密碼學後端被換過（SEC1 變成可接受），
    // 此時請一併更新：
    //   - backend/src/config.rs 的 JWT_EC_PRIVATE_KEY 錯誤訊息
    //   - scripts/newprod/gen-secrets.sh 的金鑰產生與格式檢查
    assert!(
        EncodingKey::from_ec_pem(SEC1_PEM.as_bytes()).is_err(),
        "SEC1 私鑰應被拒絕（rust_crypto feature 下 p256 只支援 PKCS8）；\
         若此處開始通過，表示密碼學後端已更換，請同步更新 config.rs 的錯誤訊息與 gen-secrets.sh"
    );
}
