import { useMemo, useEffect } from 'react'
import { Link, useSearchParams } from 'react-router-dom'
import { useAuthStore } from '@/stores/auth'
import { cn } from '@/lib/utils'
import { PageHeader } from '@/components/ui/page-header'
import { EmptyState } from '@/components/ui/empty-state'
import {
    Truck,
    ShoppingCart,
    Warehouse,
    BarChart3,
    Package,
    ChevronRight,
    FileText,
    ClipboardList,
    Users,
    Droplets,
    Activity,
    Wrench,
    TrendingUp,
    Search,
} from 'lucide-react'

interface ErpModuleItem {
    title: string
    href: string
    icon: React.ReactNode
    description?: string
}

interface ErpModule {
    id: string
    title: string
    icon: React.ReactNode
    description: string
    items: ErpModuleItem[]
}

const erpModules: ErpModule[] = [
    {
        id: 'products',
        title: '產品管理',
        icon: <Package className="h-5 w-5" />,
        description: '管理產品資料',
        items: [
            {
                title: '產品列表',
                href: '/products',
                icon: <Package className="h-4 w-4" />,
                description: '瀏覽、搜尋、管理所有產品',
            },
            {
                title: '新增產品',
                href: '/products/new',
                icon: <Package className="h-4 w-4" />,
                description: '建立新產品與 SKU',
            },
        ],
    },
    {
        id: 'documents',
        title: '單據管理',
        icon: <FileText className="h-5 w-5" />,
        description: '採購、銷貨、倉儲單據統一管理',
        items: [
            {
                title: '單據列表',
                href: '/documents',
                icon: <FileText className="h-4 w-4" />,
                description: '瀏覽所有採購、銷貨、調撥單據',
            },
            {
                title: '新增單據',
                href: '/documents/new',
                icon: <FileText className="h-4 w-4" />,
                description: '建立新的單據',
            },
        ],
    },
    {
        id: 'warehouse',
        title: '倉儲作業',
        icon: <Warehouse className="h-5 w-5" />,
        description: '庫存查詢與倉庫管理',
        items: [
            {
                title: '倉庫',
                href: '/warehouses',
                icon: <Warehouse className="h-4 w-4" />,
                description: '管理倉庫資料與貨架佈局',
            },
            {
                title: '庫存查詢',
                href: '/inventory',
                icon: <ClipboardList className="h-4 w-4" />,
                description: '查詢目前庫存狀況',
            },
            {
                title: '庫存流水',
                href: '/inventory/ledger',
                icon: <FileText className="h-4 w-4" />,
                description: '檢視庫存異動記錄',
            },
        ],
    },
    {
        id: 'equipment',
        title: '設備維護',
        icon: <Wrench className="h-5 w-5" />,
        description: '設備與校正紀錄管理',
        items: [
            {
                title: '設備管理',
                href: '/erp?tab=equipment&view=inline',
                icon: <Wrench className="h-4 w-4" />,
                description: '設備清單、校正紀錄、維護排程',
            },
        ],
    },
    {
        id: 'reports',
        title: '報表中心',
        icon: <BarChart3 className="h-5 w-5" />,
        description: '各類統計報表與分析',
        items: [
            {
                title: '庫存現況報表',
                href: '/stock-on-hand',
                icon: <BarChart3 className="h-4 w-4" />,
                description: '目前庫存狀況統計',
            },
            {
                title: '庫存流水報表',
                href: '/stock-ledger',
                icon: <FileText className="h-4 w-4" />,
                description: '庫存異動明細報表',
            },
            {
                title: '採購明細報表',
                href: '/purchase-lines',
                icon: <Truck className="h-4 w-4" />,
                description: '採購項目明細統計',
            },
            {
                title: '銷貨明細報表',
                href: '/sales-lines',
                icon: <ShoppingCart className="h-4 w-4" />,
                description: '銷貨項目明細統計',
            },
            {
                title: '成本摘要報表',
                href: '/cost-summary',
                icon: <BarChart3 className="h-4 w-4" />,
                description: '成本分析與摘要',
            },
            {
                title: '血液檢查費用報表',
                href: '/blood-test-cost',
                icon: <Droplets className="h-4 w-4" />,
                description: '依專案與日期查詢血檢費用',
            },
            {
                title: '血液檢查結果分析',
                href: '/blood-test-analysis',
                icon: <Activity className="h-4 w-4" />,
                description: '血檢數據統計、趨勢分析、異常值偵測',
            },
            {
                title: '進銷貨彙總報表',
                href: '/purchase-sales-summary',
                icon: <TrendingUp className="h-4 w-4" />,
                description: '按月份、供應商客戶、產品類別彙總分析',
            },
            {
                title: '會計報表',
                href: '/accounting',
                icon: <BarChart3 className="h-4 w-4" />,
                description: '試算表、傳票、應付／應收帳款、損益表',
            },
        ],
    },
    {
        id: 'partners',
        title: '供應商／客戶',
        icon: <Users className="h-5 w-5" />,
        description: '管理供應商與客戶',
        items: [
            {
                title: '供應商／客戶列表',
                href: '/partners',
                icon: <Users className="h-4 w-4" />,
                description: '管理供應商與客戶資料',
            },
        ],
    },
]

export function ErpPage() {
    const [searchParams, setSearchParams] = useSearchParams()
    const { hasRole, user } = useAuthStore()

    const currentTab = searchParams.get('tab') || 'products'
    const isInlineView = searchParams.get('view') === 'inline'

    const filteredModules = useMemo(() => {
        const hasErpAccess = hasRole('admin') ||
            user?.roles.some(r => ['purchasing', 'approver', 'WAREHOUSE_MANAGER', 'EXPERIMENT_STAFF'].includes(r)) ||
            user?.permissions.some(p => p.startsWith('erp.'))

        const hasEquipmentAccess = hasRole('admin') ||
            user?.permissions?.some(p => p.startsWith('equipment.'))

        if (!hasErpAccess) return []
        return erpModules.filter((m) => {
            if (m.id === 'equipment') return hasEquipmentAccess
            return true
        })
    }, [hasRole, user])

    useEffect(() => {
        const tab = searchParams.get('tab')
        const hasValidTab = filteredModules.some(m => m.id === tab)
        if ((!tab || !hasValidTab) && filteredModules.length > 0) {
            setSearchParams({ tab: filteredModules[0].id }, { replace: true })
        }
    }, [searchParams, filteredModules, setSearchParams])

    const handleTabChange = (tabId: string) => {
        setSearchParams({ tab: tabId })
    }

    const currentModule = filteredModules.find(m => m.id === currentTab)

    // Equipment 保留 inline 渲染（因為沒有獨立路由）
    if (currentTab === 'equipment' && isInlineView) {
        const { EquipmentPage } = require('@/pages/admin/EquipmentPage')
        return (
            <div className="space-y-6">
                <PageHeader title="ERP 系統" />
                <div className="flex flex-wrap gap-2 border-b border-border">
                    {filteredModules.map((module) => (
                        <button
                            key={module.id}
                            onClick={() => handleTabChange(module.id)}
                            className={cn(
                                'px-3 md:px-4 py-2 border-b-2 font-medium text-xs md:text-sm transition-colors flex items-center gap-1.5',
                                currentTab === module.id
                                    ? 'border-primary text-primary -mb-px'
                                    : 'border-transparent text-muted-foreground hover:text-foreground hover:border-border'
                            )}
                        >
                            {module.icon}
                            <span>{module.title}</span>
                        </button>
                    ))}
                </div>
                <EquipmentPage />
            </div>
        )
    }

    return (
        <div className="space-y-6">
            <PageHeader title="ERP 系統" />

            <div className="flex flex-wrap gap-2 border-b border-border">
                {filteredModules.map((module) => (
                    <button
                        key={module.id}
                        onClick={() => handleTabChange(module.id)}
                        className={cn(
                            'px-3 md:px-4 py-2 border-b-2 font-medium text-xs md:text-sm transition-colors flex items-center gap-1.5',
                            currentTab === module.id
                                ? 'border-primary text-primary -mb-px'
                                : 'border-transparent text-muted-foreground hover:text-foreground hover:border-border'
                        )}
                    >
                        {module.icon}
                        <span>{module.title}</span>
                    </button>
                ))}
            </div>

            {currentModule ? (
                <div className="space-y-6">
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {currentModule.items.map((item) => (
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
                                        {item.description && (
                                            <p className="text-xs text-muted-foreground mt-1">{item.description}</p>
                                        )}
                                    </div>
                                    <ChevronRight className="h-5 w-5 shrink-0 text-muted-foreground/30 group-hover:text-primary transition-colors" />
                                </div>
                            </Link>
                        ))}
                    </div>
                </div>
            ) : (
                <EmptyState icon={Search} title="找不到此模組" />
            )}
        </div>
    )
}
