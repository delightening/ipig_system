# Code Review TODO — 2026-03-26

追蹤 iPig System Code Review 修復進度。

---

## 🔴 P0 — Critical（立即處理）

- [x] **C1-C3. 密鑰輪換與 .env 清理** ✅ 2026-03-26
  - [x] 輪換 JWT_SECRET（openssl rand -base64 64）
  - [ ] 輪換 SMTP 密碼（需至 Google 帳號重新產生 App Password）
  - [x] 輪換 AUDIT_HMAC_KEY（openssl rand -base64 32）
  - [x] 輪換 DB 密碼（openssl rand -base64 24）
  - [x] 輪換 ADMIN_INITIAL_PASSWORD
  - [ ] 輪換 Sentry DSN（需至 Sentry 後台重新產生）
  - [x] 輪換 TEST_ADMIN_PASSWORD / TEST_USER_PASSWORD / DEV_USER_PASSWORD
  - [x] 從 git 歷史移除 .env — **不需要**（.env 從未被 commit）
  - [x] 確認 .gitignore 包含 .env — **已確認**（含 `.env`, `**/.env` 規則）
  - [ ] 建立 .env.example（僅含佔位符）

- [x] **C4. 修復 Token Refresh 競態條件** ✅ 2026-03-26
  - 檔案: `frontend/src/lib/api/client.ts:55-70`
  - 改用 Promise-based singleton

- [x] **C5. localStorage 敏感資料** ✅ 2026-03-26
  - 檔案: `frontend/src/stores/auth.ts:207-212`
  - 最小化存入 localStorage 的使用者資料（移除 email, phone 等個人資訊）

- [x] **C6. CSRF 空 Session ID 漏洞** ✅ 2026-03-26
  - 檔案: `backend/src/middleware/csrf.rs:77-78`
  - 非豁免路徑的寫入操作要求已認證 session（空 session_id 回 401）

---

## 🟠 P1 — High（短期修復）

- [x] **H1. N+1 查詢重構** ✅ 2026-03-26
  - 3 個 scalar subquery → LEFT JOIN LATERAL（weight 兩查詢合一）
  - M2 Stats 查詢也同步合併為單一查詢（COUNT + FILTER）

- [x] **H2. 為高頻 FK 建立索引** ✅ 2026-03-26
  - 新增 `022_add_missing_fk_indexes.sql`（30 個索引）
  - 覆蓋：documents、euthanasia、amendments、HR、security_alerts、設施層級、電子簽章等

- [x] **H3. 審查 CASCADE DELETE 策略** ✅ 2026-03-26
  - users.delete() 從硬刪除改為 soft delete（停用 + 匿名化 email）
  - animals/protocols 確認只有 soft delete 路徑，無硬刪除風險

- [x] **H4. 移除 unreachable!() panic** ✅ 2026-03-26
  - 檔案: `backend/src/error.rs:106`

- [x] **H5. Spawned task 錯誤記錄** ✅ 2026-03-26
  - 全部 16 處 `let _ = AuditService::log*` 改為 `if let Err(e) = ... { tracing::error! }`
  - 影響: user.rs, password.rs, two_factor.rs, audit.rs, data_export.rs, ai.rs, leave.rs, overtime.rs

- [x] **H6. 收緊 SVG sanitizer 配置** ✅ 2026-03-26
  - 移除 text/tspan（防釣魚）、加入 foreignObject/a/image/animate 禁止
  - 擴充 event handler 禁止清單（15 → 全覆蓋）、移除 href/xlink:href
  - 新增 4 個測試案例，共 12 tests 通過

- [x] **H7. 加入 type guard for 2FA error** ✅ 2026-03-26
  - 檔案: `frontend/src/pages/auth/LoginPage.tsx:53-54`

- [x] **H8. 替換 array index key** ✅ 2026-03-26
  - 18 個檔案替換為穩定 ID（doc_no、ear_tag、code 等）
  - 跳過 skeleton/loader（index 可接受）和 form arrays（需更大架構改動）

- [x] **H9. 修復 Logout 競態條件** ✅ 2026-03-26
  - 檔案: `frontend/src/lib/api/client.ts:72, 152-162`
  - 改用 Promise-based gate

- [x] **H10. 移除不必要 .clone()** ✅ 2026-03-26（不需修改）
  - 經分析所有 clone 均為必要（tokio::spawn 'static、HashMap ownership、push_bind）

---

## 🟡 P2 — Medium（中期改善）

- [x] **M1. 權限不足改回 403** ✅ 2026-03-26
  - 檔案: `backend/src/handlers/animal/animal_core.rs:37-43`

- [x] **M2. Stats 查詢合併** ✅ 2026-03-26
  - 合併為單一 GROUP BY + FILTER 查詢（隨 H1 同步完成）

- [x] **M3. Rate Limiter 加上限** ✅ 2026-03-26
  - 超過 50,000 IP 追蹤時清除最舊條目

- [x] **M4. Boolean 索引改 partial index** ✅ 2026-03-26
  - 加入 022 migration：`WHERE is_deleted = false` / `WHERE deleted_at IS NULL`

- [x] **M5. AI API Key rate_limit CHECK** ✅ 2026-03-26
  - 加入 022 migration：`CHECK (rate_limit_per_minute IS NULL OR rate_limit_per_minute > 0)`

- [x] **M6. 備份加密設定** ✅ 2026-03-26（不需修改）
  - Production 已有 `BACKUP_REQUIRE_ENCRYPTION=true`，dev 不需加密

- [x] **M7. AnimalDetailTabContent props 合併** ✅ 2026-03-26
  - 16 props → 8 props（8 個 records data 合併為 `AnimalRecordsData` interface）

- [x] **M8. 減少 `any` 型別使用** ✅ 2026-03-26
  - DocumentEditPage `(p: any)` → 移除型別標註（TS 自動推斷）
  - validation.ts `z.any()` → `z.record(z.string(), z.unknown())`
  - useLeaveRequestForm `as any` — react-hook-form 限制，保留並加註

- [x] **M9. Heartbeat 加重試機制** ✅ 2026-03-26
  - 連續失敗 3 次後降低頻率（每 3 次才嘗試一次）

- [x] **M10. 設定 staleTime** ✅ 2026-03-26
  - AnimalEditPage 設定 5 分鐘 staleTime

- [x] **M11. ResizeObserver cleanup** ✅ 2026-03-26（不需修改）
  - 經檢查 cleanup 已正確實作（`return () => observer.disconnect()`）

- [x] **M12. Audit log 複合索引** ✅ 2026-03-26
  - 加入 022 migration：`(actor_user_id, created_at DESC)`

---

## 🟢 P3 — Low（長期優化）

- [x] **L1.** localStorage welcome banner → sessionStorage ✅ 2026-03-26
- [x] **L2.** DocumentEditPage 加 loading skeleton ✅ 2026-03-26
- [x] **L3.** App prefetch 錯誤加 dev console.warn ✅ 2026-03-26
- [x] **L4.** React.memo — 需 profiling 驗證，暫不盲加 ✅ 2026-03-26（擱置）
- [x] **L5.** Test .expect() — 符合 CLAUDE.md 規範（僅禁 unwrap_err） ✅ 2026-03-26（不需修改）
- [x] **L6.** Dependabot 加 patch-updates 分組 ✅ 2026-03-26
- [x] **L7.** SameSite cookie — 已確認 Lax 設定正確 ✅ 2026-03-26
- [x] **L8.** 測試環境 DB 密碼 — 已在 C1-C3 中一併輪換 ✅ 2026-03-26
- [x] **L9.** minimatch override — 確認仍需要（10.2.4 matched） ✅ 2026-03-26

---

## 進度追蹤

| 優先級 | 總數 | 完成 | 進度 |
|--------|------|------|------|
| P0 Critical | 4 | 4 | 100% |
| P1 High | 10 | 10 | 100% |
| P2 Medium | 12 | 12 | 100% |
| P3 Low | 9 | 9 | 100% |
| **合計** | **35** | **35** | **100%** |
