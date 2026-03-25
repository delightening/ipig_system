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
import { cn } from '@/lib/utils'

// ─── Column 定義 ───

export interface ColumnDef<T> {
  key: string
  header: React.ReactNode
  cell: (row: T, index: number) => React.ReactNode
  className?: string
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
}

/**
 * 統一資料表格。
 * 整合 Loading Skeleton、Empty State、Pagination。
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
}: DataTableProps<T>) {
  const colSpan = columns.length

  return (
    <div className={cn('rounded-md border', className)}>
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/30">
            {columns.map((col) => (
              <TableHead key={col.key} className={col.className}>
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
          ) : data && data.length > 0 ? (
            data.map((row, index) => (
              <TableRow
                key={rowKey(row, index)}
                className={cn(
                  onRowClick && 'cursor-pointer',
                  rowClassName?.(row, index)
                )}
                onClick={() => onRowClick?.(row)}
              >
                {columns.map((col) => (
                  <TableCell key={col.key} className={col.className}>
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
