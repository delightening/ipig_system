import { useMemo, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { LowStockAlert, DocumentListItem } from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
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
import { formatNumber, formatDate } from '@/lib/utils'
import {
  Package,
  TrendingUp,
  TrendingDown,
  AlertTriangle,
  FileText,
  Loader2,
  Calendar,
  Settings2,
  GripVertical,
} from 'lucide-react'
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
} from '@dnd-kit/core'
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  rectSortingStrategy,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import {
  LeaveBalanceWidget,
  MyProjectsWidget,
  AnimalsOnMedicationWidget,
  VetCommentsWidget,
  StaffAttendanceWidget,
  CalendarWidget,
  DashboardWidgetConfig,
  DEFAULT_DASHBOARD_WIDGETS,
  widgetNames,
  widgetDescriptions,
  widgetPermissions,
  widgetCategories,
  widgetCategoryNames,
} from '@/components/dashboard'

// 可排序的 Widget 容器
function SortableWidgetContainer({
  id,
  children,
  isEditMode,
}: {
  id: string
  children: React.ReactNode
  isEditMode: boolean
}) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  }

  return (
    <div ref={setNodeRef} style={style} className="relative">
      {isEditMode && (
        <button
          {...attributes}
          {...listeners}
          className="absolute -top-2 -left-2 z-10 p-1 bg-slate-800 text-white rounded-full cursor-grab active:cursor-grabbing shadow-lg"
        >
          <GripVertical className="h-4 w-4" />
        </button>
      )}
      {children}
    </div>
  )
}

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
  const [tempWidgetConfig, setTempWidgetConfig] = useState<DashboardWidgetConfig[]>([])

  // dnd-kit sensors
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  )

  // 從後端取得 Widget 配置
  const { data: widgetConfigData } = useQuery({
    queryKey: ['user-preferences', 'dashboard_widgets'],
    queryFn: async () => {
      const res = await api.get<{ key: string; value: DashboardWidgetConfig[] }>('/me/preferences/dashboard_widgets')
      return res.data.value
    },
  })

  // 儲存 Widget 配置
  const saveWidgetConfigMutation = useMutation({
    mutationFn: async (config: DashboardWidgetConfig[]) => {
      return api.put('/me/preferences/dashboard_widgets', { value: config })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user-preferences', 'dashboard_widgets'] })
      toast({ title: '成功', description: '儀表板設定已儲存' })
    },
  })

  // 合併後的 Widget 配置
  const widgetConfig = useMemo(() => {
    const config = widgetConfigData || DEFAULT_DASHBOARD_WIDGETS
    return config.sort((a, b) => a.order - b.order)
  }, [widgetConfigData])

  // 根據權限過濾可顯示的 Widget
  const availableWidgets = useMemo(() => {
    return widgetConfig.filter((w) => {
      const permission = widgetPermissions[w.id]
      if (!permission) return true
      if (permission === 'erp') {
        return hasRole('admin') ||
          user?.roles.some(r => ['purchasing', 'approver', 'WAREHOUSE_MANAGER'].includes(r)) ||
          user?.permissions.some(p => p.startsWith('erp.'))
      }
      if (permission === 'admin') return hasRole('admin')
      return hasPermission(permission)
    })
  }, [widgetConfig, hasRole, hasPermission, user])

  // 可見的 Widget
  const visibleWidgets = useMemo(() => {
    return availableWidgets.filter((w) => w.visible)
  }, [availableWidgets])

  // 處理拖曳結束
  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    if (over && active.id !== over.id) {
      const oldIndex = visibleWidgets.findIndex((w) => w.id === active.id)
      const newIndex = visibleWidgets.findIndex((w) => w.id === over.id)
      const newOrder = arrayMove(visibleWidgets, oldIndex, newIndex)

      // 更新整體配置的 order
      const newConfig = widgetConfig.map((w) => {
        const newOrderIndex = newOrder.findIndex((nw) => nw.id === w.id)
        return { ...w, order: newOrderIndex >= 0 ? newOrderIndex : w.order }
      })

      saveWidgetConfigMutation.mutate(newConfig)
    }
  }

  // 開啟設定對話框
  const openSettings = () => {
    setTempWidgetConfig([...widgetConfig])
    setShowSettingsDialog(true)
  }

  // 儲存設定
  const handleSaveSettings = () => {
    saveWidgetConfigMutation.mutate(tempWidgetConfig)
    setShowSettingsDialog(false)
  }

  // 切換 Widget 顯示狀態
  const toggleWidgetVisibility = (widgetId: string) => {
    setTempWidgetConfig((prev) =>
      prev.map((w) =>
        w.id === widgetId ? { ...w, visible: !w.visible } : w
      )
    )
  }

  // ERP 相關查詢（保留原有功能）
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
      const inbound = dayDocs.filter((d) => ['GRN'].includes(d.doc_type)).length
      const outbound = dayDocs.filter((d) => ['DO', 'PR'].includes(d.doc_type)).length
      days.push({ date: dateStr, dateStr: displayDate, inbound, outbound })
    }
    return days
  }, [recentDocuments])

  // Widget 渲染函數
  const renderWidget = (widgetId: string) => {
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
      case 'low_stock_alert':
        return (
          <Card>
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
              <p className="text-xs text-muted-foreground">等待核准的單據</p>
            </CardContent>
          </Card>
        )
      case 'today_inbound':
        return (
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
              <p className="text-xs text-muted-foreground">出庫單據數量</p>
            </CardContent>
          </Card>
        )
      case 'weekly_trend':
        return (
          <Card className="col-span-2">
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
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        )
      case 'recent_documents':
        return (
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
      case 'upcoming_leaves': {
        // 使用 LeaveBalanceWidget 的資料邏輯
        return (
          <Card>
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
      }
      default:
        return null
    }
  }

  return (
    <div className="space-y-6">
      {/* Page header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">儀表板</h1>
        </div>
        <div className="flex gap-2">
          {isEditMode ? (
            <Button variant="outline" size="sm" onClick={() => setIsEditMode(false)}>
              完成
            </Button>
          ) : (
            <>
              <Button variant="outline" size="sm" onClick={() => setIsEditMode(true)}>
                <GripVertical className="h-4 w-4 mr-1" />
                調整順序
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
          拖曳 Widget 左上角的圖示來調整順序，完成後點選「完成」按鈕
        </div>
      )}

      {/* Widget Grid */}
      <DndContext
        sensors={sensors}
        collisionDetection={closestCenter}
        onDragEnd={handleDragEnd}
      >
        <SortableContext
          items={visibleWidgets.map((w) => w.id)}
          strategy={rectSortingStrategy}
        >
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
            {visibleWidgets.map((widget) => (
              <SortableWidgetContainer
                key={widget.id}
                id={widget.id}
                isEditMode={isEditMode}
              >
                {renderWidget(widget.id)}
              </SortableWidgetContainer>
            ))}
          </div>
        </SortableContext>
      </DndContext>

      {/* 設定對話框 */}
      <Dialog open={showSettingsDialog} onOpenChange={setShowSettingsDialog}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Settings2 className="h-5 w-5" />
              自訂儀表板
            </DialogTitle>
            <DialogDescription>
              選擇要顯示的 Widget，可自由開啟或關閉
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 max-h-[400px] overflow-y-auto">
            {Object.entries(widgetCategoryNames).map(([categoryId, categoryName]) => {
              const categoryWidgets = tempWidgetConfig.filter(
                (w) => widgetCategories[w.id] === categoryId && !widgetPermissions[w.id] ||
                  widgetCategories[w.id] === categoryId && availableWidgets.some(aw => aw.id === w.id)
              )
              if (categoryWidgets.length === 0) return null
              return (
                <div key={categoryId}>
                  <h4 className="text-sm font-medium text-muted-foreground mb-2">{categoryName}</h4>
                  <div className="space-y-2">
                    {categoryWidgets.map((widget) => (
                      <div
                        key={widget.id}
                        className="flex items-start gap-3 p-3 border rounded-lg hover:bg-muted/50"
                      >
                        <Checkbox
                          id={widget.id}
                          checked={widget.visible}
                          onCheckedChange={() => toggleWidgetVisibility(widget.id)}
                        />
                        <label htmlFor={widget.id} className="flex-1 cursor-pointer">
                          <p className="text-sm font-medium">{widgetNames[widget.id]}</p>
                          <p className="text-xs text-muted-foreground">
                            {widgetDescriptions[widget.id]}
                          </p>
                        </label>
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
            <Button onClick={handleSaveSettings} disabled={saveWidgetConfigMutation.isPending}>
              {saveWidgetConfigMutation.isPending && (
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
