//! App 層 AEAD at-rest 加密（R66-B2 TOTP secret / R66-C6 簽章 payload）。
//!
//! 演算法 **XChaCha20-Poly1305**：192-bit（24-byte）nonce → 每次加密用隨機 nonce、
//! 永不重複，最適合「DB 逐列加密、nonce 存信封內」的 at-rest 場景（無 nonce 管理負擔）。
//!
//! 信封字串格式：`"<key_version>:<base64(nonce(24) ‖ ciphertext+tag)>"`。
//! - `key_version` 支援未來金鑰輪替（鏡像 audit chain `hmac_version`，見 `HMAC_VERSIONING.md`）。
//! - `aad`（associated data）綁定上下文（如 `user_id` bytes），防 ciphertext 跨列/跨實體移植。
//!
//! 金鑰 32 bytes，由 `Config.encryption_key`（`ENCRYPTION_KEY` secret）載入，drop 時 zeroize。

use base64::{engine::general_purpose::STANDARD, Engine};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use rand::Rng;
use zeroize::Zeroizing;

use crate::{error::AppError, Result};

/// 目前加密金鑰版本（信封字串前綴）。未來輪替時遞增、decrypt 依版本選金鑰。
pub const ENCRYPTION_KEY_VERSION: u8 = 1;

const NONCE_LEN: usize = 24; // XChaCha20-Poly1305 192-bit nonce
const KEY_LEN: usize = 32;

/// 32-byte AEAD 金鑰。drop 時 zeroize；`Debug` 遮蔽避免金鑰寫進 log。
#[derive(Clone)]
pub struct EncryptionKey(Zeroizing<[u8; KEY_LEN]>);

impl std::fmt::Debug for EncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EncryptionKey(***redacted***)")
    }
}

impl EncryptionKey {
    /// 由 base64（standard）字串解碼 32 bytes 金鑰；長度 / 編碼錯即拒絕。
    pub fn from_base64(s: &str) -> Result<Self> {
        // 解碼後的金鑰位元組以 Zeroizing 包裝 → 離開作用域 / 出錯時清零，不殘留 heap。
        let bytes = Zeroizing::new(
            STANDARD
                .decode(s.trim())
                .map_err(|_| AppError::Internal("ENCRYPTION_KEY 非合法 base64".into()))?,
        );
        let arr: [u8; KEY_LEN] = bytes.as_slice().try_into().map_err(|_| {
            AppError::Internal("ENCRYPTION_KEY 須為 32 bytes（base64 解碼後）".into())
        })?;
        Ok(Self(Zeroizing::new(arr)))
    }

    fn cipher(&self) -> Result<XChaCha20Poly1305> {
        XChaCha20Poly1305::new_from_slice(&self.0[..])
            .map_err(|_| AppError::Internal("AEAD 金鑰長度錯誤".into()))
    }

    /// AEAD 加密 → `"<version>:<base64(nonce ‖ ciphertext+tag)>"`。
    /// `aad` 綁定上下文；解密時須帶相同值。
    pub fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> Result<String> {
        let mut nonce = [0u8; NONCE_LEN];
        rand::rng().fill_bytes(&mut nonce);
        let nonce_arr = XNonce::try_from(&nonce[..])
            .map_err(|_| AppError::Internal("AEAD nonce 長度錯誤".into()))?;
        let ciphertext = self
            .cipher()?
            .encrypt(
                &nonce_arr,
                Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| AppError::Internal("AEAD 加密失敗".into()))?;
        let mut envelope = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        envelope.extend_from_slice(&nonce);
        envelope.extend_from_slice(&ciphertext);
        Ok(format!(
            "{ENCRYPTION_KEY_VERSION}:{}",
            STANDARD.encode(&envelope)
        ))
    }

    /// 解密 `encrypt` 的輸出；`aad` 須與加密時相同。回傳 zeroize-on-drop 明文。
    pub fn decrypt(&self, stored: &str, aad: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        let (version, b64) = stored
            .split_once(':')
            .ok_or_else(|| AppError::Internal("加密信封格式錯誤（缺版本前綴）".into()))?;
        let v: u8 = version
            .parse()
            .map_err(|_| AppError::Internal("加密信封版本非數字".into()))?;
        if v != ENCRYPTION_KEY_VERSION {
            return Err(AppError::Internal(format!("不支援的加密金鑰版本：{v}")));
        }
        let raw = STANDARD
            .decode(b64)
            .map_err(|_| AppError::Internal("加密信封 base64 解碼失敗".into()))?;
        if raw.len() <= NONCE_LEN {
            return Err(AppError::Internal("加密信封長度不足".into()));
        }
        let (nonce, ciphertext) = raw.split_at(NONCE_LEN);
        let nonce_arr = XNonce::try_from(nonce)
            .map_err(|_| AppError::Internal("加密信封 nonce 長度錯誤".into()))?;
        let plaintext = self
            .cipher()?
            .decrypt(
                &nonce_arr,
                Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|_| {
                AppError::Internal("AEAD 解密失敗（金鑰 / aad / 資料不符或遭竄改）".into())
            })?;
        Ok(Zeroizing::new(plaintext))
    }
}

/// 判斷儲存值是否為本模組產生的加密信封（vs 過渡期 legacy 明文）。
/// 信封格式 `"<digits>:<base64>"`；TOTP base32 secret 不含 ':'，故可區分。
pub fn is_encrypted_envelope(stored: &str) -> bool {
    matches!(
        stored.split_once(':'),
        Some((v, _)) if !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> EncryptionKey {
        // 固定 32-byte 金鑰（base64）。
        EncryptionKey::from_base64(&STANDARD.encode([7u8; KEY_LEN])).expect("test key")
    }

    #[test]
    fn roundtrip_recovers_plaintext() {
        let key = test_key();
        let aad = b"user-id-bytes";
        let env = key.encrypt(b"JBSWY3DPEHPK3PXP", aad).expect("encrypt");
        assert!(is_encrypted_envelope(&env), "輸出應為加密信封");
        let pt = key.decrypt(&env, aad).expect("decrypt");
        assert_eq!(&*pt, b"JBSWY3DPEHPK3PXP");
    }

    #[test]
    fn same_plaintext_different_ciphertext() {
        let key = test_key();
        let a = key.encrypt(b"secret", b"aad").expect("a");
        let b = key.encrypt(b"secret", b"aad").expect("b");
        assert_ne!(a, b, "隨機 nonce → 同明文密文不同");
    }

    #[test]
    fn decrypt_rejects_wrong_aad() {
        let key = test_key();
        let env = key.encrypt(b"secret", b"aad-1").expect("encrypt");
        key.decrypt(&env, b"aad-2").expect_err("aad 不符應失敗");
    }

    #[test]
    fn decrypt_rejects_wrong_key() {
        let env = test_key().encrypt(b"secret", b"aad").expect("encrypt");
        let other = EncryptionKey::from_base64(&STANDARD.encode([9u8; KEY_LEN])).expect("key2");
        other.decrypt(&env, b"aad").expect_err("換金鑰應解不開");
    }

    #[test]
    fn decrypt_rejects_tampered_ciphertext() {
        let key = test_key();
        let env = key.encrypt(b"secret", b"aad").expect("encrypt");
        // 翻動最後一個 base64 字元 → 破壞 tag / 密文。
        let (prefix, b64) = env.split_once(':').expect("split");
        let mut chars: Vec<char> = b64.chars().collect();
        let last = chars.len() - 1;
        chars[last] = if chars[last] == 'A' { 'B' } else { 'A' };
        let tampered = format!("{prefix}:{}", chars.into_iter().collect::<String>());
        key.decrypt(&tampered, b"aad").expect_err("竄改應失敗");
    }

    #[test]
    fn is_envelope_distinguishes_legacy_base32() {
        assert!(is_encrypted_envelope("1:AAAA"));
        assert!(
            !is_encrypted_envelope("JBSWY3DPEHPK3PXP"),
            "legacy base32 無 ':'"
        );
        assert!(!is_encrypted_envelope("abc:def"), "非數字版本前綴不算信封");
    }

    #[test]
    fn from_base64_rejects_wrong_length() {
        EncryptionKey::from_base64(&STANDARD.encode([0u8; 16])).expect_err("16 bytes 應拒絕");
        EncryptionKey::from_base64("not base64!!!").expect_err("非 base64 應拒絕");
    }

    #[test]
    fn debug_redacts_key() {
        let dbg = format!("{:?}", test_key());
        assert!(!dbg.contains("7"), "Debug 不應洩漏金鑰位元組");
        assert!(dbg.contains("redacted"));
    }
}
