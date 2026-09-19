import { useTranslation } from 'react-i18next'

import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import type { ProductEditFormReturn } from '../hooks/useProductEditForm'

interface EditTrackingCardProps {
  formReturn: ProductEditFormReturn
}

export function EditTrackingCard({ formReturn }: EditTrackingCardProps) {
  const { t } = useTranslation()
  const { form, updateField } = formReturn

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('erpMaster.productDetail.trackingSettings')}</CardTitle>
        <CardDescription>{t('erpMaster.productEdit.trackingDescription')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="trackBatch"
            checked={form.trackBatch}
            onChange={(e) => updateField('trackBatch', e.target.checked)}
            className="rounded"
          />
          <Label htmlFor="trackBatch">{t('erpMaster.products.trackBatch')}</Label>
        </div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="trackExpiry"
            checked={form.trackExpiry}
            onChange={(e) => updateField('trackExpiry', e.target.checked)}
            className="rounded"
          />
          <Label htmlFor="trackExpiry">{t('erpMaster.products.trackExpiry')}</Label>
        </div>
        {form.trackExpiry && (
          <div className="grid gap-2">
            <Label htmlFor="defaultExpiryDays">{t('erpMaster.productEdit.defaultExpiryDays')}</Label>
            <Input
              id="defaultExpiryDays"
              type="number"
              min={1}
              value={form.defaultExpiryDays}
              onChange={(e) =>
                updateField(
                  'defaultExpiryDays',
                  e.target.value === '' ? '' : parseInt(e.target.value, 10),
                )
              }
              placeholder={t('erpMaster.productEdit.expiryDaysPlaceholder')}
            />
          </div>
        )}
      </CardContent>
    </Card>
  )
}
