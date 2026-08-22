import { MapPin } from 'lucide-react'

import { useFacilityLayout, getZoneColors } from '../hooks/useFacilityLayout'
import type { BuildingWithFacility, ZoneWithBuilding } from '@/types/facility'

interface BuildingTabsProps {
  /** 目前選中的棟舍 code（來自網址 `?building=`）；null → fallback 到第一棟 */
  activeBuildingCode: string | null
  onBuildingChange: (buildingCode: string) => void
  /** 各棟在欄動物數（key = building.code），來自 `/animals/stats` */
  penCountsByBuilding: Record<string, number>
  /** 目前是否停在欄位檢視。false 時整列不 highlight（選中的是下方的狀態 tag） */
  isPenView: boolean
}

function BuildingTabsSkeleton() {
  return (
    <div className="flex flex-wrap gap-2 border-b pb-2" aria-hidden="true">
      {[0, 1, 2].map(i => (
        <div key={i} className="h-9 w-28 rounded bg-muted animate-pulse" />
      ))}
    </div>
  )
}

/** 區碼色塊（A / C / D…）。顏色由使用者在設施設定自選，屬 DESIGN.md §3.3 的認可例外。 */
function ZoneChips({ zones }: { zones: ZoneWithBuilding[] }) {
  return (
    <span className="flex gap-1 ml-1">
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
    </span>
  )
}

interface BuildingTabButtonProps {
  building: BuildingWithFacility
  zones: ZoneWithBuilding[]
  count: number
  selected: boolean
  onClick: () => void
}

function BuildingTabButton({ building, zones, count, selected, onClick }: BuildingTabButtonProps) {
  return (
    <button
      type="button"
      data-testid="building-tab"
      aria-pressed={selected}
      onClick={onClick}
      className={`px-3 md:px-4 py-2 text-xs md:text-sm font-medium border-b-2 transition-colors flex items-center gap-1.5 ${
        selected
          ? 'border-primary text-primary'
          : 'border-transparent text-muted-foreground hover:text-foreground'
      }`}
    >
      <MapPin className="h-4 w-4 shrink-0" />
      {building.name} ({count})
      <ZoneChips zones={zones} />
    </button>
  )
}

/**
 * 棟別 tag 列（A 棟 / B 棟 / 檢疫舍 / 羊舍）。
 *
 * 棟舍與區碼都由 `/facilities/*` API 提供，新增棟舍不需要改這裡。
 * 每個 tag 對應一個網址（`?status=pen&building=<code>`），可直接分享或加書籤。
 */
export function BuildingTabs({
  activeBuildingCode,
  onBuildingChange,
  penCountsByBuilding,
  isPenView,
}: BuildingTabsProps) {
  const { buildings, zonesByBuilding, isLoading } = useFacilityLayout()

  if (isLoading) return <BuildingTabsSkeleton />
  if (buildings.length === 0) return null

  // 必須與 AnimalPenView 的 fallback 規則一致：網址帶了不存在的 code 時，
  // 兩邊都要落在第一棟，否則會出現「格線顯示 A 棟、但沒有任何 tag 是選中的」
  const effectiveCode =
    activeBuildingCode && buildings.some(b => b.code === activeBuildingCode)
      ? activeBuildingCode
      : buildings[0]?.code ?? null

  return (
    <div className="flex flex-wrap gap-2 border-b">
      {buildings.map(building => (
        <BuildingTabButton
          key={building.id}
          building={building}
          zones={zonesByBuilding[building.id] ?? []}
          // 沒有在欄動物的棟舍不會出現在 stats map 中，需自行 fallback 到 0
          count={penCountsByBuilding[building.code] ?? 0}
          selected={isPenView && building.code === effectiveCode}
          onClick={() => onBuildingChange(building.code)}
        />
      ))}
    </div>
  )
}
