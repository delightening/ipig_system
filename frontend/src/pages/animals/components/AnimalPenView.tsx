import { useState } from 'react'
import { Link } from 'react-router-dom'
import type { AnimalListItem } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Eye, Edit2, Loader2, MapPin } from 'lucide-react'
import {
  penBuildings,
  penZonesByBuilding,
  penNumbersByZone,
  penZoneColors,
} from '../constants'

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
  const [buildingTab, setBuildingTab] = useState<'A' | 'B'>('A')
  const [editingPenLocation, setEditingPenLocation] = useState<string | null>(null)
  const [editingEarTag, setEditingEarTag] = useState('')

  const animalsByPenLocation = new Map<string, AnimalListItem[]>()
  groupedData?.forEach(group => {
    if (group.pen_location) {
      animalsByPenLocation.set(group.pen_location, group.animals)
    }
  })

  const handleQuickMoveSubmit = (penLocation: string) => {
    if (editingEarTag.trim()) {
      onQuickMove(editingEarTag.trim(), penLocation)
      setEditingPenLocation(null)
      setEditingEarTag('')
    } else {
      setEditingPenLocation(null)
      setEditingEarTag('')
    }
  }

  const renderPenCell = (penLocation: string | null, colors: { bg: string; border: string; header: string; text: string }) => {
    if (!penLocation) {
      return <div className="px-3 py-2 text-slate-300"></div>
    }

    const penAnimals = animalsByPenLocation.get(penLocation) || []
    const cellColors = penZoneColors[penLocation.charAt(0)] || colors
    const isEditing = editingPenLocation === penLocation

    if (penAnimals.length === 0) {
      const handleSubmit = () => handleQuickMoveSubmit(penLocation)

      return (
        <div
          className="grid grid-cols-5 gap-1 px-3 py-2 items-center text-sm group"
          onMouseEnter={() => {
            if (!isEditing && !isQuickMovePending) {
              setEditingPenLocation(penLocation)
              setEditingEarTag('')
            }
          }}
        >
          <div className={`font-semibold ${cellColors.text}`}>{penLocation}</div>
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
                  if (editingPenLocation === penLocation && editingEarTag.trim()) {
                    handleSubmit()
                  } else if (editingPenLocation === penLocation && !editingEarTag.trim()) {
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
        <div className={`font-semibold ${cellColors.text}`}>{animalIdx === 0 ? penLocation : ''}</div>
        <div className={`font-medium ${cellColors.text} truncate`} title={animal.ear_tag}>{animal.ear_tag}</div>
        <div className="text-xs text-slate-500 truncate" title={animal.vet_last_viewed_at ? new Date(animal.vet_last_viewed_at).toLocaleString('zh-TW') : '-'}>
          {animal.vet_last_viewed_at ? new Date(animal.vet_last_viewed_at).toLocaleDateString('zh-TW') : '-'}
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

  const renderStandardZoneCard = (zone: string) => {
    const colors = penZoneColors[zone] || { bg: 'bg-gray-50', border: 'border-gray-300', header: 'bg-gray-500', text: 'text-gray-700' }
    const penNumbers = penNumbersByZone[zone] || []
    const halfPoint = Math.ceil(penNumbers.length / 2)
    const leftColumnPens = penNumbers.slice(0, halfPoint)
    const rightColumnPens = penNumbers.slice(halfPoint)

    let totalAnimals = 0
    penNumbers.forEach(num => {
      totalAnimals += (animalsByPenLocation.get(`${zone}${num}`) || []).length
    })

    return (
      <Card key={zone} className={`${colors.bg} ${colors.border} border-2`}>
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-3 text-lg">
            <span className={`w-8 h-8 rounded-lg ${colors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>
              {zone}
            </span>
            <span className={colors.text}>{zone} 區</span>
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
            {leftColumnPens.map((leftNum, idx) => {
              const rightNum = rightColumnPens[idx]
              const leftPenLocation = `${zone}${leftNum}`
              const rightPenLocation = rightNum ? `${zone}${rightNum}` : null

              return (
                <div key={leftNum} className={`grid grid-cols-2 gap-0 border-b last:border-b-0 ${idx % 2 === 0 ? 'bg-white' : colors.bg}`}>
                  <div className={`border-r ${colors.border}`}>
                    {renderPenCell(leftPenLocation, colors)}
                  </div>
                  <div>
                    {renderPenCell(rightPenLocation, colors)}
                  </div>
                </div>
              )
            })}
          </div>
        </CardContent>
      </Card>
    )
  }

  const renderEFGCombinedCard = () => {
    const eColors = penZoneColors['E']
    const fColors = penZoneColors['F']
    const gColors = penZoneColors['G']

    const ePenNumbers = penNumbersByZone['E'] || []
    const fPenNumbers = penNumbersByZone['F'] || []
    const gPenNumbers = penNumbersByZone['G'] || []

    const rightColumnPens = [
      ...fPenNumbers.map(num => `F${num}`),
      ...gPenNumbers.map(num => `G${num}`),
    ]
    const leftColumnPens = ePenNumbers.map(num => `E${num}`)
    const maxRows = Math.max(leftColumnPens.length, rightColumnPens.length)

    let eTotalAnimals = 0, fTotalAnimals = 0, gTotalAnimals = 0
    ePenNumbers.forEach(num => { eTotalAnimals += (animalsByPenLocation.get(`E${num}`) || []).length })
    fPenNumbers.forEach(num => { fTotalAnimals += (animalsByPenLocation.get(`F${num}`) || []).length })
    gPenNumbers.forEach(num => { gTotalAnimals += (animalsByPenLocation.get(`G${num}`) || []).length })

    return (
      <Card key="EFG" className="bg-gradient-to-r from-purple-50 via-amber-50 to-green-50 border-2 border-purple-300">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center gap-3 text-lg flex-wrap">
            <div className="flex items-center gap-2">
              <span className={`w-8 h-8 rounded-lg ${eColors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>E</span>
              <span className={eColors.text}>E 區</span>
              <Badge variant="outline" className={`${eColors.text} ${eColors.border}`}>{eTotalAnimals} 隻</Badge>
            </div>
            <span className="text-slate-300">|</span>
            <div className="flex items-center gap-2">
              <span className={`w-8 h-8 rounded-lg ${fColors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>F</span>
              <span className={fColors.text}>F 區</span>
              <Badge variant="outline" className={`${fColors.text} ${fColors.border}`}>{fTotalAnimals} 隻</Badge>
            </div>
            <span className="text-slate-300">|</span>
            <div className="flex items-center gap-2">
              <span className={`w-8 h-8 rounded-lg ${gColors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>G</span>
              <span className={gColors.text}>G 區</span>
              <Badge variant="outline" className={`${gColors.text} ${gColors.border}`}>{gTotalAnimals} 隻</Badge>
            </div>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="bg-white rounded-lg shadow-sm overflow-hidden">
            <div className="grid grid-cols-2 gap-0 border-b">
              <div className={`grid grid-cols-5 gap-1 px-3 py-2 text-xs font-semibold ${eColors.header} text-white`}>
                <div>欄位</div><div>耳號</div><div>獸醫檢視</div><div>最新異常</div><div className="text-center">操作</div>
              </div>
              <div className="grid grid-cols-5 gap-1 px-3 py-2 text-xs font-semibold bg-gradient-to-r from-amber-500 to-green-500 text-white border-l border-white/30">
                <div>欄位</div><div>耳號</div><div>獸醫檢視</div><div>最新異常</div><div className="text-center">操作</div>
              </div>
            </div>
            {Array.from({ length: maxRows }).map((_, idx) => {
              const leftPenLocation = leftColumnPens[idx] || null
              const rightPenLocation = rightColumnPens[idx] || null
              const rightZone = rightPenLocation?.charAt(0) || ''
              const rightColors = penZoneColors[rightZone] || { bg: 'bg-gray-50', border: 'border-gray-300', header: 'bg-gray-500', text: 'text-gray-700' }
              const isTransition = idx > 0 && rightPenLocation?.startsWith('G') && rightColumnPens[idx - 1]?.startsWith('F')

              return (
                <div
                  key={idx}
                  className={`grid grid-cols-2 gap-0 border-b last:border-b-0 ${isTransition ? 'border-t-2 border-t-green-400' : ''} ${idx % 2 === 0 ? 'bg-white' : 'bg-slate-50'}`}
                >
                  <div className={`border-r ${eColors.border}`}>
                    {renderPenCell(leftPenLocation, eColors)}
                  </div>
                  <div className={rightPenLocation ? rightColors.bg : ''}>
                    {renderPenCell(rightPenLocation, rightColors)}
                  </div>
                </div>
              )
            })}
          </div>
        </CardContent>
      </Card>
    )
  }

  const currentZones = penZonesByBuilding[buildingTab] || []

  return (
    <div className="space-y-4">
      {/* Building Tabs */}
      <div className="flex gap-2 border-b">
        {penBuildings.map(building => {
          const zones = penZonesByBuilding[building.value]
          return (
            <button
              key={building.value}
              onClick={() => setBuildingTab(building.value as 'A' | 'B')}
              className={`px-6 py-3 text-sm font-medium border-b-2 transition-colors flex items-center gap-2 ${
                buildingTab === building.value
                  ? 'border-purple-600 text-purple-600 bg-purple-50'
                  : 'border-transparent text-slate-500 hover:text-slate-700 hover:bg-slate-50'
              }`}
            >
              <MapPin className="h-4 w-4" />
              {building.label}
              <div className="flex gap-1 ml-2">
                {zones.map(zone => (
                  <span
                    key={zone}
                    className={`w-5 h-5 rounded text-xs font-bold flex items-center justify-center text-white ${penZoneColors[zone]?.header || 'bg-gray-500'}`}
                  >
                    {zone}
                  </span>
                ))}
              </div>
            </button>
          )
        })}
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
        </div>
      ) : buildingTab === 'A' ? (
        <div className="space-y-6">
          {currentZones.map(zone => renderStandardZoneCard(zone))}
        </div>
      ) : (
        <div className="space-y-6">
          {renderStandardZoneCard('B')}
          {renderEFGCombinedCard()}
        </div>
      )}
    </div>
  )
}
