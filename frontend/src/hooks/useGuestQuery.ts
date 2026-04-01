import { useQuery, type UseQueryOptions, type UseQueryResult } from '@tanstack/react-query'

import { useAuthStore } from '@/stores/auth'

/**
 * TanStack Query wrapper for guest demo mode.
 * When the current user is a guest, returns static demo data
 * without firing any API requests.
 */
export function useGuestQuery<T>(
  demoData: T,
  options: UseQueryOptions<T, Error, T>,
): UseQueryResult<T, Error> {
  const isGuest = useAuthStore((s) => s.isGuest)()

  return useQuery<T, Error, T>({
    ...options,
    enabled: isGuest ? false : (options.enabled ?? true),
    initialData: isGuest ? demoData : undefined,
    staleTime: isGuest ? Infinity : options.staleTime,
  })
}
