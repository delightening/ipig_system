# 資料庫輸出與歷史資料重新填寫設計

> **建立日期：** 2026-03-01  
> **目的：** 讓系統可輸出資料庫內容、閱讀過去資料，並根據歷史內容重新填寫（預填）表單或紀錄。

---

## 一、需求摘要

1. **資料庫輸出**：將指定模組/範圍的資料匯出為系統可讀格式
2. **閱讀過去資料**：系統能解析、顯示歷史匯出內容
3. **根據歷史重新填寫**：建立新紀錄時，可選擇「基於過去紀錄」預填欄位

---

## 二、現況整理

### 2.1 既有匯出功能

| 模組 | 格式 | 用途 |
|------|------|------|
| 動物醫療資料 | PDF / JSON | 單一動物或專案醫療匯出 |
| 個人資料（GDPR） | JSON | `GET /me/export` |
| 稽核日誌 | CSV / JSON | `GET /admin/audit-logs/export` |
| 活動紀錄 | CSV | `GET /admin/audit/activities/export` |

### 2.2 既有匯入功能

| 模組 | 格式 | 用途 |
|------|------|------|
| 動物基礎資料 | Excel / CSV | 批次匯入 |
| 動物體重 | Excel / CSV | 批次匯入 |
| 倉庫、產品、夥伴 | Excel | 各別匯入 + 範本下載 |

### 2.3 既有「複製／基於過去」模式

- **觀察紀錄**：`POST /animals/:id/observations/copy`，複製既有觀察紀錄建立新紀錄
- **產品**：`/products/new?copy=:id`，複製產品建立新產品
- **AUP 範本**：手術前後照護、設計區塊等有固定範本可選

### 2.4 DB 層備份

- `pg_dump` 腳本 + db-backup 容器，每天備份
- 用於災難復原，非應用層讀取

---

## 三、設計方案

### 3.1 方案 A：應用層全模組匯出（JSON）

**目標**：匯出指定模組的資料為結構化 JSON，供系統再次讀取與匯入。

**輸出格式**（範例）：

```json
{
  "format_version": "1.0",
  "exported_at": "2026-03-01T10:00:00Z",
  "modules": ["animals", "observations", "protocols"],
  "data": {
    "animals": [...],
    "animal_observations": [...],
    "protocols": [...]
  }
}
```

**建議 API**：

- `GET /api/v1/admin/data-export?modules=animals,observations&format=json`
- 權限：`admin.data.export`，僅管理員
- 支援分頁或 streams 以處理大資料量

**優點**：系統可完整讀取、驗證、再匯入  
**缺點**：需要設計模組對應與 schema 版本相容性

---

### 3.2 方案 B：依實體「基於過去紀錄」預填

**目標**：新增紀錄時，可選一筆歷史紀錄作為範本，自動預填欄位。

**適用實體**（依既有複製模式擴充）：

| 實體 | 現況 | 擴充 |
|------|------|------|
| 觀察紀錄 | ✅ 已有 copy | 可再加「選擇歷史範本」下拉 |
| 手術紀錄 | 無 | 新增 copy / 基於歷史 |
| 體重紀錄 | 無 | 基於同動物最近一筆預填欄位 |
| 產品 | ✅ 已有 copy | 維持現狀 |
| Protocol | 無 | 新增「複製既有計畫」 |
| 請假申請 | 無 | 基於上次申請預填 |

**建議 API 模式**：

- 沿用既有 `POST .../copy` 或
- 新增 `GET .../templates` 或 `GET .../recent` 供前端預填
- 各 handler 回傳「可作為範本」的欄位結構

**優點**：符合「根據過去內容重新填寫」的直接需求  
**缺點**：需逐一實作各實體

---

### 3.3 方案 C：混合（推薦）

1. **短期**：以方案 B 為主，擴充「基於歷史紀錄」到更多實體
2. **中期**：實作方案 A 的模組化 JSON 匯出，供備份與稽核
3. **長期**：設計 JSON 匯入流程，可從匯出檔還原或遷移資料

---

## 四、實作優先順序

| 階段 | 項目 | 說明 |
|------|------|------|
| Phase 1 | 手術紀錄複製 | 新增 `POST /animals/:id/surgeries/copy`，複製既有手術建立新紀錄 |
| Phase 1 | 請假申請預填 | 新增「基於上次申請」按鈕，預填假別、天數等 |
| Phase 2 | 模組化 JSON 匯出 API | `GET /admin/data-export?modules=...`，輸出 animals / observations / protocols 等 |
| Phase 2 | Protocol 複製 | 新增「複製既有計畫」建立新草稿 |
| Phase 3 | JSON 匯入 | 解析匯出檔，驗證後寫回 DB（含衝突處理） |

---

## 五、技術要點

### 5.1 權限與稽核

- 全模組匯出：`admin.data.export`，記錄至 `audit_logs`
- 各實體 copy：沿用既有 entity 權限（如 `animal.observation.create`）

### 5.2 資料保留與隱私

- 匯出內容可能含 PII，需符合 `docs/security-compliance/DATA_RETENTION_POLICY.md`
- 匯出檔應標註用途與保留期限

### 5.3 版本相容

- JSON 匯出含 `format_version`，匯入時檢查相容性
- 新欄位以 optional 處理，舊版匯出仍可匯入

---

## 六、待辦對應

本設計對應 TODO 項目：**R6-6 資料庫輸出與歷史重新填寫**。

**一鍵全庫輸出（通用格式）** 的完整規劃請參見：`docs/database/FULL_DB_EXPORT_PLAN.md`。
