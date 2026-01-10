import { Link } from 'react-router-dom'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Package,
  TrendingUp,
  ShoppingCart,
  Truck,
  DollarSign,
} from 'lucide-react'

const reportCards = [
  {
    title: '庫存現況報表',
    description: '查看各倉庫商品庫存量、成本與價值',
    href: '/reports/stock-on-hand',
    icon: Package,
    color: 'text-blue-500',
  },
  {
    title: '庫存流水報表',
    description: '追蹤所有庫存異動記錄',
    href: '/reports/stock-ledger',
    icon: TrendingUp,
    color: 'text-green-500',
  },
  {
    title: '採購明細報表',
    description: '查看採購單、採購入庫、採購退貨明細',
    href: '/reports/purchase-lines',
    icon: Truck,
    color: 'text-orange-500',
  },
  {
    title: '銷售明細報表',
    description: '查看銷售單、銷售出庫、銷售退貨明細',
    href: '/reports/sales-lines',
    icon: ShoppingCart,
    color: 'text-purple-500',
  },
  {
    title: '成本摘要報表',
    description: '查看庫存成本與價值摘要',
    href: '/reports/cost-summary',
    icon: DollarSign,
    color: 'text-emerald-500',
  },
]

export function ReportsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">報表中心</h1>
        <p className="text-muted-foreground">
          選擇要查看的報表類型
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {reportCards.map((report) => (
          <Link key={report.href} to={report.href}>
            <Card className="hover:shadow-lg transition-shadow cursor-pointer h-full">
              <CardHeader>
                <div className="flex items-center space-x-3">
                  <report.icon className={`h-8 w-8 ${report.color}`} />
                  <CardTitle className="text-lg">{report.title}</CardTitle>
                </div>
              </CardHeader>
              <CardContent>
                <CardDescription>{report.description}</CardDescription>
              </CardContent>
            </Card>
          </Link>
        ))}
      </div>
    </div>
  )
}
