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

/**
 * 台灣日曆日 `yyyy-MM-dd` → 次日。
 *
 * 刻意用當日 **12:00 UTC** 當基準再加一天：日期字串本身沒有時間，若用 00:00 當基準，
 * 任何時區換算的捨入都可能把日期推回前一天。中午離兩端各 12 小時，不會被推過界。
 */
function nextTaipeiDate(workDate: string): string | null {
    const date = new Date(`${workDate}T12:00:00Z`)
    if (Number.isNaN(date.getTime())) return null
    date.setUTCDate(date.getUTCDate() + 1)
    return date.toISOString().slice(0, 10)
}

/**
 * 補卡／更正表單的上下班時間 → 後端要的 UTC ISO 組。
 *
 * 🔴 **夜班的下班時間屬於次日**（CodeRabbit 於 PR #35 指出）。
 * 表單只讓使用者填 `HH:mm`，日期一律取 `work_date`；夜班（例：22:00 上班、06:00 下班）
 * 若把兩邊都掛在 `work_date`，換算出來的下班會**早於**上班整整 16 小時，
 * 後端 `validate_attendance_times` 的「下班必須晚於上班」直接擋掉——
 * 也就是說**夜班在修好之前根本補登不了**。
 *
 * 判斷跨日只比 `HH:mm` 字串（同格式、零填補，字典序即時間序），不做時區運算：
 * 下班字串小於上班字串就是跨了午夜。
 *
 * ⚠️ 上班未填時不判跨日——沒有比較基準，此時下班就是 `work_date` 當天那個時刻。
 * ⚠️ 後端仍有 24 小時跨度上限把關，這裡不重複驗證（同一條規則放兩邊會走味）。
 */
export function attendanceTimesToIso(
    workDate: string,
    clockIn: string,
    clockOut: string,
): { clockInIso: string | null; clockOutIso: string | null } {
    const clockInIso = taipeiTimeInputToIso(workDate, clockIn)
    if (!clockOut) return { clockInIso, clockOutIso: null }

    const crossesMidnight = Boolean(clockIn) && clockOut < clockIn
    const clockOutDate = crossesMidnight ? nextTaipeiDate(workDate) : workDate
    return {
        clockInIso,
        clockOutIso: clockOutDate ? taipeiTimeInputToIso(clockOutDate, clockOut) : null,
    }
}
