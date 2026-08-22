# GLP 注意事項 — 雙語表單

> 適用範圍：`services/print-pdf/` 所有雙語列印模板（目前 AUP Protocol；未來其他 GLP 表單沿用）。
> 對應實作：`templates/aup_protocol.html` 的 `L(zh, en)` macro + `lang: Literal["zh", "en"]` schema。

## 1. 核心原則：必須指定 master 語言

GLP（OECD GLP / FDA 21 CFR Part 58 / TFDA）本身不指定語言，但要求：

- **單一 authoritative version**：一個版本為「正本」（master），另一語言為「翻譯參考」（reference）。
- **簽字 / 核准頁僅出現在正本**。Study Director / QA / IACUC 不在 reference 版簽字。
- **翻譯一致性需 QA 驗證**並留紀錄（誰譯、誰 review、何時）。Label 意義偏差 = audit finding。
- **版本綁死**：master 改了，reference 必須同步改。SOP version divergence = critical finding。

## 2. 本系統的決策：中文為 master

**理由**：

- 場所在台灣，IACUC 用中文審核並簽署。
- 法律與監管文件主體在 TFDA / IACUC 體系下。
- 英文版主要供合作單位（國際 collaborators）、未來 pre-submission 參考。

**實作規範**：

| 項目 | zh（master） | en（reference） |
|---|---|---|
| Jinja 渲染 | `lang='zh'` | `lang='en'` |
| 頁眉 / 頁尾 | 一般標題 | 額外標註 `Translation of [doc-id] v[X.Y] (zh, authoritative)` |
| 簽字頁 | ✅ 含完整簽章 | ❌ 標註 `Refer to authoritative zh version` |
| Doc ID | 共用同一 protocol ID（例 `AUP-2026-001`） | 共用同一 ID + lang suffix |
| 版本號 | master 定版 | 同步 |

## 3. CI / 結構鎖定建議

避免 L() macro 漏譯導致兩版頁數 / 章節不對齊：

- **smoke test 比對結構**：渲完 zh / en 兩版 PDF，斷言頁數一致、章節 heading 數量一致。
- **L() macro coverage check**：任何中文字串若未包在 `L(zh, en)` 內，CI fail（防止「忘記翻譯」）。
- **PR 規範**：模板改動必同時更新 zh / en，禁止單語修改。

## 4. 未來 FDA IND / pivotal study 階段

**現在 jinja 自動產的 en 版不可直接當 FDA 提交文件**。屆時需另走：

1. **Certified translation**：第三方專業翻譯機構翻譯。
2. **QA 簽核流程**：QA reviewer 比對原中文與英譯，正式 sign-off。
3. **重新指定 master**：若 IND 對象為 FDA，英文版本身要成為 authoritative；中文版降為 reference（與目前順序相反）。
4. **完整 audit trail**：保留翻譯草稿、reviewer comment、簽核時間戳。

## 5. 風險登錄

| 風險 | 嚴重度 | 緩解 |
|---|---|---|
| 版本 drift（zh 改了 en 沒同步） | Critical | CI 結構檢查 + PR 規範 |
| Label 翻譯誤差（同一欄位意義不同） | High | 翻譯前查 GLP 慣用詞 + QA review |
| 兩版同時被視為正本 | High | 頁眉強制標 `authoritative` / `reference` |
| 直接用 jinja en 版送 FDA | Critical | 文件政策明列 jinja 產出僅供「參考用途」 |

## 6. 相關檔案

- `templates/aup_protocol.html` — 雙語 macro 實作
- `schemas/aup_protocol.py` — `lang` 欄位定義
- `samples.py` — zh / en sample payload
