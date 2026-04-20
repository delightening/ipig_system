# iPig System Code Review — 2026-03-26

## 審查範圍
- Backend: Rust (Axum) — handlers, services, models, middleware
- Frontend: React 18 (Vite/TypeScript) — pages, components, stores, API client
- Infrastructure: Docker Compose, migrations, CI/CD, .env

---

## Critical (6 件)

### C1. .env 含明文密碼已提交版控
- **位置**: `.env` (Lines 8-14, 19, 24, 29, 34-37, 47)
- **問題**: DB密碼、JWT_SECRET、SMTP密碼、HMAC Key、Sentry DSN、管理員密碼全部暴露
- **風險**: 任何有 repo 存取權的人都能取得所有生產環境憑證
- **修復**: 輪換所有密鑰，從 git 歷史移除 .env

### C2. 預設管理員密碼暴露
- **位置**: `.env:24`
- **問題**: `ADMIN_INITIAL_PASSWORD=AdminIpig2026!` 在版控中可見
- **修復**: 輪換密碼，移至 Docker Secrets

### C3. Gmail SMTP 密碼 & Sentry DSN 暴露
- **位置**: `.env:34-37, 47`
- **問題**: 活躍的 Gmail 帳號密碼 & Sentry token 暴露
- **修復**: 立即輪換，改用環境變數注入

### C4. Token Refresh 競態條件
- **位置**: `frontend/src/lib/api/client.ts:55-70`
- **問題**: 模組級變數 `isRefreshing`、`refreshSubscribers` 在多個 401 同時觸發時可能導致重複 refresh
- **修復**: 使用 Promise-based singleton 或將 refresh 狀態存入 Zustand

### C5. localStorage 存放使用者敏感資料
- **位置**: `frontend/src/stores/auth.ts:207-212`
- **問題**: auth store 透過 Zustand persist 將 user 物件存入 localStorage，XSS 可讀取
- **修復**: 僅存最小必要資訊，考慮改用 sessionStorage

### C6. CSRF Token 空 Session ID
- **位置**: `backend/src/middleware/csrf.rs:77-78`
- **問題**: 未認證使用者的 session_id 為空字串，CSRF token 可跨 session 重用
- **修復**: 未認證時使用 request-specific nonce，或要求認證後才接受 CSRF 保護請求

---

## High (10 件)

### H1. N+1 查詢 — 動物列表
- **位置**: `backend/src/services/animal/core/query.rs:120-177`
- **問題**: 每行 3 個 scalar subquery (vet_recommendations, weight, measure_date)
- **修復**: 改用 CTE 或 JOIN with DISTINCT ON

### H2. 39 個 Foreign Key 缺少索引
- **位置**: `backend/migrations/004_animal_management.sql` 等
- **問題**: DELETE/UPDATE parent row 時全表掃描
- **修復**: 為所有 FK 欄位建立索引

### H3. 過度 CASCADE DELETE
- **位置**: 所有 migrations — 39 個 CASCADE 關係
- **問題**: 刪除單一父記錄靜默級聯刪除整棵依賴樹
- **修復**: 關鍵表使用 RESTRICT，實作 soft delete

### H4. `unreachable!()` 可 panic
- **位置**: `backend/src/error.rs:106`
- **問題**: `AppError::DuplicateWarning` 分支標記 unreachable 但實際可達
- **修復**: 移除或替換為適當錯誤處理

### H5. Spawned task 錯誤被靜默丟棄
- **位置**: `backend/src/handlers/user.rs:186-194, 204-212`
- **問題**: `let _ = AuditService::log_activity(...).await;` 錯誤被忽略
- **修復**: 加上 `if let Err(e) = ... { tracing::error!(...) }`

### H6. dangerouslySetInnerHTML SVG 注入風險
- **位置**: `frontend/src/components/animal/SacrificeFormDialog.tsx:385`, `ui/handwritten-signature-pad.tsx:176`
- **問題**: DOMPurify SVG_CONFIG 允許過多屬性
- **修復**: 使用更嚴格的白名單配置

### H7. Type assertion 無驗證
- **位置**: `frontend/src/pages/auth/LoginPage.tsx:53-54`
- **問題**: `error as { is2FA?: boolean; tempToken?: string }` 無型別檢查
- **修復**: 使用 type guard function

### H8. 陣列 index 當 React key
- **位置**: 7+ 個檔案 (BloodTestFormDialog:338, ImportDialog:304, TwoFactorSetup:201 等)
- **問題**: 列表重排時導致 state 洩漏、focus 錯誤
- **修復**: 使用唯一穩定 ID

### H9. Logout 競態條件
- **位置**: `frontend/src/lib/api/client.ts:72, 152-162`
- **問題**: `isLoggingOut` flag 非原子操作，多個 401 可同時進入 cleanup
- **修復**: 使用 Promise-based gate

### H10. 多餘 `.clone()` 在熱路徑
- **位置**: handlers (40+ 處), `services/animal/core/query.rs:30,77,256`
- **問題**: 頻繁呼叫的 handler 中不必要的 string clone
- **修復**: 改用引用傳遞

---

## Medium (12 件)

### M1. 權限不足回空列表而非 403
- **位置**: `backend/src/handlers/animal/animal_core.rs:37-43`
- **修復**: 回傳 `AppError::Forbidden`

### M2. Animal Stats 兩次查詢可合一
- **位置**: `backend/src/services/animal/core/query.rs:61-97`
- **修復**: 合併為單一查詢

### M3. Rate Limiter HashMap 無上限
- **位置**: `backend/src/middleware/rate_limiter.rs:41-50`
- **修復**: 加 LRU eviction 或最大容量限制

### M4. Boolean 索引效率差
- **位置**: `migrations/004_*` — `idx_animals_is_deleted`
- **修復**: 改用 partial index `WHERE is_deleted = false`

### M5. AI API Key rate_limit 無 CHECK constraint
- **位置**: `backend/migrations/017_ai_api_keys.sql:22`
- **修復**: 加 `CHECK (rate_limit_per_minute > 0)`

### M6. 備份未加密（dev 環境）
- **位置**: `docker-compose.yml:239`
- **修復**: 設定 BACKUP_GPG_RECIPIENT

### M7. Prop drilling 13 props
- **位置**: `frontend/src/pages/animals/components/AnimalDetailTabContent.tsx:100-136`
- **修復**: 建立 AnimalDetailContext

### M8. `any` 型別 293 處
- **位置**: 88 個檔案
- **修復**: 定義適當 interface/type

### M9. Heartbeat 無重試
- **位置**: `frontend/src/hooks/useHeartbeat.ts:28-40`
- **修復**: 加 failure count 追蹤與 exponential backoff

### M10. Stale query 未設 staleTime
- **位置**: `frontend/src/pages/animals/AnimalEditPage.tsx:69-75`
- **修復**: 設定 `staleTime: 5 * 60 * 1000`

### M11. ResizeObserver 潛在洩漏
- **位置**: `frontend/src/components/ui/handwritten-signature-pad.tsx:81-117`
- **修復**: Track observers in Set，cleanup 時全部 disconnect

### M12. Audit log partition pruning 不佳
- **位置**: `backend/migrations/007_audit_erp.sql:6-36`
- **修復**: 加複合索引 `(actor_user_id, created_at)`

---

## Low (9 件)

### L1. localStorage 存 UI 狀態 (welcome banner)
- **位置**: `frontend/src/pages/DashboardPage.tsx:50-51`

### L2. 缺 loading skeleton
- **位置**: `frontend/src/pages/documents/DocumentEditPage.tsx:94-100`

### L3. console.error 被吞 (App prefetch)
- **位置**: `frontend/src/App.tsx:132-136`

### L4. 缺 React.memo 在 presentational components
- **位置**: 多個 table/list 元件

### L5. Test 用 `.expect()` 可改 assert
- **位置**: `backend/src/models/animal/mod.rs`, `middleware/real_ip.rs`

### L6. Dependabot 頻率可更積極
- **位置**: `.github/dependabot.yml`

### L7. SameSite cookie 未顯式設定
- **位置**: 需確認 backend config

### L8. 測試環境 DB 密碼過弱
- **位置**: `docker-compose.test.yml:10`

### L9. Unused npm dependency override
- **位置**: `frontend/package.json:119-121`
