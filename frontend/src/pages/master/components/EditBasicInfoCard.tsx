import { useTranslation } from 'react-i18next'

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
import { cn, formatUom } from '@/lib/utils'
import { STORAGE_CONDITIONS } from '@/lib/constants/product'
import { CATEGORY_ICONS } from '../constants'
import type { ProductEditFormReturn } from '../hooks/useProductEditForm'

interface EditBasicInfoCardProps {
  formReturn: ProductEditFormReturn
}

export function EditBasicInfoCard({ formReturn }: EditBasicInfoCardProps) {
  const { t } = useTranslation()
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
        <CardTitle>{t('erpMaster.productDetail.basicInfo')}</CardTitle>
        <CardDescription>{t('erpMaster.productEdit.basicInfoDescription')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-2">
          <Label htmlFor="name">{t('erpMaster.productEdit.productNameRequired')}</Label>
          <Input
            id="name"
            value={form.name}
            onChange={(e) => updateField('name', e.target.value)}
            placeholder={t('erpMaster.productEdit.namePlaceholder')}
            required
          />
        </div>
        <div className="grid gap-2">
          <Label htmlFor="spec">{t('erpMaster.productDetail.specDescription')}</Label>
          <Input
            id="spec"
            value={form.spec}
            onChange={(e) => updateField('spec', e.target.value)}
            placeholder={t('erpMaster.productEdit.specPlaceholder')}
          />
        </div>
        <div className="grid gap-2">
          <Label>{t('erpMaster.productEdit.categoryLabel')}</Label>
          {isDefaultCategory && (
            <p className="text-muted-foreground text-xs">
              {t('erpMaster.productEdit.defaultCategoryNote')}
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
            <Label>{t('erpMaster.createProduct.subcategory')}</Label>
            <Select
              value={form.subcategoryCode}
              onValueChange={(v) => updateField('subcategoryCode', v)}
            >
              <SelectTrigger>
                <SelectValue placeholder={t('erpMaster.createProduct.selectSubcategory')} />
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
          <Label>{t('erpMaster.productEdit.baseUomReadonly')}</Label>
          <Input
            value={`${product.base_uom} (${formatUom(product.base_uom)})`}
            disabled
            className="bg-muted"
          />
        </div>
        <div className="grid gap-2">
          <Label htmlFor="barcode">{t('erpMaster.productDetail.barcode')}</Label>
          <Input
            id="barcode"
            value={form.barcode}
            onChange={(e) => updateField('barcode', e.target.value)}
            placeholder={t('erpMaster.common.optional')}
          />
        </div>
        <div className="grid gap-2">
          <Label>{t('erpMaster.productDetail.storageCondition')}</Label>
          <Select
            value={form.storageCondition || '__none__'}
            onValueChange={(v) =>
              updateField('storageCondition', v === '__none__' ? '' : v)
            }
          >
            <SelectTrigger>
              <SelectValue placeholder={t('erpMaster.common.optional')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="__none__">{t('erpMaster.productEdit.notSet')}</SelectItem>
              {Object.entries(STORAGE_CONDITIONS).map(([code, labelKey]) => (
                <SelectItem key={code} value={code}>
                  {t(labelKey)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="grid gap-2">
          <Label htmlFor="licenseNo">{t('erpMaster.productDetail.licenseNo')}</Label>
          <Input
            id="licenseNo"
            value={form.licenseNo}
            onChange={(e) => updateField('licenseNo', e.target.value)}
            placeholder={t('erpMaster.common.optional')}
          />
        </div>
        <div className="grid gap-2">
          <Label htmlFor="tags">{t('erpMaster.productEdit.tagsLabel')}</Label>
          <Input
            id="tags"
            value={form.tagsInput}
            onChange={(e) => updateField('tagsInput', e.target.value)}
            placeholder={t('erpMaster.productEdit.tagsPlaceholder')}
          />
        </div>
        <div className="grid gap-2">
          <Label htmlFor="remark">{t('erpMaster.common.remark')}</Label>
          <Input
            id="remark"
            value={form.remark}
            onChange={(e) => updateField('remark', e.target.value)}
            placeholder={t('erpMaster.common.optional')}
          />
        </div>
      </CardContent>
    </Card>
  )
}
