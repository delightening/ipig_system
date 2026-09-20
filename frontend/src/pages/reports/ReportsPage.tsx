/**
 * R35-24: 報表中心
 *
 * 取代舊有 `/erp/reports`（僅 ERP 9 個報表入口），擴充為跨子系統 hub：
 *   - ERP 9 項：庫存 / 採購 / 銷貨 / 成本 / 血檢 / 進銷彙總 / 會計
 *   - GLP audit：操作日誌（admin only）
 *   - 動物管理：倉庫現況（指向 /warehouses 列表，內部選倉進現況報表）
 *   - AUP：研究計畫總覽
 *
 * Permission 條件式 render — user 沒權限的條目自動隱藏，避免 dead clicks。
 */
import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import {
    Truck,
    ShoppingCart,
    BarChart3,
    FileText,
    ChevronRight,
    Droplets,
    Activity,
    TrendingUp,
    ScrollText,
    Warehouse,
    FlaskConical,
    FileSearch,
    Stethoscope,
    HeartPulse,
} from 'lucide-react'
import { useAuthHasPermission, useAuthHasRole } from '@/stores/auth'
import { EmptyState } from '@/components/ui/empty-state'

interface ReportItem {
    titleKey: string
    href: string
    icon: React.ReactNode
    descriptionKey: string
    /** 顯示條件：未指定 = 對所有登入用戶顯示 */
    permission?: string
    /** 角色條件：未指定 = 不限角色 */
    role?: string
    /** 分區標籤，做視覺分組 */
    section: 'erp' | 'glp' | 'animal' | 'aup'
}

const reportItems: ReportItem[] = [
    // ERP 報表（與舊 ErpReportsPage 對齊；保留 admin gate）
    {
        titleKey: 'reportsPages.stockOnHand.title',
        href: '/stock-on-hand',
        icon: <BarChart3 className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.stockOnHand.description',
        role: 'admin',
        section: 'erp',
    },
    {
        titleKey: 'reportsPages.stockLedger.title',
        href: '/stock-ledger',
        icon: <FileText className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.stockLedger.description',
        role: 'admin',
        section: 'erp',
    },
    {
        titleKey: 'reportsPages.purchaseLines.title',
        href: '/purchase-lines',
        icon: <Truck className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.purchaseLines.description',
        role: 'admin',
        section: 'erp',
    },
    {
        titleKey: 'reportsPages.salesLines.title',
        href: '/sales-lines',
        icon: <ShoppingCart className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.salesLines.description',
        role: 'admin',
        section: 'erp',
    },
    {
        titleKey: 'reportsPages.protocolConsumption.title',
        href: '/protocol-consumption',
        icon: <FlaskConical className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.protocolConsumption.description',
        // 用 permission 而非 role：後端這支的閘就是 erp.report.view，而該權限授予
        // WAREHOUSE_MANAGER / PURCHASING / ADMIN_STAFF——他們呼叫得到 API，卻會被
        // `role: 'admin'` 擋在選單外。hasPermission 對 SYSTEM_ADMIN / admin / GUEST
        // 一律短路放行（比照後端 is_admin()），所以管理員與訪客示範不受影響。
        // ⚠️ 同區其他 ERP 報表仍是 role: 'admin'，那是既有的前後端不一致，另案處理。
        permission: 'erp.report.view',
        section: 'erp',
    },
    {
        titleKey: 'reportsPages.costSummary.title',
        href: '/cost-summary',
        icon: <BarChart3 className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.costSummary.description',
        role: 'admin',
        section: 'erp',
    },
    {
        titleKey: 'reportsPages.purchaseSales.title',
        href: '/purchase-sales-summary',
        icon: <TrendingUp className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.purchaseSales.description',
        role: 'admin',
        section: 'erp',
    },
    {
        titleKey: 'reportsPages.accounting.title',
        href: '/accounting',
        icon: <BarChart3 className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.accounting.description',
        role: 'admin',
        section: 'erp',
    },
    // 動物 / 血檢
    {
        titleKey: 'reportsPages.bloodTestCost.title',
        href: '/blood-test-cost',
        icon: <Droplets className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.bloodTestCost.description',
        role: 'admin',
        section: 'animal',
    },
    {
        titleKey: 'reportsPages.bloodTestAnalysis.title',
        href: '/blood-test-analysis',
        icon: <Activity className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.bloodTestAnalysis.description',
        section: 'animal',
    },
    {
        titleKey: 'reportsPages.hub.items.warehouseStatus.title',
        href: '/warehouses',
        icon: <Warehouse className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.warehouseStatus.description',
        section: 'animal',
    },
    {
        titleKey: 'reportsPages.hub.items.weeklyMedical.title',
        href: '/weekly-medical-report',
        icon: <HeartPulse className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.weeklyMedical.description',
        permission: 'animal.record.view',
        section: 'animal',
    },
    {
        titleKey: 'reportsPages.byproductMonthly.title',
        href: '/byproduct-monthly-report',
        icon: <FlaskConical className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.byproductMonthly.description',
        permission: 'animal.byproduct_sample.view',
        section: 'animal',
    },
    {
        titleKey: 'reportsPages.hub.items.vetPatrol.title',
        href: '/vet-patrol-reports',
        icon: <Stethoscope className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.vetPatrol.description',
        permission: 'animal.record.view',
        section: 'animal',
    },
    // GLP / AUP
    {
        titleKey: 'reportsPages.hub.items.auditLogs.title',
        href: '/admin/audit-logs',
        icon: <ScrollText className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.auditLogs.description',
        role: 'admin',
        section: 'glp',
    },
    {
        titleKey: 'reportsPages.hub.items.protocolsOverview.title',
        href: '/protocols',
        icon: <FlaskConical className="h-4 w-4" />,
        descriptionKey: 'reportsPages.hub.items.protocolsOverview.description',
        section: 'aup',
    },
]

const SECTION_LABEL_KEYS: Record<ReportItem['section'], string> = {
    erp: 'reportsPages.hub.sections.erp',
    animal: 'reportsPages.hub.sections.animal',
    glp: 'reportsPages.hub.sections.glp',
    aup: 'reportsPages.hub.sections.aup',
}

export function ReportsPage() {
    const { t } = useTranslation()
    const hasRole = useAuthHasRole()
    const hasPermission = useAuthHasPermission()

    const visible = reportItems.filter((item) => {
        if (item.role && !hasRole(item.role)) return false
        if (item.permission && !hasPermission(item.permission)) return false
        return true
    })

    const grouped = (Object.keys(SECTION_LABEL_KEYS) as ReportItem['section'][])
        .map((section) => ({
            section,
            label: t(SECTION_LABEL_KEYS[section]),
            items: visible.filter((it) => it.section === section),
        }))
        .filter((g) => g.items.length > 0)

    return (
        <div className="space-y-8">
            <div>
                <h1 className="text-3xl font-bold tracking-tight">{t('reportsPages.hub.title')}</h1>
                <p className="text-sm text-muted-foreground mt-1">
                    {t('reportsPages.hub.subtitle')}
                </p>
            </div>

            {grouped.length === 0 ? (
                <EmptyState
                    icon={FileSearch}
                    title={t('reportsPages.hub.emptyTitle')}
                    description={t('reportsPages.hub.emptyDescription')}
                />
            ) : grouped.map(({ section, label, items }) => (
                <section key={section} className="space-y-3">
                    <h2 className="text-lg font-semibold text-muted-foreground">{label}</h2>
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {items.map((item) => (
                            <Link
                                key={item.href}
                                to={item.href}
                                className="group bg-card rounded-xl border border-border p-5 hover:shadow-lg hover:border-primary/30 transition-all"
                            >
                                <div className="flex items-start space-x-4">
                                    <div className="p-2.5 bg-muted rounded-lg text-muted-foreground group-hover:bg-primary/10 group-hover:text-primary transition-colors">
                                        {item.icon}
                                    </div>
                                    <div className="flex-1 min-w-0">
                                        <h3 className="font-medium text-foreground group-hover:text-primary transition-colors">
                                            {t(item.titleKey)}
                                        </h3>
                                        <p className="text-xs text-muted-foreground mt-1">
                                            {t(item.descriptionKey)}
                                        </p>
                                    </div>
                                    <ChevronRight className="h-5 w-5 shrink-0 text-muted-foreground/30 group-hover:text-primary transition-colors" />
                                </div>
                            </Link>
                        ))}
                    </div>
                </section>
            ))}
        </div>
    )
}
