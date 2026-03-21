import { QueryClient } from '@tanstack/react-query'

export function useTransferInvalidate(animalId: string, queryClient: QueryClient) {
    return () => {
        queryClient.invalidateQueries({ queryKey: ['animal-transfers', animalId] })
        queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
        queryClient.invalidateQueries({ queryKey: ['animals'] })
        queryClient.invalidateQueries({ queryKey: ['animals-stats'] })
        queryClient.invalidateQueries({ queryKey: ['animals-by-pen'] })
    }
}
