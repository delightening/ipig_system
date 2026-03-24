import type { LucideIcon } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { TableRow, TableCell } from '@/components/ui/table'
import { cn } from '@/lib/utils'

interface EmptyStateAction {
  label: string
  onClick: () => void
  icon?: LucideIcon
}

interface EmptyStateProps {
  icon: LucideIcon
  title: string
  description?: string
  action?: EmptyStateAction
  className?: string
}

export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  className,
}: EmptyStateProps) {
  return (
    <div className={cn('flex flex-col items-center justify-center py-12 px-4 text-center', className)}>
      <Icon className="h-12 w-12 text-muted-foreground/50 mb-4" strokeWidth={1.5} />
      <h3 className="text-lg font-semibold text-foreground mb-1">{title}</h3>
      {description && (
        <p className="text-sm text-muted-foreground max-w-sm mb-4">{description}</p>
      )}
      {action && (
        <Button onClick={action.onClick} size="sm">
          {action.icon && <action.icon className="h-4 w-4 mr-2" />}
          {action.label}
        </Button>
      )}
    </div>
  )
}

interface TableEmptyRowProps {
  colSpan: number
  icon: LucideIcon
  title: string
  description?: string
  action?: EmptyStateAction
}

export function TableEmptyRow({
  colSpan,
  icon: Icon,
  title,
  description,
  action,
}: TableEmptyRowProps) {
  return (
    <TableRow>
      <TableCell colSpan={colSpan} className="text-center py-8">
        <Icon className="h-12 w-12 mx-auto mb-2 text-muted-foreground/50" strokeWidth={1.5} />
        <p className="text-muted-foreground">{title}</p>
        {description && (
          <p className="text-xs text-muted-foreground/70 mt-1">{description}</p>
        )}
        {action && (
          <Button onClick={action.onClick} variant="outline" size="sm" className="mt-4">
            {action.icon && <action.icon className="h-4 w-4 mr-2" />}
            {action.label}
          </Button>
        )}
      </TableCell>
    </TableRow>
  )
}
