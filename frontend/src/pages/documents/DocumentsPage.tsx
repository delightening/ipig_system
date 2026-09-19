import { useState } from 'react'
import { useSearchParams, Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, { deleteResource } from '@/lib/api'
import type { DocType, DocumentListItem } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { GuestHide } from '@/components/ui/guest-hide'
import { PageHeader } from '@/components/ui/page-header'
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
import { getApiErrorMessage } from '@/lib/apiError'
import { useAuthIsAdmin } from '@/stores/auth'
import { cn } from '@/lib/utils'
import { DocumentTable } from './components/DocumentTable'
import { useDocumentCategory, type DocCategory } from './hooks/useDocumentCategory'
import { DOC_STATUS_NAMES, DOC_TYPE_NAMES } from './types'


// R84-13：sales 分類原含 'SR'/'RTN'（銷貨退貨），業務上不存在銷貨退貨，已從 DocType 移除。
const CATEGORY_TYPES: Record<DocCategory, DocType[]> = {
  purchasing: ['PO', 'GRN', 'PR'],
  sales: ['SO'],
  warehouse: ['TR', 'STK', 'ADJ'],
}

// label / desc 存 i18n 鍵，渲染時才 t()（語言切換才會即時更新）
const CATEGORY_CONFIG: Record<DocCategory, { labelKey: string; icon: React.ReactNode; descKey: string }> = {
  purchasing: {
    labelKey: 'erpDocs.documents.list.category.purchasing.label',
    icon: <Truck className="h-4 w-4" />,
    descKey: 'erpDocs.documents.list.category.purchasing.desc',
  },
  sales: {
    labelKey: 'erpDocs.documents.list.category.sales.label',
    icon: <ShoppingCart className="h-4 w-4" />,
    descKey: 'erpDocs.documents.list.category.sales.desc',
  },
  warehouse: {
    labelKey: 'erpDocs.documents.list.category.warehouse.label',
    icon: <Warehouse className="h-4 w-4" />,
    descKey: 'erpDocs.documents.list.category.warehouse.desc',
  },
}

export function DocumentsPage() {
  const { t } = useTranslation()
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
  const isAdmin = useAuthIsAdmin()

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
        title: t('common.success'),
        description: variables.hard
          ? t('erpDocs.documents.list.toast.hardDeleted')
          : t('erpDocs.documents.list.toast.deleted'),
      })

      setDeleteDialogOpen(false)
      setDocumentToDelete(null)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpDocs.shared.deleteFailed')),
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
      deleteMutation.mutate({ id: documentToDelete.id, hard: isAdmin })
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

  const title = isLegacyMode
    ? DOC_TYPE_NAMES[typeFilter as DocType] || t('nav.erpDocuments')
    : t('nav.erpDocuments')

  return (
    <div className="space-y-6">
      <PageHeader
        title={title}
        description={
          isLegacyMode
            ? t('erpDocs.documents.list.manageType', { type: DOC_TYPE_NAMES[typeFilter as DocType] ?? typeFilter })
            : t('erpDocs.documents.list.description')
        }
        actions={
          <GuestHide>
            <Button size="sm" asChild>
              <Link to={isLegacyMode ? `/documents/new?type=${typeFilter}` : (subTypeFilter !== 'all' ? `/documents/new?type=${subTypeFilter}` : '/documents/new')}>
                <Plus className="mr-2 h-4 w-4" />
                {t('erpDocs.documents.newDocument')}
              </Link>
            </Button>
          </GuestHide>
        }
      />

      {/* 類別 Tab（非舊模式才顯示） */}
      {!isLegacyMode && (
        <div className="flex gap-2 border-b border-border">
          {(Object.keys(CATEGORY_CONFIG) as DocCategory[]).map((cat) => {
            const cfg = CATEGORY_CONFIG[cat]
            return (
              <button
                key={cat}
                onClick={() => handleCategoryChange(cat)}
                className={cn(
                  'flex items-center gap-1.5 px-4 py-2 border-b-2 font-medium text-sm transition-colors',
                  activeCategory === cat
                    ? 'border-primary text-primary -mb-px'
                    : 'border-transparent text-muted-foreground hover:text-foreground hover:border-border'
                )}
              >
                {cfg.icon}
                {t(cfg.labelKey)}
              </button>
            )
          })}
        </div>
      )}

      {/* 空狀態 */}
      {!isLegacyMode && !activeCategory && !isLoadingPref && (
        <div className="flex flex-col items-center justify-center py-20 text-center">
          <FileText className="h-16 w-16 mb-4 text-muted-foreground opacity-40" />
          <p className="text-lg font-medium text-muted-foreground">{t('erpDocs.documents.list.emptyPrompt')}</p>
          <p className="text-sm text-muted-foreground mt-1">
            {t('erpDocs.documents.list.emptyHint')}
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
                placeholder={t('erpDocs.documents.list.searchPlaceholder')}
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
                  <SelectValue placeholder={t('erpDocs.documents.list.allTypes')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('erpDocs.documents.list.allTypes')}</SelectItem>
                  {Object.entries(DOC_TYPE_NAMES).map(([key, name]) => (
                    <SelectItem key={key} value={key}>{name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : activeCategory ? (
              <Select value={subTypeFilter} onValueChange={(v) => setSubTypeFilter(v as DocType | 'all')}>
                <SelectTrigger className="w-40">
                  <SelectValue placeholder={t('erpDocs.documents.list.allSubTypes')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('erpDocs.documents.list.allSubTypes')}</SelectItem>
                  {CATEGORY_TYPES[activeCategory].map((docType) => (
                    <SelectItem key={docType} value={docType}>{DOC_TYPE_NAMES[docType]}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : null}

            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger className="w-40">
                <SelectValue placeholder={t('common.allStatus')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">{t('common.allStatus')}</SelectItem>
                {Object.entries(DOC_STATUS_NAMES).map(([key, name]) => (
                  <SelectItem key={key} value={key}>{name}</SelectItem>
                ))}
              </SelectContent>
            </Select>

            <div className="flex items-center gap-2">
              <div className="space-y-1">
                <Label className="text-xs text-muted-foreground">{t('erpDocs.documents.list.dateFrom')}</Label>
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
                <Label className="text-xs text-muted-foreground">{t('erpDocs.documents.list.dateTo')}</Label>
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
                {t('common.clearFilters')}
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
              {isAdmin ? t('erpDocs.documents.list.deleteDialog.adminTitle') : t('common.confirmDelete')}
            </DialogTitle>
            <DialogDescription>
              {isAdmin
                ? t('erpDocs.documents.list.deleteDialog.adminDescription', { docNo: documentToDelete?.doc_no ?? '' })
                : t('erpDocs.documents.list.deleteDialog.description', { docNo: documentToDelete?.doc_no ?? '' })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteDialogOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              variant="destructive"
              onClick={handleConfirmDelete}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {isAdmin ? t('erpDocs.documents.list.deleteDialog.hardDelete') : t('common.confirmDelete')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
