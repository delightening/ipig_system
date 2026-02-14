---
description: 
---

# Workflow: /stitch-loop

## Goal: 24小時無人值守開發循環

1. **PLAN**: 讀取專案需求與現有的 `.stitch` 檔案。
2. **EXECUTE**:
    - 開始編寫或修改代碼。
    - 使用 `browser` 工具打開預覽頁面。
3. **VERIFY (The Loop)**:
    - 執行 `Terminal: npm test` 或專案自訂測試指令。
    - 使用 `Browser: Take Screenshot` 比對設計稿。
    - **IF** 失敗:
        - 讀取錯誤日誌。
        - 回到步驟 2 重新修正代碼。
        - 重複此過程，直到驗證成功，最高重試 20 次。
4. **FINALIZE**:
    - 通過所有測試後，自動執行 `git commit -m "Auto-fix by Antigravity Stitch Loop"`.
    - 輸出 `<mission_status>COMPLETED</mission_status>`.
