import * as React from "react"
import { cn } from "@/lib/utils"

interface DateTextInputProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "type" | "onChange"> {
  error?: boolean
  defaultValue?: string
  onChange?: (value: string) => void
}

/**
 * 日期文字輸入元件，使用 text input 取代原生 date input，
 * 避免瀏覽器自動在年/月/日段落之間跳轉。
 * 顯示格式：YYYY/MM/DD，自動插入 "/" 分隔符號。
 * 對外 .value 格式為 ISO：YYYY-MM-DD。
 */
const DateTextInput = React.forwardRef<HTMLInputElement, DateTextInputProps>(
  ({ className, error, defaultValue, onChange, onBlur, ...props }, ref) => {
    const toDisplay = (iso: string) => (iso ? iso.replace(/-/g, "/") : "")
    const toIso = (display: string) => display.replace(/\//g, "-")

    const [display, setDisplay] = React.useState(() =>
      toDisplay(defaultValue ?? "")
    )

    const innerRef = React.useRef<HTMLInputElement>(null)
    const isoRef = React.useRef(toIso(display))

    // Keep isoRef in sync
    isoRef.current = toIso(display)

    // Expose the inner input element via ref, with .value returning ISO format
    React.useEffect(() => {
      const el = innerRef.current
      if (!el) return

      Object.defineProperty(el, "value", {
        get: () => isoRef.current,
        set: (v: string) => setDisplay(toDisplay(v)),
        configurable: true,
      })
    }, [])

    // Patch ref forwarding
    React.useImperativeHandle(ref, () => innerRef.current!)

    // Sync when defaultValue changes externally
    const prevDefault = React.useRef(defaultValue)
    React.useEffect(() => {
      if (defaultValue !== prevDefault.current) {
        prevDefault.current = defaultValue
        setDisplay(toDisplay(defaultValue ?? ""))
      }
    }, [defaultValue])

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      const raw = e.target.value.replace(/[^\d]/g, "")
      let formatted = ""

      for (let i = 0; i < raw.length && i < 8; i++) {
        if (i === 4 || i === 6) formatted += "/"
        formatted += raw[i]
      }

      setDisplay(formatted)
      if (raw.length === 8) {
        onChange?.(toIso(formatted))
      }
    }

    const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
      onBlur?.(e)
    }

    return (
      <input
        ref={innerRef}
        type="text"
        inputMode="numeric"
        maxLength={10}
        placeholder="YYYY/MM/DD"
        {...props}
        value={display}
        onChange={handleChange}
        onBlur={handleBlur}
        className={cn(
          "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50",
          error && "border-red-500 focus-visible:ring-red-500",
          className
        )}
      />
    )
  }
)
DateTextInput.displayName = "DateTextInput"

export { DateTextInput }
