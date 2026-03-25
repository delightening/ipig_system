import { useState, useMemo, useRef, useCallback } from 'react'
import type { AnimalListItem } from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Loader2, MapPin } from 'lucide-react'
import { useFacilityLayout, getZoneColors } from '../hooks/useFacilityLayout'
import type { ZoneWithBuilding, PenDetails } from '@/types/facility'
import { PenGridHeader } from './PenGridHeader'
import { PenCell } from './PenCell'

interface AnimalPenViewProps {
  groupedData: { pen_location: string; animals: AnimalListItem[] }[] | undefined
  isLoading: boolean
  onQuickMove: (earTag: string, targetPenLocation: string) => void
  isQuickMovePending: boolean
}

export function AnimalPenView({
  groupedData,
  isLoading,
  onQuickMove,
  isQuickMovePending,
}: AnimalPenViewProps) {
  const { buildings, zonesByBuilding, pensByZone, isLoading: facilityLoading } = useFacilityLayout()
  const [activeBuildingId, setActiveBuildingId] = useState<string | null>(null)
  const [editingPenLocation, setEditingPenLocation] = useState<string | null>(null)
  const [editingEarTag, setEditingEarTag] = useState('')
  const editingRef = useRef({ penLocation: null as string | null, earTag: '' })

  const currentBuildingId = activeBuildingId ?? buildings[0]?.id ?? null

  const animalsByPenLocation = useMemo(() => {
    const map = new Map<string, AnimalListItem[]>()
    groupedData?.forEach(group => {
      if (group.pen_location) {
        map.set(group.pen_location, group.animals)
      }
    })
    return map
  }, [groupedData])

  const handleQuickMoveSubmit = useCallback((penLocation: string) => {
    if (editingEarTag.trim()) {
      onQuickMove(editingEarTag.trim(), penLocation)
    }
    setEditingPenLocation(null)
    setEditingEarTag('')
    editingRef.current = { penLocation: null, earTag: '' }
  }, [editingEarTag, onQuickMove])

  const buildPenGrid = (pens: PenDetails[]) => {
    const leftPens = pens.filter(p => (p.col_index ?? 0) === 0).sort((a, b) => (a.row_index ?? 0) - (b.row_index ?? 0))
    const rightPens = pens.filter(p => (p.col_index ?? 1) === 1).sort((a, b) => (a.row_index ?? 0) - (b.row_index ?? 0))
    const maxRows = Math.max(leftPens.length, rightPens.length)
    return { leftPens, rightPens, maxRows }
  }

  const renderPenCellWrapper = (penCode: string | null, colors: ReturnType<typeof getZoneColors>) => {
    const penAnimals = penCode ? (animalsByPenLocation.get(penCode) || []) : []
    return (
      <PenCell
        penCode={penCode}
        colors={colors}
        penAnimals={penAnimals}
        isEditing={editingPenLocation === penCode}
        editingEarTag={editingPenLocation === penCode ? editingEarTag : ''}
        isQuickMovePending={isQuickMovePending}
        onMouseEnter={() => {
          if (penCode && editingPenLocation !== penCode && !isQuickMovePending) {
            setEditingPenLocation(penCode)
            setEditingEarTag('')
            editingRef.current = { penLocation: penCode, earTag: '' }
          }
        }}
        onEarTagChange={(value) => {
          setEditingEarTag(value)
          editingRef.current.earTag = value
        }}
        onSubmit={() => penCode && handleQuickMoveSubmit(penCode)}
        onCancel={() => {
          setEditingPenLocation(null)
          setEditingEarTag('')
          editingRef.current = { penLocation: null, earTag: '' }
        }}
      />
    )
  }

  const renderZoneCard = (zone: ZoneWithBuilding) => {
    const colors = getZoneColors(zone.color)
    const zonePens = pensByZone[zone.id] ?? []
    const { leftPens, rightPens, maxRows } = buildPenGrid(zonePens)

    let totalAnimals = 0
    zonePens.forEach(pen => {
      totalAnimals += (animalsByPenLocation.get(pen.code) || []).length
    })

    return (
      <Card key={zone.id} className={`${colors.bg} ${colors.border} border-2`} style={colors.borderStyle}>
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-3 text-lg">
            <span className={`w-8 h-8 rounded-lg ${colors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`} style={colors.headerStyle}>
              {zone.code}
            </span>
            <span className={colors.text}>{zone.name ?? `${zone.code} 區`}</span>
            <Badge variant="outline" className={`ml-2 ${colors.text} ${colors.border}`} style={colors.borderStyle}>
              共 {totalAnimals} 隻
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="bg-background rounded-lg shadow-sm overflow-hidden">
            <div className="grid grid-cols-2 gap-0 border-b">
              <PenGridHeader colors={colors} />
              <PenGridHeader colors={colors} borderLeft />
            </div>
            {Array.from({ length: maxRows }).map((_, idx) => (
              <div key={idx} className={`grid grid-cols-2 gap-0 border-b last:border-b-0 ${idx % 2 === 0 ? 'bg-background' : colors.bg}`}>
                <div className={`border-r ${colors.border}`}>
                  {renderPenCellWrapper(leftPens[idx]?.code ?? null, colors)}
                </div>
                <div>
                  {renderPenCellWrapper(rightPens[idx]?.code ?? null, colors)}
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    )
  }

  const renderCombinedZoneCard = (zones: ZoneWithBuilding[]) => {
    const leftZones = zones.filter(z => {
      const cfg = z.layout_config as Record<string, unknown> | null
      return cfg?.group_position === 'left'
    })
    const rightZones = zones.filter(z => {
      const cfg = z.layout_config as Record<string, unknown> | null
      return cfg?.group_position === 'right'
    }).sort((a, b) => {
      const aOrder = ((a.layout_config as Record<string, unknown>)?.group_order as number) ?? 0
      const bOrder = ((b.layout_config as Record<string, unknown>)?.group_order as number) ?? 0
      return aOrder - bOrder
    })

    const leftPens = leftZones.flatMap(z => (pensByZone[z.id] ?? []).map(p => ({ ...p, _zone: z })))
    const rightPens = rightZones.flatMap(z => (pensByZone[z.id] ?? []).map(p => ({ ...p, _zone: z })))
    const maxRows = Math.max(leftPens.length, rightPens.length)

    const zoneAnimalCounts = new Map<string, number>()
    for (const z of zones) {
      let count = 0
      ;(pensByZone[z.id] ?? []).forEach(p => {
        count += (animalsByPenLocation.get(p.code) || []).length
      })
      zoneAnimalCounts.set(z.id, count)
    }

    const firstLeftZone = leftZones[0]
    const leftColors = firstLeftZone ? getZoneColors(firstLeftZone.color) : getZoneColors(null)

    const zoneHexColors = zones.map(z => {
      const raw = z.color || '#94a3b8'
      return raw.startsWith('#') ? raw : `#${raw}`
    })
    const gradientBg = zoneHexColors.length >= 2
      ? `linear-gradient(to right, ${zoneHexColors.map(c => `${c}15`).join(', ')})`
      : undefined
    const borderColor = zoneHexColors[0] || '#94a3b8'

    const rightHeaderStyle = {
      background: rightZones.length >= 2
        ? `linear-gradient(to right, ${rightZones.map(z => { const c = z.color || '#94a3b8'; return c.startsWith('#') ? c : `#${c}` }).join(', ')})`
        : rightZones[0]?.color?.startsWith('#') ? rightZones[0].color : `#${rightZones[0]?.color || '94a3b8'}`,
    }

    return (
      <Card key={zones.map(z => z.id).join('-')} className="border-2" style={{ background: gradientBg, borderColor }}>
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-3 text-lg flex-wrap">
            {zones.map((z, i) => {
              const c = getZoneColors(z.color)
              return (
                <div key={z.id} className="flex items-center gap-2">
                  {i > 0 && <span className="text-muted-foreground">|</span>}
                  <span className={`w-8 h-8 rounded-lg ${c.header} text-white flex items-center justify-center font-bold text-lg shadow-md`} style={c.headerStyle}>{z.code}</span>
                  <span className={c.text}>{z.name ?? `${z.code} 區`}</span>
                  <Badge variant="outline" className={`${c.text} ${c.border}`} style={c.borderStyle}>{zoneAnimalCounts.get(z.id) ?? 0} 隻</Badge>
                </div>
              )
            })}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="bg-background rounded-lg shadow-sm overflow-hidden">
            <div className="grid grid-cols-2 gap-0 border-b">
              <PenGridHeader colors={leftColors} />
              <PenGridHeader colors={leftColors} borderLeft style={rightHeaderStyle} />
            </div>
            {Array.from({ length: maxRows }).map((_, idx) => {
              const leftPen = leftPens[idx]
              const rightPen = rightPens[idx]
              const rightZoneObj = rightPen?._zone
              const rightColors = rightZoneObj ? getZoneColors(rightZoneObj.color) : getZoneColors(null)
              const prevRightPen = idx > 0 ? rightPens[idx - 1] : null
              const isTransition = prevRightPen && rightPen && prevRightPen._zone.id !== rightPen._zone.id

              return (
                <div
                  key={idx}
                  className={`grid grid-cols-2 gap-0 border-b last:border-b-0 ${isTransition ? 'border-t-2 border-t-status-success-text' : ''} ${idx % 2 === 0 ? 'bg-background' : 'bg-muted'}`}
                >
                  <div className={`border-r ${leftColors.border}`}>
                    {renderPenCellWrapper(leftPen?.code ?? null, leftColors)}
                  </div>
                  <div className={rightPen ? rightColors.bg : ''}>
                    {renderPenCellWrapper(rightPen?.code ?? null, rightColors)}
                  </div>
                </div>
              )
            })}
          </div>
        </CardContent>
      </Card>
    )
  }

  const getZoneRenderGroups = (buildingId: string) => {
    const zones = zonesByBuilding[buildingId] ?? []
    const standalone: ZoneWithBuilding[] = []
    const combinedGroups = new Map<string, ZoneWithBuilding[]>()

    for (const z of zones) {
      const cfg = z.layout_config as Record<string, unknown> | null
      const displayGroup = cfg?.display_group as string | undefined
      if (displayGroup) {
        if (!combinedGroups.has(displayGroup)) combinedGroups.set(displayGroup, [])
        combinedGroups.get(displayGroup)!.push(z)
      } else {
        standalone.push(z)
      }
    }

    return { standalone, combinedGroups }
  }

  const loading = isLoading || facilityLoading

  return (
    <div className="space-y-4">
      <div className="flex gap-2 border-b">
        {buildings.map(building => {
          const zones = zonesByBuilding[building.id] ?? []
          return (
            <button
              key={building.id}
              onClick={() => setActiveBuildingId(building.id)}
              className={`px-6 py-3 text-sm font-medium border-b-2 transition-colors flex items-center gap-2 ${
                currentBuildingId === building.id
                  ? 'border-primary text-primary bg-primary/5'
                  : 'border-transparent text-muted-foreground hover:text-foreground hover:bg-muted'
              }`}
            >
              <MapPin className="h-4 w-4" />
              {building.name}
              <div className="flex gap-1 ml-2">
                {zones.map(zone => {
                  const c = getZoneColors(zone.color)
                  return (
                    <span
                      key={zone.id}
                      className={`w-5 h-5 rounded text-xs font-bold flex items-center justify-center text-white ${c.header}`}
                      style={c.headerStyle}
                    >
                      {zone.code}
                    </span>
                  )
                })}
              </div>
            </button>
          )
        })}
      </div>

      {loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      ) : currentBuildingId ? (
        <div className="space-y-6">
          {(() => {
            const { standalone, combinedGroups } = getZoneRenderGroups(currentBuildingId)
            return (
              <>
                {standalone.map(zone => renderZoneCard(zone))}
                {Array.from(combinedGroups.values()).map(zones => renderCombinedZoneCard(zones))}
              </>
            )
          })()}
        </div>
      ) : null}
    </div>
  )
}
