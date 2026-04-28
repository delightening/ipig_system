import * as React from "react"
import { cn } from "@/lib/utils"

interface DateTextInputProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "type" | "onChange"> {
  error?: boolean
  defaultValue?: string
  onChange?: (value: string) => void
}

/**
 * 日期文字輸入元件。
 * 顯示格式：YYYY/MM/DD，自動插入 "/" 分隔符號。
 * ISO 值透過 data-iso 屬性暴露，外部透過 setIsoValue() 設值。
 */
const DateTextInput = React.forwardRef<HTMLInputElement, DateTextInputProps>(
  ({ className, error, defaultValue, onChange, onBlur, ...props }, ref) => {
    const toDisplay = (iso: string) => (iso ? iso.replace(/-/g, "/") : "")
    const toIso = (display: string) => display.replace(/\//g, "-")

    const [display, setDisplay] = React.useState(() =>
      toDisplay(defaultValue ?? "")
    )

    const innerRef = React.useRef<HTMLInputElement>(null)

    // Expose a composite ref: the real DOM element + a setIsoValue helper
    React.useImperativeHandle(ref, () => {
      const el = innerRef.current!
      // Attach a helper method for external callers to set ISO date value
      ;(el as DateInputElement).setIsoValue = (iso: string) => {
        setDisplay(toDisplay(iso))
      }
      return el
    }, [])

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
        data-iso={toIso(display)}
        onChange={handleChange}
        onBlur={handleBlur}
        className={cn(
          "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50",
          error && "border-status-error-solid focus-visible:ring-red-500",
          className
        )}
      />
    )
  }
)
DateTextInput.displayName = "DateTextInput"

/** Extended element type with setIsoValue helper */
interface DateInputElement extends HTMLInputElement {
  setIsoValue?: (iso: string) => void
}

export { DateTextInput }
export type { DateInputElement }
