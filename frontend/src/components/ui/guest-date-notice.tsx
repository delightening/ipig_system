import { Info } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useAuthIsGuest } from '@/stores/auth'

/**
 * 訪客示範模式下，日期欄位不會改變結果——這裡把這件事講出來。
 *
 * ## 為什麼需要它
 *
 * 訪客模式的資料是寫死在 `lib/guest-demo/` 的靜態夾具，而 `getGuestDemoData`
 * 在比對路由前就把 query string 剝掉了（`routes.ts` 的 `cleanPath`），
 * 所以十幾支報表的日期欄位在示範模式下**全部沒有作用**。
 * 使用者改了日期發現畫面不動，只能猜是壞了還是真的沒資料。
 *
 * ## 為什麼是「講出來」而不是「讓它真的能篩」
 *
 * 讓夾具跟著日期篩，對半數報表是**錯的語意**：
 *
 * - 庫存現況、帳齡、試算表、損益表是**快照或期間彙總**，改日期的正確行為是
 *   重算數字，不是篩掉幾列。硬篩會端出一份看起來合理、實際錯誤的示範。
 * - 案件消耗報表更極端：它的每一列是後端 `GROUP BY` 之後的彙總
 *   （`qty_base` 已加總、`first/last_trx_date` 是 MIN/MAX），跨越邊界的列
 *   必須重算才對，而夾具裡只有總數，資訊已經不在了——整列留著會顯示錯的
 *   數字，整列丟掉則是漏資料，兩個都錯。
 *
 * 示範資料的目的是展示介面，不是重現運算。與其假裝能篩，不如講清楚。
 *
 * ## 文案為什麼描述現象而不是承諾語意
 *
 * 用到這個元件的報表裡，日期輸入有兩種意思：範圍篩選（庫存流水、銷貨明細…）
 * 與 as-of 單一日期（帳齡、試算表）。文案只說「不會改變下方結果」，
 * 這對兩者都成立，也不需要隨報表類型換句話。
 *
 * ## 用法
 *
 * 放在報表的篩選列**下方**。非訪客身分時完全不渲染，正式環境看不到。
 */
export function GuestDateNotice() {
  const isGuest = useAuthIsGuest()
  const { t } = useTranslation()
  if (!isGuest) return null
  return (
    <p className="flex items-center gap-1.5 text-xs text-muted-foreground">
      <Info className="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
      {t('guest.dateFilterInactive')}
    </p>
  )
}

export default GuestDateNotice
