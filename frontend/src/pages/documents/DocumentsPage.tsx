import { useState } from 'react'
import { useSearchParams, Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource } from '@/lib/api'
import type { DocType, DocumentListItem } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useToast } from '@/components/ui/use-toast'
import { Plus, Search, Loader2, Calendar, X, FileText, Truck, ShoppingCart, Warehouse } from 'lucide-react'
import { STALE_TIME } from '@/lib/query'
import { getApiErrorMessage } from '@/lib/validation'
import { useAuthStore } from '@/stores/auth'
import { cn } from '@/lib/utils'
import { DocumentTable } from './components/DocumentTable'
import { useDocumentCategory, type DocCategory } from './hooks/useDocumentCategory'


const CATEGORY_TYPES: Record<DocCategory, DocType[]> = {
  purchasing: ['PO', 'GRN', 'PR'],
  sales: ['SO', 'DO'],
  warehouse: ['TR', 'STK', 'ADJ', 'RM'],
}

const CATEGORY_CONFIG: Record<DocCategory, { label: string; icon: React.ReactNode; desc: string }> = {
  purchasing: {
    label: '採購類',
    icon: <Truck className="h-4 w-4" />,
    desc: '採購單、採購入庫、採購退貨',
  },
  sales: {
    label: '銷貨類',
    icon: <ShoppingCart className="h-4 w-4" />,
    desc: '銷貨單、銷貨出庫',
  },
  warehouse: {
    label: '倉儲類',
    icon: <Warehouse className="h-4 w-4" />,
    desc: '調撥單、盤點單、調整單、退料單',
  },
}

const TYPE_NAMES: Record<DocType, string> = {
  PO: '採購單',
  GRN: '採購入庫',
  PR: '採購退貨',
  SO: '銷貨單',
  DO: '銷貨出庫',
  TR: '調撥單',
  STK: '盤點單',
  ADJ: '調整單',
  RM: '退料單',
}

const STATUS_NAMES: Record<string, string> = {
  draft: '草稿',
  submitted: '待核准',
  approved: '已核准',
  cancelled: '已作廢',
}

export function DocumentsPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const typeFilter = searchParams.get('type') || ''
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')
  const [dateFrom, setDateFrom] = useState('')
  const [dateTo, setDateTo] = useState('')
  const [subTypeFilter, setSubTypeFilter] = useState<DocType | 'all'>('all')
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [documentToDelete, setDocumentToDelete] = useState<DocumentListItem | null>(null)
  const queryClient = useQueryClient()
  const { toast } = useToast()

  const { activeCategory, setCategory, isLoadingPref } = useDocumentCategory()

  // URL 有 type= 時沿用舊模式（向下相容）
  const isLegacyMode = Boolean(typeFilter)

  const buildQueryParams = () => {
    let params = ''
    if (isLegacyMode) {
      params += `doc_type=${typeFilter}&`
    } else if (activeCategory) {
      const types =
        subTypeFilter !== 'all' ? [subTypeFilter] : CATEGORY_TYPES[activeCategory]
      params += `doc_types=${types.join(',')}&`
    }
    if (statusFilter && statusFilter !== 'all') params += `status=${statusFilter}&`
    if (search) params += `keyword=${encodeURIComponent(search)}&`
    if (dateFrom) params += `date_from=${dateFrom}&`
    if (dateTo) params += `date_to=${dateTo}&`
    return params
  }

  const shouldFetch = isLegacyMode || Boolean(activeCategory)

  const { data: documents, isLoading } = useQuery({
    queryKey: ['documents', typeFilter, activeCategory, subTypeFilter, statusFilter, search, dateFrom, dateTo],
    staleTime: STALE_TIME.LIST,
    enabled: shouldFetch,
    queryFn: async () => {
      const response = await api.get<DocumentListItem[]>(`/documents?${buildQueryParams()}`)
      return response.data
    },
  })

  const deleteMutation = useMutation({
    mutationFn: async ({ id, hard }: { id: string; hard: boolean }) => {
      await deleteResource(`/documents/${id}${hard ? '?hard=true' : ''}`)
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['documents'] })
      toast({
        title: '成功',
        description: variables.hard ? '單據已永久刪除' : '單據已刪除',
      })

      setDeleteDialogOpen(false)
      setDocumentToDelete(null)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '刪除失敗'),
        variant: 'destructive',
      })
    },
  })

  const handleDeleteClick = (doc: DocumentListItem) => {
    setDocumentToDelete(doc)
    setDeleteDialogOpen(true)
  }

  const handleConfirmDelete = () => {
    if (documentToDelete) {
      const isAdmin = useAuthStore.getState().user?.roles.includes('admin')
      deleteMutation.mutate({ id: documentToDelete.id, hard: !!isAdmin })
    }
  }

  const handleCategoryChange = (cat: DocCategory) => {
    setCategory(cat === activeCategory ? null : cat)
    setSubTypeFilter('all')
  }


  const clearFilters = () => {
    setSearch('')
    setStatusFilter('all')
    setDateFrom('')
    setDateTo('')
    setSubTypeFilter('all')
  }

  const hasFilters = search || (statusFilter && statusFilter !== 'all') || dateFrom || dateTo || subTypeFilter !== 'all'

  const newDocHref = isLegacyMode
    ? `/documents/new?type=${typeFilter}`
    : subTypeFilter !== 'all'
    ? `/documents/new?type=${subTypeFilter}`
    : undefined

  const title = isLegacyMode
    ? TYPE_NAMES[typeFilter as DocType] || '單據管理'
    : '單據管理'

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">{title}</h1>
          <p className="text-muted-foreground">
            {isLegacyMode ? `管理${TYPE_NAMES[typeFilter as DocType]}` : '採購、銷貨、倉儲單據統一管理'}
          </p>
        </div>
        <Button asChild>
          <Link to={isLegacyMode ? `/documents/new?type=${typeFilter}` : (subTypeFilter !== 'all' ? `/documents/new?type=${subTypeFilter}` : '/documents/new')}>
            <Plus className="mr-2 h-4 w-4" />
            新增單據
          </Link>
        </Button>
      </div>

      {/* 類別 Tab（非舊模式才顯示） */}
      {!isLegacyMode && (
        <div className="flex gap-2 border-b border-slate-200">
          {(Object.keys(CATEGORY_CONFIG) as DocCategory[]).map((cat) => {
            const cfg = CATEGORY_CONFIG[cat]
            return (
              <button
                key={cat}
                onClick={() => handleCategoryChange(cat)}
                className={cn(
                  'flex items-center gap-1.5 px-4 py-2 border-b-2 font-medium text-sm transition-colors',
                  activeCategory === cat
                    ? 'border-blue-500 text-blue-600 -mb-px'
                    : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
                )}
              >
                {cfg.icon}
                {cfg.label}
              </button>
            )
          })}
        </div>
      )}

      {/* 空狀態 */}
      {!isLegacyMode && !activeCategory && !isLoadingPref && (
        <div className="flex flex-col items-center justify-center py-20 text-center">
          <FileText className="h-16 w-16 mb-4 text-muted-foreground opacity-40" />
          <p className="text-lg font-medium text-slate-600">請選擇上方的類別以開始查詢</p>
          <p className="text-sm text-muted-foreground mt-1">
            採購類包含採購單、採購入庫、採購退貨；銷貨類包含銷貨單、銷貨出庫
          </p>
        </div>
      )}

      {/* 篩選列 + 表格（有類別或舊模式才顯示） */}
      {(isLegacyMode || activeCategory) && (
        <>
          <div className="flex flex-wrap gap-4 items-end">
            <div className="relative flex-1 min-w-[200px] max-w-sm">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="搜尋單號..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                className="pl-9"
              />
            </div>

            {/* 舊模式：顯示全類型 dropdown；類別模式：顯示子類型 dropdown */}
            {isLegacyMode ? (
              <Select
                value={typeFilter || 'all'}
                onValueChange={(value) => {
                  if (value && value !== 'all') {
                    setSearchParams({ type: value })
                  } else {
                    setSearchParams({})
                  }
                }}
              >
                <SelectTrigger className="w-40">
                  <SelectValue placeholder="全部類型" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部類型</SelectItem>
                  {Object.entries(TYPE_NAMES).map(([key, name]) => (
                    <SelectItem key={key} value={key}>{name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : activeCategory ? (
              <Select value={subTypeFilter} onValueChange={(v) => setSubTypeFilter(v as DocType | 'all')}>
                <SelectTrigger className="w-40">
                  <SelectValue placeholder="全部子類型" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部子類型</SelectItem>
                  {CATEGORY_TYPES[activeCategory].map((t) => (
                    <SelectItem key={t} value={t}>{TYPE_NAMES[t]}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : null}

            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder="全部狀態" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部狀態</SelectItem>
                {Object.entries(STATUS_NAMES).map(([key, name]) => (
                  <SelectItem key={key} value={key}>{name}</SelectItem>
                ))}
              </SelectContent>
            </Select>

            <div className="flex items-center gap-2">
              <div className="space-y-1">
                <Label className="text-xs text-muted-foreground">起始日期</Label>
                <div className="relative">
                  <Calendar className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    type="date"
                    value={dateFrom}
                    onChange={(e) => setDateFrom(e.target.value)}
                    className="pl-9 w-[150px]"
                  />
                </div>
              </div>
              <span className="text-muted-foreground mt-5">~</span>
              <div className="space-y-1">
                <Label className="text-xs text-muted-foreground">結束日期</Label>
                <div className="relative">
                  <Calendar className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    type="date"
                    value={dateTo}
                    onChange={(e) => setDateTo(e.target.value)}
                    className="pl-9 w-[150px]"
                  />
                </div>
              </div>
            </div>

            {hasFilters && (
              <Button variant="ghost" size="sm" onClick={clearFilters} className="mt-5">
                <X className="h-4 w-4 mr-1" />
                清除篩選
              </Button>
            )}
          </div>

          <DocumentTable
            documents={documents}
            isLoading={isLoading}
            onDeleteClick={handleDeleteClick}
          />
        </>
      )}


      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {useAuthStore.getState().user?.roles.includes('admin') ? '管理員權限：永久刪除單據' : '確認刪除'}
            </DialogTitle>
            <DialogDescription>
              {useAuthStore.getState().user?.roles.includes('admin')
                ? `警告：具有管理員權限，刪除單據「${documentToDelete?.doc_no}」將永久從資料庫中移除資料（包含明細與庫存異動紀錄），此操作無法復原。確定要執行硬刪除嗎？`
                : `確定要刪除單據「${documentToDelete?.doc_no}」嗎？此操作無法復原。`}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteDialogOpen(false)}>
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={handleConfirmDelete}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {useAuthStore.getState().user?.roles.includes('admin') ? '執行硬刪除' : '確認刪除'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
