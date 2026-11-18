import { useState, useCallback, useRef, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Eye, GripVertical, Ruler } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { facilityApi } from '@/lib/api/facility'
import type { PenDetails, UpdatePenRequest } from '@/types/facility'
import type { ZoneWithBuilding } from '@/types/facility'
import { cn } from '@/lib/utils'
import { ApplyPenSizeDialog } from './ApplyPenSizeDialog'
import { exactAreaM2, formatArea, formatMeters } from '../utils/penSize'

/** 超標時的「需／有」面積字串參數；非超標回 null。有面積用長寬算的精確值（後端 area_m2 已四捨五入） */
function exceededAreaArgs(pen: PenDetails): { required: string; area: string } | null {
  const fa = pen.floor_area
  if (fa?.status !== 'exceeded' || fa.area_m2 == null) return null
  return {
    required: formatMeters(fa.required_area_m2),
    area: formatArea(exactAreaM2(pen.length_m, pen.width_m, Number(fa.area_m2))),
  }
}

interface PenLayoutPreviewProps {
  pens: PenDetails[]
  zones: ZoneWithBuilding[]
  canManage: boolean
}

export function PenLayoutPreview({ pens, zones, canManage }: PenLayoutPreviewProps) {
  const { t } = useTranslation()
  const [selectedZone, setSelectedZone] = useState<string>('all')

  // 找出有欄位的區域
  const zonesWithPens = zones.filter(z => pens.some(p => p.zone_id === z.id))
  const filteredPens = selectedZone === 'all' ? pens : pens.filter(p => p.zone_id === selectedZone)

  // 按區域分組
  const pensByZone = new Map<string, PenDetails[]>()
  for (const pen of filteredPens) {
    const key = pen.zone_id
    if (!pensByZone.has(key)) pensByZone.set(key, [])
    pensByZone.get(key)!.push(pen)
  }

  if (pens.length === 0) return null

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base flex items-center gap-2">
            <Eye className="h-4 w-4" />
            {t('admin.penLayoutPreview.title')}
          </CardTitle>
          <Select value={selectedZone} onValueChange={setSelectedZone}>
            <SelectTrigger className="w-48" aria-label={t('admin.penLayoutPreview.zoneFilterLabel')}>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('admin.penLayoutPreview.allZones')}</SelectItem>
              {zonesWithPens.map(z => (
                <SelectItem key={z.id} value={z.id}>
                  <span className="flex items-center gap-2">
                    {z.color && <span className="w-2.5 h-2.5 rounded-sm inline-block" style={{ backgroundColor: z.color }} />}
                    {t('admin.penLayoutPreview.zoneLabel', { building: z.building_code, zone: z.code })}
                  </span>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {Array.from(pensByZone.entries()).map(([zoneId, zonePens]) => {
          const zone = zones.find(z => z.id === zoneId)
          return (
            <ZoneGrid
              key={zoneId}
              zone={zone}
              pens={zonePens}
              canManage={canManage}
            />
          )
        })}
        {filteredPens.length === 0 && (
          <div className="text-center py-6 text-muted-foreground text-sm">
            {t('admin.penLayoutPreview.noPensInZone')}
          </div>
        )}
      </CardContent>
    </Card>
  )
}

function ZoneGrid({ zone, pens, canManage }: {
  zone: ZoneWithBuilding | undefined
  pens: PenDetails[]
  canManage: boolean
}) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [dragPen, setDragPen] = useState<PenDetails | null>(null)
  const [dropTarget, setDropTarget] = useState<{ row: number; col: number } | null>(null)
  const [applySizeOpen, setApplySizeOpen] = useState(false)
  const gridRef = useRef<HTMLDivElement>(null)

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdatePenRequest }) =>
      facilityApi.updatePen(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pens'] })
      queryClient.invalidateQueries({ queryKey: ['facility-pens'] })
    },
    onError: (err: unknown) =>
      toast({ title: t('admin.penLayoutPreview.updateFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  // 計算 grid 尺寸
  const maxRow = Math.max(0, ...pens.map(p => p.row_index ?? 0))
  const maxCol = Math.max(0, ...pens.map(p => p.col_index ?? 0))
  const rows = maxRow + 1
  const cols = maxCol + 1

  // 建立 grid map；未排位（沒有座標）或座標重複的欄位不能互相覆蓋，否則被蓋掉的那格連同它的超標警示會消失，
  // 改為另外列出（extraPens）
  const { gridMap, extraPens } = useMemo(() => {
    const map = new Map<string, PenDetails>()
    const extra: PenDetails[] = []
    for (const pen of pens) {
      const key = `${pen.row_index ?? 0}-${pen.col_index ?? 0}`
      if (map.has(key)) extra.push(pen)
      else map.set(key, pen)
    }
    return { gridMap: map, extraPens: extra }
  }, [pens])
  const exceededPens = pens.filter(p => exceededAreaArgs(p) !== null)

  const color = zone?.color || '#94a3b8'

  const handleDragStart = useCallback((pen: PenDetails) => {
    if (!canManage) return
    setDragPen(pen)
  }, [canManage])

  const handleDragOver = useCallback((row: number, col: number) => {
    setDropTarget({ row, col })
  }, [])

  const handleDrop = useCallback((targetRow: number, targetCol: number) => {
    if (!dragPen) return

    const targetPen = gridMap.get(`${targetRow}-${targetCol}`)

    // Swap positions
    if (targetPen && targetPen.id !== dragPen.id) {
      updateMutation.mutate({
        id: dragPen.id,
        data: { row_index: targetRow, col_index: targetCol },
      })
      updateMutation.mutate({
        id: targetPen.id,
        data: { row_index: dragPen.row_index ?? 0, col_index: dragPen.col_index ?? 0 },
      })
    } else if (!targetPen) {
      updateMutation.mutate({
        id: dragPen.id,
        data: { row_index: targetRow, col_index: targetCol },
      })
    }

    setDragPen(null)
    setDropTarget(null)
  }, [dragPen, gridMap, updateMutation])

  return (
    <div className="border rounded-lg p-3">
      <div className="flex items-center gap-2 mb-2">
        <span className="w-3 h-3 rounded-sm" style={{ backgroundColor: color }} />
        <span className="text-sm font-medium">
          {t('admin.penLayoutPreview.zoneHeading', { building: zone?.building_code ?? '', zone: zone?.code ?? '', name: zone?.name ?? '' })}
        </span>
        <Badge variant="secondary" className="text-xs">{t('admin.penLayoutPreview.penCount', { count: pens.length })}</Badge>
        {canManage && zone && (
          <Button type="button" variant="outline" size="sm" className="ml-auto" onClick={() => setApplySizeOpen(true)}>
            <Ruler className="h-4 w-4 mr-1" /> {t('admin.penLayoutPreview.applySizeButton')}
          </Button>
        )}
      </div>
      <div
        ref={gridRef}
        className="grid gap-1"
        style={{ gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }}
      >
        {Array.from({ length: rows }).map((_, row) =>
          Array.from({ length: cols }).map((_, col) => {
            const pen = gridMap.get(`${row}-${col}`)
            const isDropHere = dropTarget?.row === row && dropTarget?.col === col
            return (
              <PenCell
                key={`${row}-${col}`}
                pen={pen}
                color={color}
                isDragging={dragPen?.id === pen?.id}
                isDropTarget={isDropHere}
                canManage={canManage}
                onDragStart={() => pen && handleDragStart(pen)}
                onDragOver={() => handleDragOver(row, col)}
                onDrop={() => handleDrop(row, col)}
                onDragEnd={() => { setDragPen(null); setDropTarget(null) }}
              />
            )
          })
        )}
      </div>
      {extraPens.length > 0 && (
        <div className="mt-2">
          <p className="text-xs text-muted-foreground mb-1">{t('admin.penLayoutPreview.unplacedHeading')}</p>
          <div className="grid gap-1" style={{ gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }}>
            {extraPens.map(pen => (
              <PenCell
                key={pen.id}
                pen={pen}
                color={color}
                isDragging={dragPen?.id === pen.id}
                isDropTarget={false}
                canManage={canManage}
                onDragStart={() => handleDragStart(pen)}
                onDragOver={() => undefined}
                onDrop={() => undefined}
                onDragEnd={() => { setDragPen(null); setDropTarget(null) }}
              />
            ))}
          </div>
        </div>
      )}
      {exceededPens.length > 0 && (
        // 超標資訊不只靠游標移上去的提示：觸控與鍵盤使用者也要看得到
        <ul className="mt-2 space-y-0.5 text-xs text-status-warning-solid">
          {exceededPens.map(pen => (
            <li key={pen.id}>{pen.code} — {t('admin.penLayoutPreview.cellAreaExceeded', exceededAreaArgs(pen) ?? undefined)}</li>
          ))}
        </ul>
      )}
      {canManage && pens.length > 1 && (
        <p className="text-xs text-muted-foreground mt-2">{t('admin.penLayoutPreview.dragHint')}</p>
      )}
      {canManage && zone && (
        <ApplyPenSizeDialog open={applySizeOpen} onOpenChange={setApplySizeOpen} zone={zone} pens={pens} />
      )}
    </div>
  )
}

function PenCell({ pen, color, isDragging, isDropTarget, canManage, onDragStart, onDragOver, onDrop, onDragEnd }: {
  pen: PenDetails | undefined
  color: string
  isDragging: boolean
  isDropTarget: boolean
  canManage: boolean
  onDragStart: () => void
  onDragOver: () => void
  onDrop: () => void
  onDragEnd: () => void
}) {
  const { t } = useTranslation()
  if (!pen) {
    return (
      <div
        className={cn(
          'h-10 rounded border border-dashed border-border flex items-center justify-center text-xs text-muted-foreground',
          isDropTarget && 'border-primary bg-primary/10'
        )}
        onDragOver={(e) => { e.preventDefault(); onDragOver() }}
        onDrop={(e) => { e.preventDefault(); onDrop() }}
        aria-label={t('admin.penLayoutPreview.emptySlotLabel')}
      />
    )
  }

  const statusColor = pen.status === 'active'
    ? color
    : pen.status === 'maintenance' ? '#f59e0b' : '#d1d5db'

  const baseTitle = pen.name
    ? t('admin.penLayoutPreview.cellTitleNamed', { code: pen.code, name: pen.name, current: pen.current_count, capacity: pen.capacity })
    : t('admin.penLayoutPreview.cellTitle', { code: pen.code, current: pen.current_count, capacity: pen.capacity })

  // 地板面積：超標加橘框與「!」，游標移上去顯示「需 X／有 Y」；未設定（no_area）維持原樣
  const exceededArgs = exceededAreaArgs(pen)
  const exceeded = exceededArgs !== null
  const penTitle = exceededArgs
    ? `${baseTitle} — ${t('admin.penLayoutPreview.cellAreaExceeded', exceededArgs)}`
    : baseTitle

  return (
    <div
      draggable={canManage}
      onDragStart={onDragStart}
      onDragOver={(e) => { e.preventDefault(); onDragOver() }}
      onDrop={(e) => { e.preventDefault(); onDrop() }}
      onDragEnd={onDragEnd}
      className={cn(
        'relative h-10 rounded flex items-center justify-center text-xs font-mono font-medium text-white select-none transition-all',
        canManage && 'cursor-grab active:cursor-grabbing',
        isDragging && 'opacity-40 scale-95',
        exceeded && 'ring-2 ring-status-warning-solid ring-offset-1',
        isDropTarget && 'ring-2 ring-ring'
      )}
      style={{ backgroundColor: statusColor }}
      title={penTitle}
    >
      {canManage && <GripVertical className="h-3 w-3 mr-0.5 opacity-50" />}
      {pen.code}
      {/* 一般 div 上的 aria-label 讀不到，改放螢幕閱讀器專用文字（含超標資訊） */}
      <span className="sr-only">{penTitle}</span>
      {exceeded && (
        <span
          aria-hidden="true"
          className="absolute -top-1.5 -right-1.5 flex h-4 w-4 items-center justify-center rounded-full bg-status-warning-solid text-[10px] font-bold text-white"
        >
          !
        </span>
      )}
    </div>
  )
}
