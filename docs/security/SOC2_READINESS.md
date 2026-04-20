# SOC 2 合規準備度評估

> **版本**：1.0  
> **建立日期**：2026-03-01  
> **參考**：2017 Trust Services Criteria (Revised Points of Focus 2022)

本文件對照 **Trust Services Criteria (TSC)**，評估 iPig 系統的合規準備度，供未來 SOC 2 認證參考。  
**注意**：本文件為自我評估，非正式稽核報告。正式認證需由獨立稽核機構執行。

---

## 1. 總覽

| 準則 | 說明 | 現況 | 待補強 |
|------|------|------|--------|
| **Security** | 保護系統與資料免於未授權存取（必選） | 大部分滿足 | 憑證輪換機制、SOC 2 正式認證 |
| **Availability** | 系統與資料於需要時可取得 | 大部分滿足 | SLA 文件化、災難復原演練 |
| **Processing Integrity** | 資料處理準確、完整、有效、及時 | 滿足 | 持續監控 |
| **Confidentiality** | 機密資訊存取限制 | 滿足 | — |
| **Privacy** | 個人資料保護與權利 | 部分滿足 | GDPR 資料主體權利（P1-M2） |

---

## 2. Security（安全 — 必選）

### 2.1 控制環境

| 要點 | iPig 現況 | 狀態 |
|------|-----------|------|
| 組織結構與責任 | `docs/operations/OPERATIONS.md` 定義服務擁有者、on-call、升級流程 | ✓ |
| 權限與職責分離 | RBAC、角色權限、敏感操作二級認證 | ✓ |
| 承諾誠信與價值觀 | 隱私政策、Cookie 同意、資料保留政策 | ✓ |

### 2.2 通訊

| 要點 | iPig 現況 | 狀態 |
|------|-----------|------|
| 內部溝通 | 維運文件、架構文件、部署手冊 | ✓ |
| 外部溝通 | 隱私政策、服務條款、Cookie 同意橫幅 | ✓ |

### 2.3 風險評估

| 要點 | iPig 現況 | 狀態 |
|------|-----------|------|
| 風險識別 | 安全配置檢查、WAF、依賴掃描（cargo audit、npm audit） | ✓ |
| 風險分析 | 文件化威脅與對策（如 DEPLOYMENT、WAF、ELECTRONIC_SIGNATURE_COMPLIANCE） | ✓ |

### 2.4 監控

| 要點 | iPig 現況 | 狀態 |
|------|-----------|------|
| 持續監控 | Prometheus、Grafana、Alertmanager、`/api/health`、`/api/metrics` | ✓ |
| 異常偵測 | 安全警報、登入異常偵測、可疑活動標記 | ✓ |

### 2.5 控制活動

| 要點 | iPig 現況 | 狀態 |
|------|-----------|------|
| 邏輯存取 | JWT + Refresh Token、TOTP 2FA、RBAC、權限檢查 | ✓ |
| 實體存取 | 容器化部署、非 root 容器、三層網路隔離 | ✓ |
| 系統作業 | Graceful shutdown、健康檢查、結構化日誌 | ✓ |
| 變更管理 | Migration 腳本、版本控管、CI/CD | ✓ |

### 2.6 邏輯與實體存取控制

| 要點 | iPig 現況 | 狀態 |
|------|-----------|------|
| 身分驗證 | Argon2 密碼雜湊、JWT、TOTP 2FA、HttpOnly Cookie | ✓ |
| 授權 | RBAC、`require_permission!`、IDOR 防護 | ✓ |
| 存取限制 | Rate Limiting（API/Auth/Write/Upload 分級）、CSRF | ✓ |
| 憑證管理 | Docker Secrets、`read_secret()`、`CREDENTIAL_ROTATION.md` | ✓ |
| 憑證輪換 | 文件化流程；無 90 天自動輪換機制 | 🟡 待補強 |

### 2.7 風險緩解

| 要點 | iPig 現況 | 狀態 |
|------|-----------|------|
| 威脅防護 | WAF（ModSecurity + OWASP CRS）、DOMPurify、輸入驗證 | ✓ |
| 備份與還原 | GPG 加密備份、pg_restore 驗證、DR Runbook | ✓ |

---

## 3. Availability（可用性）

| 要點 | iPig 現況 | 狀態 |
|------|-----------|------|
| 系統可用性 | 健康檢查、監控告警、容器自動重啟 | ✓ |
| 容量規劃 | DB pool 指標、k6 效能基準、`PERFORMANCE_BENCHMARK.md` | ✓ |
| 災難復原 | 備份腳本、還原驗證、`DB_ROLLBACK.md` | ✓ |
| SLA 文件化 | 無正式 SLA 或 RTO/RPO 定義 | 🟡 待補強 |
| 災難復原演練 | 文件化流程；無定期演練紀錄 | 🟡 待補強 |

---

## 4. Processing Integrity（處理完整性）

| 要點 | iPig 現況 | 狀態 |
|------|-----------|------|
| 資料準確性 | 輸入驗證（Zod）、Optimistic Locking、交易處理 | ✓ |
| 處理授權 | RBAC、簽章前密碼驗證、敏感操作二級認證 | ✓ |
| 稽核軌跡 | `user_activity_logs`、`audit_logs`、HMAC 驗證鏈 | ✓ |
| 21 CFR Part 11 | 電子簽章合規、`ELECTRONIC_SIGNATURE_COMPLIANCE.md` | ✓ |

---

## 5. Confidentiality（機密性）

| 要點 | iPig 現況 | 狀態 |
|------|-----------|------|
| 機密資訊識別 | 密碼、JWT、TOTP secret 等敏感資料 | ✓ |
| 存取限制 | RBAC、權限檢查、敏感欄位加密（TOTP secret） | ✓ |
| 傳輸保護 | HTTPS、COOKIE_SECURE、CORS 限制 | ✓ |
| 儲存保護 | Docker Secrets、Argon2、非 root 容器 | ✓ |

---

## 6. Privacy（隱私）

| 要點 | iPig 現況 | 狀態 |
|------|-----------|------|
| 隱私政策 | `PrivacyPolicyPage`、Cookie 同意 | ✓ |
| 同意管理 | Cookie 同意橫幅、localStorage 記憶 | ✓ |
| 資料主體權利 | 無 `GET /me/export`、`DELETE /me/account` | 🔴 待實作（P1-M2） |
| 資料保留 | `docs/` 資料保留政策、稽核日誌 10 年 | ✓ |
| 個人資料處理 | 最小化收集、目的限制 | ✓ |

---

## 7. 待補強項目彙總

| 優先級 | 項目 | 說明 |
|--------|------|------|
| P1 | GDPR 資料主體權利 | `GET /me/export`、`DELETE /me/account`（P1-M2） |
| P2 | 憑證輪換自動化 | 90 天輪換提醒或半自動化（目前僅文件化） |
| P2 | SLA / RTO / RPO | 正式定義並文件化 |
| P3 | 災難復原演練 | 定期演練並紀錄 |
| P3 | SOC 2 正式稽核 | 委託獨立稽核機構 |

---

## 8. 相關文件

| 文件 | 用途 |
|------|------|
| [OPERATIONS.md](../operations/OPERATIONS.md) | 維運、on-call、故障排除 |
| [CREDENTIAL_ROTATION.md](CREDENTIAL_ROTATION.md) | 憑證輪換流程 |
| [DEPLOYMENT.md](../DEPLOYMENT.md) | 部署、備份、監控 |
| [ELECTRONIC_SIGNATURE_COMPLIANCE.md](ELECTRONIC_SIGNATURE_COMPLIANCE.md) | 21 CFR Part 11 合規 |
| [ARCHITECTURE.md](../ARCHITECTURE.md) | 系統架構 |
| [WAF.md](WAF.md) | WAF 部署與規則 |

---

*文件產出於 2026-03-01，依據 2017 Trust Services Criteria 與 iPig 現況撰寫。*
