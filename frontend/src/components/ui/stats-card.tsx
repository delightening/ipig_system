import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { cn } from '@/lib/utils'

type AccentColor = 'info' | 'success' | 'warning' | 'error' | 'neutral'

const accentBorder: Record<AccentColor, string> = {
  info: 'border-l-status-info-solid',
  success: 'border-l-status-success-solid',
  warning: 'border-l-status-warning-solid',
  error: 'border-l-status-error-solid',
  neutral: 'border-l-status-neutral-solid',
}

const accentIconBg: Record<AccentColor, string> = {
  info: 'bg-status-info-bg',
  success: 'bg-status-success-bg',
  warning: 'bg-status-warning-bg',
  error: 'bg-status-error-bg',
  neutral: 'bg-status-neutral-bg',
}

const accentIconText: Record<AccentColor, string> = {
  info: 'text-status-info-text',
  success: 'text-status-success-text',
  warning: 'text-status-warning-text',
  error: 'text-status-error-text',
  neutral: 'text-status-neutral-text',
}

interface StatsCardProps {
  icon: React.ComponentType<{ className?: string }>
  label: string
  value: string | number
  description?: string
  iconClassName?: string
  valueClassName?: string
  className?: string
  accentColor?: AccentColor
}

export function StatsCard({
  icon: Icon,
  label,
  value,
  description,
  iconClassName,
  valueClassName,
  className,
  accentColor,
}: StatsCardProps) {
  const hasAccent = !!accentColor

  return (
    <Card className={cn(
      hasAccent && `border-l-2 ${accentBorder[accentColor]}`,
      className,
    )}>
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-sm font-medium">{label}</CardTitle>
        {hasAccent ? (
          <div className={cn('p-2 rounded-lg', accentIconBg[accentColor])}>
            <Icon className={cn('h-4 w-4', iconClassName ?? accentIconText[accentColor])} />
          </div>
        ) : (
          <Icon className={cn('h-4 w-4', iconClassName ?? 'text-muted-foreground')} />
        )}
      </CardHeader>
      <CardContent>
        <div className={cn('text-2xl font-bold', valueClassName)}>{value}</div>
        {description && (
          <p className="text-xs text-muted-foreground">{description}</p>
        )}
      </CardContent>
    </Card>
  )
}
