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
  ProtocolListItem,
} from '@/lib/api'
import { getErrorMessage } from '@/types/error'
import { logger } from '@/lib/logger'
import { toast } from '@/components/ui/use-toast'

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

  const { data: dataBoundary } = useQuery({
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

  const { data: approvedProtocols } = useQuery({
    queryKey: ['approved-protocols'],
    queryFn: async () => {
      const res = await api.get<ProtocolListItem[]>('/protocols')
      return res.data.filter(p => {
        if (p.status === 'CLOSED') return false
        return (p.status === 'APPROVED' || p.status === 'APPROVED_WITH_CONDITIONS') && p.iacuc_no
      })
    },
    staleTime: 60_000,
  })

  const { data: observations, error: observationsError } = useQuery({
    queryKey: ['animal-observations', animalId, afterParam],
    queryFn: async () => {
      const res = await api.get<AnimalObservation[]>(
        `/animals/${animalId}/observations${afterParam}`,
      )
      return res.data
    },
    enabled: activeTab === 'observations' || activeTab === 'timeline',
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
    enabled: activeTab === 'surgeries' || activeTab === 'timeline',
  })

  const { data: weights } = useQuery({
    queryKey: ['animal-weights', animalId, afterParam],
    queryFn: async () => {
      const res = await api.get<AnimalWeight[]>(
        `/animals/${animalId}/weights${afterParam}`,
      )
      return res.data
    },
    enabled: activeTab === 'weights' || activeTab === 'timeline',
  })

  const { data: vaccinations } = useQuery({
    queryKey: ['animal-vaccinations', animalId, afterParam],
    queryFn: async () => {
      const res = await api.get<AnimalVaccination[]>(
        `/animals/${animalId}/vaccinations${afterParam}`,
      )
      return res.data
    },
    enabled: activeTab === 'vaccinations',
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
