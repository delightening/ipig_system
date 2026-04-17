import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'

import { facilityApi } from '@/lib/api/facility'
import { STALE_TIME } from '@/lib/query'
import type { BuildingWithFacility, ZoneWithBuilding, PenDetails } from '@/types/facility'

/** 色系映射：zone.color → Tailwind class 組合 */
const COLOR_SCHEMES: Record<string, ZoneColorSet> = {
  blue:   { bg: 'bg-blue-100',   border: 'border-blue-300',   header: 'bg-status-info-solid',     text: 'text-status-info-text' },
  orange: { bg: 'bg-orange-50',  border: 'border-orange-300', header: 'bg-orange-500',            text: 'text-orange-700' },
  yellow: { bg: 'bg-yellow-50',  border: 'border-yellow-300', header: 'bg-yellow-500',            text: 'text-yellow-700' },
  cyan:   { bg: 'bg-cyan-50',    border: 'border-cyan-300',   header: 'bg-cyan-500',              text: 'text-cyan-700' },
  purple: { bg: 'bg-purple-50',  border: 'border-purple-300', header: 'bg-purple-500',            text: 'text-purple-700' },
  amber:  { bg: 'bg-amber-100',  border: 'border-amber-300',  header: 'bg-status-warning-solid',  text: 'text-status-warning-text' },
  green:  { bg: 'bg-green-100',  border: 'border-green-300',  header: 'bg-status-success-solid',  text: 'text-status-success-text' },
  red:    { bg: 'bg-red-100',    border: 'border-red-300',    header: 'bg-status-error-solid',    text: 'text-status-error-text' },
  gray:   { bg: 'bg-muted',      border: 'border-border',     header: 'bg-muted-foreground',      text: 'text-foreground' },
}

const DEFAULT_COLORS: ZoneColorSet = COLOR_SCHEMES.gray

export interface ZoneColorSet {
  bg: string
  border: string
  header: string
  text: string
  /** 當使用 hex 色碼時，提供 inline style 物件 */
  headerStyle?: React.CSSProperties
  borderStyle?: React.CSSProperties
}

export function getZoneColors(color: string | null): ZoneColorSet {
  if (!color) return DEFAULT_COLORS

  // 優先匹配 named color
  if (COLOR_SCHEMES[color]) return COLOR_SCHEMES[color]

  // hex 色碼 → 動態生成 inline style 版本
  const hex = color.startsWith('#') ? color : `#${color}`
  return {
    bg: '',
    border: '',
    header: '',
    text: '',
    headerStyle: { backgroundColor: hex },
    borderStyle: { borderColor: hex },
  }
}

export interface FacilityLayoutData {
  buildings: BuildingWithFacility[]
  zonesByBuilding: Record<string, ZoneWithBuilding[]>
  pensByZone: Record<string, PenDetails[]>
  isLoading: boolean
}

export function useFacilityLayout(): FacilityLayoutData {
  const { data: buildings = [], isLoading: buildingsLoading } = useQuery({
    queryKey: ['facility-buildings'],
    queryFn: async () => (await facilityApi.listBuildings()).data,
    staleTime: STALE_TIME.REFERENCE,
  })

  const { data: zones = [], isLoading: zonesLoading } = useQuery({
    queryKey: ['facility-zones'],
    queryFn: async () => (await facilityApi.listZones()).data,
    staleTime: STALE_TIME.REFERENCE,
  })

  const { data: pens = [], isLoading: pensLoading } = useQuery({
    queryKey: ['facility-pens'],
    queryFn: async () => (await facilityApi.listPens()).data,
    staleTime: STALE_TIME.REFERENCE,
  })

  const zonesByBuilding = useMemo(() => {
    const map: Record<string, ZoneWithBuilding[]> = {}
    for (const z of zones) {
      if (!map[z.building_id]) map[z.building_id] = []
      map[z.building_id].push(z)
    }
    return map
  }, [zones])

  const pensByZone = useMemo(() => {
    const map: Record<string, PenDetails[]> = {}
    for (const p of pens) {
      if (!map[p.zone_id]) map[p.zone_id] = []
      map[p.zone_id].push(p)
    }
    for (const key of Object.keys(map)) {
      map[key].sort((a, b) => a.code.localeCompare(b.code))
    }
    return map
  }, [pens])

  return {
    buildings,
    zonesByBuilding,
    pensByZone,
    isLoading: buildingsLoading || zonesLoading || pensLoading,
  }
}
