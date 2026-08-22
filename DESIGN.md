# iPig System — Design System Document

> 實驗動物管理平台 (iPig) 設計系統文件
> Last updated: 2026-03-24

---

## 1. Product Overview

iPig 是一套完整的實驗動物管理平台，整合 IACUC/AUP 計畫書管理、動物照護追蹤、ERP 進銷存、HR 人事、以及報表分析。目標使用者為實驗室管理者、研究人員、獸醫師、倉管人員和行政人員。

**核心子系統：**
- **AUP 計畫書** — 動物使用計畫書的建立、審查、修正（Amendment）
- **動物管理** — 動物登記、健康紀錄、血液檢測、用藥追蹤
- **ERP 模組** — 產品主檔、進銷存單據、庫存倉位管理
- **HR 模組** — 出缺勤、請假、加班、年假額度、訓練紀錄
- **報表中心** — 庫存現況、成本摘要、血檢分析、會計報表
- **系統管理** — 使用者 / 角色 / 權限、稽核日誌、通知路由、設施管理

---

## 1.5 Brand Identity

> 本節為 `/brand-guidelines` skill 的主要參照來源。所有 UI 文字、設計決策需符合以下宣言。

### Product Identity

| 項目 | 規範 |
|------|------|
| **正式名稱** | iPig 實驗動物管理平台 |
| **英文名稱** | iPig Laboratory Animal Management System |
| **簡稱** | iPig |
| **大小寫規範** | `iPig`（小 i，大 P）|
| **禁止寫法** | IPIG、Ipig、i-pig、i_pig、ipig |
| **產品定位** | 實驗室專業管理工具（非消費者應用） |

### Brand Personality

| 特質 | 說明 |
|------|------|
| **專業** | 實驗室環境，精確、可靠、可稽核 |
| **效率** | 減少行政負擔，讓研究人員專注於科學 |
| **合規** | IACUC / AUP 法規遵循為核心 |
| **可信** | 資料完整性與稽核日誌為基石 |

### Visual Identity Summary

- **主色**：藍色系 `--primary`（217° 91% 60%）→ 傳遞信任與專業感
- **子系統**：藍色系分化（180°–240°），統一感中保有區別（詳見 §16）
- **字體**：`Noto Sans TC` + `Inter` → 雙語專業感
- **圓角**：`0.5rem`（柔和但不失正式）
- **設計哲學**：功能導向 > 裝飾性；資訊密度優先

### Voice & Tone（UI 文字規範）

- **語言**：繁體中文（台灣）優先，英文為輔（雙語介面，所有 UI 文字透過 `t()` i18n 化）
- **語調原則**：

| ✅ 應該 | ❌ 避免 |
|--------|--------|
| 直接、具體：「新增動物紀錄」 | 模糊：「點擊以開始您的旅程」 |
| 溫暖但不隨意：「尚無紀錄」 | 過度輕鬆：「Oops, nothing here!」 |
| 動作導向：按鈕用動詞開頭（新增、儲存、匯出） | 名詞按鈕：「確定」不如「儲存變更」 |
| 錯誤訊息說明原因 + 下一步 | 只說「發生錯誤，請再試一次」 |
| 空白狀態有 CTA 引導（見 §17） | 空白狀態無任何引導文字 |

---

## 2. Design Stack

| Layer | Technology |
|-------|-----------|
| UI Framework | React 18 + TypeScript |
| Build Tool | Vite |
| Component Library | shadcn/ui（基於 Radix UI primitives） |
| Styling | Tailwind CSS + CSS Variables (HSL) |
| Icons | Lucide React |
| State Management | Zustand（auth, sidebar, uiPreferences） |
| Server State | TanStack React Query |
| Forms | React Hook Form + Zod resolvers |
| Routing | React Router v6 |
| i18n | react-i18next（zh-TW / en） |
| Calendar | FullCalendar |
| Dashboard Grid | react-grid-layout |
| Drag & Drop | @dnd-kit |
| Charts | Recharts |
| Testing | Vitest + Playwright + Storybook |

---

## 3. Color System

基於 HSL CSS Variables，支援 Light / Dark 雙主題。

### 3.1 Core Palette

| Token | Light | Dark | Usage |
|-------|-------|------|-------|
| `--background` | `0 0% 100%` (白) | `222.2 84% 4.9%` (深藍黑) | 頁面背景 |
| `--foreground` | `222.2 84% 4.9%` | `210 40% 98%` | 主文字 |
| `--primary` | `217.2 91.2% 59.8%` (藍) | 同 | 主要操作、連結 |
| `--secondary` | `210 40% 96.1%` (淺灰藍) | `217.2 32.6% 17.5%` | 次要背景 |
| `--destructive` | `0 84.2% 60.2%` (紅) | `0 62.8% 30.6%` | 刪除、錯誤 |
| `--success` | `142 76% 36%` (綠) | `142 76% 46%` | 成功狀態 |
| `--muted` | `210 40% 96.1%` | `217.2 32.6% 17.5%` | 禁用、輔助文字 |
| `--accent` | `210 40% 96.1%` | `217.2 32.6% 17.5%` | Hover 高亮 |

### 3.2 SKU Segment Colors（產品編碼色彩系統）

專為 SKU 編碼的各段落設計的語義色彩：

| Token | Color | Meaning |
|-------|-------|---------|
| `--sku-name` | 紫色 263° | 產品名稱 |
| `--sku-spec` | 青色 187° | 規格 |
| `--sku-unit` | 琥珀色 38° | 單位 |
| `--sku-date` | 翠綠色 160° | 日期 |
| `--sku-seq` | 靛藍色 239° | 序號 |
| `--sku-chk` | 粉紅色 330° | 檢查碼 |

### 3.3 Data-viz 色票例外（禁硬編色的認可例外）

> §3「禁硬編色（一律用 token）」規則針對**狀態語意**色彩。以下 **data-viz / 使用者識別色**屬**認可例外**，允許使用 Tailwind 調色盤或 hex：
>
> - **欄舍/區域顏色**（`useFacilityLayout` 9 色 blue/orange/yellow/cyan/purple/amber/green/red/gray）：使用者自選，用來視覺辨識欄舍分組，**非狀態語意**；且 orange/cyan/purple 等無對應 status token。
> - **圖表色**（Recharts 系列色、`PainAssessmentChart` 圖例色）：data-viz 調色盤需多樣色相區分序列。
>
> **判準**：顏色代表「狀態/語意」（成功/警告/危險/資訊）→ 用 status token（禁硬編）；顏色只是「識別標籤 / 圖表序列」→ 可用調色盤（本例外）。（2026-07-04 決策）

### 3.4 狀態色兩階（Solid / Soft）— 何時用哪階

> 每個 status 語意色（success/warning/error/info/neutral/purple）在 `index.css` 都有**兩階** token。**兩階都是設計系統的一部分，刻意保留**，依「密度 / 強調度」選階，**不是二選一全站統一**。

| 階 | Token 寫法 | 長相 | 適用場景 |
|----|-----------|------|----------|
| **Solid（實心）** | `bg-status-X-solid` + `text-white` | 飽和底 + 白字 | **單顆、需醒目**的狀態；**計數 / 通知** badge；`destructive` 等需抓眼球的警示 |
| **Soft（柔和）** | `bg-status-X-bg` + `text-status-X-text` + `border-status-X-border` | 淺底 + 深字 + 淺框 | **表格每列、chip 群**等成群、重複、密集出現的狀態標籤（降低「聖誕燈牆」視覺噪音） |

**判準一句話**：**成群/密集重複 → soft；單顆/需搶眼 → solid。**

**實作**：一律走 `StatusBadge`（`tone="solid" | "soft"`，預設 `solid`）或 `Badge variant`（solid）。**禁止**再手刻 `bg-status-X-bg text-status-X-text` 於**狀態標籤**（提示框 / alert 面板 / 區塊底色的淺底不在此限——那不是 badge）。

```tsx
// 表格每列狀態 / chip 群 → soft
<StatusBadge variant="success" tone="soft">實驗中</StatusBadge>
// 單顆醒目狀態 / 警示 → solid（預設）
<StatusBadge variant="warning">超標</StatusBadge>
```

> ⚠️ **已知 a11y 待辦**：solid `warning`（`38 92% 50%` 橘黃）配白字對比約 2:1，**不過 WCAG AA**。多數 warning 落在表格 → 改 soft（深字淺底，合格）後問題大幅縮小；殘留的 solid warning 之前景色另案處理（見 §20 Debt Registry）。（2026-07-04 決策）

---

## 4. Typography

| Property | Value |
|----------|-------|
| Primary Font | `Noto Sans TC`, `Inter`, `system-ui`, sans-serif |
| Monospace Font | `JetBrains Mono`, `Fira Code`, `Source Code Pro`, monospace |
| Font Feature | `rlig`, `calt` enabled |

- 頁面標題：行動端 `text-xl`，桌面端 `text-2xl`，`font-bold tracking-tight`
- SKU 顯示：使用 monospace font + `letter-spacing: 0.5px`
- 行動端字體縮放：`≤768px` → 20px，桌面端維持 16px（預設）
- **字體大小偏好**：`<html>` 套用 `font-size-large`（桌面 17px / 行動 22px）或 `font-size-xl`（桌面 18px / 行動 24px）class；存於 `uiPreferences` Zustand store，在 ProfileSettingsPage 設定

### 4.1 Dashboard Widget 字體階層（強制）

> **背景**：shadcn `CardTitle` 預設 `text-2xl font-semibold`、`CardDescription` 預設 `text-sm`。各 widget 若未覆寫預設、或各自選字級，會造成同一儀表板內標題大小不一（曾出現 14px vs 24px 並排）。以下為 widget 內所有文字的唯一字級來源，新增 / 修改 widget 必須對齊。

#### 標題與描述

| 角色 | class（必用） | 說明 |
|------|--------------|------|
| Widget 標題 | `text-sm font-medium` | **必覆寫** shadcn `CardTitle` 預設 `text-2xl`；不可用 `font-semibold` |
| 標題 icon | `h-4 w-4` + token 色 | 統一小 icon；禁止 `h-5 w-5`；色彩用 token（見下方禁則） |
| Widget 描述 | `text-xs text-muted-foreground` | **必覆寫** `CardDescription` 預設 `text-sm` |

#### 內文三級制

| 級別 | class | 用途 | 範例 |
|------|-------|------|------|
| **L1** 統計大數字 | `text-2xl font-bold` | KPI / 計數 / 主視覺數字 | MyProjects 的總數、ErpWidgets StatWidget 的 value |
| **L2** 主要內文 / 列表項 | `text-sm` | 列表標題、主要內容、表格儲存格 | VetComments 留言者、列表項標題 |
| **L3** 次要 metadata | `text-xs text-muted-foreground` | 日期、標籤、輔助說明、區塊小標 | 日期、耳標 badge、「審查中 / 執行中」分組標 |

> 區塊小標（分組 heading）若需強調可加 `font-medium`，但字級維持 L3 `text-xs`。

#### 最小字級下限

> **widget 內任何文字（含 Badge、metadata、標籤）最小為 `text-xs`（12px）。** 10px 以下對長時間閱讀者（含放大偏好使用者）負擔過大，一律不使用。

#### 禁則

- ❌ `text-[10px]` 等任意像素值（VetComments / Staff / Calendar 舊用法，含 Badge 內）→ 一律改 `text-xs`
- ❌ widget 內使用 `text-lg`（介於 L1/L2 之間造成飄移）→ 升 L1 或降 L2
- ❌ widget 標題不覆寫 `CardTitle` 預設（會吃到 `text-2xl`）
- ❌ icon 顏色硬編碼 `text-*-500`（`text-emerald-500`、`text-indigo-500` 等）→ 用 §3 token（如 `text-status-success-solid`、`text-primary`）
- ❌ widget 殼層不用 `Card`（QuickActionsWidget 舊用裸 `<div>`+`<h3>`）→ 統一用 `Card` + `CardHeader` + `CardTitle`

---

## 5. Spacing & Border Radius

| Token | Value |
|-------|-------|
| `--radius` | `0.5rem` (8px) |
| `border-radius-lg` | `0.5rem` |
| `border-radius-md` | `calc(0.5rem - 2px)` = 6px |
| `border-radius-sm` | `calc(0.5rem - 4px)` = 4px |
| Container max-width | `1400px` (2xl breakpoint) |
| Container padding | `2rem` |

---

## 6. Layout Architecture

### 6.1 Page Structure

```
┌─────────────────────────────────────────┐
│  TopBar (語言切換 / 通知 / 用戶選單)      │
├────────┬────────────────────────────────┤
│        │                                │
│  Side  │     Page Content (Outlet)      │
│  bar   │                                │
│        │                                │
│  可收   │     (Suspense + ErrorBoundary) │
│  摺    │                                │
├────────┴────────────────────────────────┤
│  Toaster / SessionTimeout / CookieConsent│
└─────────────────────────────────────────┘
```

### 6.2 Sidebar

- 桌面端：可收摺（`sidebarOpen` state），寬度切換
- 行動端：overlay 模式（`mobileSidebarOpen`）
- 支援 **拖放排序** 導航項目（@dnd-kit）
- 按角色 / 權限過濾顯示的選單項
- 群組可展開 / 收合（expandedItems state）

### 6.3 Auth Flow

```
Login → [Force Change Password?] → Dashboard / My Projects
         ↓ (無 dashboard 權限)
         → /my-projects（研究人員首頁）
```

- 角色型首頁：admin / ERP 角色 → `/dashboard`，一般研究人員 → `/my-projects`
- Session timeout 警告彈窗
- Impersonation 模式（admin 可模擬其他用戶）

---

## 7. Component Inventory

### 7.1 Base UI Components (shadcn/ui, 46 files)

AlertDialog, Badge, Button, Card, Checkbox, ConfirmDialog, DatePicker, DateTextInput,
DeleteReasonDialog, Dialog, ErrorBoundary, FileUpload, FormField, HandwrittenSignaturePad,
Input, Label, LoadingOverlay, PageErrorBoundary, PanelIcon, Repeater, Select, Skeleton,
Slider, Switch, Table, TableSkeleton, Tabs, Toast, Toaster, Tooltip, etc.

### 7.2 Domain Components

| Directory | Domain | Key Components |
|-----------|--------|----------------|
| `components/animal/` | 動物管理 | 動物卡片、健康紀錄、血檢結果 |
| `components/protocol/` | AUP 計畫書 | 計畫書表單、審查流程 |
| `components/protocols/` | 計畫書列表 | 篩選、狀態標籤 |
| `components/product/` | 產品主檔 | SKU 生成器、產品表單 |
| `components/sku/` | SKU 系統 | 段落色彩、預覽、驗證 |
| `components/dashboard/` | 儀表板 | Widget 系統、可拖放排列 |
| `components/inventory/` | 庫存管理 | 倉位佈局、庫存卡 |
| `components/warehouse/` | 倉庫管理 | 倉位視覺化 |
| `components/partner/` | 合作夥伴 | 夥伴列表、表單 |
| `components/admin/` | 系統管理 | 用戶管理、角色配置 |
| `components/layout/` | 佈局 | Sidebar, NotificationDropdown, PasswordChangeDialog |
| `components/auth/` | 認證 | ProtectedRoute, RequirePermission |

### 7.3 Custom Hooks (18+)

| Hook | Purpose |
|------|---------|
| `useConfirmDialog` | 確認彈窗流程 |
| `useUnsavedChangesGuard` | 離開未儲存變更警告 |
| `useDateRangeFilter` | 日期範圍篩選 |
| `useListFilters` | 列表頁共用篩選邏輯 |
| `useDebounce` | 搜尋輸入防抖 |
| `useSelection` | 多選邏輯 |
| `useSteps` | 步驟流程 |
| `useTabState` | Tab 狀態同步 URL |
| `useSecurityAlerts` | 安全警報 |
| `useHeartbeat` | Session 心跳 |
| `useCalendarSync` | 行事曆同步 |
| `useSkuCategories` | SKU 類別管理 |
| `usePermissionManager` | 權限 CRUD |

---

## 8. Animation System

所有動畫使用 Tailwind keyframes + CSS animations：

| Animation | Duration | Usage |
|-----------|----------|-------|
| `fade-in` | 300ms ease-out | 頁面進場 |
| `slide-in` | 300ms ease-out | Sidebar 展開 |
| `segment-fill` | 300ms ease | SKU 段落填入 |
| `segment-highlight` | 500ms ease | SKU 段落高亮 |
| `success-bounce` | 500ms ease | 成功狀態 |
| `slide-in-right` | 300ms ease | 步驟切換 |
| `shake` | 300ms ease | 錯誤提示 |
| `shimmer` | 2s infinite | 載入骨架 |
| `skeleton-pulse` | 1.5s ease infinite | 骨架屏 |
| `draw-check` | 400ms ease (delay 200ms) | 打勾動畫 |
| `blink-caret` | 1s step-end infinite | 輸入游標 |

---

## 9. Responsive Strategy

### Breakpoints (Tailwind default)

| Name | Min-width | Use |
|------|-----------|-----|
| `sm` | 640px | — |
| `md` | 768px | 手機 → 桌面切換點 |
| `lg` | 1024px | — |
| `xl` | 1280px | — |
| `2xl` | 1400px | Container max |

### Patterns

- **表格**：詳見下方「Table RWD 規則」
- **篩選列**：行動端堆疊 `flex-col`，桌面端並排 `flex-row`（`.filter-row`）
- **Sidebar**：桌面端收摺，行動端 overlay
- **Dialog**：行動端 `font-size: 16px !important` 防止 iOS 縮放，padding 減少
- **Dashboard**：12 / 9 / 6 / 4 / 2 欄 responsive grid

### Dialog 寬度標準（重要）

> 全站對話框寬度**只用以下 5 種標準尺寸**，禁止各處硬編 `max-w-*` / `w-[…px]`（曾散落 ~16 種寬度、含 7 種硬編 px）。透過 `<DialogContent size="…">` / `<AlertDialogContent size="…">` 指定，定義於 `components/ui/dialogSize.ts`。

| size | Tailwind | px | 用途 |
|------|----------|----|------|
| `sm` | `max-w-md` | 448 | 確認框 / 2FA / 刪除原因 / 單欄極簡 |
| `md`（預設）| `max-w-lg` | 512 | 標準 CRUD 表單（絕大多數） |
| `lg` | `max-w-2xl` | 672 | 較大 / 雙欄表單 |
| `xl` | `max-w-4xl` | 896 | 寬內容 / 多欄 / 含表格（如匯入體重） |
| `2xl` | `max-w-6xl` | 1152 | 資料密集檢視（稽核紀錄、計畫比對、報告檢視） |

**規則**：
- 新增 dialog 一律用 `size` prop，**不寫** `className="max-w-…"` 控寬。
- 不夠寬時往上選一級標準，不可硬編中間值。
- 行動端（`<sm`）由 base 元件統一處理（bottom-sheet 滿版），`size` 僅影響桌面端 max-width。
- 盤點與遷移工具：`docs/design/dialog-width/`嚗?冽?隞塚?（標準預覽 + 全站盤點清單）。

### Table RWD 規則（重要）

> 產品表格 RWD 曾因 `table-fixed` + `overflow-x-hidden` + 最小欄寬總和 > 容器寬度，造成「操作」欄被裁切。以下為永久守則。

**三選一原則**：當容器寬度不足以容納表格所有欄位時，以下三條必須至少放棄一條——不得全部堅持。

| 原則 | 說明 |
|------|------|
| ❶ 不可裁剪 | 重要欄位（操作、狀態）任何斷點都必須完整顯示 |
| ❷ 不可隱藏 | 禁止用 `hidden xl:table-cell` 之類讓欄位消失 |
| ❸ 不可橫向卷動 | 禁用外層 `overflow-x-auto`，保持 `overflow-x-hidden` |

**唯一解法**：當容器寬度 `< 表格最小總和` 時，**整張表格切換為卡片列表**。
參考實作：`frontend/src/pages/master/components/ProductTable.tsx` 的 `MIN_TABLE_WIDTH` + `canRenderTable` + `ProductCardList`。

**設計時必算的數字**：
- 策略 B 最小欄寬：次要欄（規格 / 單位 / 批號 / 效期）= 65px，其餘維持原設計
- 計算「最小表格總和」寫死為常數，作為切卡片的臨界點
- Desktop 切 mobile 用 `containerRef.clientWidth`，**不是** viewport 寬（側邊欄會扣）

### Table RWD QA Checklist

PR 含表格修改時，必須在以下寬度驗證：

- [ ] **320px**（手機）：卡片模式，無橫向卷軸
- [ ] **768px**（md 斷點，側邊欄展開）：容器真實寬約 500–600px，應走卡片模式
- [ ] **768px**（側邊欄收起）：容器約 720px，表格剛好可 render
- [ ] **1024px**（lg）：表格策略 B，所有欄位顯示完整
- [ ] **1280px**（xl）：表格策略 B 或切到策略 A
- [ ] **1536px+**：表格策略 A，名稱欄自動吸收剩餘空間
- [ ] 「操作」欄在所有斷點都可見（檢查表頭 + 按鈕群組皆不被裁）
- [ ] 無水平卷軸（`document.body.scrollWidth <= window.innerWidth`）
- [ ] 切換側邊欄展開 / 收起時，表格↔卡片切換流暢、不出現裁切 flash

### Table 視覺標準（全域統一，2026-07-03 起強制）

> 基準表格＝`pages/admin/components/UserTable.tsx`。**所有新增 / 修訂的表格一律照此標準**。
> 多數項目已由**共用元件**內建（改元件即全站生效），個別表**不得覆蓋回舊樣式**。

| 面向 | 標準 | 由誰保證 |
|------|------|----------|
| **共用元件** | 一律用 `@/components/ui/table` 的 `Table/TableHeader/TableHead/TableBody/TableRow/TableCell`。**禁止原生 `<table>`**（含展開列明細） | 手動遵守 |
| **表頭底色** | `bg-muted/50`（`TableHead` 預設內建） | `components/ui/table.tsx` |
| **列高密度** | 緊湊 `py-1.5`（`TableCell` 預設內建）。**勿在個別表加回 `py-4`** | `components/ui/table.tsx` |
| **可排序表頭** | `SortableTableHead` 或 `button` + `ArrowUpDown/ArrowUp/ArrowDown` | 手動遵守 |
| **狀態標籤** | `<Badge variant>` 或 `StatusBadge`。**兩階選階（§3.4）**：表格每列成群狀態 → `StatusBadge tone="soft"`；單顆醒目 / 警示 → solid（預設）。**禁止**把 `bg-green-100`/`bg-yellow-100`/`text-red-500` 等 Tailwind 調色盤，或手刻 `bg-status-X-bg text-status-X-text`，塞進狀態標籤 className | `badge.tsx` / `status-badge.tsx` |
| **空 / 載入態** | 一律共用 `TableSkeleton`（載入）+ `TableEmptyRow`（空）。禁止表格內手刻 `Loader2` / 純文字 / 自訂 EmptyState | 手動遵守 |
| **顏色** | 只用 CSS variable token（`bg-muted`/`text-muted-foreground`/`text-destructive`/`bg-status-*`）。禁硬編色（呼應 §3） | 手動遵守 |
| **RWD** | `@container` 表格↔卡片 @600px（見上）。**新表禁止 JS ResizeObserver 斷點** | 手動遵守 |

**狀態 → Badge variant 對映慣例**：正向/完成 → `success`；警示/進行中/待審 → `warning`；失敗/危險/作廢 → `destructive`；次要/草稿/中性 → `secondary`；資訊/已提交 → `default`。

**新增 / 修訂表格 self-check**：
- [ ] 用共用 Table 元件、無原生 `<table>`
- [ ] 沒有硬編色 Badge（`grep bg-.*-100` 於狀態欄應為 0）
- [ ] 空/載入態用 `TableSkeleton` + `TableEmptyRow`
- [ ] 沒有在個別表覆蓋回 `py-4` / 移除 `bg-muted/50`

---

## 10. Accessibility

- 基於 Radix UI primitives — 內建 ARIA 屬性、鍵盤導航、焦點管理
- 手寫簽名板：`touch-action: none` 防止觸控穿透
- 對話框：`overscroll-behavior: contain` + `body[data-scroll-locked]` 防止背景滾動
- 載入狀態：300ms 延遲的 spinner（避免閃爍）
- 自訂 scrollbar 樣式

---

## 11. Key Interaction Patterns

### 11.1 CRUD 頁面模式

```
ListPage → DetailPage → EditPage
  ↓ (新增)
  CreatePage
```

- 列表頁：篩選 + 搜尋 + 分頁 + 批次操作
- 詳情頁：唯讀檢視 + 操作按鈕
- 編輯頁：React Hook Form + Zod 驗證
- 刪除：`DeleteReasonDialog`（需填寫原因）

### 11.2 Dashboard Widget System

- 使用者可自訂 widget 佈局（react-grid-layout）
- 編輯模式切換（鎖定 / 解鎖）
- Widget 依權限顯示 / 隱藏
- 佈局持久化至後端（user preferences API）

### 11.3 SKU 生成系統

- 多段落色彩編碼：名稱（紫）→ 規格（青）→ 單位（橙）→ 日期（綠）→ 序號（靛）→ 檢查碼（粉）
- 即時預覽 + hover 動效（上移 + shadow）
- Monospace 字體 + letter-spacing

### 11.4 手寫簽名

- Canvas-based 手寫簽名板
- 觸控裝置支援（`touch-action: none`）
- 簽名狀態徽章（已簽 / 未簽）
- SVG 格式儲存 + 預覽

### 11.5 Documents / AUP 流程

- 多步驟表單（`useSteps`）
- 未儲存變更保護（`useUnsavedChangesGuard`）
- 角色型審查流程
- Amendment（變更申請）追蹤

---

## 12. i18n

- 支援 `zh-TW`（繁體中文）和 `en`（英文）
- 語言切換在 TopBar
- 所有 UI 文字通過 `useTranslation()` 的 `t()` 函數
- Sidebar 導航標題有專用翻譯函數

---

## 13. Permission Model

```
User → Roles → Permissions
```

### 13.1 Permission Naming Convention

**Standardized naming pattern for all permissions:**

| Operation Type | Pattern | Example | Semantics |
|---|---|---|---|
| **Create** | `animal.{resource}.create` | `animal.vet_advice.create` | 新增/建立資源 |
| **Read** | `animal.{resource}.read` | `animal.vet_advice.read` | 查看/讀取資源 |
| **Update** | `animal.{resource}.update` | `animal.vet_advice.update` | 修改/編輯資源 |
| **Delete** | `animal.{resource}.delete` | `animal.vet_advice.delete` | 刪除資源（最高權限） |
| **Custom Action** | `animal.{resource}.{action}` | `animal.vet.recommend` | 特定業務行為 |

**Key Constraints:**
- ✅ Delete operations MUST use `{resource}.delete` or `{resource}.{subaction}.delete` permission
- ❌ Never reuse create/write permissions for delete operations (semantic safety)
- ✅ All new handlers MUST follow this pattern
- 🔄 Legacy permissions planned for gradual migration per sprint

**Example: Veterinary Advice Deletion**
```rust
pub async fn delete_vet_advice(current_user: &CurrentUser) -> Result<()> {
    // ✅ Correct: Use delete-specific permission
    require_permission!(current_user, "animal.vet_advice.delete");
    // ... proceed with deletion
}

// ❌ Anti-pattern: Reusing create permission for delete
// require_permission!(current_user, "animal.vet.recommend"); // WRONG
```

### 13.2 Frontend Permission Guards

- 路由級守衛：`<ProtectedRoute>`、`<AdminRoute>`、`<DashboardRoute>`
- 組件級守衛：`<RequirePermission permission="..." />`
- 混合條件：`<RequirePermission anyOf={[{role:'admin'}, {permission:'training.view'}]}>`
- 前端 Store：`useAuthStore` 提供 `hasRole()`, `hasPermission()`

### 13.3 Backend Permission Enforcement

- **Macro:** `require_permission!(user, "{permission_string}")` in handler layer
- **Middleware:** `Permission` cache (5-min TTL DashMap) + DB fallback for stale window mitigation
- **Coverage:** Target >95% for sensitive operations (create/update/delete); audit via CI grep

---

## 14. Design Principles

1. **功能導向** — 實驗室操作效率優先，非裝飾性設計
2. **shadcn/ui 一致性** — 所有基礎組件遵循 shadcn/ui 規範，HSL CSS Variables 主題
3. **漸進式披露** — 角色型首頁、權限篩選導航、展開式群組
4. **防錯設計** — 未儲存警告、刪除需填原因、Session timeout 提示
5. **雙語支援** — 所有使用者可見文字均 i18n 化
6. **響應式優先** — 行動端觸控友善，桌面端資訊密度高
7. **可客製化** — Dashboard widget 可拖放、Sidebar 可排序
8. **Design Token 優先** — 禁止硬編碼 `text-slate-*`、`bg-blue-*` 等，一律使用 CSS Variable token（`text-primary`、`bg-muted` 等）

---

## 15. Button Guidelines (按鈕規範)

### 15.1 按鈕高度一致性

同一 toolbar / PageHeader 內的所有按鈕（含 primary action）必須使用相同的 `size`，禁止混用不同高度。

| 場景 | Size | 高度 | 說明 |
|------|------|------|------|
| Toolbar / PageHeader actions | `sm` | h-9 (36px) | 列表頁操作列：新增、匯入、匯出、編輯分類等 |
| 表單提交區域 | `default` | h-10 (40px) | 獨立表單頁底部：儲存、送出、取消 |
| 重要步驟 CTA | `lg` | h-11 (44px) | 多步驟表單最終送出、Landing page CTA |
| 表格行內操作 | `icon` | h-10 w-10 | 編輯、刪除等 icon-only 按鈕 |

**規則：**
- PageHeader `actions` 區域內所有按鈕統一使用 `size="sm"`，包括 primary action（如「新增產品」）
- 禁止同一按鈕群組中混用 `sm` + `default` 造成高度不齊
- 表單頁底部的送出/取消按鈕可使用 `default` 或 `lg`

### 15.2 按鈕顏色策略

按鈕顏色全站統一，**不按子系統分化**。子系統差異化透過 Sidebar active indicator、頁面 accent 等非按鈕元素表達。

| Variant | 用途 | 說明 |
|---------|------|------|
| `default` (primary) | 主要操作 | 新增、儲存、送出 — 統一使用 `--primary` 藍色 |
| `outline` | 次要操作 | 匯入、匯出、編輯分類等輔助操作 |
| `destructive` | 破壞性操作 | 刪除、停用 |
| `ghost` | 低優先操作 | 取消選擇、關閉 |
| `secondary` | 中性操作 | 不強調的替代選項 |

**規則：**
- 所有子系統的 primary button 統一使用 `--primary` 色彩
- 子系統色相（`--subsystem-*`）僅用於 Sidebar active indicator、Badge、Accent 等非操作元素
- 操作語義（藍=主要、紅=破壞、灰=次要）優先於子系統品牌識別

---

## 16. Subsystem Color Identity (藍色系分化)

各子系統使用藍色系近鄰色相區分，保持統一的專業感。用於 Auth 背景、Sidebar active indicator、頁面 accent 等場景。

| Subsystem | Hue | CSS Variable | Usage |
|-----------|-----|-------------|-------|
| **AUP 計畫書** | 220° (深藍) | `--subsystem-aup` | 計畫書審查流程頁面 |
| **ERP 進銷存** | 200° (青藍) | `--subsystem-erp` | 單據、庫存、產品頁面 |
| **動物管理** | 180° (綠藍) | `--subsystem-animal` | 動物紀錄、健康管理頁面 |
| **HR 人事** | 240° (藍紫) | `--subsystem-hr` | 出缺勤、請假、訓練頁面 |
| **系統管理** | 210° (灰藍) | `--subsystem-admin` | 用戶管理、稽核、設定頁面 |

### Auth 頁面背景規範

所有 Auth 相關頁面（登入、忘記密碼、重設密碼、強制變更密碼）統一使用：
```
bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900
```
**禁止**使用 `via-purple-900` 或其他脫離藍色系的漸層。

Standalone 頁面（404、隱私政策、服務條款）統一使用：
```
bg-gradient-to-br from-slate-50 to-blue-50
```

---

## 17. Empty State Design (統一空白狀態)

### EmptyState Component 規範

所有空白狀態必須使用統一的 `<EmptyState>` 元件，位於 `components/ui/empty-state.tsx`。

```tsx
interface EmptyStateProps {
  icon: LucideIcon        // 語義化 icon（非 icon-in-circle）
  title: string           // 簡短標題（如「尚無動物紀錄」）
  description?: string    // 引導性描述（如「新增第一筆動物紀錄以開始使用」）
  action?: {
    label: string         // CTA 文字（如「新增動物」）
    onClick: () => void
    icon?: LucideIcon
  }
}
```

### Empty State 設計原則

1. **溫暖而非冷漠** — 「尚無動物紀錄」而非「No data found」
2. **引導而非空白** — 每個 empty state 至少有一個 CTA
3. **Icon 直接顯示** — 使用 `text-muted-foreground` 的 icon，不用 `rounded-full bg-*` 包裹
4. **i18n** — 所有文字通過 `t()` 函數

### 適用場景

| 場景 | Title | Description | CTA |
|------|-------|-------------|-----|
| 列表無資料 | 「尚無{實體}」 | 「新增第一筆...」 | 新增按鈕 |
| 搜尋無結果 | 「找不到符合的結果」 | 「請嘗試調整篩選條件」 | 清除篩選 |
| 首次使用 | 「歡迎使用 {模組}」 | 簡短功能介紹 | 開始使用 |
| 權限不足 | 「無權限存取」 | 「請聯繫管理員」 | 返回首頁 |

---

## 18. First-Time User Experience

### 歡迎狀態

新用戶首次登入（無任何資料）時，各首頁應顯示歡迎型 EmptyState：

- **Dashboard（管理者）**：歡迎訊息 + 快速設定引導（新增用戶、設定倉庫等）
- **MyProjects（研究人員）**：歡迎訊息 + 「建立第一個計畫書」CTA
- **Animals（獸醫師）**：歡迎訊息 + 「登錄第一隻動物」CTA

### 判斷方式

使用後端 API 回傳的資料筆數。若列表 API 回傳 `total: 0` 且無任何篩選條件，視為首次使用狀態。

---

## 19. Accessibility Roadmap

### 現有基礎（來自 Radix UI）
- Dialog, Select, Checkbox 等元件內建 ARIA 屬性
- 焦點管理、鍵盤導航
- Dialog scroll lock

### 待補充項目（優先級由高到低）

| Item | Priority | Description |
|------|----------|-------------|
| Skip-to-content | Medium | 在 MainLayout 加入 `<a href="#main-content" className="sr-only focus:not-sr-only">` |
| ARIA landmarks | Medium | `<main>`, `<nav>`, `<aside>` 語義標籤 |
| 觸控目標 | Medium | 所有可點擊元素最小 44x44px |
| 色彩對比度 | Low | WCAG AA 標準（4.5:1 文字、3:1 大文字） |
| 鍵盤快捷鍵 | Low | Sidebar 導航、表格操作 |
| Screen reader 測試 | Low | NVDA / VoiceOver 測試清單 |

---

## 20. Design Debt Registry

以下為設計審查中發現的技術債，按優先級排列：

| # | Item | Severity | Files Affected | Effort |
|---|------|----------|---------------|--------|
| 1 | ~~Auth 頁面紫色漸層統一為藍色~~ | ~~High~~ | ~~ForceChangePasswordPage, ResetPasswordPage, ForgotPasswordPage~~ | DONE |
| 2 | ~~建立 EmptyState 統一元件~~ | ~~High~~ | ~~新增 components/ui/empty-state.tsx + 替換 15+ 頁面~~ | DONE |
| 3 | ~~Auth/Standalone 頁面遷移到 design token~~ | ~~Medium~~ | ~~10 files, 79 處硬編碼色彩~~ | DONE |
| 4 | ~~減少 icon-in-circle 模式~~ | ~~Medium~~ | ~~11 處 rounded-full...flex items-center justify-center~~ | DONE |
| 5 | ~~ERP 導航模式統一（Tab vs 獨立路由）~~ | ~~Low~~ | ~~ErpPage.tsx + routing~~ | DONE |
| 6 | ~~ProfileSettingsPage 去除過度裝飾~~ | ~~Low~~ | ~~漸層文字、深色卡片~~ | DONE |
| 7 | ~~首次使用引導~~ | ~~Low~~ | ~~Dashboard, MyProjects, Animals~~ | DONE |
| 8 | ~~表格空/載入態手刻 → 共用 TableSkeleton/TableEmptyRow~~ | ~~Low~~ | ~~14 檔轉換；3 類架構不同誠實跳過：dialog 級 guard(BloodTestDetailDialog)、容器級雙版 gate(HrAnnualLeave)、dashboard widget overlay(ErpWidgets)~~ | DONE |
| 9 | ~~animal 動作按鈕硬編色 `bg-purple-600`/`hover:bg-purple-700`/`hover:bg-green-700` → token~~ | ~~Low~~ | ~~21 檔 44 處（新增/儲存/確認按鈕）~~ | DONE |
| 10 | ~~`useFacilityLayout` 欄舍色票 → 決定維持顏色（data-viz 例外，見 §3.3）~~ | ~~Low~~ | ~~useFacilityLayout.ts~~ | DONE(維持) |
| 11 | ~~簽章狀態 badge `@apply bg-green-100`/`bg-amber-100` → token~~ | ~~Low~~ | ~~index.css `.signature-status-signed`/`-unsigned`~~ | DONE |
| 12 | ~~animal 剩餘硬編色小尾巴：`bg-indigo-600`（4 個轉讓鈕）→ primary、`hover:text-green-700`（連結色）→ token~~ | ~~Low~~ | ~~TransferInitiateForm/TransferTab/TransferSignatureForms/VetPatrolReportDialog~~ | DONE |
| 13 | ~~data-viz 色票政策 → 決定：維持顏色、註記為禁硬編色的認可例外（見 §3.3）~~ | ~~Low~~ | ~~useFacilityLayout.ts, PainAssessmentChart.tsx~~ | DONE |
| 14 | 落在表格 / chip 群的成群 `StatusBadge` 由 solid 改 `tone="soft"`；手刻 `bg-status-X-bg text-status-X-text` 的**狀態標籤**收斂進 `StatusBadge tone="soft"`（提示框 / alert / 區塊底色不屬狀態標籤，不在範圍） | Medium | admin/ERP/HR 表格 StatusBadge + 預約頁 chip | Medium |
| 15 | solid `warning`（`38 92% 50%` 橘黃）白字對比僅約 2:1，不過 WCAG AA → 前景色改深字或降 solid 亮度 | Low | `badge.tsx` / `status-badge.tsx` + 用到 solid warning 之處 | Low |

---

## 21. Decisions Log

> **格式：** 反向時間序（新→舊）。新決策加在 header 行之後。

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-07-04 | **狀態色兩階（Solid / Soft）制度化**：每個 status 語意色的 `-solid`（醒目/單顆/警示）與 `-bg+-text`（密集/成群）**刻意保留為兩階**，依「密度」選階（成群→soft / 單顆→solid），非二選一統一。統一走 `StatusBadge`（`tone` prop，預設 `solid`）與 `Badge variant`，禁手刻；規範入 §3.4 | 使用者觀察到兩種配色（飽底白字 vs 淺底深字）並存不一致。成熟設計系統（Primer/Radix/Atlassian）皆保留兩階解不同問題：成群標籤用 solid 會成「聖誕燈牆」噪音、單顆狀態用 soft 又不夠搶眼，故不統一為一。順帶記錄 solid warning 白字不過 AA（見 §20 #15）。預覽：`docs/design/status-badge-tone-tiers.html`嚗?冽?隞塚? |
| 2026-07-04 | GLP 合規 8 頁改掛「系統管理 → GLP 合規」可收合子群組（三層巢狀），非平鋪或獨立頂層群組 | 頁面原有路由但側邊欄無入口（盤點時「找不到」）。子群組對齊盤點清單自身的「系統管理 · GLP/合規」分組、保留系統管理清爽、沿用 `NestedGroup` 既有渲染與 Guest 可見（GLP read-only 賣點）；各葉節點以自身 `xxx.view` 權限 gating、空群組自動隱藏。 |
| 2026-07-04 | Design Debt #8–#13 全數清償 + data-viz 色票例外定案 | 表格空/載入態收斂共用元件（14 檔）；animal 按鈕/簽章/indigo 硬編色 → token（52 檔）；`bg-status-warning-bg0` typo 全清。data-viz 色票（欄舍 9 色、圖表色）決定**維持顏色**並列為禁硬編色的認可例外（見 §3.3）。Debt Registry §20 #1–#13 全 DONE。 |
| 2026-07-03 | 全域表格視覺標準化（以 UserTable 為基準） | 91 張表稽核發現不一致（硬編色 Badge、表頭底色缺漏、列高鬆、空/載入態手刻）。訂共用元件內建緊湊列高(`py-1.5`)+表頭底色(`bg-muted/50`)、狀態一律實心 Badge/StatusBadge、禁硬編色；標準見 §9「Table 視覺標準」。清冊：`docs/design/table-consistency-catalog.html`嚗?冽?隞塚? |
| 2026-07-03 | 修 Badge `warning` variant 壞 class | `bg-status-warning-bg0`（typo，無此 token）→ `bg-status-warning-solid`；全站 warning badge 原為透明底白字（不可見） |
| 2026-07-03 | **使用者管理表格（`UserTable`）欄寬設計**：`Email` 彈性吸收剩餘寬 + `名稱` 永遠完整；container-query 隱藏順序 狀態`@[900px]`→角色`@[720px]`→最後登入`@[560px]`；**角色 badge 上下堆疊**；**狀態欄直書**（`[writing-mode:vertical-rl]` 表頭＋pill 字元上下排、無排序箭頭、點整格切換排序）；**最後登入可排序**；**操作 2×3 grid** | 原表 auto-layout 造成 CJK 姓名折行、表頭「狀態」折行、動作欄過寬。採 `ProductTable` 的 Tailwind v4 container-query pattern（禁 JS ResizeObserver）。使用者逐項拍板：角色改直排以縮欄、狀態改直書把該欄壓到一字寬並以點格取代箭頭、最後登入補排序、動作 6 顆改 2×3。互動預覽 `docs/design/user-table-column-width.html`嚗?冽?隞塚? |
| 2026-06-30 | **Dialog 寬度標準化為 5 種**（sm448/md512/lg672/xl896/2xl1152），由 `DialogContent`/`AlertDialogContent` 的 `size` prop 取用（`components/ui/dialogSize.ts`），禁止硬編 `max-w-*`/`w-[px]` | 全站盤點發現 159 個 dialog 散落 ~16 種寬度（含 425/450/500/600/860/900/1100/1200px 等 7 種硬編 + `w-[90vw]`），同語意 dialog 寬度不一。收斂為個位數標準尺寸；最大宗 `max-w-lg`(~95 處)=md 不變故低風險，`size` 預設 md + `cn` 用 twMerge → 既有 className override 仍相容，可漸進遷移。盤點/預覽工具見 `docs/design/dialog-width/`嚗?冽?隞塚? |
| 2026-06-26 | 動物病程時間軸（`AnimalTimelineView`）由左右交錯改**單側左軸 EHR 版式**：日期分組吸頂 + 里程碑（進場/手術/結局）以**尺寸放大非顏色**強調 + 體重收斂為 Recharts 趨勢圖（`AnimalWeightTrendCard`）+ 型別篩選 chips；**沿用原每型別配色** + 觀察/手術保留操作按鈕。安樂死與其犧牲/採樣**合併為單一事件** | 左右交錯傷垂直掃讀、手機破版；資料型病程 log 業界（EHR / GitHub / Linear）皆單軸。十種事件型別全上色會色彩過載、真正該突出的死亡反不明顯，故里程碑改以尺寸強調、顏色僅沿用語義（使用者明確要求保留原配色，避免功能退化故保留操作按鈕）。體重屬高頻低訊號，逐筆洗版時間軸 → 收斂為趨勢圖（逐筆值入 tooltip，比照 EHR vitals）。安樂死動物必有犧牲紀錄（見 euthanasia/sacrifice 模型），原顯示兩筆重複故合併。決策經互動 mockup `docs/design/animal-timeline/animal-timeline-mockup.html`嚗?冽?隞塚?（A/B/C，A 定案）。PR #799 / #800 |
| 2026-06-19 | Dashboard widget 字體統一為「標題 `text-sm font-medium` + 描述 `text-xs` + 內文三級制（L1 `text-2xl font-bold` / L2 `text-sm` / L3 `text-xs`）」，**最小字級下限 `text-xs`（12px，含 Badge）**，icon 統一 `h-4 w-4` + token 色，規範入 §4.1 | shadcn `CardTitle` 預設 `text-2xl`，部分 widget（ErpWidgets 大卡）未覆寫造成標題 14px vs 24px 並排；描述一半 `text-xs` 一半吃預設 `text-sm`；內文混用 `text-[10px]`/`text-lg` 各自飄移。10px 對放大偏好使用者負擔過大故設 12px 下限。後續抽共用殼層元件根治 DRY |
| 2026-06-13 | 動物試驗申請須知＝「全院共用單一生效版＋版次制」；簽署閘門僅於**初次送審**（DRAFT→SUBMITTED）檢查；已被簽署引用的版本**內容不可變**（改建新版次） | 須知是院區層級受控文件（非個別計畫快照），故全院一份 + partial unique index 保證至多一個生效版。簽署是「初審前一次性同意」：`acknowledge` 僅允許 DRAFT、簽署卡片亦限 DRAFT，若補件重送仍檢查會造成「要簽卻無法簽」死鎖，故閘門 scope 到初次送審。受控文件完整性：已有人簽署的版本若可改內容＝竄改其已同意之條款，故 `update_content` 守衛「有簽署引用則拒改」（同 tx 防 TOCTOU）。正文存純文字（前端 `whitespace-pre-wrap`，無 markdown 渲染器、避 R31 strict CSP 風險） |
| 2026-06-04 | 「我的計劃」(`get_my_protocols`) 改純成員制，不依 `aup.protocol.view_all` 看全部 | EXPERIMENT_STAFF 等角色帶 view_all（給「計畫書管理」全覽用），舊邏輯讓「我的計劃」也爆全部。改為單一機制：可見＝成員（`user_protocols`）＋ PI（`pi_user_id`）＋ SD（`study_director_user_id`）＋ 被指派審查。要把計畫歸到某使用者＝將其加為成員，不靠角色權限放寬可見範圍。全覽走「計畫書管理」(`list_protocols`，view_all 者不受影響) |
| 2026-05-29 | 既有已通過計劃以「匯入成 live protocol（跳審查、直接 APPROVED）」處理，而非另建平行 archive 表 | 會計與豬隻管理皆接 `protocols` 表，平行 archive 接不上；唯有成為 live protocol 才能進行會計/管理。跳過審查屬合規敏感動作，故以獨立 audit `event_type=PROTOCOL_IMPORT_APPROVED` + iacuc_no 唯一性 + 明確權限（`aup.protocol.import_approved`，給 EXPERIMENT_STAFF）留下「匯入既有核准」軌跡與區隔 |
| 2026-04-18 | 表格 RWD 改為「容器寬度切卡片」，不再隱藏欄位 | 先前「依斷點隱藏次要欄」造成「操作」欄在 md 斷點被 overflow-x-hidden 裁切。新規：不裁剪 / 不隱藏 / 不橫向捲動三選一時，唯一解為整張表切卡片；minTableWidth 設為 720px（次要欄最小 65px） |
| 2026-04-17 | Dialog 行動端改為 bottom sheet | 手機操作拇指觸及區在下方，底部滑入比置中彈窗更易操作；桌面端維持置中 |
| 2026-04-17 | 字體大小偏好只提供放大方向 | 行動端主要問題為文字太小，故偏好選項為標準/大/特大，不提供縮小 |
| 2026-04-17 | 表格欄位依斷點精簡 | md(<768px) 隱藏次要欄，lg(<1024px) 隱藏輔助欄；保留主要識別欄+操作欄供行動使用 |
| 2026-04-17 | FilterBar 行動端搜尋常駐+額外篩選可收合 | 搜尋為最高頻操作，不收合；額外篩選折疊節省螢幕空間，有活躍篩選自動展開 |
| 2026-03-26 | 文件記錄規則統一 | 統一時間排序（反向）、表格欄位、編號格式；變更日誌單一來源（PROGRESS.md §9） |
| 2026-03-26 | 按鈕顏色不按子系統分化 | 操作語義（藍=主要、紅=破壞）優先於子系統品牌，子系統差異化透過 Sidebar/Badge 等非按鈕元素 |
| 2026-03-26 | 按鈕高度統一規範 | PageHeader/toolbar 內所有按鈕統一 `size="sm"`，消除 primary action 與次要按鈕的高度差異 |
| 2026-03-24 | ERP 導航模式記為設計債 | Tab 嵌入 vs 獨立路由不一致，未來重構統一 |
| 2026-03-24 | 加入 a11y roadmap | 記錄無障礙改進方向，不立即實作 |
| 2026-03-24 | 加入首次使用引導規範 | 提升新用戶 onboarding 體驗 |
| 2026-03-24 | Auth 背景統一為藍色漸層 | 消除紫色漸層 AI slop，保持品牌一致性 |
| 2026-03-24 | 統一 EmptyState 元件規範 | 消除各頁面不一致的空白狀態處理 |
| 2026-03-24 | 子系統色彩採用藍色系分化 | 保持實驗室專業感，用色相微調（180°-240°）區分子系統 |
| 2026-03-24 | Initial DESIGN.md created | 由 /design-consultation + /plan-design-review 建立 |
