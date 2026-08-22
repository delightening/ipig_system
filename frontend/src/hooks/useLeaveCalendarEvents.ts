/**
 * 請假行事曆 Hook（原生資料，非 Google 同步）
 *
 * 與 `useCalendarEvents` 平行存在，刻意不合併：
 * 前者打 `/hr/calendar/events`（Google 已同步的**已核准**事件），
 * 本 hook 打 `/hr/leaves/calendar`（直接讀 leave_requests，含**審核中**的假單，
 * 且可見範圍由後端依觀看者的部門與代理關係決定）。
 */
import { useState, useCallback, useMemo } from 'react'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { format, startOfMonth, endOfMonth, addDays } from 'date-fns'
import api from '@/lib/api'
import { LEAVE_TYPE_COLORS } from './useCalendarEvents'
import type { FullCalendarEvent } from './useCalendarEvents'
import type { LeaveCalendarEntry, LeaveCalendarResponse, LeaveCalendarScope } from '@/types/hr'

/** 「與我相關」= 我請的假 + 我要代理的假 */
export type LeaveCalendarFilter = 'all' | 'mine'

/**
 * 把 6 位色碼加上透明度後綴。
 *
 * 審核中的假單要看得出「還沒定案」，但不能淡到看不見——'55'（約 33%）
 * 在淺底與深底下都還讀得出顏色，同時與已核准的實心色塊明顯有別。
 */
function withAlpha(hex: string): string {
    return /^#[0-9a-fA-F]{6}$/.test(hex) ? `${hex}55` : hex
}

/**
 * 營運時區固定為台北。
 *
 * 帶時間的事件必須寫成含位移的絕對時刻——不加位移的 `YYYY-MM-DDTHH:mm:ss`
 * 會被 FullCalendar 依**瀏覽器**時區解讀，而 `CalendarView` 的 popover 一律
 * 用 `Asia/Taipei` 格式化，兩者在非 UTC+8 的裝置上會顯示不同時間。
 */
const TAIPEI_OFFSET = '+08:00'

/**
 * 一筆假單 → FullCalendar 事件。
 *
 * 標題沿用既有格式 `[假別] 員工名（代理人）`，讓 `CalendarView` 的
 * `parseEventTitle` 與假別篩選 chips 完全不必改動即可吃這份資料。
 * 代理人未確認時在括號內就標出來，避免只看色塊的人誤以為代理已成立。
 */
function toEvent(entry: LeaveCalendarEntry): FullCalendarEvent {
    const proxySuffix = entry.proxy_user_name
        ? `（${entry.proxy_user_name}${entry.proxy_confirmed ? '' : '・未確認'}）`
        : ''
    // 不足整日的假必須標出時數：請假表單只收時數不收時段，這種假在日曆上
    // 只能畫成整天色塊，不標時數的話 4.5 小時的假與整天假長得一模一樣。
    const hoursSuffix =
        entry.is_partial_day && entry.total_hours ? ` ${Number(entry.total_hours)}h` : ''
    const baseColor = LEAVE_TYPE_COLORS[entry.leave_type_display]
    const isPending = Boolean(entry.pending_stage_display)

    // FullCalendar 全天事件的 end 是排他的，資料庫的 end_date 是包含的，故 +1 天。
    const end = entry.is_all_day
        ? format(addDays(new Date(`${entry.end_date}T00:00:00`), 1), 'yyyy-MM-dd')
        : `${entry.end_date}T${entry.end_time ?? '18:00:00'}${TAIPEI_OFFSET}`
    const start = entry.is_all_day
        ? entry.start_date
        : `${entry.start_date}T${entry.start_time ?? '09:00:00'}${TAIPEI_OFFSET}`

    return {
        id: entry.id,
        title: `[${entry.leave_type_display}] ${entry.user_name}${proxySuffix}${hoursSuffix}`,
        start,
        end,
        allDay: entry.is_all_day,
        ...(baseColor
            ? {
                  backgroundColor: isPending ? withAlpha(baseColor) : baseColor,
                  borderColor: baseColor,
              }
            : {}),
        ...(isPending ? { classNames: ['ipig-leave-pending'] } : {}),
        extendedProps: {
            ...(entry.pending_stage_display ? { pendingStage: entry.pending_stage_display } : {}),
            ...(entry.proxy_user_name ? { proxyConfirmed: entry.proxy_confirmed } : {}),
            ...(entry.department_name ? { departmentName: entry.department_name } : {}),
            isMine: entry.is_mine,
            isMyProxyDuty: entry.is_my_proxy_duty,
        },
    }
}

export function useLeaveCalendarEvents(isActive: boolean, filter: LeaveCalendarFilter = 'all') {
    const [range, setRange] = useState(() => ({
        start: startOfMonth(new Date()),
        end: endOfMonth(new Date()),
    }))

    // 查詢鍵用格式化後的字串而非 Date 物件：Date 每次 render 都是新參考，
    // 直接放進 key 會讓 react-query 認為條件變了而不停重抓。
    const startStr = format(range.start, 'yyyy-MM-dd')
    const endStr = format(range.end, 'yyyy-MM-dd')

    const { data, isLoading, isFetching, isError, error, refetch } = useQuery({
        queryKey: ['leave-calendar', startStr, endStr],
        queryFn: async () => {
            const res = await api.get<LeaveCalendarResponse>(
                `/hr/leaves/calendar?start_date=${startStr}&end_date=${endStr}`,
            )
            return res.data
        },
        enabled: isActive,
        placeholderData: keepPreviousData,
        staleTime: 5 * 60 * 1000,
    })

    const handleDatesSet = useCallback((info: { start: Date; end: Date }) => {
        setRange(prev => {
            // 只比對日期字串：FullCalendar 每次都給新的 Date 物件，
            // 直接比參考會在重新掛載時觸發多餘的請求。
            const same =
                format(prev.start, 'yyyy-MM-dd') === format(info.start, 'yyyy-MM-dd') &&
                format(prev.end, 'yyyy-MM-dd') === format(info.end, 'yyyy-MM-dd')
            return same ? prev : { start: info.start, end: info.end }
        })
    }, [])

    const allEntries = useMemo(() => data?.entries ?? [], [data])

    // 篩選在同一處完成，events 與 entries 永遠是同一組資料的兩種表示。
    // 先前 entries 未套篩選，導致「與我相關」時圖例的審核中筆數與畫面上的
    // 虛線色塊數量對不上。
    const entries = useMemo(
        () =>
            filter === 'mine'
                ? allEntries.filter(e => e.is_mine || e.is_my_proxy_duty)
                : allEntries,
        [allEntries, filter],
    )

    const events = useMemo(() => entries.map(toEvent), [entries])

    return {
        events,
        entries,
        scope: (data?.scope ?? 'department') as LeaveCalendarScope,
        isLoading,
        isFetching,
        // 查詢失敗必須與「查詢成功但沒有假單」分開回報：兩者都給空陣列的話，
        // 後端掛掉會顯示成「這段期間沒有請假紀錄」，把不可用的排班資料
        // 冒充成可信的空結果。
        isError,
        error,
        refetch,
        handleDatesSet,
    }
}
