import * as React from "react"
import { format, parse, isValid, addMonths, subMonths, startOfMonth, endOfMonth, startOfWeek, endOfWeek, eachDayOfInterval, isSameMonth, isSameDay, isToday } from "date-fns"
import { zhTW, enUS } from "date-fns/locale"
import { useTranslation } from "react-i18next"
import { ChevronLeft, ChevronRight, Calendar } from "lucide-react"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import { Popover, PopoverContent, PopoverTrigger } from "@radix-ui/react-popover"

interface DatePickerProps {
    value: string // ISO date string (YYYY-MM-DD)
    onChange: (value: string) => void
    placeholder?: string
    required?: boolean
    className?: string
}

export function DatePicker({
    value,
    onChange,
    placeholder,
    required,
    className,
}: DatePickerProps) {
    const { t, i18n } = useTranslation()
    const [open, setOpen] = React.useState(false)
    const [viewDate, setViewDate] = React.useState(() => {
        if (value) {
            const parsed = parse(value, 'yyyy-MM-dd', new Date())
            return isValid(parsed) ? parsed : new Date()
        }
        return new Date()
    })

    // Get the correct locale based on i18n language
    const locale = i18n.language === 'zh-TW' ? zhTW : enUS

    // Parse the value to a Date object
    const selectedDate = React.useMemo(() => {
        if (!value) return null
        const parsed = parse(value, 'yyyy-MM-dd', new Date())
        return isValid(parsed) ? parsed : null
    }, [value])

    // Update viewDate when selectedDate changes
    React.useEffect(() => {
        if (selectedDate) {
            setViewDate(selectedDate)
        }
    }, [selectedDate])

    // Get calendar days for the current month view
    const calendarDays = React.useMemo(() => {
        const monthStart = startOfMonth(viewDate)
        const monthEnd = endOfMonth(viewDate)
        const calendarStart = startOfWeek(monthStart, { locale })
        const calendarEnd = endOfWeek(monthEnd, { locale })
        return eachDayOfInterval({ start: calendarStart, end: calendarEnd })
    }, [viewDate, locale])

    // Get weekday headers
    const weekDays = React.useMemo(() => {
        const start = startOfWeek(new Date(), { locale })
        return Array.from({ length: 7 }, (_, i) => {
            const date = new Date(start)
            date.setDate(date.getDate() + i)
            return format(date, 'EEEEEE', { locale }) // Returns short 2-letter day names
        })
    }, [locale])

    const handleDateSelect = (date: Date) => {
        onChange(format(date, 'yyyy-MM-dd'))
        setOpen(false)
    }

    const handleClear = () => {
        onChange('')
        setOpen(false)
    }

    const handleToday = () => {
        const today = new Date()
        onChange(format(today, 'yyyy-MM-dd'))
        setViewDate(today)
        setOpen(false)
    }

    const goToPreviousMonth = () => {
        setViewDate(prev => subMonths(prev, 1))
    }

    const goToNextMonth = () => {
        setViewDate(prev => addMonths(prev, 1))
    }

    // Format display value
    const displayValue = selectedDate
        ? format(selectedDate, 'yyyy/MM/dd', { locale })
        : ''

    return (
        <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger asChild>
                <button
                    type="button"
                    className={cn(
                        "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background",
                        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                        "disabled:cursor-not-allowed disabled:opacity-50",
                        "items-center justify-between text-left",
                        !displayValue && "text-muted-foreground",
                        className
                    )}
                >
                    <span>{displayValue || placeholder || t('datePicker.placeholder', 'Select date')}</span>
                    <Calendar className="h-4 w-4 opacity-50" />
                </button>
            </PopoverTrigger>
            <PopoverContent
                className="w-auto p-0 bg-white rounded-md border shadow-lg z-50"
                align="start"
                sideOffset={4}
            >
                <div className="p-3">
                    {/* Header with month navigation */}
                    <div className="flex items-center justify-between mb-3">
                        <Button
                            variant="ghost"
                            size="icon"
                            className="h-7 w-7"
                            onClick={goToPreviousMonth}
                        >
                            <ChevronLeft className="h-4 w-4" />
                        </Button>
                        <span className="font-medium text-sm">
                            {format(viewDate, i18n.language === 'zh-TW' ? 'yyyy年M月' : 'MMMM yyyy', { locale })}
                        </span>
                        <Button
                            variant="ghost"
                            size="icon"
                            className="h-7 w-7"
                            onClick={goToNextMonth}
                        >
                            <ChevronRight className="h-4 w-4" />
                        </Button>
                    </div>

                    {/* Weekday headers */}
                    <div className="grid grid-cols-7 gap-1 mb-1">
                        {weekDays.map((day, idx) => (
                            <div
                                key={idx}
                                className="h-8 w-8 flex items-center justify-center text-xs text-muted-foreground font-medium"
                            >
                                {day}
                            </div>
                        ))}
                    </div>

                    {/* Calendar grid */}
                    <div className="grid grid-cols-7 gap-1">
                        {calendarDays.map((date, idx) => {
                            const isCurrentMonth = isSameMonth(date, viewDate)
                            const isSelected = selectedDate && isSameDay(date, selectedDate)
                            const isTodayDate = isToday(date)

                            return (
                                <button
                                    key={idx}
                                    type="button"
                                    onClick={() => handleDateSelect(date)}
                                    className={cn(
                                        "h-8 w-8 flex items-center justify-center text-sm rounded-md transition-colors",
                                        !isCurrentMonth && "text-muted-foreground opacity-50",
                                        isCurrentMonth && !isSelected && "hover:bg-slate-100",
                                        isSelected && "bg-blue-600 text-white hover:bg-blue-700",
                                        isTodayDate && !isSelected && "text-blue-600 font-bold"
                                    )}
                                >
                                    {format(date, 'd')}
                                </button>
                            )
                        })}
                    </div>

                    {/* Footer with Clear and Today buttons */}
                    <div className="flex justify-between mt-3 pt-3 border-t">
                        <Button
                            variant="ghost"
                            size="sm"
                            className="text-xs text-muted-foreground"
                            onClick={handleClear}
                        >
                            {t('datePicker.clear', 'Clear')}
                        </Button>
                        <Button
                            variant="ghost"
                            size="sm"
                            className="text-xs text-blue-600"
                            onClick={handleToday}
                        >
                            {t('datePicker.today', 'Today')}
                        </Button>
                    </div>
                </div>
            </PopoverContent>
        </Popover>
    )
}
