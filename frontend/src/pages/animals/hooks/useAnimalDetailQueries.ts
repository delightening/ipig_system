import { useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'

import api, {
  Animal,
  AnimalObservation,
  AnimalSurgery,
  AnimalWeight,
  AnimalVaccination,
  AnimalSacrifice,
  AnimalSuddenDeath,
  AnimalEvent,
  transferApi,
} from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { logger } from '@/lib/logger'
import { toast } from '@/components/ui/use-toast'
import { useAssignableProtocols } from '@/hooks/useAssignableProtocols'

import type { TabType } from '../constants'

interface UseAnimalDetailQueriesParams {
  animalId: string
  activeTab: TabType
}

export function useAnimalDetailQueries({
  animalId,
  activeTab,
}: UseAnimalDetailQueriesParams) {
  const { data: animal, isLoading: animalLoading } = useQuery({
    queryKey: ['animal', animalId],
    queryFn: async () => {
      const res = await api.get<Animal>(`/animals/${animalId}`)
      return res.data
    },
  })

  // W7：boundary 決定下方 4 個子查詢的 afterParam。若不等它 settle 就發查詢，
  // afterParam 會先以 '' 發一次、boundary 回來後 queryKey 變再發一次 = 雙重抓取。
  // 故下方 afterParam-相依查詢 gate 在 !boundaryPending（成功或錯誤皆放行，避免卡死）。
  const { data: dataBoundary, isPending: boundaryPending } = useQuery({
    queryKey: ['animal-data-boundary', animalId],
    queryFn: async () => {
      const res = await transferApi.getDataBoundary(animalId)
      return res.data
    },
    staleTime: 600_000,
  })

  const afterParam = dataBoundary?.boundary
    ? `?after=${encodeURIComponent(dataBoundary.boundary)}`
    : ''

  const { data: approvedProtocols } = useAssignableProtocols()

  const { data: observations, error: observationsError } = useQuery({
    queryKey: ['animal-observations', animalId, afterParam],
    queryFn: async () => {
      const res = await api.get<AnimalObservation[]>(
        `/animals/${animalId}/observations${afterParam}`,
      )
      return res.data
    },
    enabled: (activeTab === 'observations' || activeTab === 'timeline') && !boundaryPending,
  })

  useEffect(() => {
    if (observationsError) {
      logger.error('Failed to load observations:', observationsError)
      toast({
        title: '錯誤',
        description: getErrorMessage(observationsError) || '載入觀察紀錄失敗',
        variant: 'destructive',
      })
    }
  }, [observationsError])

  const { data: surgeries } = useQuery({
    queryKey: ['animal-surgeries', animalId, afterParam],
    queryFn: async () => {
      const res = await api.get<AnimalSurgery[]>(
        `/animals/${animalId}/surgeries${afterParam}`,
      )
      return res.data
    },
    enabled: (activeTab === 'surgeries' || activeTab === 'timeline') && !boundaryPending,
  })

  const { data: weights } = useQuery({
    queryKey: ['animal-weights', animalId, afterParam],
    queryFn: async () => {
      const res = await api.get<AnimalWeight[]>(
        `/animals/${animalId}/weights${afterParam}`,
      )
      return res.data
    },
    enabled: (activeTab === 'weights' || activeTab === 'timeline') && !boundaryPending,
  })

  const { data: vaccinations } = useQuery({
    queryKey: ['animal-vaccinations', animalId, afterParam],
    queryFn: async () => {
      const res = await api.get<AnimalVaccination[]>(
        `/animals/${animalId}/vaccinations${afterParam}`,
      )
      return res.data
    },
    enabled: activeTab === 'vaccinations' && !boundaryPending,
  })

  const { data: sacrifice } = useQuery({
    queryKey: ['animal-sacrifice', animalId],
    queryFn: async () => {
      const res = await api.get<AnimalSacrifice>(`/animals/${animalId}/sacrifice`)
      return res.data
    },
    enabled: activeTab === 'sacrifice' || activeTab === 'timeline',
  })

  const { data: suddenDeath } = useQuery({
    queryKey: ['animal-sudden-death', animalId],
    queryFn: async () => {
      const res = await api.get<AnimalSuddenDeath>(`/animals/${animalId}/sudden-death`)
      return res.data
    },
    enabled: activeTab === 'timeline',
  })

  const { data: iacucEvents } = useQuery({
    queryKey: ['animal-iacuc-events', animalId],
    queryFn: async () => {
      const res = await api.get<AnimalEvent[]>(`/animals/${animalId}/events`)
      return res.data
    },
    enabled: activeTab === 'timeline',
  })

  const { data: transfers } = useQuery({
    queryKey: ['animal-transfers', animalId],
    queryFn: async () => {
      const res = await transferApi.list(animalId)
      return res.data
    },
  })

  return {
    animal,
    animalLoading,
    afterParam,
    approvedProtocols,
    observations,
    surgeries,
    weights,
    vaccinations,
    sacrifice,
    suddenDeath,
    iacucEvents,
    transfers,
  }
}
