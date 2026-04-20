# 電子簽章合規審查 — 21 CFR Part 11 / 等效法規

> **版本**：1.0  
> **最後更新**：2026-02-25  
> **對象**：品質／法規／稽核人員、系統管理員  
> **關聯**：GLP、AUP 審查、犧牲／觀察／安樂死／轉讓／計畫書簽核

---

## 1. 目的與範圍

本文件針對 iPig 系統之**電子簽章（Electronic Signatures）**功能，依 **21 CFR Part 11**（美國 FDA 電子紀錄與電子簽章法規）及等效 GLP／實驗動物管理實務進行合規審查，作為上線前合規佐證與後續稽核參考。

**適用範圍**：

- 犧牲紀錄（Sacrifice）簽章
- 觀察紀錄（Observation）簽章
- 安樂死單據（Euthanasia Order）簽章
- 轉讓紀錄（Transfer）簽章
- 計畫書審查（Protocol / AUP）簽章
- 紀錄標註（Annotation）與變更原因

---

## 2. 法規對照摘要

### 2.1 21 CFR Part 11 子章 B — 電子紀錄

| 條款 | 要求 | iPig 對應實作 | 狀態 |
|------|------|----------------|------|
| 11.10(a) | 系統驗證：確保準確、可靠、可辨識無效或竄改紀錄 | 系統經測試與部署驗證；簽章與紀錄內容雜湊綁定 | ✓ 已實作 |
| 11.10(b) | 僅授權人員可存取 | 依 RBAC 權限（如 `animal.record.sacrifice`）控制簽章端點 | ✓ 已實作 |
| 11.10(c) | 操作與事件之適當順序 | 簽章前須有有效紀錄；簽章後可鎖定紀錄防篡改 | ✓ 已實作 |
| 11.10(e) | 可辨識之操作者、時間戳 | 簽章寫入 `signer_id`、`signed_at`；活動紀錄於 `user_activity_logs` | ✓ 已實作 |
| 11.10(k) | 稽核追蹤：安全、電腦產生、時間戳、不可刪改 | `user_activity_logs`、`audit_logs`；簽章與紀錄內容雜湊 | ✓ 已實作 |

### 2.2 21 CFR Part 11 子章 C — 電子簽章

| 條款 | 要求 | iPig 對應實作 | 狀態 |
|------|------|----------------|------|
| 11.100(a) | 簽章具唯一性、不重複使用或再指派 | 每筆簽章獨立 UUID；簽章與 `signer_id` 綁定 | ✓ 已實作 |
| 11.100(b) | 簽章前驗證身分 | 密碼驗證（`AuthService::verify_password_by_id`）或手寫簽名即時綁定登入者 | ✓ 已實作 |
| 11.100(c) | 簽章至少兩項組成（如 ID + 密碼） | 登入身分（JWT/session）+ 密碼或手寫簽名 | ✓ 已實作 |
| 11.50(a) | 簽章與紀錄連結，防止複製／移轉造假 | 簽章含 `record_type`、`record_id` 及紀錄內容雜湊 | ✓ 已實作 |
| 11.50(b) | 顯示簽署者姓名、簽署日期時間、簽章意義 | 簽章狀態 API 回傳 `signer_name`、`signed_at`、`signature_type` | ✓ 已實作 |

### 2.3 紀錄完整性與稽核

| 項目 | 要求 | iPig 對應 | 狀態 |
|------|------|-----------|------|
| 簽章後鎖定 | 特定紀錄類型簽章後不可再編輯 | Sacrifice / Observation 等 `lock_record` 機制 | ✓ 已實作 |
| 簽章方式區分 | 密碼 vs 手寫可辨識 | `signature_method`：`password` / `handwriting` | ✓ 已實作 |
| 手寫資料保留 | 手寫簽名可重現審查 | `handwriting_svg`、可選 `stroke_data` 儲存 | ✓ 已實作 |
| 稽核日誌 | 操作可追溯 | `user_activity_logs`、`audit_logs` | ✓ 已實作 |

---

## 3. 系統實作對照

### 3.1 簽章流程（共通）

1. **身分**：已透過 JWT 登入之使用者（CurrentUser）。
2. **驗證**：密碼驗證（`password`）或手寫簽名（`handwriting_svg`）二擇一，不可兩者皆空。
3. **綁定**：簽章與當時紀錄內容（如 `sacrifice_id` + 關鍵欄位）產生關聯／雜湊，防止事後篡改。
4. **寫入**：寫入 `signatures` 表（`record_type`、`record_id`、`signer_id`、`signed_at`、`signature_type`、`signature_method`、`handwriting_svg` 等）。
5. **鎖定**（依紀錄類型）：如犧牲、觀察等於簽章後呼叫 `SignatureService::lock_record`，禁止再編輯。

### 3.2 簽章類型與權限

| 紀錄類型 | 簽章類型 (signature_type) | 權限／角色 | API 路徑 |
|----------|----------------------------|------------|----------|
| 犧牲 | WITNESS / APPROVE / Confirm | animal.record.sacrifice | POST/GET `/signatures/sacrifice/:id` |
| 觀察 | Confirm | animal.record.view | POST `/signatures/observation/:id` |
| 安樂死 | APPROVE / WITNESS / Confirm | 依流程 | POST/GET `/signatures/euthanasia/:id` |
| 轉讓 | APPROVE / WITNESS / Confirm | 依流程 | POST/GET `/signatures/transfer/:id` |
| 計畫書審查 | 審查核准 | 計畫書審查權限 | POST/GET `/signatures/protocol/:id` |

### 3.3 標註（Annotation）與變更原因

- 標註 API：`GET/POST /annotations/:record_type/:record_id`。
- 類型含 CORRECTION 等；需密碼時由 `CreateAnnotationRequest.password` 驗證。
- 與稽核及紀錄歷程整合，符合「變更可追溯」之精神。

---

## 4. 合規差距與建議

### 4.1 已符合項目（摘要）

- 電子簽章與紀錄連結、簽署者與時間戳記錄完整。
- 密碼與手寫雙模式、簽章後鎖定、稽核日誌與活動紀錄已建置。
- 授權僅限具權限之使用者，符合 21 CFR Part 11 子章 B/C 核心要求。

### 4.2 建議強化（非阻擋上線）

| 項目 | 說明 | 建議 |
|------|------|------|
| 書面政策 | 21 CFR 11.10(k)、11.100 要求書面政策與個人責任 | 建議訂定〈電子簽章與電子紀錄管理規範〉，明訂簽章責任與稽核留存期限 |
| 訓練紀錄 | 使用電子簽章之人員須經訓練 | 建議將「電子簽章與 GLP 紀錄」納入必讀／必訓並留存紀錄 |
| 顯示／列印 | 簽署者姓名、日期時間、簽章意義於顯示與列印一致 | 前端報表／PDF 匯出時確保已帶入簽章狀態 API 之 `signer_name`、`signed_at`、`signature_type` |
| 定期審查 | 簽章與稽核日誌之定期審查 | 建議納入品質／稽核週期（如每季抽核） |

### 4.3 不適用或排除

- **密碼歷史／密碼過期**：依專案待辦禁止事項，不實作密碼歷史與密碼過期策略；不影響「簽章當下以密碼驗證身分」之合規性。
- **生物辨識**：目前未使用生物辨識簽章；若未來導入，須另行評估 Part 11 對生物辨識之要求。

---

## 5. 結論

iPig 電子簽章設計與實作在**技術面**已對齊 21 CFR Part 11 及等效 GLP 對電子紀錄與電子簽章之核心要求（唯一性、身分驗證、與紀錄連結、時間戳、稽核追蹤、授權存取與鎖定）。  
建議於正式上線前完成**書面政策與訓練紀錄**之補強，並在報表／匯出中確保簽章資訊完整顯示與列印，以利稽查時佐證。

---

## 6. 參考

- FDA 21 CFR Part 11 — Electronic Records; Electronic Signatures（含 Subpart B、C）
- 專案規格：`docs/Profiling_Spec/02_CORE_DOMAIN_MODEL.md`（§3.7 電子簽章與標註）、`07_SECURITY_AUDIT.md`
- 後端簽章服務：`backend/src/services/signature.rs`、`backend/src/handlers/signature.rs`
