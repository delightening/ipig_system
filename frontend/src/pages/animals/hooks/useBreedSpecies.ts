import { useQuery } from '@tanstack/react-query'
import { facilityApi } from '@/lib/api/facility'
import { STALE_TIME } from '@/lib/query'

export function useBreedSpecies() {
  const { data: speciesList = [] } = useQuery({
    queryKey: ['facility-species'],
    queryFn: async () => (await facilityApi.listSpecies()).data,
    staleTime: STALE_TIME.REFERENCE,
  })
  const breedSpecies = speciesList.filter(s => s.parent_id !== null)
  return { breedSpecies }
}
