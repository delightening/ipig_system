import { Package } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'
import { CATEGORY_ICONS } from '../constants'
import type { CreateProductFormReturn } from '../hooks/useCreateProductForm'

interface BasicInfoCardProps {
  form: CreateProductFormReturn
}

export function BasicInfoCard({ form }: BasicInfoCardProps) {
  const { t } = useTranslation()
  const { formData, setFormData, isCreated, displayCategories, getSubcategories, skuCategoriesLoading } = form

  return (
    <Card>
      <CardContent className="pt-6">
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Package className="h-5 w-5 text-primary" />
          {t('erpMaster.productDetail.basicInfo')}
        </h3>
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>{t('erpMaster.productDetail.productName')}</Label>
              <Input
                value={formData.name}
                onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                placeholder={t('erpMaster.productDetail.productName')}
                disabled={isCreated}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('erpMaster.productDetail.specDescription')}</Label>
              <Input
                value={formData.spec}
                onChange={(e) => setFormData(prev => ({ ...prev, spec: e.target.value }))}
                placeholder={t('erpMaster.common.spec')}
                disabled={isCreated}
              />
            </div>
          </div>

          {/* Category buttons */}
          <div className="space-y-2">
            <Label>{t('erpMaster.createProduct.categoryRecommended')}</Label>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {displayCategories.slice(0, 4).map((cat) => (
                <button
                  key={cat.code}
                  type="button"
                  onClick={() => setFormData(prev => ({ ...prev, category: cat.code, subcategory: '' }))}
                  disabled={isCreated || skuCategoriesLoading}
                  className={cn(
                    "flex items-center gap-3 p-3 rounded-lg border-2 text-left transition-all",
                    formData.category === cat.code
                      ? "border-primary bg-primary/5"
                      : "border-border hover:border-primary/50"
                  )}
                >
                  <div className={cn(
                    "p-2 rounded-md",
                    formData.category === cat.code
                      ? "bg-primary/10 text-primary"
                      : "bg-muted"
                  )}>
                    {CATEGORY_ICONS[cat.code]}
                  </div>
                  <span className="font-medium">{cat.name}</span>
                  {cat.code === 'DRG' && formData.name?.toLowerCase().match(/cillin|mycin|oxacin/) && (
                    <span className="ml-auto text-xs bg-success/10 text-success px-2 py-0.5 rounded-full">
                      {t('erpMaster.createProduct.recommended')}
                    </span>
                  )}
                </button>
              ))}
            </div>
          </div>

          {/* Subcategory dropdown */}
          {formData.category && (() => {
            const subs = getSubcategories(formData.category)
            if (subs.length === 0) return null
            return (
              <div className="space-y-2">
                <Label>{t('erpMaster.createProduct.subcategory')}</Label>
                <Select
                  value={formData.subcategory}
                  onValueChange={(v) => setFormData(prev => ({ ...prev, subcategory: v }))}
                  disabled={isCreated || skuCategoriesLoading}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('erpMaster.createProduct.selectSubcategory')} />
                  </SelectTrigger>
                  <SelectContent>
                    {subs.map((sub) => (
                      <SelectItem key={sub.code} value={sub.code}>
                        {sub.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )
          })()}
        </div>
      </CardContent>
    </Card>
  )
}
