# Heartbeat 自動化維護系統

> 透過 Claude Code 定期排程，持續維護 ipigsystem 的程式碼品質、安全性與功能完整性。

## 報告類型

| 類型 | 頻率 | 檔名格式 | 說明 |
|------|------|----------|------|
| 每日分段 Code Review | 週一至週五 08:00 | `YYYY-MM-DD.md` | 10 天一輪迴，每天 review 一個模組區塊 |
| 每日健康檢查 | 每日 07:00 | `health-YYYY-MM-DD.md` | 編譯、測試、lint、CVE 掃描 |
| 月度架構審查 | 每月 1 日 09:00 | `architecture-YYYY-MM.md` | 量化指標、重複程式碼、依賴健康度 |

## Code Review 輪迴表（10 天）

| 天 | 範圍 |
|----|------|
| D1 | Backend: handlers/auth/ + middleware/ + services/auth/ |
| D2 | Backend: handlers/protocol/ + services/protocol/ |
| D3 | Backend: handlers/animal/ + services/animal/ |
| D4 | Backend: handlers/hr/ + services/hr/ + services/calendar/ |
| D5 | Backend: ERP 相關（document, product, stock, warehouse, sku, partner, accounting） |
| D6 | Backend: services/notification/ + email/ + pdf/ + repositories/ |
| D7 | Backend: 剩餘 services + models/ |
| D8 | Frontend: pages/protocols/ + animals/ + amendments/ |
| D9 | Frontend: pages/admin/ + hr/ + dashboard/ + auth/ |
| D10 | Frontend: pages/erp/ + inventory/ + master/ + documents/ + reports/ + 共用 components/ |

## 審查項目

1. 函數長度 > 50 行（Rust）或元件 > 300 行（React）
2. 巢狀深度 > 3 層
3. SQL injection 風險（字串拼接 SQL、format!() 動態 SQL）
4. 權限檢查完整性（handler 是否有對應 permission guard）
5. unwrap() 殘留（非測試碼）
6. TODO/FIXME/HACK 註解
7. CLAUDE.md 架構分層合規

## 嚴重度定義

| 等級 | 說明 |
|------|------|
| Critical | 安全漏洞、資料遺失風險、production 阻斷 |
| High | 功能 bug、架構違規、效能瓶頸 |
| Medium | 程式碼品質、規範偏離、可維護性 |
| Low | 命名不一致、格式問題、建議改善 |
