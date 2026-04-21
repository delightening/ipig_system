# Session 與登出功能整合說明

> **參考標準**：OWASP Session Management Cheat Sheet、OWASP Testing for Logout Functionality  
> **最後更新**：2026-03-06

本文件彙整本專案的登出／Session 機制，對照業界資安標準，並說明如何在維護資安的前提下避免使用者重複登入的困擾。

---

## 一、登出情境與業界對照總覽

| 登出情境 | 業界／資安參考 | 本專案現況 | 避免重複登入作法 |
|---------|----------------|-----------|------------------|
| **1. 使用者主動登出** | OWASP：必須 server-side 撤銷、清除 cookies | ✅ `POST /auth/logout`：JWT 黑名單、結束 sessions、撤銷 refresh token、清除 cookies | 使用者主動操作，無困擾 |
| **2. Session 逾時（無操作）** | OWASP：Idle timeout（15–30 分低風險、2–5 分高風險）、Server 端強制 | ✅ 前端 6 小時 + 60 秒前預警對話框；後端 `cleanup_expired` 依 `session_timeout_minutes` | ✅ Heartbeat 有操作時每 60 秒更新 `last_activity_at`，延長有效時間 |
| **3. Refresh token 失效（401）** | OWASP：token 過期後應導向登入，不再自動續期 | ✅ `clearAuth()` 清除前端狀態、導向登入（不 call 後端 logout） | 僅在 token 已失效時登出，屬預期；避免 503 誤判為登出 |
| **4. 管理員強制登出** | OWASP：支援 admin 撤銷可疑 session | ✅ `POST /admin/audit/sessions/:id/logout` 強制結束指定 session、audit log | 被踢出者下次操作即 401，需重新登入 |
| **5. 密碼變更後** | OWASP：敏感操作後建議撤銷其他 sessions | 需確認專案實作 | 可採「其他裝置下次操作才登出」以減少打斷 |
| **6. 帳號刪除（GDPR）** | OWASP / GDPR：停用帳號時結束所有 sessions | ✅ `delete_me_account` 結束所有 sessions、撤銷 refresh tokens、清除 cookies | 一次性停用，不影響日常 UX |
| **7. 503 等暫時性錯誤** | 業界：區分暫時錯誤 vs 認證失效，避免誤登出 | ✅ 503 會重試 2 次，不當成 logout；401 才觸發 clearAuth | 伺服器短暫不可用時不會誤登出 |
| **8. 多裝置／多 tab** | OWASP：多 tab/session 管理 | ✅ 多 tab 共用 cookies；heartbeat 更新當前 session；後端 `session_timeout_minutes` 可調 | 有操作的 tab 會續期； idle 裝置逾時登出 |

---

## 二、資安與 UX 平衡原則對照

| 原則 | 業界建議 | 本專案對應 |
|------|----------|-----------|
| **Server-side 撤銷** | 登出必須在 server 端撤銷 session/token | ✅ JWT blacklist、`user_sessions` 標記、`refresh_tokens` revoked_at |
| **逾時雙軌** | Idle timeout + Absolute timeout | ✅ Idle：heartbeat 更新；Absolute：`session_timeout_minutes` 可調 |
| **Cookie 清除** | 登出時覆寫並清除 cookies | ✅ `build_clear_cookie` 清除 access_token、refresh_token |
| **活動續期** | 有操作時延長 session，減少頻繁登入 | ✅ useHeartbeat：滑鼠／鍵盤／點擊／滾動每 60 秒發送 heartbeat |
| **預警與續期** | 逾時前給予續期選項，避免突兀中斷 | ✅ 60 秒前「登入即將過期」對話框，可點「繼續使用」呼叫 refresh |
| **防誤登出** | 區分暫時性錯誤 vs 認證失效 | ✅ 503 重試、401 才 clearAuth；`isLoggingOut` 鎖避免併行登出 |

---

## 三、技術實作摘要

### 3.1 前端

| 元件 | 職責 |
|------|------|
| `useAuthStore.logout()` | 呼叫 `POST /auth/logout`，清除 `user`、`isAuthenticated`、`sessionExpiresAt` |
| `useAuthStore.clearAuth()` | 僅清除前端 state，不呼叫後端（供 401 refresh 失敗時使用） |
| `SessionTimeoutWarning` | 監控 `sessionExpiresAt`，剩 60 秒時顯示對話框，可「續期」或「登出」 |
| `useHeartbeat` | 偵測使用者活動，約每 60 秒發送 `POST /auth/heartbeat` 更新 session 活躍時間 |
| `api.ts` interceptor | 401 時嘗試 refresh；refresh 失敗則 clearAuth；503 時重試，不當成登出 |

### 3.2 後端

| 元件 | 職責 |
|------|------|
| `handlers::auth::logout` | JWT blacklist、`LoginTracker::log_logout`、`SessionManager::end_all_sessions`、`AuthService::logout`、清除 cookies |
| `SessionManager::end_all_sessions` | 將該使用者所有 active sessions 標記為結束 |
| `SessionManager::force_logout` | 強制結束指定 session（管理員） |
| `SessionManager::cleanup_expired` | 依 `session_timeout_minutes` 將 idle 逾時 sessions 標記為 timeout |
| `AuthService::logout` | 將該使用者所有 `refresh_tokens` 設為 `revoked_at` |

### 3.3 設定參數

| 參數 | 位置 | 說明 |
|------|------|------|
| `SESSION_TIMEOUT_MS` | 前端 `auth.ts` | 前端逾時顯示用（6 小時） |
| `session_timeout_minutes` | `system_settings` | 後端 idle 逾時（預設 360） |
| `ACCESS_TOKEN_EXPIRY_HOURS` | 後端 `constants.rs` | Access token 24h |
| `REFRESH_TOKEN_EXPIRY_DAYS` | 後端 `constants.rs` | Refresh token 30d |

---

## 四、未來可優化項目

| 項目 | 建議 | 目的 |
|------|------|------|
| **密碼變更後** | 若尚未實作：撤銷其他 refresh token，或採「其他裝置下次操作才登出」 | 兼顧安全與 UX |
| **Session 數量限制** | 2FA 已有 session 數限制；一般帳號可考慮限制並行 session 數 | 降低帳號遭濫用風險 |
| **Token 壽命對齊** | Access 24h、Refresh 30d 與 `session_timeout_minutes` 不一致時，可文件化策略 | 避免行為與預期不符 |

---

## 五、相關文件

- [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- [OWASP Testing for Logout Functionality](https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/06-Session_Management_Testing/06-Testing_for_Logout_Functionality.html)
- [CREDENTIAL_ROTATION.md](./CREDENTIAL_ROTATION.md) — 憑證輪換
- [spec/07_SECURITY_AUDIT.md](../spec/07_SECURITY_AUDIT.md) — 安全稽核規格
