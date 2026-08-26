/**
 * 補卡的時間換算。
 *
 * `attendance_records` 的 clock_in_time / clock_out_time 存的是 **UTC**，
 * 而補卡表單讓使用者填的是台灣時間的 HH:mm。直接把 `<input type="time">` 的值
 * 丟給後端會差 8 小時——這兩個函式是唯一的換算點，兩個方向都走它。
 */

/** 台灣時區固定 UTC+8（本系統無日光節約時間需求，寫死 offset 即可） */
const TAIPEI_UTC_OFFSET = '+08:00'

/** 補卡理由的最短長度，與後端 `MIN_CORRECTION_REASON_CHARS` 同值（4 字，讓「忘記打卡」過得了） */
export const MIN_CORRECTION_REASON_LENGTH = 4

/**
 * UTC ISO 字串 → `<input type="time">` 用的台灣時間 `HH:mm`。
 * 空值或無法解析回空字串（表單顯示為未填）。
 */
export function isoToTaipeiTimeInput(iso: string | null): string {
    if (!iso) return ''
    const date = new Date(iso)
    if (Number.isNaN(date.getTime())) return ''
    // en-GB + hour12:false 保證輸出 `HH:mm`（不受 UI 語系影響，這是表單值不是顯示文字）
    return date.toLocaleTimeString('en-GB', {
        timeZone: 'Asia/Taipei',
        hour: '2-digit',
        minute: '2-digit',
        hour12: false,
    })
}

/**
 * 台灣時間 `HH:mm` + 工作日期 `yyyy-MM-dd` → UTC ISO 字串。
 * 任一為空回 `null`（代表這一邊不填，後端會保留原值 / 視為缺漏）。
 */
export function taipeiTimeInputToIso(workDate: string, time: string): string | null {
    if (!workDate || !time) return null
    const date = new Date(`${workDate}T${time}:00${TAIPEI_UTC_OFFSET}`)
    if (Number.isNaN(date.getTime())) return null
    return date.toISOString()
}
