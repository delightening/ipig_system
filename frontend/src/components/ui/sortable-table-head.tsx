/**
 * 可排序表頭元件
 * 點擊表頭即可切換排序方向 (asc → desc → 取消)
 *
 * a11y：互動元素為原生 `<button>`（鍵盤 Enter/Space 可啟動、可 Tab 聚焦），
 * 排序狀態透過 `<th aria-sort>` 對輔助技術公開。先前僅在 `<th>` 綁 onClick，
 * 鍵盤使用者無法排序、螢幕閱讀器也讀不出目前排序方向。
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
  /**
   * 僅在明確的斷點（如 `<wbr />`）換行，CJK 字元間不任意斷字。
   * 適用於需控制換行位置的多字標題（例：計畫書<wbr />主持人）。
   */
  breakKeep?: boolean
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
  breakKeep,
  ...props
}: SortableTableHeadProps) {
  const isActive = currentSort === sortKey

  const Icon = isActive
    ? currentDirection === 'asc' ? ArrowUp : ArrowDown
    : ArrowUpDown

  // 未排序時為 'none'，讓螢幕閱讀器知道此欄可排序但目前未套用
  const ariaSort: React.AriaAttributes['aria-sort'] = isActive
    ? currentDirection === 'asc' ? 'ascending' : 'descending'
    : 'none'

  return (
    <TableHead
      aria-sort={ariaSort}
      className={cn('select-none hover:bg-muted/50', className)}
      {...props}
    >
      <button
        type="button"
        onClick={() => onSort(sortKey)}
        className={cn(
          'flex w-full cursor-pointer items-center gap-1',
          'focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
          vertical
            ? 'flex-col gap-0.5'
            : align === 'center' ? 'justify-center text-center' : 'text-left',
        )}
      >
        {vertical ? (
          <span className="[writing-mode:vertical-rl] [text-orientation:upright] tracking-wider leading-tight">{children}</span>
        ) : (
          <span className={cn('line-clamp-2 leading-tight', breakKeep ? 'break-keep' : 'break-words')}>{children}</span>
        )}
        <Icon className={cn('h-3 w-3 shrink-0', isActive ? 'text-foreground' : 'text-muted-foreground/50')} />
      </button>
    </TableHead>
  )
}
