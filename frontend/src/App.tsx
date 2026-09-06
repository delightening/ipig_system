// R51 watcher self-deploy 驗證 commit（PR #404 後第一次測試 deploy-prod EAP=Continue
// 是否能成功跑完 docker build/up，無 NativeCommandError 拋出）
import { useEffect, lazy, Suspense } from 'react'
import { Routes, Route, Navigate, useLocation } from 'react-router-dom'
import { Toaster } from '@/components/ui/toaster'
import { useAuthStore } from '@/stores/auth'
import { RequirePermission, ProtectedRoute, ForcePasswordRoute, DashboardRoute, AdminRoute, GuestBlock, useHasDashboardAccess } from '@/components/auth'
import { CookieConsent } from '@/components/CookieConsent'
import { useUIPreferences } from '@/stores/uiPreferences'

// Layouts — 保持靜態 import（每個受保護路由都需要）
import { MainLayout } from '@/layouts/MainLayout'
import { AuthLayout } from '@/layouts/AuthLayout'
import { LoadingOverlay } from '@/components/ui/loading-overlay'

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
const LotMovementsPage = lazy(() => import('@/pages/inventory/LotMovementsPage').then(m => ({ default: m.LotMovementsPage })))
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
// GLP Compliance Pages
const DocumentControlPage = lazy(() => import('@/pages/admin/DocumentControlPage').then(m => ({ default: m.DocumentControlPage })))
const ManagementReviewPage = lazy(() => import('@/pages/admin/ManagementReviewPage').then(m => ({ default: m.ManagementReviewPage })))
const RiskRegisterPage = lazy(() => import('@/pages/admin/RiskRegisterPage').then(m => ({ default: m.RiskRegisterPage })))
const ChangeControlPage = lazy(() => import('@/pages/admin/ChangeControlPage').then(m => ({ default: m.ChangeControlPage })))
const EnvironmentMonitoringPage = lazy(() => import('@/pages/admin/EnvironmentMonitoringPage').then(m => ({ default: m.EnvironmentMonitoringPage })))
const CompetencyAssessmentPage = lazy(() => import('@/pages/admin/CompetencyAssessmentPage').then(m => ({ default: m.CompetencyAssessmentPage })))
const StudyFinalReportPage = lazy(() => import('@/pages/admin/StudyFinalReportPage').then(m => ({ default: m.StudyFinalReportPage })))
const FormulationRecordsPage = lazy(() => import('@/pages/admin/FormulationRecordsPage').then(m => ({ default: m.FormulationRecordsPage })))
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
const WeeklyMedicalReportPage = lazy(() => import('@/pages/reports/WeeklyMedicalReportPage').then(m => ({ default: m.WeeklyMedicalReportPage })))
const ByproductMonthlyReportPage = lazy(() => import('@/pages/reports/ByproductMonthlyReportPage').then(m => ({ default: m.ByproductMonthlyReportPage })))
const PurchaseSalesSummaryPage = lazy(() => import('@/pages/reports/PurchaseSalesSummaryPage').then(m => ({ default: m.PurchaseSalesSummaryPage })))

// 報表中心（R35-24：跨子系統 hub，取代舊有 /erp/reports）
const ReportsPage = lazy(() => import('@/pages/reports/ReportsPage').then(m => ({ default: m.ReportsPage })))
const MessagingPage = lazy(() => import('@/pages/messaging/MessagingPage'))
const NotificationsPage = lazy(() => import('@/pages/NotificationsPage'))
const VetPatrolReportListPage = lazy(() => import('@/pages/animals/VetPatrolReportListPage'))
const EquipmentPage = lazy(() => import('@/pages/admin/EquipmentPage').then(m => ({ default: m.EquipmentPage })))
const EquipmentHistoryPage = lazy(() => import('@/pages/admin/EquipmentHistoryPage').then(m => ({ default: m.EquipmentHistoryPage })))

// AUP Protocol Pages
const ProtocolsPage = lazy(() => import('@/pages/protocols/ProtocolsPage').then(m => ({ default: m.ProtocolsPage })))
const ProtocolDetailPage = lazy(() => import('@/pages/protocols/ProtocolDetailPage').then(m => ({ default: m.ProtocolDetailPage })))
const ProtocolEditPage = lazy(() => import('@/pages/protocols/ProtocolEditPage').then(m => ({ default: m.ProtocolEditPage })))
const ImportApprovedProtocolPage = lazy(() => import('@/pages/protocols/ImportApprovedProtocolPage').then(m => ({ default: m.ImportApprovedProtocolPage })))
const ImportReviewPage = lazy(() => import('@/pages/protocols/ImportReviewPage').then(m => ({ default: m.ImportReviewPage })))

// My Projects & Amendments
const MyProjectsPage = lazy(() => import('@/pages/my-projects/MyProjectsPage').then(m => ({ default: m.MyProjectsPage })))
const MyAmendmentsPage = lazy(() => import('@/pages/amendments/MyAmendmentsPage').then(m => ({ default: m.MyAmendmentsPage })))
const AmendmentDetailPage = lazy(() => import('@/pages/amendments/AmendmentDetailPage').then(m => ({ default: m.AmendmentDetailPage })))

// Animal Management Pages
const AnimalsPage = lazy(() => import('@/pages/animals/AnimalsPage').then(m => ({ default: m.AnimalsPage })))
const AvailablePigsPage = lazy(() => import('@/pages/animals/AvailablePigsPage').then(m => ({ default: m.AvailablePigsPage })))
const ReservationPlanningPage = lazy(() => import('@/pages/animals/ReservationPlanningPage').then(m => ({ default: m.ReservationPlanningPage })))
const AnimalDetailPage = lazy(() => import('@/pages/animals/AnimalDetailPage').then(m => ({ default: m.AnimalDetailPage })))
const AnimalEditPage = lazy(() => import('@/pages/animals/AnimalEditPage').then(m => ({ default: m.AnimalEditPage })))
const AnimalSourcesPage = lazy(() => import('@/pages/animals/AnimalSourcesPage').then(m => ({ default: m.AnimalSourcesPage })))

// Public Pages
const PrivacyPolicyPage = lazy(() => import('@/pages/PrivacyPolicyPage').then(m => ({ default: m.PrivacyPolicyPage })))
const TermsOfServicePage = lazy(() => import('@/pages/TermsOfServicePage').then(m => ({ default: m.TermsOfServicePage })))

// R30-27c-2：手機從 QR 開的公開簽名頁
const MobileSignPage = lazy(() => import('@/pages/sign/MobileSignPage').then(m => ({ default: m.MobileSignPage })))

// 404
const NotFoundPage = lazy(() => import('@/pages/NotFoundPage').then(m => ({ default: m.NotFoundPage })))

function App() {
    const { checkAuth, isAuthenticated, isGuest: isGuestFn } = useAuthStore()
    const hasDashboardAccess = useHasDashboardAccess()
    const location = useLocation()
    const { fontSize } = useUIPreferences()

    useEffect(() => {
        const root = document.documentElement
        root.classList.remove('font-size-large', 'font-size-xl')
        if (fontSize === 'large') root.classList.add('font-size-large')
        else if (fontSize === 'xl') root.classList.add('font-size-xl')
    }, [fontSize])

    // 公開路由不需要檢查認證狀態
    const publicPaths = ['/login', '/forgot-password', '/reset-password', '/privacy', '/terms', '/invite', '/sign']
    const isPublicRoute = publicPaths.some(path => location.pathname.startsWith(path))

    // Validate auth on app initialization (Cookie 自動傳送，不需檢查 localStorage)
    useEffect(() => {
        if (isPublicRoute) {
            // 公開頁面不呼叫 /api/me，直接標記為已初始化
            useAuthStore.setState({ isInitialized: true })
            return
        }
        checkAuth()
            .catch((err) => {
                // Token validation failed, will be handled by checkAuth.
                // Dev-only log 留作 R28-10 root cause 調查線索（prod 不噪音）。
                if (import.meta.env.DEV) {
                    console.warn('[App] checkAuth failed', err)
                }
            })
            .finally(() => {
                // Kill-switch：無論 checkAuth 內部 try/catch 是否設過 isInitialized，
                // 這裡保證 spinner 一定會解開（避免 interceptor refresh 流程吞掉 reject
                // 造成 ProtectedRoute 永久卡 loading 的 edge case）。
                //
                // 此處 get→set 非原子，理論上若有多個 checkAuth().finally chain 並行
                // 可能重複 setState；實際單一 useEffect 只觸發一次 chain，worst case
                // 是 React 多收一次 same-value set（zustand 不去重但 React 會 bail out
                // unchanged value re-render），acceptable race。
                if (!useAuthStore.getState().isInitialized) {
                    useAuthStore.setState({ isInitialized: true })
                }
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
                    () => import('@/pages/reports/ReportsPage'),
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
        // 訪客模式：導向 /demo 作為試用首頁
        if (isGuestFn()) return '/demo'

        // 與 DashboardRoute 共用同一個判準：兩處對「這個人看不看得到儀表板」
        // 給出不同答案，正是本系列在收斂的那種漂移。
        return hasDashboardAccess ? "/dashboard" : "/my-projects"
    }



    return (
        <>
            <Suspense fallback={<LoadingOverlay fullScreen message="頁面載入中..." />}>
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

                {/* R30-27c-2：手機從 QR 開的公開簽名頁 */}
                <Route path="/sign/:id" element={<MobileSignPage />} />

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
                        {/* 訪客試用模式入口 URL（登入後導向，與 /dashboard 內容相同） */}
                        <Route path="/demo" element={<DashboardPage />} />
                        <Route path="/erp" element={<Navigate to="/products" replace />} />
                        {/* R35-24：報表中心新路由；舊 /erp/reports 內 ProtectedRoute 已套上，
                          *  redirect 後仍由外層保護 — 用戶若有 /reports 權限會看到 hub，
                          *  個別報表項目自帶 admin/permission gate */}
                        <Route path="/reports" element={<ReportsPage />} />
                        <Route path="/erp/reports" element={<Navigate to="/reports" replace />} />
                        <Route path="/notifications" element={<NotificationsPage />} />
                        <Route path="/messaging" element={<RequirePermission permission="messaging.send" fallback="redirect"><MessagingPage /></RequirePermission>} />
                        <Route path="/vet-patrol-reports" element={<RequirePermission permission="animal.record.view" fallback="redirect"><VetPatrolReportListPage /></RequirePermission>} />
                        <Route path="/equipment" element={<RequirePermission permission="equipment.view"><EquipmentPage /></RequirePermission>} />
                        <Route path="/equipment/:id/history" element={<RequirePermission permission="equipment.view"><EquipmentHistoryPage /></RequirePermission>} />

                        <Route path="/products" element={<ProductsPage />} />
                        <Route path="/products/new" element={<GuestBlock><CreateProductPage /></GuestBlock>} />
                        <Route path="/products/:id" element={<ProductDetailPage />} />
                        <Route path="/products/:id/edit" element={<GuestBlock><ProductEditPage /></GuestBlock>} />
                        <Route path="/warehouses" element={<WarehouseLayoutPage />} />
                        <Route path="/partners" element={<PartnersPage />} />
                        <Route path="/blood-test-templates" element={<RequirePermission permission="animal.blood_test_template.manage" fallback="redirect"><BloodTestTemplatesPage /></RequirePermission>} />
                        <Route path="/blood-test-panels" element={<RequirePermission permission="animal.blood_test_template.manage" fallback="redirect"><BloodTestPanelsPage /></RequirePermission>} />
                        <Route path="/blood-test-presets" element={<RequirePermission permission="animal.blood_test_template.manage" fallback="redirect"><BloodTestPresetsPage /></RequirePermission>} />

                        {/* 單據管理 */}
                        <Route path="/documents" element={<DocumentsPage />} />
                        <Route path="/documents/new" element={<GuestBlock><DocumentEditPage /></GuestBlock>} />
                        <Route path="/documents/:id" element={<DocumentDetailPage />} />
                        <Route path="/documents/:id/edit" element={<GuestBlock><DocumentEditPage /></GuestBlock>} />

                        {/* 庫存管理 */}
                        <Route path="/inventory" element={<InventoryPage />} />
                        <Route path="/inventory/ledger" element={<StockLedgerPage />} />
                        <Route path="/inventory/lot-movements" element={<LotMovementsPage />} />
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
                        <Route path="/weekly-medical-report" element={<WeeklyMedicalReportPage />} />
                        <Route path="/byproduct-monthly-report" element={<RequirePermission permission="animal.byproduct_sample.view" fallback="redirect"><ByproductMonthlyReportPage /></RequirePermission>} />
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
                        <Route path="/admin/notification-routing" element={<NotificationRoutingPage />} />
                        <Route path="/admin/treatment-drugs" element={<TreatmentDrugOptionsPage />} />
                        <Route path="/admin/facilities" element={<FacilitiesPage />} />
                    </Route>

                    {/* QAU／GLP 業務頁面——2026-09-05（P0-2）移出 AdminRoute：
                        這些頁面各自已用 RequirePermission 把關對應模組的權限碼
                        （qau / dms / glp / risk / change / env / competency / study / formulation），
                        ⚠️ 這裡刻意不寫 glob 形式的權限碼：註解裡出現 `*` 接 `/` 會提前關閉 JSX 註解區塊，
                        整段路由當場變成語法錯誤（2026-09-06 CI 實際踩到，tsc TS1003/TS1161）。
                        路徑不變，只是不再額外要求字面 admin 角色——QAU 等角色持有對應權限碼卻進不了頁面正是 P0-2 要修的問題。
                        見 docs/reviews/2026-09-03-code-side-issues.md P0-2。 */}
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
                    {/* GLP Compliance Pages */}
                    <Route path="/admin/document-control" element={
                      <RequirePermission permission="dms.document.view">
                        <DocumentControlPage />
                      </RequirePermission>
                    } />
                    <Route path="/admin/management-reviews" element={
                      <RequirePermission permission="glp.management_review.view">
                        <ManagementReviewPage />
                      </RequirePermission>
                    } />
                    <Route path="/admin/risk-register" element={
                      <RequirePermission permission="risk.register.view">
                        <RiskRegisterPage />
                      </RequirePermission>
                    } />
                    <Route path="/admin/change-control" element={
                      <RequirePermission permission="change.request.view">
                        <ChangeControlPage />
                      </RequirePermission>
                    } />
                    <Route path="/admin/environment-monitoring" element={
                      <RequirePermission permission="env.monitoring.view">
                        <EnvironmentMonitoringPage />
                      </RequirePermission>
                    } />
                    <Route path="/admin/competency-assessments" element={
                      <RequirePermission permission="competency.assessment.view">
                        <CompetencyAssessmentPage />
                      </RequirePermission>
                    } />
                    <Route path="/admin/study-reports" element={
                      <RequirePermission permission="study.report.view">
                        <StudyFinalReportPage />
                      </RequirePermission>
                    } />
                    <Route path="/admin/formulation-records" element={
                      <RequirePermission permission="formulation.record.view">
                        <FormulationRecordsPage />
                      </RequirePermission>
                    } />

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
                    {/* 邀請管理已自 /admin 移至人員管理（僅需 invitation.view，不再受 AdminRoute admin 限制）*/}
                    <Route path="/admin/invitations" element={<Navigate to="/hr/invitations" replace />} />

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
                    <Route path="/hr/invitations" element={
                        <RequirePermission permission="invitation.view">
                            <InvitationsPage />
                        </RequirePermission>
                    } />

                    {/* AUP 計畫書管理 */}
                    <Route path="/protocols" element={<ProtocolsPage />} />
                    {/* R51 follow-up: guest 改為可進 /protocols/new 試填寫（demo data + fieldset disabled），側邊提醒不會儲存 */}
                    <Route path="/protocols/new" element={<ProtocolEditPage />} />
                    <Route path="/protocols/import-approved" element={<GuestBlock><RequirePermission permission="aup.protocol.import_approved"><ImportApprovedProtocolPage /></RequirePermission></GuestBlock>} />
                    <Route path="/protocols/:id/import-review" element={<GuestBlock><RequirePermission permission="aup.protocol.import_approved"><ImportReviewPage /></RequirePermission></GuestBlock>} />
                    {/* 變更申請審查 detail（修復通知 deep-link /protocols/amendments/:id；須在 /protocols/:id 之前註冊） */}
                    <Route path="/protocols/amendments/:id" element={<AmendmentDetailPage />} />
                    <Route path="/protocols/:id" element={<ProtocolDetailPage />} />
                    <Route path="/protocols/:id/edit" element={<GuestBlock><ProtocolEditPage /></GuestBlock>} />

                    {/* 我的計劃 */}
                    <Route path="/my-projects" element={<MyProjectsPage />} />
                    <Route path="/my-projects/:id" element={<ProtocolDetailPage />} />

                    {/* 我的變更申請 */}
                    <Route path="/my-amendments" element={<MyAmendmentsPage />} />

                    {/* 實驗動物管理 */}
                    <Route path="/animals" element={<AnimalsPage />} />
                    <Route path="/animals/available" element={<AvailablePigsPage />} />
                    {/* 檢視走 animal.planning.view（執秘 / 試驗工作人員含 SD / 負責人）；
                        頁內操作另由 <Can permission="animal.planning.manage"> 個別守（僅執秘）。 */}
                    <Route path="/animals/reservation-planning" element={<RequirePermission permission="animal.planning.view" fallback="redirect"><ReservationPlanningPage /></RequirePermission>} />
                    <Route path="/animals/:id" element={<AnimalDetailPage />} />
                    <Route path="/animals/:id/edit" element={<GuestBlock><AnimalEditPage /></GuestBlock>} />
                    <Route path="/animal-sources" element={<AnimalSourcesPage />} />
                    <Route path="/animals/animal-field-corrections" element={
                        <GuestBlock>
                            <RequirePermission role="admin">
                                <AnimalFieldCorrectionsPage />
                            </RequirePermission>
                        </GuestBlock>
                    } />

                    {/* 個人設定 */}
                    <Route path="/profile/settings" element={<ProfileSettingsPage />} />
                </Route>

                {/* 404 */}
                <Route path="*" element={<NotFoundPage />} />
            </Routes>
            </Suspense>
            <Toaster />
            <CookieConsent />
        </>
    )
}

export default App
