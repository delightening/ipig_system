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

## 6. Architectural Recommendations（按風險降低 ROI 排序）

| 優先級 | 方案 | 風險降低 | 成本 |
|--------|------|----------|------|
| 1 | **CI `_current_user` 掃描** — 任何 handler 中出現 `_current_user` + `Path` 參數 = CI fail | 消除全部同類 IDOR | 極低（一個 grep 腳本） |
| 2 | **auth middleware 加 `is_active` 檢查** — 每次請求驗證帳號狀態 | 消除 BIZ-16 | 低（加一個 DB query，可快取） |
| 3 | **狀態機 enum 化** — Protocol/Amendment/Leave/Document 狀態轉換用 match arm 而非字串比較 | 消除 BIZ-14/17/18 全部狀態跳關 | 中（需重構 status 相關邏輯） |
| 4 | **self-approval guard** — 全域 utility `assert_not_self_approval(submitter_id, approver_id)` | 消除 BIZ-10/11 | 低 |

---

## 7. Remaining Uncertainty

以下項目**無法透過靜態分析確認**，需要 running instance + dynamic test：

1. **Timing oracle 實際可利用性** — 理論上 fetch-first-then-check 有 timing 差異，但在網路延遲下是否可靠區分需要實測
2. **Permission cache race condition** — v1 說 <1 秒 window，但併發壓力測試下可能更大
3. **HMAC chain 斷裂恢復** — 如果 HMAC 計算在 INSERT 和 UPDATE 之間系統崩潰，鏈是否可恢復
4. **Rate limiter 繞過** — 使用多 IP 是否可繞過 per-IP rate limit（依賴部署架構）
5. **Regression test 完整覆蓋** — 需要 running instance 才能驗證所有 PoC
