import { useParams } from 'react-router-dom'
import { ArrowLeft, Loader2, Package } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { useProductEditForm } from './hooks/useProductEditForm'
import { EditBasicInfoCard } from './components/EditBasicInfoCard'
import { EditPackagingCard } from './components/EditPackagingCard'
import { EditTrackingCard } from './components/EditTrackingCard'
import { EditInventoryCard } from './components/EditInventoryCard'

export function ProductEditPage() {
  const { id } = useParams<{ id: string }>()
  const formReturn = useProductEditForm(id)
  const { product, isLoading, error, updateMutation, handleSubmit, navigate } =
    formReturn

  if (isLoading) {
    return (
      <div className="flex justify-center items-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (error || !product) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] text-center">
        <Package className="h-12 w-12 text-muted-foreground mb-4" />
        <h2 className="text-xl font-semibold mb-2">找不到產品</h2>
        <p className="text-muted-foreground mb-4">
          該產品可能已被刪除或不存在
        </p>
        <Button variant="outline" onClick={() => navigate('/products')}>
          返回產品列表
        </Button>
      </div>
    )
  }

  const isDefaultCategory =
    (product.category_code || 'GEN') === 'GEN' &&
    (product.subcategory_code || 'OTH') === 'OTH'

  return (
    <div className="container max-w-3xl py-6">
      <div className="flex items-center gap-4 mb-6">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => navigate(`/products/${id}`)}
        >
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div>
          <h1 className="text-2xl font-bold tracking-tight">編輯產品</h1>
          <p className="text-muted-foreground text-sm">
            SKU: {product.sku}
            {isDefaultCategory
              ? '（目前為預設分類 GEN-OTH，變更分類後將自動產生新 SKU）'
              : '（唯讀，不可修改）'}
          </p>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <div className="grid gap-6">
          <EditBasicInfoCard formReturn={formReturn} />
          <EditPackagingCard formReturn={formReturn} />
          <EditTrackingCard formReturn={formReturn} />
          <EditInventoryCard formReturn={formReturn} />

          <div className="flex gap-2">
            <Button type="submit" disabled={updateMutation.isPending}>
              {updateMutation.isPending && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              儲存變更
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => navigate(`/products/${id}`)}
            >
              取消
            </Button>
          </div>
        </div>
      </form>
    </div>
  )
}
