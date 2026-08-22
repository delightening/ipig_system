import { cn } from '@/lib/utils'

type StatusVariant =
  | 'success'
  | 'warning'
  | 'error'
  | 'info'
  | 'neutral'
  | 'purple'

/** 視覺強度階（DESIGN.md §3.4）：solid=醒目單顆/警示、soft=表格成群密集標籤。 */
type StatusTone = 'solid' | 'soft'

// solid（實心 + 白字）：單顆醒目狀態、計數、需抓眼球的警示。
const solidStyles: Record<StatusVariant, string> = {
  success: 'bg-status-success-solid text-white border-transparent',
  warning: 'bg-status-warning-solid text-white border-transparent',
  error: 'bg-status-error-solid text-white border-transparent',
  info: 'bg-status-info-solid text-white border-transparent',
  neutral: 'bg-status-neutral-solid text-white border-transparent',
  purple: 'bg-status-purple-solid text-white border-transparent',
}

// soft（淺底 + 深字 + 淺框）：表格每列、chip 群等密集重複標籤，降低視覺噪音。
const softStyles: Record<StatusVariant, string> = {
  success: 'bg-status-success-bg text-status-success-text border-status-success-border',
  warning: 'bg-status-warning-bg text-status-warning-text border-status-warning-border',
  error: 'bg-status-error-bg text-status-error-text border-status-error-border',
  info: 'bg-status-info-bg text-status-info-text border-status-info-border',
  neutral: 'bg-status-neutral-bg text-status-neutral-text border-status-neutral-border',
  purple: 'bg-status-purple-bg text-status-purple-text border-status-purple-border',
}

interface StatusBadgeProps {
  variant: StatusVariant
  /** 視覺強度階，預設 'solid'。密集/成群標籤請用 'soft'（見 DESIGN.md §3.4）。 */
  tone?: StatusTone
  children: React.ReactNode
  className?: string
  dot?: boolean
}

/**
 * 語義化狀態標籤。
 * 使用 CSS Variable tokens，不含任何硬編碼色彩。
 * 兩階（tone）：'solid'（醒目）/ 'soft'（密集）。選階規則見 DESIGN.md §3.4。
 */
export function StatusBadge({ variant, tone = 'solid', children, className, dot }: StatusBadgeProps) {
  const styles = tone === 'soft' ? softStyles[variant] : solidStyles[variant]
  // solid 白字 → dot 用半透明白；soft 深字 → dot 跟隨文字色。
  const dotCls = tone === 'soft' ? 'bg-current opacity-70' : 'bg-white/85'
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-medium',
        styles,
        className
      )}
    >
      {dot && (
        <span className={cn('h-1.5 w-1.5 rounded-full', dotCls)} aria-hidden="true" />
      )}
      {children}
    </span>
  )
}

export type { StatusVariant, StatusTone }
