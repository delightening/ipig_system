import { useQuery, type UseQueryOptions, type UseQueryResult } from '@tanstack/react-query'

import { useAuthStore } from '@/stores/auth'

/**
 * TanStack Query wrapper for guest demo mode.
 *
 * When the current user is a guest:
 *   - Overrides queryFn to return demoData directly (no API call)
 *   - staleTime: Infinity prevents any refetch
 *   - initialData ensures zero-latency first render (no loading flash)
 *
 * When auth is not yet initialized (any role):
 *   - Blocks the query (enabled: false) to prevent premature fetches
 *   before isGuest() is reliable.
 */
export function useGuestQuery<T>(
  demoData: T,
  options: UseQueryOptions<T, Error, T>,
): UseQueryResult<T, Error> {
  const isGuestFn = useAuthStore((s) => s.isGuest)
  const isGuest = isGuestFn()
  const isInitialized = useAuthStore((s) => s.isInitialized)

  return useQuery<T, Error, T>({
    ...options,
    // Guest: always run, but queryFn resolves with demo data (no network call)
    // Non-guest pre-init: block until auth is resolved
    queryFn: isGuest ? () => Promise.resolve(demoData) : options.queryFn,
    enabled: isGuest ? true : (isInitialized ? (options.enabled ?? true) : false),
    // initialData prevents loading flash on first render for guests
    initialData: isGuest ? demoData : undefined,
    staleTime: isGuest ? Infinity : options.staleTime,
  })
}
