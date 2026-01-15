import { useState, useMemo } from 'react'
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
  MoreHorizontal,
  PowerOff,
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
} from 'lucide-react'
import { formatNumber, cn } from '@/lib/utils'

// 品類定義
const CATEGORIES = [
  { code: 'DRG', name: '藥品', subcategories: [
    { code: 'ABX', name: '抗生素' },
    { code: 'ANL', name: '止痛藥' },
    { code: 'VIT', name: '維生素' },
    { code: 'OTH', name: '其他藥品' },
  ]},
  { code: 'MED', name: '醫材', subcategories: [
    { code: 'SYR', name: '注射器材' },
    { code: 'BND', name: '敷料繃帶' },
    { code: 'GLV', name: '手套' },
    { code: 'OTH', name: '其他醫材' },
  ]},
  { code: 'CON', name: '耗材', subcategories: [
    { code: 'GLV', name: '手套' },
    { code: 'GAU', name: '紗布敷料' },
    { code: 'CLN', name: '清潔消毒' },
    { code: 'TAG', name: '標示耗材' },
    { code: 'LAB', name: '實驗耗材' },
    { code: 'OTH', name: '其他耗材' },
  ]},
  { code: 'CHM', name: '化學品', subcategories: [
    { code: 'RGT', name: '試劑' },
    { code: 'SOL', name: '溶劑' },
    { code: 'STD', name: '標準品' },
    { code: 'OTH', name: '其他化學品' },
  ]},
  { code: 'EQP', name: '設備', subcategories: [
    { code: 'INS', name: '儀器' },
    { code: 'TOL', name: '工具' },
    { code: 'PRT', name: '零件' },
    { code: 'OTH', name: '其他設備' },
  ]},
]

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

// 庫存單位
const UOM_MAP: Record<string, string> = {
  'EA': '個',
  'TB': '錠',
  'CP': '膠囊',
  'BT': '瓶',
  'BX': '盒',
  'PK': '包',
  'RL': '卷',
  'SET': '組',
  'ML': 'mL',
  'L': 'L',
  'G': 'g',
  'KG': 'kg',
  'pcs': '個',
}

interface ExtendedProduct extends Product {
  category_name?: string
  subcategory_name?: string
  status?: 'active' | 'inactive' | 'discontinued'
  storage_condition?: string
}

interface PaginatedResponse<T> {
  data: T[]
  total: number
  page: number
  per_page: number
  total_pages: number
}

export function ProductsPage() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  // 搜尋與篩選狀態
  const [search, setSearch] = useState('')
  const [categoryFilter, setCategoryFilter] = useState('all')
  const [subcategoryFilter, setSubcategoryFilter] = useState('all')
  const [statusFilter, setStatusFilter] = useState('all')
  const [trackBatchFilter, setTrackBatchFilter] = useState('all')
  const [trackExpiryFilter, setTrackExpiryFilter] = useState('all')
  const [showAdvancedFilters, setShowAdvancedFilters] = useState(false)

  // 分頁與排序
  const [page, setPage] = useState(1)
  const [perPage, setPerPage] = useState(20)
  const [sortBy, setSortBy] = useState<string>('')
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc')

  // 批次選擇
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())

  // 對話框狀態
  const [statusDialogOpen, setStatusDialogOpen] = useState(false)
  const [statusAction, setStatusAction] = useState<'activate' | 'deactivate' | 'discontinue'>('activate')
  const [targetProduct, setTargetProduct] = useState<ExtendedProduct | null>(null)
  const [batchStatusDialogOpen, setBatchStatusDialogOpen] = useState(false)

  // 取得子類列表
  const subcategories = useMemo(() => {
    const category = CATEGORIES.find(c => c.code === categoryFilter)
    return category?.subcategories || []
  }, [categoryFilter])

  // 重置子類篩選當品類變更
  const handleCategoryChange = (value: string) => {
    setCategoryFilter(value)
    setSubcategoryFilter('all')
  }

  // 建立查詢參數
  const queryParams = useMemo(() => {
    const params = new URLSearchParams()
    if (search) params.append('keyword', search)
    if (categoryFilter && categoryFilter !== 'all') params.append('category_code', categoryFilter)
    if (subcategoryFilter && subcategoryFilter !== 'all') params.append('subcategory_code', subcategoryFilter)
    if (statusFilter && statusFilter !== 'all') params.append('status', statusFilter)
    if (trackBatchFilter && trackBatchFilter !== 'all') params.append('track_batch', trackBatchFilter)
    if (trackExpiryFilter && trackExpiryFilter !== 'all') params.append('track_expiry', trackExpiryFilter)
    params.append('page', page.toString())
    params.append('per_page', perPage.toString())
    if (sortBy) {
      params.append('sort_by', sortBy)
      params.append('sort_order', sortOrder)
    }
    return params.toString()
  }, [search, categoryFilter, subcategoryFilter, statusFilter, trackBatchFilter, trackExpiryFilter, page, perPage, sortBy, sortOrder])

  // 查詢產品列表
  const { data: response, isLoading, isFetching } = useQuery({
    queryKey: ['products', queryParams],
    queryFn: async () => {
      const res = await api.get<ExtendedProduct[] | PaginatedResponse<ExtendedProduct>>(`/products?${queryParams}`)
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
      setStatusDialogOpen(false)
      setTargetProduct(null)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '狀態更新失敗',
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
      toast({ title: '成功', description: `已更新 ${selectedIds.size} 個產品的狀態` })
      setBatchStatusDialogOpen(false)
      setSelectedIds(new Set())
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '批次更新失敗',
        variant: 'destructive',
      })
    },
  })

  // 處理排序
  const handleSort = (field: string) => {
    if (sortBy === field) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc')
    } else {
      setSortBy(field)
      setSortOrder('asc')
    }
    setPage(1)
  }

  // 處理全選
  const handleSelectAll = () => {
    if (selectedIds.size === products.length) {
      setSelectedIds(new Set())
    } else {
      setSelectedIds(new Set(products.map(p => p.id)))
    }
  }

  // 處理單選
  const handleSelect = (id: string) => {
    const newSelected = new Set(selectedIds)
    if (newSelected.has(id)) {
      newSelected.delete(id)
    } else {
      newSelected.add(id)
    }
    setSelectedIds(newSelected)
  }

  // 複製 SKU
  const handleCopySku = async (sku: string) => {
    await navigator.clipboard.writeText(sku)
    toast({ title: '已複製', description: `SKU: ${sku}` })
  }

  // 計算篩選數量
  const activeFilterCount = [
    categoryFilter !== 'all' ? categoryFilter : '',
    subcategoryFilter !== 'all' ? subcategoryFilter : '',
    statusFilter !== 'all' ? statusFilter : '',
    trackBatchFilter !== 'all' ? trackBatchFilter : '',
    trackExpiryFilter !== 'all' ? trackExpiryFilter : '',
  ].filter(Boolean).length

  // 清除所有篩選
  const clearAllFilters = () => {
    setSearch('')
    setCategoryFilter('all')
    setSubcategoryFilter('all')
    setStatusFilter('all')
    setTrackBatchFilter('all')
    setTrackExpiryFilter('all')
    setPage(1)
  }

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
      sortBy === field ? "text-primary" : "text-muted-foreground/50"
    )} />
  )

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">產品管理</h1>
          <p className="text-muted-foreground">管理系統中的產品/品項資料</p>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" disabled>
            <Upload className="mr-2 h-4 w-4" />
            匯入
          </Button>
          <Button variant="outline" size="sm" disabled>
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
              value={search}
              onChange={(e) => { setSearch(e.target.value); setPage(1) }}
              className="pl-9 pr-9"
            />
            {search && (
              <button
                onClick={() => setSearch('')}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                <X className="h-4 w-4" />
              </button>
            )}
          </div>

          {/* 品類篩選 */}
          <Select value={categoryFilter} onValueChange={handleCategoryChange}>
            <SelectTrigger className="w-[140px]">
              <SelectValue placeholder="品類" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部品類</SelectItem>
              {CATEGORIES.map(cat => (
                <SelectItem key={cat.code} value={cat.code}>
                  {cat.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {/* 子類篩選 */}
          <Select
            value={subcategoryFilter}
            onValueChange={(v) => { setSubcategoryFilter(v); setPage(1) }}
            disabled={categoryFilter === 'all'}
          >
            <SelectTrigger className="w-[140px]">
              <SelectValue placeholder="子類" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部子類</SelectItem>
              {subcategories.map(sub => (
                <SelectItem key={sub.code} value={sub.code}>
                  {sub.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {/* 狀態篩選 */}
          <Select value={statusFilter} onValueChange={(v) => { setStatusFilter(v); setPage(1) }}>
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
            onClick={() => setShowAdvancedFilters(!showAdvancedFilters)}
            className="relative"
          >
            <Filter className="mr-2 h-4 w-4" />
            更多篩選
            {activeFilterCount > 0 && (
              <span className="absolute -top-1 -right-1 h-4 w-4 rounded-full bg-primary text-[10px] text-primary-foreground flex items-center justify-center">
                {activeFilterCount}
              </span>
            )}
          </Button>

          {/* 清除篩選 */}
          {(search || activeFilterCount > 0) && (
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
              <Select value={trackBatchFilter} onValueChange={(v) => { setTrackBatchFilter(v); setPage(1) }}>
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
              <Select value={trackExpiryFilter} onValueChange={(v) => { setTrackExpiryFilter(v); setPage(1) }}>
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
      {selectedIds.size > 0 && (
        <div className="flex items-center gap-3 p-3 bg-primary/5 border border-primary/20 rounded-lg animate-fade-in">
          <div className="flex items-center gap-2">
            <Check className="h-4 w-4 text-primary" />
            <span className="text-sm font-medium">
              已選擇 {selectedIds.size} 個產品
            </span>
          </div>
          <div className="flex-1" />
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              setStatusAction('deactivate')
              setBatchStatusDialogOpen(true)
            }}
          >
            <PowerOff className="mr-2 h-4 w-4" />
            批次停用
          </Button>
          <Button variant="outline" size="sm" disabled>
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
            onClick={() => setSelectedIds(new Set())}
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
                  checked={products.length > 0 && selectedIds.size === products.length}
                  onChange={handleSelectAll}
                  className="h-4 w-4 rounded border-input"
                />
              </TableHead>
              <TableHead
                className="w-[180px] cursor-pointer select-none"
                onClick={() => handleSort('sku')}
              >
                SKU <SortIndicator field="sku" />
              </TableHead>
              <TableHead
                className="cursor-pointer select-none"
                onClick={() => handleSort('name')}
              >
                名稱 <SortIndicator field="name" />
              </TableHead>
              <TableHead className="w-[150px]">規格</TableHead>
              <TableHead className="w-[60px]">單位</TableHead>
              <TableHead
                className="w-[100px] text-right cursor-pointer select-none"
                onClick={() => handleSort('safety_stock')}
              >
                安全庫存 <SortIndicator field="safety_stock" />
              </TableHead>
              <TableHead className="w-[60px] text-center">批號</TableHead>
              <TableHead className="w-[60px] text-center">效期</TableHead>
              <TableHead
                className="w-[80px] cursor-pointer select-none"
                onClick={() => handleSort('status')}
              >
                狀態 <SortIndicator field="status" />
              </TableHead>
              <TableHead className="w-[60px] text-right">操作</TableHead>
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
                    {search || activeFilterCount > 0 ? '找不到符合條件的產品' : '尚無產品資料'}
                  </p>
                  {!search && activeFilterCount === 0 && (
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
                    selectedIds.has(product.id) && "bg-primary/5"
                  )}
                >
                  <TableCell>
                    <input
                      type="checkbox"
                      checked={selectedIds.has(product.id)}
                      onChange={() => handleSelect(product.id)}
                      className="h-4 w-4 rounded border-input"
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
                    <div className="relative inline-block">
                      <select
                        className="appearance-none bg-transparent border-0 cursor-pointer p-1 rounded hover:bg-muted focus:outline-none focus:ring-2 focus:ring-ring text-transparent w-8"
                        onChange={(e) => {
                          const action = e.target.value
                          e.target.value = ''
                          switch (action) {
                            case 'view':
                              navigate(`/products/${product.id}`)
                              break
                            case 'edit':
                              navigate(`/products/${product.id}/edit`)
                              break
                            case 'copy':
                              navigate(`/products/new?copy=${product.id}`)
                              break
                            case 'activate':
                              setTargetProduct(product)
                              setStatusAction('activate')
                              setStatusDialogOpen(true)
                              break
                            case 'deactivate':
                              setTargetProduct(product)
                              setStatusAction('deactivate')
                              setStatusDialogOpen(true)
                              break
                            case 'discontinue':
                              setTargetProduct(product)
                              setStatusAction('discontinue')
                              setStatusDialogOpen(true)
                              break
                          }
                        }}
                        title="操作選單"
                      >
                        <option value="">⋯</option>
                        <option value="view">檢視</option>
                        <option value="edit">編輯</option>
                        <option value="copy">複製</option>
                        <option disabled>───</option>
                        {product.is_active ? (
                          <option value="deactivate">停用</option>
                        ) : (
                          <option value="activate">啟用</option>
                        )}
                        <option value="discontinue">標記停產</option>
                      </select>
                      <MoreHorizontal className="absolute right-1 top-1/2 -translate-y-1/2 h-4 w-4 pointer-events-none" />
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
            顯示 {(page - 1) * perPage + 1}-{Math.min(page * perPage, totalItems)} 共 {totalItems} 筆
            {isFetching && !isLoading && (
              <Loader2 className="inline-block ml-2 h-3 w-3 animate-spin" />
            )}
          </div>
          <div className="flex items-center gap-2">
            <Select
              value={perPage.toString()}
              onValueChange={(v) => { setPerPage(parseInt(v)); setPage(1) }}
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
                onClick={() => setPage(1)}
                disabled={page === 1}
              >
                <ChevronsLeft className="h-4 w-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                onClick={() => setPage(p => Math.max(1, p - 1))}
                disabled={page === 1}
              >
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
                      onClick={() => setPage(pageNum)}
                    >
                      {pageNum}
                    </Button>
                  )
                })}
              </div>
              <Button
                variant="outline"
                size="icon"
                onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                disabled={page === totalPages}
              >
                <ChevronRight className="h-4 w-4" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                onClick={() => setPage(totalPages)}
                disabled={page === totalPages}
              >
                <ChevronsRight className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* 狀態變更對話框 */}
      <Dialog open={statusDialogOpen} onOpenChange={setStatusDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {statusAction === 'activate' && '啟用產品'}
              {statusAction === 'deactivate' && '停用產品'}
              {statusAction === 'discontinue' && '標記停產'}
            </DialogTitle>
            <DialogDescription>
              {statusAction === 'activate' && '確定要啟用此產品嗎？啟用後可在採購、銷售等模組中使用。'}
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
              onClick={() => setStatusDialogOpen(false)}
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
      <Dialog open={batchStatusDialogOpen} onOpenChange={setBatchStatusDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>批次停用產品</DialogTitle>
            <DialogDescription>
              確定要停用選中的 {selectedIds.size} 個產品嗎？停用後將無法在新單據中選擇這些產品。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setBatchStatusDialogOpen(false)}
              disabled={batchStatusMutation.isPending}
            >
              取消
            </Button>
            <Button
              onClick={() => {
                batchStatusMutation.mutate({
                  ids: Array.from(selectedIds),
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
    </div>
  )
}
