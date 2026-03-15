# Code Review: iPig System

> **Reviewed:** 2026-03-15 | **Scope:** Full codebase (backend, frontend, infrastructure) | **Verdict:** 🟡 Request Changes

---

## Summary

iPig System 是一套架構完善、安全意識高的實驗動物管理平台。Backend (Rust/Axum) 約 60,000 行，Frontend (React/TypeScript) 結構清晰，Docker 部署配置完整。整體程式碼品質優良，但在上線前仍有數個 **Critical** 與 **High** 級別問題需修正，主要集中在基礎設施安全與部分程式碼品質面向。

---

## Critical Issues

| # | 位置 | 問題 | 嚴重度 |
|---|------|------|--------|
| 1 | `docker-compose.waf.yml:20` | WAF 設為 **DetectionOnly** 模式，攻擊僅記錄不阻擋。生產環境必須改為 `"On"` | 🔴 Critical |
| 2 | `.github/workflows/ci.yml:40-54` | CI 測試密碼硬編碼於 workflow 並提交至 Git：`JWT_SECRET`, `DEV_USER_PASSWORD`, `ADMIN_INITIAL_PASSWORD`。應改用 GitHub Secrets 並立即 rotate | 🔴 Critical |
| 3 | `docker-compose.yml:129` | Frontend web 服務 port 未綁定 localhost（`"${WEB_PORT:-8080}:8080"`），任何人可直接存取。應改為 `"127.0.0.1:${WEB_PORT:-8080}:8080"` | 🔴 Critical |
| 4 | `docker-compose.monitoring.yml:42` | Grafana 預設密碼為 `admin`，必須在 `.env` 強制設定強密碼 | 🔴 Critical |
| 5 | `backend/src/bin/create_admin.rs:30` | Admin 密碼 fallback 為硬編碼 `"admin123"`。應僅接受環境變數，無值時 panic 而非使用弱密碼 | 🔴 Critical |

---

## High Severity Issues

| # | 位置 | 問題 | 類別 |
|---|------|------|------|
| 1 | `docker-compose.prod.yml:112` | `WATCHTOWER_API_TOKEN` 以環境變數傳入，應改用 Docker Secrets | Security |
| 2 | `docker-compose.prod.yml:121` | SMTP 密碼以環境變數傳入，應改用 Docker Secrets | Security |
| 3 | `backend/src/services/file.rs:118-166` | 檔案上傳僅驗證 magic number，未做沙盒處理。ZIP/SVG 可能夾帶惡意內容 | Security |
| 4 | DB migrations (003, 007) | `audit_logs` 與 `user_activity_logs` 的 HMAC integrity hash 未以 DB trigger 強制驗證，攻擊者可竄改日誌 | Security |
| 5 | `scripts/backup/pg_backup.sh:17` | `PGPASSWORD` 以環境變數傳入，密碼暴露於 process listing。應改用 `.pgpass` 或 Docker Secrets | Security |
| 6 | `deploy/waf/REQUEST-900-EXCLUSION-RULES-BEFORE-CRS.conf` | WAF 排除規則過寬：移除了 `/api/auth/*` 的 SQL injection 防護、`/api/protocols\|observations\|surgeries` 的所有 XSS 防護 | Security |
| 7 | Backup encryption | GPG 加密為選用，備份可能以明文儲存。生產環境應強制加密 | Security |
| 8 | Docker images | 未使用 image digest pinning（如 `postgres:16-alpine@sha256:...`），存在 supply chain 攻擊風險 | Security |

---

## Medium Severity Issues

| # | 位置 | 問題 | 類別 |
|---|------|------|------|
| 1 | `backend/src/middleware/rate_limiter.rs` | Rate limiter 使用 in-memory DashMap，分散式部署時各 server 獨立計數，無法有效限流。建議改用 Redis | Performance |
| 2 | `backend/src/handlers/animal/animal_core.rs:44-62` | N+1 模式：先 fetch 所有動物再 in-memory filter，應將條件推入 SQL WHERE clause | Performance |
| 3 | `backend/src/services/file.rs:227+` | 大檔案（最大 100MB）完整載入記憶體後才驗證，應改為串流 (chunked) 驗證 | Performance |
| 4 | Backend 全域 | 約 728 處 `.unwrap()` 呼叫（非測試碼），其中多數安全但部分在 hot path 可能導致 panic | Correctness |
| 5 | `backend/src/middleware/csrf.rs` | CSRF token 非綁定 session/user，每次 response 重新產生。雖然 double-submit cookie 模式可接受，但安全性可再強化 | Security |
| 6 | `frontend/src/pages/admin/hooks/useUserManagement.ts` | `UserFormDialogs` 手動處理輸入，未使用 Zod validation，email 與 display name 無驗證 | Correctness |
| 7 | `frontend/src/components/ui/file-upload.tsx` | 前端檔案類型僅檢查副檔名（`.jpg`, `.png`），未驗證 MIME type | Security |
| 8 | Session timeout | 前端硬編碼 6 小時 session timeout，對敏感實驗動物資料而言偏長 | Security |
| 9 | `monitoring/prometheus/alert_rules.yml` | Alert 門檻過寬：HighErrorRate 5%（建議 1-2%）、HighP95Latency 2s（建議 500ms）、DbPoolSaturated 80%（建議 60%） | Operations |
| 10 | Prometheus / Grafana | 無 authentication 保護，metrics 與 dashboard 暴露於 internal network | Security |

---

## Low Severity / Suggestions

| # | 位置 | 建議 | 類別 |
|---|------|------|------|
| 1 | `backend/src/handlers/auth.rs` | 767 行，建議拆分（如 login、2fa、password_reset 各自獨立檔案） | Maintainability |
| 2 | `backend/src/services/auth.rs` | 1,038 行，超過 CLAUDE.md 規範的 300 行上限 | Maintainability |
| 3 | `backend/src/handlers/animal/signature.rs` | 995 行，建議按簽章類型拆分 | Maintainability |
| 4 | `backend/src/services/product.rs` | 1,326 行，建議拆分為 product_core / product_import / product_export | Maintainability |
| 5 | Frontend | 無外部 error tracking（如 Sentry），production 錯誤無法有效追蹤 | Operations |
| 6 | Frontend | Cookie consent banner 僅顯示資訊，未實際阻擋 Google Fonts 等第三方請求 | Compliance |
| 7 | Frontend | 密碼強度僅要求 6 字元，未強制複雜度規則 | Security |
| 8 | `docker-compose.prod.yml:109` | Watchtower poll interval 30 秒過於頻繁，生產建議 3600+ 秒 | Operations |
| 9 | DB migrations | 缺少 `login_events(email, event_type)` 複合索引，影響暴力破解偵測查詢效能 | Performance |
| 10 | DB migrations | JSONB 欄位（`equipment_used`, `treatments`, `before_data`, `after_data`）無 schema validation constraint | Correctness |

---

## What Looks Good

以下是這個專案做得特別好的地方：

- **密碼安全**：使用 Argon2 + SaltString::generate()，業界最佳實踐
- **JWT 管理**：雙層 blacklist（in-memory + PostgreSQL），background cleanup，mutex poisoning 時 fail-closed
- **CSRF 防護**：Double-submit cookie pattern，所有 state-changing request 自動附加 token
- **多層 Rate Limiting**：Auth 30/min、API 600/min、Write 120/min、File upload 30/min
- **稽核日誌**：HMAC-SHA256 完整性驗證、不可變 append-only 設計（需加 DB trigger 強制）
- **Config 驗證**：JWT_SECRET ≥ 32 字元、啟動時 fail fast、dev/prod 配置衝突檢測
- **權限系統**：`require_permission!` macro、50+ permission codes、角色與權限分離
- **Cookie 安全**：HttpOnly、SameSite=Lax、Secure flag 可配置
- **前端認證**：HttpOnly cookie（非 localStorage）、token refresh queue 防 race condition、reauth token 用於敏感操作
- **前端 XSS 防護**：DOMPurify + SVG-specific config 用於 dangerouslySetInnerHTML
- **網路隔離**：Docker 三層網路（frontend/backend/database）
- **SQL 安全**：全面使用 sqlx 參數化查詢，零字串拼接
- **型別安全**：sqlx compile-time query checking、Zod schema validation
- **CI 安全掃描**：cargo-audit、cargo-deny、Trivy container scanning
- **Code 規範**：CLAUDE.md 定義清楚的分層架構與量化門檻

---

## Verdict: 🟡 Request Changes

整體而言，iPig System 是一個**架構成熟、安全意識強**的專案。Backend Rust 程式碼品質高，Frontend 遵循現代 React 最佳實踐。然而，在上線前有 **5 個 Critical** 與 **8 個 High** 級別問題需要優先處理，主要集中在：

1. **基礎設施安全**（WAF 模式、port binding、密碼管理）
2. **Secret 管理**（CI 硬編碼密碼、環境變數洩漏）
3. **稽核完整性**（HMAC chain 未在 DB 層強制執行）

### 建議優先順序

| 時程 | 行動 |
|------|------|
| **立即（24hr）** | Rotate CI 硬編碼密碼、啟用 WAF blocking、綁定 web port 到 localhost、強制 Grafana 密碼 |
| **短期（1 週）** | 遷移所有 secret 至 Docker Secrets、強制備份加密、實作 audit log DB trigger |
| **中期（1 月）** | Redis rate limiter、N+1 query 修正、檔案上傳沙盒化、image digest pinning |
| **長期（3 月）** | 外部 error tracking (Sentry)、自動 secret rotation、mTLS service 通訊 |
