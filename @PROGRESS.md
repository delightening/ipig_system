# 進度紀錄 (@PROGRESS.md)

## AUP 流程自動化測試
- [x] 建立 14 步完整流程測試規範 (`aup_workflow_spec.md`)
- [x] 更新自動化測試腳本 (`aup_test_standalone.py`)
- [x] 自動化分配測試帳號角色權限 (`assign_missing_roles.py`)
- [x] 完成 14 步完整生命週期驗證 (End-to-End)
- [x] 產出 V2 驗證報告 (`walkthrough_AUP_v2.md`) 並同步至測試目錄

## UI/UX 修正
- [x] 修正計畫詳情頁面審查人員列表的翻譯顯示問題 (pendingComment)
- [x] 實作跨版本評論統計：解決改版後委員狀態重置為「待發表」的問題
- [x] 優化版本紀錄查看模式：將 JSON 顯示改為正式計畫書預覽格式
- [x] 修復後端編譯警告：處理 `src/services/protocol.rs` 中的 unused variable (`is_vet`, `user_has_role`)
- [x] 實作獸醫師審查表 (Vet Review Form)：整合線上 12 項查檢與 PDF 報表動態同步
- [x] 修復 `src/models/protocol.rs` 編譯錯誤：解決 `ProtocolResponse` 定義不完整導致的括號不匹配問題，並補足遺失的 `save_vet_review_form` 方法

## 系統安全與審計
- [x] 修復登入事件、活動紀錄、安全警報收錄問題 (2026-02-11)
    - [x] 修復登入成功紀錄失效問題 (SQL INET 類型轉換與異步處理優化)
    - [x] 補全 AUP 計劃書編輯、評論、指派與各階段審核活動紀錄 (同步至全域活動紀錄 `user_activity_logs`)
    - [x] 優化非上班時間登入警報判定 (調整為 18:00-08:00 並確保記錄完整)
- [x] 更新 `tests/.pytest_cache/README.md` 作為完整 14 步流程之導引說明文件 (2026-02-11)
- [x] 驗證安全審計功能 (2026-02-11)
    - [x] API 端點驗證 24/24 全部通過（Dashboard、Activities、Logins、Sessions、Alerts、Protocol Activities）
    - [x] Protocol Activities 完整性：38 筆活動、11 種活動類型（CREATED → APPROVED 全覆蓋）
    - ⚠️ 已知問題：`user_activity_logs` 為空 — `AuditService::log_activity` 被 `let _ =` 靜默吞掉錯誤
    - [x] 驗證腳本：`tests/audit_verify.py`
    - [x] 修復 `let _ = AuditService::log_activity(...)` 為 `if let Err(e)` + `tracing::error!`（3 處）

---
*最後更新: 2026-02-11 01:25*
