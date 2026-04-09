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

---

## 4. Typography

| Property | Value |
|----------|-------|
| Primary Font | `Noto Sans TC`, `Inter`, `system-ui`, sans-serif |
| Monospace Font | `JetBrains Mono`, `Fira Code`, `Source Code Pro`, monospace |
| Font Feature | `rlig`, `calt` enabled |

- 頁面標題：行動端 `text-xl`，桌面端 `text-2xl`，`font-bold tracking-tight`
- SKU 顯示：使用 monospace font + `letter-spacing: 0.5px`

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

- **表格**：行動端 `overflow-x-auto` 可橫向捲動（`.table-responsive`）
- **篩選列**：行動端堆疊 `flex-col`，桌面端並排 `flex-row`（`.filter-row`）
- **Sidebar**：桌面端收摺，行動端 overlay
- **Dialog**：行動端 `font-size: 16px !important` 防止 iOS 縮放，padding 減少
- **Dashboard**：12 / 9 / 6 / 4 / 2 欄 responsive grid

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

- 路由級守衛：`<ProtectedRoute>`、`<AdminRoute>`、`<DashboardRoute>`
- 組件級守衛：`<RequirePermission permission="..." />`
- 混合條件：`<RequirePermission anyOf={[{role:'admin'}, {permission:'training.view'}]}>`
- 前端 Store：`useAuthStore` 提供 `hasRole()`, `hasPermission()`

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

---

## 21. Decisions Log

> **格式：** 反向時間序（新→舊）。新決策加在 header 行之後。

| Date | Decision | Rationale |
|------|----------|-----------|
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
