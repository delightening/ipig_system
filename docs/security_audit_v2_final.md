# Security Audit v2 — Adversarial Re-Audit Report

> **日期**: 2026-04-14  
> **類型**: 對抗性複查（Adversarial Re-Audit）  
> **範圍**: 對 v1 審計報告的獨立驗證 + 遺漏發現

---

## 1. Executive Summary

v1 審計找到並修復了 12 個漏洞，其中 3 個 CRITICAL。但 v1 的核心方法論缺陷導致它**遺漏了至少 7 個同類 IDOR 漏洞**，並且在覆蓋率聲明上不準確（聲稱 576/576 = 0 FAIL，實際上至少有 9 個 handler 存在授權失敗）。

**v1 做對的事**：加密方式審計品質高、JWT/CSRF/Argon2 結論可信賴、PUT /me 自我提權和 BIZ-1/3/5 業務邏輯修復正確。

**v1 做錯的事**：IDOR 修復不完整（修了 6 個同類漏洞中的 6 個，卻遺漏了同目錄下至少 7 個相同模式的漏洞）、覆蓋率矩陣結論過度自信（0 FAIL 是錯的）、regression test 品質不足（多個 false negative 風險）。

---

## 2. Credibility Assessment

| 原始聲明 | 驗證方式 | 可信？ | 問題 |
|----------|----------|--------|------|
| ES256 簽章安全 | 靜態讀碼 | ✅ 可信 | Algorithm binding + aud/iss 驗證確實存在 |
| Argon2id + timing attack 防護 | 靜態讀碼 | ✅ 可信 | Dummy hash 邏輯確實正確 |
| CSRF HMAC-SHA256 constant-time | 靜態讀碼 | ✅ 可信 | 程式碼實作正確 |
| SQL Injection: 0 hit | grep `format!`+SQL | ⚠️ 部分可信 | grep 方法正確但 data_import.rs 有字串拼接 SQL（已由白名單保護） |
| SSRF: 外部 URL 皆 hardcoded | 靜態讀碼 | ⚠️ 部分可信 | 靜態分析只能確認「設計意圖」，無法確認 runtime 行為 |
| IDOR: 6 個 handler 已修復 | 靜態讀碼 | ❌ 不可信 | 遺漏至少 7 個同模式漏洞 |
| 576 handler, 0 FAIL | Agent 掃描 | ❌ 不可信 | 實際至少 9 個 FAIL，handler 數量也有偏差 |
| BIZ-3 簽章修復 | 靜態讀碼 | ⚠️ 部分可信 | 密碼驗證已加，但 signature_data 仍使用常數 "handwriting" |
| Permission cache 安全 | 靜態讀碼 | ✅ 可信 | 角色變更時 immediate invalidation |
| Cookie 安全 | 靜態讀碼 | ✅ 可信 | HttpOnly + SameSite + CRLF 過濾 |

---

## 3. New Findings（v2 獨立發現，v1 遺漏）

### 3.1 IDOR 遺漏（v2 新修 7 個）

| 檔案 | 函數 | 問題 | 嚴重度 |
|------|------|------|--------|
| `sacrifice_pathology.rs` | `get_animal_sacrifice` | `_current_user` 未使用，無 access check | CRITICAL |
| `sacrifice_pathology.rs` | `get_animal_pathology_report` | 有 `require_permission!` 但缺 `require_animal_access` | HIGH |
| `sudden_death.rs` | `get_animal_sudden_death` | `_current_user` 未使用，無 access check | CRITICAL |
| `observation.rs` | `get_observation_versions` | `_current_user` 未使用，版本歷程可跨計畫存取 | HIGH |
| `surgery.rs` | `get_surgery_versions` | `_current_user` 未使用，版本歷程可跨計畫存取 | HIGH |
| `care_record.rs` | `list_observation_care_records` | `_current_user` 未使用，照護紀錄可跨計畫存取 | HIGH |
| `care_record.rs` | `list_surgery_care_records` | `_current_user` 未使用，照護紀錄可跨計畫存取 | HIGH |
| `animal_core.rs` | `get_animal_events` | `_current_user` 未使用，審計事件可跨計畫存取 | HIGH |
| `dashboard.rs` | `get_vet_comments` | `_current_user` 未使用，全系統獸醫評論無過濾 | MEDIUM |

**根因分析**：v1 審計「修了已知的 6 個」但沒有系統性搜索 `_current_user`（底線=未使用）模式。v2 使用 `grep "_current_user"` 在 handlers/ 中搜索全部實例，才找出遺漏。

### 3.2 Timing Oracle（Phase 3 發現）

已修復的 IDOR 端點（如 `get_animal_blood_test`）採用 fetch-first-then-check 模式：
```rust
let test = Service::get_by_id(&state.db, id).await?;  // 先 fetch
access::require_animal_access(...).await?;              // 再 check
```

這產生 timing oracle：valid-but-forbidden ID 的回應時間比 invalid ID 長（多了一次 access check query）。攻擊者可藉此列舉有效 ID。

**嚴重度**: LOW（需要大量請求和精確計時，實際利用難度高）

### 3.3 Business Logic（Phase 4 發現，v1 完全未涵蓋）

| ID | 漏洞 | 嚴重度 | 說明 |
|----|------|--------|------|
| BIZ-10 | Leave 自我核准 | HIGH | 主管可以核准自己的請假，無 `submitter != approver` 檢查 |
| BIZ-11 | Overtime 自我核准 | HIGH | 同上，加班單 |
| BIZ-14 | Amendment 狀態跳關 | HIGH | `change_status()` 無 WHERE 驗證當前狀態，admin 可 DRAFT→APPROVED |
| BIZ-16 | 停用帳號仍可操作 | HIGH | auth middleware 不檢查 `is_active`/`expires_at`，JWT 有效期內仍可操作 |
| BIZ-17 | Leave 拒絕無狀態檢查 | MEDIUM | 已 APPROVED 的請假可以被再次 REJECTED |

### 3.4 21 CFR Part 11 缺口

| 條文 | 要求 | 現狀 | 缺口 |
|------|------|------|------|
| §11.10(b) | 審計日誌防竄改 | HMAC 鏈式完整性 | HMAC 是 INSERT 後 UPDATE 寫入（audit.rs:315），理論上可被中斷 |
| §11.10(c) | 記錄保護 | 無 UPDATE 限制 on approved protocols | approved 計畫的 `working_content` 可被修改 |
| §11.50 | 簽章含義/時間/身分 | `signature_data` 使用常數 `"handwriting"` | 手寫簽章的 signature_data 可被預測/重算 |
| §11.100 | 唯一識別 | UUID-based user ID | ✅ 合規 |

---

## 4. Severity Corrections

| 項目 | v1 嚴重度 | v2 嚴重度 | 理由 |
|------|-----------|-----------|------|
| BIZ-3 手寫簽章免密碼 | HIGH (GLP: CRITICAL) | **CRITICAL** | 在任何 GLP 部署中這就是 CRITICAL，不應迴避 |
| BIZ-7 過期計畫可寫入 | MEDIUM (待確認) | **HIGH** | GLP §3.4 要求到期計畫鎖定，不是「業務決策」 |
| BIZ-2 Co-editor 當 reviewer | MEDIUM (待確認) | **HIGH** | IACUC 獨立審查原則不是可選的 |

---

## 5. Regression Test 品質評估

| 測試 | 斷言方式 | False Negative 風險 | 問題 |
|------|----------|---------------------|------|
| VULN-004 role_ids | 檢查回應 body 角色 | **中** | 如果 API 回傳 200 但 role 未變，test pass — 但 role 可能在 DB 層已寫入只是回傳舊值 |
| VULN-001 報表 | status code 403 | **低** | 正確，但假設 PI 沒有 erp.report.view — 若 PI 被賦予此權限則 test pass 但漏洞仍在 |
| VULN-002 IDOR | 使用假 UUID | **高** | 假 UUID 會回傳 404（不存在）而非 403（無權限）。Test 接受 404 作為「安全」，但這不驗證 ownership check — 只驗證了「不存在的 ID 不可存取」 |
| VULN-005 Admin 互相模擬 | 硬編碼 True | **致命** | 沒有實際測試，直接 `t.record(..., True, "已在程式碼層級驗證")` |
| BIZ-1 assign_co_editor | 無測試 | — | 已修復但無 regression test |
| BIZ-3 簽章密碼 | 無測試 | — | 已修復但無 regression test |
| BIZ-5 Leave race | 無測試 | — | 已修復但無 regression test |

**VULN-002 的 IDOR test 是最嚴重的 false negative**：它用不存在的 UUID 測試，永遠會得到 404。正確的測試應該用 User A 建立的真實動物 ID，然後用 User B 嘗試存取。

---

## 6. 待修復項目（按風險排序，附 deadline 建議）

### Tier 1 — 下一個 sprint 必修（合規阻斷）

| ID | 問題 | 風險 | 具體修法 |
|----|------|------|---------|
| BIZ-14 | Amendment `change_status()` 無 WHERE 狀態驗證 | GLP 審核完整性 | 加 `WHERE status = $current_status` |
| BIZ-10/11 | Leave/Overtime 自我核准 | HR 合規 + 利益衝突 | 加 `if current.user_id == approver_id { return Err }` |
| BIZ-17 | Leave 拒絕無 WHERE status 條件 | 資料一致性 | 同 BIZ-5 修法 |

### Tier 2 — 本季度修復（安全強化）

**Timing Oracle**：目前的 IDOR 修復用 fetch-first-then-check，導致 invalid ID 回 404（快）而 valid-but-forbidden 回 403（慢）。

正確做法**不是**「把 ownership check 整合進 SQL WHERE」（SQLx 抽象讓這在多 JOIN 場景下不實際）。正確做法是**統一回傳 404**：

```rust
// 修改 services/access.rs 中的 require_animal_access 錯誤型別
// 改前：Err(AppError::Forbidden("..."))
// 改後：Err(AppError::NotFound("Animal not found"))
// 讓 forbidden 和 not-found 不可區分
```

只需改 `services/access.rs` 一處，所有 IDOR 修復自動受益。

### Tier 3 — 記錄並追蹤

| ID | 問題 | 為什麼不急 |
|----|------|-----------|
| BIZ-13 | Approved protocol content 可修改 | 需確認編輯 handler 是否已有狀態檢查 |
| BIZ-15 | Audit log HMAC 是 INSERT 後 UPDATE | 需 DB access 才能利用，風險低 |

---

## 7. 21 CFR Part 11 §11.50 — 簽章問題詳解

**現狀**（`services/signature/mod.rs:188-196`）：

```rust
let hash_input = password_hash.unwrap_or("handwriting");  // 手寫簽章 = 常數字串
let signature_input = format!("{}:{}:{}:{}",
    signer_id, content_hash, timestamp.timestamp(), hash_input);
let signature_data = Self::compute_hash(&signature_input);  // SHA-256
```

**問題**：手寫簽章的 `signature_data` 使用常數 `"handwriting"` 而非使用者密碼 hash。已知 `signer_id`（UUID，JWT 可見）+ `content_hash`（可計算）+ `timestamp`（response 可觀察）+ 常數 = 第三方可重算 `signature_data`。

**§11.50 要求**：電子簽章必須包含簽署者唯一識別且不可偽造。當 `signature_data` 可被第三方重算時不具不可否認性。

**已修復**：BIZ-3 fix 讓手寫簽章要求密碼。但 `signature_data` 計算邏輯仍用常數。

**建議**：`sign_with_handwriting()` 呼叫時，改為傳入 `password_hash`（密碼驗證步驟已取得）。

---

## 8. BIZ-16 殘留風險聲明

帳號停用後的存取撤銷有三層防護，但仍有一個**不可消除的架構限制**：

| 層 | 機制 | 生效延遲 | 覆蓋範圍 |
|----|------|----------|---------|
| 1 | `update_user` 停用時撤銷 refresh tokens + 終止 sessions | **即時** | 阻止 token refresh 和新 session |
| 2 | auth middleware 查 `is_active` / `expires_at` | **≤ 5 分鐘**（TTL 快取） | 攔截新的 API 請求 |
| 3 | JWT access token 自然過期 | **≤ jwt_expiration_seconds** | JWT 本身的 stateless 限制 |

**殘留風險**：JWT access token 是 stateless 的。伺服器端不知道使用者目前持有哪些 JWT（只有 logout 時才知道 jti）。因此：

- 如果 JWT expiration = 15 分鐘，停用帳號後最多 15 分鐘內舊 JWT 可通過 Layer 3
- Layer 2（TTL 查詢）提供了更強的防護——5 分鐘內 middleware 會查 DB 發現 `is_active = false` 並拒絕
- **最壞情況**：帳號停用後 5 分鐘內仍可存取（非 15 分鐘，因為 Layer 2 更快生效）

**GLP 影響**：在安全事件場景（帳號盜用），5 分鐘視窗是可接受的——因為實際的 data exfiltration 受限於 rate limiter（100 req/sec）。但此風險**必須在 GLP validation protocol 中記錄**為已知限制。

**若需要 zero-delay**：唯一做法是在 auth middleware 中**每次請求**都查 DB（移除 TTL 快取），或改用 opaque session token 取代 JWT。這會增加每請求 ~1-3ms DB latency。

---

## 9. Timing Oracle vs Response Leakage — 拆分為兩個問題

原始報告將這兩個問題混為一談。它們是不同的攻擊：

**問題 A：Response Leakage（錯誤碼不一致）**
- 攻擊者用 valid-but-forbidden ID 得到 403，用 invalid ID 得到 404
- 不需要計時就能區分
- **修法**：`services/access.rs` 中 `require_animal_access` 回傳 NotFound 取代 Forbidden

**問題 B：Timing Oracle（回應時間不一致）**
- 即使回傳碼統一為 404，valid-but-forbidden ID 仍比 invalid ID 多一次 DB query
- 需要高精度計時（通常 <10ms 差異），在有網路延遲的環境中難以利用
- **修法**：將 ownership check 整合進初始 fetch query（不是另開一個 query）

問題 A 嚴重、易修、應立即處理。問題 B 低嚴重度、難利用、可列為 Tier 3。原始報告把兩者混在一起用一個修法解決，但其實只解決了 A 沒解決 B。

---

## 10. 漏洞發現來源分布

| 發現方法 | 漏洞數量 | 佔比 | 代表漏洞 |
|----------|----------|------|---------|
| **CI `_current_user` grep** | 16 | **53%** | sacrifice, sudden_death, versions, care_records, events, delete |
| **業務邏輯 code review** | 8 | 27% | BIZ-10/11/14/16/17, self-approval, status skip |
| **Threat model targeted review** | 3 | 10% | VULN-004 self-escalation, BIZ-1, BIZ-3 |
| **權限字串比對** | 3 | 10% | VULN-001 report endpoints |

**重要限制**：53% 這個數字的**分母是「本次發現的漏洞」**，不是「所有可能存在的漏洞」。CI grep 無法捕獲的漏洞類別（狀態機跳關、自我核准、race condition、temporal attack）根本不在此統計的樣本中。因此：

- ✅ 可以說：「在 IDOR 類別中，CI grep 幾乎能覆蓋 100%」
- ❌ 不能說：「CI grep 能捕獲 53% 的未來漏洞」
- 業務邏輯漏洞（27%）需要**人工 code review**，無法自動化
- 腳本 `scripts/ci_handler_security_scan.sh` 的價值在於消除一個**特定漏洞類別**，不是通用防護

---

## 11. Remaining Uncertainty

以下項目**無法透過靜態分析確認**，需要 running instance + dynamic test：

1. **Timing oracle 實際可利用性** — 需要實測網路延遲下的區分度
2. **Permission cache race condition** — 需要併發壓力測試
3. **HMAC chain 斷裂恢復** — 需要模擬系統崩潰場景
4. **Regression test 完整覆蓋** — 需要 running instance
