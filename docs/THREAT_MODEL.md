# iPig System Threat Model

> **版本**: 1.0  
> **日期**: 2026-04-14  
> **目的**: 驅動安全審計的分類與優先排序，而非事後追溯

---

## 1. 系統描述

iPig 是一個 GLP 合規的實驗動物管理系統，部署於單一機構內部（非多租戶 SaaS）。

**資產清單**（按敏感度排序）：

| 資產 | CIA 權重 | 說明 |
|------|----------|------|
| 動物醫療記錄 | C:高 I:高 A:中 | GLP 法規要求的實驗數據完整性 |
| 實驗計畫書 | C:高 I:高 A:中 | 包含未公開研究方法，審核流程需不可竄改 |
| 使用者憑證 | C:高 I:高 A:高 | 密碼、2FA secret、session token |
| 電子簽章 | C:中 I:高 A:中 | 法律效力，不可偽造或重放 |
| 審計日誌 | C:低 I:高 A:高 | GLP 合規的證據鏈，完整性 > 機密性 |
| 財務/ERP 數據 | C:中 I:中 A:中 | 進銷存、會計報表 |
| HR 個資 | C:高 I:中 A:低 | 員工出勤、請假、薪資相關 |

## 2. 威脅行為者（Threat Actors）

| 代號 | 描述 | 動機 | 能力 |
|------|------|------|------|
| **T1** | 惡意內部人員（已認證的 PI/研究員） | 存取不屬於自己的計畫數據；竄改審核結果 | 有效帳號、了解系統業務流程 |
| **T2** | 被入侵的 Admin 帳號 | 全系統控制；資料外洩 | Admin 權限 + 可模擬其他使用者 |
| **T3** | 外部攻擊者（無帳號） | 取得初始存取；帳號枚舉 | 網路存取、自動化工具 |
| **T4** | 被入侵的合作夥伴帳號（低權限） | 橫向移動至高權限資料 | 受限帳號（PI 或 EXPERIMENT_STAFF） |
| **T5** | 離職員工 | 殘留存取權；資料攜出 | 可能持有舊 token 或知道密碼 |

## 3. 攻擊面與風險矩陣

### 3.1 身分與存取管理

| ID | 威脅 | 行為者 | 影響 | 現有防護 | 殘留風險 | 嚴重度 |
|----|------|--------|------|----------|----------|--------|
| IAM-1 | JWT 偽造 | T3 | 假冒任意使用者 | ES256 非對稱簽章 + algorithm binding | 需私鑰外洩才可能 | **低**（架構防護） |
| IAM-2 | 自我提權（PUT /me） | T1,T4 | 變成 SYSTEM_ADMIN | role_ids/is_internal/expires_at 遮蔽 | **已修復** | **已緩解** |
| IAM-3 | Admin 橫向模擬 | T2 | 以另一 admin 身分操作 | 禁止模擬 admin 角色 | **已修復** | **已緩解** |
| IAM-4 | 角色指派越權 | T2 | 非 SYSTEM_ADMIN 指派 SYSTEM_ADMIN | actor 必須是 SYSTEM_ADMIN + role_id 存在性驗證 | **已修復** | **已緩解** |
| IAM-5 | Session 殘留 | T5 | 離職後仍可存取 | JWT blacklist + session manager + account deactivation | 依賴即時停用帳號 | **中** |
| IAM-6 | Permission cache 時間窗 | T2 | 降權後短暫保留舊權限 | 角色變更時立即清除快取 | <1秒 race window | **低** |

### 3.2 資料存取控制（IDOR）

| ID | 威脅 | 行為者 | 影響 | 現有防護 | 殘留風險 | 嚴重度 |
|----|------|--------|------|----------|----------|--------|
| DAC-1 | 跨計畫動物記錄存取 | T1,T4 | 看到別人計畫的醫療數據 | `access::require_animal_access()` 逐端點加入 | **已修復** — 但依賴逐端點人工加入 | **中**（架構缺口） |
| DAC-2 | 報表越權存取 | T1,T4 | 看到全系統財務數據 | `require_permission!(erp.report.view)` | **已修復** | **已緩解** |
| DAC-3 | 未來新端點遺漏 access check | T1 | 新 handler 忘記加權限 | 無全域 policy layer | **高**（架構缺口） | **高** |

### 3.3 業務邏輯

| ID | 威脅 | 行為者 | 影響 | 現有防護 | 殘留風險 | 嚴重度 |
|----|------|--------|------|----------|----------|--------|
| BIZ-1 | 研究員自行加入不屬於的計畫 | T1 | 存取未授權計畫的全部數據 | 待驗證 — user_protocols INSERT 權限 | **待查** | **高** |
| BIZ-2 | 審核流程跳關 | T1 | 繞過 IACUC 審查直接核准 | 待驗證 — 狀態機轉換邏輯 | **待查** | **高** |
| BIZ-3 | 電子簽章偽造 | T1 | 偽造他人簽名 | 待驗證 — 簽章 handler 密碼驗證 | **待查** | **高** |
| BIZ-4 | 審計日誌可刪除 | T2 | 銷毀證據鏈 | HMAC 鏈式完整性 + 無 DELETE 端點 | 需驗證無直接 DB access | **中** |

### 3.4 注入與基礎設施

| ID | 威脅 | 行為者 | 影響 | 現有防護 | 驗證方式 | 嚴重度 |
|----|------|--------|------|----------|----------|--------|
| INJ-1 | SQL Injection | T3,T1 | 資料庫完全控制 | SQLx 參數化查詢 | 靜態分析：全 codebase grep `format!`+SQL — 0 hit | **低** |
| INJ-2 | Path Traversal | T1 | 讀取伺服器任意檔案 | `canonicalize()` + `starts_with()` + 檔名過濾 | 靜態分析 + 程式碼 review | **低** |
| INJ-3 | SSRF | T1 | 從伺服器發起內網請求 | 外部 URL 皆 hardcoded 於 config | 靜態分析：所有 reqwest 呼叫 URL 來源追溯 | **低** |
| INJ-4 | Cookie Header Injection | T3 | 設定惡意 cookie | CRLF/分號過濾 + domain 字元白名單 | **已修復** | **已緩解** |

## 4. 架構層缺口

### 4.1 缺乏全域 Access Control Policy Layer

**問題**: 目前的 access check 是「逐端點人工加入」模式。`require_permission!` 巨集和 `access::require_animal_access()` 必須在每個 handler function body 中手動呼叫。這意味著：

- 新增 handler 時忘記加 = 漏洞
- 沒有編譯期保證
- Code review 是唯一防線

**建議方案**:

| 方案 | 說明 | 改動量 |
|------|------|--------|
| A. Route-level middleware | 在 `routes/*.rs` 層用 `.route_layer()` 統一加權限 | 中 |
| B. Macro attribute | `#[require_permission("xxx")]` 放在 handler 函數上 | 高 |
| C. Default-deny test | CI 中列舉所有 handler 函數，比對已知白名單 | 低 |

**最務實的短期方案是 C**：寫一個 CI 腳本，grep 所有 `pub async fn` 在 handlers/ 中，比對一份白名單。任何新 handler 沒在白名單中出現 = CI fail。

### 4.2 嚴重度分類的 Threat-Model 依據

此系統為 **單機構部署**（非多租戶 SaaS），因此：

- Admin 互相模擬 → **HIGH**（同一機構的 admin 互相可以溝通，風險低於多租戶場景）
- 若未來改為多租戶 → 升級為 **CRITICAL**
- GLP 合規相關（審計日誌、電子簽章）→ 法規層面的 **CRITICAL**，不分部署模式

## 5. 驗證方法論

| 方法 | 適用範圍 | 本次使用 | 局限 |
|------|----------|----------|------|
| **靜態分析（grep/AST）** | 注入類、missing check | ✅ 全 codebase | 無法驗證 runtime 行為 |
| **PoC Regression Test** | 已知漏洞 | ✅ `test_security_regression.py` | 僅涵蓋已知漏洞 |
| **Handler 覆蓋率矩陣** | IDOR、missing permission | ✅ 全 handler 窮舉 | 不含 service 層邏輯 |
| **業務邏輯 code review** | 流程繞過、race condition | ✅ 深度 review | 依賴 reviewer 領域知識 |
| **動態測試 / Penetration Test** | 所有類別 | ❌ 未執行 | 需要執行中的系統實例 |
| **Fuzzing** | 輸入驗證邊界 | ❌ 未執行 | 需要大量計算資源 |

**此次審計的邊界聲明**: 本次為靜態分析 + PoC regression test。未執行動態 penetration test（需要 running instance + test database）。靜態分析可確認「程式碼中存在防護邏輯」，但無法確認「防護邏輯在 runtime 正確運作」。PoC test 部分彌補了這個缺口，但僅限已知漏洞路徑。
