# 安全審計報告 — 加密方式與權限隔離

**日期**: 2026-04-14  
**審計範圍**: 非對稱加密機制、權限隔離（RBAC + 資料層 IDOR 防護）  
**審計員**: Claude Code AI Security Audit

---

## 一、加密方式審計結果

### 1.1 JWT 簽章 — ES256（ECDSA P-256）✅ 安全

| 項目 | 狀態 | 說明 |
|------|------|------|
| 演算法 | ✅ ES256 (非對稱) | `config.rs:169-183` — 使用 ECDSA P-256 取代 HS256 |
| Algorithm Binding | ✅ 明確指定 | `auth.rs:105` — `Validation::new(Algorithm::ES256)` 防止 algorithm substitution |
| Audience / Issuer | ✅ 已驗證 | `auth.rs:106-107` — `aud: "ipig-system"`, `iss: "ipig-backend"` |
| JWT Blacklist | ✅ 雙層撤銷 | `jwt_blacklist.rs` — Memory (RwLock) + PostgreSQL 雙層 |
| JTI | ✅ 唯一識別碼 | 每個 JWT 含 `jti` UUID，支援即時撤銷 |
| 金鑰管理 | ✅ Docker Secrets | `config.rs:35-48` — 支援 `_FILE` 後綴讀取 Docker Secrets |

### 1.2 密碼雜湊 — Argon2id ✅ 安全

| 項目 | 狀態 | 說明 |
|------|------|------|
| 演算法 | ✅ Argon2id | `password.rs:90-98` — 業界推薦的記憶體硬化 KDF |
| Salt | ✅ CSPRNG 生成 | `SaltString::generate(&mut OsRng)` |
| 密碼強度 | ✅ 強制驗證 | `password.rs:63-88` — ≥10 字元 + 大小寫 + 數字 + 弱密碼黑名單 |
| Timing Attack 防護 | ✅ Dummy Hash | `login.rs:70-78` — 使用者不存在時仍執行虛擬 Argon2 驗證 |

### 1.3 CSRF 防護 — HMAC-SHA256 ✅ 安全

| 項目 | 狀態 | 說明 |
|------|------|------|
| 模式 | ✅ Signed Double Submit Cookie | `csrf.rs:1-11` |
| 簽章 | ✅ HMAC-SHA256 | `csrf.rs:98-109` — 綁定 session ID |
| 比較 | ✅ Constant-Time | `csrf.rs:112-117` — 防止 timing side-channel |
| Session 綁定 | ✅ 強制認證 | `csrf.rs:142-152` — SEC-C6: 非豁免寫入端點要求已認證 session |

### 1.4 其他加密機制

| 機制 | 演算法 | 位置 | 狀態 |
|------|--------|------|------|
| Refresh Token | SHA-256(CSPRNG 256-bit) | `session.rs:210-240` | ✅ |
| Password Reset Token | CSPRNG 256-bit + SHA-256 | `password.rs:264-271` | ✅ |
| TOTP 2FA | TOTP-SHA1 + Argon2 Backup Codes | `two_factor.rs` | ✅ |
| API Key | SHA-256(CSPRNG 256-bit) | `ai_auth.rs:153-179` | ✅ |
| Audit Log Integrity | HMAC-SHA256（可選） | `audit.rs:17-26` | ✅ |
| Reauth Token | ES256 短期 JWT (5min) | `password.rs:178-218` | ✅ |
| 帳號鎖定 | PostgreSQL Advisory Lock | `login.rs:25-31` | ✅ 防 TOCTOU |
| TLS | rustls (reqwest) / native-tls (lettre) | `Cargo.toml` | ✅ |

### 1.5 加密方式總結

**評級: 優秀** — 系統全面採用非對稱加密（ES256）簽發 JWT，密碼使用 Argon2id，所有 token 採用 CSPRNG + SHA-256。加密實作符合 OWASP 最佳實踐。

---

## 二、權限隔離審計結果

### 2.1 RBAC 架構 ✅ 完善

- **角色模型**: `users → user_roles → roles → role_permissions → permissions`
- **權限載入**: auth middleware 從 DB 動態載入（非 JWT 嵌入），TTL 5 分鐘快取
- **Permission 巨集**: `require_permission!` 統一檢查
- **Admin 權限**: `has_permission()` 對 admin 直接回傳 true
- **Guest 隔離**: `guest_guard_middleware` 攔截 guest 使用者寫入

### 2.2 已修復的 CRITICAL 漏洞

#### 🔴 VULN-001: 報表端點缺少權限檢查（9 個端點）

**嚴重度**: CRITICAL  
**檔案**: `handlers/report.rs`  
**問題**: 9 個報表端點接受 `_current_user` 參數但未驗證任何權限，任何已認證使用者可存取全部財務報表。  
**修復**: 全部加入 `require_permission!(current_user, "erp.report.view")`

受影響端點：
- `GET /reports/stock-on-hand`
- `GET /reports/stock-ledger`
- `GET /reports/purchase-lines`
- `GET /reports/sales-lines`
- `GET /reports/cost-summary`
- `GET /reports/blood-test-cost`
- `GET /reports/purchase-sales-monthly`
- `GET /reports/purchase-sales-by-partner`
- `GET /reports/purchase-sales-by-category`

#### 🔴 VULN-002: 動物醫療記錄 IDOR（跨計畫資料洩漏）

**嚴重度**: CRITICAL  
**問題**: 多個動物子模組的 GET 端點未呼叫 `access::require_animal_access()`，任何已認證使用者可透過猜測 UUID 存取不屬於其計畫的動物醫療記錄。

**修復**: 全部加入 `access::require_animal_access()` 檢查

| 檔案 | 受影響端點 | 修復方式 |
|------|-----------|---------|
| `blood_test.rs` | list/get blood tests | 加入 `require_animal_access` |
| `surgery.rs` | list/get surgeries | 加入 `require_animal_access` |
| `weight_vaccination.rs` | list weights / list vaccinations | 加入 `require_animal_access` |
| `vet_recommendation.rs` | get by animal/observation/surgery | 加入 `require_animal_access` + `get_observation_animal_id` |
| `vet_advice.rs` | get/list vet advice | 加入 `require_animal_access` |
| `transfer.rs` | list/get transfers + vet evaluation | 加入 `require_animal_access` |

#### 🟡 VULN-003: 獸醫巡場報告無權限檢查

**嚴重度**: HIGH  
**檔案**: `handlers/animal/vet_patrol.rs`  
**問題**: list/get/create/update/delete 全部缺少權限檢查，任何已認證使用者可 CRUD 巡場報告。  
**修復**:
- list/get: 加入 `require_permission!(current_user, "animal.record.view")`
- create/update/delete: 加入 `require_permission!(current_user, "animal.vet.recommend")`

### 2.3 已確認安全的模組

| 模組 | 權限機制 | 狀態 |
|------|---------|------|
| 使用者管理 | `admin.user.*` + reauth token | ✅ |
| 角色管理 | `admin.role.*` + reauth token | ✅ |
| 觀察記錄 | `access::require_animal_access` | ✅ |
| 照護紀錄 | `access::require_animal_access` | ✅ |
| 計畫書 | `access::require_protocol_view_access` | ✅ |
| HR 請假 | `user_id` 過濾 + `hr.leave.view_all` | ✅ |
| HR 出勤 | `user_id` 過濾 | ✅ |
| HR 加班 | `user_id` 過濾 | ✅ |
| 通知 | `current_user.id` 過濾 | ✅ |
| MCP Keys | `user.id` 過濾 | ✅ |
| 使用者偏好 | `current_user.id` 過濾 | ✅ |
| 設備管理 | service 層權限檢查 | ✅ |
| 倉儲/庫存 | `erp.warehouse.*` / `erp.stock.*` | ✅ |
| 單據管理 | `erp.document.*` | ✅ |
| 系統設定 | `is_admin()` 檢查 | ✅ |
| 全庫匯出 | `admin.data.export` + reauth token | ✅ |
| 審計日誌 | `audit.logs.view` | ✅ |
| GLP 合規 | 各 `require_permission!` | ✅ |

### 2.4 攻擊場景（修復前）

**場景 1: 跨計畫醫療記錄存取**
1. 使用者 A 為計畫 X 成員
2. 使用者 B 為計畫 Y 成員
3. 使用者 B 呼叫 `GET /api/v1/animals/{計畫X動物ID}/blood-tests`
4. 修復前：回傳計畫 X 的血液檢查資料（❌ 資料洩漏）
5. 修復後：回傳 403 Forbidden（✅ 已阻斷）

**場景 2: 無權限查看財務報表**
1. 一般使用者（僅有 `animal.record.view` 權限）
2. 呼叫 `GET /api/v1/reports/cost-summary`
3. 修復前：回傳完整成本彙總報表（❌ 財務資料外洩）
4. 修復後：回傳 403 "Permission denied: requires erp.report.view"（✅ 已阻斷）

---

## 三、建議改善事項（非漏洞）

| # | 建議 | 優先級 | 說明 |
|---|------|--------|------|
| 1 | CSRF Secret 獨立配置 | MEDIUM | 目前 fallback 從 JWT 私鑰派生，建議正式環境設定獨立 CSRF_SECRET |
| 2 | Audit HMAC Key 強制啟用 | LOW | `audit_hmac_key` 目前為 Optional，正式環境建議強制設定 |
| 3 | 匯入模板下載權限 | LOW | `download_*_import_template()` 無權限檢查（partner/product/warehouse），但僅為空白範本，風險低 |

---

## 四、修改檔案清單

| 檔案 | 變更說明 |
|------|---------|
| `handlers/report.rs` | 9 個端點加入 `require_permission!(current_user, "erp.report.view")` |
| `handlers/animal/blood_test.rs` | list/get 加入 `access::require_animal_access` |
| `handlers/animal/surgery.rs` | list/list_with_recommendations/get 加入 `access::require_animal_access` |
| `handlers/animal/weight_vaccination.rs` | list weights/vaccinations 加入 `access::require_animal_access` |
| `handlers/animal/vet_recommendation.rs` | get by animal/observation/surgery 加入 access 檢查 |
| `handlers/animal/vet_advice.rs` | get/list 加入 `access::require_animal_access` |
| `handlers/animal/vet_patrol.rs` | 全部 5 個端點加入 `require_permission!` |
| `handlers/animal/transfer.rs` | list/get/vet_evaluation 加入 `access::require_animal_access` |
