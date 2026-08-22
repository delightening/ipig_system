import { useMemo, useRef, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { facilityApi } from '@/lib/api/facility'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { EmptyState } from '@/components/ui/empty-state'
import { cn } from '@/lib/utils'
import { ZoomIn, ZoomOut, RotateCcw, Network, UserCog } from 'lucide-react'
import type { DepartmentWithManager, DepartmentMember } from '@/types/facility'

const ZOOM_MIN = 0.4
const ZOOM_MAX = 2
const ZOOM_STEP = 0.2

interface ChartData {
  roots: DepartmentWithManager[]
  childrenByParent: Map<string, DepartmentWithManager[]>
  membersByDept: Map<string, DepartmentMember[]>
}

/** 單一部門節點卡片：名稱 + 代碼 + 主管 + 全部成員。 */
function DeptCard({ dept, members }: { dept: DepartmentWithManager; members: DepartmentMember[] }) {
  const { t } = useTranslation()
  return (
    <div className="min-w-[11rem] max-w-[16rem] rounded-lg border bg-card px-3 py-2 shadow-sm">
      <div className="text-sm font-medium">{dept.name}</div>
      <div className="font-mono text-xs text-muted-foreground">{dept.code}</div>
      <div className="mt-1 flex items-center gap-1 text-xs">
        <UserCog className="h-3.5 w-3.5 text-primary" />
        <span className="text-muted-foreground">{t('admin.departmentTab.orgChart.manager')}：</span>
        <span>{dept.manager_name ?? '—'}</span>
      </div>
      <div className="mt-2 border-t pt-1.5">
        {members.length === 0 ? (
          <div className="text-xs text-muted-foreground">{t('admin.departmentTab.orgChart.noMembers')}</div>
        ) : (
          <div className="flex flex-wrap gap-1">
            {members.map(m => (
              <Badge key={m.id} variant="secondary" className="text-xs font-normal">{m.display_name}</Badge>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

/** 遞迴節點：卡片 + 向下連接線 + 子部門。visited 防資料異常成環。 */
function OrgChartNode({ node, data, visited }: { node: DepartmentWithManager; data: ChartData; visited: Set<string> }) {
  const nextVisited = new Set(visited).add(node.id)
  const children = (data.childrenByParent.get(node.id) ?? []).filter(c => !nextVisited.has(c.id))
  return (
    <div className="inline-flex flex-col items-center">
      <DeptCard dept={node} members={data.membersByDept.get(node.id) ?? []} />
      {children.length > 0 && (
        <>
          <div className="h-5 w-px bg-border" />
          <div className="flex items-start">
            {children.map((c, i) => (
              <div key={c.id} className="relative flex flex-col items-center px-3">
                {children.length > 1 && (
                  <div
                    className={cn(
                      'absolute top-0 h-px bg-border',
                      i === 0 ? 'left-1/2 right-0' : i === children.length - 1 ? 'left-0 right-1/2' : 'left-0 right-0',
                    )}
                  />
                )}
                <div className="h-5 w-px bg-border" />
                <OrgChartNode node={c} data={data} visited={nextVisited} />
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  )
}

export function DepartmentOrgChartTab() {
  const { t } = useTranslation()
  const [zoom, setZoom] = useState(1)
  const [pan, setPan] = useState({ x: 24, y: 24 })
  const dragRef = useRef<{ startX: number; startY: number; panX: number; panY: number } | null>(null)

  const { data: departments = [], isError: departmentsError } = useQuery({
    queryKey: ['departments'],
    queryFn: async () => (await facilityApi.listDepartments()).data,
  })
  const { data: allMembers = [], isError: membersError } = useQuery({
    queryKey: ['department-members-all'],
    queryFn: async () => (await facilityApi.listAllDepartmentMembers()).data,
  })

  const data = useMemo<ChartData>(() => {
    const ids = new Set(departments.map(d => d.id))
    const childrenByParent = new Map<string, DepartmentWithManager[]>()
    const roots: DepartmentWithManager[] = []
    for (const d of departments) {
      if (d.parent_id && ids.has(d.parent_id)) {
        const arr = childrenByParent.get(d.parent_id) ?? []
        arr.push(d)
        childrenByParent.set(d.parent_id, arr)
      } else {
        roots.push(d)
      }
    }
    const membersByDept = new Map<string, DepartmentMember[]>()
    for (const m of allMembers) {
      if (!m.department_id) continue
      const arr = membersByDept.get(m.department_id) ?? []
      arr.push(m)
      membersByDept.set(m.department_id, arr)
    }
    return { roots, childrenByParent, membersByDept }
  }, [departments, allMembers])

  const clampZoom = (z: number) => Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, z))

  const onPointerDown = (e: React.PointerEvent) => {
    dragRef.current = { startX: e.clientX, startY: e.clientY, panX: pan.x, panY: pan.y }
    e.currentTarget.setPointerCapture(e.pointerId)
  }
  const onPointerMove = (e: React.PointerEvent) => {
    const d = dragRef.current
    if (!d) return
    setPan({ x: d.panX + (e.clientX - d.startX), y: d.panY + (e.clientY - d.startY) })
  }
  const onPointerUp = (e: React.PointerEvent) => {
    dragRef.current = null
    // pointerup 後瀏覽器可能已隱式釋放擷取，先檢查避免 releasePointerCapture 拋 NotFoundError。
    if (e.currentTarget.hasPointerCapture(e.pointerId)) {
      e.currentTarget.releasePointerCapture(e.pointerId)
    }
  }

  // 鍵盤平移（a11y）：方向鍵移動畫布，讓無滑鼠使用者也能導覽較大的組織樹。
  const onKeyDown = (e: React.KeyboardEvent) => {
    const STEP = 40
    const delta: Record<string, [number, number]> = {
      ArrowUp: [0, STEP],
      ArrowDown: [0, -STEP],
      ArrowLeft: [STEP, 0],
      ArrowRight: [-STEP, 0],
    }
    const move = delta[e.key]
    if (!move) return
    e.preventDefault()
    setPan(p => ({ x: p.x + move[0], y: p.y + move[1] }))
  }

  const reset = () => { setZoom(1); setPan({ x: 24, y: 24 }) }

  // 兩支查詢任一失敗都會讓組織圖缺內容：部門掛了整棵樹是空的（與「還沒建部門」
  // 同形），成員掛了則是每個部門都顯示 0 人（與「部門真的沒人」同形）。兩種都會
  // 讓人以為資料沒建，因此一起擋在空狀態之前。
  if ((departmentsError && departments.length === 0) || (membersError && allMembers.length === 0)) {
    return (
      <p className="py-12 text-center text-sm text-status-error-text">{t('common.loadFailed')}</p>
    )
  }

  if (departments.length === 0) {
    return <EmptyState icon={Network} title={t('admin.departmentTab.orgChart.empty')} />
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-sm text-muted-foreground">{t('admin.departmentTab.orgChart.hint')}</span>
        <div className="flex items-center gap-1">
          <Button variant="outline" size="icon" onClick={() => setZoom(z => clampZoom(z - ZOOM_STEP))} aria-label={t('admin.departmentTab.orgChart.zoomOut')}><ZoomOut className="h-4 w-4" /></Button>
          <span className="w-12 text-center text-sm tabular-nums text-muted-foreground">{Math.round(zoom * 100)}%</span>
          <Button variant="outline" size="icon" onClick={() => setZoom(z => clampZoom(z + ZOOM_STEP))} aria-label={t('admin.departmentTab.orgChart.zoomIn')}><ZoomIn className="h-4 w-4" /></Button>
          <Button variant="outline" size="icon" onClick={reset} aria-label={t('admin.departmentTab.orgChart.reset')}><RotateCcw className="h-4 w-4" /></Button>
        </div>
      </div>
      <div
        className="relative h-[70vh] overflow-hidden rounded-lg border bg-muted/20 cursor-grab active:cursor-grabbing focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        role="application"
        aria-label={t('admin.departmentTab.orgChart.hint')}
        tabIndex={0}
        onKeyDown={onKeyDown}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerLeave={onPointerUp}
      >
        <div
          className="absolute origin-top-left"
          style={{ transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})` }}
        >
          <div className="flex items-start gap-10">
            {data.roots.map(r => (
              <OrgChartNode key={r.id} node={r} data={data} visited={new Set()} />
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
