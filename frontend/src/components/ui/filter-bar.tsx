import type React from 'react'
import { Search, X } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface FilterBarProps {
  /** 搜尋欄值 */
  search?: string
  /** 搜尋變更回呼 */
  onSearchChange?: (value: string) => void
  /** 搜尋欄 placeholder */
  searchPlaceholder?: string
  /** 是否有啟用中的篩選條件（顯示清除按鈕） */
  hasActiveFilters?: boolean
  /** 清除全部篩選 */
  onClearFilters?: () => void
  /** 篩選器插槽（下拉、日期等） */
  children?: React.ReactNode
  className?: string
}

/**
 * 統一篩選列。
 * 包含可選的搜尋框 + 任意篩選器子元素 + 可選的清除按鈕。
 */
export function FilterBar({
  search,
  onSearchChange,
  searchPlaceholder = '搜尋...',
  hasActiveFilters,
  onClearFilters,
  children,
  className,
}: FilterBarProps) {
  return (
    <div className={cn('filter-row', className)}>
      {onSearchChange !== undefined && (
        <div className="relative w-full md:w-64">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            value={search ?? ''}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder={searchPlaceholder}
            className="pl-9"
          />
        </div>
      )}
      {children}
      {hasActiveFilters && onClearFilters && (
        <Button variant="ghost" size="sm" onClick={onClearFilters} className="shrink-0">
          <X className="h-4 w-4 mr-1" />
          清除篩選
        </Button>
      )}
    </div>
  )
}
