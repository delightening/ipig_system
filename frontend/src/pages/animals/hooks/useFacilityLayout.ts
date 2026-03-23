import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'

import { facilityApi } from '@/lib/api/facility'
import { STALE_TIME } from '@/lib/query'
import type { BuildingWithFacility, ZoneWithBuilding, PenDetails } from '@/types/facility'

/** 色系映射：zone.color → Tailwind class 組合 */
const COLOR_SCHEMES: Record<string, { bg: string; border: string; header: string; text: string }> = {
  blue:   { bg: 'bg-blue-50',   border: 'border-blue-300',   header: 'bg-blue-500',   text: 'text-blue-700' },
  orange: { bg: 'bg-orange-50', border: 'border-orange-300', header: 'bg-orange-500', text: 'text-orange-700' },
  yellow: { bg: 'bg-yellow-50', border: 'border-yellow-300', header: 'bg-yellow-500', text: 'text-yellow-700' },
  cyan:   { bg: 'bg-cyan-50',   border: 'border-cyan-300',   header: 'bg-cyan-500',   text: 'text-cyan-700' },
  purple: { bg: 'bg-purple-50', border: 'border-purple-300', header: 'bg-purple-500', text: 'text-purple-700' },
  amber:  { bg: 'bg-amber-50',  border: 'border-amber-300',  header: 'bg-amber-500',  text: 'text-amber-700' },
  green:  { bg: 'bg-green-50',  border: 'border-green-300',  header: 'bg-green-500',  text: 'text-green-700' },
  red:    { bg: 'bg-red-50',    border: 'border-red-300',    header: 'bg-red-500',    text: 'text-red-700' },
  gray:   { bg: 'bg-gray-50',   border: 'border-gray-300',   header: 'bg-gray-500',   text: 'text-gray-700' },
}

const DEFAULT_COLORS = COLOR_SCHEMES.gray

export function getZoneColors(color: string | null) {
  return color ? (COLOR_SCHEMES[color] ?? DEFAULT_COLORS) : DEFAULT_COLORS
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
    // Sort pens by code within each zone
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
