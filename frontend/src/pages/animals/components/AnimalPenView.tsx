import { useState, useMemo } from 'react'
import { Link } from 'react-router-dom'
import type { AnimalListItem } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Eye, Edit2, Loader2, MapPin } from 'lucide-react'
import { useFacilityLayout, getZoneColors } from '../hooks/useFacilityLayout'
import type { ZoneWithBuilding, PenDetails } from '@/types/facility'

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

  // 預設選第一棟
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

  const handleQuickMoveSubmit = (penLocation: string) => {
    if (editingEarTag.trim()) {
      onQuickMove(editingEarTag.trim(), penLocation)
    }
    setEditingPenLocation(null)
    setEditingEarTag('')
  }

  const renderPenCell = (penCode: string | null, colors: { bg: string; border: string; header: string; text: string }) => {
    if (!penCode) {
      return <div className="px-3 py-2 text-slate-300"></div>
    }

    const penAnimals = animalsByPenLocation.get(penCode) || []
    const isEditing = editingPenLocation === penCode

    if (penAnimals.length === 0) {
      const handleSubmit = () => handleQuickMoveSubmit(penCode)

      return (
        <div
          className="grid grid-cols-5 gap-1 px-3 py-2 items-center text-sm group"
          onMouseEnter={() => {
            if (!isEditing && !isQuickMovePending) {
              setEditingPenLocation(penCode)
              setEditingEarTag('')
            }
          }}
        >
          <div className={`font-semibold ${colors.text}`}>{penCode}</div>
          {isEditing ? (
            <Input
              className="h-7 text-sm"
              value={editingEarTag}
              onChange={(e) => setEditingEarTag(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && editingEarTag.trim()) {
                  e.preventDefault()
                  handleSubmit()
                } else if (e.key === 'Escape') {
                  e.preventDefault()
                  setEditingPenLocation(null)
                  setEditingEarTag('')
                }
              }}
              onBlur={() => {
                setTimeout(() => {
                  if (editingPenLocation === penCode && editingEarTag.trim()) {
                    handleSubmit()
                  } else if (editingPenLocation === penCode && !editingEarTag.trim()) {
                    setEditingPenLocation(null)
                    setEditingEarTag('')
                  }
                }, 150)
              }}
              placeholder="輸入耳號"
              autoFocus
              disabled={isQuickMovePending}
            />
          ) : (
            <div className="text-slate-400 italic group-hover:text-slate-600 transition-colors cursor-text">空</div>
          )}
          <div className="text-slate-300">-</div>
          <div className="text-slate-300">-</div>
          <div></div>
        </div>
      )
    }

    return penAnimals.map((animal, animalIdx) => (
      <div key={animal.id} className={`grid grid-cols-5 gap-1 px-3 py-2 items-center text-sm ${animalIdx > 0 ? 'border-t border-dashed border-slate-200' : ''}`}>
        <div className={`font-semibold ${colors.text}`}>{animalIdx === 0 ? penCode : ''}</div>
        <Link
          to={`/animals/${animal.id}`}
          className="font-medium truncate text-orange-600 hover:text-orange-700 hover:underline cursor-pointer"
          title={`點擊進入動物詳情 · ${animal.ear_tag}`}
        >
          {animal.ear_tag}
        </Link>
        <div className="text-xs text-slate-500 truncate" title={animal.vet_last_viewed_at ? new Date(animal.vet_last_viewed_at).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei' }) : '-'}>
          {animal.vet_last_viewed_at ? new Date(animal.vet_last_viewed_at).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' }) : '-'}
        </div>
        <div className={`text-xs truncate ${animal.has_abnormal_record ? 'text-red-600 font-medium' : 'text-slate-400'}`}>
          {animal.has_abnormal_record ? '有異常' : '-'}
        </div>
        <div className="flex items-center justify-center gap-1">
          <Button variant="ghost" size="icon" className="h-6 w-6" asChild title="檢視">
            <Link to={`/animals/${animal.id}`}>
              <Eye className="h-3 w-3" />
            </Link>
          </Button>
          <Button variant="ghost" size="icon" className="h-6 w-6" asChild title="編輯">
            <Link to={`/animals/${animal.id}/edit`}>
              <Edit2 className="h-3 w-3" />
            </Link>
          </Button>
        </div>
      </div>
    ))
  }

  /** 將 pens 依 row_index/col_index 排列成兩欄格子 */
  const buildPenGrid = (pens: PenDetails[]) => {
    const sortByRow = (a: PenDetails, b: PenDetails) => (a.row_index ?? 0) - (b.row_index ?? 0)
    const leftPens = pens.filter(p => p.col_index === 0).sort(sortByRow)
    const rightPens = pens.filter(p => p.col_index === 1).sort(sortByRow)

    // 如果 col_index 全為 null（單欄區域如 E/F/G），所有 pen 放左欄
    if (leftPens.length === 0 && rightPens.length === 0) {
      return { leftPens: [...pens].sort(sortByRow), rightPens: [] as PenDetails[], maxRows: pens.length }
    }

    const maxRows = Math.max(leftPens.length, rightPens.length)
    return { leftPens, rightPens, maxRows }
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
      <Card key={zone.id} className={`${colors.bg} ${colors.border} border-2`}>
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-3 text-lg">
            <span className={`w-8 h-8 rounded-lg ${colors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>
              {zone.code}
            </span>
            <span className={colors.text}>{zone.name ?? `${zone.code} 區`}</span>
            <Badge variant="outline" className={`ml-2 ${colors.text} ${colors.border}`}>
              共 {totalAnimals} 隻
            </Badge>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="bg-white rounded-lg shadow-sm overflow-hidden">
            <div className="grid grid-cols-2 gap-0 border-b">
              <div className={`grid grid-cols-5 gap-1 px-3 py-2 text-xs font-semibold ${colors.header} text-white`}>
                <div>欄位</div><div>耳號</div><div>獸醫檢視</div><div>最新異常</div><div className="text-center">操作</div>
              </div>
              <div className={`grid grid-cols-5 gap-1 px-3 py-2 text-xs font-semibold ${colors.header} text-white border-l border-white/30`}>
                <div>欄位</div><div>耳號</div><div>獸醫檢視</div><div>最新異常</div><div className="text-center">操作</div>
              </div>
            </div>
            {Array.from({ length: maxRows }).map((_, idx) => {
              const leftPen = leftPens[idx]
              const rightPen = rightPens[idx]

              return (
                <div key={idx} className={`grid grid-cols-2 gap-0 border-b last:border-b-0 ${idx % 2 === 0 ? 'bg-white' : colors.bg}`}>
                  <div className={`border-r ${colors.border}`}>
                    {renderPenCell(leftPen?.code ?? null, colors)}
                  </div>
                  <div>
                    {renderPenCell(rightPen?.code ?? null, colors)}
                  </div>
                </div>
              )
            })}
          </div>
        </CardContent>
      </Card>
    )
  }

  /** 渲染合併區域卡片（如 EFG 合併） */
  const renderCombinedZoneCard = (zones: ZoneWithBuilding[]) => {
    // 找出 left/right 區域
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

    // 左欄 pens
    const leftPens = leftZones.flatMap(z => (pensByZone[z.id] ?? []).map(p => ({ ...p, _zone: z })))
    // 右欄 pens（依 zone 順序連接）
    const rightPens = rightZones.flatMap(z => (pensByZone[z.id] ?? []).map(p => ({ ...p, _zone: z })))
    const maxRows = Math.max(leftPens.length, rightPens.length)

    // 各區動物數
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

    return (
      <Card key={zones.map(z => z.id).join('-')} className="bg-gradient-to-r from-purple-50 via-amber-50 to-green-50 border-2 border-purple-300">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-3 text-lg flex-wrap">
            {zones.map((z, i) => {
              const c = getZoneColors(z.color)
              return (
                <div key={z.id} className="flex items-center gap-2">
                  {i > 0 && <span className="text-slate-300">|</span>}
                  <span className={`w-8 h-8 rounded-lg ${c.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>{z.code}</span>
                  <span className={c.text}>{z.name ?? `${z.code} 區`}</span>
                  <Badge variant="outline" className={`${c.text} ${c.border}`}>{zoneAnimalCounts.get(z.id) ?? 0} 隻</Badge>
                </div>
              )
            })}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="bg-white rounded-lg shadow-sm overflow-hidden">
            <div className="grid grid-cols-2 gap-0 border-b">
              <div className={`grid grid-cols-5 gap-1 px-3 py-2 text-xs font-semibold ${leftColors.header} text-white`}>
                <div>欄位</div><div>耳號</div><div>獸醫檢視</div><div>最新異常</div><div className="text-center">操作</div>
              </div>
              <div className="grid grid-cols-5 gap-1 px-3 py-2 text-xs font-semibold bg-gradient-to-r from-amber-500 to-green-500 text-white border-l border-white/30">
                <div>欄位</div><div>耳號</div><div>獸醫檢視</div><div>最新異常</div><div className="text-center">操作</div>
              </div>
            </div>
            {Array.from({ length: maxRows }).map((_, idx) => {
              const leftPen = leftPens[idx]
              const rightPen = rightPens[idx]
              const rightZoneObj = rightPen?._zone
              const rightColors = rightZoneObj ? getZoneColors(rightZoneObj.color) : getZoneColors(null)
              // 偵測右欄區域轉換
              const prevRightPen = idx > 0 ? rightPens[idx - 1] : null
              const isTransition = prevRightPen && rightPen && prevRightPen._zone.id !== rightPen._zone.id

              return (
                <div
                  key={idx}
                  className={`grid grid-cols-2 gap-0 border-b last:border-b-0 ${isTransition ? 'border-t-2 border-t-green-400' : ''} ${idx % 2 === 0 ? 'bg-white' : 'bg-slate-50'}`}
                >
                  <div className={`border-r ${leftColors.border}`}>
                    {renderPenCell(leftPen?.code ?? null, leftColors)}
                  </div>
                  <div className={rightPen ? rightColors.bg : ''}>
                    {renderPenCell(rightPen?.code ?? null, rightColors)}
                  </div>
                </div>
              )
            })}
          </div>
        </CardContent>
      </Card>
    )
  }

  /** 取得棟舍下的區域，分為獨立渲染和合併渲染 */
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
      {/* Building Tabs */}
      <div className="flex gap-2 border-b">
        {buildings.map(building => {
          const zones = zonesByBuilding[building.id] ?? []
          return (
            <button
              key={building.id}
              onClick={() => setActiveBuildingId(building.id)}
              className={`px-6 py-3 text-sm font-medium border-b-2 transition-colors flex items-center gap-2 ${
                currentBuildingId === building.id
                  ? 'border-purple-600 text-purple-600 bg-purple-50'
                  : 'border-transparent text-slate-500 hover:text-slate-700 hover:bg-slate-50'
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
          <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
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
