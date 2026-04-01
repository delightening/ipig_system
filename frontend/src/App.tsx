import { useEffect, lazy } from 'react'
import { Routes, Route, Navigate, useLocation } from 'react-router-dom'
import { Toaster } from '@/components/ui/toaster'
import { useAuthStore } from '@/stores/auth'
import { RequirePermission, ProtectedRoute, ForcePasswordRoute, DashboardRoute, AdminRoute, DASHBOARD_ROLES } from '@/components/auth'
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
const InvitationAcceptPage = lazy(() => import('@/pages/auth/InvitationAcceptPage').then(m => ({ default: m.InvitationAcceptPage })))

// Dashboard & Profile
const DashboardPage = lazy(() => import('@/pages/DashboardPage').then(m => ({ default: m.DashboardPage })))
const ProfileSettingsPage = lazy(() => import('@/pages/ProfileSettingsPage').then(m => ({ default: m.ProfileSettingsPage })))

// Master Data Pages
const ProductsPage = lazy(() => import('@/pages/master/ProductsPage').then(m => ({ default: m.ProductsPage })))
const CreateProductPage = lazy(() => import('@/pages/master/CreateProductPage').then(m => ({ default: m.CreateProductPage })))
const ProductDetailPage = lazy(() => import('@/pages/master/ProductDetailPage').then(m => ({ default: m.ProductDetailPage })))
const ProductEditPage = lazy(() => import('@/pages/master/ProductEditPage').then(m => ({ default: m.ProductEditPage })))
const PartnersPage = lazy(() => import('@/pages/master/PartnersPage').then(m => ({ default: m.PartnersPage })))
const BloodTestTemplatesPage = lazy(() => import('@/pages/master/BloodTestTemplatesPage').then(m => ({ default: m.BloodTestTemplatesPage })))
const BloodTestPanelsPage = lazy(() => import('@/pages/master/BloodTestPanelsPage').then(m => ({ default: m.BloodTestPanelsPage })))
const BloodTestPresetsPage = lazy(() => import('@/pages/master/BloodTestPresetsPage').then(m => ({ default: m.BloodTestPresetsPage })))

// Document Pages
const DocumentsPage = lazy(() => import('@/pages/documents/DocumentsPage').then(m => ({ default: m.DocumentsPage })))
const DocumentDetailPage = lazy(() => import('@/pages/documents/DocumentDetailPage').then(m => ({ default: m.DocumentDetailPage })))
const DocumentEditPage = lazy(() => import('@/pages/documents/DocumentEditPage').then(m => ({ default: m.DocumentEditPage })))

// Inventory Pages
const InventoryPage = lazy(() => import('@/pages/inventory/InventoryPage').then(m => ({ default: m.InventoryPage })))
const StockLedgerPage = lazy(() => import('@/pages/inventory/StockLedgerPage').then(m => ({ default: m.StockLedgerPage })))
const WarehouseLayoutPage = lazy(() => import('@/pages/inventory/WarehouseLayoutPage').then(m => ({ default: m.WarehouseLayoutPage })))
const WarehouseReportPage = lazy(() => import('@/pages/inventory/WarehouseReportPage').then(m => ({ default: m.WarehouseReportPage })))

// Admin Pages
const UsersPage = lazy(() => import('@/pages/admin/UsersPage').then(m => ({ default: m.UsersPage })))
const RolesPage = lazy(() => import('@/pages/admin/RolesPage').then(m => ({ default: m.RolesPage })))
const SettingsPage = lazy(() => import('@/pages/admin/SettingsPage').then(m => ({ default: m.SettingsPage })))
const AuditLogsPage = lazy(() => import('@/pages/admin/AuditLogsPage').then(m => ({ default: m.AuditLogsPage })))
const AnimalFieldCorrectionsPage = lazy(() => import('@/pages/admin/AnimalFieldCorrectionsPage').then(m => ({ default: m.AnimalFieldCorrectionsPage })))
const AdminAuditPage = lazy(() => import('@/pages/admin/AdminAuditPage').then(m => ({ default: m.AdminAuditPage })))
const NotificationRoutingPage = lazy(() => import('@/pages/admin/NotificationRoutingPage').then(m => ({ default: m.NotificationRoutingPage })))
const TreatmentDrugOptionsPage = lazy(() => import('@/pages/admin/TreatmentDrugOptionsPage').then(m => ({ default: m.TreatmentDrugOptionsPage })))
const TrainingRecordsPage = lazy(() => import('@/pages/admin/TrainingRecordsPage').then(m => ({ default: m.TrainingRecordsPage })))
const QAUDashboardPage = lazy(() => import('@/pages/admin/QAUDashboardPage').then(m => ({ default: m.QAUDashboardPage })))
const QAInspectionPage = lazy(() => import('@/pages/admin/QAInspectionPage').then(m => ({ default: m.QAInspectionPage })))
const QANonConformancePage = lazy(() => import('@/pages/admin/QANonConformancePage').then(m => ({ default: m.QANonConformancePage })))
const QASopPage = lazy(() => import('@/pages/admin/QASopPage').then(m => ({ default: m.QASopPage })))
const QASchedulePage = lazy(() => import('@/pages/admin/QASchedulePage').then(m => ({ default: m.QASchedulePage })))
const FacilitiesPage = lazy(() => import('@/pages/admin/FacilitiesPage').then(m => ({ default: m.FacilitiesPage })))
const InvitationsPage = lazy(() => import('@/pages/admin/InvitationsPage').then(m => ({ default: m.InvitationsPage })))

// HR Pages
const HrAttendancePage = lazy(() => import('@/pages/hr/HrAttendancePage').then(m => ({ default: m.HrAttendancePage })))
const HrLeavePage = lazy(() => import('@/pages/hr/HrLeavePage').then(m => ({ default: m.HrLeavePage })))
const HrOvertimePage = lazy(() => import('@/pages/hr/HrOvertimePage').then(m => ({ default: m.HrOvertimePage })))
const HrAnnualLeavePage = lazy(() => import('@/pages/hr/HrAnnualLeavePage').then(m => ({ default: m.HrAnnualLeavePage })))
const CalendarSyncSettingsPage = lazy(() => import('@/pages/hr/CalendarSyncSettingsPage').then(m => ({ default: m.CalendarSyncSettingsPage })))

// Report Pages
const StockOnHandReportPage = lazy(() => import('@/pages/reports/StockOnHandReportPage').then(m => ({ default: m.StockOnHandReportPage })))
const StockLedgerReportPage = lazy(() => import('@/pages/reports/StockLedgerReportPage').then(m => ({ default: m.StockLedgerReportPage })))
const PurchaseLinesReportPage = lazy(() => import('@/pages/reports/PurchaseLinesReportPage').then(m => ({ default: m.PurchaseLinesReportPage })))
const SalesLinesReportPage = lazy(() => import('@/pages/reports/SalesLinesReportPage').then(m => ({ default: m.SalesLinesReportPage })))
const CostSummaryReportPage = lazy(() => import('@/pages/reports/CostSummaryReportPage').then(m => ({ default: m.CostSummaryReportPage })))
const BloodTestCostReportPage = lazy(() => import('@/pages/reports/BloodTestCostReportPage').then(m => ({ default: m.BloodTestCostReportPage })))
const BloodTestAnalysisPage = lazy(() => import('@/pages/reports/BloodTestAnalysisPage').then(m => ({ default: m.BloodTestAnalysisPage })))
const AccountingReportPage = lazy(() => import('@/pages/reports/AccountingReportPage').then(m => ({ default: m.AccountingReportPage })))
const PurchaseSalesSummaryPage = lazy(() => import('@/pages/reports/PurchaseSalesSummaryPage').then(m => ({ default: m.PurchaseSalesSummaryPage })))

// ERP Pages
const ErpReportsPage = lazy(() => import('@/pages/erp/ErpReportsPage').then(m => ({ default: m.ErpReportsPage })))
const EquipmentPage = lazy(() => import('@/pages/admin/EquipmentPage').then(m => ({ default: m.EquipmentPage })))
const EquipmentHistoryPage = lazy(() => import('@/pages/admin/EquipmentHistoryPage').then(m => ({ default: m.EquipmentHistoryPage })))

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

function App() {
    const { checkAuth, isAuthenticated, user, hasRole } = useAuthStore()
    const location = useLocation()

    // 公開路由不需要檢查認證狀態
    const publicPaths = ['/login', '/forgot-password', '/reset-password', '/privacy', '/terms', '/invite']
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
                load().catch((err) => {
                    if (import.meta.env.DEV) {
                        console.warn('[Prefetch] Route chunk 預載失敗:', err)
                    }
                })
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
                    () => import('@/pages/erp/ErpReportsPage'),
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
                        () => import('@/pages/reports/StockOnHandReportPage'),
                        () => import('@/pages/reports/StockLedgerReportPage'),
                        () => import('@/pages/reports/PurchaseLinesReportPage'),
                        () => import('@/pages/reports/SalesLinesReportPage'),
                        () => import('@/pages/reports/CostSummaryReportPage'),
                        () => import('@/pages/reports/BloodTestCostReportPage'),
                        () => import('@/pages/reports/BloodTestAnalysisPage'),
                        () => import('@/pages/reports/AccountingReportPage'),
                        () => import('@/pages/master/ProductsPage'),
                        () => import('@/pages/master/PartnersPage'),
                        () => import('@/pages/master/BloodTestTemplatesPage'),
                        () => import('@/pages/master/BloodTestPanelsPage'),
                        () => import('@/pages/master/BloodTestPresetsPage'),
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
            user?.roles.some(r => DASHBOARD_ROLES.includes(r)) ||
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

                {/* 客戶邀請註冊（公開路由） */}
                <Route path="/invite/:token" element={<InvitationAcceptPage />} />

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
                        <Route path="/dashboard" element={<DashboardPage />} />
                        <Route path="/erp" element={<Navigate to="/products" replace />} />
                        <Route path="/erp/reports" element={<AdminRoute><ErpReportsPage /></AdminRoute>} />
                        <Route path="/equipment" element={<RequirePermission permission="equipment.view"><EquipmentPage /></RequirePermission>} />
                        <Route path="/equipment/:id/history" element={<RequirePermission permission="equipment.view"><EquipmentHistoryPage /></RequirePermission>} />

                        <Route path="/products" element={<ProductsPage />} />
                        <Route path="/products/new" element={<CreateProductPage />} />
                        <Route path="/products/:id" element={<ProductDetailPage />} />
                        <Route path="/products/:id/edit" element={<ProductEditPage />} />
                        <Route path="/warehouses" element={<WarehouseLayoutPage />} />
                        <Route path="/partners" element={<PartnersPage />} />
                        <Route path="/blood-test-templates" element={<RequirePermission permission="animal.blood_test_template.manage" fallback="redirect"><BloodTestTemplatesPage /></RequirePermission>} />
                        <Route path="/blood-test-panels" element={<RequirePermission permission="animal.blood_test_template.manage" fallback="redirect"><BloodTestPanelsPage /></RequirePermission>} />
                        <Route path="/blood-test-presets" element={<RequirePermission permission="animal.blood_test_template.manage" fallback="redirect"><BloodTestPresetsPage /></RequirePermission>} />

                        {/* 單據管理 */}
                        <Route path="/documents" element={<DocumentsPage />} />
                        <Route path="/documents/new" element={<DocumentEditPage />} />
                        <Route path="/documents/:id" element={<DocumentDetailPage />} />
                        <Route path="/documents/:id/edit" element={<DocumentEditPage />} />

                        {/* 庫存管理 */}
                        <Route path="/inventory" element={<InventoryPage />} />
                        <Route path="/inventory/ledger" element={<StockLedgerPage />} />
                        <Route path="/inventory/layout" element={<WarehouseLayoutPage />} />
                        <Route path="/inventory/warehouse-report/:warehouseId" element={<WarehouseReportPage />} />

                        {/* 報表中心 */}
                        <Route path="/stock-on-hand" element={<StockOnHandReportPage />} />
                        <Route path="/stock-ledger" element={<StockLedgerReportPage />} />
                        <Route path="/purchase-lines" element={<PurchaseLinesReportPage />} />
                        <Route path="/sales-lines" element={<SalesLinesReportPage />} />
                        <Route path="/cost-summary" element={<CostSummaryReportPage />} />
                        <Route path="/blood-test-cost" element={<BloodTestCostReportPage />} />
                        <Route path="/blood-test-analysis" element={<BloodTestAnalysisPage />} />
                        <Route path="/accounting" element={<AccountingReportPage />} />
                        <Route path="/purchase-sales-summary" element={<PurchaseSalesSummaryPage />} />
                    </Route>

                    {/* 系統管理 - 需要 admin 角色 */}
                    <Route element={<AdminRoute />}>
                        <Route path="/admin/users" element={<UsersPage />} />
                        <Route path="/admin/roles" element={<RolesPage />} />
                        <Route path="/admin/settings" element={<SettingsPage />} />
                        <Route path="/admin/audit-logs" element={<AuditLogsPage />} />
                        <Route path="/admin/audit" element={<AdminAuditPage />} />
                        <Route path="/admin/qau" element={
                          <RequirePermission permission="qau.dashboard.view">
                            <QAUDashboardPage />
                          </RequirePermission>
                        } />
                        <Route path="/admin/qau/inspections" element={
                          <RequirePermission permission="qau.inspection.view">
                            <QAInspectionPage />
                          </RequirePermission>
                        } />
                        <Route path="/admin/qau/non-conformances" element={
                          <RequirePermission permission="qau.nc.view">
                            <QANonConformancePage />
                          </RequirePermission>
                        } />
                        <Route path="/admin/qau/sop" element={
                          <RequirePermission permission="qau.sop.view">
                            <QASopPage />
                          </RequirePermission>
                        } />
                        <Route path="/admin/qau/schedules" element={
                          <RequirePermission permission="qau.schedule.view">
                            <QASchedulePage />
                          </RequirePermission>
                        } />
                        <Route path="/admin/notification-routing" element={<NotificationRoutingPage />} />
                        <Route path="/admin/treatment-drugs" element={<TreatmentDrugOptionsPage />} />
                        <Route path="/admin/facilities" element={<FacilitiesPage />} />
                        <Route path="/admin/invitations" element={
                          <RequirePermission permission="invitation.view">
                            <InvitationsPage />
                          </RequirePermission>
                        } />
                    </Route>

                    {/* 人員訓練 - admin 或 training.view/manage/manage_own 可存取 */}
                    <Route path="/hr/training-records" element={
                        <RequirePermission anyOf={[
                            { role: 'admin' },
                            { permission: 'training.view' },
                            { permission: 'training.manage' },
                            { permission: 'training.manage_own' }
                        ]}>
                            <TrainingRecordsPage />
                        </RequirePermission>
                    } />
                    {/* 設備維護舊路徑導向 */}
                    <Route path="/admin/equipment" element={<Navigate to="/equipment" replace />} />
                    {/* 修正審核已移至實驗動物管理 */}
                    <Route path="/admin/animal-field-corrections" element={<Navigate to="/animals/animal-field-corrections" replace />} />

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
                    <Route path="/protocols/new" element={<ProtocolEditPage />} />
                    <Route path="/protocols/:id" element={<ProtocolDetailPage />} />
                    <Route path="/protocols/:id/edit" element={<ProtocolEditPage />} />

                    {/* 我的計劃 */}
                    <Route path="/my-projects" element={<MyProjectsPage />} />
                    <Route path="/my-projects/:id" element={<ProtocolDetailPage />} />

                    {/* 我的變更申請 */}
                    <Route path="/my-amendments" element={<MyAmendmentsPage />} />

                    {/* 實驗動物管理 */}
                    <Route path="/animals" element={<AnimalsPage />} />
                    <Route path="/animals/:id" element={<AnimalDetailPage />} />
                    <Route path="/animals/:id/edit" element={<AnimalEditPage />} />
                    <Route path="/animal-sources" element={<AnimalSourcesPage />} />
                    <Route path="/animals/animal-field-corrections" element={
                        <RequirePermission role="admin">
                            <AnimalFieldCorrectionsPage />
                        </RequirePermission>
                    } />

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
