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
} from 'lucide-react'

interface ReportItem {
    title: string
    href: string
    icon: React.ReactNode
    description: string
}

const reportItems: ReportItem[] = [
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
]

export function ErpReportsPage() {
    return (
        <div className="space-y-6">
            <h1 className="text-3xl font-bold tracking-tight">報表中心</h1>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {reportItems.map((item) => (
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
                                <p className="text-xs text-muted-foreground mt-1">{item.description}</p>
                            </div>
                            <ChevronRight className="h-5 w-5 shrink-0 text-muted-foreground/30 group-hover:text-primary transition-colors" />
                        </div>
                    </Link>
                ))}
            </div>
        </div>
    )
}
