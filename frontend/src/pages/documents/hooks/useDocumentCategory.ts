import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { STALE_TIME } from '@/lib/query'

export type DocCategory = 'purchasing' | 'sales' | 'warehouse'

const PREF_KEY = 'document_default_category'

interface UseDocumentCategory {
  activeCategory: DocCategory | null
  setCategory: (cat: DocCategory | null) => void
  isLoadingPref: boolean
}

export function useDocumentCategory(): UseDocumentCategory {
  const queryClient = useQueryClient()
  const [activeCategory, setActiveCategoryState] = useState<DocCategory | null>(null)
  const [prefLoaded, setPrefLoaded] = useState(false)

  const { data: prefData, isLoading: isLoadingPref } = useQuery({
    queryKey: queryKeys.users.preferences(PREF_KEY),
    queryFn: async () => {
      const res = await api.get<{ key: string; value: DocCategory | null }>(
        `/me/preferences/${PREF_KEY}`
      )
      return res.data?.value ?? null
    },
    staleTime: STALE_TIME.SETTINGS,
  })

  useEffect(() => {
    if (!isLoadingPref && !prefLoaded) {
      setActiveCategoryState(prefData ?? null)
      setPrefLoaded(true)
    }
  }, [isLoadingPref, prefData, prefLoaded])

  const savePrefMutation = useMutation({
    mutationFn: (cat: DocCategory | null) =>
      api.put(`/me/preferences/${PREF_KEY}`, { value: cat }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.users.preferences(PREF_KEY) })
    },
  })

  const setCategory = (cat: DocCategory | null) => {
    setActiveCategoryState(cat)
    savePrefMutation.mutate(cat)
  }

  return { activeCategory, setCategory, isLoadingPref: isLoadingPref && !prefLoaded }
}
