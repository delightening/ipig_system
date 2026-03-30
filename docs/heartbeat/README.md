# Heartbeat Daily Code Review

每個工作天自動執行一輪代碼審查，以 2026-01-01 為基準日計算工作天數（排除週末），用 `(工作天數 % 10)` 決定當日模組（0→D10, 1→D1, ..., 9→D9）。

## 排程表

| 代號 | 模組範圍 |
|------|---------|
| D1 | backend/src/handlers/auth/ + backend/src/middleware/ + backend/src/services/auth/ |
| D2 | backend/src/handlers/protocol/ + backend/src/services/protocol/ |
| D3 | backend/src/handlers/animal/ + backend/src/services/animal/ |
| D4 | backend/src/handlers/hr/ + backend/src/services/hr/ + backend/src/services/calendar/ |
| D5 | ERP 相關 handlers + services（document, product, stock, warehouse, sku, partner, accounting） |
| D6 | backend/src/services/notification/ + email/ + pdf/ + repositories/ |
| D7 | 剩餘 backend/src/services/ + backend/src/models/ |
| D8 | frontend/src/pages/protocols/ + animals/ + amendments/ |
| D9 | frontend/src/pages/admin/ + hr/ + dashboard/ + auth/ |
| D10 | frontend/src/pages/erp/ + inventory/ + master/ + documents/ + reports/ + 共用 components/ + lib/ + hooks/ + stores/ + types/ |

## 報告格式

報告存放於本目錄，檔名格式：`YYYY-MM-DD.md`

## 審查項目

1. 函數長度 > 50 行（Rust）或元件 > 300 行（React）
2. 巢狀深度 > 3 層
3. SQL injection 風險（字串拼接 SQL、format!() 動態 SQL）
4. 權限檢查完整性（handler 是否有對應 permission guard）
5. unwrap() 殘留（非測試碼）
6. TODO/FIXME/HACK 註解
7. CLAUDE.md 架構分層合規
