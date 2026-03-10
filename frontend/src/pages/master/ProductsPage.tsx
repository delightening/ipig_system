import { useState, useMemo } from 'react'
import { useToggle } from '@/hooks/useToggle'
import { useSelection } from '@/hooks/useSelection'
import { useProductListState } from './hooks/useProductListState'
import { useDialogSet } from '@/hooks/useDialogSet'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Product } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import {
  Plus,
  Search,
  Loader2,
  Package,
  Eye,
  Pencil,
  Copy,
  Power,
  PowerOff,
  Ban,
  Download,
  Upload,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  ArrowUpDown,
  Filter,
  X,
  Check,
  Tags,
  ClipboardCopy,
  FolderEdit,
  Trash2,
} from 'lucide-react'
import { formatNumber, cn, UOM_MAP } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'
import { ProductImportDialog } from '@/components/product/ProductImportDialog'
import { EditCategoriesDialog } from '@/components/product/EditCategoriesDialog'
import { useSkuCategories } from '@/hooks/useSkuCategories'
import { useAuthStore } from '@/stores/auth'
import type { CategoryOption } from './hooks/useProductListState'
import type { PaginatedResponse } from '@/types/common'

// 產品狀態
const STATUS_OPTIONS = [
  { value: 'all', label: '全部狀態' },
  { value: 'active', label: '啟用' },
  { value: 'inactive', label: '停用' },
  { value: 'discontinued', label: '停產' },
]

// 布林篩選選項
const BOOLEAN_OPTIONS = [
  { value: 'all', label: '全部' },
  { value: 'true', label: '是' },
  { value: 'false', label: '否' },
]



interface ExtendedProduct extends Product {
  category_code?: string
  subcategory_code?: string
  category_name?: string
  subcategory_name?: string
  status?: 'active' | 'inactive' | 'discontinued'
  storage_condition?: string
}

export function ProductsPage() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { user, hasRole } = useAuthStore()
  const { categories: skuCategories, subcategoriesByCategory } = useSkuCategories({ enabled: !!user })
  const categoriesForFilter: CategoryOption[] = useMemo(
    () =>
      skuCategories.map((c) => ({
        code: c.code,
        name: c.name,
        subcategories: subcategoriesByCategory[c.code] ?? [],
      })),
    [skuCategories, subcategoriesByCategory]
  )

  const listState = useProductListState(categoriesForFilter)
  const [showAdvancedFilters, toggleAdvancedFilters] = useToggle()

  // 批次選擇
  const selection = useSelection<string>()

  // 對話框狀態
  const dialogs = useDialogSet(['status', 'batchStatus', 'import', 'editCategories', 'hardDelete'] as const)
  const [statusAction, setStatusAction] = useState<'activate' | 'deactivate' | 'discontinue'>('activate')
  const [targetProduct, setTargetProduct] = useState<ExtendedProduct | null>(null)
  const [hardDeleteProduct, setHardDeleteProduct] = useState<ExtendedProduct | null>(null)
  const isAdmin = hasRole('admin') || hasRole('SYSTEM_ADMIN')

  // 查詢產品列表（僅在已登入時發送，避免 session 失效時大量 401）
  const { data: response, isLoading, isFetching } = useQuery({
    queryKey: ['products', listState.queryParams],
    queryFn: async () => {
      const res = await api.get<ExtendedProduct[] | PaginatedResponse<ExtendedProduct>>(`/products?${listState.queryParams}`)
      // 處理非分頁和分頁兩種回應格式
      if (Array.isArray(res.data)) {
        return {
          data: res.data,
          total: res.data.length,
          page: 1,
          per_page: res.data.length,
          total_pages: 1,
        }
      }
      return res.data
    },
    enabled: !!user,
  })

  const products = response?.data || []
  const totalItems = response?.total || 0
  const totalPages = response?.total_pages || 1

  // 變更狀態 Mutation
  const statusMutation = useMutation({
    mutationFn: async ({ id, status }: { id: string; status: string }) => {
      return api.patch(`/products/${id}/status`, { status })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] })
      toast({ title: '成功', description: '產品狀態已更新' })
      dialogs.close('status')
      setTargetProduct(null)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '狀態更新失敗'),
        variant: 'destructive',
      })
    },
  })

  // 硬刪除（僅 admin）
  const hardDeleteMutation = useMutation({
    mutationFn: async (id: string) => {
      return api.post(`/products/${id}/hard-delete`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] })
      toast({ title: '成功', description: '產品已永久刪除' })
      dialogs.close('hardDelete')
      setHardDeleteProduct(null)
    },
    onError: (error: unknown) => {
      toast({
        title: '硬刪除失敗',
        description: getApiErrorMessage(error, '無法硬刪除產品'),
        variant: 'destructive',
      })
    },
  })

  // 批次變更狀態
  const batchStatusMutation = useMutation({
    mutationFn: async ({ ids, status }: { ids: string[]; status: string }) => {
      return Promise.all(ids.map(id => api.patch(`/products/${id}/status`, { status })))
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] })
      toast({ title: '成功', description: `已更新 ${selection.size} 個產品的狀態` })
      dialogs.close('batchStatus')
      selection.clear()
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '批次更新失敗'),
        variant: 'destructive',
      })
    },
  })

  // 處理全選
  const handleSelectAll = () => {
    selection.selectAll(products.map(p => p.id))
  }

  // 處理單選
  const handleSelect = (id: string) => {
    selection.toggle(id)
  }

  // 複製 SKU
  const handleCopySku = async (sku: string) => {
    await navigator.clipboard.writeText(sku)
    toast({ title: '已複製', description: `SKU: ${sku}` })
  }


  // 清除所有篩選
  const clearAllFilters = () => listState.resetFilters()

  // 取得狀態 Badge
  const getStatusBadge = (product: ExtendedProduct) => {
    const status = product.status || (product.is_active ? 'active' : 'inactive')
    switch (status) {
      case 'active':
        return <Badge variant="success">啟用</Badge>
      case 'inactive':
        return <Badge variant="warning">停用</Badge>
      case 'discontinued':
        return <Badge variant="destructive">停產</Badge>
      default:
        return <Badge variant="secondary">未知</Badge>
    }
  }

  // 排序指示器
  const SortIndicator = ({ field }: { field: string }) => (
    <ArrowUpDown className={cn(
      "ml-1 h-3 w-3 inline-block transition-colors",
      listState.sortBy === field ? "text-primary" : "text-muted-foreground/50"
    )} />
  )

  // 匯出 CSV
  const handleExportCSV = () => {
    if (products.length === 0) {
      toast({ title: '無資料可匯出', description: '請先篩選或新增產品', variant: 'destructive' })
      return
    }
    const headers = ['SKU', '名稱', '規格', '品類', '子類', '單位', '安全庫存', '追蹤批號', '追蹤效期', '狀態']
    const rows = products.map(p => [
      p.sku,
      p.name,
      p.spec || '',
      p.category_code || '',
      p.subcategory_code || '',
      UOM_MAP[p.base_uom] || p.base_uom,
      p.safety_stock?.toString() ?? '',
      p.track_batch ? '是' : '否',
      p.track_expiry ? '是' : '否',
      p.is_active ? '啟用' : '停用',
    ])
    const csvContent = [headers, ...rows]
      .map(row => row.map(cell => `"${String(cell).replace(/"/g, '""')}"`).join(','))
      .join('\n')
    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `products_${new Date().toISOString().split('T')[0]}.csv`
    link.click()
    URL.revokeObjectURL(link.href)
    toast({ title: '匯出成功', description: `已匯出 ${products.length} 筆產品` })
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">產品管理</h1>
          <p className="text-muted-foreground">管理系統中的產品/品項資料</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={() => dialogs.open('editCategories')}>
            <FolderEdit className="mr-2 h-4 w-4" />
            編輯分類
          </Button>
          <Button variant="outline" size="sm" onClick={() => dialogs.open('import')}>
            <Upload className="mr-2 h-4 w-4" />
            匯入
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={handleExportCSV}
            disabled={products.length === 0}
          >
            <Download className="mr-2 h-4 w-4" />
            匯出
          </Button>
          <Button onClick={() => navigate('/products/new')}>
            <Plus className="mr-2 h-4 w-4" />
            新增產品
          </Button>
        </div>
      </div>

      {/* Search & Filters */}
      <div className="space-y-4">
        <div className="flex flex-wrap gap-3 items-center">
          {/* 關鍵字搜尋 */}
          <div className="relative flex-1 min-w-[240px] max-w-md">
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
            onClick={toggleAdvancedFilters}
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
            <Button variant="ghost" size="sm" onClick={clearAllFilters}>
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

      {/* Batch Actions Bar */}
      {selection.size > 0 && (
        <div className="flex items-center gap-3 p-3 bg-primary/5 border border-primary/20 rounded-lg animate-fade-in">
          <div className="flex items-center gap-2">
            <Check className="h-4 w-4 text-primary" />
            <span className="text-sm font-medium">
              已選擇 {selection.size} 個產品
            </span>
          </div>
          <div className="flex-1" />
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              setStatusAction('deactivate')
              dialogs.open('batchStatus')
            }}
          >
            <PowerOff className="mr-2 h-4 w-4" />
            批次停用
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              const toExport = products.filter(p => selection.has(p.id))
              if (toExport.length === 0) return
              const headers = ['SKU', '名稱', '規格', '品類', '子類', '單位', '安全庫存', '追蹤批號', '追蹤效期', '狀態']
              const rows = toExport.map(p => [
                p.sku,
                p.name,
                p.spec || '',
                p.category_code || '',
                p.subcategory_code || '',
                UOM_MAP[p.base_uom] || p.base_uom,
                p.safety_stock?.toString() ?? '',
                p.track_batch ? '是' : '否',
                p.track_expiry ? '是' : '否',
                p.is_active ? '啟用' : '停用',
              ])
              const csvContent = [headers, ...rows]
                .map(row => row.map(cell => `"${String(cell).replace(/"/g, '""')}"`).join(','))
                .join('\n')
              const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
              const link = document.createElement('a')
              link.href = URL.createObjectURL(blob)
              link.download = `products_selected_${new Date().toISOString().split('T')[0]}.csv`
              link.click()
              URL.revokeObjectURL(link.href)
              toast({ title: '匯出成功', description: `已匯出 ${toExport.length} 筆產品` })
            }}
          >
            <Download className="mr-2 h-4 w-4" />
            批次匯出
          </Button>
          <Button variant="outline" size="sm" disabled>
            <Tags className="mr-2 h-4 w-4" />
            批次設定標籤
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => selection.clear()}
          >
            取消選擇
          </Button>
        </div>
      )}

      {/* Table */}
      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <TableHead className="w-[40px]">
                <input
                  type="checkbox"
                  checked={products.length > 0 && selection.size === products.length}
                  onChange={handleSelectAll}
                  className="h-4 w-4 rounded border-input"
                  aria-label="全選產品"
                />
              </TableHead>
              <TableHead
                className="w-[180px] cursor-pointer select-none"
                onClick={() => listState.handleSort('sku')}
              >
                SKU <SortIndicator field="sku" />
              </TableHead>
              <TableHead
                className="cursor-pointer select-none"
                onClick={() => listState.handleSort('name')}
              >
                名稱 <SortIndicator field="name" />
              </TableHead>
              <TableHead className="w-[150px]">規格</TableHead>
              <TableHead className="w-[60px]">單位</TableHead>
              <TableHead
                className="w-[100px] text-right cursor-pointer select-none"
                onClick={() => listState.handleSort('safety_stock')}
              >
                安全庫存 <SortIndicator field="safety_stock" />
              </TableHead>
              <TableHead className="w-[60px] text-center">批號</TableHead>
              <TableHead className="w-[60px] text-center">效期</TableHead>
              <TableHead
                className="w-[80px] cursor-pointer select-none"
                onClick={() => listState.handleSort('status')}
              >
                狀態 <SortIndicator field="status" />
              </TableHead>
              <TableHead className="w-[200px] text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={10} className="text-center py-12">
                  <Loader2 className="h-8 w-8 animate-spin mx-auto text-muted-foreground" />
                  <p className="mt-2 text-sm text-muted-foreground">載入中...</p>
                </TableCell>
              </TableRow>
            ) : products.length === 0 ? (
              <TableRow>
                <TableCell colSpan={10} className="text-center py-12">
                  <Package className="h-12 w-12 mx-auto mb-3 text-muted-foreground/50" />
                  <p className="text-muted-foreground">
                    {listState.filters.search || listState.activeFilterCount > 0 ? '找不到符合條件的產品' : '尚無產品資料'}
                  </p>
                  {!listState.filters.search && listState.activeFilterCount === 0 && (
                    <Button
                      variant="outline"
                      className="mt-4"
                      onClick={() => navigate('/products/new')}
                    >
                      <Plus className="mr-2 h-4 w-4" />
                      建立第一個產品
                    </Button>
                  )}
                </TableCell>
              </TableRow>
            ) : (
              products.map((product) => (
                <TableRow
                  key={product.id}
                  className={cn(
                    "group",
                    selection.has(product.id) && "bg-primary/5"
                  )}
                >
                  <TableCell>
                    <input
                      type="checkbox"
                      checked={selection.has(product.id)}
                      onChange={() => handleSelect(product.id)}
                      className="h-4 w-4 rounded border-input"
                      aria-label={`選擇產品 ${product.sku}`}
                    />
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-1.5 group/sku">
                      <code
                        className="font-mono text-xs bg-muted px-1.5 py-0.5 rounded cursor-pointer hover:bg-muted-foreground/20 transition-colors max-w-[160px] truncate"
                        title={product.sku}
                        onClick={() => handleCopySku(product.sku)}
                      >
                        {product.sku}
                      </code>
                      <button
                        onClick={() => handleCopySku(product.sku)}
                        className="opacity-0 group-hover/sku:opacity-100 transition-opacity"
                        title="複製 SKU"
                        aria-label="複製 SKU"
                      >
                        <ClipboardCopy className="h-3 w-3 text-muted-foreground hover:text-foreground" />
                      </button>
                    </div>
                  </TableCell>
                  <TableCell>
                    <button
                      className="font-medium text-left hover:text-primary hover:underline transition-colors"
                      onClick={() => navigate(`/products/${product.id}`)}
                    >
                      {product.name}
                    </button>
                  </TableCell>
                  <TableCell className="text-muted-foreground">
                    {product.spec || '-'}
                  </TableCell>
                  <TableCell>
                    <span className="text-xs px-1.5 py-0.5 bg-muted rounded">
                      {UOM_MAP[product.base_uom] || product.base_uom}
                    </span>
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {product.safety_stock ? (
                      <span>
                        {formatNumber(product.safety_stock, 0)}
                        <span className="text-muted-foreground text-xs ml-1">
                          {UOM_MAP[product.base_uom] || product.base_uom}
                        </span>
                      </span>
                    ) : (
                      <span className="text-muted-foreground">-</span>
                    )}
                  </TableCell>
                  <TableCell className="text-center">
                    {product.track_batch ? (
                      <Badge variant="secondary" className="text-[10px] px-1.5">啟用</Badge>
                    ) : (
                      <span className="text-muted-foreground">-</span>
                    )}
                  </TableCell>
                  <TableCell className="text-center">
                    {product.track_expiry ? (
                      <Badge variant="secondary" className="text-[10px] px-1.5">啟用</Badge>
                    ) : (
                      <span className="text-muted-foreground">-</span>
                    )}
                  </TableCell>
                  <TableCell>
                    {getStatusBadge(product)}
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end gap-0.5" aria-label={`產品 ${product.sku} 操作`}>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8"
                        title="檢視"
                        onClick={() => navigate(`/products/${product.id}`)}
                      >
                        <Eye className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8"
                        title="編輯"
                        onClick={() => navigate(`/products/${product.id}/edit`)}
                      >
                        <Pencil className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8"
                        title="複製"
                        onClick={() => navigate(`/products/new?copy=${product.id}`)}
                      >
                        <Copy className="h-4 w-4" />
                      </Button>
                      {product.is_active ? (
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          title="停用"
                          onClick={() => {
                            setTargetProduct(product)
                            setStatusAction('deactivate')
                            dialogs.open('status')
                          }}
                        >
                          <PowerOff className="h-4 w-4 text-destructive" />
                        </Button>
                      ) : (
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          title="啟用"
                          onClick={() => {
                            setTargetProduct(product)
                            setStatusAction('activate')
                            dialogs.open('status')
                          }}
                        >
                          <Power className="h-4 w-4 text-green-600" />
                        </Button>
                      )}
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8"
                        title="標記停產"
                        onClick={() => {
                          setTargetProduct(product)
                          setStatusAction('discontinue')
                          dialogs.open('status')
                        }}
                      >
                        <Ban className="h-4 w-4 text-muted-foreground" />
                      </Button>
                      {isAdmin && (
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-destructive hover:text-destructive"
                          title="硬刪除（僅管理員）"
                          onClick={() => {
                            setHardDeleteProduct(product)
                            dialogs.open('hardDelete')
                          }}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {/* Pagination */}
      {products.length > 0 && (
        <div className="flex items-center justify-between">
          <div className="text-sm text-muted-foreground">
            顯示 {(listState.page - 1) * listState.perPage + 1}-{Math.min(listState.page * listState.perPage, totalItems)} 共 {totalItems} 筆
            {isFetching && !isLoading && (
              <Loader2 className="inline-block ml-2 h-3 w-3 animate-spin" />
            )}
          </div>
          <div className="flex items-center gap-2">
            <Select
              value={listState.perPage.toString()}
              onValueChange={(v) => { listState.setPerPage(parseInt(v)); listState.setPage(1) }}
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
            <div className="flex items-center gap-1">
              <Button
                variant="outline"
                size="icon"
                onClick={() => listState.setPage(1)}
                disabled={listState.page === 1}
              >
                <ChevronsLeft className="h-4 w-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                onClick={() => listState.setPage(Math.max(1, listState.page - 1))}
                disabled={listState.page === 1}
              >
                <ChevronLeft className="h-4 w-4" />
              </Button>
              <div className="flex items-center gap-1 px-2">
                {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                  let pageNum: number
                  if (totalPages <= 5) {
                    pageNum = i + 1
                  } else if (listState.page <= 3) {
                    pageNum = i + 1
                  } else if (listState.page >= totalPages - 2) {
                    pageNum = totalPages - 4 + i
                  } else {
                    pageNum = listState.page - 2 + i
                  }
                  return (
                    <Button
                      key={pageNum}
                      variant={listState.page === pageNum ? "default" : "ghost"}
                      size="sm"
                      className="w-8 h-8 p-0"
                      onClick={() => listState.setPage(pageNum)}
                    >
                      {pageNum}
                    </Button>
                  )
                })}
              </div>
              <Button
                variant="outline"
                size="icon"
                onClick={() => listState.setPage(Math.min(totalPages, listState.page + 1))}
                disabled={listState.page === totalPages}
              >
                <ChevronRight className="h-4 w-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                onClick={() => listState.setPage(totalPages)}
                disabled={listState.page === totalPages}
              >
                <ChevronsRight className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* 狀態變更對話框 */}
      <Dialog open={dialogs.isOpen('status')} onOpenChange={dialogs.setOpen('status')}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {statusAction === 'activate' && '啟用產品'}
              {statusAction === 'deactivate' && '停用產品'}
              {statusAction === 'discontinue' && '標記停產'}
            </DialogTitle>
            <DialogDescription>
              {statusAction === 'activate' && '確定要啟用此產品嗎？啟用後可在採購、銷貨等模組中使用。'}
              {statusAction === 'deactivate' && '確定要停用此產品嗎？停用後將無法在新單據中選擇此產品。'}
              {statusAction === 'discontinue' && '確定要將此產品標記為停產嗎？停產後僅供歷史查詢，無法恢復為啟用狀態。'}
            </DialogDescription>
          </DialogHeader>
          {targetProduct && (
            <div className="py-4">
              <div className="p-3 bg-muted rounded-lg">
                <div className="flex items-center gap-3">
                  <Package className="h-8 w-8 text-muted-foreground" />
                  <div>
                    <p className="font-medium">{targetProduct.name}</p>
                    <p className="text-sm text-muted-foreground font-mono">{targetProduct.sku}</p>
                  </div>
                </div>
              </div>
            </div>
          )}
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => dialogs.close('status')}
              disabled={statusMutation.isPending}
            >
              取消
            </Button>
            <Button
              variant={statusAction === 'discontinue' ? 'destructive' : 'default'}
              onClick={() => {
                if (!targetProduct) return
                const status = statusAction === 'activate' ? 'active'
                  : statusAction === 'deactivate' ? 'inactive'
                    : 'discontinued'
                statusMutation.mutate({ id: targetProduct.id, status })
              }}
              disabled={statusMutation.isPending}
            >
              {statusMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              確認
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 批次狀態變更對話框 */}
      <Dialog open={dialogs.isOpen('batchStatus')} onOpenChange={dialogs.setOpen('batchStatus')}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>批次停用產品</DialogTitle>
            <DialogDescription>
              確定要停用選中的 {selection.size} 個產品嗎？停用後將無法在新單據中選擇這些產品。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => dialogs.close('batchStatus')}
              disabled={batchStatusMutation.isPending}
            >
              取消
            </Button>
            <Button
              onClick={() => {
                batchStatusMutation.mutate({
                  ids: Array.from(selection.selectedIds),
                  status: 'inactive',
                })
              }}
              disabled={batchStatusMutation.isPending}
            >
              {batchStatusMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              確認停用
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 硬刪除確認對話框（僅 admin） */}
      <Dialog open={dialogs.isOpen('hardDelete')} onOpenChange={dialogs.setOpen('hardDelete')}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="text-destructive">硬刪除產品</DialogTitle>
            <DialogDescription>
              此操作將永久刪除產品資料，無法復原。若產品已有單據、庫存或藥物選單關聯則無法執行。確定要硬刪除此產品嗎？
            </DialogDescription>
          </DialogHeader>
          {hardDeleteProduct && (
            <div className="py-4">
              <div className="p-3 bg-destructive/10 border border-destructive/30 rounded-lg">
                <div className="flex items-center gap-3">
                  <Package className="h-8 w-8 text-destructive" />
                  <div>
                    <p className="font-medium">{hardDeleteProduct.name}</p>
                    <p className="text-sm text-muted-foreground font-mono">{hardDeleteProduct.sku}</p>
                  </div>
                </div>
              </div>
            </div>
          )}
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                dialogs.close('hardDelete')
                setHardDeleteProduct(null)
              }}
              disabled={hardDeleteMutation.isPending}
            >
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                if (!hardDeleteProduct) return
                hardDeleteMutation.mutate(hardDeleteProduct.id)
              }}
              disabled={hardDeleteMutation.isPending}
            >
              {hardDeleteMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              確認硬刪除
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 產品匯入對話框 */}
      <ProductImportDialog open={dialogs.isOpen('import')} onOpenChange={dialogs.setOpen('import')} />
      <EditCategoriesDialog open={dialogs.isOpen('editCategories')} onOpenChange={dialogs.setOpen('editCategories')} />
    </div>
  )
}
