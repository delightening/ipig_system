import { useQuery } from '@tanstack/react-query'

export interface InternalUserBrief {
  id: string
  display_name: string
  /** 是否為本場受僱人員。`includeExternal` 關閉時恆為 true */
  is_internal: boolean
}

/**
 * 人員精簡清單（id + display_name + is_internal），來源 `/hr/internal-users`。
 * 供部門主管下拉、部門成員指派等 ≥2 處復用（DRY）。
 *
 * @param enabled 控制查詢啟用時機（例如 dialog 開啟時才抓）。
 * @param includeExternal 一併列出外部人員。
 *
 * `includeExternal` 存在的理由：**「屬於哪個部門」與「適不適用人事作業」
 * 是兩件正交的事**。IACUC 是內部部門卻聘用外部委員，那些委員需要被編入
 * 部門（組織圖、審查流程），但不該出現在特休／加班／訓練清單裡。
 * 預設關閉，讓常見情況（把同仁編進試驗部）的名單維持乾淨；需要編入外聘
 * 人員時由使用者做一個**明確的動作**才看得到他們。
 *
 * 後端對放寬名單另外要求 `facility.manage` 權限。
 */
export function useInternalUsersBrief(enabled = true, includeExternal = false) {
  return useQuery({
    // includeExternal 必須進 queryKey——否則開關切換時會拿到上一份快取，
    // 使用者會以為「打開了但外部人員沒出現」。
    queryKey: ['internal-users-brief', includeExternal],
    queryFn: async () => {
      const qs = includeExternal ? '?include_external=true' : ''
      const res = await import('@/lib/api').then(m =>
        m.default.get<InternalUserBrief[]>(`/hr/internal-users${qs}`),
      )
      return res.data
    },
    enabled,
  })
}
