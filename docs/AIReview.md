# AI 預審計畫

> 目標：利用 AI 在計劃書正式進入人工審查前，自動檢查格式完整性與法規符合性，減少「退回修改」的來回次數，加速 IACUC 審查流程。

---

## 一、問題背景

### 目前的痛點

```
客戶提交計劃書 → 執行秘書 Pre-Review → 發現格式/內容缺漏
→ 退回修改 → 客戶修改 → 重新提交 → 再次 Pre-Review
→ 可能再次退回...（平均 2-3 次來回）
```

**每次來回耗時**：客戶修改 3-7 天 + 秘書審查 1-2 天 = 4-9 天
**AI 預審可節省**：在提交前攔截 70-80% 的格式/完整性問題

### AI 預審的定位

```
客戶填寫計劃書 → [AI 即時預審] → 標記問題 → 客戶當場修正
→ 提交時已通過基本檢查 → 執行秘書專注實質內容審查
```

**AI 不取代人工審查**，而是作為「智慧校對助手」。

---

## 二、功能分層

### Level 1：規則式檢查（不需要 LLM）

利用程式邏輯檢查格式與完整性，零成本、即時回饋：

| 檢查項目 | 說明 |
|----------|------|
| 必填欄位完整性 | 所有 required 欄位是否已填寫 |
| 字數門檻 | 研究目的 ≥ 100 字、替代方案說明 ≥ 50 字等 |
| 日期邏輯 | 結束日 > 開始日、計畫期限 ≤ 3 年 |
| 動物數量合理性 | 數量 > 0、總數與分組數一致 |
| 3Rs 原則完整性 | Replacement / Reduction / Refinement 三項均已填寫 |
| 人員資格 | PI 是否有填寫經歷年數、訓練證照 |
| 疼痛分類一致性 | 疼痛類別與麻醉/止痛方案是否對應 |
| 附件檢查 | 必要附件是否已上傳 |
| 參考文獻 | 替代方案資料庫搜尋是否已填寫 ≥ 2 個平台 |

**實作方式**：擴展現有 `validation.ts`（前端）+ `services/protocol/` 新增 validation service（後端）

### Level 2：AI 內容審查（需要 LLM）

利用 Claude API 檢查內容品質與法規符合性：

| 檢查項目 | AI Prompt 方向 |
|----------|---------------|
| 3Rs 論述品質 | Replacement：是否充分說明為何必須使用動物？Reduction：統計方法是否合理支持動物數量？Refinement：痛苦最小化措施是否具體？ |
| 實驗設計合理性 | 對照組設計是否適當？樣本數是否有統計依據？ |
| 人道終點明確性 | 終止條件是否具體可執行？觀察頻率是否足夠？ |
| 麻醉/止痛方案 | 藥物選擇是否適合該物種？劑量是否在合理範圍？ |
| 術後照護 | 恢復期監測是否足夠？併發症處理方案是否完整？ |
| 安樂死方法 | 方法是否符合 AVMA 指南？是否為該物種推薦方法？ |
| 前後邏輯一致性 | 「研究目的」與「實驗設計」是否呼應？動物數量與實驗設計是否一致？ |

### Level 3：知識庫比對（進階，Phase 2+）

| 功能 | 說明 |
|------|------|
| 歷史計劃書比對 | 類似實驗的過往計劃書如何寫的 |
| 常見退回原因 | 根據歷史審查意見，預測可能被退回的理由 |
| 法規更新提醒 | 新法規是否影響此計劃書 |

---

## 三、使用者體驗設計

### 3.1 觸發時機

```
A. 即時檢查（Level 1）
   - 使用者填寫/修改欄位時即時驗證
   - 紅色錯誤提示（必須修正）+ 黃色警告（建議改善）

B. 提交前預審（Level 1 + Level 2）
   - 使用者點擊「提交」按鈕時觸發
   - 顯示 AI 預審報告，分為「必須修正」與「建議改善」
   - 使用者可選擇：修正後再提交 / 忽略建議直接提交

C. 手動觸發（Level 2）
   - 編輯頁面提供「AI 預審」按鈕
   - 使用者可在撰寫過程中隨時請求 AI 檢查
```

### 3.2 UI 呈現

```
┌──────────────────────────────────────────────────────┐
│  🔍 AI 預審報告                                      │
├──────────────────────────────────────────────────────┤
│                                                      │
│  ❌ 必須修正（3 項）                                  │
│  ├─ [3Rs-Reduction] 動物數量缺乏統計依據              │
│  │   「建議補充樣本數計算方法（如 power analysis）」     │
│  ├─ [術後照護] 未說明術後觀察頻率                      │
│  │   「建議明確術後 24/48/72 小時觀察時間點」           │
│  └─ [人道終點] 終止條件過於模糊                        │
│      「『動物狀況惡化時終止』→ 應列出具體指標」          │
│                                                      │
│  ⚠️ 建議改善（2 項）                                  │
│  ├─ [替代方案] 建議增加搜尋的資料庫平台                │
│  └─ [麻醉方案] Isoflurane 誘導濃度建議調整為 3-5%     │
│                                                      │
│  ✅ 通過檢查（15 項）                                 │
│     研究目的、實驗設計、人員資格...                     │
│                                                      │
│  [修正後重新檢查]  [忽略建議，直接提交]                │
└──────────────────────────────────────────────────────┘
```

---

## 四、技術架構

### 4.1 系統架構

```
Frontend (React)
    │
    ▼
[提交前觸發 AI 預審]
    │
    ▼
Backend API: POST /api/protocols/{id}/ai-review
    │
    ├─ Level 1: 規則引擎（Rust 內建）
    │   └─ 即時回傳結果
    │
    └─ Level 2: Claude API 呼叫
        │
        ├─ System Prompt（IACUC 審查專家角色 + 法規知識）
        ├─ 計劃書內容（JSON → 結構化文本）
        └─ 輸出格式（結構化 JSON，含問題分類與建議）
        │
        ▼
    [組合結果] → Response → Frontend 顯示報告
```

### 4.2 Claude API 整合

#### System Prompt 設計

```
你是一位資深的 IACUC（動物實驗倫理委員會）審查委員，
擁有實驗動物科學與獸醫學背景。

你的任務是預審動物實驗計劃書（AUP），檢查以下面向：
1. 3Rs 原則（替代、減量、精緻化）的論述是否充分
2. 實驗設計是否合理
3. 麻醉/止痛/術後照護方案是否適當
4. 人道終點是否明確可執行
5. 安樂死方法是否符合 AVMA 指南
6. 各段落之間的邏輯一致性

注意：
- 你是「預審助手」，不是最終決策者
- 區分「必須修正」（明顯缺漏/錯誤）和「建議改善」（可更好但非必要）
- 回覆使用繁體中文
- 以 JSON 格式回傳結果
```

#### 輸出格式

```json
{
    "summary": "此計劃書主要架構完整，但 3Rs-Reduction 和術後照護部分需要補強",
    "score": 72,
    "issues": [
        {
            "severity": "error",
            "category": "3Rs-Reduction",
            "section": "purpose",
            "message": "動物數量（N=30）缺乏統計依據",
            "suggestion": "建議補充 power analysis 或引用文獻支持樣本數合理性",
            "reference": "IACUC 審查指引 §4.2"
        },
        {
            "severity": "warning",
            "category": "anesthesia",
            "section": "design",
            "message": "Isoflurane 誘導濃度 2% 偏低",
            "suggestion": "豬隻建議誘導濃度 3-5%，維持 1.5-2.5%",
            "reference": "Flecknell, Laboratory Animal Anaesthesia, 4th ed."
        }
    ],
    "passed": [
        "research_purpose",
        "replacement_justification",
        "personnel_qualifications",
        "euthanasia_method"
    ]
}
```

### 4.3 成本控制

| 措施 | 說明 |
|------|------|
| Level 1 先行 | 規則檢查不通過就不呼叫 AI，節省 API 費用 |
| 快取機制 | 同一版本計劃書 24 小時內只呼叫一次 AI |
| 模型選擇 | 預審用 Claude Haiku（快速、低成本），深度審查用 Sonnet |
| Token 限制 | 限制單次輸入 ≤ 8K tokens，輸出 ≤ 2K tokens |
| 每日額度 | 每位使用者每日最多 10 次 AI 預審 |

**預估成本**：
- Haiku：~$0.01/次 × 10 次/天 × 20 天/月 = ~$2/月
- Sonnet（深度）：~$0.05/次 × 偶爾使用 = ~$5/月
- **總計：< $10/月**

---

## 五、實作階段

### Phase 1：規則式檢查（預估 3-4 天）

```
□ Backend: services/protocol/validation.rs — 擴展驗證規則
    - 字數門檻檢查
    - 日期/數量邏輯檢查
    - 3Rs 完整性
    - 疼痛分類 vs 麻醉方案一致性
□ Backend: handlers/protocol/ — 新增 POST /api/protocols/{id}/validate
□ Frontend: 提交前觸發驗證 + 報告 UI 元件
□ 測試：各規則的 unit test
```

### Phase 2：Claude API 整合（預估 4-5 天）

```
□ Backend: services/ai/ — 擴展現有 AI service
    - System prompt 設計與調校
    - 計劃書內容序列化為 AI 輸入格式
    - AI 回應解析與結構化
    - 快取機制（Redis 或 DB）
    - Rate limiting 與成本控制
□ Backend: handlers/ — 新增 POST /api/protocols/{id}/ai-review
□ Frontend: AI 預審按鈕 + 結果呈現 UI
□ Frontend: 提交流程整合（提交前自動觸發）
□ 測試：mock AI response 的整合測試
```

### Phase 3：調校與優化（持續）

```
□ 收集真實審查意見，對比 AI 預審結果
□ 調整 system prompt 提高準確率
□ 建立「常見退回原因」資料庫
□ 追蹤指標：退回次數是否減少
```

---

## 六、與現有系統整合

### 利用既有元件

| 需求 | 既有元件 | 說明 |
|------|---------|------|
| AI 呼叫 | `services/ai.rs` + `handlers/ai.rs` | 已有 AI service 基礎架構 |
| 計劃書資料 | `models/protocol.rs` | `working_content` JSON 可直接送入 AI |
| 前端表單 | `ProtocolEditPage` 10 段表單 | 在現有表單旁顯示 AI 建議 |
| 驗證框架 | `validation.ts` (前端) | 擴展既有驗證邏輯 |
| 通知 | `services/notification/` | AI 預審完成通知 |
| 審計 | `services/audit.rs` | 記錄 AI 預審觸發與結果 |

### 新增/修改檔案

```
Backend:
  src/services/protocol/ai_review.rs    # AI 預審邏輯
  src/services/protocol/validation.rs   # 擴展規則式檢查（或整合現有）
  src/handlers/protocol/ai_review.rs    # AI 預審 endpoint
  src/models/ai_review.rs              # 預審結果 model

Frontend:
  src/components/protocol/AIReviewPanel.tsx    # AI 預審報告面板
  src/components/protocol/AIReviewButton.tsx   # 觸發按鈕
  src/lib/api/aiReview.ts                      # API 函式
  src/types/aiReview.ts                        # 型別定義
```

---

## 七、雙角色 AI 審查架構

AI 審查分為兩個角色，共用底層架構但服務不同對象：

```
                    ┌─────────────────────────────────────┐
                    │     protocol_ai_reviews 表           │
                    │     + AI Service 基礎架構            │
                    │     + 規則引擎 + Claude API          │
                    └───────────┬─────────────┬───────────┘
                                │             │
              ┌─────────────────┘             └─────────────────┐
              ▼                                                 ▼
    ┌──────────────────┐                            ┌──────────────────┐
    │ 客戶端 AI 預審    │                            │ 執行秘書 AI 標註  │
    │ (本文件)          │                            │ (clientsAccess.md)│
    ├──────────────────┤                            ├──────────────────┤
    │ review_type:      │                            │ review_type:      │
    │ client_pre_submit │                            │ staff_pre_review  │
    │                   │                            │                   │
    │ 對象：PI（客戶）  │                            │ 對象：IACUC_STAFF │
    │ 時機：提交前      │                            │ 時機：Pre-Review  │
    │ 語氣：指導性      │                            │ 語氣：輔助性      │
    │ 「你需要修正...」 │                            │ 「請注意此處...」 │
    │                   │                            │                   │
    │ Endpoint:         │                            │ Endpoint:         │
    │ POST /protocols/  │                            │ POST /protocols/  │
    │   {id}/ai-review  │                            │ {id}/staff-review │
    │                   │                            │   -assist         │
    └──────────────────┘                            └──────────────────┘
```

**共用元件**：`protocol_ai_reviews` 表、`services/protocol/ai_review.rs`、規則引擎、Claude API 呼叫邏輯
**差異**：System prompt 不同、輸出格式不同、觸發時機不同、權限不同

---

## 八、成功指標

| 指標 | 目標 | 衡量方式 |
|------|------|----------|
| Pre-Review 退回率 | 降低 50% | 退回次數 / 提交次數 |
| 平均審查週期 | 縮短 30% | 提交到核准的天數 |
| AI 預審準確率 | ≥ 80% | AI 標記問題 vs 人工審查標記問題的重疊率 |
| 客戶滿意度 | 正面回饋 | 客戶訪談 |
| API 成本 | < $10/月 | Claude API 帳單 |

---

## 八、風險與緩解

| 風險 | 緩解措施 |
|------|---------|
| AI 誤判（False Positive） | 區分「必須修正」vs「建議改善」，客戶可忽略建議 |
| AI 漏判（False Negative） | AI 預審不取代人工審查，僅為輔助 |
| API 服務中斷 | Level 1 規則檢查不依賴外部 API，可獨立運作 |
| 成本失控 | Rate limiting + 快取 + 每日額度限制 |
| 隱私疑慮 | 計劃書內容傳送至 Claude API — 需確認客戶同意 |
