import { useState } from 'react'
import { Can } from '@/components/auth'
import { useAuthHasPermission } from '@/stores/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, { Product, DocumentListItem } from '@/lib/api'
import { DOC_TYPE_NAMES } from '@/pages/documents/types'
import { PendingOwnerBadge } from '@/components/PendingOwnerBadge'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { StatusBadge } from '@/components/ui/status-badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import {
  ArrowLeft,
  Edit,
  MoreHorizontal,
  Loader2,
  Package,
  ClipboardCopy,
  Boxes,
  Calendar,
  User,
  FileText,
  History,
} from 'lucide-react'
import { formatDate, formatDateTime, formatNumber, formatUom } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/apiError'
import { EmptyState } from '@/components/ui/empty-state'
import { useSkuCategories } from '@/hooks/useSkuCategories'

import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import { ProductInventorySnapshot } from './components/ProductInventorySnapshot'
import { STORAGE_CONDITIONS } from '@/lib/constants/product'



interface ExtendedProduct extends Product {
  category_code?: string
  subcategory_code?: string
  category_name?: string
  subcategory_name?: string
  status?: 'active' | 'inactive' | 'discontinued'
  storage_condition?: string
  barcode?: string
  license_no?: string
  default_expiry_days?: number
  tags?: string[]
  remark?: string
  pack_qty?: number
  image_url?: string
  created_by_name?: string
}

const DOC_STATUS_CONFIG: Record<string, { labelKey: string; variant: 'neutral' | 'warning' | 'success' | 'error' }> = {
  draft: { labelKey: 'erpMaster.productDetail.docStatus.draft', variant: 'neutral' },
  submitted: { labelKey: 'erpMaster.productDetail.docStatus.submitted', variant: 'warning' },
  approved: { labelKey: 'erpMaster.productDetail.docStatus.approved', variant: 'success' },
  cancelled: { labelKey: 'erpMaster.productDetail.docStatus.cancelled', variant: 'error' },
}

export function ProductDetailPage() {
  const { t } = useTranslation()
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { categories: skuCategories, subcategoriesByCategory } = useSkuCategories()
  // 「更多操作」是原生 <select>，選項不能包成 <Can>（非法子節點），故用 hook 逐項判斷
  const hasPermission = useAuthHasPermission()
  const canCreateProduct = hasPermission(PERMISSIONS.ERP_PRODUCT_CREATE)
  const canEditProduct = hasPermission(PERMISSIONS.ERP_PRODUCT_EDIT)
  const [statusDialogOpen, setStatusDialogOpen] = useState(false)
  const [statusAction, setStatusAction] = useState<'activate' | 'deactivate' | 'discontinue'>('activate')

  const { data: product, isLoading, error } = useQuery({
    queryKey: ['product', id],
    queryFn: async () => {
      const response = await api.get<ExtendedProduct>(`/products/${id}`)
      return response.data
    },
    enabled: !!id,
  })

  // 相關單據：含此產品明細的採購單 / 採購入庫 / 銷貨單 / 調整單等（依 M-4 建立者 scope 由後端過濾）
  const { data: relatedDocs, isLoading: docsLoading, isError: docsError } = useQuery({
    queryKey: ['product-documents', id],
    queryFn: async () => {
      const response = await api.get<DocumentListItem[]>(`/documents?product_id=${id}`)
      return response.data
    },
    enabled: !!id,
  })

  const statusMutation = useMutation({
    mutationFn: async (status: string) => {
      return api.patch(`/products/${id}/status`, { status })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['product', id] })
      queryClient.invalidateQueries({ queryKey: ['products'] })
      toast({ title: t('common.success'), description: t('erpMaster.products.toast.statusUpdated') })
      setStatusDialogOpen(false)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpMaster.products.toast.statusUpdateFailed')),
        variant: 'destructive',
      })
    },
  })

  const handleCopySku = async () => {
    if (product) {
      await navigator.clipboard.writeText(product.sku)
      toast({ title: t('erpMaster.common.copied'), description: `SKU: ${product.sku}` })
    }
  }

  const getStatusBadge = () => {
    if (!product) return null
    const status = product.status || (product.is_active ? 'active' : 'inactive')
    switch (status) {
      case 'active':
        return <Badge variant="success" className="text-sm">● {t('erpMaster.common.active')}</Badge>
      case 'inactive':
        return <Badge variant="warning" className="text-sm">● {t('erpMaster.common.inactive')}</Badge>
      case 'discontinued':
        return <Badge variant="destructive" className="text-sm">● {t('erpMaster.products.status.discontinued')}</Badge>
      default:
        return <Badge variant="secondary" className="text-sm">● {t('erpMaster.common.unknown')}</Badge>
    }
  }

  const getCategoryName = () => {
    if (!product) return '-'
    if (product.category_name) return product.category_name
    const categoryCode = product.category_code === 'LAB' ? 'CON' : product.category_code
    if (categoryCode) return skuCategories.find((c) => c.code === categoryCode)?.name ?? product.category_code ?? '-'
    return '-'
  }

  const getSubcategoryName = () => {
    if (!product) return '-'
    if (product.subcategory_name) return product.subcategory_name
    const categoryCode = product.category_code === 'LAB' ? 'CON' : product.category_code
    if (categoryCode && product.subcategory_code) {
      const subs = subcategoriesByCategory[categoryCode] ?? []
      return subs.find((s) => s.code === product.subcategory_code)?.name ?? product.subcategory_code ?? '-'
    }
    return '-'
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (error || !product) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] text-center">
        <Package className="h-12 w-12 text-muted-foreground mb-4" />
        <h2 className="text-xl font-semibold mb-2">{t('erpMaster.productDetail.notFoundTitle')}</h2>
        <p className="text-muted-foreground mb-4">{t('erpMaster.productDetail.notFoundDescription')}</p>
        <Button variant="outline" onClick={() => navigate('/products')}>
          {t('erpMaster.productDetail.backToList')}
        </Button>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div className="flex items-start gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate('/products')}
            aria-label={t('erpMaster.common.back')}
          >
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div className="flex items-start gap-6">
            {/* Product Image Placeholder */}
            <div className="w-24 h-24 rounded-lg bg-muted flex items-center justify-center shrink-0">
              {product.image_url ? (
                <img
                  src={product.image_url}
                  alt={product.name}
                  className="w-full h-full object-cover rounded-lg"
                />
              ) : (
                <Package className="h-10 w-10 text-muted-foreground" />
              )}
            </div>
            <div>
              <h1 className="text-2xl font-bold tracking-tight mb-1">{product.name}</h1>
              <div className="flex items-center gap-2 mb-2">
                <code className="font-mono text-sm bg-muted px-2 py-0.5 rounded">
                  {product.sku}
                </code>
                <button
                  onClick={handleCopySku}
                  className="text-muted-foreground hover:text-foreground transition-colors"
                  title={t('erpMaster.productDetail.copySku')}
                >
                  <ClipboardCopy className="h-4 w-4" />
                </button>
              </div>
              <div className="flex items-center gap-3 text-sm text-muted-foreground">
                <span>{t('erpMaster.productDetail.categoryLabel', { name: getCategoryName() })}</span>
                <span>{'>'}</span>
                <span>{getSubcategoryName()}</span>
                <span className="mx-2">│</span>
                {getStatusBadge()}
              </div>
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Can permission={PERMISSIONS.ERP_PRODUCT_EDIT}>
            <Button variant="outline" onClick={() => navigate(`/products/${id}/edit`)}>
              <Edit className="mr-2 h-4 w-4" />
              {t('common.edit')}
            </Button>
          </Can>
          {(canCreateProduct || canEditProduct) && (
          <div className="relative">
            <select
              className="appearance-none bg-background border rounded-md px-3 py-2 pr-8 cursor-pointer hover:bg-muted focus:outline-hidden focus:ring-2 focus:ring-ring text-sm"
              aria-label={t('erpMaster.productDetail.moreActions')}
              onChange={(e) => {
                const action = e.target.value
                e.target.value = ''
                switch (action) {
                  case 'copy':
                    navigate(`/products/new?copy=${id}`)
                    break
                  case 'activate':
                    setStatusAction('activate')
                    setStatusDialogOpen(true)
                    break
                  case 'deactivate':
                    setStatusAction('deactivate')
                    setStatusDialogOpen(true)
                    break
                  case 'discontinue':
                    setStatusAction('discontinue')
                    setStatusDialogOpen(true)
                    break
                }
              }}
              defaultValue=""
            >
              <option value="" disabled>{t('erpMaster.productDetail.moreActionsPlaceholder')}</option>
              {canCreateProduct && <option value="copy">{t('erpMaster.productDetail.copyProduct')}</option>}
              {canCreateProduct && canEditProduct && <option disabled>───</option>}
              {canEditProduct && (product.is_active ? (
                <option value="deactivate">{t('erpMaster.products.actions.deactivate')}</option>
              ) : (
                <option value="activate">{t('erpMaster.products.actions.activate')}</option>
              ))}
              {canEditProduct && <option value="discontinue">{t('erpMaster.products.statusDialog.discontinueTitle')}</option>}
            </select>
            <MoreHorizontal className="absolute right-2 top-1/2 -translate-y-1/2 h-4 w-4 pointer-events-none text-muted-foreground" />
          </div>
          )}
        </div>
      </div>

      {/* Tabs */}
      <PageTabs
        tabs={[
          { value: 'basic', label: t('erpMaster.productDetail.basicInfo'), icon: Package },
          { value: 'inventory', label: t('erpMaster.productDetail.tabs.inventory'), icon: Boxes },
          { value: 'documents', label: t('erpMaster.productDetail.relatedDocuments'), icon: FileText },
          { value: 'history', label: t('erpMaster.productDetail.changeHistory'), icon: History },
        ]}
        defaultTab="basic"
      >
        <PageTabContent value="basic">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card>
            <CardHeader>
              <CardTitle>{t('erpMaster.productDetail.basicInfo')}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <InfoRow label={t('erpMaster.productDetail.productName')} value={product.name} />
              <InfoRow label={t('erpMaster.productDetail.specDescription')} value={product.spec || '-'} />
              <InfoRow label={t('erpMaster.products.category')} value={getCategoryName()} />
              <InfoRow label={t('erpMaster.products.subcategory')} value={getSubcategoryName()} />
              <InfoRow
                label={t('erpMaster.productDetail.baseUom')}
                value={`${product.base_uom} (${formatUom(product.base_uom)})`}
              />
              <InfoRow label={t('erpMaster.productDetail.packQty')} value={product.pack_qty?.toString() || '-'} />
              <InfoRow label={t('erpMaster.productDetail.barcode')} value={product.barcode || '-'} />
              <InfoRow
                label={t('erpMaster.productDetail.storageCondition')}
                value={product.storage_condition
                  ? (STORAGE_CONDITIONS[product.storage_condition]
                    ? t(STORAGE_CONDITIONS[product.storage_condition])
                    : product.storage_condition)
                  : '-'}
              />
              <InfoRow label={t('erpMaster.productDetail.licenseNo')} value={product.license_no || '-'} />
              {product.tags && product.tags.length > 0 && (
                <div className="flex justify-between py-2 border-b">
                  <span className="text-muted-foreground">{t('erpMaster.productDetail.tags')}</span>
                  <div className="flex gap-1 flex-wrap justify-end">
                    {product.tags.map((tag) => (
                      <Badge key={tag} variant="secondary" className="text-xs">
                        {tag}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
              <InfoRow label={t('erpMaster.common.remark')} value={product.remark || '-'} multiline />
            </CardContent>
          </Card>

          <div className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>{t('erpMaster.productDetail.trackingSettings')}</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <InfoRow
                  label={t('erpMaster.products.trackBatch')}
                  value={product.track_batch ? t('common.yes') : t('common.no')}
                  badge={product.track_batch}
                />
                <InfoRow
                  label={t('erpMaster.products.trackExpiry')}
                  value={product.track_expiry
                    ? t('erpMaster.productDetail.trackExpiryYes', { days: product.default_expiry_days || '-' })
                    : t('common.no')
                  }
                  badge={product.track_expiry}
                />
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>{t('erpMaster.productDetail.systemInfo')}</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center gap-2 py-2 border-b">
                  <Calendar className="h-4 w-4 text-muted-foreground" />
                  <span className="text-muted-foreground">{t('erpMaster.productDetail.createdAt')}</span>
                  <span className="ml-auto">{formatDateTime(product.created_at)}</span>
                </div>
                <div className="flex items-center gap-2 py-2 border-b">
                  <User className="h-4 w-4 text-muted-foreground" />
                  <span className="text-muted-foreground">{t('erpMaster.productDetail.createdBy')}</span>
                  <span className="ml-auto">{product.created_by_name || '-'}</span>
                </div>
                <div className="flex items-center gap-2 py-2 border-b">
                  <Calendar className="h-4 w-4 text-muted-foreground" />
                  <span className="text-muted-foreground">{t('erpMaster.productDetail.updatedAt')}</span>
                  <span className="ml-auto">{formatDateTime(product.updated_at)}</span>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
        </PageTabContent>

        <PageTabContent value="inventory">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card>
            <CardHeader>
              <CardTitle>{t('erpMaster.productDetail.inventoryManagement')}</CardTitle>
              <CardDescription>{t('erpMaster.productDetail.inventoryManagementDescription')}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <InfoRow
                label={t('erpMaster.products.safetyStock')}
                value={product.safety_stock
                  ? `${formatNumber(product.safety_stock, 0)} ${formatUom(product.base_uom)}`
                  : t('erpMaster.productDetail.notSet')
                }
              />
              <InfoRow
                label={t('erpMaster.productDetail.reorderPoint')}
                value={product.reorder_point
                  ? `${formatNumber(product.reorder_point, 0)} ${formatUom(product.base_uom)}`
                  : t('erpMaster.productDetail.notSet')
                }
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>{t('erpMaster.productDetail.snapshot.title')}</CardTitle>
              <CardDescription>{t('erpMaster.productDetail.snapshot.description')}</CardDescription>
            </CardHeader>
            <CardContent>
              <ProductInventorySnapshot
                productId={id ?? ''}
                uomLabel={formatUom(product.base_uom)}
              />
            </CardContent>
          </Card>
        </div>
        </PageTabContent>

        <PageTabContent value="documents">
        <Card>
          <CardHeader>
            <CardTitle>{t('erpMaster.productDetail.relatedDocuments')}</CardTitle>
            <CardDescription>{t('erpMaster.productDetail.relatedDocumentsDescription')}</CardDescription>
          </CardHeader>
          <CardContent>
            {docsLoading ? (
              <div className="flex justify-center py-12">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : docsError ? (
              <div className="py-12 text-center text-sm text-destructive">
                {t('erpMaster.productDetail.docsLoadFailed')}
              </div>
            ) : !relatedDocs || relatedDocs.length === 0 ? (
              <EmptyState icon={FileText} title={t('erpMaster.productDetail.docsEmpty')} />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t('erpMaster.productDetail.docType')}</TableHead>
                    <TableHead>{t('erpMaster.productDetail.docNo')}</TableHead>
                    <TableHead>{t('erpMaster.common.status')}</TableHead>
                    <TableHead>{t('erpMaster.productDetail.docDate')}</TableHead>
                    <TableHead>{t('erpMaster.productDetail.partner')}</TableHead>
                    <TableHead className="text-right">{t('erpMaster.productDetail.lineCount')}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {relatedDocs.map((doc) => (
                    <TableRow key={doc.id}>
                      <TableCell>{DOC_TYPE_NAMES[doc.doc_type] ?? doc.doc_type}</TableCell>
                      <TableCell>
                        <Link
                          to={`/documents/${doc.id}`}
                          className="font-medium text-primary hover:underline"
                        >
                          {doc.doc_no}
                        </Link>
                      </TableCell>
                      <TableCell>
                        <PendingOwnerBadge owner={doc.pending_owner}>
                          <StatusBadge
                            variant={DOC_STATUS_CONFIG[doc.status]?.variant ?? 'neutral'}
                            tone="soft"
                          >
                            {DOC_STATUS_CONFIG[doc.status] ? t(DOC_STATUS_CONFIG[doc.status].labelKey) : doc.status}
                          </StatusBadge>
                        </PendingOwnerBadge>
                      </TableCell>
                      <TableCell>{formatDate(doc.doc_date)}</TableCell>
                      <TableCell>{doc.partner_name ?? '-'}</TableCell>
                      <TableCell className="text-right">{doc.line_count}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>
        </PageTabContent>

        <PageTabContent value="history">
        <Card>
          <CardHeader>
            <CardTitle>{t('erpMaster.productDetail.changeHistory')}</CardTitle>
            <CardDescription>{t('erpMaster.productDetail.changeHistoryDescription')}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-center py-12 text-muted-foreground">
              <History className="h-10 w-10 mx-auto mb-2 opacity-50" />
              <p>{t('erpMaster.productDetail.changeHistoryEmpty')}</p>
            </div>
          </CardContent>
        </Card>
        </PageTabContent>
      </PageTabs>

      {/* 狀態變更對話框 */}
      <Dialog open={statusDialogOpen} onOpenChange={setStatusDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {statusAction === 'activate' && t('erpMaster.products.statusDialog.activateTitle')}
              {statusAction === 'deactivate' && t('erpMaster.products.statusDialog.deactivateTitle')}
              {statusAction === 'discontinue' && t('erpMaster.products.statusDialog.discontinueTitle')}
            </DialogTitle>
            <DialogDescription>
              {statusAction === 'activate' && t('erpMaster.products.statusDialog.activateDescription')}
              {statusAction === 'deactivate' && t('erpMaster.products.statusDialog.deactivateDescription')}
              {statusAction === 'discontinue' && t('erpMaster.products.statusDialog.discontinueDescription')}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setStatusDialogOpen(false)}
              disabled={statusMutation.isPending}
            >
              {t('common.cancel')}
            </Button>
            <Button
              variant={statusAction === 'discontinue' ? 'destructive' : 'default'}
              onClick={() => {
                const status = statusAction === 'activate' ? 'active'
                  : statusAction === 'deactivate' ? 'inactive'
                    : 'discontinued'
                statusMutation.mutate(status)
              }}
              disabled={statusMutation.isPending}
            >
              {statusMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t('common.confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

function InfoRow({
  label,
  value,
  badge,
  multiline,
}: {
  label: string
  value: string
  badge?: boolean
  multiline?: boolean
}) {
  return (
    <div className={`flex ${multiline ? 'flex-col gap-1' : 'justify-between'} py-2 border-b`}>
      <span className="text-muted-foreground">{label}</span>
      <span className={multiline ? 'text-sm' : ''}>
        {badge !== undefined ? (
          <Badge variant={badge ? 'success' : 'secondary'} className="text-xs">
            {value}
          </Badge>
        ) : (
          value
        )}
      </span>
    </div>
  )
}
