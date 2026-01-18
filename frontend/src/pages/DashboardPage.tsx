import { useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { LowStockAlert, DocumentListItem } from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Slider } from '@/components/ui/slider'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { toast } from '@/components/ui/use-toast'
import { formatDate } from '@/lib/utils'
import {
  TrendingUp,
  TrendingDown,
  AlertTriangle,
  FileText,
  Loader2,
  Calendar,
  Settings2,
  Lock,
  Unlock,
} from 'lucide-react'
// react-grid-layout v2.x - using legacy API for backwards compatibility
import { Responsive, WidthProvider, LayoutItem, Layout } from 'react-grid-layout/legacy'
import 'react-grid-layout/css/styles.css'
import 'react-resizable/css/styles.css'

// Create responsive width-aware layout component
const ResponsiveGridLayout = WidthProvider(Responsive)

// Responsive breakpoints and column configurations
const BREAKPOINTS = { lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }
const COLS = { lg: 12, md: 9, sm: 6, xs: 4, xxs: 2 }
import {
  LeaveBalanceWidget,
  MyProjectsWidget,
  AnimalsOnMedicationWidget,
  VetCommentsWidget,
  StaffAttendanceWidget,
  CalendarWidget,
  GoogleCalendarEventsWidget,
  WidgetLayoutItem,
  DEFAULT_DASHBOARD_LAYOUT,
  GRID_COLS,
  GRID_ROW_HEIGHT,
  widgetNames,
  widgetDescriptions,
  widgetPermissions,
  widgetCategories,
  widgetCategoryNames,
  widgetOptionsConfig,
} from '@/components/dashboard'

// 即將到期假期內容組件
interface BalanceSummaryData {
  expiring_soon_days: number
  expiring_soon_hours: number
}

function UpcomingLeavesContent() {
  const { data, isLoading, error } = useQuery({
    queryKey: ['hr-balance-summary-expiring'],
    queryFn: async () => {
      const res = await api.get<BalanceSummaryData>('/hr/balances/summary')
      return res.data
    },
  })

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-4">
        <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (error) {
    return <p className="text-sm text-muted-foreground">載入失敗</p>
  }

  const hasExpiring = (data?.expiring_soon_days ?? 0) > 0 || (data?.expiring_soon_hours ?? 0) > 0

  if (!hasExpiring) {
    return (
      <div className="flex flex-col items-center justify-center py-4 text-muted-foreground">
        <Calendar className="h-8 w-8 mb-2 text-green-500" />
        <p className="text-sm">30天內沒有即將到期的假期</p>
      </div>
    )
  }

  return (
    <div className="space-y-3">
      {(data?.expiring_soon_days ?? 0) > 0 && (
        <div className="flex justify-between items-center p-2 bg-orange-50 rounded-lg border border-orange-200">
          <span className="text-sm text-orange-700">即將到期（特休）</span>
          <div className="text-right">
            <span className="text-lg font-semibold text-orange-600">
              {data?.expiring_soon_days ?? 0}
            </span>
            <span className="text-sm text-orange-600 ml-1">天</span>
          </div>
        </div>
      )}
      {(data?.expiring_soon_hours ?? 0) > 0 && (
        <div className="flex justify-between items-center p-2 bg-orange-50 rounded-lg border border-orange-200">
          <span className="text-sm text-orange-700">即將到期（補休）</span>
          <div className="text-right">
            <span className="text-lg font-semibold text-orange-600">
              {data?.expiring_soon_hours ?? 0}
            </span>
            <span className="text-sm text-orange-600 ml-1">小時</span>
          </div>
        </div>
      )}
      <p className="text-xs text-muted-foreground text-center">30天內到期</p>
    </div>
  )
}

export function DashboardPage() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { user, hasRole, hasPermission } = useAuthStore()
  const [showSettingsDialog, setShowSettingsDialog] = useState(false)
  const [isEditMode, setIsEditMode] = useState(false)
  const [tempLayout, setTempLayout] = useState<WidgetLayoutItem[]>([])

  // 從後端取得 Widget 配置
  const { data: layoutData } = useQuery({
    queryKey: ['user-preferences', 'dashboard_widgets'],
    queryFn: async () => {
      const res = await api.get<{ key: string; value: WidgetLayoutItem[] }>('/me/preferences/dashboard_widgets')
      return res.data.value
    },
  })

  // 儲存 Widget 配置
  const saveLayoutMutation = useMutation({
    mutationFn: async (layout: WidgetLayoutItem[]) => {
      return api.put('/me/preferences/dashboard_widgets', { value: layout })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user-preferences', 'dashboard_widgets'] })
      toast({ title: '成功', description: '儀表板佈局已儲存' })
    },
  })

  // 合併後的佈局配置
  const currentLayout = useMemo(() => {
    return layoutData || DEFAULT_DASHBOARD_LAYOUT
  }, [layoutData])

  // 根據權限過濾可顯示的 Widget
  const availableWidgets = useMemo(() => {
    return currentLayout.filter((w) => {
      const permission = widgetPermissions[w.i]
      if (!permission) return true
      if (permission === 'erp') {
        return hasRole('admin') ||
          user?.roles.some(r => ['purchasing', 'approver', 'WAREHOUSE_MANAGER'].includes(r)) ||
          user?.permissions.some(p => p.startsWith('erp.'))
      }
      if (permission === 'admin') return hasRole('admin')
      return hasPermission(permission)
    })
  }, [currentLayout, hasRole, hasPermission, user])

  // 可見的 Widget
  const visibleWidgets = useMemo(() => {
    return availableWidgets.filter((w) => w.visible !== false)
  }, [availableWidgets])

  // 處理佈局變更
  const handleLayoutChange = (newLayout: LayoutItem[]) => {
    if (!isEditMode) return

    // 合併新佈局和現有的自訂屬性
    const updatedLayout = currentLayout.map(item => {
      const layoutItem = newLayout.find(l => l.i === item.i)
      if (layoutItem) {
        return {
          ...item,
          x: layoutItem.x,
          y: layoutItem.y,
          w: layoutItem.w,
          h: layoutItem.h,
        }
      }
      return item
    })

    saveLayoutMutation.mutate(updatedLayout)
  }

  // 開啟設定對話框
  const openSettings = () => {
    setTempLayout([...currentLayout])
    setShowSettingsDialog(true)
  }

  // 儲存設定
  const handleSaveSettings = () => {
    saveLayoutMutation.mutate(tempLayout)
    setShowSettingsDialog(false)
  }

  // 切換 Widget 顯示狀態
  const toggleWidgetVisibility = (widgetId: string) => {
    setTempLayout((prev) =>
      prev.map((w) =>
        w.i === widgetId ? { ...w, visible: !w.visible } : w
      )
    )
  }

  // 變更 Widget 選項
  const changeWidgetOption = (widgetId: string, key: string, value: number) => {
    setTempLayout((prev) =>
      prev.map((w) =>
        w.i === widgetId ? { ...w, options: { ...w.options, [key]: value } } : w
      )
    )
  }

  // ERP 相關查詢
  const { data: lowStockAlerts, isLoading: loadingAlerts } = useQuery({
    queryKey: ['low-stock-alerts'],
    queryFn: async () => {
      const response = await api.get<LowStockAlert[]>('/inventory/low-stock')
      return response.data
    },
  })

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
      TR: '調撥單',
      STK: '盤點單',
      ADJ: '調整單',
      RM: '退料單',
    }
    return names[type] || type
  }

  // 產生趨勢資料函數
  const getTrendData = (days: number = 7) => {
    if (!recentDocuments) return []
    const today = new Date()
    const result: { date: string; dateStr: string; inbound: number; outbound: number }[] = []
    for (let i = days - 1; i >= 0; i--) {
      const date = new Date(today)
      date.setDate(date.getDate() - i)
      const dateStr = date.toISOString().split('T')[0]
      const displayDate = date.toLocaleDateString('zh-TW', { month: 'short', day: 'numeric' })
      const dayDocs = recentDocuments.filter(
        (d) => d.status === 'approved' && d.approved_at?.startsWith(dateStr)
      )
      const inbound = dayDocs.filter((d) => ['GRN'].includes(d.doc_type)).length
      const outbound = dayDocs.filter((d) => ['DO', 'PR'].includes(d.doc_type)).length
      result.push({ date: dateStr, dateStr: displayDate, inbound, outbound })
    }
    return result
  }

  // Widget 渲染函數
  const renderWidget = (widgetItem: WidgetLayoutItem) => {
    const widgetId = widgetItem.i
    switch (widgetId) {
      case 'calendar_widget':
        return <CalendarWidget />
      case 'leave_balance':
        return <LeaveBalanceWidget />
      case 'my_projects':
        return <MyProjectsWidget />
      case 'animals_on_medication':
        return <AnimalsOnMedicationWidget />
      case 'vet_comments':
        return <VetCommentsWidget />
      case 'staff_attendance':
        return <StaffAttendanceWidget />
      case 'google_calendar_events':
        return <GoogleCalendarEventsWidget />
      case 'low_stock_alert':
        return (
          <Card className="h-full overflow-auto">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">低庫存警示</CardTitle>
              <AlertTriangle className="h-4 w-4 text-yellow-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {loadingAlerts ? '-' : lowStockAlerts?.length || 0}
              </div>
              <p className="text-xs text-muted-foreground">需要補貨的品項</p>
            </CardContent>
          </Card>
        )
      case 'pending_documents':
        return (
          <Card className="h-full overflow-auto">
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
              <p className="text-xs text-muted-foreground">等待核准的單據</p>
            </CardContent>
          </Card>
        )
      case 'today_inbound':
        return (
          <Card className="h-full overflow-auto">
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
                      ['GRN'].includes(d.doc_type) &&
                      d.status === 'approved' &&
                      new Date(d.approved_at || '').toDateString() === new Date().toDateString()
                  ).length || 0}
              </div>
              <p className="text-xs text-muted-foreground">入庫單據數量</p>
            </CardContent>
          </Card>
        )
      case 'today_outbound':
        return (
          <Card className="h-full overflow-auto">
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
              <p className="text-xs text-muted-foreground">出庫單據數量</p>
            </CardContent>
          </Card>
        )
      case 'weekly_trend': {
        const days = widgetItem.options?.days || 7
        const trendData = getTrendData(days)
        return (
          <Card className="h-full overflow-auto">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Calendar className="h-5 w-5 text-indigo-500" />
                近 {days} 天出入庫趨勢
              </CardTitle>
              <CardDescription>最近 {days} 天的出入庫單據統計</CardDescription>
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
                      <TableHead className="text-right">入庫</TableHead>
                      <TableHead className="text-right">出庫</TableHead>
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
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        )
      }
      case 'recent_documents':
        return (
          <Card className="h-full overflow-auto">
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
                      <TableRow
                        key={doc.id}
                        className="cursor-pointer hover:bg-muted/50 transition-colors"
                        onClick={() => navigate(`/documents/${doc.id}`)}
                      >
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
        )
      case 'upcoming_leaves':
        return (
          <Card className="h-full overflow-auto">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium flex items-center gap-2">
                <Calendar className="h-4 w-4 text-orange-500" />
                即將到期假期
              </CardTitle>
              <CardDescription>請記得使用即將過期的假期</CardDescription>
            </CardHeader>
            <CardContent>
              <UpcomingLeavesContent />
            </CardContent>
          </Card>
        )
      default:
        return null
    }
  }

  // 轉換為 react-grid-layout 需要的格式
  const baseLayout: LayoutItem[] = visibleWidgets.map(w => ({
    i: w.i,
    x: w.x,
    y: w.y,
    w: w.w,
    h: w.h,
    minW: w.minW,
    minH: w.minH,
    maxW: w.maxW,
    maxH: w.maxH,
  }))

  // 生成響應式佈局
  const generateResponsiveLayouts = (base: LayoutItem[]) => {
    // 大螢幕: 使用原始佈局
    const lgLayout = base

    // 中螢幕 (9列): 調整寬度
    const mdLayout = base.map(item => ({
      ...item,
      w: Math.min(item.w, 9),
      x: Math.min(item.x, 9 - Math.min(item.w, 9)),
    }))

    // 小螢幕 (6列): 重新排列為較窄佈局
    const smLayout = base.map((item, idx) => ({
      ...item,
      x: (idx % 2) * 3,
      y: Math.floor(idx / 2) * 4,
      w: 3,
      h: 4,
    }))

    // 超小螢幕 (4列): 2x2 網格
    const xsLayout = base.map((item, idx) => ({
      ...item,
      x: (idx % 2) * 2,
      y: Math.floor(idx / 2) * 4,
      w: 2,
      h: 4,
    }))

    // 最小螢幕 (2列): 單列佈局
    const xxsLayout = base.map((item, idx) => ({
      ...item,
      x: 0,
      y: idx * 4,
      w: 2,
      h: 4,
    }))

    return { lg: lgLayout, md: mdLayout, sm: smLayout, xs: xsLayout, xxs: xxsLayout }
  }

  const responsiveLayouts = generateResponsiveLayouts(baseLayout)

  return (
    <div className="space-y-6">
      {/* Page header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">儀表板</h1>
        </div>
        <div className="flex gap-2">
          {isEditMode ? (
            <>
              <Button variant="outline" size="sm" onClick={() => setIsEditMode(false)}>
                <Lock className="h-4 w-4 mr-1" />
                鎖定佈局
              </Button>
            </>
          ) : (
            <>
              <Button variant="outline" size="sm" onClick={() => setIsEditMode(true)}>
                <Unlock className="h-4 w-4 mr-1" />
                編輯佈局
              </Button>
              <Button variant="outline" size="sm" onClick={openSettings}>
                <Settings2 className="h-4 w-4 mr-1" />
                自訂儀表板
              </Button>
            </>
          )}
        </div>
      </div>

      {/* 編輯模式提示 */}
      {isEditMode && (
        <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg text-sm text-blue-700">
          拖曳 Widget 調整位置，拖曳右下角調整大小，完成後點選「鎖定佈局」
        </div>
      )}

      {/* Widget Grid */}
      <ResponsiveGridLayout
        className="layout"
        layouts={responsiveLayouts}
        breakpoints={BREAKPOINTS}
        cols={COLS}
        rowHeight={GRID_ROW_HEIGHT}
        onLayoutChange={(currentLayout) => handleLayoutChange([...currentLayout] as LayoutItem[])}
        isDraggable={isEditMode}
        isResizable={isEditMode}
        margin={[16, 16]}
        containerPadding={[0, 0]}
        useCSSTransforms={true}
      >
        {visibleWidgets.map((widget) => (
          <div
            key={widget.i}
            className={`h-full overflow-hidden ${isEditMode ? 'ring-2 ring-blue-300 ring-offset-2 rounded-lg' : ''}`}
          >
            {renderWidget(widget)}
          </div>
        ))}
      </ResponsiveGridLayout>

      {/* 設定對話框 */}
      <Dialog open={showSettingsDialog} onOpenChange={setShowSettingsDialog}>
        <DialogContent className="max-w-lg max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Settings2 className="h-5 w-5" />
              自訂儀表板
            </DialogTitle>
            <DialogDescription>
              選擇要顯示的 Widget，可自由開啟或關閉
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            {Object.entries(widgetCategoryNames).map(([categoryId, categoryName]) => {
              const categoryWidgets = tempLayout.filter(
                (w) => widgetCategories[w.i] === categoryId && !widgetPermissions[w.i] ||
                  widgetCategories[w.i] === categoryId && availableWidgets.some(aw => aw.i === w.i)
              )
              if (categoryWidgets.length === 0) return null
              return (
                <div key={categoryId}>
                  <h4 className="text-sm font-medium text-muted-foreground mb-2">{categoryName}</h4>
                  <div className="space-y-2">
                    {categoryWidgets.map((widget) => (
                      <div
                        key={widget.i}
                        className="p-3 border rounded-lg hover:bg-muted/50 space-y-3"
                      >
                        <div className="flex items-start gap-3">
                          <Checkbox
                            id={widget.i}
                            checked={widget.visible !== false}
                            onCheckedChange={() => toggleWidgetVisibility(widget.i)}
                          />
                          <label htmlFor={widget.i} className="flex-1 cursor-pointer">
                            <p className="text-sm font-medium">{widgetNames[widget.i]}</p>
                            <p className="text-xs text-muted-foreground">
                              {widgetDescriptions[widget.i]}
                            </p>
                          </label>
                        </div>
                        {/* weekly_trend 天數設定 */}
                        {widget.visible !== false && widget.i === 'weekly_trend' && (
                          <div className="flex items-center gap-2 ml-6">
                            <span className="text-xs text-muted-foreground">天數：</span>
                            <Slider
                              value={widget.options?.days || 7}
                              min={3}
                              max={7}
                              step={1}
                              quickValues={[3, 5, 7]}
                              onChange={(value: number) => changeWidgetOption(widget.i, 'days', value)}
                              className="w-48"
                            />
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )
            })}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowSettingsDialog(false)}>
              取消
            </Button>
            <Button onClick={handleSaveSettings} disabled={saveLayoutMutation.isPending}>
              {saveLayoutMutation.isPending && (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              )}
              儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
