# P2-R4-13：Animal 元件群 `any` 型別消除 — 執行計畫

> **範圍：** 約 41 處 `any`，分布在 18 個檔案  
> **預估工時：** 2.5–3 小時  
> **建議策略：** 分階段、低風險優先

---

## 一、現況盤點

### 1.1 依類型分類

| 類型 | 數量 | 檔案數 | 難度 | 說明 |
|------|------|--------|------|------|
| **A. onError (error: any)** | 25 | 15 | 低 | 統一改為 `unknown` + `getApiErrorMessage()` |
| **B. handleChange (value: any)** | 2 | 2 | 低 | `UpdateAnimalRequest[key]` 的 value 型別 |
| **C. 排序比較 (aVal/bVal: any)** | 2 | 1 | 低 | `AnimalListTable` 排序欄位值 |
| **D. payload / form (payload: any)** | 4 | 2 | 中 | `CreateAnimalRequest` / `NewAnimalForm` |
| **E. Timeline record/raw (record: any, raw?: any)** | 2 | 1 | 中 | `AnimalObservation \| AnimalSurgery` |
| **F. vital_signs map (vs: any)** | 1 | 1 | 低 | `AnimalSurgery['vital_signs'][0]` |
| **G. onSuccess (data: any)** | 1 | 1 | 低 | quickMove 回傳型別 |

### 1.2 依檔案分布

```
frontend/src/
├── pages/animals/
│   ├── AnimalsPage.tsx              4 處 (payload, onSuccess, mutationFn)
│   ├── AnimalEditPage.tsx           2 處 (onError, handleChange)
│   ├── AnimalSourcesPage.tsx        3 處 (onError)
│   └── components/
│       ├── AnimalAddDialog.tsx      2 處 (pendingPayload, onConfirm)
│       └── AnimalListTable.tsx      2 處 (aVal, bVal)
├── components/animal/
│   ├── TransferTab.tsx              6 處 (onError)
│   ├── AnimalTimelineView.tsx       2 處 (onEdit record, raw)
│   ├── SurgeryFormDialog.tsx        2 處 (vital_signs map, onError)
│   ├── QuickEditAnimalDialog.tsx    2 處 (onError, handleChange)
│   ├── ImportDialog.tsx             2 處 (onError, catch)
│   ├── ExportDialog.tsx             1 處 (onError)
│   ├── ObservationFormDialog.tsx   1 處 (onError)
│   ├── ObservationsTab.tsx          2 處 (onError)
│   ├── SurgeriesTab.tsx             2 處 (onError)
│   ├── WeightsTab.tsx               2 處 (onError)
│   ├── VaccinationsTab.tsx          2 處 (onError)
│   ├── PathologyTab.tsx             1 處 (onError)
│   ├── PainAssessmentTab.tsx        3 處 (onError)
│   ├── SacrificeFormDialog.tsx      2 處 (onError)
│   ├── VetRecommendationDialog.tsx  1 處 (onError)
│   ├── EmergencyMedicationDialog.tsx 1 處 (onError)
│   ├── EuthanasiaOrderDialog.tsx    1 處 (onError)
│   ├── EuthanasiaPendingPanel.tsx   2 處 (onError)
│   └── EuthanasiaChairArbitrationPanel.tsx 1 處 (onError)
```

---

## 二、執行順序與步驟

### 階段 0：前置準備（約 10 分鐘）

1. **確認既有型別**
   - `frontend/src/types/animal.ts` 已有：`CreateAnimalRequest`、`UpdateAnimalRequest`、`AnimalSurgery`、`AnimalObservation` 等
   - `AnimalSurgery.vital_signs` 已定義為 `{ time, heart_rate, respiration_rate, temperature, spo2 }[]`

2. **新增型別（若需）**
   - `frontend/src/types/animal.ts` 新增：
     ```ts
     /** Timeline 可編輯的紀錄類型 */
     export type AnimalTimelineRecord = AnimalObservation | AnimalSurgery

     /** UpdateAnimalRequest 欄位值（依 key 而異） */
     export type UpdateAnimalRequestValue = UpdateAnimalRequest[keyof UpdateAnimalRequest]
     ```

---

### 階段 1：低風險 — onError 統一（約 45 分鐘）

**目標：** 25 處 `onError: (error: any)` → `onError: (error: unknown)`

**作法：** 使用 `getApiErrorMessage(error)` 取代直接存取 `error?.response?.data`。

**執行順序（建議一次改 2–3 個檔案，每批後跑 `npm run build`）：**

| 批次 | 檔案 | 處數 |
|------|------|------|
| 1 | TransferTab.tsx | 6 |
| 2 | ObservationsTab, SurgeriesTab, WeightsTab, VaccinationsTab | 8 |
| 3 | PathologyTab, PainAssessmentTab, SacrificeFormDialog | 5 |
| 4 | ObservationFormDialog, VetRecommendationDialog, EmergencyMedicationDialog | 3 |
| 5 | EuthanasiaOrderDialog, EuthanasiaPendingPanel, EuthanasiaChairArbitrationPanel | 4 |
| 6 | AnimalsPage, AnimalEditPage, AnimalSourcesPage, QuickEditAnimalDialog, ImportDialog, ExportDialog | 9 |

**替換範例：**
```ts
// Before
onError: (error: any) => {
  toast({ title: '錯誤', description: error?.response?.data?.error?.message || '操作失敗', variant: 'destructive' })
}

// After
onError: (error: unknown) => {
  toast({ title: '錯誤', description: getApiErrorMessage(error), variant: 'destructive' })
}
```

**ImportDialog catch：**
```ts
// Before
} catch (error: any) { ... }

// After
} catch (error: unknown) {
  toast({ ..., description: getApiErrorMessage(error) })
}
```

---

### 階段 2：低風險 — 排序、handleChange、vital_signs（約 20 分鐘）

| 檔案 | 修改內容 |
|------|----------|
| **AnimalListTable.tsx** | `let aVal: any` → `let aVal: string \| number`，`bVal` 同理 |
| **QuickEditAnimalDialog.tsx** | `handleChange(field, value: any)` → `value: UpdateAnimalRequestValue` |
| **AnimalEditPage.tsx** | 同上 |
| **SurgeryFormDialog.tsx** | `(vs: any)` → `(vs: NonNullable<AnimalSurgery['vital_signs']>[number])` |

---

### 階段 3：中風險 — payload / form / Timeline（約 40 分鐘）

| 檔案 | 修改內容 |
|------|----------|
| **AnimalsPage.tsx** | `const payload: any` → `const payload: CreateAnimalRequest`（或擴展型別含 `pen_location` 等） |
| **AnimalsPage.tsx** | `onSuccess: (data: any, variables)` → `data: { notFound?: boolean; ... } \| AxiosResponse<Animal>`，依 quickMove 實際回傳定義 |
| **AnimalsPage.tsx** | `mutationFn: (payload: any)` → `mutationFn: (payload: CreateAnimalRequest)` |
| **AnimalAddDialog.tsx** | `pendingPayload: any` → `pendingPayload: CreateAnimalRequest`，`onConfirm: (payload: any)` → `onConfirm: (payload: CreateAnimalRequest)` |
| **AnimalTimelineView.tsx** | `onEdit: (type, record: any)` → `onEdit: (type, record: AnimalTimelineRecord)`，`raw?: any` → `raw?: AnimalTimelineRecord` |

**注意：** `NewAnimalForm` 與 `CreateAnimalRequest` 結構可能略有差異（如 `breed_other`），需確認 API 實際接受欄位後再統一型別。

---

### 階段 4：驗證與收尾（約 15 分鐘）

1. `npm run build` 通過
2. `npm run lint` 無新增錯誤
3. 手動 smoke test：動物新增、編輯、觀察/手術紀錄、安樂死流程

---

## 三、風險與應對

| 風險 | 應對 |
|------|------|
| `CreateAnimalRequest` 與 `NewAnimalForm` 欄位不一致 | 比對 API schema，必要時擴展 `CreateAnimalRequest` 或建立 `NewAnimalPayload` |
| quickMove `onSuccess` 的 `data` 結構複雜 | 定義 `QuickMoveResult` 介面，或暫時用 `unknown` 並在 callback 內做型別收斂 |
| 改動引發其他檔案型別錯誤 | 每階段完成後立即 build，問題限縮在該階段 |

---

## 四、檢查清單

- [x] 階段 0：型別定義補齊（2026-02-28 完成）
- [x] 階段 1：25 處 onError 改為 unknown
- [x] 階段 2：AnimalListTable、handleChange、SurgeryFormDialog
- [x] 階段 3：AnimalsPage payload、AnimalAddDialog、AnimalTimelineView
- [x] 階段 4：build + lint + smoke test
- [x] 更新 `docs/PROGRESS.md` 紀錄完成

---

## 五、參考：既有型別位置

- `frontend/src/types/animal.ts`：Animal、CreateAnimalRequest、UpdateAnimalRequest、AnimalSurgery、AnimalObservation 等
- `frontend/src/lib/validation.ts`：`getApiErrorMessage(error: unknown)`
- `frontend/src/pages/animals/components/AnimalAddDialog.tsx`：`NewAnimalForm`、`DuplicateWarningData`
