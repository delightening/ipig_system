import { useMemo, useState, useCallback } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, { deleteResource } from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import { logger } from '@/lib/logger'
import { Loader2, Settings2, Unlock, Save } from 'lucide-react'
import { Responsive, WidthProvider, LayoutItem, Layout } from 'react-grid-layout/legacy'
import 'react-grid-layout/css/styles.css'
import 'react-resizable/css/styles.css'

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
  GRID_ROW_HEIGHT,
  widgetPermissions,
} from '@/components/dashboard'
import { useDashboardData } from './dashboard/hooks/useDashboardData'
import {
  LowStockAlertWidget,
  PendingDocumentsWidget,
  TodayInboundWidget,
  TodayOutboundWidget,
  WeeklyTrendWidget,
  RecentDocumentsWidget,
  UpcomingLeavesWidget,
} from './dashboard/components/ErpWidgets'
import { DashboardSettingsDialog } from './dashboard/components/DashboardSettingsDialog'

const ResponsiveGridLayout = WidthProvider(Responsive)
const BREAKPOINTS = { lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }
const COLS = { lg: 12, md: 9, sm: 6, xs: 4, xxs: 2 }

export function DashboardPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { user, hasRole, hasPermission } = useAuthStore()
  const [showSettingsDialog, setShowSettingsDialog] = useState(false)
  const [isEditMode, setIsEditMode] = useState(false)
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)
  const [pendingLayout, setPendingLayout] = useState<WidgetLayoutItem[] | null>(null)

  // 從後端取得 Widget 配置
  const { data: layoutData } = useQuery({
    queryKey: ['user-preferences', 'dashboard_widgets'],
    queryFn: async () => {
      const res = await api.get<{ key: string; value: WidgetLayoutItem[] }>('/me/preferences/dashboard_widgets')
      return res.data.value
    },
    staleTime: 1_800_000,
  })

  const saveLayoutMutation = useMutation({
    mutationFn: async (layout: WidgetLayoutItem[]) => {
      return api.put('/me/preferences/dashboard_widgets', { value: layout })
    },
    onSuccess: (_data, variables) => {
      queryClient.setQueryData(['user-preferences', 'dashboard_widgets'], variables)
    },
    onError: (error) => {
      logger.error('儲存佈局失敗:', error)
      toast({ title: '錯誤', description: '儲存佈局失敗', variant: 'destructive' })
    },
  })

  const currentLayout = useMemo(() => layoutData || DEFAULT_DASHBOARD_LAYOUT, [layoutData])

  const hasErpPermission = useMemo(() => {
    return hasRole('admin') ||
      (user?.roles.some(r => ['purchasing', 'approver', 'WAREHOUSE_MANAGER'].includes(r)) ?? false) ||
      (user?.permissions.some(p => p.startsWith('erp.')) ?? false)
  }, [hasRole, user])

  const { lowStockAlerts, loadingAlerts, recentDocuments, loadingDocuments, todayApprovedDocs, getTrendData } =
    useDashboardData(hasErpPermission)

  const availableWidgets = useMemo(() => {
    return currentLayout.filter((w) => {
      const permission = widgetPermissions[w.i]
      if (!permission) return true
      if (permission === 'erp') return hasErpPermission
      if (permission === 'admin') return hasRole('admin')
      return hasPermission(permission)
    })
  }, [currentLayout, hasRole, hasPermission, user, hasErpPermission])

  const visibleWidgets = useMemo(() => {
    return availableWidgets.filter((w) => w.visible !== false)
  }, [availableWidgets])

  // 處理佈局變更
  const handleLayoutChange = useCallback((_currentLayout: Layout, allLayouts: Partial<Record<string, Layout>>) => {
    if (!isEditMode) return
    const lgLayout = allLayouts.lg || _currentLayout

    const updatedLayout = currentLayout.map(item => {
      const layoutItem = lgLayout.find(l => l.i === item.i)
      if (layoutItem) {
        return { ...item, x: layoutItem.x, y: layoutItem.y, w: layoutItem.w, h: layoutItem.h }
      }
      return item
    })

    setPendingLayout(updatedLayout)
    setHasUnsavedChanges(true)
  }, [isEditMode, currentLayout])

  const handleSaveLayout = () => {
    if (pendingLayout && hasUnsavedChanges) {
      saveLayoutMutation.mutate(pendingLayout, {
        onSuccess: () => {
          setIsEditMode(false)
          setHasUnsavedChanges(false)
          setPendingLayout(null)
          toast({ title: '成功', description: '佈局已儲存' })
        },
      })
    } else {
      setIsEditMode(false)
      setHasUnsavedChanges(false)
      setPendingLayout(null)
    }
  }

  const handleResetLayout = async () => {
    try {
      await deleteResource('/me/preferences/dashboard_widgets')
      queryClient.invalidateQueries({ queryKey: ['user-preferences', 'dashboard_widgets'] })
      setShowSettingsDialog(false)
      setIsEditMode(false)
      setHasUnsavedChanges(false)
      setPendingLayout(null)
      toast({ title: '成功', description: '佈局已重設為預設值' })
    } catch {
      toast({ title: '錯誤', description: '重設失敗', variant: 'destructive' })
    }
  }

  const handleSaveSettings = (layout: WidgetLayoutItem[]) => {
    saveLayoutMutation.mutate(layout)
    setShowSettingsDialog(false)
  }

  // Widget 渲染
  const renderWidget = (widgetItem: WidgetLayoutItem) => {
    const widgetId = widgetItem.i
    switch (widgetId) {
      case 'calendar_widget': return <CalendarWidget />
      case 'leave_balance': return <LeaveBalanceWidget />
      case 'my_projects': return <MyProjectsWidget />
      case 'animals_on_medication': return <AnimalsOnMedicationWidget />
      case 'vet_comments': return <VetCommentsWidget />
      case 'staff_attendance': return <StaffAttendanceWidget />
      case 'google_calendar_events': return <GoogleCalendarEventsWidget />
      case 'low_stock_alert':
        return <LowStockAlertWidget alerts={lowStockAlerts} isLoading={loadingAlerts} />
      case 'pending_documents':
        return <PendingDocumentsWidget documents={recentDocuments} isLoading={loadingDocuments} />
      case 'today_inbound':
        return <TodayInboundWidget todayApprovedDocs={todayApprovedDocs} isLoading={loadingDocuments} />
      case 'today_outbound':
        return <TodayOutboundWidget todayApprovedDocs={todayApprovedDocs} isLoading={loadingDocuments} />
      case 'weekly_trend': {
        const days = widgetItem.options?.days || 7
        return <WeeklyTrendWidget trendData={getTrendData(days)} days={days} isLoading={loadingDocuments} />
      }
      case 'recent_documents':
        return <RecentDocumentsWidget documents={recentDocuments} isLoading={loadingDocuments} />
      case 'upcoming_leaves':
        return <UpcomingLeavesWidget />
      default:
        return null
    }
  }

  // 轉換為 react-grid-layout 格式
  const baseLayout: LayoutItem[] = useMemo(() => visibleWidgets.map(w => ({
    i: w.i, x: w.x, y: w.y, w: w.w, h: w.h,
    minW: w.minW, minH: w.minH, maxW: w.maxW, maxH: w.maxH,
  })), [visibleWidgets])

  const responsiveLayouts = useMemo(() => {
    const lgLayout = baseLayout
    const mdLayout = baseLayout.map(item => ({
      ...item, w: Math.min(item.w, 9), x: Math.min(item.x, 9 - Math.min(item.w, 9)),
    }))
    const smLayout = baseLayout.map((item, idx) => ({
      ...item, x: (idx % 2) * 3, y: Math.floor(idx / 2) * 4, w: 3, h: 4,
    }))
    const xsLayout = baseLayout.map((item, idx) => ({
      ...item, x: (idx % 2) * 2, y: Math.floor(idx / 2) * 4, w: 2, h: 4,
    }))
    const xxsLayout = baseLayout.map((item, idx) => ({
      ...item, x: 0, y: idx * 4, w: 2, h: 4,
    }))
    return { lg: lgLayout, md: mdLayout, sm: smLayout, xs: xsLayout, xxs: xxsLayout }
  }, [baseLayout])

  return (
    <div className="space-y-6">
      {/* Page header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl md:text-3xl font-bold tracking-tight">{t('dashboard.title')}</h1>
        </div>
        <div className="flex flex-wrap gap-2">
          {isEditMode ? (
            <Button size="sm" onClick={handleSaveLayout} disabled={saveLayoutMutation.isPending}>
              {saveLayoutMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />}
              <Save className="h-4 w-4 mr-1" />
              {t('dashboard.editMode.lock')}
            </Button>
          ) : (
            <>
              <Button variant="outline" size="sm" onClick={() => setIsEditMode(true)}>
                <Unlock className="h-4 w-4 mr-1" />
                {t('dashboard.editMode.unlock')}
              </Button>
              <Button variant="outline" size="sm" onClick={handleResetLayout}>
                重設佈局
              </Button>
              <Button variant="outline" size="sm" onClick={() => setShowSettingsDialog(true)}>
                <Settings2 className="h-4 w-4 mr-1" />
                {t('dashboard.settings.title')}
              </Button>
            </>
          )}
        </div>
      </div>

      {isEditMode && (
        <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg text-sm text-blue-700">
          {t('dashboard.editMode.hint')}
        </div>
      )}

      {/* Widget Grid */}
      <ResponsiveGridLayout
        className="layout"
        layouts={responsiveLayouts}
        breakpoints={BREAKPOINTS}
        cols={COLS}
        rowHeight={GRID_ROW_HEIGHT}
        onLayoutChange={handleLayoutChange}
        isDraggable={isEditMode}
        isResizable={isEditMode}
        margin={[16, 16]}
        containerPadding={[0, 0]}
        useCSSTransforms={true}
      >
        {visibleWidgets.map((widget) => (
          <div
            key={widget.i}
            className="h-full"
            style={isEditMode ? {
              boxShadow: '0 0 0 2px rgb(147, 197, 253), 0 0 0 4px rgba(147, 197, 253, 0.3)',
              borderRadius: '0.5rem',
            } : undefined}
          >
            {renderWidget(widget)}
          </div>
        ))}
      </ResponsiveGridLayout>

      {/* 設定對話框 */}
      <DashboardSettingsDialog
        open={showSettingsDialog}
        onOpenChange={setShowSettingsDialog}
        currentLayout={currentLayout}
        availableWidgets={availableWidgets}
        onSave={handleSaveSettings}
        onReset={handleResetLayout}
        isSaving={saveLayoutMutation.isPending}
      />
    </div>
  )
}
