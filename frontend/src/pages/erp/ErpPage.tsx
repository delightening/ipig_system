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
    ArrowLeftRight,
    ClipboardList,
    Settings2,
    Box,
    Users,
    Droplets,
    Activity,
    Wrench,
} from 'lucide-react'
import { EquipmentPage } from '@/pages/admin/EquipmentPage'
import { ProductsPage } from '@/pages/master/ProductsPage'
import { PartnersPage } from '@/pages/master/PartnersPage'

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
        id: 'purchasing',
        title: '採購管理',
        icon: <Truck className="h-5 w-5" />,
        description: '管理採購訂單與採購退貨',
        items: [
            {
                title: '採購單',
                href: '/documents?type=PO',
                icon: <FileText className="h-4 w-4" />,
                description: '建立與管理採購訂單',
            },
            {
                title: '採購退貨',
                href: '/documents?type=PR',
                icon: <ArrowLeftRight className="h-4 w-4" />,
                description: '處理採購退貨作業',
            },
        ],
    },
    {
        id: 'sales',
        title: '銷貨管理',
        icon: <ShoppingCart className="h-5 w-5" />,
        description: '管理銷貨訂單與出庫作業',
        items: [
            {
                title: '銷貨單',
                href: '/documents?type=SO',
                icon: <FileText className="h-4 w-4" />,
                description: '建立與管理銷貨訂單',
            },
            {
                title: '銷貨出庫',
                href: '/documents?type=DO',
                icon: <Box className="h-4 w-4" />,
                description: '處理銷貨出庫作業',
            },
        ],
    },
    {
        id: 'warehouse',
        title: '倉儲作業',
        icon: <Warehouse className="h-5 w-5" />,
        description: '庫存查詢與倉儲異動管理',
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
            {
                title: '調撥單',
                href: '/documents?type=TR',
                icon: <ArrowLeftRight className="h-4 w-4" />,
                description: '倉庫間調撥作業',
            },
            {
                title: '盤點單',
                href: '/documents?type=STK',
                icon: <ClipboardList className="h-4 w-4" />,
                description: '庫存盤點作業',
            },
            {
                title: '調整單',
                href: '/documents?type=ADJ',
                icon: <Settings2 className="h-4 w-4" />,
                description: '庫存數量調整',
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
                title: '會計報表',
                href: '/accounting',
                icon: <BarChart3 className="h-4 w-4" />,
                description: '試算表、傳票、應付／應收帳款',
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
