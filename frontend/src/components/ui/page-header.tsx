import type React from 'react'
import { cn } from '@/lib/utils'

interface PageHeaderProps {
  title: string
  description?: string
  actions?: React.ReactNode
  className?: string
  children?: React.ReactNode
}

/**
 * 統一頁面標題區。
 * 左側標題 + 描述，右側操作按鈕。
 */
export function PageHeader({ title, description, actions, className, children }: PageHeaderProps) {
  return (
    <div className={cn('flex flex-col gap-x-4 gap-y-3 sm:flex-row sm:flex-wrap sm:items-center sm:justify-between', className)}>
      <div className="sm:min-w-max">
        <h1 className="page-title">{title}</h1>
        {description && (
          <p className="text-sm text-muted-foreground mt-1">{description}</p>
        )}
      </div>
      {actions && (
        <div className="flex flex-wrap items-center gap-2 shrink-0">
          {actions}
        </div>
      )}
      {children}
    </div>
  )
}
