import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'

import api from '@/lib/api'
import { STALE_TIME } from '@/lib/query'
import { useGuestQuery } from '@/hooks/useGuestQuery'
import {
  DEMO_ANIMALS_PAGINATED,
  DEMO_ANIMAL_STATS,
  DEMO_ANIMALS_BY_PEN,
} from '@/lib/guest-demo'
import type { AnimalListItem, AnimalSource, AnimalStatsResponse } from '@/types/animal'
import type { PaginatedResponse } from '@/types/common'

interface QueryOptions {
  statusFilter: string
  breedFilter: string
  appliedSearch: string
  page: number
  perPage: number
}

export function useAnimalsQueries({ statusFilter, breedFilter, appliedSearch, page, perPage }: QueryOptions) {
  const { data: statsData } = useGuestQuery(DEMO_ANIMAL_STATS as AnimalStatsResponse, {
    queryKey: ['animals-stats'],
    queryFn: async () => {
      const res = await api.get<AnimalStatsResponse>('/animals/stats')
      return res.data
    },
    staleTime: STALE_TIME.REFERENCE,
  })

  const { data: animalsResp, isLoading } = useGuestQuery(
    DEMO_ANIMALS_PAGINATED as unknown as PaginatedResponse<AnimalListItem>,
    {
      queryKey: ['animals', statusFilter, breedFilter, appliedSearch, page],
      queryFn: async () => {
        const params = new URLSearchParams()
        if (statusFilter && statusFilter !== 'all' && statusFilter !== 'pen') params.append('status', statusFilter)
        if (breedFilter && breedFilter !== 'all') params.append('breed', breedFilter)
        if (appliedSearch) params.append('keyword', appliedSearch)
        params.append('page', String(page))
        params.append('per_page', String(perPage))
        const res = await api.get<PaginatedResponse<AnimalListItem>>(`/animals?${params}`)
        return res.data
      },
      staleTime: STALE_TIME.LIST,
    },
  )

  const { data: sourcesData } = useQuery({
    queryKey: ['animal-sources'],
    queryFn: async () => {
      const res = await api.get<AnimalSource[]>('/animal-sources')
      return res.data
    },
    staleTime: STALE_TIME.REFERENCE,
  })

  const { data: groupedData, isLoading: groupedLoading } = useGuestQuery(
    DEMO_ANIMALS_BY_PEN as unknown as { pen_location: string; animals: AnimalListItem[] }[],
    {
      queryKey: ['animals-by-pen'],
      queryFn: async () => {
        const res = await api.get<{ pen_location: string; animals: AnimalListItem[] }[]>('/animals/by-pen')
        return res.data
      },
      enabled: statusFilter === 'pen',
      staleTime: STALE_TIME.REALTIME,
      refetchOnWindowFocus: true,
    },
  )

  const animals = animalsResp?.data ?? []
  const totalPages = animalsResp?.total_pages ?? 1
  const totalAnimals = animalsResp?.total ?? 0

  /** 欄位頁：搜尋／品種僅套用在「有欄位」的動物（by-pen 資料），不包含無欄位歷史動物 */
  const penViewGroupedData = useMemo(() => {
    if (statusFilter !== 'pen' || !groupedData) return groupedData
    const kw = (appliedSearch ?? '').trim().toLowerCase()
    const hasKeyword = kw.length > 0
    const breedOk = (a: AnimalListItem) => breedFilter === 'all' || (a.breed && String(a.breed).toLowerCase() === breedFilter.toLowerCase())
    const keywordOk = (a: AnimalListItem) =>
      !hasKeyword ||
      [a.ear_tag, a.pen_location, a.iacuc_no].some(
        (v) => v && String(v).toLowerCase().includes(kw)
      )
    return groupedData
      .map((group) => ({
        pen_location: group.pen_location,
        animals: group.animals.filter((a) => breedOk(a) && keywordOk(a)),
      }))
      .filter((group) => group.animals.length > 0)
  }, [statusFilter, groupedData, appliedSearch, breedFilter])

  /** 欄位頁有搜尋/品種時：攤平為列表，用於顯示表格（圖二） */
  const penViewAnimals = useMemo(
    () => (statusFilter === 'pen' && penViewGroupedData ? penViewGroupedData.flatMap((g) => g.animals) : []),
    [statusFilter, penViewGroupedData]
  )

  const statusCounts = statsData?.status_counts ?? {}
  const penAnimalsCount = statsData?.pen_animals_count ?? 0
  const allAnimalsCount = statsData?.total ?? 0

  return {
    animals, sourcesData, groupedData, isLoading, groupedLoading,
    totalPages, totalAnimals, penViewGroupedData, penViewAnimals,
    statusCounts, penAnimalsCount, allAnimalsCount,
  }
}
