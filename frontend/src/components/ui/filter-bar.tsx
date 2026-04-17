import type React from 'react'
import { useState, useEffect } from 'react'
import { Search, X, SlidersHorizontal, ChevronDown } from 'lucide-react'
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
 * 桌面端並排，行動端：搜尋框常駐 + 額外篩選可收合。
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
  const [mobileExpanded, setMobileExpanded] = useState(hasActiveFilters ?? false)
  const hasExtraFilters = Boolean(children)

  useEffect(() => {
    if (hasActiveFilters) setMobileExpanded(true)
  }, [hasActiveFilters])

  const searchInput = onSearchChange !== undefined && (
    <div className="relative w-full md:w-64">
      <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
      <Input
        value={search ?? ''}
        onChange={(e) => onSearchChange(e.target.value)}
        placeholder={searchPlaceholder}
        className="pl-9"
      />
    </div>
  )

  return (
    <div className={cn('space-y-2', className)}>
      {/* 行動端：搜尋 + 展開按鈕同列 */}
      <div className="flex items-center gap-2 md:hidden">
        {searchInput && <div className="flex-1">{searchInput}</div>}
        {hasExtraFilters && (
          <Button
            variant="outline"
            size="sm"
            className={cn("shrink-0 gap-1.5 h-10", hasActiveFilters && "border-primary text-primary")}
            onClick={() => setMobileExpanded((prev) => !prev)}
            aria-expanded={mobileExpanded}
          >
            <SlidersHorizontal className="h-4 w-4" />
            {hasActiveFilters && <span className="h-2 w-2 rounded-full bg-primary" aria-hidden="true" />}
            <ChevronDown className={cn("h-3.5 w-3.5 transition-transform", mobileExpanded && "rotate-180")} />
          </Button>
        )}
        {hasActiveFilters && onClearFilters && (
          <Button variant="ghost" size="icon" className="shrink-0 h-10 w-10" onClick={onClearFilters} aria-label="清除篩選">
            <X className="h-4 w-4" />
          </Button>
        )}
      </div>

      {/* 行動端：可展開的額外篩選 */}
      {hasExtraFilters && mobileExpanded && (
        <div className="flex flex-col gap-3 md:hidden">
          {children}
        </div>
      )}

      {/* 桌面端：全部並排 */}
      <div className="hidden md:flex md:items-center md:gap-4">
        {searchInput}
        {children}
        {hasActiveFilters && onClearFilters && (
          <Button variant="ghost" size="sm" onClick={onClearFilters} className="shrink-0">
            <X className="h-4 w-4 mr-1" />
            清除篩選
          </Button>
        )}
      </div>
    </div>
  )
}
