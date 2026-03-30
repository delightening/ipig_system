import * as React from "react"
import { format, parse, isValid, addMonths, subMonths, startOfMonth, endOfMonth, startOfWeek, endOfWeek, eachDayOfInterval, isSameMonth, isSameDay, isToday } from "date-fns"
import { zhTW, enUS } from "date-fns/locale"
import { useTranslation } from "react-i18next"
import { ChevronLeft, ChevronRight, Calendar } from "lucide-react"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import { Popover, PopoverContent, PopoverTrigger } from "@radix-ui/react-popover"
import { Input } from "@/components/ui/input"

interface DatePickerProps {
    value: string // ISO date string (YYYY-MM-DD)
    onChange: (value: string) => void
    placeholder?: string
    required?: boolean
    className?: string
    disabled?: boolean
}

export function DatePicker({
    value,
    onChange,
    placeholder,
    required,
    className,
    disabled,
}: DatePickerProps) {
    const { t, i18n } = useTranslation()
    const [open, setOpen] = React.useState(false)
    const [inputValue, setInputValue] = React.useState("")
    const [viewDate, setViewDate] = React.useState(new Date())

    // Get the correct locale based on i18n language
    const locale = i18n.language === 'zh-TW' ? zhTW : enUS

    // Parse the value to a Date object
    const selectedDate = React.useMemo(() => {
        if (!value) return null
        const parsed = parse(value, 'yyyy-MM-dd', new Date())
        return isValid(parsed) ? parsed : null
    }, [value])

    // Update internal state when external value changes
    React.useEffect(() => {
        if (value) {
            const parsed = parse(value, 'yyyy-MM-dd', new Date())
            if (isValid(parsed)) {
                setInputValue(format(parsed, 'yyyy-MM-dd'))
                setViewDate(parsed)
            }
        } else {
            setInputValue("")
        }
    }, [value])

    // Handle manual input
    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const val = e.target.value
        setInputValue(val)

        // Try various formats
        const formats = ['yyyy-MM-dd', 'yyyy/MM/dd', 'yyyyMMdd', 'yyyy.MM.dd']
        let parsedDate: Date | null = null

        for (const fmt of formats) {
            const parsed = parse(val, fmt, new Date())
            if (isValid(parsed) && val.length >= 8) {
                // Ensure the year is reasonable (e.g. 4 digits)
                const year = parsed.getFullYear()
                if (year > 1000 && year < 3000) {
                    parsedDate = parsed
                    break
                }
            }
        }

        if (parsedDate) {
            const isoString = format(parsedDate, 'yyyy-MM-dd')
            if (isoString !== value) {
                onChange(isoString)
                setViewDate(parsedDate)
            }
        } else if (val === "") {
            onChange("")
        }
    }

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
        const isoString = format(date, 'yyyy-MM-dd')
        onChange(isoString)
        setInputValue(isoString)
        setOpen(false)
    }

    const handleClear = () => {
        onChange('')
        setInputValue('')
        setOpen(false)
    }

    const handleToday = () => {
        const today = new Date()
        const isoString = format(today, 'yyyy-MM-dd')
        onChange(isoString)
        setInputValue(isoString)
        setViewDate(today)
        setOpen(false)
    }

    const goToPreviousMonth = () => {
        setViewDate(prev => subMonths(prev, 1))
    }

    const goToNextMonth = () => {
        setViewDate(prev => addMonths(prev, 1))
    }

    return (
        <Popover open={open} onOpenChange={setOpen}>
            <div className={cn("relative flex items-center", className)}>
                <Input
                    type="text"
                    value={inputValue}
                    onChange={handleInputChange}
                    placeholder={placeholder || t('datePicker.placeholder', 'Select date')}
                    required={required}
                    disabled={disabled}
                    className="pr-10" // Space for the calendar icon
                    onFocus={() => setOpen(true)}
                />
                <PopoverTrigger asChild>
                    <button
                        type="button"
                        disabled={disabled}
                        className="absolute right-3 text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
                    >
                        <Calendar className="h-4 w-4" />
                    </button>
                </PopoverTrigger>
            </div>
            <PopoverContent
                className="w-auto p-0 bg-white rounded-md border shadow-lg z-[100]"
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
                        {weekDays.map((day) => (
                            <div
                                key={day}
                                className="h-8 w-8 flex items-center justify-center text-xs text-muted-foreground font-medium"
                            >
                                {day}
                            </div>
                        ))}
                    </div>

                    {/* Calendar grid */}
                    <div className="grid grid-cols-7 gap-1">
                        {calendarDays.map((date) => {
                            const isCurrentMonth = isSameMonth(date, viewDate)
                            const isSelected = selectedDate && isSameDay(date, selectedDate)
                            const isTodayDate = isToday(date)

                            return (
                                <button
                                    key={date.toISOString()}
                                    type="button"
                                    onClick={() => handleDateSelect(date)}
                                    className={cn(
                                        "h-8 w-8 flex items-center justify-center text-sm rounded-md transition-colors",
                                        !isCurrentMonth && "text-muted-foreground opacity-50",
                                        isCurrentMonth && !isSelected && "hover:bg-muted",
                                        isSelected && "bg-primary text-white hover:bg-primary/90",
                                        isTodayDate && !isSelected && "text-status-info-text font-bold"
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
                            className="text-xs text-status-info-text"
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
