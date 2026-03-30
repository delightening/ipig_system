# Heartbeat Code Review 計畫

> 目標：透過定期自動化任務，持續維護 ipigsystem 的程式碼品質、安全性與功能完整性。

---

## 一、每日分段 Code Review（10 個工作天一輪迴）

將整個 codebase 依模組拆成 10 段，每天 review 一段，兩週完成一個完整輪迴。

### 每日排程表

| 天 | 範圍 | 涵蓋內容 | 預估檔案數 |
|----|------|----------|-----------|
| D1 | Backend: `handlers/auth/` + `middleware/` + `services/auth/` | 登入、JWT、CSRF、Rate Limiter、2FA、Session | ~21 檔 |
| D2 | Backend: `handlers/protocol/` + `services/protocol/` | AUP CRUD、Review、Comment、Status 狀態機 | ~13 檔 |
| D3 | Backend: `handlers/animal/` + `services/animal/` | 動物管理、手術、觀察、血檢、安樂死 | ~32 檔 |
| D4 | Backend: `handlers/hr/` + `services/hr/` + `services/calendar/` | 出勤、請假、加班、Google Calendar 同步 | ~12 檔 |
| D5 | Backend: ERP 相關 (`handlers/` + `services/` 中的 document, product, stock, warehouse, sku, partner, accounting) | 庫存、採購、銷售、倉儲、SKU | ~25 檔 |
| D6 | Backend: `services/notification/` + `services/email/` + `services/pdf/` + `repositories/` | 通知路由、Email 模板、PDF 生成、Repository 層 | ~25 檔 |
| D7 | Backend: 剩餘 services（audit, equipment, facility, report, signature, amendment, ai, scheduler 等）+ `models/` | 審計、設備、報表、簽章、修正案 | ~45 檔 |
| D8 | Frontend: `pages/protocols/` + `pages/animals/` + `pages/amendments/` | 計劃書表單、動物管理頁面 | ~80 檔 |
| D9 | Frontend: `pages/admin/` + `pages/hr/` + `pages/dashboard/` + `pages/auth/` | 管理後台、HR、儀表板、認證頁面 | ~85 檔 |
| D10 | Frontend: `pages/erp/` + `pages/inventory/` + `pages/master/` + `pages/documents/` + `pages/reports/` + 共用 `components/` + `lib/` + `hooks/` + `stores/` + `types/` | ERP 頁面、報表、共用元件與工具 | ~80 檔 |

### 每日 Review 檢查項目

```
□ 程式碼品質
  - 函數長度 ≤ 50 行、圈複雜度 ≤ 10、巢狀 ≤ 3 層
  - DRY 原則：重複邏輯 ≥ 2 處是否已抽出
  - 命名一致性（Rust snake_case / TS camelCase / React PascalCase）
  - 未使用的 import、變數、函式

□ 安全漏洞
  - SQL injection（是否有字串拼接 SQL）
  - 權限檢查是否完整（handler 是否有對應 permission guard）
  - 敏感資料是否外洩（password、token 是否出現在 response/log）
  - unwrap() 是否出現在非測試碼

□ 未處理的 TODO / FIXME / HACK
  - 列出所有 TODO/FIXME 並評估優先級
  - 記錄新發現的技術債

□ CLAUDE.md 規範符合度
  - 架構分層是否正確（Handler → Service → Repository → Model）
  - 錯誤處理是否使用 AppError
  - Frontend API 是否統一使用 TanStack Query
```

### 產出格式

每日 review 結果寫入 `docs/heartbeat/YYYY-MM-DD.md`：

```markdown
# Heartbeat Review - YYYY-MM-DD

## 範圍：D{N} - {模組名稱}

### 發現問題
| # | 嚴重度 | 檔案 | 行號 | 說明 | 建議修復 |
|---|--------|------|------|------|----------|

### TODO/FIXME 清單
| 檔案 | 內容 | 建議優先級 |

### 品質指標
- 檢查檔案數：
- 發現問題數：
- Critical：/ High：/ Medium：/ Low：

### 本日結論
{一句話總結}
```

---

## 二、每日健康檢查

每天早上自動執行，與分段 review 獨立：

### 檢查項目

```
1. 測試執行
   - cargo test（Backend 142+ 單元測試）
   - cargo clippy -- -D warnings（Lint 零警告）
   - npx playwright test（E2E 34 測試）
   - 記錄失敗的測試與錯誤訊息

2. 依賴安全掃描
   - cargo deny check advisories（Rust CVE 掃描）
   - cargo deny check licenses（授權合規）
   - npm audit（Frontend CVE 掃描）
   - 列出所有 HIGH/CRITICAL 漏洞

3. 編譯檢查
   - cargo build --release（確認 release 編譯無誤）
   - npm run build（Frontend 編譯無誤）
   - 記錄 warning 數量趨勢

4. 資料庫 Migration 一致性
   - 檢查 pending migrations
   - 驗證 migration 檔案命名順序正確
```

### 產出格式

寫入 `docs/heartbeat/health-YYYY-MM-DD.md`：

```markdown
# Health Check - YYYY-MM-DD

| 項目 | 狀態 | 備註 |
|------|------|------|
| cargo test | ✅/❌ | {失敗數}/{總數} |
| cargo clippy | ✅/❌ | {warning 數} |
| E2E tests | ✅/❌ | {失敗數}/{總數} |
| cargo deny | ✅/❌ | {CVE 數} |
| npm audit | ✅/❌ | {漏洞數} |
| cargo build | ✅/❌ | |
| npm run build | ✅/❌ | |
| migrations | ✅/❌ | |

## 需要關注
{列出需要處理的問題}
```

---

## 三、月度架構審查（每月 1 日）

深度檢查整體架構是否偏離規範：

### 檢查項目

```
1. CLAUDE.md 規範合規
   - 架構分層依賴方向是否正確
   - Handler 是否包含業務邏輯（應在 Service）
   - Service 是否直接寫 SQL（應在 Repository）
   - Utils 是否依賴 AppState
   - Models 是否依賴其他層

2. 量化指標掃描
   - 超過 50 行的函數列表
   - 圈複雜度 > 10 的函數
   - 超過 300 行的 React 元件
   - 超過 6 個 Props 的元件
   - 超過 5 個參數的函數

3. 重複程式碼偵測
   - 相同 SQL 出現 ≥ 2 處
   - 相同驗證邏輯 ≥ 2 處
   - 相同 UI pattern ≥ 2 處

4. 依賴健康度
   - 未使用的 npm/cargo 依賴
   - 過時的依賴版本（major version behind）
   - 授權風險（GPL 依賴混入）

5. 測試覆蓋率
   - 新增的 handler 是否有對應測試
   - 關鍵業務邏輯（status 狀態機、權限檢查）測試覆蓋
   - E2E 測試是否覆蓋主要用戶流程
```

### 產出格式

寫入 `docs/heartbeat/architecture-YYYY-MM.md`：

```markdown
# 月度架構審查 - YYYY-MM

## 規範合規摘要
| 規範 | 合規率 | 違規數 | 趨勢 |
|------|--------|--------|------|

## 量化指標
| 指標 | 數量 | 對比上月 |
|------|------|----------|

## 重複程式碼
{列表}

## 依賴健康度
{掃描結果}

## 建議改進項目（寫入 TODO.md）
| # | 優先級 | 說明 |
```

---

## 四、實作方式

使用 Claude Code `/schedule` 建立三個排程任務：

### 排程 1：每日分段 Code Review
- **頻率**：週一至週五，每日一次
- **輪迴週期**：10 個工作天（約兩週）
- **觸發方式**：Claude Code remote trigger (cron schedule)
- **自動判斷**：根據當前日期計算是 D1-D10 的哪一天

### 排程 2：每日健康檢查
- **頻率**：每日一次（含週末）
- **前置條件**：Docker 環境可用（PostgreSQL for tests）
- **觸發方式**：Claude Code remote trigger (cron schedule)

### 排程 3：月度架構審查
- **頻率**：每月 1 日
- **觸發方式**：Claude Code remote trigger (cron schedule)

---

## 五、目錄結構

```
docs/
  heartbeat/
    README.md                      # Heartbeat 系統說明
    YYYY-MM-DD.md                  # 每日分段 review
    health-YYYY-MM-DD.md           # 每日健康檢查
    architecture-YYYY-MM.md        # 月度架構審查
```

---

## 六、成功指標

| 指標 | 目標 |
|------|------|
| 測試通過率 | 100%（每日） |
| Clippy 警告數 | 0（每日） |
| CVE 漏洞數 | 0 Critical/High |
| 超長函數數 | 月度遞減 |
| 重複程式碼 | 月度遞減 |
| review 覆蓋率 | 每兩週 100% codebase |
