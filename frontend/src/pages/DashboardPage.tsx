import { useMemo, useState, useCallback, useRef } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { LayoutItem, Layout } from 'react-grid-layout/legacy'
import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { PageHeader } from '@/components/ui/page-header'
import { toast } from '@/components/ui/use-toast'
import { logger } from '@/lib/logger'

import {
  LeaveBalanceWidget,
  MyProjectsWidget,
  AnimalsOnMedicationWidget,
  VetCommentsWidget,
  StaffAttendanceWidget,
  CalendarWidget,
  QuickActionsWidget,
  GoogleCalendarEventsWidget,
  RoleWelcomeGuide,
  WidgetLayoutItem,
  BreakpointKey,
  BreakpointLayouts,
  GridPos,
  DashboardWidgetsPref,
  normalizeDashboardPref,
  DEFAULT_DASHBOARD_LAYOUT,
  GRID_COLS,
  widgetPermissions,
  packLayoutCompact,
  buildResponsiveLayouts,
} from '@/components/dashboard'
import { useWelcomeGuidePref } from '@/hooks/useWelcomeGuidePref'
import { useDashboardData } from './dashboard/hooks/useDashboardData'
import {
  LowStockAlertWidget,
  ExpiryAlertWidget,
  PendingDocumentsWidget,
  TodayInboundWidget,
  TodayOutboundWidget,
  WeeklyTrendWidget,
  RecentDocumentsWidget,
  RecentMaintenanceWidget,
  EquipmentStatusWidget,
  UpcomingLeavesWidget,
} from './dashboard/components/ErpWidgets'
import { DashboardSettingsDialog } from './dashboard/components/DashboardSettingsDialog'
import { DashboardHeaderActions } from './dashboard/components/DashboardHeaderActions'
import { DashboardEditHint } from './dashboard/components/DashboardEditHint'
import { DashboardWidgetGrid } from './dashboard/components/DashboardWidgetGrid'
import { DashboardWidgetGridSkeleton } from './dashboard/components/DashboardWidgetGridSkeleton'

export function DashboardPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  // R34-12: hasErpPermission 改用 store selector — 只在 user.roles / user.permissions 真的變動時
  // 重算，避免任意 user 欄位（如 last_login_at refresh）觸發 dashboard 全 re-render
  const hasRole = useAuthStore((s) => s.hasRole)
  const hasPermission = useAuthStore((s) => s.hasPermission)
  const hasErpPermission = useAuthStore((s) => {
    if (s.hasRole('admin')) return true
    const u = s.user
    if (!u) return false
    // coderabbit-fix: 涵蓋精確 'erp' 權限（widgetPermissions 用 'erp' 作 key）
    if (u.permissions.includes('erp')) return true
    if (u.roles.some((r) => ['purchasing', 'approver', 'WAREHOUSE_MANAGER'].includes(r))) return true
    if (u.permissions.some((p) => p.startsWith('erp.'))) return true
    return false
  })
  const [showSettingsDialog, setShowSettingsDialog] = useState(false)
  const [isEditMode, setIsEditMode] = useState(false)
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)
  // 編輯暫存：pendingLayout＝widget 集（顯示/隱藏 + lg 座標）；pendingByBreakpoint＝各斷點座標 override。
  const [pendingLayout, setPendingLayout] = useState<WidgetLayoutItem[] | null>(null)
  const [pendingByBreakpoint, setPendingByBreakpoint] = useState<BreakpointLayouts | null>(null)
  // 目前作用中的響應式斷點（由 grid onBreakpointChange 更新），決定拖曳要存到哪個斷點。
  // 用 ref 而非 state：(1) handleLayoutChange 需同步讀取，避免斷點切換與佈局變更同一 render
  // tick 觸發時 closure 讀到過期斷點（把新斷點座標誤存到舊斷點）；(2) 渲染與 memo 皆不依賴此值
  // （responsiveLayouts 只看 baseLayout / workingByBreakpoint），故無需 state 觸發 re-render。
  const currentBreakpointRef = useRef<BreakpointKey>('lg')

  // 從後端取得 Widget 配置（正規化為 v2：相容舊的純陣列格式）
  // isPending 期間不渲染網格（改顯示骨架）：否則會先用 DEFAULT_DASHBOARD_LAYOUT 畫一次，
  // 偏好回來後再跳到使用者存檔的位置 —— 就是登入時看到的「版面瞬間重排」。
  const { data: prefData, isPending: isPrefPending } = useQuery({
    queryKey: ['user-preferences', 'dashboard_widgets'],
    queryFn: async () => {
      const res = await api.get<{ key: string; value: unknown }>('/me/preferences/dashboard_widgets')
      return normalizeDashboardPref(res.data.value)
    },
    staleTime: 1_800_000,
  })
  const layoutData = prefData?.widgets
  const savedByBreakpoint = prefData?.byBreakpoint

  // 歡迎橫幅的顯示與否也是一支偏好，且橫幅位在網格上方 —— 它晚一步定案就會把整個網格推移。
  // 兩支偏好都到齊才畫定版面：任一未到就維持骨架、橫幅不渲染，全頁只繪製一次。
  const { isPending: isWelcomePrefPending } = useWelcomeGuidePref()
  const isLayoutPending = isPrefPending || isWelcomePrefPending

  const saveLayoutMutation = useMutation({
    mutationFn: async (pref: DashboardWidgetsPref) => {
      return api.put('/me/preferences/dashboard_widgets', { value: pref })
    },
    onSuccess: (_data, pref) => {
      queryClient.setQueryData(['user-preferences', 'dashboard_widgets'], pref)
    },
    onError: (error) => {
      logger.error('儲存佈局失敗:', error)
      toast({ title: '錯誤', description: '儲存佈局失敗', variant: 'destructive' })
    },
  })

  // 將預設佈局中新增的 widget 自動合併到已儲存的佈局
  const currentLayout = useMemo(() => {
    if (!layoutData) return DEFAULT_DASHBOARD_LAYOUT
    const savedIds = new Set(layoutData.map((w) => w.i))
    const newWidgets = DEFAULT_DASHBOARD_LAYOUT.filter((w) => !savedIds.has(w.i))
    if (newWidgets.length === 0) return layoutData
    return [...layoutData, ...newWidgets]
  }, [layoutData])

  // 編輯中以 pendingLayout 為準（拖曳 / 隱藏暫存變更即時反映），否則用伺服器存檔的 currentLayout。
  const workingLayout = useMemo(() => pendingLayout ?? currentLayout, [pendingLayout, currentLayout])
  // 各斷點座標 override：編輯中以 pending 為準，否則用伺服器存檔值。
  const workingByBreakpoint = useMemo(
    () => pendingByBreakpoint ?? savedByBreakpoint ?? {},
    [pendingByBreakpoint, savedByBreakpoint],
  )

  const { lowStockAlerts, loadingAlerts, expiryAlerts, expiryAlertsTotal, loadingExpiryAlerts, recentDocuments, loadingDocuments, recentMaintenance, loadingMaintenance, equipmentStats, loadingEquipment, todayApprovedDocs, getTrendData } =
    useDashboardData(hasErpPermission)

  // 單一 widget 的權限判定（'erp' / 'admin' 為特例，其餘走一般 hasPermission）。
  // 陣列採 OR 語意：持有其中任一權限即可顯示（對齊後端 OR 授權的端點）。
  const isWidgetPermitted = useCallback((widgetId: string) => {
    const permission = widgetPermissions[widgetId]
    if (!permission) return true
    const hasWidgetPermission = (p: string) =>
      p === 'erp' ? hasErpPermission : p === 'admin' ? hasRole('admin') : hasPermission(p)
    return Array.isArray(permission) ? permission.some(hasWidgetPermission) : hasWidgetPermission(permission)
  }, [hasErpPermission, hasRole, hasPermission])

  const availableWidgets = useMemo(
    () => workingLayout.filter((w) => isWidgetPermitted(w.i)),
    [workingLayout, isWidgetPermitted],
  )

  const visibleWidgets = useMemo(() => {
    return availableWidgets.filter((w) => w.visible !== false)
  }, [availableWidgets])

  // 處理佈局變更（拖曳 / 縮放）。依目前作用斷點分流：
  // - lg（桌機 base）：寫回 pendingLayout 的 widget 座標（沿用既有疊加邏輯，保留先前隱藏變更）。
  // - 其餘斷點：寫進 pendingByBreakpoint[bp] 的座標 override，各斷點獨立一份排版。
  const handleLayoutChange = useCallback((activeLayout: Layout, allLayouts: Partial<Record<string, Layout>>) => {
    if (!isEditMode) return
    const activeBreakpoint = currentBreakpointRef.current
    if (activeBreakpoint === 'lg') {
      const lgLayout = allLayouts.lg
      if (!lgLayout) return
      const lgLayoutMap = new Map(lgLayout.map(l => [l.i, l]))
      setPendingLayout((prev) => {
        const base = prev ?? currentLayout
        return base.map(item => {
          const layoutItem = lgLayoutMap.get(item.i)
          if (layoutItem) {
            return { ...item, x: layoutItem.x, y: layoutItem.y, w: layoutItem.w, h: layoutItem.h }
          }
          return item
        })
      })
    } else {
      const positions: Record<string, GridPos> = {}
      for (const l of activeLayout) positions[l.i] = { x: l.x, y: l.y, w: l.w, h: l.h }
      setPendingByBreakpoint((prev) => ({ ...(prev ?? savedByBreakpoint ?? {}), [activeBreakpoint]: positions }))
    }
    setHasUnsavedChanges(true)
  }, [isEditMode, currentLayout, savedByBreakpoint])

  // 編輯模式下點 widget 右上角 ✕：將該 widget 設為隱藏（visible=false，可從「自訂儀表板」還原）。
  // 不從佈局陣列移除座標（保留還原時的原始位置）；隱藏後 visibleWidgets 少一個，grid
  // compactType="vertical" 會自動把其餘卡片向上壓實補滿空位，並經 onLayoutChange 回寫新座標。
  const handleHideWidget = useCallback((widgetId: string) => {
    setPendingLayout((prev) => {
      const base = prev ?? currentLayout
      return base.map((w) => (w.i === widgetId ? { ...w, visible: false } : w))
    })
    setHasUnsavedChanges(true)
  }, [currentLayout])

  // 由 grid 回報目前作用中的斷點。identity 穩定（[] deps）：grid 內部以 effect 呼叫，
  // 若每次 render 換新 function 會讓該 effect 每次重跑。
  const handleBreakpointChange = useCallback((bp: BreakpointKey) => {
    currentBreakpointRef.current = bp
  }, [])

  const exitEditMode = useCallback(() => {
    setIsEditMode(false)
    setHasUnsavedChanges(false)
    setPendingLayout(null)
    setPendingByBreakpoint(null)
  }, [])

  const handleSaveLayout = () => {
    if ((pendingLayout || pendingByBreakpoint) && hasUnsavedChanges) {
      const pref: DashboardWidgetsPref = {
        v: 2,
        widgets: pendingLayout ?? currentLayout,
        byBreakpoint: pendingByBreakpoint ?? savedByBreakpoint ?? {},
      }
      saveLayoutMutation.mutate(pref, {
        onSuccess: () => {
          exitEditMode()
          toast({ title: '成功', description: '佈局已儲存' })
        },
      })
    } else {
      exitEditMode()
    }
  }

  // 「重設佈局」= 完全重設：恢復預設顯示/隱藏，並對「此使用者有權限且預設顯示」的 widget
  // 重新最佳化排版（方法 A：保序 + 壓實）。只打包該使用者實際看得到的子集，故自動適應不同
  // 權限的使用者（5 個就排 5 個、17 個就排 17 個），不會留下其他人 widget 的空洞。
  const handleResetLayout = () => {
    const permitted = DEFAULT_DASHBOARD_LAYOUT.filter((w) => isWidgetPermitted(w.i))
    const visible = permitted.filter((w) => w.visible !== false)
    const hidden = permitted.filter((w) => w.visible === false)
    const resetLayout = [...packLayoutCompact(visible, GRID_COLS), ...hidden]
    // 重設＝完全回預設，連各斷點 override 一併清空（byBreakpoint: {}）。
    saveLayoutMutation.mutate({ v: 2, widgets: resetLayout, byBreakpoint: {} }, {
      onSuccess: () => {
        setShowSettingsDialog(false)
        exitEditMode()
        toast({ title: '成功', description: '佈局已重設為預設值' })
      },
    })
  }

  const handleSaveSettings = (layout: WidgetLayoutItem[]) => {
    // 設定對話框只改 widget 集（顯示/隱藏/選項）；保留既有各斷點 override。
    saveLayoutMutation.mutate({ v: 2, widgets: layout, byBreakpoint: savedByBreakpoint ?? {} })
    setShowSettingsDialog(false)
  }

  // Widget 渲染
  const renderWidget = (widgetItem: WidgetLayoutItem) => {
    const widgetId = widgetItem.i
    switch (widgetId) {
      case 'quick_actions': return <QuickActionsWidget shortcuts={widgetItem.options?.shortcuts} />
      case 'calendar_widget': return <CalendarWidget />
      case 'leave_balance': return <LeaveBalanceWidget />
      case 'my_projects': return <MyProjectsWidget />
      case 'animals_on_medication': return <AnimalsOnMedicationWidget />
      case 'vet_comments': return <VetCommentsWidget />
      case 'staff_attendance': return <StaffAttendanceWidget />
      case 'google_calendar_events': return <GoogleCalendarEventsWidget />
      case 'low_stock_alert':
        return <LowStockAlertWidget alerts={lowStockAlerts} isLoading={loadingAlerts} />
      case 'expiry_alert':
        return <ExpiryAlertWidget alerts={expiryAlerts} total={expiryAlertsTotal} isLoading={loadingExpiryAlerts} />
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
      case 'recent_maintenance':
        return <RecentMaintenanceWidget records={recentMaintenance} isLoading={loadingMaintenance} />
      case 'equipment_status':
        return <EquipmentStatusWidget stats={equipmentStats} isLoading={loadingEquipment} />
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

  // 各斷點佈局：有 override 的斷點用存檔座標、手機單欄堆疊、其餘 seed 自最近已存斷點（見 widgetConfig）。
  const responsiveLayouts = useMemo(
    () => buildResponsiveLayouts(baseLayout, workingByBreakpoint),
    [baseLayout, workingByBreakpoint],
  )

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('dashboard.title')}
        actions={
          <DashboardHeaderActions
            isEditMode={isEditMode}
            isSaving={saveLayoutMutation.isPending}
            onSaveLayout={handleSaveLayout}
            onEnterEditMode={() => setIsEditMode(true)}
            onResetLayout={handleResetLayout}
            onOpenSettings={() => setShowSettingsDialog(true)}
          />
        }
      />

      {!isEditMode && !isLayoutPending && <RoleWelcomeGuide />}
      {isEditMode && <DashboardEditHint />}

      {isLayoutPending ? (
        <DashboardWidgetGridSkeleton />
      ) : (
        <DashboardWidgetGrid
          layouts={responsiveLayouts}
          visibleWidgets={visibleWidgets}
          isEditMode={isEditMode}
          onLayoutChange={handleLayoutChange}
          onBreakpointChange={handleBreakpointChange}
          onHideWidget={handleHideWidget}
          renderWidget={renderWidget}
        />
      )}

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
