import { Check, ListPlus, FileText, LayoutGrid } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { formatUom } from '@/lib/utils'
import { UNITS } from '../constants'
import type { CreateProductFormReturn } from '../hooks/useCreateProductForm'

interface StepSuccessProps {
  form: CreateProductFormReturn
}

export function StepSuccess({ form }: StepSuccessProps) {
  const { t } = useTranslation()

  return (
    <div className="animate-fade-in">
      <Card className="overflow-hidden">
        <div className="bg-gradient-to-r from-success/10 via-success/5 to-transparent p-8 text-center">
          <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-success/20 flex items-center justify-center animate-success-bounce">
            <Check className="w-8 h-8 text-success" />
          </div>
          <h2 className="text-2xl font-bold text-foreground mb-2">
            {t('erpMaster.createProduct.created')}
          </h2>
          <p className="text-muted-foreground">
            {form.formData.name} {form.formData.spec}
          </p>
        </div>

        <CardContent className="pt-6">
          <div className="space-y-4">
            <div className="p-4 rounded-lg bg-muted border border-border">
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm text-muted-foreground">SKU</span>
                <span className="font-mono font-bold text-lg text-primary">{form.finalSku}</span>
              </div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm text-muted-foreground">{t('erpMaster.createProduct.category')}</span>
                <span>{form.skuCategories.find(c => c.code === form.formData.category)?.name ?? '—'}</span>
              </div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm text-muted-foreground">{t('erpMaster.common.unit')}</span>
                <span>{UNITS.base.some(u => u.code === form.formData.baseUnit) ? formatUom(form.formData.baseUnit) : '—'}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-muted-foreground">{t('erpMaster.createProduct.track')}</span>
                <span>
                  {form.formData.trackBatch && t('erpMaster.products.table.batchNo')} {form.formData.trackBatch && form.formData.trackExpiry && '/'} {form.formData.trackExpiry && t('erpMaster.products.table.expiry')}
                  {!form.formData.trackBatch && !form.formData.trackExpiry && t('erpMaster.createProduct.trackNone')}
                </span>
              </div>
            </div>

            <p className="text-sm text-muted-foreground">
              {t('erpMaster.createProduct.nextSteps')}
            </p>

            <div className="grid grid-cols-3 gap-3">
              <Button variant="outline" className="flex-col h-auto py-4" onClick={form.handleReset}>
                <ListPlus className="h-5 w-5 mb-1" />
                <span className="text-xs">{t('erpMaster.createProduct.addAnother')}</span>
              </Button>
              <Button variant="outline" className="flex-col h-auto py-4" onClick={() => form.navigate('/documents?type=PO')}>
                <FileText className="h-5 w-5 mb-1" />
                <span className="text-xs">{t('erpMaster.createProduct.createPurchaseOrder')}</span>
              </Button>
              <Button variant="outline" className="flex-col h-auto py-4" onClick={() => form.navigate('/products')}>
                <LayoutGrid className="h-5 w-5 mb-1" />
                <span className="text-xs">{t('erpMaster.createProduct.productList')}</span>
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
