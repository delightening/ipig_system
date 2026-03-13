import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Loader2,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
} from 'lucide-react'

import type { ProductListState } from './productTypes'

interface ProductPaginationProps {
  listState: ProductListState
  totalItems: number
  totalPages: number
  isFetching: boolean
  isLoading: boolean
}

export function ProductPagination({
  listState,
  totalItems,
  totalPages,
  isFetching,
  isLoading,
}: ProductPaginationProps) {
  return (
    <div className="flex items-center justify-between">
      <div className="text-sm text-muted-foreground">
        顯示 {(listState.page - 1) * listState.perPage + 1}-{Math.min(listState.page * listState.perPage, totalItems)} 共 {totalItems} 筆
        {isFetching && !isLoading && (
          <Loader2 className="inline-block ml-2 h-3 w-3 animate-spin" />
        )}
      </div>
      <div className="flex items-center gap-2">
        <PerPageSelect
          perPage={listState.perPage}
          onChange={(v) => { listState.setPerPage(v); listState.setPage(1) }}
        />
        <PageButtons
          page={listState.page}
          totalPages={totalPages}
          onPageChange={listState.setPage}
        />
      </div>
    </div>
  )
}

/** 每頁筆數選擇器 */
function PerPageSelect({ perPage, onChange }: { perPage: number; onChange: (v: number) => void }) {
  return (
    <Select
      value={perPage.toString()}
      onValueChange={(v) => onChange(parseInt(v))}
    >
      <SelectTrigger className="w-[100px]">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="10">10 筆/頁</SelectItem>
        <SelectItem value="20">20 筆/頁</SelectItem>
        <SelectItem value="50">50 筆/頁</SelectItem>
        <SelectItem value="100">100 筆/頁</SelectItem>
      </SelectContent>
    </Select>
  )
}

/** 頁碼按鈕群組 */
function PageButtons({
  page,
  totalPages,
  onPageChange,
}: {
  page: number
  totalPages: number
  onPageChange: (p: number) => void
}) {
  return (
    <div className="flex items-center gap-1">
      <Button variant="outline" size="icon" onClick={() => onPageChange(1)} disabled={page === 1}>
        <ChevronsLeft className="h-4 w-4" />
      </Button>
      <Button variant="outline" size="icon" onClick={() => onPageChange(Math.max(1, page - 1))} disabled={page === 1}>
        <ChevronLeft className="h-4 w-4" />
      </Button>
      <div className="flex items-center gap-1 px-2">
        {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
          let pageNum: number
          if (totalPages <= 5) {
            pageNum = i + 1
          } else if (page <= 3) {
            pageNum = i + 1
          } else if (page >= totalPages - 2) {
            pageNum = totalPages - 4 + i
          } else {
            pageNum = page - 2 + i
          }
          return (
            <Button
              key={pageNum}
              variant={page === pageNum ? "default" : "ghost"}
              size="sm"
              className="w-8 h-8 p-0"
              onClick={() => onPageChange(pageNum)}
            >
              {pageNum}
            </Button>
          )
        })}
      </div>
      <Button variant="outline" size="icon" onClick={() => onPageChange(Math.min(totalPages, page + 1))} disabled={page === totalPages}>
        <ChevronRight className="h-4 w-4" />
      </Button>
      <Button variant="outline" size="icon" onClick={() => onPageChange(totalPages)} disabled={page === totalPages}>
        <ChevronsRight className="h-4 w-4" />
      </Button>
    </div>
  )
}
