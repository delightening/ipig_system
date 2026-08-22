/**
 * 行事曆事件標題的解析工具。
 *
 * 獨立成一個模組而非留在 `CalendarView.tsx`：從元件檔匯出非元件會破壞
 * React Fast Refresh（eslint `react-refresh/only-export-components`），
 * 而這幾個函式需要被單元測試直接呼叫。
 *
 * 標題是 Google 同步事件與原生請假行事曆的**共同介面**——兩邊都組成
 * `[假別] 員工名（代理人）` 的形狀，`CalendarView` 才能不分來源地渲染。
 */

/**
 * 解析事件標題為結構化資訊。
 *
 * 支援格式（後兩段皆可省略）：
 *   [假別] 員工名
 *   [假別] 員工名（代理人）
 *   [假別] 員工名 4.5h
 *   [假別] 員工名（代理人） 4.5h
 *
 * 括號為全形中文括號 （）。
 *
 * ⚠️ 時數後綴那一段是必要的：原生請假行事曆對不足整日的假會在標題尾端補
 * 「4.5h」（見 `useLeaveCalendarEvents`）。若正規表示式不吃這一段，
 * `（代理人）` 就不再位於字串結尾、整個代理人群組比對失敗，結果是
 * **半天假的 popover 完全看不到代理人**——不會有錯誤、不會有警告，
 * 那一行只是安靜消失。已於 `__tests__/pages/hr/parseEventTitle.test.ts` 鎖住。
 */
export function parseEventTitle(title: string): {
    leaveType?: string
    employeeName: string
    agentName?: string
    hours?: string
} {
    const match = title.match(/^\[(.+?)\]\s*(.+?)(?:（(.+?)）)?(?:\s+([\d.]+h))?\s*$/)
    if (match) {
        return {
            leaveType: match[1],
            employeeName: match[2].trim(),
            agentName: match[3]?.trim(),
            hours: match[4],
        }
    }
    return { employeeName: title }
}

/**
 * 代理人未確認時，標題會帶「・未確認」後綴（見 `useLeaveCalendarEvents`）。
 * popover 另有獨立欄位標示確認狀態，這裡把後綴去掉避免同一件事講兩次。
 */
export function stripUnconfirmedSuffix(agentName?: string): string | undefined {
    return agentName?.replace(/・未確認$/, '')
}
