import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { UOM_MAP } from '@/lib/utils'
import { cn } from '@/lib/utils'
import { STORAGE_CONDITIONS } from '@/lib/constants/product'
import { CATEGORY_ICONS } from '../constants'
import type { ProductEditFormReturn } from '../hooks/useProductEditForm'

interface EditBasicInfoCardProps {
  formReturn: ProductEditFormReturn
}

export function EditBasicInfoCard({ formReturn }: EditBasicInfoCardProps) {
  const {
    form,
    updateField,
    product,
    displayCategories,
    subcategories,
    skuCategoriesLoading,
    hasSubcategories,
  } = formReturn

  if (!product) return null

  const isDefaultCategory =
    (product.category_code || 'GEN') === 'GEN' &&
    (product.subcategory_code || 'OTH') === 'OTH'

  return (
    <Card>
      <CardHeader>
        <CardTitle>基本資訊</CardTitle>
        <CardDescription>產品名稱、規格、分類等</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-2">
          <Label htmlFor="name">產品名稱 *</Label>
          <Input
            id="name"
            value={form.name}
            onChange={(e) => updateField('name', e.target.value)}
            placeholder="例：紗布"
            required
          />
        </div>
        <div className="grid gap-2">
          <Label htmlFor="spec">規格描述</Label>
          <Input
            id="spec"
            value={form.spec}
            onChange={(e) => updateField('spec', e.target.value)}
            placeholder="例：4x4"
          />
        </div>
        <div className="grid gap-2">
          <Label>分類（與新增產品一致）</Label>
          {isDefaultCategory && (
            <p className="text-muted-foreground text-xs">
              此產品為匯入預設 GEN-OTH，選擇新分類並儲存後將自動產生新 SKU。
            </p>
          )}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
            {displayCategories.slice(0, 4).map((cat) => (
              <button
                key={cat.code}
                type="button"
                onClick={() => {
                  updateField('categoryCode', cat.code)
                  updateField(
                    'subcategoryCode',
                    hasSubcategories(cat.code) ? '' : cat.code,
                  )
                }}
                disabled={skuCategoriesLoading}
                className={cn(
                  'flex items-center gap-3 p-3 rounded-lg border-2 text-left transition-all',
                  form.categoryCode === cat.code
                    ? 'border-primary bg-primary/5'
                    : 'border-border hover:border-primary/50',
                )}
              >
                <div
                  className={cn(
                    'p-2 rounded-md',
                    form.categoryCode === cat.code
                      ? 'bg-primary/10 text-primary'
                      : 'bg-muted',
                  )}
                >
                  {CATEGORY_ICONS[cat.code]}
                </div>
                <span className="font-medium">{cat.name}</span>
              </button>
            ))}
          </div>
        </div>
        {subcategories.length > 0 && (
          <div className="grid gap-2">
            <Label>子分類</Label>
            <Select
              value={form.subcategoryCode}
              onValueChange={(v) => updateField('subcategoryCode', v)}
            >
              <SelectTrigger>
                <SelectValue placeholder="選擇子分類" />
              </SelectTrigger>
              <SelectContent>
                {subcategories.map((s) => (
                  <SelectItem key={s.code} value={s.code}>
                    {s.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}
        <div className="grid gap-2">
          <Label>庫存單位（消耗單位，唯讀）</Label>
          <Input
            value={`${product.base_uom} (${UOM_MAP[product.base_uom] || product.base_uom})`}
            disabled
            className="bg-muted"
          />
        </div>
        <div className="grid gap-2">
          <Label htmlFor="barcode">原廠條碼</Label>
          <Input
            id="barcode"
            value={form.barcode}
            onChange={(e) => updateField('barcode', e.target.value)}
            placeholder="選填"
          />
        </div>
        <div className="grid gap-2">
          <Label>保存條件</Label>
          <Select
            value={form.storageCondition || '__none__'}
            onValueChange={(v) =>
              updateField('storageCondition', v === '__none__' ? '' : v)
            }
          >
            <SelectTrigger>
              <SelectValue placeholder="選填" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="__none__">不設定</SelectItem>
              {Object.entries(STORAGE_CONDITIONS).map(([code, label]) => (
                <SelectItem key={code} value={code}>
                  {label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="grid gap-2">
          <Label htmlFor="licenseNo">許可證號</Label>
          <Input
            id="licenseNo"
            value={form.licenseNo}
            onChange={(e) => updateField('licenseNo', e.target.value)}
            placeholder="選填"
          />
        </div>
        <div className="grid gap-2">
          <Label htmlFor="tags">搜尋標籤（逗號分隔）</Label>
          <Input
            id="tags"
            value={form.tagsInput}
            onChange={(e) => updateField('tagsInput', e.target.value)}
            placeholder="例：敷料, 急救"
          />
        </div>
        <div className="grid gap-2">
          <Label htmlFor="remark">備註</Label>
          <Input
            id="remark"
            value={form.remark}
            onChange={(e) => updateField('remark', e.target.value)}
            placeholder="選填"
          />
        </div>
      </CardContent>
    </Card>
  )
}
