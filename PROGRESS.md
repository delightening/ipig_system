# 專案進度

## 2026-02-23 正式上線準備 — 第三批基礎建設

### 已完成（4 項）

#### 7.2 可觀測性
- ✅ Prometheus `/metrics` 端點（HTTP 指標 + DB Pool 狀態）

#### 7.1 測試覆蓋率
- ✅ 前端 Vitest 測試框架初始化（vitest.config.ts + smoke test + CI 整合）
- ✅ Playwright E2E 框架初始化（3 瀏覽器 + 登入流程範例測試）

#### 7.3 備份與災難復原
- ✅ 災難復原手冊 `DR_RUNBOOK.md`（RPO/RTO 目標 + 多情境 SOP + 演練記錄表）

### 變更檔案
- `backend/Cargo.toml`（新增 metrics 依賴）
- `backend/src/handlers/metrics.rs`（新增）
- `backend/src/handlers/mod.rs`
- `backend/src/routes.rs`
- `backend/src/main.rs`（PrometheusBuilder 初始化）
- `frontend/package.json`（新增 vitest/playwright 依賴與腳本）
- `frontend/vitest.config.ts`（新增）
- `frontend/src/__tests__/setup.ts`（新增）
- `frontend/src/__tests__/smoke.test.ts`（新增）
- `frontend/playwright.config.ts`（新增）
- `frontend/e2e/login.spec.ts`（新增）
- `.github/workflows/ci.yml`（新增 Vitest CI step）
- `DR_RUNBOOK.md`（新增）
- `description.md`（更新 §7 checklist）

## 2026-02-23 正式上線準備 — Beta 階段基礎建設

### 已完成（5 項）

#### 7.3 備份與災難復原
- ✅ 備份 GPG 加密（`pg_backup.sh` + `Dockerfile.backup` 安裝 gnupg）
- ✅ `/uploads` 目錄異地備份（整合至 rsync 流程）
- ✅ GeoIP 自動更新腳本（`scripts/update_geoip.sh` + SHA256 驗證 + 舊版備份）

#### 7.5 GLP 合規
- ✅ 稽核紀錄不可刪改驗證（確認無 audit delete API，HMAC 完整性已實作）

#### 7.7 使用者文件
- ✅ 管理員部署/維運手冊 `DEPLOYMENT.md`（系統需求、部署、備份還原、監控、故障排除、安全維護）

### 變更檔案
- `scripts/backup/pg_backup.sh`（GPG 加密 + uploads rsync）
- `scripts/backup/Dockerfile.backup`（安裝 gnupg）
- `scripts/update_geoip.sh`（新增）
- `docker-compose.yml`（新增 BACKUP_GPG_RECIPIENT + uploads volume）
- `DEPLOYMENT.md`（新增）
- `.env.example`（新增環境變數）
- `description.md`（更新 §7 checklist 狀態）

## 2026-02-23 正式上線準備 — Alpha 階段基礎建設

### 已完成（5 項）

#### 7.2 可觀測性
- ✅ 新增 `GET /api/health` 健康檢查端點（DB 連通性 + 延遲量測，200/503 回應）
- ✅ 條件式 JSON 結構化日誌（`RUST_LOG_FORMAT=json` 環境變數啟用，預設保持文字格式）
- ✅ Request ID 全鏈路追蹤（`X-Request-Id` 自動產生與傳遞）

#### 7.4 安全性補強
- ✅ CI 新增 `npm audit` 掃描 job（`audit-level=high`）
- ✅ CI 新增 Trivy 容器安全掃描 job（僅 push 至 main 觸發，掃描 backend + frontend 映像）

#### 7.6 效能基準
- ✅ Nginx gzip 壓縮策略強化（壓縮等級 6、啟用 Vary、最小 1024 bytes、新增 font/svg 類型）
- ✅ Nginx 靜態資源快取區塊保留 `X-Content-Type-Options` 安全標頭

### 變更檔案
- `backend/src/handlers/health.rs`（新增）
- `backend/src/handlers/mod.rs`
- `backend/src/routes.rs`
- `backend/src/main.rs`
- `.github/workflows/ci.yml`
- `frontend/nginx.conf`

## 2026-02-23 修復 Clippy Linting 錯誤

### 已完成
- ✅ `derivable_impls`：`user.rs`、`product.rs`、`storage_location.rs` 改用 `#[derive(Default)]`
- ✅ `ptr_arg`：`requests.rs` `validate_pen_location` 改為 `&str`
- ✅ `upper_case_acronyms`：`enums.rs` 加 `#[allow(clippy::upper_case_acronyms)]`
- ✅ `useless_format`：`numbering.rs`、`import_export.rs` 移除無效 `format!()`
- ✅ `get_first`：`import_export.rs` `.get(0)` → `.first()`
- ✅ `iter_cloned_collect`：`status.rs` → `.to_vec()`
- ✅ `unused_enumerate_index`：`import_export.rs` 移除未使用的 enumerate
- ✅ `no_effect_replace`：`product.rs` 移除無效 `.replace("___", "___")`
- ✅ `unwrap_used`：所有測試 `.unwrap()` → `.expect()` 並附描述訊息
- ✅ 測試更新：`validate_pen_location` 測試配合 `&str` 簽名變更

## 2026-02-17 修復「返回管理員」按鈕導致管理員登出

### 問題
管理員模擬登入其他使用者後，點擊「← 返回管理員」會導致管理員被登出。

### 根因
1. 前端 `stopImpersonating` 呼叫 `/auth/logout` 銷毀所有 token（邏輯錯誤）
2. Nginx `proxy_buffer_size` 不足，JWT Set-Cookie header 過大導致 `upstream sent too big header` → 502

### 已完成
- ✅ 後端新增 `POST /auth/stop-impersonate` API（handler + service + route）
- ✅ 後端新增 `AuthService::impersonate_restore` 方法（恢復管理員 token）
- ✅ 後端新增 `AuditAction::StopImpersonate` 稽核動作
- ✅ 前端 `stopImpersonating` 改呼叫新 API，恢復 session 後導向首頁
- ✅ Nginx 加大 `proxy_buffer_size 16k` / `proxy_buffers 4 16k` 修復 502
- ✅ Docker build 成功（api + web），容器重啟驗證通過

## 2026-02-14 資安分析與修復

### 已完成（11 項修復）

#### P0 嚴重
- ✅ SEC-01：Refresh Token 雜湊改用 SHA-256
- ✅ SEC-03：移除硬編碼管理員密碼，改環境變數
- ✅ SEC-06：開發帳號設 `must_change_password=true`

#### P1 高風險
- ✅ SEC-04：API Rate Limiting（auth 10次/分，一般 120次/分）
- ✅ SEC-07：Nginx 新增 5 個安全標頭
- ✅ SEC-08：Docker 後端容器改用非 root 用戶運行
- ✅ SEC-10：新增密碼強度驗證

#### P2 中風險
- ✅ SEC-09：JWT Access Token 有效期 24h → 1h
- ✅ SEC-11：Impersonate 安全增強（JWT impersonated_by + 30min + 審計日誌）
- ✅ SEC-16：隱藏 Nginx 版本號
- ✅ SEC-02：Token 從 localStorage 改存 HttpOnly Cookie（前後端完整遷移）

### 依賴版本升級
- ✅ Node.js 20 → 22 LTS（前端 Dockerfile + docker-compose）
- ✅ Docker 映像固定版本：rust:1.92、nginx:1.28-alpine
- ✅ cargo update：84 個 patch 更新
- ✅ npm update：38 個 packages 更新
- ✅ Docker 重建並部署成功

### 驗證
- ✅ 後端 `cargo check` 編譯通過
- ✅ Docker build 成功（api + web）
- ✅ 所有容器健康運行（ipig-db healthy、ipig-api up、ipig-web up）
- ✅ API 回應正常（status=200）

## 2026-02-14 P1-3 後端 Service 模組拆分

### 已完成
- ✅ `notification.rs`（1,737 行）→ 11 子模組
- ✅ `hr.rs`（1,453 行）→ 5 子模組
- ✅ `email.rs`（1,192 行）→ 4 子模組
- ✅ 三次 `cargo check` 全部通過，零編譯錯誤

## 2026-02-14 P1-4 後端 Handler 模組拆分

### 已完成
- ✅ `animal.rs`（1,952 行）→ 12 子模組（pig、source、observation、surgery、weight_vaccination、sacrifice_pathology、vet_recommendation、import_export、blood_test、dashboard）
- ✅ `protocol.rs`（1,081 行）→ 3 子模組（crud、review、export）
- ✅ `hr.rs`（881 行）→ 5 子模組（attendance、overtime、leave、balance、dashboard）
- ✅ 所有 `cargo check` 通過，零編譯錯誤

## 2026-02-14 P1-2 前端 api.ts 型別拆分

### 已完成
- ✅ `api.ts`（1,538 行）→ ~150 行（僅保留 axios 配置 + API 函數 + re-export）
- ✅ 建立 9 個型別檔：auth、erp、animal、aup、report、audit、notification、amendment、upload
- ✅ 更新 `types/index.ts` 統一 re-export
- ✅ 修正 `common.ts` 重複型別衝突（User、LoginResponse、UploadResponse、Role、UserTraining）
- ✅ `npx tsc --noEmit` 通過，零錯誤

## 2026-02-14 P2-5 Rust 單元測試

### 已完成
- ✅ 新增 48 個純函數單元測試（含既有共 54 個）
- ✅ 覆蓋範圍：`auth.rs`（密碼/hash/token）、`file.rs`（MIME/副檔名/類別）、`models/user.rs`（enum serde）、`models/animal.rs`（enum/驗證器）、`models/mod.rs`（分頁計算）
- ✅ `cargo test` 54 passed / 0 failed

## 2026-02-14 P2-6 sqlx migrate 確認

### 已完成
- ✅ 確認數字前綴（001_~010_）為 sqlx 合法格式，無需轉換
- ✅ `sqlx::migrate!` 巨集正常運作

## 2026-02-14 P2-7 CI/CD Pipeline

### 已完成
- ✅ 建立 `.github/workflows/ci.yml`
- ✅ 4 個 Jobs：backend-check、backend-test、backend-lint、frontend-check
- ✅ 使用 Cargo / npm 快取加速
- ✅ `SQLX_OFFLINE=true` 避免 CI 需要 PostgreSQL
