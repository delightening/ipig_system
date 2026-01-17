import { useEffect } from 'react'
import { Routes, Route, Navigate, Outlet } from 'react-router-dom'
import { Toaster } from '@/components/ui/toaster'
import { useAuthStore } from '@/stores/auth'

// Layouts
import { MainLayout } from '@/layouts/MainLayout'
import { AuthLayout } from '@/layouts/AuthLayout'

// Auth Pages
import { LoginPage } from '@/pages/auth/LoginPage'
import { ForgotPasswordPage } from '@/pages/auth/ForgotPasswordPage'
import { ResetPasswordPage } from '@/pages/auth/ResetPasswordPage'
import { ForceChangePasswordPage } from '@/pages/auth/ForceChangePasswordPage'

// Dashboard
import { DashboardPage } from '@/pages/DashboardPage'

// Master Data Pages
import { ProductsPage } from '@/pages/master/ProductsPage'
import { CreateProductPage } from '@/pages/master/CreateProductPage'
import { ProductDetailPage } from '@/pages/master/ProductDetailPage'
import { WarehousesPage } from '@/pages/master/WarehousesPage'
import { PartnersPage } from '@/pages/master/PartnersPage'

// Document Pages
import { DocumentsPage } from '@/pages/documents/DocumentsPage'
import { DocumentDetailPage } from '@/pages/documents/DocumentDetailPage'
import { DocumentEditPage } from '@/pages/documents/DocumentEditPage'

// Inventory Pages
import { InventoryPage } from '@/pages/inventory/InventoryPage'
import { StockLedgerPage } from '@/pages/inventory/StockLedgerPage'

// Admin Pages
import { UsersPage } from '@/pages/admin/UsersPage'
import { RolesPage } from '@/pages/admin/RolesPage'
import { SettingsPage } from '@/pages/admin/SettingsPage'
import { AuditLogsPage } from '@/pages/admin/AuditLogsPage'
import { AdminAuditPage } from '@/pages/admin/AdminAuditPage'

// HR Pages
import { HrAttendancePage } from '@/pages/hr/HrAttendancePage'
import { HrLeavePage } from '@/pages/hr/HrLeavePage'
import { HrOvertimePage } from '@/pages/hr/HrOvertimePage'
import { CalendarSyncSettingsPage } from '@/pages/hr/CalendarSyncSettingsPage'

// Report Pages
import { ReportsPage } from '@/pages/reports/ReportsPage'
import { StockOnHandReportPage } from '@/pages/reports/StockOnHandReportPage'
import { StockLedgerReportPage } from '@/pages/reports/StockLedgerReportPage'
import { PurchaseLinesReportPage } from '@/pages/reports/PurchaseLinesReportPage'
import { SalesLinesReportPage } from '@/pages/reports/SalesLinesReportPage'
import { CostSummaryReportPage } from '@/pages/reports/CostSummaryReportPage'

// AUP Protocol Pages
import { ProtocolsPage } from '@/pages/protocols/ProtocolsPage'
import { ProtocolDetailPage } from '@/pages/protocols/ProtocolDetailPage'
import { ProtocolEditPage } from '@/pages/protocols/ProtocolEditPage'

// My Projects Pages
import { MyProjectsPage } from '@/pages/my-projects/MyProjectsPage'
import { MyProjectDetailPage } from '@/pages/my-projects/MyProjectDetailPage'

// Pig Management Pages
import { PigsPage } from '@/pages/pigs/PigsPage'
import { PigDetailPage } from '@/pages/pigs/PigDetailPage'
import { PigEditPage } from '@/pages/pigs/PigEditPage'
import { PigSourcesPage } from '@/pages/pigs/PigSourcesPage'

// Protected Route component
function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, user } = useAuthStore()

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }

  // 首次登入強制變更密碼
  if (user?.must_change_password) {
    return <Navigate to="/force-change-password" replace />
  }

  return <>{children}</>
}

// Force Change Password Route - 需要登入但未變更密碼
function ForcePasswordRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, user } = useAuthStore()

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }

  // 如果已變更密碼，導向 dashboard
  if (!user?.must_change_password) {
    return <Navigate to="/dashboard" replace />
  }

  return <>{children}</>
}

// Erp Route - 僅限具備 ERP 權限的使用者
function ErpRoute({ children }: { children?: React.ReactNode }) {
  const { user, hasRole, hasPermission } = useAuthStore()

  const hasErpAccess = hasRole('admin') ||
    user?.roles.some(r => ['purchasing', 'approver', 'WAREHOUSE_MANAGER'].includes(r)) ||
    user?.permissions.some(p => p.startsWith('erp.'))

  if (!hasErpAccess) {
    return <Navigate to="/my-projects" replace />
  }

  return children ? <>{children}</> : <Outlet />
}

function App() {
  const { checkAuth, isAuthenticated, user, hasRole } = useAuthStore()

  // Validate token on app initialization
  useEffect(() => {
    const token = localStorage.getItem('access_token')
    if (token) {
      checkAuth().catch(() => {
        // Token validation failed, will be handled by checkAuth
      })
    }
  }, [checkAuth])

  // 判斷首頁導向
  const getHomeRedirect = () => {
    const hasErpAccess = hasRole('admin') ||
      user?.roles.some(r => ['warehouse', 'purchasing', 'sales', 'approver'].includes(r)) ||
      user?.permissions.some(p => p.startsWith('erp.'))

    return hasErpAccess ? "/dashboard" : "/my-projects"
  }

  return (
    <>
      <Routes>
        {/* Public Auth Routes */}
        <Route element={<AuthLayout />}>
          <Route path="/login" element={<LoginPage />} />
        </Route>

        {/* Public Password Routes */}
        <Route path="/forgot-password" element={<ForgotPasswordPage />} />
        <Route path="/reset-password" element={<ResetPasswordPage />} />

        {/* Force Change Password Route */}
        <Route
          path="/force-change-password"
          element={
            <ForcePasswordRoute>
              <ForceChangePasswordPage />
            </ForcePasswordRoute>
          }
        />

        {/* Protected Routes */}
        <Route
          element={
            <ProtectedRoute>
              <MainLayout />
            </ProtectedRoute>
          }
        >
          <Route path="/" element={<Navigate to={getHomeRedirect()} replace />} />

          {/* ERP 模組路由 */}
          <Route element={<ErpRoute />}>
            <Route path="/dashboard" element={<DashboardPage />} />

            {/* 基礎資料 */}
            <Route path="/products" element={<ProductsPage />} />
            <Route path="/products/new" element={<CreateProductPage />} />
            <Route path="/products/:id" element={<ProductDetailPage />} />
            <Route path="/products/:id/edit" element={<CreateProductPage />} />
            <Route path="/warehouses" element={<WarehousesPage />} />
            <Route path="/partners" element={<PartnersPage />} />

            {/* 單據管理 */}
            <Route path="/documents" element={<DocumentsPage />} />
            <Route path="/documents/new" element={<DocumentEditPage />} />
            <Route path="/documents/:id" element={<DocumentDetailPage />} />
            <Route path="/documents/:id/edit" element={<DocumentEditPage />} />

            {/* 庫存管理 */}
            <Route path="/inventory" element={<InventoryPage />} />
            <Route path="/inventory/ledger" element={<StockLedgerPage />} />

            {/* 報表中心 */}
            <Route path="/reports" element={<ReportsPage />} />
            <Route path="/reports/stock-on-hand" element={<StockOnHandReportPage />} />
            <Route path="/reports/stock-ledger" element={<StockLedgerReportPage />} />
            <Route path="/reports/purchase-lines" element={<PurchaseLinesReportPage />} />
            <Route path="/reports/sales-lines" element={<SalesLinesReportPage />} />
            <Route path="/reports/cost-summary" element={<CostSummaryReportPage />} />
          </Route>

          {/* 系統管理 */}
          <Route path="/admin/users" element={<UsersPage />} />
          <Route path="/admin/roles" element={<RolesPage />} />
          <Route path="/admin/settings" element={<SettingsPage />} />
          <Route path="/admin/audit-logs" element={<AuditLogsPage />} />
          <Route path="/admin/audit" element={<AdminAuditPage />} />

          {/* HR 人員管理 */}
          <Route path="/hr/attendance" element={<HrAttendancePage />} />
          <Route path="/hr/leaves" element={<HrLeavePage />} />
          <Route path="/hr/overtime" element={<HrOvertimePage />} />
          <Route path="/hr/calendar" element={<CalendarSyncSettingsPage />} />

          {/* AUP 計畫書管理 */}
          <Route path="/protocols" element={<ProtocolsPage />} />
          <Route path="/protocols/new" element={<ProtocolEditPage />} />
          <Route path="/protocols/:id" element={<ProtocolDetailPage />} />
          <Route path="/protocols/:id/edit" element={<ProtocolEditPage />} />

          {/* 我的計劃 */}
          <Route path="/my-projects" element={<MyProjectsPage />} />
          <Route path="/my-projects/:id" element={<MyProjectDetailPage />} />

          {/* 實驗動物管理 */}
          <Route path="/pigs" element={<PigsPage />} />
          <Route path="/pigs/:id" element={<PigDetailPage />} />
          <Route path="/pigs/:id/edit" element={<PigEditPage />} />
          <Route path="/pig-sources" element={<PigSourcesPage />} />
        </Route>

        {/* Catch all */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
      <Toaster />
    </>
  )
}

export default App
