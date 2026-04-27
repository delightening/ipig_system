# 日曆頁面審視與改進規劃

本文件審視 HR 日曆頁（Google Calendar 同步設定／事件檢視）的現況、業界常見做法，並提供未來改進規劃供實作參考。

---

## 1. 現況摘要

### 1.1 架構

| 層級 | 檔案／職責 |
|------|------------|
| 頁面 | `CalendarSyncSettingsPage.tsx`：Tabs（日曆／同步歷史／衝突）、連線狀態、事件 API、日曆顯示條件 |
| 視圖 | `CalendarView.tsx`：FullCalendar 包裝，`initialDate` + `events` + `onDatesSet` |
| 資料 | React Query `['calendar-events', calendarDateRange]`，依目前可見範圍 fetch |

### 1.2 狀態與資料流

- **calendarDateRange**（`{ start, end }`）：由 `datesSet` 回報的可見範圍，同時作為：
  - 事件 API 的 query key（決定 fetch 哪個區間）
  - 日曆「顯示哪一月」的依據：`key={format(calendarDateRange.start, 'yyyy-MM')}` + `initialDate={calendarDateRange.start}`，月份一變就重掛 `CalendarView`，用 `initialDate` 鎖定顯示月。
- **handleDatesSet**：收到 FullCalendar 的 `datesSet` 時更新 `calendarDateRange`；目前僅在「refetch 中」且新範圍較舊時忽略，避免被拉回上個月。
- **fullCalendarEvents**：`loadingEvents` 時傳空陣列，避免把 React Query 的 stale（上一月）事件傳給 FullCalendar 觸發錯誤的 `datesSet`。
- **calendarMounted**：延後兩幀掛載 FullCalendar，減輕初次載入的 reflow／Violation。

### 1.3 已知問題與對策（已實作／曾討論）

| 現象 | 對策 |
|------|------|
| 載入日曆時 Console Violation（message/reflow/setTimeout） | 延後掛載、refetch 時不卸載日曆、`fullCalendarEvents` 用 useMemo |
| 點「下個月」後顯示仍本月 | 僅初次載入顯示全螢幕 loading；refetch 時保持日曆掛載 |
| 點「下個月」一次進、一次退（3 月↔4 月交替） | refetch 時傳空事件；handleDatesSet 在 loading 時忽略「較舊」範圍 |
| loading 完成後畫面跳回 3 月（例如從 5 月） | 需在 handleDatesSet 中：僅接受「往後」或「恰好上一個月」的 datesSet，其餘往前的誤觸發忽略（見下方建議） |

---

## 2. 業界常見做法

### 2.1 單一真相來源（Single Source of Truth）

- **概念**：目前可見的「日期／範圍」由父層 state 決定，日曆只是「顯示」該 state；使用者操作透過 callback 回報，由父層更新 state 再驅動 API 與 UI。
- **實作要點**：
  - 父層持有 `currentDate` 或 `visibleRange`。
  - 日曆為受控：`value`/`initialDate`/`visibleRange` 由 props 傳入，變更透過 `onChange`/`onDatesSet` 回報。
  - 與 API 的同步只依這份 state（例如 query key = f(visibleRange)），不依賴日曆內部未暴露的狀態。

我們目前已接近此模式：`calendarDateRange` 為真相來源，並用 `key` + `initialDate` 強制顯示該月；需補強的是「哪些 datesSet 要接受」的規則，避免 FullCalendar 誤觸發把 state 改回舊月。

### 2.2 FullCalendar 官方／社群建議

- **initialDate / initialView**：僅在「首次掛載」有效，之後改 prop **不會**更新畫面；要以程式改變顯示日期需用 **ref + API**。
- **ref + getApi()**：取得 calendar 實例後，用 `calendarRef.current.getApi().gotoDate(date)`、`changeView(viewType)` 做程式化切換，適合「外部篩選／按鈕切到指定月」等情境。
- **datesSet**：用於「可見範圍變更時」拉取資料（fetch on range change），與我們用法一致；需注意 FullCalendar 在收到新 `events` 後有時會再觸發一次 `datesSet`，且可能回傳非預期的舊範圍，因此**由父層過濾不可信的 datesSet** 是合理做法。
- **事件來源**：若事件來自 API，常見做法是「依目前可見範圍 fetch，再將結果傳給 FullCalendar」；loading 時可傳空陣列或保留上一筆並標示 stale，避免用「舊範圍事件」驅動視圖。

與我們差異在於：我們用 **key + 重掛** 來對齊「顯示月」與 state，而非用 ref 呼叫 `gotoDate`；兩者皆可，重掛的優點是邏輯簡單、不依賴 FullCalendar 實例生命週期，缺點是每月切換會有一次 remount。

### 2.3 與 API 同步的常見模式

- **Range-based fetch**：以 `datesSet` 回報的 `start`/`end` 作為 request 參數，query key 綁定該範圍；我們已採用。
- **Stale-while-revalidate**：切月時可先顯示上一筆 cache（或空），再在背景 refetch 新範圍；我們目前是 loading 時傳空、完成後傳新資料，必要時可改為保留上一月並加 loading 指示。
- **防抖／過濾**：對 `datesSet` 做 debounce 或「僅接受與當前 state 相容的更新」（例如只接受往後或僅退一個月），以過濾庫的誤觸發；我們已在討論中納入「僅接受往後或恰好上一個月」的邏輯。

---

## 3. 改進建議與優先順序

### 3.1 高優先：穩定「顯示月」邏輯（建議立即補齊）

**問題**：loading 完成後 FullCalendar 有時會再觸發 `datesSet(更早的月)`，若無過濾會把 `calendarDateRange` 改回舊月，畫面跳回 3 月。

**建議**：在 `handleDatesSet` 內明確規定「只接受合理的使用者導覽」：

- 新範圍 **等於** 目前範圍 → 不更新。
- 新範圍 **晚於** 目前範圍 → 接受（使用者點「下個月」／切到未來）。
- 新範圍 **早於** 目前範圍 → **僅在「恰好早一個月」時接受**（使用者點「上個月」），其餘一律忽略。

實作要點：用 `date-fns` 的 `startOfMonth`、`addMonths` 比較「新範圍的月初」是否等於「目前月初的前一個月」；若是才更新，否則 `return prev`。這樣可避免誤觸發把 5 月改回 3 月，同時保留「上個月」按鈕正常。

### 3.2 中優先：結構與可讀性

- **自訂 hook**：例如 `useCalendarEvents(visibleRange, enabled)`，內部包 React Query + 轉成 FullCalendar 的 events 格式；頁面只關心「目前範圍」與「顯示條件」，減少頁面元件體積。
- **datesSet 過濾邏輯集中**：將「是否接受此次 datesSet」抽成純函式（例如 `shouldAcceptDatesSet(newRange, prevRange, { loading })`），方便單測與註解。
- **子元件拆分**：將「連線狀態區塊」「同步歷史表格」「衝突列表」拆成獨立元件或子頁面，日曆區塊單獨負責「可見範圍 state + CalendarView + events」。

### 3.3 低優先：進階優化

- **Ref 方案**：若希望不重掛日曆就切月，可改為單一 `CalendarView` 實例 + ref，在 `calendarDateRange` 變更時 `getApi().gotoDate(calendarDateRange.start)`；需處理「首次掛載」與「ref 未就緒」的邊界。
- **Violation 再優化**：若仍有多餘 reflow，可評估 FullCalendar 的 `contentHeight`/`aspectRatio` 或改為固定高度、或與官方 issue 追蹤是否有新解法。
- **E2E**：為「下個月」連續點擊、loading 完成後仍停留在正確月份撰寫 E2E，防止回歸。

---

## 4. 建議的 handleDatesSet 邏輯（供貼上／對照）

以下為「僅接受往後或恰好上一個月」的實作範例，可取代目前僅依 `loadingEvents` 忽略往前的版本：

```ts
import { addMonths, startOfMonth } from 'date-fns'

const handleDatesSet = useCallback((dateInfo: { start: Date; end: Date }) => {
    setCalendarDateRange((prev) => {
        const newStart = dateInfo.start.getTime()
        const prevStart = prev.start.getTime()
        if (newStart === prevStart) return prev
        if (newStart > prevStart) return { start: dateInfo.start, end: dateInfo.end }
        // 新範圍比當前早：僅在「恰好早一個月」時接受（使用者按「上個月」）
        const prevMonthStart = startOfMonth(addMonths(prev.start, -1)).getTime()
        const newMonthStart = startOfMonth(dateInfo.start).getTime()
        if (newMonthStart === prevMonthStart) return { start: dateInfo.start, end: dateInfo.end }
        return prev
    })
}, [])
```

依此可避免「loading 完成後誤觸發 datesSet(3月) 導致畫面跳回 3 月」的問題。

---

## 5. 檔案與參考

- 頁面：`frontend/src/pages/hr/CalendarSyncSettingsPage.tsx`
- 視圖：`frontend/src/pages/hr/CalendarView.tsx`
- FullCalendar React：<https://fullcalendar.io/docs/react>
- initialDate / 程式化控制：<https://fullcalendar.io/docs/initialDate>；以 ref + `getApi().gotoDate()` 更新顯示為官方建議做法之一。

---

*文件建立供未來重構或新人接手時對齊設計與業界做法。實作時請依當時程式碼狀態調整。*
