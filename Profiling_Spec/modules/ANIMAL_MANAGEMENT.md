# 動物管理系統規格

> **模組**：實驗動物生命週期管理  
> **最後更新**：2026-02-03

---

## 1. 系統目的

管理豬隻從入場到實驗完畢的完整生命週期：

- 豬隻基本資料管理
- 實驗分配與狀態追蹤
- 觀察/手術/體重/疫苗紀錄
- 獸醫師建議與照護紀錄
- 病歷匯出

---

## 2. 使用者角色

| 角色 | 說明 |
|------|------|
| SYSTEM_ADMIN | 全權管理 |
| IACUC_STAFF | 管理所有計劃進度 |
| VET | 健康管理、提供建議 |
| EXPERIMENT_STAFF | 實驗紀錄操作 |
| PI | 檢視自己計劃的豬隻 |
| CLIENT | 檢視委託計劃（唯讀） |

---

## 3. 豬隻狀態流程

```
未分配 ──(分配至計劃)──→ 已分配 ──(進入實驗)──→ 實驗中 ──(完成實驗)──→ 實驗完畢
```

| 狀態 | 說明 |
|------|------|
| `unassigned` | 尚未指派至任何計劃 |
| `assigned` | 已指派至計劃，實驗未開始 |
| `in_experiment` | 實驗進行中 |
| `completed` | 實驗完畢 |

---

## 4. 核心資料模型

### 4.1 豬隻主檔 (pigs)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| pig_no | SERIAL | 系統號（顯示用） |
| ear_tag | VARCHAR(10) | 耳號（唯一） |
| status | ENUM | 狀態 |
| breed | ENUM | 品種 |
| gender | ENUM | 性別 |
| birth_date | DATE | 出生日期 |
| entry_date | DATE | 進場日期 |
| entry_weight | DECIMAL | 進場體重 |
| pen_location | VARCHAR(10) | 欄位編號 |
| iacuc_no | VARCHAR(20) | IACUC 計劃編號 |
| source_id | UUID | FK → pig_sources.id |

### 4.2 觀察試驗紀錄 (pig_observations)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| pig_id | UUID | FK → pigs.id |
| event_date | DATE | 事件日期 |
| record_type | ENUM | 異常/試驗/觀察 |
| content | TEXT | 內容 |
| treatments | JSONB | 治療方式（多筆） |
| vet_read | BOOLEAN | 獸醫師已讀 |

### 4.3 手術紀錄 (pig_surgeries)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | UUID | 主鍵 |
| pig_id | UUID | FK → pigs.id |
| surgery_date | DATE | 手術日期 |
| surgery_site | VARCHAR | 手術部位 |
| induction_anesthesia | JSONB | 誘導麻醉 |
| vital_signs | JSONB | 生理數值（多筆） |

---

## 5. 豬隻詳情頁面 (7 Tab)

| Tab | 資料類型 | 說明 |
|-----|----------|------|
| 觀察試驗紀錄 | 多筆 | 日常觀察、異常紀錄 |
| 手術紀錄 | 多筆 | 手術與麻醉紀錄 |
| 體重紀錄 | 多筆 | 體重測量歷程 |
| 疫苗/驅蟲紀錄 | 多筆 | 疫苗接種與驅蟲 |
| 犧牲/採樣紀錄 | 單筆 | 實驗結束處置 |
| 豬隻資料 | 單筆 | 基本資料 |
| 病理組織報告 | 單筆+附件 | 病理報告 |

---

## 6. API 端點

### 6.1 豬隻管理

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/pigs` | 豬隻列表 |
| POST | `/pigs` | 新增豬隻 |
| GET | `/pigs/{id}` | 豬隻詳情 |
| PATCH | `/pigs/{id}` | 更新資料 |
| GET | `/pigs/by-pen` | 依欄位分組 |

### 6.2 批次操作

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/pigs/import/basic` | 匯入基本資料 |
| POST | `/pigs/import/weight` | 匯入體重 |
| POST | `/pigs/batch/assign` | 批次分配 |
| POST | `/pigs/batch/start-experiment` | 批次進入實驗 |

### 6.3 紀錄管理

| 方法 | 端點 | 說明 |
|------|------|------|
| GET/POST | `/pigs/{id}/observations` | 觀察紀錄 |
| GET/POST | `/pigs/{id}/surgeries` | 手術紀錄 |
| GET/POST | `/pigs/{id}/weights` | 體重紀錄 |
| GET/POST | `/pigs/{id}/vaccinations` | 疫苗紀錄 |
| GET/POST | `/pigs/{id}/sacrifice` | 犧牲紀錄 |

### 6.4 獸醫師操作

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/pigs/{id}/vet-read` | 標記已讀 |
| POST | `/observations/{id}/recommendations` | 新增建議 |
| POST | `/surgeries/{id}/recommendations` | 新增建議 |

---

## 7. 品種與性別

**品種 (breed)**：
- `miniature` - 迷你豬
- `white` - 白豬
- `LYD` - LYD 豬
- `other` - 其他

**性別 (gender)**：
- `male` - 雄性
- `female` - 雌性

---

## 8. 前端路由

| 路由 | 頁面 |
|------|------|
| `/pigs` | 豬隻列表 |
| `/pigs/{id}` | 豬隻詳情（7 Tab） |
| `/pigs/{id}/edit` | 編輯豬隻 |
| `/pig-sources` | 豬隻來源管理 |

---

## 9. 我的計劃

PI/CLIENT 可透過「我的計劃」檢視自己的計劃：

| Tab | 內容 |
|-----|------|
| 申請表 | AUP 計畫書內容（唯讀） |
| 豬隻紀錄 | 該計劃下已分配豬隻清單 |

---

## 10. 相關文件

- [AUP 系統](./AUP_SYSTEM.md) - 計劃與豬隻的關聯
- [通知系統](./NOTIFICATION_SYSTEM.md) - 獸醫師建議通知
- [稽核日誌](../guides/AUDIT_LOGGING.md) - GLP 合規紀錄
