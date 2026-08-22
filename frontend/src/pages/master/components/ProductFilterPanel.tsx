import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Search, Filter, X } from 'lucide-react'

import type { ProductListState, CategoryOption } from './productTypes'
import { STATUS_OPTIONS, BOOLEAN_OPTIONS } from './productTypes'

interface ProductFilterPanelProps {
  listState: ProductListState
  categoriesForFilter: CategoryOption[]
  showAdvancedFilters: boolean
  onToggleAdvancedFilters: () => void
}

export function ProductFilterPanel({
  listState,
  categoriesForFilter,
  showAdvancedFilters,
  onToggleAdvancedFilters,
}: ProductFilterPanelProps) {
  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-3 items-center">
        {/* 關鍵字搜尋 */}
        <div className="relative w-full sm:w-[280px]">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜尋 SKU、名稱、規格、標籤..."
            value={listState.filters.search}
            onChange={(e) => listState.setFilter('search', e.target.value)}
            className="pl-9 pr-9"
          />
          {listState.filters.search && (
            <button
              onClick={() => listState.setFilter('search', '')}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              aria-label="清除搜尋"
            >
              <X className="h-4 w-4" />
            </button>
          )}
        </div>

        {/* 品類 + 子類（綁定為一組，一起換行） */}
        <div className="flex gap-3 items-center">
          {/* 品類篩選 */}
          <Select value={listState.filters.categoryFilter} onValueChange={listState.handleCategoryChange}>
            <SelectTrigger className="w-[140px]">
              <SelectValue placeholder="品類" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部品類</SelectItem>
              {categoriesForFilter.map(cat => (
                <SelectItem key={cat.code} value={cat.code}>
                  {cat.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {/* 子類篩選 */}
          <Select
            value={listState.filters.subcategoryFilter}
            onValueChange={(v) => listState.setFilter('subcategoryFilter', v)}
            disabled={listState.filters.categoryFilter === 'all'}
          >
            <SelectTrigger className="w-[140px]">
              <SelectValue placeholder="子類" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部子類</SelectItem>
              {listState.subcategories.map(sub => (
                <SelectItem key={sub.code} value={sub.code}>
                  {sub.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {/* 狀態篩選 */}
        <Select value={listState.filters.statusFilter} onValueChange={(v) => listState.setFilter('statusFilter', v)}>
          <SelectTrigger className="w-[120px]">
            <SelectValue placeholder="狀態" />
          </SelectTrigger>
          <SelectContent>
            {STATUS_OPTIONS.map(opt => (
              <SelectItem key={opt.value} value={opt.value}>
                {opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        {/* 更多篩選按鈕 */}
        <Button
          variant={showAdvancedFilters ? "secondary" : "outline"}
          size="sm"
          onClick={onToggleAdvancedFilters}
          className="relative"
        >
          <Filter className="mr-2 h-4 w-4" />
          更多篩選
          {listState.activeFilterCount > 0 && (
            <span className="absolute -top-1 -right-1 h-4 w-4 rounded-full bg-primary text-[10px] text-primary-foreground flex items-center justify-center">
              {listState.activeFilterCount}
            </span>
          )}
        </Button>

        {/* 清除篩選 */}
        {(listState.filters.search || listState.activeFilterCount > 0) && (
          <Button variant="ghost" size="sm" onClick={listState.resetFilters}>
            <X className="mr-1 h-4 w-4" />
            清除篩選
          </Button>
        )}
      </div>

      {/* 進階篩選 */}
      {showAdvancedFilters && (
        <div className="flex flex-wrap gap-3 p-4 bg-muted/50 rounded-lg border">
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground whitespace-nowrap">追蹤批號：</span>
            <Select value={listState.filters.trackBatchFilter} onValueChange={(v) => listState.setFilter('trackBatchFilter', v)}>
              <SelectTrigger className="w-[100px]">
                <SelectValue placeholder="全部" />
              </SelectTrigger>
              <SelectContent>
                {BOOLEAN_OPTIONS.map(opt => (
                  <SelectItem key={opt.value} value={opt.value}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground whitespace-nowrap">追蹤效期：</span>
            <Select value={listState.filters.trackExpiryFilter} onValueChange={(v) => listState.setFilter('trackExpiryFilter', v)}>
              <SelectTrigger className="w-[100px]">
                <SelectValue placeholder="全部" />
              </SelectTrigger>
              <SelectContent>
                {BOOLEAN_OPTIONS.map(opt => (
                  <SelectItem key={opt.value} value={opt.value}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      )}
    </div>
  )
}
