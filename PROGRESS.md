# 專案進度

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
