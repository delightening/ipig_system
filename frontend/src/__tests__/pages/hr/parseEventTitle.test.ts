/**
 * `parseEventTitle` 的回歸測試。
 *
 * 這支測試存在的理由：行事曆的 popover 完全靠這個正規表示式從**標題字串**
 * 反推「假別 / 員工 / 代理人」，而標題格式由 `useLeaveCalendarEvents` 組出來。
 * 兩邊沒有型別關聯，一旦標題多了一段後綴而規則沒跟著改，
 * 症狀是 **popover 上的「代理：」那一行安靜消失**——沒有錯誤、沒有警告，
 * 只有肉眼盯畫面才看得出來。
 *
 * 實際踩過一次：加上半天假的「4.5h」後綴後，`（代理人）` 不再位於字串結尾，
 * 整個代理人群組比對失敗，所有半天假的代理人都顯示不出來。
 */
import { describe, it, expect } from 'vitest'
import { parseEventTitle, stripUnconfirmedSuffix } from '@/pages/hr/calendarEventTitle'

describe('parseEventTitle', () => {
    it('解析「假別 + 員工」', () => {
        expect(parseEventTitle('[病假] 使用者A')).toEqual({
            leaveType: '病假',
            employeeName: '使用者A',
            agentName: undefined,
            hours: undefined,
        })
    })

    it('解析「假別 + 員工 + 代理人」', () => {
        expect(parseEventTitle('[特休假] 使用者C（使用者D）')).toEqual({
            leaveType: '特休假',
            employeeName: '使用者C',
            agentName: '使用者D',
            hours: undefined,
        })
    })

    it('代理人未確認的後綴保留在代理人欄位內（由 popover 另行處理）', () => {
        expect(parseEventTitle('[特休假] 使用者C（使用者D・未確認）').agentName).toBe(
            '使用者D・未確認',
        )
    })

    it('半天假：時數後綴不得吃掉代理人', () => {
        expect(parseEventTitle('[事假] 使用者C（使用者D） 4.5h')).toEqual({
            leaveType: '事假',
            employeeName: '使用者C',
            agentName: '使用者D',
            hours: '4.5h',
        })
    })

    it('半天假且無代理人', () => {
        expect(parseEventTitle('[事假] 使用者C 4.5h')).toEqual({
            leaveType: '事假',
            employeeName: '使用者C',
            agentName: undefined,
            hours: '4.5h',
        })
    })

    it('不符格式時整串當員工名，不丟例外', () => {
        expect(parseEventTitle('某個 Google 行事曆事件')).toEqual({
            employeeName: '某個 Google 行事曆事件',
        })
    })
})

describe('stripUnconfirmedSuffix', () => {
    it('去掉「・未確認」後綴（popover 另有欄位標示，不重複講同一件事）', () => {
        expect(stripUnconfirmedSuffix('使用者D・未確認')).toBe('使用者D')
    })

    it('已確認的代理人維持原樣', () => {
        expect(stripUnconfirmedSuffix('使用者D')).toBe('使用者D')
    })

    it('沒有代理人時回 undefined，不變成空字串', () => {
        expect(stripUnconfirmedSuffix(undefined)).toBeUndefined()
    })
})
