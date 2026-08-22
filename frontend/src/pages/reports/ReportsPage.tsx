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
    title: string
    href: string
    icon: React.ReactNode
    description: string
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
        title: '庫存現況報表',
        href: '/stock-on-hand',
        icon: <BarChart3 className="h-4 w-4" />,
        description: '目前庫存狀況統計',
        role: 'admin',
        section: 'erp',
    },
    {
        title: '庫存流水報表',
        href: '/stock-ledger',
        icon: <FileText className="h-4 w-4" />,
        description: '庫存異動明細報表',
        role: 'admin',
        section: 'erp',
    },
    {
        title: '採購明細報表',
        href: '/purchase-lines',
        icon: <Truck className="h-4 w-4" />,
        description: '採購項目明細統計',
        role: 'admin',
        section: 'erp',
    },
    {
        title: '銷貨明細報表',
        href: '/sales-lines',
        icon: <ShoppingCart className="h-4 w-4" />,
        description: '銷貨項目明細統計',
        role: 'admin',
        section: 'erp',
    },
    {
        title: '成本摘要報表',
        href: '/cost-summary',
        icon: <BarChart3 className="h-4 w-4" />,
        description: '成本分析與摘要',
        role: 'admin',
        section: 'erp',
    },
    {
        title: '進銷貨彙總報表',
        href: '/purchase-sales-summary',
        icon: <TrendingUp className="h-4 w-4" />,
        description: '按月份、供應商客戶、產品類別彙總分析',
        role: 'admin',
        section: 'erp',
    },
    {
        title: '會計報表',
        href: '/accounting',
        icon: <BarChart3 className="h-4 w-4" />,
        description: '試算表、傳票、應付／應收帳款、損益表',
        role: 'admin',
        section: 'erp',
    },
    // 動物 / 血檢
    {
        title: '血液檢查費用報表',
        href: '/blood-test-cost',
        icon: <Droplets className="h-4 w-4" />,
        description: '依專案與日期查詢血檢費用',
        role: 'admin',
        section: 'animal',
    },
    {
        title: '血液檢查結果分析',
        href: '/blood-test-analysis',
        icon: <Activity className="h-4 w-4" />,
        description: '血檢數據統計、趨勢分析、異常值偵測',
        section: 'animal',
    },
    {
        title: '倉庫現況報表',
        href: '/warehouses',
        icon: <Warehouse className="h-4 w-4" />,
        description: '從倉庫列表進入單倉的現況/平面圖/PDF 報表',
        section: 'animal',
    },
    {
        title: '每週病歷彙整報表',
        href: '/weekly-medical-report',
        icon: <HeartPulse className="h-4 w-4" />,
        description: '依耳號、計畫案、日期查詢每日醫療事件，可匯出 Excel',
        permission: 'animal.record.view',
        section: 'animal',
    },
    {
        title: '廢棄物再利用月結報表',
        href: '/byproduct-monthly-report',
        icon: <FlaskConical className="h-4 w-4" />,
        description: '依時間區間查詢採樣紀錄，匯出 Excel 交負責人對帳',
        permission: 'animal.byproduct_sample.view',
        section: 'animal',
    },
    {
        title: '獸醫巡場報告',
        href: '/vet-patrol-reports',
        icon: <Stethoscope className="h-4 w-4" />,
        description: '巡場觀察 / 建議 / 追蹤改善歷史紀錄；可篩選我的草稿、待追蹤、已完成',
        permission: 'animal.record.view',
        section: 'animal',
    },
    // GLP / AUP
    {
        title: '操作日誌',
        href: '/admin/audit-logs',
        icon: <ScrollText className="h-4 w-4" />,
        description: 'GLP 合規操作軌跡 / HMAC chain 審計',
        role: 'admin',
        section: 'glp',
    },
    {
        title: '研究計畫總覽',
        href: '/protocols',
        icon: <FlaskConical className="h-4 w-4" />,
        description: 'AUP 研究計畫列表（已核准 / 審核中 / 修正案）',
        section: 'aup',
    },
]

const SECTION_LABELS: Record<ReportItem['section'], string> = {
    erp: 'ERP / 進銷存',
    animal: '動物管理 / 血檢',
    glp: 'GLP 合規',
    aup: '研究計畫',
}

export function ReportsPage() {
    const hasRole = useAuthHasRole()
    const hasPermission = useAuthHasPermission()

    const visible = reportItems.filter((item) => {
        if (item.role && !hasRole(item.role)) return false
        if (item.permission && !hasPermission(item.permission)) return false
        return true
    })

    const grouped = (Object.keys(SECTION_LABELS) as ReportItem['section'][])
        .map((section) => ({
            section,
            label: SECTION_LABELS[section],
            items: visible.filter((it) => it.section === section),
        }))
        .filter((g) => g.items.length > 0)

    return (
        <div className="space-y-8">
            <div>
                <h1 className="text-3xl font-bold tracking-tight">報表中心</h1>
                <p className="text-sm text-muted-foreground mt-1">
                    跨子系統報表入口；依個人權限顯示可用項目
                </p>
            </div>

            {grouped.length === 0 ? (
                <EmptyState
                    icon={FileSearch}
                    title="尚無可存取的報表項目"
                    description="您目前的權限沒有對應的報表入口；請聯絡系統管理員確認 ERP / GLP / AUP 相關權限是否已開通。"
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
                                            {item.title}
                                        </h3>
                                        <p className="text-xs text-muted-foreground mt-1">
                                            {item.description}
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
