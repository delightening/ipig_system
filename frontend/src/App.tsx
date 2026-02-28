import { useEffect, lazy } from 'react'
import { Routes, Route, Navigate, Outlet, useLocation } from 'react-router-dom'
import { Toaster } from '@/components/ui/toaster'
import { PageErrorBoundary } from '@/components/ui/page-error-boundary'
import { useAuthStore } from '@/stores/auth'
import { RequirePermission } from '@/components/auth'
import { useHeartbeat } from '@/hooks/useHeartbeat'
import { SessionTimeoutWarning } from '@/components/SessionTimeoutWarning'
import { CookieConsent } from '@/components/CookieConsent'

// Layouts — 保持靜態 import（每個受保護路由都需要）
import { MainLayout } from '@/layouts/MainLayout'
import { AuthLayout } from '@/layouts/AuthLayout'

// ============================================
// 路由層級 Code-Splitting：所有頁面元件以 React.lazy 動態載入
// ============================================

// Auth Pages
const LoginPage = lazy(() => import('@/pages/auth/LoginPage').then(m => ({ default: m.LoginPage })))
const ForgotPasswordPage = lazy(() => import('@/pages/auth/ForgotPasswordPage').then(m => ({ default: m.ForgotPasswordPage })))
const ResetPasswordPage = lazy(() => import('@/pages/auth/ResetPasswordPage').then(m => ({ default: m.ResetPasswordPage })))
const ForceChangePasswordPage = lazy(() => import('@/pages/auth/ForceChangePasswordPage').then(m => ({ default: m.ForceChangePasswordPage })))

// Dashboard & Profile
const DashboardPage = lazy(() => import('@/pages/DashboardPage').then(m => ({ default: m.DashboardPage })))
const ProfileSettingsPage = lazy(() => import('@/pages/ProfileSettingsPage').then(m => ({ default: m.ProfileSettingsPage })))

// Master Data Pages
const ProductsPage = lazy(() => import('@/pages/master/ProductsPage').then(m => ({ default: m.ProductsPage })))
const CreateProductPage = lazy(() => import('@/pages/master/CreateProductPage').then(m => ({ default: m.CreateProductPage })))
const ProductDetailPage = lazy(() => import('@/pages/master/ProductDetailPage').then(m => ({ default: m.ProductDetailPage })))
const WarehousesPage = lazy(() => import('@/pages/master/WarehousesPage').then(m => ({ default: m.WarehousesPage })))
const PartnersPage = lazy(() => import('@/pages/master/PartnersPage').then(m => ({ default: m.PartnersPage })))
const BloodTestTemplatesPage = lazy(() => import('@/pages/master/BloodTestTemplatesPage').then(m => ({ default: m.BloodTestTemplatesPage })))
const BloodTestPanelsPage = lazy(() => import('@/pages/master/BloodTestPanelsPage').then(m => ({ default: m.BloodTestPanelsPage })))

// Document Pages
const DocumentsPage = lazy(() => import('@/pages/documents/DocumentsPage').then(m => ({ default: m.DocumentsPage })))
const DocumentDetailPage = lazy(() => import('@/pages/documents/DocumentDetailPage').then(m => ({ default: m.DocumentDetailPage })))
const DocumentEditPage = lazy(() => import('@/pages/documents/DocumentEditPage').then(m => ({ default: m.DocumentEditPage })))

// Inventory Pages
const InventoryPage = lazy(() => import('@/pages/inventory/InventoryPage').then(m => ({ default: m.InventoryPage })))
const StockLedgerPage = lazy(() => import('@/pages/inventory/StockLedgerPage').then(m => ({ default: m.StockLedgerPage })))
const WarehouseLayoutPage = lazy(() => import('@/pages/inventory/WarehouseLayoutPage').then(m => ({ default: m.WarehouseLayoutPage })))

// Admin Pages
const UsersPage = lazy(() => import('@/pages/admin/UsersPage').then(m => ({ default: m.UsersPage })))
const RolesPage = lazy(() => import('@/pages/admin/RolesPage').then(m => ({ default: m.RolesPage })))
const SettingsPage = lazy(() => import('@/pages/admin/SettingsPage').then(m => ({ default: m.SettingsPage })))
const AuditLogsPage = lazy(() => import('@/pages/admin/AuditLogsPage').then(m => ({ default: m.AuditLogsPage })))
const AdminAuditPage = lazy(() => import('@/pages/admin/AdminAuditPage').then(m => ({ default: m.AdminAuditPage })))
const NotificationRoutingPage = lazy(() => import('@/pages/admin/NotificationRoutingPage').then(m => ({ default: m.NotificationRoutingPage })))
const TreatmentDrugOptionsPage = lazy(() => import('@/pages/admin/TreatmentDrugOptionsPage').then(m => ({ default: m.TreatmentDrugOptionsPage })))

// HR Pages
const HrAttendancePage = lazy(() => import('@/pages/hr/HrAttendancePage').then(m => ({ default: m.HrAttendancePage })))
const HrLeavePage = lazy(() => import('@/pages/hr/HrLeavePage').then(m => ({ default: m.HrLeavePage })))
const HrOvertimePage = lazy(() => import('@/pages/hr/HrOvertimePage').then(m => ({ default: m.HrOvertimePage })))
const HrAnnualLeavePage = lazy(() => import('@/pages/hr/HrAnnualLeavePage').then(m => ({ default: m.HrAnnualLeavePage })))
const CalendarSyncSettingsPage = lazy(() => import('@/pages/hr/CalendarSyncSettingsPage').then(m => ({ default: m.CalendarSyncSettingsPage })))

// Report Pages
const ReportsPage = lazy(() => import('@/pages/reports/ReportsPage').then(m => ({ default: m.ReportsPage })))
const StockOnHandReportPage = lazy(() => import('@/pages/reports/StockOnHandReportPage').then(m => ({ default: m.StockOnHandReportPage })))
const StockLedgerReportPage = lazy(() => import('@/pages/reports/StockLedgerReportPage').then(m => ({ default: m.StockLedgerReportPage })))
const PurchaseLinesReportPage = lazy(() => import('@/pages/reports/PurchaseLinesReportPage').then(m => ({ default: m.PurchaseLinesReportPage })))
const SalesLinesReportPage = lazy(() => import('@/pages/reports/SalesLinesReportPage').then(m => ({ default: m.SalesLinesReportPage })))
const CostSummaryReportPage = lazy(() => import('@/pages/reports/CostSummaryReportPage').then(m => ({ default: m.CostSummaryReportPage })))
const BloodTestCostReportPage = lazy(() => import('@/pages/reports/BloodTestCostReportPage').then(m => ({ default: m.BloodTestCostReportPage })))
const BloodTestAnalysisPage = lazy(() => import('@/pages/reports/BloodTestAnalysisPage').then(m => ({ default: m.BloodTestAnalysisPage })))

// ERP Page
const ErpPage = lazy(() => import('@/pages/erp/ErpPage').then(m => ({ default: m.ErpPage })))

// AUP Protocol Pages
const ProtocolsPage = lazy(() => import('@/pages/protocols/ProtocolsPage').then(m => ({ default: m.ProtocolsPage })))
const ProtocolDetailPage = lazy(() => import('@/pages/protocols/ProtocolDetailPage').then(m => ({ default: m.ProtocolDetailPage })))
const ProtocolEditPage = lazy(() => import('@/pages/protocols/ProtocolEditPage').then(m => ({ default: m.ProtocolEditPage })))

// My Projects & Amendments
const MyProjectsPage = lazy(() => import('@/pages/my-projects/MyProjectsPage').then(m => ({ default: m.MyProjectsPage })))
const MyAmendmentsPage = lazy(() => import('@/pages/amendments/MyAmendmentsPage').then(m => ({ default: m.MyAmendmentsPage })))

// Animal Management Pages
const AnimalsPage = lazy(() => import('@/pages/animals/AnimalsPage').then(m => ({ default: m.AnimalsPage })))
const AnimalDetailPage = lazy(() => import('@/pages/animals/AnimalDetailPage').then(m => ({ default: m.AnimalDetailPage })))
const AnimalEditPage = lazy(() => import('@/pages/animals/AnimalEditPage').then(m => ({ default: m.AnimalEditPage })))
const AnimalSourcesPage = lazy(() => import('@/pages/animals/AnimalSourcesPage').then(m => ({ default: m.AnimalSourcesPage })))

// Public Pages
const PrivacyPolicyPage = lazy(() => import('@/pages/PrivacyPolicyPage').then(m => ({ default: m.PrivacyPolicyPage })))
const TermsOfServicePage = lazy(() => import('@/pages/TermsOfServicePage').then(m => ({ default: m.TermsOfServicePage })))

// 404
const NotFoundPage = lazy(() => import('@/pages/NotFoundPage').then(m => ({ default: m.NotFoundPage })))

// Protected Route component
function ProtectedRoute({ children }: { children: React.ReactNode }) {
    const { isAuthenticated, isInitialized, user } = useAuthStore()

    // 啟動 heartbeat 監聽使用者活動
    useHeartbeat(isAuthenticated)

    // SEC-24: 等待初始驗證完成，防止 stale localStorage state
    if (!isInitialized) {
        return (
            <div className="flex items-center justify-center min-h-screen">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
            </div>
        )
    }

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

// Dashboard Route - 僅限具備權限或是審查/主席/獸醫角色的人員
function DashboardRoute({ children }: { children?: React.ReactNode }) {
    const { user, hasRole } = useAuthStore()

    const hasDashboardAccess = hasRole('admin') ||
        user?.roles.some(r => ['purchasing', 'approver', 'WAREHOUSE_MANAGER', 'EXPERIMENT_STAFF', 'REVIEWER', 'VET', 'IACUC_CHAIR'].includes(r)) ||
        user?.permissions.some(p => p.startsWith('erp.'))

    if (!hasDashboardAccess) {
        return <Navigate to="/my-projects" replace />
    }

    return children ? <>{children}</> : <Outlet />
}

// Admin Route - 僅限管理員
function AdminRoute({ children }: { children?: React.ReactNode }) {
    const { hasRole } = useAuthStore()

    if (!hasRole('admin')) {
        return (
            <RequirePermission role="admin">
                {children || <Outlet />}
            </RequirePermission>
        )
    }

    return children ? <>{children}</> : <Outlet />
}

function App() {
    const { checkAuth, isAuthenticated, user, hasRole } = useAuthStore()
    const location = useLocation()

    // 公開路由不需要檢查認證狀態
    const publicPaths = ['/login', '/forgot-password', '/reset-password', '/privacy', '/terms']
    const isPublicRoute = publicPaths.some(path => location.pathname.startsWith(path))

    // Validate auth on app initialization (Cookie 自動傳送，不需檢查 localStorage)
    useEffect(() => {
        if (isPublicRoute) {
            // 公開頁面不呼叫 /api/me，直接標記為已初始化
            useAuthStore.setState({ isInitialized: true })
            return
        }
        checkAuth().catch(() => {
            // Token validation failed, will be handled by checkAuth
        })
    }, [checkAuth, isPublicRoute])

    // ============================================
    // 閒置時預載：主頁面渲染完成後，背景預載所有路由 chunk
    // ============================================
    useEffect(() => {
        if (!isAuthenticated) return

        const prefetchBatch = (modules: Array<() => Promise<unknown>>) => {
            modules.forEach(load => {
                // 靜默預載，忽略錯誤（例如網路波動）
                load().catch(() => { })
            })
        }

        const scheduleIdle = (fn: () => void) => {
            if ('requestIdleCallback' in window) {
                requestIdleCallback(fn)
            } else {
                setTimeout(fn, 2000)
            }
        }

        // 分批預載，優先級由高至低
        scheduleIdle(() => {
            // 第一批：高頻頁面
            prefetchBatch([
                () => import('@/pages/DashboardPage'),
                () => import('@/pages/my-projects/MyProjectsPage'),
                () => import('@/pages/animals/AnimalsPage'),
                () => import('@/pages/protocols/ProtocolsPage'),
                () => import('@/pages/animals/AnimalDetailPage'),
                () => import('@/pages/protocols/ProtocolDetailPage'),
            ])

            scheduleIdle(() => {
                // 第二批：次要頁面
                prefetchBatch([
                    () => import('@/pages/erp/ErpPage'),
                    () => import('@/pages/protocols/ProtocolEditPage'),
                    () => import('@/pages/animals/AnimalEditPage'),
                    () => import('@/pages/hr/HrAttendancePage'),
                    () => import('@/pages/hr/HrLeavePage'),
                    () => import('@/pages/hr/HrOvertimePage'),
                    () => import('@/pages/ProfileSettingsPage'),
                    () => import('@/pages/documents/DocumentsPage'),
                    () => import('@/pages/inventory/InventoryPage'),
                ])

                scheduleIdle(() => {
                    // 第三批：低頻管理/報表頁面
                    prefetchBatch([
                        () => import('@/pages/admin/UsersPage'),
                        () => import('@/pages/admin/RolesPage'),
                        () => import('@/pages/admin/SettingsPage'),
                        () => import('@/pages/admin/AuditLogsPage'),
                        () => import('@/pages/admin/AdminAuditPage'),
                        () => import('@/pages/admin/NotificationRoutingPage'),
                        () => import('@/pages/admin/TreatmentDrugOptionsPage'),
                        () => import('@/pages/reports/ReportsPage'),
                        () => import('@/pages/reports/StockOnHandReportPage'),
                        () => import('@/pages/reports/StockLedgerReportPage'),
                        () => import('@/pages/reports/PurchaseLinesReportPage'),
                        () => import('@/pages/reports/SalesLinesReportPage'),
                        () => import('@/pages/reports/CostSummaryReportPage'),
                        () => import('@/pages/reports/BloodTestCostReportPage'),
                        () => import('@/pages/reports/BloodTestAnalysisPage'),
                        () => import('@/pages/master/ProductsPage'),
                        () => import('@/pages/master/WarehousesPage'),
                        () => import('@/pages/master/PartnersPage'),
                        () => import('@/pages/master/BloodTestTemplatesPage'),
                        () => import('@/pages/master/BloodTestPanelsPage'),
                        () => import('@/pages/documents/DocumentDetailPage'),
                        () => import('@/pages/documents/DocumentEditPage'),
                        () => import('@/pages/inventory/StockLedgerPage'),
                        () => import('@/pages/inventory/WarehouseLayoutPage'),
                        () => import('@/pages/hr/HrAnnualLeavePage'),
                        () => import('@/pages/hr/CalendarSyncSettingsPage'),
                        () => import('@/pages/amendments/MyAmendmentsPage'),
                        () => import('@/pages/animals/AnimalSourcesPage'),
                        () => import('@/pages/master/CreateProductPage'),
                        () => import('@/pages/master/ProductDetailPage'),
                    ])
                })
            })
        })
    }, [isAuthenticated])

    // 判斷首頁導向
    const getHomeRedirect = () => {
        const hasDashboardAccess = hasRole('admin') ||
            user?.roles.some(r => ['warehouse', 'purchasing', 'sales', 'approver', 'EXPERIMENT_STAFF', 'REVIEWER', 'VET', 'IACUC_CHAIR'].includes(r)) ||
            user?.permissions.some(p => p.startsWith('erp.'))

        return hasDashboardAccess ? "/dashboard" : "/my-projects"
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

                {/* Public Static Pages */}
                <Route path="/privacy" element={<PrivacyPolicyPage />} />
                <Route path="/terms" element={<TermsOfServicePage />} />

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

                    {/* Dashboard 與 ERP 模組路由 */}
                    <Route element={<DashboardRoute />}>
                        <Route path="/dashboard" element={<PageErrorBoundary><DashboardPage /></PageErrorBoundary>} />
                        <Route path="/erp" element={<ErpPage />} />

                        <Route path="/products" element={<ProductsPage />} />
                        <Route path="/products/new" element={<CreateProductPage />} />
                        <Route path="/products/:id" element={<ProductDetailPage />} />
                        <Route path="/products/:id/edit" element={<CreateProductPage />} />
                        <Route path="/warehouses" element={<WarehousesPage />} />
                        <Route path="/partners" element={<PartnersPage />} />
                        <Route path="/blood-test-templates" element={<BloodTestTemplatesPage />} />
                        <Route path="/blood-test-panels" element={<BloodTestPanelsPage />} />

                        {/* 單據管理 */}
                        <Route path="/documents" element={<DocumentsPage />} />
                        <Route path="/documents/new" element={<DocumentEditPage />} />
                        <Route path="/documents/:id" element={<DocumentDetailPage />} />
                        <Route path="/documents/:id/edit" element={<DocumentEditPage />} />

                        {/* 庫存管理 */}
                        <Route path="/inventory" element={<InventoryPage />} />
                        <Route path="/inventory/ledger" element={<StockLedgerPage />} />
                        <Route path="/inventory/layout" element={<WarehouseLayoutPage />} />

                        {/* 報表中心 */}
                        <Route path="/reports" element={<ReportsPage />} />
                        <Route path="/reports/stock-on-hand" element={<StockOnHandReportPage />} />
                        <Route path="/reports/stock-ledger" element={<StockLedgerReportPage />} />
                        <Route path="/reports/purchase-lines" element={<PurchaseLinesReportPage />} />
                        <Route path="/reports/sales-lines" element={<SalesLinesReportPage />} />
                        <Route path="/reports/cost-summary" element={<CostSummaryReportPage />} />
                        <Route path="/reports/blood-test-cost" element={<BloodTestCostReportPage />} />
                        <Route path="/reports/blood-test-analysis" element={<BloodTestAnalysisPage />} />
                    </Route>

                    {/* 系統管理 - 需要 admin 角色 */}
                    <Route element={<AdminRoute />}>
                        <Route path="/admin/users" element={<UsersPage />} />
                        <Route path="/admin/roles" element={<RolesPage />} />
                        <Route path="/admin/settings" element={<SettingsPage />} />
                        <Route path="/admin/audit-logs" element={<AuditLogsPage />} />
                        <Route path="/admin/audit" element={<AdminAuditPage />} />
                        <Route path="/admin/notification-routing" element={<NotificationRoutingPage />} />
                        <Route path="/admin/treatment-drugs" element={<TreatmentDrugOptionsPage />} />
                    </Route>

                    {/* HR 人員管理 */}
                    <Route path="/hr/attendance" element={<HrAttendancePage />} />
                    <Route path="/hr/leaves" element={<HrLeavePage />} />
                    <Route path="/hr/overtime" element={<HrOvertimePage />} />
                    <Route path="/hr/annual-leave" element={
                        <RequirePermission anyOf={[
                            { permission: 'hr.balance.manage' },
                            { role: 'admin' }
                        ]}>
                            <HrAnnualLeavePage />
                        </RequirePermission>
                    } />
                    <Route path="/hr/calendar" element={<CalendarSyncSettingsPage />} />

                    {/* AUP 計畫書管理 */}
                    <Route path="/protocols" element={<ProtocolsPage />} />
                    <Route path="/protocols/new" element={<PageErrorBoundary><ProtocolEditPage /></PageErrorBoundary>} />
                    <Route path="/protocols/:id" element={<ProtocolDetailPage />} />
                    <Route path="/protocols/:id/edit" element={<PageErrorBoundary><ProtocolEditPage /></PageErrorBoundary>} />

                    {/* 我的計劃 */}
                    <Route path="/my-projects" element={<MyProjectsPage />} />
                    <Route path="/my-projects/:id" element={<ProtocolDetailPage />} />

                    {/* 我的變更申請 */}
                    <Route path="/my-amendments" element={<MyAmendmentsPage />} />

                    {/* 實驗動物管理 */}
                    <Route path="/animals" element={<AnimalsPage />} />
                    <Route path="/animals/:id" element={<PageErrorBoundary><AnimalDetailPage /></PageErrorBoundary>} />
                    <Route path="/animals/:id/edit" element={<AnimalEditPage />} />
                    <Route path="/animal-sources" element={<AnimalSourcesPage />} />

                    {/* 個人設定 */}
                    <Route path="/profile/settings" element={<ProfileSettingsPage />} />
                </Route>

                {/* 404 */}
                <Route path="*" element={<NotFoundPage />} />
            </Routes>
            <Toaster />
            <SessionTimeoutWarning />
            <CookieConsent />
        </>
    )
}

export default App
