import { useMemo, useEffect } from 'react'
import { Link, useSearchParams, useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/auth'
import { cn } from '@/lib/utils'
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
} from 'lucide-react'
import { EquipmentPage } from '@/pages/admin/EquipmentPage'
import { ProductsPage } from '@/pages/master/ProductsPage'
import { PartnersPage } from '@/pages/master/PartnersPage'
import { DocumentsPage } from '@/pages/documents/DocumentsPage'

// 定義 ERP 子模組
interface ErpModule {
    id: string
    title: string
    titleKey?: string
    icon: React.ReactNode
    description: string
    descriptionKey?: string
    items: {
        title: string
        titleKey?: string
        href: string
        icon: React.ReactNode
        description?: string
        descriptionKey?: string
    }[]
}

const erpModules: ErpModule[] = [
    {
        id: 'products',
        title: '產品管理',
        icon: <Package className="h-5 w-5" />,
        description: '管理產品資料',
        items: [],
    },
    {
        id: 'documents',
        title: '單據管理',
        icon: <FileText className="h-5 w-5" />,
        description: '採購、銷貨、倉儲單據統一管理',
        items: [],
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
        items: [],
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
        title: '供應商/客戶',
        icon: <Users className="h-5 w-5" />,
        description: '管理供應商與客戶',
        items: [],
    },
]

export function ErpPage() {
  useNavigate() // router context
    const [searchParams, setSearchParams] = useSearchParams()
    const { hasRole, user } = useAuthStore()

    // 從 URL 取得當前 tab，預設為第一個模組 'products'
    const currentTab = searchParams.get('tab') || 'products'

    // 根據權限過濾模組
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

    // 如果沒有 tab 參數，或 tab 已無效（如已刪除的 master），自動導向第一個模組
    useEffect(() => {
        const tab = searchParams.get('tab')
        const hasValidTab = filteredModules.some(m => m.id === tab)
        if ((!tab || !hasValidTab) && filteredModules.length > 0) {
            setSearchParams({ tab: filteredModules[0].id }, { replace: true })
        }
    }, [searchParams, filteredModules, setSearchParams])

    // 切換 tab
    const handleTabChange = (tabId: string) => {
        setSearchParams({ tab: tabId })
    }

    // 取得當前模組
    const currentModule = filteredModules.find(m => m.id === currentTab)

    return (
        <div className="space-y-6">
            {/* 頁面標題 */}
            <div className="flex items-center justify-between">
                <h1 className="text-3xl font-bold tracking-tight">ERP 系統</h1>
            </div>

            {/* Tab 導覽列（參考動物列表 tags 樣式） */}
            <div className="flex flex-wrap gap-2 border-b border-slate-200">
                {filteredModules.map((module) => (
                    <button
                        key={module.id}
                        onClick={() => handleTabChange(module.id)}
                        className={cn(
                            'px-3 md:px-4 py-2 border-b-2 font-medium text-xs md:text-sm transition-colors flex items-center gap-1.5',
                            currentTab === module.id
                                ? 'border-blue-500 text-blue-600 -mb-px'
                                : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
                        )}
                    >
                        {module.icon}
                        <span>{module.title}</span>
                    </button>
                ))}
            </div>

            {/* Tab 內容區 */}
            {currentModule ? (
                currentTab === 'equipment' ? (
                    <EquipmentPage />
                ) : currentTab === 'products' ? (
                    <ProductsPage />
                ) : currentTab === 'partners' ? (
                    <PartnersPage />
                ) : currentTab === 'documents' ? (
                    <DocumentsPage />
                ) : (
                <div className="space-y-6">
                    {/* 功能列表（無說明欄位） */}
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        {currentModule.items.map((item) => (
                            <Link
                                key={item.href}
                                to={item.href}
                                className="group bg-white rounded-xl border border-slate-200 p-5 hover:shadow-lg hover:border-blue-200 transition-all"
                            >
                                <div className="flex items-start space-x-4">
                                    <div className="p-2.5 bg-slate-100 rounded-lg text-slate-600 group-hover:bg-blue-100 group-hover:text-blue-600 transition-colors">
                                        {item.icon}
                                    </div>
                                    <div className="flex-1 min-w-0">
                                        <h3 className="font-medium text-slate-900 group-hover:text-blue-600 transition-colors">
                                            {item.title}
                                        </h3>
                                    </div>
                                    <ChevronRight className="h-5 w-5 shrink-0 text-slate-300 group-hover:text-blue-500 transition-colors" />
                                </div>
                            </Link>
                        ))}
                    </div>
                </div>
                )
            ) : (
                // 未找到模組
                <div className="text-center py-12">
                    <p className="text-slate-500">找不到此模組</p>
                </div>
            )}
        </div>
    )
}
