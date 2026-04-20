/**
 * 可排序表頭元件
 * 點擊表頭即可切換排序方向 (asc → desc → 取消)
 */
import * as React from 'react'
import { ArrowUp, ArrowDown, ArrowUpDown } from 'lucide-react'
import { TableHead } from './table'
import type { SortDirection } from '@/hooks/useTableSort'
import { cn } from '@/lib/utils'

interface SortableTableHeadProps extends React.ThHTMLAttributes<HTMLTableCellElement> {
  /** 此欄位對應的 sort key */
  sortKey: string
  /** 目前排序的欄位 */
  currentSort: string | null
  /** 目前排序方向 */
  currentDirection: SortDirection
  /** 切換排序的 callback */
  onSort: (key: string) => void
  /** 直向標題（中文字正立由上而下），適用窄欄 */
  vertical?: boolean
  /** 內容對齊方式（預設 left） */
  align?: 'left' | 'center'
}

export function SortableTableHead({
  sortKey,
  currentSort,
  currentDirection,
  onSort,
  children,
  className,
  vertical,
  align = 'left',
  ...props
}: SortableTableHeadProps) {
  const isActive = currentSort === sortKey

  const Icon = isActive
    ? currentDirection === 'asc' ? ArrowUp : ArrowDown
    : ArrowUpDown

  return (
    <TableHead
      className={cn('cursor-pointer select-none hover:bg-muted/50', className)}
      onClick={() => onSort(sortKey)}
      {...props}
    >
      {vertical ? (
        <div className="flex flex-col items-center gap-0.5">
          <span className="[writing-mode:vertical-rl] [text-orientation:upright] tracking-wider leading-tight">{children}</span>
          <Icon className={cn('h-3 w-3 flex-shrink-0', isActive ? 'text-foreground' : 'text-muted-foreground/50')} />
        </div>
      ) : (
        <div className={cn('flex items-center gap-1', align === 'center' && 'justify-center')}>
          <span className="line-clamp-2 break-words leading-tight">{children}</span>
          <Icon className={cn('h-3 w-3 flex-shrink-0', isActive ? 'text-foreground' : 'text-muted-foreground/50')} />
        </div>
      )}
    </TableHead>
  )
}
