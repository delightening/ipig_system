import { useState, useMemo } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import { useToggle } from '@/hooks/useToggle'
import { useSelection } from '@/hooks/useSelection'
import { useProductListState } from './hooks/useProductListState'
import { useDialogSet } from '@/hooks/useDialogSet'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { toast } from '@/components/ui/use-toast'
import { Plus, Upload, Download, FolderEdit } from 'lucide-react'
import { getApiErrorMessage } from '@/lib/validation'
import { useSkuCategories } from '@/hooks/useSkuCategories'
import { useAuthStore } from '@/stores/auth'
import type { CategoryOption } from './hooks/useProductListState'
import type { PaginatedResponse } from '@/types/common'

import type { ExtendedProduct, StatusAction } from './components/productTypes'
import { ProductFilterPanel } from './components/ProductFilterPanel'
import { ProductTable } from './components/ProductTable'
import { ProductBatchActions, exportProductsCsv } from './components/ProductBatchActions'
import { ProductPagination } from './components/ProductPagination'
import {
  StatusChangeDialog,
  BatchStatusDialog,
  HardDeleteDialog,
} from './components/ProductDialogs'
import { ProductImportDialog } from '@/components/product/ProductImportDialog'
import { EditCategoriesDialog } from '@/components/product/EditCategoriesDialog'

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
  const selection = useSelection<string>()
  const dialogs = useDialogSet(['status', 'batchStatus', 'import', 'editCategories', 'hardDelete'] as const)
  const [statusAction, setStatusAction] = useState<StatusAction>('activate')
  const [targetProduct, setTargetProduct] = useState<ExtendedProduct | null>(null)
  const [hardDeleteProduct, setHardDeleteProduct] = useState<ExtendedProduct | null>(null)
  const isAdmin = hasRole('admin') || hasRole('SYSTEM_ADMIN')

  // 查詢產品列表
  const { data: response, isLoading, isFetching } = useQuery({
    queryKey: ['products', listState.queryParams],
    queryFn: async () => {
      const res = await api.get<ExtendedProduct[] | PaginatedResponse<ExtendedProduct>>(`/products?${listState.queryParams}`)
      if (Array.isArray(res.data)) {
        return { data: res.data, total: res.data.length, page: 1, per_page: res.data.length, total_pages: 1 }
      }
      return res.data
    },
    enabled: !!user,
  })

  const products = response?.data || []
  const totalItems = response?.total || 0
  const totalPages = response?.total_pages || 1

  // Mutations
  const statusMutation = useMutation({
    mutationFn: async ({ id, status }: { id: string; status: string }) => api.patch(`/products/${id}/status`, { status }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] })
      toast({ title: '成功', description: '產品狀態已更新' })
      dialogs.close('status')
      setTargetProduct(null)
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(error, '狀態更新失敗'), variant: 'destructive' })
    },
  })

  const hardDeleteMutation = useMutation({
    mutationFn: async (id: string) => api.post(`/products/${id}/hard-delete`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] })
      toast({ title: '成功', description: '產品已永久刪除' })
      dialogs.close('hardDelete')
      setHardDeleteProduct(null)
    },
    onError: (error: unknown) => {
      toast({ title: '硬刪除失敗', description: getApiErrorMessage(error, '無法硬刪除產品'), variant: 'destructive' })
    },
  })

  const batchStatusMutation = useMutation({
    mutationFn: async ({ ids, status }: { ids: string[]; status: string }) =>
      Promise.all(ids.map(id => api.patch(`/products/${id}/status`, { status }))),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] })
      toast({ title: '成功', description: `已更新 ${selection.size} 個產品的狀態` })
      dialogs.close('batchStatus')
      selection.clear()
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(error, '批次更新失敗'), variant: 'destructive' })
    },
  })

  const handleStatusChange = (product: ExtendedProduct, action: StatusAction) => {
    setTargetProduct(product)
    setStatusAction(action)
    dialogs.open('status')
  }

  const handleExportCSV = () => {
    if (products.length === 0) {
      toast({ title: '無資料可匯出', description: '請先篩選或新增產品', variant: 'destructive' })
      return
    }
    exportProductsCsv(products, 'products')
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="產品管理"
        description="管理系統中的產品/品項資料"
        actions={
          <>
            <Button variant="outline" size="sm" onClick={() => dialogs.open('editCategories')}>
              <FolderEdit className="mr-2 h-4 w-4" />
              編輯分類
            </Button>
            <Button variant="outline" size="sm" onClick={() => dialogs.open('import')}>
              <Upload className="mr-2 h-4 w-4" />
              匯入
            </Button>
            <Button variant="outline" size="sm" onClick={handleExportCSV} disabled={products.length === 0}>
              <Download className="mr-2 h-4 w-4" />
              匯出
            </Button>
            <Button onClick={() => navigate('/products/new')}>
              <Plus className="mr-2 h-4 w-4" />
              新增產品
            </Button>
          </>
        }
      />

      {/* Search & Filters */}
      <ProductFilterPanel
        listState={listState}
        categoriesForFilter={categoriesForFilter}
        showAdvancedFilters={showAdvancedFilters}
        onToggleAdvancedFilters={toggleAdvancedFilters}
      />

      {/* Batch Actions */}
      <ProductBatchActions
        selectionSize={selection.size}
        selectedIds={selection.selectedIds}
        products={products}
        onBatchDeactivate={() => {
          setStatusAction('deactivate')
          dialogs.open('batchStatus')
        }}
        onClearSelection={() => selection.clear()}
      />

      {/* Table */}
      <ProductTable
        products={products}
        isLoading={isLoading}
        listState={listState}
        selectionHas={selection.has}
        selectionSize={selection.size}
        onSelectAll={() => selection.selectAll(products.map(p => p.id))}
        onSelect={(id) => selection.toggle(id)}
        onStatusChange={handleStatusChange}
        onHardDelete={(product) => {
          setHardDeleteProduct(product)
          dialogs.open('hardDelete')
        }}
        isAdmin={isAdmin}
      />

      {/* Pagination */}
      {products.length > 0 && (
        <ProductPagination
          listState={listState}
          totalItems={totalItems}
          totalPages={totalPages}
          isFetching={isFetching}
          isLoading={isLoading}
        />
      )}

      {/* Dialogs */}
      <StatusChangeDialog
        open={dialogs.isOpen('status')}
        onOpenChange={dialogs.setOpen('status')}
        product={targetProduct}
        action={statusAction}
        isPending={statusMutation.isPending}
        onConfirm={() => {
          if (!targetProduct) return
          const status = statusAction === 'activate' ? 'active'
            : statusAction === 'deactivate' ? 'inactive'
              : 'discontinued'
          statusMutation.mutate({ id: targetProduct.id, status })
        }}
        onClose={() => dialogs.close('status')}
      />

      <BatchStatusDialog
        open={dialogs.isOpen('batchStatus')}
        onOpenChange={dialogs.setOpen('batchStatus')}
        selectionSize={selection.size}
        isPending={batchStatusMutation.isPending}
        onConfirm={() => {
          batchStatusMutation.mutate({
            ids: Array.from(selection.selectedIds),
            status: 'inactive',
          })
        }}
        onClose={() => dialogs.close('batchStatus')}
      />

      <HardDeleteDialog
        open={dialogs.isOpen('hardDelete')}
        onOpenChange={dialogs.setOpen('hardDelete')}
        product={hardDeleteProduct}
        isPending={hardDeleteMutation.isPending}
        onConfirm={() => {
          if (!hardDeleteProduct) return
          hardDeleteMutation.mutate(hardDeleteProduct.id)
        }}
        onClose={() => {
          dialogs.close('hardDelete')
          setHardDeleteProduct(null)
        }}
      />

      <ProductImportDialog open={dialogs.isOpen('import')} onOpenChange={dialogs.setOpen('import')} />
      <EditCategoriesDialog open={dialogs.isOpen('editCategories')} onOpenChange={dialogs.setOpen('editCategories')} />
    </div>
  )
}
