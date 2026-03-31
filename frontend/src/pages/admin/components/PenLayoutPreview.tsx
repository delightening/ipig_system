import { useState, useCallback, useRef, useMemo } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Eye, GripVertical } from 'lucide-react'
import { facilityApi } from '@/lib/api/facility'
import type { PenDetails, UpdatePenRequest } from '@/types/facility'
import type { ZoneWithBuilding } from '@/types/facility'
import { cn } from '@/lib/utils'

interface PenLayoutPreviewProps {
  pens: PenDetails[]
  zones: ZoneWithBuilding[]
  canManage: boolean
}

export function PenLayoutPreview({ pens, zones, canManage }: PenLayoutPreviewProps) {
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
            欄位配置預覽
          </CardTitle>
          <Select value={selectedZone} onValueChange={setSelectedZone}>
            <SelectTrigger className="w-48">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部區域</SelectItem>
              {zonesWithPens.map(z => (
                <SelectItem key={z.id} value={z.id}>
                  <span className="flex items-center gap-2">
                    {z.color && <span className="w-2.5 h-2.5 rounded-sm inline-block" style={{ backgroundColor: z.color }} />}
                    {z.building_code}棟 {z.code}區
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
            該區域尚無欄位
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
  const queryClient = useQueryClient()
  const [dragPen, setDragPen] = useState<PenDetails | null>(null)
  const [dropTarget, setDropTarget] = useState<{ row: number; col: number } | null>(null)
  const gridRef = useRef<HTMLDivElement>(null)

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdatePenRequest }) =>
      facilityApi.updatePen(id, data),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['pens'] }),
    onError: (err: unknown) =>
      toast({ title: '更新失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  // 計算 grid 尺寸
  const maxRow = Math.max(0, ...pens.map(p => p.row_index ?? 0))
  const maxCol = Math.max(0, ...pens.map(p => p.col_index ?? 0))
  const rows = maxRow + 1
  const cols = maxCol + 1

  // 建立 grid map
  const gridMap = useMemo(() => {
    const map = new Map<string, PenDetails>()
    for (const pen of pens) {
      const key = `${pen.row_index ?? 0}-${pen.col_index ?? 0}`
      map.set(key, pen)
    }
    return map
  }, [pens])

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
          {zone?.building_code}棟 {zone?.code}區 {zone?.name || ''}
        </span>
        <Badge variant="secondary" className="text-xs">{pens.length} 欄</Badge>
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
      {canManage && pens.length > 1 && (
        <p className="text-xs text-muted-foreground mt-2">拖拉欄位可調整位置</p>
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
  if (!pen) {
    return (
      <div
        className={cn(
          'h-10 rounded border border-dashed border-border flex items-center justify-center text-xs text-muted-foreground',
          isDropTarget && 'border-primary bg-primary/10'
        )}
        onDragOver={(e) => { e.preventDefault(); onDragOver() }}
        onDrop={(e) => { e.preventDefault(); onDrop() }}
      />
    )
  }

  const statusColor = pen.status === 'active'
    ? color
    : pen.status === 'maintenance' ? '#f59e0b' : '#d1d5db'

  return (
    <div
      draggable={canManage}
      onDragStart={onDragStart}
      onDragOver={(e) => { e.preventDefault(); onDragOver() }}
      onDrop={(e) => { e.preventDefault(); onDrop() }}
      onDragEnd={onDragEnd}
      className={cn(
        'h-10 rounded flex items-center justify-center text-xs font-mono font-medium text-white select-none transition-all',
        canManage && 'cursor-grab active:cursor-grabbing',
        isDragging && 'opacity-40 scale-95',
        isDropTarget && 'ring-2 ring-ring'
      )}
      style={{ backgroundColor: statusColor }}
      title={`${pen.code}${pen.name ? ` (${pen.name})` : ''} — ${pen.current_count}/${pen.capacity}`}
    >
      {canManage && <GripVertical className="h-3 w-3 mr-0.5 opacity-50" />}
      {pen.code}
    </div>
  )
}
