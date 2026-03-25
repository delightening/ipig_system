import { cn } from '@/lib/utils'

type StatusVariant =
  | 'success'
  | 'warning'
  | 'error'
  | 'info'
  | 'neutral'
  | 'purple'

const variantStyles: Record<StatusVariant, string> = {
  success: 'bg-status-success-bg text-status-success-text border-status-success-border',
  warning: 'bg-status-warning-bg text-status-warning-text border-status-warning-border',
  error: 'bg-status-error-bg text-status-error-text border-status-error-border',
  info: 'bg-status-info-bg text-status-info-text border-status-info-border',
  neutral: 'bg-status-neutral-bg text-status-neutral-text border-status-neutral-border',
  purple: 'bg-status-purple-bg text-status-purple-text border-status-purple-border',
}

interface StatusBadgeProps {
  variant: StatusVariant
  children: React.ReactNode
  className?: string
  dot?: boolean
}

/**
 * 語義化狀態標籤。
 * 使用 CSS Variable tokens，不含任何硬編碼色彩。
 */
export function StatusBadge({ variant, children, className, dot }: StatusBadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-medium',
        variantStyles[variant],
        className
      )}
    >
      {dot && (
        <span
          className={cn(
            'h-1.5 w-1.5 rounded-full',
            variant === 'success' && 'bg-status-success-text',
            variant === 'warning' && 'bg-status-warning-text',
            variant === 'error' && 'bg-status-error-text',
            variant === 'info' && 'bg-status-info-text',
            variant === 'neutral' && 'bg-status-neutral-text',
            variant === 'purple' && 'bg-status-purple-text',
          )}
        />
      )}
      {children}
    </span>
  )
}

export type { StatusVariant }
