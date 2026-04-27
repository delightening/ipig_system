# UPSERT Pattern 安全掃描報告（R28-3）

> **背景**：R26-13（PR #197）發現 `storage_location.rs::create_inventory_item` 的
> `INSERT...ON CONFLICT DO UPDATE` upsert 樣式有兩個問題：(1) audit 缺 before
> snapshot；(2) 兩並發請求互相覆蓋。
>
> **R28-3 目的**：全 backend 掃描相同 pattern 並逐一驗證是否需修。
>
> **掃描日期**：2026-04-27 · **執行者**：Claude (R28-3 PR)

## 評估標準

每個 `INSERT...ON CONFLICT DO UPDATE` 上呼叫點，依以下三軸判斷：

1. **Race protection**：上游有 `SELECT ... FOR UPDATE` 鎖住相關 row 嗎？或業務上 race 不關鍵（最後寫者勝可接受）？
2. **Audit completeness**：upsert 前有讀 before snapshot 並寫 `log_activity_tx` 嗎？或是 idempotent 操作不需 audit？
3. **Data sensitivity**：寫入欄位是否屬於合規（GLP / §11 / 個資 / 安全設定）關鍵欄位？

## 掃描結果（17 處）

| # | 檔案 | 用途 | Race | Audit | 結論 |
|---|---|---|---|---|---|
| 1 | `bin/create_test_user.rs:38` | dev tool 建立測試帳號 | n/a | n/a | ✅ SKIP — 非 prod code |
| 2 | `handlers/data_export.rs:104` | docstring 文字「ON CONFLICT DO UPDATE」 | n/a | n/a | ✅ SKIP — 註解非實際 SQL |
| 3 | `repositories/glp_compliance.rs:239` | `document_acknowledgments` (user 已讀文件) | 無 | 無 | ✅ ACCEPTABLE — idempotent receipt，acknowledgment 事件本身就是 audit |
| 4 | `repositories/qa_plan.rs:528` | `qa_sop_acknowledgments` (user 已讀 SOP) | 無 | 無 | ✅ ACCEPTABLE — 同 #3 |
| 5 | `services/calendar.rs:128` | `google_calendar_config` 單列設定 | 單列 | 無 | 🟡 MINOR audit gap（admin-only，race 不關鍵）— 入 R28-10 後續處理 |
| 6 | `services/data_import.rs:863` | 通用 import 表 upsert | 無 | import session log（高層級） | ✅ ACCEPTABLE — admin 觸發，獨立 audit |
| 7 | `services/animal/observation.rs:420` | `observation_vet_reads` (vet 已讀觀察記錄) | 無 | 無 | ✅ ACCEPTABLE — idempotent receipt |
| 8 | `services/animal/medical.rs:336` | `animal_sacrifices` (GLP 關鍵) | ✅ `animal SELECT ... FOR UPDATE` (L301) | ✅ before/after diff + log_activity_tx | ✅ PROTECTED — animal row 鎖序列化 |
| 9 | `services/animal/medical.rs:986` | `animal_pathology_reports` 僅 touches `updated_at` | n/a | n/a | ✅ ACCEPTABLE — 僅標記時間，無實際資料變更 |
| 10 | `services/animal/surgery.rs:447` | `surgery_vet_reads` (vet 已讀手術) | 無 | 無 | ✅ ACCEPTABLE — 同 #7 |
| 11 | `services/hr/attendance.rs:242` | `attendance_records` 同日重複打卡 | 無 | 無 | ✅ ACCEPTABLE — 業務允許「修正打卡」最後寫者勝；audit 由打卡事件本身覆蓋 |
| 12 | `services/notification/crud.rs:209` | `notification_settings` 僅 touches `updated_at` | n/a | n/a | ✅ ACCEPTABLE — 同 #9 |
| 13 | `services/storage_location.rs:400` | `storage_location_inventory_items` | ✅ R26-13 已修為顯式 SELECT FOR UPDATE | ✅ | ✅ DONE — PR #197 |
| 14 | `services/system_settings.rs:62` | `system_settings` 設定值（含 SMTP / app config） | 無 | ❌ **無 audit** | 🔴 **NEEDS FIX** — 設定變更應審計（含 smtp_password 等敏感欄位 redact） |
| 15 | `services/stock/ledger.rs:220` | `stock_ledger` 庫存帳冊（running balance） | sub-select read | 部分 | 🟡 MINOR — 庫存異動由 stock movement event 上層 audit；upsert 本身只是更新統計 cache，acceptable |
| 16 | `services/protocol/my_protocols.rs:158` | `vet_review_assignments` 重派獸醫 | 無 | 無 | 🟡 MINOR audit gap — 重派應審計，入 R28-10 後續處理 |
| 17 | `services/protocol/review.rs:41,78,167` | `review_assignments` / `vet_review_assignments` 重派 reviewer | 無 | 無 | 🟡 MINOR audit gap — 同 #16 |

## 結論

**17 處 upsert 中**：
- ✅ 14 處 acceptable（idempotent receipt / 已有 FOR UPDATE 序列化保護 / 業務允許）
- 🟡 3 處 minor audit gap（calendar config / protocol reviewer 重派）— 入 **R28-10** 後續處理（合併一個 audit-coverage PR）
- 🔴 1 處 NEEDS FIX：`system_settings.rs::update_settings` — **本 PR 修補**

## R28-3 完成標準

- [x] 全 backend grep `ON CONFLICT.*DO UPDATE` scan 完
- [x] 每處依 Race / Audit / Sensitivity 三軸評估
- [x] 真實 audit gap (`system_settings`) 修補
- [x] minor gaps 入 R28-10 backlog 不遺失
- [x] 報告寫入 `docs/security/` 供未來新增 upsert pattern 時參考

## 維護建議

新增 `INSERT...ON CONFLICT DO UPDATE` 時必須：

1. **Race 評估**：兩個並發 upsert 同時送達會產生不一致狀態嗎？若會 → 改用 SELECT FOR UPDATE + 顯式 INSERT/UPDATE 分支（參考 `storage_location.rs::create_inventory_item`）。
2. **Audit 評估**：寫入欄位有業務語意（非純 timestamp）嗎？若有 → 上游必須 read before snapshot + `log_activity_tx`。
3. **本表更新**：新增上呼叫點時把該行加進本報告，標明評估結論。
