# UI/UX Guidelines

> **Version**: 2.0  
> **Last Updated**: 2026-01-18  
> **Audience**: Designers, Frontend Developers

---

## 1. Design Principles

### 1.1 Core Principles

| Principle | Description |
|-----------|-------------|
| **Clarity** | Clear visual hierarchy, obvious actions |
| **Efficiency** | Minimize clicks for common tasks |
| **Consistency** | Uniform patterns across modules |
| **Accessibility** | Support Chinese/English, light/dark modes |

### 1.2 Technology Stack

| Tool | Purpose |
|------|---------|
| React 18 | UI Framework |
| TailwindCSS | Utility-first styling |
| shadcn/ui | Component library |
| Zustand | State management |
| React Query | Server state |
| React Router 6 | Client-side routing |

---

## 2. Layout Structure

### 2.1 Main Layout

```
┌─────────────────────────────────────────────────────────────┐
│ Header (Navbar)                                 [Notifications] [User] │
├──────────────┬──────────────────────────────────────────────┤
│              │                                              │
│   Sidebar    │              Main Content                    │
│   (260px)    │                                              │
│              │                                              │
│   [Logo]     │   ┌─────────────────────────────────────┐   │
│              │   │     Page Header                      │   │
│   [Nav]      │   ├─────────────────────────────────────┤   │
│              │   │                                     │   │
│              │   │     Page Content                    │   │
│              │   │                                     │   │
│              │   └─────────────────────────────────────┘   │
│              │                                              │
└──────────────┴──────────────────────────────────────────────┘
```

### 2.2 Navigation Structure

```
📊 儀表板 (Dashboard)
📋 動物使用計畫 (Protocols)
🐷 動物管理 (Animal Management)
   ├── 豬隻 (Pigs)
   └── 我的計劃 (My Projects)
📦 ERP
   ├── 單據管理 (Documents)
   ├── 庫存查詢 (Inventory)
   └── 報表 (Reports)
👥 人員管理 (HR)
   ├── 出勤打卡 (Attendance)
   ├── 請假申請 (Leave)
   └── 日曆設定 (Calendar)
⚙️ 系統管理 (Admin)
   ├── 使用者管理 (Users)
   ├── 角色管理 (Roles)
   └── 稽核日誌 (Audit)
🏢 基礎資料 (Master Data)
   ├── 產品 (Products)
   ├── 夥伴 (Partners)
   └── 倉庫 (Warehouses)
```

---

## 3. Component Guidelines

### 3.1 Buttons

| Type | Usage |
|------|-------|
| Primary | Main actions (Submit, Save) |
| Secondary | Alternative actions |
| Outline | Less prominent actions |
| Ghost | Subtle actions, icons |
| Destructive | Delete, dangerous actions |

### 3.2 Forms

- Use labels above inputs
- Group related fields
- Show validation inline
- Required fields marked with asterisk
- Use appropriate input types (date, select, textarea)

### 3.3 Tables

- Zebra striping for readability
- Sortable columns with indicators
- Pagination for large datasets
- Action buttons on right
- Status badges with colors

### 3.4 Cards

- Use for grouped information
- Clear header with title
- Consistent padding
- Optional footer for actions

---

## 4. Color Scheme

### 4.1 Brand Colors

| Name | Usage |
|------|-------|
| Primary | Main actions, links |
| Secondary | Supporting elements |
| Accent | Highlights |

### 4.2 Semantic Colors

| Color | Usage |
|-------|-------|
| Green | Success, approved |
| Red | Error, rejected, destructive |
| Yellow | Warning, pending |
| Blue | Info, in progress |
| Gray | Neutral, disabled |

### 4.3 Status Badge Colors

| Status | Color |
|--------|-------|
| Draft | Gray |
| Pending | Yellow |
| Approved | Green |
| Rejected | Red |
| In Progress | Blue |
| Completed | Green |

---

## 5. Icons

Use **Lucide React** icons consistently:

| Category | Icons |
|----------|-------|
| Navigation | Home, FileText, Pig, Package, Users, Settings |
| Actions | Plus, Edit, Trash, Eye, Download, Upload |
| Status | Check, X, Clock, AlertTriangle |
| UI | ChevronDown, ChevronRight, Menu, Search |

---

## 6. Responsive Design

### 6.1 Breakpoints

| Name | Width | Usage |
|------|-------|-------|
| Mobile | <768px | Full-width, collapsed sidebar |
| Tablet | 768-1024px | Reduced sidebar |
| Desktop | >1024px | Full layout |

### 6.2 Mobile Considerations

- Collapsible sidebar
- Stack cards vertically
- Touch-friendly targets (44px min)
- Simplified tables

---

## 7. Animations

### 7.1 Transitions

| Element | Duration | Easing |
|---------|----------|--------|
| Hover | 150ms | ease-out |
| Modal | 200ms | ease-in-out |
| Sidebar | 300ms | ease-in-out |
| Toast | 300ms | ease-out |

### 7.2 Loading States

- Skeleton loaders for content
- Spinner for buttons during submit
- Progress bar for long operations

---

## 8. Internationalization

### 8.1 Supported Languages

| Code | Language |
|------|----------|
| zh-TW | 繁體中文 (default) |
| en | English |

### 8.2 Guidelines

- Use translation keys, not hardcoded strings
- Support RTL for future expansion
- Format dates/numbers per locale
- Store preference in user settings

---

## 9. Accessibility

### 9.1 Requirements

- ARIA labels for interactive elements
- Keyboard navigation support
- Focus indicators
- Sufficient color contrast
- Alt text for images

### 9.2 Testing

- Screen reader testing
- Keyboard-only navigation
- Color blindness simulation

---

## 10. Dark Mode

### 10.1 Implementation

- System preference detection
- User preference override
- CSS variables for theming
- Smooth transition between modes

### 10.2 Color Adjustments

| Element | Light | Dark |
|---------|-------|------|
| Background | White | Slate 900 |
| Text | Slate 900 | Slate 100 |
| Cards | White | Slate 800 |
| Borders | Slate 200 | Slate 700 |

---

*Next: [Naming Conventions](./11_NAMING_CONVENTIONS.md)*
