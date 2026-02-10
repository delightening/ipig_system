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

---
*最後更新: 2026-02-10 23:48*
