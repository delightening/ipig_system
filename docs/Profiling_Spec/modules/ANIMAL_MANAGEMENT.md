# 動物管理系統規格

> **模組**：實驗動物生命週期管理  
> **版本**：7.0  
> **最後更新**：2026-03-02

---

## 1. 系統目的

管理動物從入場到實驗完畢（或轉讓/猝死）的完整生命週期：

- 動物基本資料管理（CRUD、匯入匯出、**不可變欄位修正申請**）
- 實驗分配與狀態追蹤（6 種狀態 + 狀態轉換驗證）
- 觀察/手術/體重/疫苗/犧牲/病理紀錄
- 血液檢查子系統（模板、組合、費用追蹤、結果分析）
- 安樂死申請與核准流程
- **動物轉讓流程**（6 步驟、資料隔離）
- **猝死登記**
- 獸醫師建議與照護紀錄
- 病歷匯出（PDF/Excel）
- **手寫電子簽章**（犧牲紀錄整合）
- GLP 合規（電子簽章、記錄鎖定、版本控制、附註）

---

## 2. 使用者角色

| 角色 | 說明 |
|------|------|
| SYSTEM_ADMIN | 全權管理 |
| IACUC_STAFF | 管理所有計劃進度 |
| IACUC_CHAIR | 審查決策、資料隔離特權 |
| VET | 健康管理、提供建議、資料隔離特權 |
| EXPERIMENT_STAFF | 實驗紀錄操作 |
| PI | 檢視自己計劃的動物、轉讓簽名 |
| CLIENT | 檢視委託計劃（唯讀） |

---

## 3. 動物狀態流程

### 3.1 狀態定義

| 狀態 | 說明 | 終端狀態 |
|------|------|:--------:|
| `unassigned` | 尚未指派至任何計劃 | — |
| `in_experiment` | 實驗進行中 | — |
| `completed` | 實驗完畢 | — |
| `euthanized` | 已安樂死 | ✅ |
| `sudden_death` | 猝死 | ✅ |
| `transferred` | 已轉讓至其他計劃 | ✅ |

### 3.2 狀態轉換規則

```
                                              ┌──→ euthanized (安樂死)
                                              │
unassigned ──→ in_experiment ──→ completed ───┤──→ sudden_death (猝死)
                    │                         │
                    │                         └──→ transferred (轉讓)
                    │
                    ├──→ euthanized (安樂死)
                    └──→ sudden_death (猝死)
```

**狀態轉換驗證** (`can_transition_to()`):
- `unassigned` → `in_experiment`
- `in_experiment` → `completed`, `euthanized`, `sudden_death`
- `completed` → `euthanized`, `sudden_death`, `transferred`
- 終端狀態（`euthanized`, `sudden_death`, `transferred`）不可再轉換

**自動行為**：
- 安樂死核准執行 → 自動設為 `euthanized` + 建立空犧牲紀錄
- 猝死登記 → 自動設為 `sudden_death`
- 轉讓完成 → 自動設為 `transferred`

---

## 4. 核心資料模型

### 4.1 動物主檔 (animals)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | SERIAL | 主鍵 |
| ear_tag | VARCHAR(10) | 耳號（唯一） |
| status | animal_status | 狀態（6 種） |
| breed | animal_breed | 品種 |
| gender | animal_gender | 性別 |
| birth_date | DATE | 出生日期 |
| entry_date | DATE | 進場日期 |
| entry_weight | DECIMAL | 進場體重 |
| pen_location | VARCHAR(10) | 欄位編號 |
| iacuc_no | VARCHAR(20) | IACUC 計劃編號 |
| source_id | UUID | FK → animal_sources.id |
| experiment_date | DATE | 實驗開始日 |
| pre_experiment_code | VARCHAR(20) | 術前代碼 |
| is_deleted | BOOLEAN | 軟刪除標記 |

### 4.2 觀察試驗紀錄 (animal_observations)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | SERIAL | 主鍵 |
| animal_id | INTEGER | FK → animals.id |
| event_date | DATE | 事件日期 |
| record_type | record_type | 異常/試驗/觀察 |
| content | TEXT | 內容 |
| treatments | JSONB | 治療方式（多筆） |
| vet_read | BOOLEAN | 獸醫師已讀 |
| is_deleted | BOOLEAN | 軟刪除（含 delete_reason） |

### 4.3 手術紀錄 (animal_surgeries)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | SERIAL | 主鍵 |
| animal_id | INTEGER | FK → animals.id |
| surgery_date | DATE | 手術日期 |
| surgery_site | VARCHAR | 手術部位 |
| induction_anesthesia | JSONB | 誘導麻醉 |
| anesthesia_maintenance | JSONB | 維持麻醉 |
| vital_signs | JSONB | 生理數值（多筆時序資料） |
| vet_read | BOOLEAN | 獸醫已讀 |
| is_deleted | BOOLEAN | 軟刪除 |

### 4.4 猝死紀錄 (animal_sudden_deaths)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | SERIAL | 主鍵 |
| animal_id | INTEGER | FK → animals.id（UNIQUE） |
| death_date | DATE | 死亡日期 |
| description | TEXT | 死亡情境描述 |
| discovered_by | UUID | 發現者 |
| reported_by | UUID | 登記者 |
| created_at | TIMESTAMPTZ | 建立時間 |

### 4.5 動物轉讓 (animal_transfers)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | SERIAL | 主鍵 |
| animal_id | INTEGER | FK → animals.id |
| status | animal_transfer_status | 轉讓狀態（6 步） |
| from_protocol_id | UUID | 來源計劃 |
| to_protocol_id | UUID | 目標計劃 |
| reason | TEXT | 轉讓原因 |
| initiated_by | UUID | 發起者 |
| transfer_date | TIMESTAMPTZ | 轉讓完成日期 |

**轉讓狀態列舉 (animal_transfer_status)**：

| 狀態 | 說明 | 操作者 |
|------|------|--------|
| `pending_source_pi` | 等待來源 PI 確認 | 發起者 |
| `pending_vet_evaluation` | 等待獸醫評估 | VET |
| `pending_target_pi` | 等待目標 PI 確認 | 目標 PI |
| `pending_iacuc_approval` | 等待 IACUC 核准 | IACUC_STAFF/CHAIR |
| `approved` | 已核准（待執行） | — |
| `completed` | 已完成 | EXPERIMENT_STAFF |

### 4.6 轉讓獸醫評估 (transfer_vet_evaluations)

| 欄位 | 類型 | 說明 |
|------|------|------|
| id | SERIAL | 主鍵 |
| transfer_id | INTEGER | FK → animal_transfers.id |
| vet_id | UUID | 獸醫 |
| health_status | TEXT | 健康狀態 |
| is_fit_for_transfer | BOOLEAN | 是否適合轉讓 |
| notes | TEXT | 備註 |

---

## 5. 轉讓流程（6 步驟）

```
發起轉讓 → 來源 PI 確認 → 獸醫評估 → 目標 PI 確認 → IACUC 核准 → 執行完成
```

### 5.1 資料隔離機制

轉讓完成後，新計劃的使用者**預設只能看到轉讓時間之後的資料**：

- 觀察紀錄、手術紀錄、體重紀錄、疫苗紀錄、血液檢查均會加上 `?after=` 時間過濾
- 透過 `GET /animals/:id/data-boundary` API 取得隔離時間點

**特權角色可繞過隔離**：ADMIN、VET、IACUC_STAFF、IACUC_CHAIR

---

## 6. 動物詳情頁面 (8 Tab)

| Tab | 資料類型 | 說明 |
|-----|----------|------|
| 觀察試驗紀錄 | 多筆 | 日常觀察、異常紀錄 |
| 手術紀錄 | 多筆 | 手術與麻醉紀錄 |
| 體重紀錄 | 多筆 | 體重測量歷程 |
| 疫苗/驅蟲紀錄 | 多筆 | 疫苗接種與驅蟲 |
| 犧牲/採樣紀錄 | 單筆 | 實驗結束處置（含手寫簽章） |
| 動物資料 | 單筆 | 基本資料 |
| 病理組織報告 | 單筆+附件 | 病理報告 |
| **轉讓紀錄** | 多筆 | Stepper 進度條 + 6 步角色表單 + 歷史紀錄 |

> 轉讓 Tab 僅在動物狀態為 `completed` 或 `transferred` 時顯示。

---

## 7. 血液檢查子系統

### 7.1 功能

| 功能 | 說明 |
|------|------|
| 血液檢查 CRUD | `BloodTestTab` 完整表單 |
| 檢驗模板管理 | 64 個預設模板，`BloodTestTemplatesPage` |
| 檢驗組合管理 (Panel) | 14 組預設組合 + CRUD，`BloodTestPanelsPage` |
| 組合快速勾選 | Toggle 按鈕列 |
| 組合停用/恢復 | 軟刪除 |

### 7.2 資料分析

| 功能 | 說明 |
|------|------|
| 統計分析 | `BloodTestAnalysisPage`，依動物/專案/日期區間 |
| 異常值標記 | 醒目標記 + 警示區塊 |
| 趨勢圖表 | Recharts 折線圖 + 自訂盒鬚圖 |
| 匯出 | CSV + Excel (xlsx) |

---

## 8. 安樂死流程

```
申請人建立 → PI 初審 → 獸醫評估 → IACUC 審查 → 委員裁決 → 執行
```

支援申訴機制（`appeal` + `decide`）。

---

## 9. 手寫電子簽章整合

犧牲紀錄場景整合手寫簽名（`HandwrittenSignaturePad.tsx`）：

| 欄位 | 說明 |
|------|------|
| `handwriting_svg` | 手寫簽名 SVG 圖片 |
| `stroke_data` | 原始筆跡座標數據 |
| `signature_method` | 簽章方式（`password` / `handwriting`） |

---

## 10. API 端點

### 10.1 動物管理

| 方法 | 端點 | 說明 |
|------|------|------|
| GET | `/animals` | 動物列表 |
| POST | `/animals` | 新增動物 |
| GET | `/animals/:id` | 動物詳情 |
| PUT | `/animals/:id` | 更新資料 |
| GET | `/animals/by-pen` | 依欄位分組 |
| GET | `/animals/vet-comments` | 獸醫待閱 |
| GET | `/animals/:id/data-boundary` | 資料隔離邊界 |

### 10.2 批次操作

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/animals/import/basic` | 匯入基本資料 |
| POST | `/animals/import/weights` | 匯入體重 |
| GET | `/animals/import/template/basic` | 下載基本資料模板 |
| GET | `/animals/import/template/weight` | 下載體重模板 |
| POST | `/animals/batch/assign` | 批次分配 |
| POST | `/animals/batch/start-experiment` | 批次進入實驗 |

### 10.3 紀錄管理

| 方法 | 端點 | 說明 |
|------|------|------|
| GET/POST | `/animals/:id/observations` | 觀察紀錄 |
| POST | `/animals/:id/observations/copy` | 複製觀察紀錄 |
| GET/POST | `/animals/:id/surgeries` | 手術紀錄 |
| POST | `/animals/:id/surgeries/copy` | 複製手術紀錄 |
| GET/POST | `/animals/:id/weights` | 體重紀錄 |
| GET/POST | `/animals/:id/vaccinations` | 疫苗紀錄 |
| GET/POST | `/animals/:id/sacrifice` | 犧牲紀錄 |
| GET/POST | `/animals/:id/pathology` | 病理報告 |

### 10.4 獸醫師操作

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/animals/:id/vet-read` | 標記已讀 |
| POST | `/observations/:id/recommendations` | 觀察建議 |
| POST | `/surgeries/:id/recommendations` | 手術建議 |

### 10.5 猝死登記

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/animals/:id/sudden-death` | 登記猝死 |
| GET | `/animals/:id/sudden-death` | 查詢猝死紀錄 |

### 10.6 動物欄位修正申請

耳號、出生日期、性別、品種等不可變欄位，若 staff 輸入錯誤，可提交修正申請，經 admin 批准後套用。

| 方法 | 端點 | 說明 | 權限 |
|------|------|------|------|
| POST | `/animals/:id/field-corrections` | 建立修正申請 | animal.animal.edit |
| GET | `/animals/:id/field-corrections` | 查詢該動物申請列表 | animal.animal.edit |
| GET | `/animals/animal-field-corrections/pending` | 列出待審申請 | admin |
| POST | `/animals/animal-field-corrections/:id/review` | 批准/拒絕申請 | admin |

### 10.7 動物轉讓

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/animals/:id/transfers` | 發起轉讓 |
| GET | `/animals/:id/transfers` | 轉讓紀錄列表 |
| GET | `/transfers/:id` | 轉讓詳情 |
| POST | `/transfers/:id/source-pi-confirm` | 來源 PI 確認 |
| POST | `/transfers/:id/vet-evaluate` | 獸醫評估 |
| POST | `/transfers/:id/target-pi-confirm` | 目標 PI 確認 |
| POST | `/transfers/:id/iacuc-approve` | IACUC 核准 |
| POST | `/transfers/:id/complete` | 執行完成 |

### 10.8 安樂死

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/euthanasia/orders` | 建立安樂死申請 |
| GET | `/euthanasia/orders` | 申請列表 |
| POST | `/euthanasia/orders/:id/approve` | 核准 |
| POST | `/euthanasia/orders/:id/appeal` | 申訴 |
| POST | `/euthanasia/orders/:id/execute` | 執行 |
| POST | `/euthanasia/appeals/:id/decide` | 申訴裁決 |

### 10.9 匯出

| 方法 | 端點 | 說明 |
|------|------|------|
| POST | `/animals/:id/export` | 單一動物病歷匯出 |
| POST | `/animals/export/project` | 計劃批次匯出 |

---

## 11. 品種與性別

**品種 (animal_breed)**：
- `miniature` - 迷你豬
- `white` - 白豬
- `LYD` - LYD 豬
- `other` - 其他

**性別 (animal_gender)**：
- `male` - 雄性
- `female` - 雌性

---

## 12. 前端路由

| 路由 | 頁面 |
|------|------|
| `/animals` | 動物列表（含 Tab 篩選、分組視圖、排序） |
| `/animals/:id` | 動物詳情（8 Tab） |
| `/animals/:id/edit` | 編輯動物（含「申請修正」按鈕） |
| `/animals/animal-field-corrections` | 修正審核（實驗動物管理） |
| `/animal-sources` | 動物來源管理 |
| `/blood-test-templates` | 血液檢查模板管理 |
| `/blood-test-panels` | 血液檢查組合管理 |
| `/reports/blood-test-analysis` | 血液檢查結果分析 |

---

## 13. GLP 合規功能

| 功能 | 實作方式 |
|------|----------|
| 軟刪除 + 刪除原因 | `is_deleted` + `delete_reason` |
| 變更原因記錄 | `change_reasons` 表 |
| 審計日誌增強 | `old_value`, `new_value` |
| 電子簽章 | `SignatureService`（密碼驗證 + 手寫簽名） |
| 記錄鎖定 | 簽章後自動鎖定 |
| 附註/更正 | `AnnotationService` |
| 版本控制 | `record_versions` 表歷史快照 |

---

## 14. 我的計劃

PI/CLIENT 可透過「我的計劃」檢視自己的計劃：

| Tab | 內容 |
|-----|------|
| 申請表 | AUP 計畫書內容（唯讀） |
| 動物紀錄 | 該計劃下已分配動物清單 |

---

## 15. 相關文件

- [AUP 系統](./AUP_SYSTEM.md) - 計劃與動物的關聯
- [通知系統](./NOTIFICATION_SYSTEM.md) - 獸醫師建議通知
- [稽核日誌](../guides/AUDIT_LOGGING.md) - GLP 合規紀錄
- [安全與稽核](../07_SECURITY_AUDIT.md) - 安全中間件
