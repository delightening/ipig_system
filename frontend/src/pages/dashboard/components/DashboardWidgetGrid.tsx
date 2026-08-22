import { useEffect, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Responsive, LayoutItem, Layout } from 'react-grid-layout/legacy'
import 'react-grid-layout/css/styles.css'
import 'react-resizable/css/styles.css'
import { X } from 'lucide-react'
import {
  WidgetLayoutItem,
  BreakpointKey,
  GRID_ROW_HEIGHT,
  GRID_BREAKPOINTS,
  GRID_COLS_BY_BREAKPOINT,
  breakpointFromWidth,
} from '@/components/dashboard'
import { useGridWidth } from '../hooks/useGridWidth'

interface DashboardWidgetGridProps {
  layouts: Record<string, LayoutItem[]>
  visibleWidgets: WidgetLayoutItem[]
  isEditMode: boolean
  onLayoutChange: (currentLayout: Layout, allLayouts: Partial<Record<string, Layout>>) => void
  onBreakpointChange: (breakpoint: BreakpointKey) => void
  onHideWidget: (widgetId: string) => void
  renderWidget: (widget: WidgetLayoutItem) => ReactNode
}

/**
 * 儀表板 widget 網格：封裝 react-grid-layout 的 ResponsiveGridLayout 與 widget 卡片渲染。
 * 編輯模式下啟用拖曳 / 縮放、widget 右上角顯示 ✕ 隱藏鈕；compactType="vertical" 自動向上壓實，
 * 拖曳 / 縮放 / 隱藏 widget 後其餘卡片自動上移補滿垂直空隙（react-grid-layout 單軸壓實，僅向上）。
 *
 * 寬度自行量測（useGridWidth）而非用官方 WidthProvider：後者固定從 1280px 起跑，
 * 在更寬的螢幕上會先畫一幀 lg 版面才跳到正確斷點（見 useGridWidth 註解）。
 */
export function DashboardWidgetGrid({
  layouts,
  visibleWidgets,
  isEditMode,
  onLayoutChange,
  onBreakpointChange,
  onHideWidget,
  renderWidget,
}: DashboardWidgetGridProps) {
  const { t } = useTranslation()
  const { width, containerRef } = useGridWidth()
  const breakpoint = width === null ? null : breakpointFromWidth(width)

  // 目前斷點改由自量寬度推導，不用 grid 的 onBreakpointChange：後者只在「斷點變動」時回呼，
  // 首幀寬度已正確時根本不會觸發，呼叫端會一直停在初始值 lg，導致拖曳座標存錯斷點。
  useEffect(() => {
    if (breakpoint) onBreakpointChange(breakpoint)
  }, [breakpoint, onBreakpointChange])

  // 首次渲染只放量尺 div：ref callback 會在 paint 前同步量到寬度並觸發重繪，
  // 故這個空 div 實際上不會被使用者看到。
  if (width === null) return <div ref={containerRef} />

  return (
    <Responsive
      innerRef={containerRef}
      width={width}
      className="layout"
      layouts={layouts}
      breakpoints={GRID_BREAKPOINTS}
      cols={GRID_COLS_BY_BREAKPOINT}
      rowHeight={GRID_ROW_HEIGHT}
      onLayoutChange={onLayoutChange}
      isDraggable={isEditMode}
      isResizable={isEditMode}
      compactType="vertical"
      preventCollision={false}
      draggableCancel=".widget-remove-btn"
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
          {isEditMode && (
            <button
              type="button"
              aria-label={t('dashboard.editMode.removeWidget')}
              title={t('dashboard.editMode.removeWidget')}
              className="widget-remove-btn absolute -right-2 -top-2 z-20 flex h-6 w-6 items-center justify-center rounded-full bg-destructive text-destructive-foreground shadow-md transition-colors hover:bg-destructive/90"
              onMouseDown={(e) => e.stopPropagation()}
              onClick={(e) => {
                e.stopPropagation()
                onHideWidget(widget.i)
              }}
            >
              <X className="h-3.5 w-3.5" />
            </button>
          )}
          <div className="h-full" style={isEditMode ? { pointerEvents: 'none' } : undefined}>
            {renderWidget(widget)}
          </div>
        </div>
      ))}
    </Responsive>
  )
}
