# useState → Custom Hooks 重構規劃

> **建立日期：** 2026-03-01  
> **目標：** 將前端元件內散落的 `useState` 抽出為可重用 custom hooks，提升可維護性與測試性。

---

## 一、現況分析

### 1.1 使用量統計

| 類別 | 檔案數 | 說明 |
|------|--------|------|
| 使用 useState 的元件 | ~90 | 散布於 pages、components |
| 既有 global hooks | 7 | `src/hooks/` 目錄 |
| 既有 page/feature hooks | 4+ | 分散於各 feature 目錄 |

### 1.2 既有 Custom Hooks（可沿用）

| Hook | 路徑 | 用途 |
|------|------|------|
| `useConfirmDialog` | `src/hooks/useConfirmDialog.ts` | 取代原生 `confirm()`，統一 Dialog 確認 |
| `useDebounce` | `src/hooks/useDebounce.ts` | 防抖輸入 |
| `usePermissionManager` | `src/hooks/usePermissionManager.ts` | 權限檢查 |
| `useApiError` | `src/hooks/useApiError.ts` | API 錯誤處理 |
| `useUnsavedChangesGuard` | `src/hooks/useUnsavedChangesGuard.ts` | 離開表單前確認 |
| `useSecurityAlerts` | `src/hooks/useSecurityAlerts.ts` | 安全告警 |
| `useHeartbeat` | `src/hooks/useHeartbeat.ts` | Session 心跳 |

### 1.3 既有 Feature Hooks（參考範例）

| Hook | 路徑 | 模式 |
|------|------|------|
| `useAnimalsPageState` | `pages/animals/hooks/useAnimalsPageState.ts` | 拆成 `useAnimalFilters`、`useAnimalDialogs`、`useAnimalSelection`、`useAnimalForms` |
| `useDocumentForm` | `pages/documents/hooks/useDocumentForm.ts` | 複雜表單 + API 整合 |
| `useUserManagement` | `pages/admin/hooks/useUserManagement.ts` | CRUD 對話框 + 表單 + 排序 |
| `useBloodTestTemplates` | `pages/master/hooks/useBloodTestTemplates.ts` | 主檔 CRUD 邏輯 |
| `useSurgeryForm` | `components/animal/useSurgeryForm.ts` | 表單欄位 + 驗證 |

---

## 二、重複模式辨識

### 2.1 列表頁：篩選 / 分頁 / 排序

**出現位置：** `ProductsPage`、`WarehousesPage`、`BloodTestPanelsPage`、`HrLeavePage`、`AdminAuditPage`、`AuditLogsPage`、`PartnersPage`、`RolesPage` 等。

**典型 state 組合：**

```tsx
const [search, setSearch] = useState('')
const [categoryFilter, setCategoryFilter] = useState('all')
const [statusFilter, setStatusFilter] = useState('all')
const [page, setPage] = useState(1)
const [perPage, setPerPage] = useState(20)
const [sortColumn, setSortColumn] = useState<string | null>(null)
const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc')
const [showAdvancedFilters, setShowAdvancedFilters] = useState(false)
```

**建議 hook：** `useListFilters` / `useTableState`

---

### 2.2 對話框開關

**出現位置：** 多數 CRUD 頁面，如 `HrLeavePage`、`HrAnnualLeavePage`、`TreatmentDrugOptionsPage`、`AmendmentsTab`、`ReviewersTab`、`EquipmentPage` 等。

**典型 state：**

```tsx
const [showCreateDialog, setShowCreateDialog] = useState(false)
const [showEditDialog, setShowEditDialog] = useState(false)
const [showImportDialog, setShowImportDialog] = useState(false)
const [showDeleteDialog, setShowDeleteDialog] = useState(false)
```

**建議 hook：** `useDialogSet` / `useDialogs`

---

### 2.3 設定頁表單（API 同步 + dirty 追蹤）

**出現位置：** `SettingsPage`、`CalendarSyncSettingsPage`、`ProfileSettingsPage` 等。

**典型 pattern：**

```tsx
const [companyName, setCompanyName] = useState('')
const [emailHost, setEmailHost] = useState('')
// ... 多個欄位
useEffect(() => {
  if (!sysSettings) return
  setCompanyName(sysSettings.company_name || '')
  setEmailHost(sysSettings.smtp_host || '')
  // ...
}, [sysSettings])
const [settingsDirty, setSettingsDirty] = useState(false)
```

**建議 hook：** `useFormSync` / `useSettingsForm`

---

### 2.4 多步驟表單

**出現位置：** `CreateProductPage`、`TwoFactorSetup`、部分 wizard 流程。

**典型 state：**

```tsx
const [step, setStep] = useState(0)
const [canProceed, setCanProceed] = useState(false)
```

**建議 hook：** `useSteps`

---

### 2.5 密碼可見性切換

**出現位置：** `SettingsPage`、`LoginPage`、`ForceChangePasswordPage`、`PasswordChangeDialog` 等。

**典型 state：**

```tsx
const [showPassword, setShowPassword] = useState(false)
```

**建議 hook：** `useToggle`（通用）或 `usePasswordVisibility`（語意化）

---

### 2.6 選擇狀態（勾選 / 選取項目）

**出現位置：** `useAnimalSelection`、`RolesPage`、表格勾選列、下拉選取。

**典型 state：**

```tsx
const [selectedIds, setSelectedIds] = useState<string[]>([])
const [selectedItem, setSelectedItem] = useState<T | null>(null)
```

**建議 hook：** `useSelection`（擴展現有 `useAnimalSelection` 模式為通用版）

---

### 2.7 Tab 切換

**出現位置：** `HrLeavePage`、`AdminAuditPage`、`BloodTestCostReportPage` 等。

**典型 state：**

```tsx
const [activeTab, setActiveTab] = useState('my-leaves')
```

**建議 hook：** `useTabState`（或沿用 `useState`，因邏輯極簡）

---

### 2.8 日期區間篩選

**出現位置：** `HrLeavePage`、`AdminAuditPage`、`AuditLogsPage`、`AccountingReportPage` 等。

**典型 state：**

```tsx
const [filterFrom, setFilterFrom] = useState('')
const [filterTo, setFilterTo] = useState('')
```

**建議 hook：** `useDateRangeFilter`

---

## 三、建議 Custom Hooks 清單

### 3.1 通用 Hooks（放 `src/hooks/`）

| Hook | 用途 | 優先級 | 預估工時 |
|------|------|--------|----------|
| `useToggle` | 布林切換（密碼可見、進階篩選等） | 高 | 0.5h |
| `useListFilters` | 列表：search、filters、page、perPage、sort | 高 | 1.5h |
| `useDialogSet` | 多個 Dialog 開關集中管理 | 高 | 1h |
| `useSteps` | 多步驟 wizard 流程 | 中 | 1h |
| `useSelection` | 勾選／選取項目 | 中 | 1h |
| `useDateRangeFilter` | 日期區間篩選 | 低 | 0.5h |
| `useFormSync` | 表單與 API 資料同步 + dirty 追蹤 | 中 | 1.5h |

### 3.2 Feature 專用 Hooks（放各 feature 目錄）

| Hook | 路徑 | 用途 | 優先級 |
|------|------|------|--------|
| `useSettingsForm` | `pages/admin/hooks/` | SettingsPage 表單邏輯 | 中 |
| `useLeaveRequestForm` | `pages/hr/hooks/` | HrLeavePage 假單表單 | 中 |
| `useProductListState` | `pages/master/hooks/` | ProductsPage 篩選＋分頁＋排序 | 中 |
| `useCreateProductWizard` | `pages/master/hooks/` | CreateProductPage 步驟與欄位 | 低 |

---

## 四、實作階段與優先順序

### Phase 1：低風險通用 Hooks（建議先行）

1. **`useToggle`**
   - 實作簡單，風險低
   - 可立即替換多處 `const [x, setX] = useState(false)` + `setX(!x)` 或 `setX(v => !v)`
   - 範例檔案：`SettingsPage`、`PasswordChangeDialog`、`LoginPage`

2. **`useDialogSet`**
   - 統一 Dialog 開關管理
   - 範例：`useDialogSet(['create', 'edit', 'import', 'delete'])` 回傳 `{ create: { open, open: fn, close }, ... }`
   - 範例檔案：`TreatmentDrugOptionsPage`、`HrAnnualLeavePage`、`AmendmentsTab`

### Phase 2：列表頁標準化

3. **`useListFilters`**
   - 參數：`{ initialFilters?, defaultPerPage?, syncToUrl?: boolean }`
   - 回傳：`search`、`filters`、`page`、`perPage`、`sort`、`setSearch`、`setFilter`、`setPage` 等
   - 可選：與 `useSearchParams` 整合（參考 `useAnimalFilters`）
   - 範例檔案：`ProductsPage`、`WarehousesPage`、`PartnersPage`、`BloodTestPanelsPage`

### Phase 3：表單與複雜頁面

4. **`useFormSync`**
   - 從 API 載入資料並同步到表單 state
   - 支援 `dirty` 追蹤、`reset`、`hasChanges`
   - 範例檔案：`SettingsPage`

5. **`useSteps`**
   - 步驟索引、next/prev、canProceed
   - 範例檔案：`CreateProductPage`、`TwoFactorSetup`

6. **`useSelection`**
   - 勾選／取消、全選、清空
   - 範例：擴展現有 `useAnimalSelection` 抽成通用版

### Phase 4：Feature 專用（依業務需求）

7. **`useSettingsForm`** — SettingsPage 專用  
8. **`useLeaveRequestForm`** — HrLeavePage 專用  
9. **`useProductListState`** — ProductsPage 專用  

---

## 五、`useToggle` 範例 API

```ts
// src/hooks/useToggle.ts
export function useToggle(initial = false): [boolean, () => void, (value: boolean) => void] {
  const [value, setValue] = useState(initial)
  const toggle = useCallback(() => setValue((v) => !v), [])
  return [value, toggle, setValue]
}
```

---

## 六、`useDialogSet` 範例 API

```ts
// src/hooks/useDialogSet.ts
type DialogKeys = string

export function useDialogSet<K extends DialogKeys>(keys: K[]) {
  const [openDialogs, setOpenDialogs] = useState<Record<K, boolean>>(
    () => Object.fromEntries(keys.map((k) => [k, false])) as Record<K, boolean>
  )
  const open = useCallback((key: K) => setOpenDialogs((p) => ({ ...p, [key]: true })), [])
  const close = useCallback((key: K) => setOpenDialogs((p) => ({ ...p, [key]: false })), [])
  const setOpen = useCallback(
    (key: K) => (v: boolean) => setOpenDialogs((p) => ({ ...p, [key]: v })),
    []
  )
  return {
    isOpen: (key: K) => !!openDialogs[key],
    open,
    close,
    setOpen,
    openDialogs,
  }
}
```

---

## 七、`useListFilters` 範例 API（草案）

```ts
// src/hooks/useListFilters.ts
export interface ListFiltersConfig<TFilters extends Record<string, string>> {
  initialFilters?: TFilters
  defaultPerPage?: number
  syncToUrl?: boolean
  urlParamKeys?: Record<keyof TFilters | 'page' | 'perPage' | 'search', string>
}

export function useListFilters<TFilters extends Record<string, string>>(
  config: ListFiltersConfig<TFilters> = {}
) {
  const {
    initialFilters = {} as TFilters,
    defaultPerPage = 20,
    syncToUrl = false,
  } = config

  const [search, setSearch] = useState('')
  const [filters, setFilters] = useState<TFilters>(initialFilters)
  const [page, setPage] = useState(1)
  const [perPage, setPerPage] = useState(defaultPerPage)
  const [sortColumn, setSortColumn] = useState<string | null>(null)
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc')

  const setFilter = useCallback(<K extends keyof TFilters>(key: K, value: TFilters[K]) => {
    setFilters((p) => ({ ...p, [key]: value }))
    setPage(1)
  }, [])

  const resetFilters = useCallback(() => {
    setSearch('')
    setFilters(initialFilters)
    setPage(1)
    setSortColumn(null)
    setSortDirection('asc')
  }, [initialFilters])

  // 可選：與 useSearchParams 整合
  // ...

  return {
    search,
    setSearch,
    filters,
    setFilter,
    setFilters,
    page,
    setPage,
    perPage,
    setPerPage,
    sortColumn,
    setSortColumn,
    sortDirection,
    setSortDirection,
    resetFilters,
  }
}
```

---

## 八、風險與注意事項

1. **漸進式遷移**：每次只改 1–2 個元件，確保 E2E 與單元測試通過。  
2. **不強制全部替換**：簡單的單一 `useState`（如 `activeTab`）可保留，不必為了用 hook 而用。  
3. **避免過度抽象**：若邏輯僅出現 1–2 次，可維持 inline state。  
4. **型別安全**：`useListFilters`、`useDialogSet` 使用泛型，確保 filter keys 與 dialog keys 型別正確。  
5. **URL 同步**：若 `useListFilters` 與 URL 綁定，需考慮 browser back/forward 行為與預設值一致性。

---

## 九、驗收標準

- [x] 新增 hooks 通過 TypeScript 編譯（2026-03-01）
- [ ] 新增 hooks 通過單元測試
- [ ] 遷移後的頁面功能與 E2E 測試行為一致
- [x] 無新增 linter / TypeScript 錯誤
- [x] 文件更新：`docs/PROGRESS.md` 紀錄完成項目

**Phase 1–2 已完成**（2026-03-01）：useToggle、useDialogSet、useListFilters 實作並遷移 10+ 元件。

**Phase 3 已完成**（2026-03-01 續）：useSteps、useSelection 實作並遷移 CreateProductPage、ProductsPage、TreatmentDrugOptionsPage/ErpImportDialog；TwoFactorSetup 用 useDialogSet。

**Phase 4 已完成**（2026-03-01）：useSettingsForm、useLeaveRequestForm、useProductListState 實作並遷移 SettingsPage、HrLeavePage、ProductsPage。  

---

## 十、參考文件

- [React: Building Your Own Hooks](https://react.dev/learn/reusing-logic-with-custom-hooks)
- 既有 `useAnimalsPageState`、`useDocumentForm`、`useUserManagement` 實作
- `docs/development/IMPROVEMENT_PLAN_MARKET_REVIEW.md`（市場基準與改進計劃）

---

*文件產出於 2026-03-01。*
