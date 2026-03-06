/**
 * FullCalendar 視圖元件（獨立模組，僅在日曆已連接時動態載入）
 * 避免在未設定日曆時載入 FullCalendar，防止 cssRules 等錯誤
 */
import FullCalendar from '@fullcalendar/react'
import dayGridPlugin from '@fullcalendar/daygrid'
import timeGridPlugin from '@fullcalendar/timegrid'
import interactionPlugin from '@fullcalendar/interaction'
interface CalendarViewProps {
    /** 要顯示的月份（由父層控制，避免點「下個月」後被舊資料或 re-render 拉回上個月） */
    initialDate: Date
    events: Array<{
        id: string
        title: string
        start: string
        end: string
        allDay: boolean
        extendedProps?: { description?: string; location?: string; htmlLink?: string }
    }>
    onDatesSet: (dateInfo: { start: Date; end: Date }) => void
}

export function CalendarView({ initialDate, events, onDatesSet }: CalendarViewProps) {
    return (
        <div className="calendar-wrapper">
            <FullCalendar
                plugins={[dayGridPlugin, timeGridPlugin, interactionPlugin]}
                initialView="dayGridMonth"
                initialDate={initialDate}
                headerToolbar={{
                    left: 'prev,next today',
                    center: 'title',
                    right: 'dayGridMonth,timeGridWeek,timeGridDay',
                }}
                locale="zh-tw"
                buttonText={{
                    today: '今天',
                    month: '月',
                    week: '週',
                    day: '日',
                }}
                events={events}
                datesSet={onDatesSet}
                eventClick={(info) => {
                    const htmlLink = info.event.extendedProps.htmlLink as string | undefined
                    if (htmlLink) {
                        window.open(htmlLink, '_blank')
                    }
                }}
                height="auto"
                dayMaxEvents={3}
            />
        </div>
    )
}
