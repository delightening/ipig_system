// 啟動時安全檢查（H7 等）

/// H7 (ISO A.8.24)：檢查 JWT 私鑰檔案權限是否過鬆。
///
/// 適用情境：使用者透過 `JWT_EC_PRIVATE_KEY_FILE` env 指定檔案路徑（Docker
/// Secrets / 自管 Vault）載入金鑰。檔案權限若 group/other 可讀，私鑰可能被
/// 同主機其他使用者讀走 → 簽發任意 JWT。
///
/// 行為：
/// - Unix：檢查 mode；group/other 任一非零 → 印 warn（含建議 `chmod 600`）
/// - Windows / 非 Unix：印 info skip（NTFS ACL 檢查不在本檢查範圍）
/// - `path = None`：靜默略過（key 由 PEM env 直接提供）
///
/// 不阻擋啟動 — 已部署的環境若誤設不應因檢查中斷服務；改 prod-fail 留待 Ops
/// 共識後另一 PR。
///
/// 路徑由 `Config::jwt_ec_private_key_file` 提供（CLAUDE.md：禁止散落 std::env::var）。
pub fn check_jwt_key_file_permissions(path: Option<&str>) {
    let path = match path {
        Some(p) if !p.is_empty() => p,
        _ => return,
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(path) {
            Ok(meta) => {
                let mode = meta.permissions().mode() & 0o777;
                if mode & 0o077 != 0 {
                    tracing::warn!(
                        "[Security/H7] JWT 私鑰檔 {} 權限過鬆（mode={:o}），\
                         group/other 可讀。建議：chmod 600 {} && \
                         chown <api-uid> {}",
                        path,
                        mode,
                        path,
                        path
                    );
                } else {
                    tracing::info!(
                        "[Security/H7] ✓ JWT 私鑰檔 {} 權限符合（mode={:o}）",
                        path,
                        mode
                    );
                }
            }
            Err(e) => {
                tracing::warn!(
                    "[Security/H7] 無法讀取 JWT 私鑰檔 {} 的 metadata: {} \
                     (檔案不存在？權限不足？)",
                    path,
                    e
                );
            }
        }
    }

    #[cfg(not(unix))]
    {
        tracing::info!(
            "[Security/H7] jwt_ec_private_key_file={} 已設定；非 Unix 平台，\
             權限檢查 skipped（請手動確認 ACL 限制 read 至 API 進程帳號）",
            path
        );
    }
}
