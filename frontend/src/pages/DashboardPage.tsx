import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import api, { LowStockAlert, DocumentListItem } from '@/lib/api'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { formatNumber, formatDate } from '@/lib/utils'
import {
  Package,
  TrendingUp,
  TrendingDown,
  AlertTriangle,
  FileText,
  Loader2,
  Calendar,
} from 'lucide-react'

export function DashboardPage() {
  // 低庫存警示
  const { data: lowStockAlerts, isLoading: loadingAlerts } = useQuery({
    queryKey: ['low-stock-alerts'],
    queryFn: async () => {
      const response = await api.get<LowStockAlert[]>('/inventory/low-stock')
      return response.data
    },
  })

  // 最近單據
  const { data: recentDocuments, isLoading: loadingDocuments } = useQuery({
    queryKey: ['recent-documents'],
    queryFn: async () => {
      const response = await api.get<DocumentListItem[]>('/documents')
      return response.data.slice(0, 10)
    },
  })

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'draft':
        return <Badge variant="secondary">草稿</Badge>
      case 'submitted':
        return <Badge variant="warning">待核准</Badge>
      case 'approved':
        return <Badge variant="success">已核准</Badge>
      case 'cancelled':
        return <Badge variant="destructive">已作廢</Badge>
      default:
        return <Badge variant="outline">{status}</Badge>
    }
  }

  const getDocTypeName = (type: string) => {
    const names: Record<string, string> = {
      PO: '採購單',
      GRN: '採購入庫',
      PR: '採購退貨',
      SO: '銷售單',
      DO: '銷售出庫',
      SR: '銷售退貨',
      TR: '調撥單',
      STK: '盤點單',
      ADJ: '調整單',
    }
    return names[type] || type
  }

  // 計算近 7 天出入庫趨勢
  const trendData = useMemo(() => {
    if (!recentDocuments) return []

    const today = new Date()
    const days: { date: string; dateStr: string; inbound: number; outbound: number }[] = []

    for (let i = 6; i >= 0; i--) {
      const date = new Date(today)
      date.setDate(date.getDate() - i)
      const dateStr = date.toISOString().split('T')[0]
      const displayDate = date.toLocaleDateString('zh-TW', { month: 'short', day: 'numeric' })

      const dayDocs = recentDocuments.filter(
        (d) => d.status === 'approved' && d.approved_at?.startsWith(dateStr)
      )

      const inbound = dayDocs.filter((d) => ['GRN', 'SR'].includes(d.doc_type)).length
      const outbound = dayDocs.filter((d) => ['DO', 'PR'].includes(d.doc_type)).length

      days.push({ date: dateStr, dateStr: displayDate, inbound, outbound })
    }

    return days
  }, [recentDocuments])

  return (
    <div className="space-y-6">
      {/* Page header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">儀表板</h1>
        <p className="text-muted-foreground">
          歡迎使用進銷存管理系統
        </p>
      </div>

      {/* Stats cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">低庫存警示</CardTitle>
            <AlertTriangle className="h-4 w-4 text-yellow-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {loadingAlerts ? '-' : lowStockAlerts?.length || 0}
            </div>
            <p className="text-xs text-muted-foreground">
              需要補貨的品項
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">待處理單據</CardTitle>
            <FileText className="h-4 w-4 text-blue-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {loadingDocuments
                ? '-'
                : recentDocuments?.filter((d) => d.status === 'submitted').length || 0}
            </div>
            <p className="text-xs text-muted-foreground">
              等待核准的單據
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">今日入庫</CardTitle>
            <TrendingUp className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {loadingDocuments
                ? '-'
                : recentDocuments?.filter(
                    (d) =>
                      ['GRN', 'SR'].includes(d.doc_type) &&
                      d.status === 'approved' &&
                      new Date(d.approved_at || '').toDateString() === new Date().toDateString()
                  ).length || 0}
            </div>
            <p className="text-xs text-muted-foreground">
              入庫單據數量
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">今日出庫</CardTitle>
            <TrendingDown className="h-4 w-4 text-red-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {loadingDocuments
                ? '-'
                : recentDocuments?.filter(
                    (d) =>
                      ['DO', 'PR'].includes(d.doc_type) &&
                      d.status === 'approved' &&
                      new Date(d.approved_at || '').toDateString() === new Date().toDateString()
                  ).length || 0}
            </div>
            <p className="text-xs text-muted-foreground">
              出庫單據數量
            </p>
          </CardContent>
        </Card>
      </div>

      {/* 近 7 天出入庫趨勢 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Calendar className="h-5 w-5 text-indigo-500" />
            近 7 天出入庫趨勢
          </CardTitle>
          <CardDescription>最近一週的出入庫單據統計</CardDescription>
        </CardHeader>
        <CardContent>
          {loadingDocuments ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>日期</TableHead>
                  <TableHead className="text-right">入庫單據</TableHead>
                  <TableHead className="text-right">出庫單據</TableHead>
                  <TableHead className="text-right">淨變動</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {trendData.map((day) => (
                  <TableRow key={day.date}>
                    <TableCell className="font-medium">{day.dateStr}</TableCell>
                    <TableCell className="text-right">
                      <span className="inline-flex items-center gap-1 text-green-600">
                        <TrendingUp className="h-3 w-3" />
                        {day.inbound}
                      </span>
                    </TableCell>
                    <TableCell className="text-right">
                      <span className="inline-flex items-center gap-1 text-red-600">
                        <TrendingDown className="h-3 w-3" />
                        {day.outbound}
                      </span>
                    </TableCell>
                    <TableCell className="text-right">
                      <span
                        className={
                          day.inbound - day.outbound > 0
                            ? 'text-green-600'
                            : day.inbound - day.outbound < 0
                            ? 'text-red-600'
                            : 'text-muted-foreground'
                        }
                      >
                        {day.inbound - day.outbound > 0 ? '+' : ''}
                        {day.inbound - day.outbound}
                      </span>
                    </TableCell>
                  </TableRow>
                ))}
                {/* 總計行 */}
                <TableRow className="bg-muted/50 font-medium">
                  <TableCell>總計</TableCell>
                  <TableCell className="text-right text-green-600">
                    {trendData.reduce((sum, d) => sum + d.inbound, 0)}
                  </TableCell>
                  <TableCell className="text-right text-red-600">
                    {trendData.reduce((sum, d) => sum + d.outbound, 0)}
                  </TableCell>
                  <TableCell className="text-right">
                    {(() => {
                      const net =
                        trendData.reduce((sum, d) => sum + d.inbound, 0) -
                        trendData.reduce((sum, d) => sum + d.outbound, 0)
                      return (
                        <span
                          className={
                            net > 0 ? 'text-green-600' : net < 0 ? 'text-red-600' : ''
                          }
                        >
                          {net > 0 ? '+' : ''}
                          {net}
                        </span>
                      )
                    })()}
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <div className="grid gap-4 md:grid-cols-2">
        {/* Low stock alerts */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-yellow-500" />
              低庫存警示
            </CardTitle>
            <CardDescription>庫存量低於安全庫存的品項</CardDescription>
          </CardHeader>
          <CardContent>
            {loadingAlerts ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : lowStockAlerts && lowStockAlerts.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>品項</TableHead>
                    <TableHead>倉庫</TableHead>
                    <TableHead className="text-right">現有量</TableHead>
                    <TableHead className="text-right">安全庫存</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {lowStockAlerts.slice(0, 5).map((alert) => (
                    <TableRow key={`${alert.warehouse_id}-${alert.product_id}`}>
                      <TableCell>
                        <div>
                          <div className="font-medium">{alert.product_name}</div>
                          <div className="text-xs text-muted-foreground">{alert.product_sku}</div>
                        </div>
                      </TableCell>
                      <TableCell>{alert.warehouse_name}</TableCell>
                      <TableCell className="text-right text-red-500">
                        {formatNumber(alert.qty_on_hand, 0)}
                      </TableCell>
                      <TableCell className="text-right">
                        {formatNumber(alert.safety_stock, 0)}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                <Package className="h-12 w-12 mb-2" />
                <p>目前沒有低庫存警示</p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Recent documents */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <FileText className="h-5 w-5 text-blue-500" />
              最近單據
            </CardTitle>
            <CardDescription>最近建立或更新的單據</CardDescription>
          </CardHeader>
          <CardContent>
            {loadingDocuments ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : recentDocuments && recentDocuments.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>單號</TableHead>
                    <TableHead>類型</TableHead>
                    <TableHead>狀態</TableHead>
                    <TableHead>日期</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {recentDocuments.slice(0, 5).map((doc) => (
                    <TableRow key={doc.id}>
                      <TableCell className="font-medium">{doc.doc_no}</TableCell>
                      <TableCell>{getDocTypeName(doc.doc_type)}</TableCell>
                      <TableCell>{getStatusBadge(doc.status)}</TableCell>
                      <TableCell>{formatDate(doc.doc_date)}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                <FileText className="h-12 w-12 mb-2" />
                <p>尚無單據資料</p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
