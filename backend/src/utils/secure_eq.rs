//! 定速字串比較工具，用於 token / shared secret 驗證，
//! 防止透過 RTT side-channel 推測前綴而 byte-by-byte 暴力破解。
//!
//! 不引入額外依賴 — 簡易常數時間 byte 比對，包含長度差異也以 OR 累積，
//! 確保 timing 不洩漏「長度匹配但內容不符」與「長度不符」的區別。

/// 以常數時間（不 short-circuit）比較兩段 bytes 是否相等。
///
/// 比 `a == b` 安全的場景：當 `b` 是攻擊者可控、`a` 是 server 持有的密鑰
/// （例如 webhook bearer token、metrics scrape token）時。
pub fn const_time_eq(a: &[u8], b: &[u8]) -> bool {
    // 將長度差異也納入 diff，避免 length mismatch 早退
    let mut diff: u8 =
        (a.len() as u32 ^ b.len() as u32) as u8 | ((a.len() as u32 ^ b.len() as u32) >> 8) as u8;
    let len = a.len().min(b.len());
    for i in 0..len {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_bytes() {
        assert!(const_time_eq(b"hello", b"hello"));
    }

    #[test]
    fn different_bytes_same_length() {
        assert!(!const_time_eq(b"hello", b"world"));
    }

    #[test]
    fn different_lengths() {
        assert!(!const_time_eq(b"hello", b"hello!"));
        assert!(!const_time_eq(b"", b"x"));
    }

    #[test]
    fn empty_equal() {
        assert!(const_time_eq(b"", b""));
    }
}
