import * as React from "react"
import { ChevronsDownUp, ChevronsUpDown } from "lucide-react"

import { cn } from "@/lib/utils"
import { Textarea, type TextareaProps } from "@/components/ui/input"

export interface AutoGrowTextareaProps extends TextareaProps {
  /** 收合時的固定高度（px），預設 80 */
  collapsedHeight?: number
  /** 自動長高的上限（px），0 = 不限制；超過時內部捲動 */
  maxAutoHeight?: number
}

/**
 * 計畫書表單專用：內容多時自動長高，右上角按鈕可將該欄收回固定高度（內部捲動）。
 * 復用 base Textarea，不更動其 DOM 契約；收合按鈕需要的定位容器封裝在此 wrapper。
 */
const AutoGrowTextarea = React.forwardRef<HTMLTextAreaElement, AutoGrowTextareaProps>(
  (
    { className, collapsedHeight = 80, maxAutoHeight = 0, value, onChange, ...props },
    forwardedRef,
  ) => {
    const innerRef = React.useRef<HTMLTextAreaElement>(null)
    const [collapsed, setCollapsed] = React.useState(false)

    // 合併 forwarded ref 與內部測量用 ref
    React.useImperativeHandle(forwardedRef, () => innerRef.current as HTMLTextAreaElement, [])

    const resize = React.useCallback(() => {
      const el = innerRef.current
      if (!el) return
      if (collapsed) {
        el.style.height = `${collapsedHeight}px`
        el.style.overflowY = "auto"
        return
      }
      el.style.height = "auto"
      let next = el.scrollHeight
      if (maxAutoHeight > 0 && next > maxAutoHeight) {
        next = maxAutoHeight
        el.style.overflowY = "auto"
      } else {
        el.style.overflowY = "hidden"
      }
      el.style.height = `${next}px`
    }, [collapsed, collapsedHeight, maxAutoHeight])

    // value 由外部變動、或收合狀態切換時重新計算高度
    React.useLayoutEffect(() => {
      resize()
    }, [resize, value])

    return (
      <div className="relative">
        <Textarea
          ref={innerRef}
          value={value}
          onChange={(e) => {
            onChange?.(e)
            resize()
          }}
          className={cn("resize-none pr-9", className)}
          {...props}
        />
        {!props.disabled && (
          <button
            type="button"
            tabIndex={-1}
            onClick={() => setCollapsed((c) => !c)}
            aria-label={collapsed ? "展開欄位" : "收合欄位"}
            className="absolute top-1.5 right-1.5 rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
          >
            {collapsed ? <ChevronsUpDown className="h-4 w-4" /> : <ChevronsDownUp className="h-4 w-4" />}
          </button>
        )}
      </div>
    )
  },
)
AutoGrowTextarea.displayName = "AutoGrowTextarea"

export { AutoGrowTextarea }
