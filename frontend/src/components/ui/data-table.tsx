import type React from 'react'
import type { LucideIcon } from 'lucide-react'
import { ChevronLeft, ChevronRight } from 'lucide-react'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Button } from '@/components/ui/button'
import { EmptyState } from '@/components/ui/empty-state'
import { cn } from '@/lib/utils'

// ─── Column 定義 ───

export interface ColumnDef<T> {
  key: string
  header: React.ReactNode
  cell: (row: T, index: number) => React.ReactNode
  className?: string
  /**
   * Tailwind @container-based hide rules applied to both <TableHead> and <TableCell>.
   * Pass literal class strings so Tailwind JIT can scan them, e.g. `'hidden @[750px]:table-cell'`.
   */
  hideClassName?: string
}

// ─── Card breakpoint lookup (literal classes for Tailwind JIT) ───

export type CardBreakpoint = 500 | 600 | 700 | 800

const CARD_BREAKPOINT_CLASSES: Record<CardBreakpoint, { table: string; cards: string }> = {
  500: { table: 'hidden @[500px]:block', cards: '@[500px]:hidden' },
  600: { table: 'hidden @[600px]:block', cards: '@[600px]:hidden' },
  700: { table: 'hidden @[700px]:block', cards: '@[700px]:hidden' },
  800: { table: 'hidden @[800px]:block', cards: '@[800px]:hidden' },
}

// ─── Pagination ───

interface PaginationProps {
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  totalItems?: number
}

function DataTablePagination({ page, totalPages, onPageChange, totalItems }: PaginationProps) {
  if (totalPages <= 1) return null
  return (
    <div className="flex items-center justify-between px-2 py-3">
      <p className="text-sm text-muted-foreground">
        {totalItems !== undefined ? `共 ${totalItems} 筆` : `第 ${page} / ${totalPages} 頁`}
      </p>
      <div className="flex items-center gap-1">
        <Button
          variant="outline"
          size="sm"
          disabled={page <= 1}
          onClick={() => onPageChange(page - 1)}
        >
          <ChevronLeft className="h-4 w-4" />
        </Button>
        <span className="text-sm px-2">
          {page} / {totalPages}
        </span>
        <Button
          variant="outline"
          size="sm"
          disabled={page >= totalPages}
          onClick={() => onPageChange(page + 1)}
        >
          <ChevronRight className="h-4 w-4" />
        </Button>
      </div>
    </div>
  )
}

// ─── DataTable ───

interface DataTableProps<T> {
  columns: ColumnDef<T>[]
  data: T[] | undefined
  isLoading?: boolean
  /** Loading skeleton 行數 */
  skeletonRows?: number
  /** Empty state */
  emptyIcon?: LucideIcon
  emptyTitle?: string
  emptyDescription?: string
  /** Pagination */
  page?: number
  totalPages?: number
  totalItems?: number
  onPageChange?: (page: number) => void
  /** Row key extractor */
  rowKey: (row: T, index: number) => string | number
  /** 額外的 row className */
  rowClassName?: (row: T, index: number) => string
  /** 點擊行 */
  onRowClick?: (row: T) => void
  className?: string
  /**
   * Optional mobile card renderer. When provided, the container switches to a
   * stacked card layout below `cardBreakpoint`.
   */
  mobileCard?: (row: T, index: number) => React.ReactNode
  /** Container width (px) below which `mobileCard` is used. Default 600. */
  cardBreakpoint?: CardBreakpoint
}

/**
 * 統一資料表格。
 * 整合 Loading Skeleton、Empty State、Pagination。
 * 使用 @container queries：在欄 `hideClassName` 指定隱藏時機，或傳入 `mobileCard` 在窄螢幕改為卡片版。
 */
export function DataTable<T>({
  columns,
  data,
  isLoading,
  skeletonRows = 5,
  emptyIcon,
  emptyTitle = '尚無資料',
  emptyDescription,
  page,
  totalPages,
  totalItems,
  onPageChange,
  rowKey,
  rowClassName,
  onRowClick,
  className,
  mobileCard,
  cardBreakpoint = 600,
}: DataTableProps<T>) {
  const colSpan = columns.length
  const bp = CARD_BREAKPOINT_CLASSES[cardBreakpoint]
  const hasData = !!data && data.length > 0

  const tableNode = (
    <Table>
      <TableHeader>
        <TableRow className="bg-muted/50 hover:bg-muted/50">
          {columns.map((col) => (
            <TableHead key={col.key} className={cn(col.className, col.hideClassName)}>
              {col.header}
            </TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody>
        {isLoading ? (
          <TableRow>
            <TableCell colSpan={colSpan} className="p-0">
              <TableSkeleton rows={skeletonRows} cols={colSpan} />
            </TableCell>
          </TableRow>
        ) : hasData ? (
          data!.map((row, index) => (
            <TableRow
              key={rowKey(row, index)}
              className={cn(
                onRowClick && 'cursor-pointer',
                rowClassName?.(row, index)
              )}
              onClick={() => onRowClick?.(row)}
            >
              {columns.map((col) => (
                <TableCell key={col.key} className={cn(col.className, col.hideClassName)}>
                  {col.cell(row, index)}
                </TableCell>
              ))}
            </TableRow>
          ))
        ) : emptyIcon ? (
          <TableEmptyRow
            colSpan={colSpan}
            icon={emptyIcon}
            title={emptyTitle}
            description={emptyDescription}
          />
        ) : (
          <TableRow>
            <TableCell colSpan={colSpan} className="text-center py-8 text-muted-foreground">
              {emptyTitle}
            </TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  )

  return (
    <div className={cn('rounded-lg border bg-card overflow-hidden @container', className)}>
      {mobileCard ? (
        <>
          <div className={bp.table}>{tableNode}</div>
          <div className={cn(bp.cards, 'divide-y')}>
            {isLoading ? (
              <div className="p-3">
                <TableSkeleton rows={Math.min(skeletonRows, 3)} cols={1} />
              </div>
            ) : hasData ? (
              data!.map((row, index) => (
                <div
                  key={rowKey(row, index)}
                  className={cn(
                    'p-3',
                    onRowClick && 'cursor-pointer hover:bg-muted/50',
                    rowClassName?.(row, index)
                  )}
                  onClick={() => onRowClick?.(row)}
                >
                  {mobileCard(row, index)}
                </div>
              ))
            ) : emptyIcon ? (
              <EmptyState
                icon={emptyIcon}
                title={emptyTitle}
                description={emptyDescription}
              />
            ) : (
              <p className="text-center py-8 text-muted-foreground text-sm">{emptyTitle}</p>
            )}
          </div>
        </>
      ) : (
        tableNode
      )}
      {page !== undefined && totalPages !== undefined && onPageChange && (
        <DataTablePagination
          page={page}
          totalPages={totalPages}
          totalItems={totalItems}
          onPageChange={onPageChange}
        />
      )}
    </div>
  )
}
