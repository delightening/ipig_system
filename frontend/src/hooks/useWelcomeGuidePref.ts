import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { STALE_TIME } from '@/lib/query'

/**
 * 歡迎指引偏好的 query key。
 *
 * DashboardPage / RoleWelcomeGuide / DisplayPreferencesCard 共用同一把 key，
 * React Query 會把同一 render 週期內的多處呼叫去重成一個請求；
 * 寫入端（設定頁的 toggle mutation）也用這把 key 做樂觀更新。
 */
export const WELCOME_GUIDE_PREF_KEY = ['user-preferences', 'show_welcome_guide']

/** useWelcomeGuidePref 的回傳值 */
export interface WelcomeGuidePref {
  /** 是否顯示歡迎指引（後端未設定時預設顯示）。isPending 為 true 時此值尚未定案，不可據以渲染 */
  enabled: boolean
  /** 偏好尚未取回 */
  isPending: boolean
}

/**
 * 讀取「顯示歡迎指引」使用者偏好。
 *
 * 呼叫端務必先看 `isPending`：偏好未到手前 `enabled` 只是樂觀預設值（true），
 * 若直接拿去渲染，會發生「橫幅先畫出來、偏好回報已關閉後再整塊消失」——
 * 橫幅位在儀表板網格上方，消失時會把下方版面往上拉約 100px，就是登入時看到的跳動。
 */
export function useWelcomeGuidePref(): WelcomeGuidePref {
  const { data, isPending } = useQuery({
    queryKey: WELCOME_GUIDE_PREF_KEY,
    queryFn: async () => {
      const res = await api.get<{ key: string; value: boolean }>(
        '/me/preferences/show_welcome_guide',
      )
      return res.data.value
    },
    staleTime: STALE_TIME.SETTINGS,
  })

  return { enabled: data ?? true, isPending }
}
