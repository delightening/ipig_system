import { useMemo, useEffect } from 'react'
import { Link, useSearchParams, useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
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
} from 'lucide-react'

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
        title: '銷售管理',
        icon: <ShoppingCart className="h-5 w-5" />,
        description: '管理銷售訂單與出庫作業',
        items: [
            {
                title: '銷售單',
                href: '/documents?type=SO',
                icon: <FileText className="h-4 w-4" />,
                description: '建立與管理銷售訂單',
            },
            {
                title: '銷售出庫',
                href: '/documents?type=DO',
                icon: <Box className="h-4 w-4" />,
                description: '處理銷售出庫作業',
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
            {
                title: '倉庫佈局',
                href: '/inventory/layout',
                icon: <Warehouse className="h-4 w-4" />,
                description: '視覺化管理貨架位置',
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
                title: '銷售明細報表',
                href: '/sales-lines',
                icon: <ShoppingCart className="h-4 w-4" />,
                description: '銷售項目明細統計',
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
        ],
    },
    {
        id: 'master',
        title: '基礎資料',
        icon: <Package className="h-5 w-5" />,
        description: '管理產品、倉庫與往來對象',
        items: [
            {
                title: '產品管理',
                href: '/products',
                icon: <Package className="h-4 w-4" />,
                description: '管理產品資料',
            },
            {
                title: '倉庫管理',
                href: '/warehouses',
                icon: <Warehouse className="h-4 w-4" />,
                description: '管理倉庫設定',
            },
            {
                title: '供應商/客戶',
                href: '/partners',
                icon: <Users className="h-4 w-4" />,
                description: '管理供應商與客戶',
            },
            {
                title: '血液檢查項目',
                href: '/blood-test-templates',
                icon: <Droplets className="h-4 w-4" />,
                description: '管理血液檢查項目模板',
            },
        ],
    },
]

export function ErpPage() {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars -- i18n for future use
  const { t } = useTranslation()
  useNavigate() // router context
    const [searchParams, setSearchParams] = useSearchParams()
    const { hasRole, user, hasPermission } = useAuthStore()

    // 從 URL 取得當前 tab，預設為第一個模組 'purchasing'
    const currentTab = searchParams.get('tab') || 'purchasing'

    // 根據權限過濾模組
    const filteredModules = useMemo(() => {
        const hasErpAccess = hasRole('admin') ||
            user?.roles.some(r => ['purchasing', 'approver', 'WAREHOUSE_MANAGER', 'EXPERIMENT_STAFF'].includes(r)) ||
            user?.permissions.some(p => p.startsWith('erp.'))

        if (!hasErpAccess) return []
        return erpModules
    // eslint-disable-next-line react-hooks/exhaustive-deps -- hasPermission stable
    }, [hasRole, user, hasPermission])

    // 如果沒有 tab 參數，自動導向第一個模組
    useEffect(() => {
        if (!searchParams.get('tab') && filteredModules.length > 0) {
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
        <div className="p-6 space-y-6">
            {/* 頁面標題 */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold text-slate-900">ERP 系統</h1>
                    <p className="text-slate-500 mt-1">企業資源規劃與庫存管理</p>
                </div>
            </div>

            {/* Tab 導覽列 */}
            <div className="border-b border-slate-200">
                <nav className="-mb-px flex flex-wrap gap-x-6">
                    {/* 各模組 Tabs */}
                    {filteredModules.map((module) => (
                        <button
                            key={module.id}
                            onClick={() => handleTabChange(module.id)}
                            className={cn(
                                'py-3 px-1 border-b-2 font-medium text-sm transition-colors',
                                currentTab === module.id
                                    ? 'border-blue-500 text-blue-600'
                                    : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
                            )}
                        >
                            <div className="flex items-center space-x-2">
                                {module.icon}
                                <span>{module.title}</span>
                            </div>
                        </button>
                    ))}
                </nav>
            </div>

            {/* Tab 內容區 */}
            {currentModule ? (
                // 模組的詳細內容
                <div className="space-y-6">
                    {/* 模組描述 */}
                    <div className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-xl p-6 border border-blue-100">
                        <div className="flex items-start space-x-4">
                            <div className="p-3 bg-white rounded-xl shadow-sm text-blue-600">
                                {currentModule.icon}
                            </div>
                            <div>
                                <h2 className="text-xl font-semibold text-slate-900">{currentModule.title}</h2>
                                <p className="text-slate-600 mt-1">{currentModule.description}</p>
                            </div>
                        </div>
                    </div>

                    {/* 功能列表 */}
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
                                    <div className="flex-1">
                                        <h3 className="font-medium text-slate-900 group-hover:text-blue-600 transition-colors">
                                            {item.title}
                                        </h3>
                                        {item.description && (
                                            <p className="text-sm text-slate-500 mt-1">{item.description}</p>
                                        )}
                                    </div>
                                    <ChevronRight className="h-5 w-5 text-slate-300 group-hover:text-blue-500 transition-colors" />
                                </div>
                            </Link>
                        ))}
                    </div>
                </div>
            ) : (
                // 未找到模組
                <div className="text-center py-12">
                    <p className="text-slate-500">找不到此模組</p>
                </div>
            )}
        </div>
    )
}
