# 人員管理系統實作計畫

> **建立日期：** 2026-01-XX  
> **依據文件：** `humanResourceManager.md`

---

## 一、需求確認

### 1.1 適用範圍
- ✅ **僅限公司內部員工**（`is_internal = true`）
- ❌ **不包含**：審查人員、IACUC 主席等外部角色

### 1.2 核心功能
1. **請假流程**
   - 請假申請（含緊急請假、事後補請）
   - 多層級審核：主管簽核 → 行政簽核 → 負責人核假
   - 請假狀態追蹤

2. **補休累計**
   - 依加班時數自動累計
   - 補休假使用記錄
   - 補休假到期提醒與轉換

3. **剩餘特休**
   - 依年資自動計算特休假額度
   - 特休假使用記錄
   - 特休假遞延管理

4. **補休計算**
   - 自動計算補休假時數（依加班時數）
   - FIFO 原則使用補休假
   - 到期自動轉換為加班費

---

## 二、檔案規劃清單

### 2.1 資料庫 Migration 檔案

#### 新增 Migration 檔案
- `backend/migrations/024_hr_leave_system.sql`
  - 建立請假相關資料表
  - 建立特休假額度表
  - 建立補休假額度表
  - 建立請假審核記錄表
  - 建立請假記錄表
  - 建立 Google 行事曆整合表（選用）

**主要資料表：**
- `leave_requests` - 請假申請表
- `annual_leave_balances` - 特休假額度表
- `compensatory_leave_balances` - 補休假額度表
- `leave_records` - 請假記錄表
- `leave_approvals` - 請假審核記錄表
- `google_calendar_tokens` - Google 行事曆授權記錄（選用）
- `leave_calendar_events` - 請假與行事曆事件對應表（選用）

**新增 ENUM 類型：**
- `leave_type` - 請假類別（ANNUAL, PERSONAL, SICK, ...）
- `leave_status` - 請假狀態（DRAFT, PENDING_L1, APPROVED, ...）
- `approval_action` - 審核動作（APPROVE, REJECT, REQUEST_REVISION）

### 2.2 後端檔案

#### 2.2.1 Models（資料模型）
- `backend/src/models/leave.rs` - 請假相關資料模型
  - `LeaveRequest` - 請假申請
  - `AnnualLeaveBalance` - 特休假額度
  - `CompensatoryLeaveBalance` - 補休假額度
  - `LeaveRecord` - 請假記錄
  - `LeaveApproval` - 請假審核記錄

#### 2.2.2 Services（業務邏輯）
- `backend/src/services/leave.rs` - 請假業務邏輯
  - 請假申請 CRUD
  - 請假額度檢查
  - 特休假自動計算
  - 補休假自動計算
  - 請假審核流程
  - 部門請假人數限制檢查
  - 補休假到期轉換

#### 2.2.3 Handlers（API 處理器）
- `backend/src/handlers/leave.rs` - 請假 API 處理器
  - `create_leave` - 建立請假申請
  - `list_leaves` - 查詢請假列表
  - `get_leave` - 查詢單一請假
  - `update_leave` - 更新請假申請
  - `delete_leave` - 刪除請假申請
  - `submit_leave` - 送審請假申請
  - `withdraw_leave` - 撤回請假申請
  - `cancel_leave` - 取消請假申請
  - `revoke_leave` - 銷假
  - `approve_leave` - 核准請假
  - `reject_leave` - 駁回請假
  - `request_revision` - 要求補件
  - `get_leave_balances` - 查詢請假額度
  - `get_pending_leaves` - 查詢待審核請假

#### 2.2.4 修改現有檔案
- `backend/src/routes.rs` - 新增人員管理系統路由
  ```rust
  // 請假申請相關
  .route("/hr/leaves", get(handlers::list_leaves).post(handlers::create_leave))
  .route("/hr/leaves/:id", get(handlers::get_leave).put(handlers::update_leave).delete(handlers::delete_leave))
  .route("/hr/leaves/:id/submit", post(handlers::submit_leave))
  .route("/hr/leaves/:id/withdraw", post(handlers::withdraw_leave))
  .route("/hr/leaves/:id/cancel", post(handlers::cancel_leave))
  .route("/hr/leaves/:id/revoke", post(handlers::revoke_leave))
  
  // 請假審核相關
  .route("/hr/leaves/pending", get(handlers::get_pending_leaves))
  .route("/hr/leaves/:id/approve", post(handlers::approve_leave))
  .route("/hr/leaves/:id/reject", post(handlers::reject_leave))
  .route("/hr/leaves/:id/request-revision", post(handlers::request_revision))
  
  // 請假額度查詢
  .route("/hr/leaves/balances", get(handlers::get_leave_balances))
  .route("/hr/leaves/balances/annual", get(handlers::get_annual_leave_balance))
  .route("/hr/leaves/balances/compensatory", get(handlers::get_compensatory_leave_balance))
  
  // 報表相關
  .route("/hr/leaves/reports/summary", get(handlers::get_leave_summary_report))
  .route("/hr/leaves/reports/calendar", get(handlers::get_leave_calendar))
  .route("/hr/leaves/reports/export", get(handlers::export_leave_records))
  ```

- `backend/src/handlers/mod.rs` - 匯出 leave 模組
- `backend/src/models/mod.rs` - 匯出 leave 模型
- `backend/src/services/mod.rs` - 匯出 leave 服務

#### 2.2.5 權限設定
- `backend/migrations/025_hr_leave_permissions.sql` - 新增人員管理系統權限
  - `hr.leave.view.own` - 查看個人請假記錄
  - `hr.leave.create` - 申請請假
  - `hr.leave.edit.own` - 編輯個人請假申請
  - `hr.leave.delete.own` - 刪除個人請假申請
  - `hr.leave.view.department` - 查看部門請假記錄
  - `hr.leave.approve.l1` - 審核一級請假
  - `hr.leave.approve.l2` - 審核二級請假
  - `hr.leave.approve.hr` - 審核人資請假
  - `hr.leave.approve.gm` - 審核總經理請假
  - `hr.leave.view.all` - 查看所有請假記錄
  - `hr.leave.balance.manage` - 管理請假額度
  - `hr.leave.export` - 匯出請假報表

### 2.3 前端檔案

#### 2.3.1 Pages（頁面）
- `frontend/src/pages/hr/LeavesPage.tsx` - 請假申請列表頁
- `frontend/src/pages/hr/LeaveDetailPage.tsx` - 請假申請詳情頁
- `frontend/src/pages/hr/LeaveCreatePage.tsx` - 新增請假申請頁
- `frontend/src/pages/hr/LeaveEditPage.tsx` - 編輯請假申請頁
- `frontend/src/pages/hr/PendingLeavesPage.tsx` - 待審核請假列表頁
- `frontend/src/pages/hr/LeaveBalancesPage.tsx` - 請假額度查詢頁
- `frontend/src/pages/hr/LeaveReportsPage.tsx` - 請假統計報表頁
- `frontend/src/pages/hr/LeaveCalendarPage.tsx` - 請假行事曆頁

#### 2.3.2 Components（元件）
- `frontend/src/components/hr/LeaveFormDialog.tsx` - 請假申請表單對話框
- `frontend/src/components/hr/LeaveApprovalDialog.tsx` - 請假審核對話框
- `frontend/src/components/hr/LeaveBalanceCard.tsx` - 請假額度卡片
- `frontend/src/components/hr/LeaveCalendar.tsx` - 請假行事曆元件
- `frontend/src/components/hr/LeaveStatusBadge.tsx` - 請假狀態標籤
- `frontend/src/components/hr/LeaveTypeSelect.tsx` - 請假類別選擇器

#### 2.3.3 API 定義
- `frontend/src/lib/api.ts` - 新增請假相關 API 型別與函數
  ```typescript
  // 型別定義
  interface LeaveRequest { ... }
  interface AnnualLeaveBalance { ... }
  interface CompensatoryLeaveBalance { ... }
  interface LeaveApproval { ... }
  
  // API 函數
  export const createLeave = (data: CreateLeaveRequest) => ...
  export const listLeaves = (params: ListLeavesParams) => ...
  export const getLeave = (id: string) => ...
  export const updateLeave = (id: string, data: UpdateLeaveRequest) => ...
  export const deleteLeave = (id: string) => ...
  export const submitLeave = (id: string) => ...
  export const withdrawLeave = (id: string) => ...
  export const cancelLeave = (id: string) => ...
  export const revokeLeave = (id: string, data: RevokeLeaveRequest) => ...
  export const approveLeave = (id: string, data: ApproveLeaveRequest) => ...
  export const rejectLeave = (id: string, data: RejectLeaveRequest) => ...
  export const requestRevision = (id: string, data: RequestRevisionRequest) => ...
  export const getLeaveBalances = () => ...
  export const getPendingLeaves = () => ...
  export const getLeaveSummaryReport = (params: ReportParams) => ...
  export const getLeaveCalendar = (params: CalendarParams) => ...
  export const exportLeaveRecords = (params: ExportParams) => ...
  ```

#### 2.3.4 路由設定
- `frontend/src/App.tsx` 或路由設定檔案 - 新增人員管理系統路由
  ```typescript
  <Route path="/hr/leaves" element={<LeavesPage />} />
  <Route path="/hr/leaves/new" element={<LeaveCreatePage />} />
  <Route path="/hr/leaves/:id" element={<LeaveDetailPage />} />
  <Route path="/hr/leaves/:id/edit" element={<LeaveEditPage />} />
  <Route path="/hr/leaves/pending" element={<PendingLeavesPage />} />
  <Route path="/hr/leaves/balances" element={<LeaveBalancesPage />} />
  <Route path="/hr/leaves/reports" element={<LeaveReportsPage />} />
  <Route path="/hr/leaves/calendar" element={<LeaveCalendarPage />} />
  ```

#### 2.3.5 側邊欄導覽
- `frontend/src/layouts/MainLayout.tsx` - 新增人員管理系統選單項目
  ```typescript
  {
    title: '人員管理',
    icon: <Users className="h-5 w-5" />,
    permission: 'hr', // 僅內部員工可見
    children: [
      { title: '請假申請', href: '/hr/leaves' },
      { title: '待審核請假', href: '/hr/leaves/pending' },
      { title: '請假額度', href: '/hr/leaves/balances' },
      { title: '請假報表', href: '/hr/leaves/reports' },
      { title: '請假行事曆', href: '/hr/leaves/calendar' },
    ],
  }
  ```

### 2.4 規格文件

#### 修改現有檔案
- `_Spec.md` - ✅ 已完成，已新增人員管理系統規格
- `role.md` - 需新增人員管理系統權限說明（已在 _Spec.md 中涵蓋）

#### 新增檔案（選用）
- `HR_SYSTEM_API_SPEC.md` - 詳細 API 規格文件（如需要）

---

## 三、儀表板自訂權限評估

### 3.1 需求說明
根據 `todo.md`，需要為各使用者創造「自訂其儀表板顯示項目之權限」。

### 3.2 合理性評估

#### ✅ **合理且可施行**

**優點：**
1. **提升使用者體驗**：不同角色關注的資訊不同，自訂儀表板可讓使用者專注於自己需要的資訊
2. **降低資訊過載**：避免顯示過多不相關的資訊
3. **符合現代系統設計**：大多數現代系統都支援儀表板自訂

**實作難度：** 中等

#### 3.3 實作建議

##### 3.3.1 資料庫設計
```sql
-- 儀表板元件設定表
CREATE TABLE dashboard_widgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE, -- 元件代碼，如 'low-stock-alerts', 'recent-documents'
    name VARCHAR(100) NOT NULL, -- 元件名稱
    description TEXT, -- 元件說明
    category VARCHAR(50), -- 分類：erp, hr, animal, aup
    default_visible BOOLEAN NOT NULL DEFAULT true, -- 預設是否顯示
    required_permission VARCHAR(100), -- 需要的權限代碼
    sort_order INTEGER DEFAULT 0, -- 預設排序
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 使用者儀表板設定表
CREATE TABLE user_dashboard_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    widget_id UUID NOT NULL REFERENCES dashboard_widgets(id) ON DELETE CASCADE,
    is_visible BOOLEAN NOT NULL DEFAULT true, -- 是否顯示
    sort_order INTEGER NOT NULL DEFAULT 0, -- 排序順序
    widget_config JSONB, -- 元件特定設定（如：顯示筆數、時間範圍等）
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, widget_id)
);

CREATE INDEX idx_user_dashboard_settings_user_id ON user_dashboard_settings(user_id);
```

##### 3.3.2 後端 API
```rust
// 取得可用的儀表板元件列表
GET /api/dashboard/widgets

// 取得使用者的儀表板設定
GET /api/dashboard/settings

// 更新使用者的儀表板設定
PUT /api/dashboard/settings
Body: {
  "widgets": [
    { "widget_id": "uuid", "is_visible": true, "sort_order": 1, "config": {...} },
    ...
  ]
}
```

##### 3.3.3 前端實作
- 在 `DashboardPage.tsx` 中：
  1. 讀取使用者的儀表板設定
  2. 依設定顯示/隱藏元件
  3. 依排序順序排列元件
  4. 提供「自訂儀表板」按鈕，開啟設定對話框

- 新增元件：
  - `frontend/src/components/dashboard/DashboardSettingsDialog.tsx` - 儀表板設定對話框
  - `frontend/src/components/dashboard/DashboardWidget.tsx` - 可拖曳的儀表板元件包裝器

##### 3.3.4 權限控制
- 權限代碼：`dashboard.customize` - 自訂儀表板權限
- 所有使用者預設擁有此權限（可選）

#### 3.4 建議實作優先順序
1. **Phase 1（基礎）**：顯示/隱藏元件
2. **Phase 2（進階）**：排序調整
3. **Phase 3（進階）**：元件特定設定（如顯示筆數、時間範圍）

---

## 四、後端接口完整性檢查

### 4.1 現有接口評估

根據 `humanResourceManager.md` 第 6.2 節的 API 規格，需要實作以下接口：

#### ✅ 需要新增的接口（目前不存在）

**請假申請相關：**
- [ ] `POST /api/hr/leaves` - 建立請假申請
- [ ] `GET /api/hr/leaves` - 查詢請假申請列表
- [ ] `GET /api/hr/leaves/:id` - 查詢單一請假申請
- [ ] `PUT /api/hr/leaves/:id` - 更新請假申請
- [ ] `DELETE /api/hr/leaves/:id` - 刪除請假申請
- [ ] `POST /api/hr/leaves/:id/submit` - 送審請假申請
- [ ] `POST /api/hr/leaves/:id/withdraw` - 撤回請假申請
- [ ] `POST /api/hr/leaves/:id/cancel` - 取消請假申請
- [ ] `POST /api/hr/leaves/:id/revoke` - 銷假

**請假審核相關：**
- [ ] `GET /api/hr/leaves/pending` - 查詢待審核請假列表
- [ ] `POST /api/hr/leaves/:id/approve` - 核准請假申請
- [ ] `POST /api/hr/leaves/:id/reject` - 駁回請假申請
- [ ] `POST /api/hr/leaves/:id/request-revision` - 要求補件

**請假額度查詢：**
- [ ] `GET /api/hr/leaves/balances` - 查詢個人請假額度
- [ ] `GET /api/hr/leaves/balances/annual` - 查詢特休假額度
- [ ] `GET /api/hr/leaves/balances/compensatory` - 查詢補休假額度

**報表相關：**
- [ ] `GET /api/hr/leaves/reports/summary` - 請假統計報表
- [ ] `GET /api/hr/leaves/reports/calendar` - 請假行事曆
- [ ] `GET /api/hr/leaves/reports/export` - 匯出請假記錄

### 4.2 計算邏輯接口

根據 `humanResourceManager.md`，需要實作以下計算邏輯：

#### ✅ 需要實作的計算邏輯

1. **特休假自動計算**（第 3.2 節）
   - 依年資計算特休假天數
   - 年度中到職的比例計算
   - 遞延特休假處理

2. **補休假自動計算**（第 4.1 節）
   - 依加班時數計算補休假時數
   - FIFO 原則使用補休假
   - 補休假到期轉換為加班費

3. **請假額度檢查**（第 6.3.1 節）
   - 檢查特休假額度是否足夠
   - 檢查補休假額度是否足夠
   - 檢查年度請假上限（如事假 14 天、病假 30 天）

4. **部門請假人數限制檢查**（第 6.3.2 節）
   - 同一部門同一天請特休人數不得超過部門總人數的 30%

5. **補休假自動到期轉換**（第 6.3.4 節）
   - 定期任務：每日檢查即將到期或已到期的補休假
   - 自動轉換為加班費

### 4.3 建議實作順序

1. **Phase 1：基礎資料表與模型**
   - 建立 migration 檔案
   - 建立資料模型（Models）
   - 建立基本 CRUD API

2. **Phase 2：請假申請流程**
   - 請假申請 CRUD
   - 請假送審、撤回、取消
   - 請假狀態管理

3. **Phase 3：審核流程**
   - 多層級審核邏輯
   - 待審核列表
   - 審核動作（核准、駁回、補件）

4. **Phase 4：額度計算與管理**
   - 特休假自動計算
   - 補休假自動計算
   - 額度查詢 API

5. **Phase 5：報表與整合**
   - 請假統計報表
   - 請假行事曆
   - 與通知系統整合
   - Google 行事曆整合（選用）

---

## 五、規格文件確認

### 5.1 已完成的規格文件
- ✅ `_Spec.md` - 已新增人員管理系統規格（第 4 節）
- ✅ `humanResourceManager.md` - 詳細的人力資源管理章程（完整）

### 5.2 需要確認的項目

#### ✅ 規格完整性檢查

1. **資料庫設計** ✅
   - `humanResourceManager.md` 第 6.1 節提供完整的資料表設計
   - 包含所有必要的欄位與索引

2. **API 規格** ✅
   - `humanResourceManager.md` 第 6.2 節提供完整的 API 端點規格
   - 涵蓋所有必要的操作

3. **業務邏輯規則** ✅
   - `humanResourceManager.md` 第 6.3 節提供詳細的業務邏輯規則
   - 包含特休假計算、補休假計算、額度檢查等

4. **權限設定** ✅
   - `humanResourceManager.md` 第 6.5 節提供完整的權限代碼
   - `_Spec.md` 第 4.5 節已整合權限說明

5. **審核流程** ✅
   - `humanResourceManager.md` 第 5 節提供完整的審核流程說明
   - 包含多層級審核與時效要求

### 5.3 潛在問題與建議

#### ⚠️ 需要注意的事項

1. **使用者組織架構**
   - 需要確認 `users` 表是否有 `department_id` 欄位
   - 需要確認如何識別「直屬主管」、「部門主管」
   - **建議**：在 `users` 表中新增 `department_id` 和 `manager_id` 欄位

2. **加班記錄來源**
   - 補休假需要從加班記錄計算
   - 需要確認是否有「加班記錄」系統或資料表
   - **建議**：如無，需要先建立 `overtime_records` 表

3. **通知整合**
   - 請假申請、審核需要發送通知
   - 需要確認現有通知系統是否足夠
   - **建議**：使用現有的通知系統，新增請假相關通知類型

4. **Google 行事曆整合（選用）**
   - 需要 Google OAuth 2.0 設定
   - 需要額外的依賴套件（`google-calendar3`, `yup-oauth2`）
   - **建議**：Phase 1 先不實作，後續再新增

---

## 六、總結

### 6.1 實作範圍確認

✅ **已確認的需求：**
- 請假流程（含緊急請假、事後補請）
- 多層級審核（主管 → 行政 → 負責人）
- 補休累計與計算
- 特休假管理與計算
- 僅限內部員工使用

✅ **規格文件完整性：**
- `humanResourceManager.md` 提供完整的規格
- `_Spec.md` 已整合人員管理系統規格
- 資料庫設計、API 規格、業務邏輯規則都已定義

✅ **後端接口規劃：**
- 所有必要的 API 端點都已規劃
- 計算邏輯都已定義
- 需要實作的項目已列出

### 6.2 下一步行動

1. **建立資料庫 Migration**
   - 建立 `024_hr_leave_system.sql`
   - 建立 `025_hr_leave_permissions.sql`

2. **實作後端 API**
   - 建立 Models、Services、Handlers
   - 新增路由設定

3. **實作前端頁面**
   - 建立請假相關頁面與元件
   - 新增側邊欄選單項目

4. **測試與驗證**
   - 單元測試
   - 整合測試
   - 使用者驗收測試

---

**文件維護者：** 開發團隊  
**最後更新：** 2026-01-XX
