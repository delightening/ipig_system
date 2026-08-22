import { useQuery } from '@tanstack/react-query'
import { useMemo } from 'react'
import { facilityApi } from '@/lib/api/facility'
import { STALE_TIME } from '@/lib/query'
import type { SpeciesListItem } from '@/types/facility'

// 模組層常數：`= []` 每次 render 都是新 reference，會害下游的 useMemo 失去穩定性
const EMPTY_SPECIES: SpeciesListItem[] = []

/**
 * 動物表單可選的物種清單。
 *
 * 取「葉節點」物種——自己底下沒有子物種的那些。物種主檔是兩層階層（豬 → 迷你豬 / 白豬），
 * 選頂層的「豬」沒有意義，要選它的子層；而 admin 自助新增的物種（例如「山羊」）
 * 沒有子層、本身就是葉節點，因此不必改任何程式碼就會出現在表單裡。
 *
 * 註：清單 API 只回傳啟用中的物種，停用的不會出現在這裡。
 */
export function useBreedSpecies() {
  const { data: speciesList = EMPTY_SPECIES } = useQuery({
    queryKey: ['facility-species'],
    queryFn: async () => (await facilityApi.listSpecies()).data,
    staleTime: STALE_TIME.REFERENCE,
  })

  const breedSpecies = useMemo(
    () => speciesList.filter(s => !speciesList.some(other => other.parent_id === s.id)),
    [speciesList],
  )

  return { breedSpecies }
}
