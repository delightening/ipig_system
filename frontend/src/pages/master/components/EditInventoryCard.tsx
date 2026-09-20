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
import { UOM_MAP, formatUom } from '@/lib/utils'
import type { ProductEditFormReturn } from '../hooks/useProductEditForm'

// 單位代碼清單（顯示名稱在渲染時走 formatUom，語系切換後才會更新）
const UOM_CODES = Object.keys(UOM_MAP)

interface EditInventoryCardProps {
  formReturn: ProductEditFormReturn
}

export function EditInventoryCard({ formReturn }: EditInventoryCardProps) {
  const { t } = useTranslation()
  const { form, updateField } = formReturn

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('erpMaster.productDetail.inventoryManagement')}</CardTitle>
        <CardDescription>{t('erpMaster.productEdit.inventoryDescription')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div className="grid gap-2">
            <Label htmlFor="safetyStock">{t('erpMaster.products.safetyStock')}</Label>
            <Input
              id="safetyStock"
              type="number"
              min={0}
              value={form.safetyStock}
              onChange={(e) =>
                updateField(
                  'safetyStock',
                  e.target.value === '' ? '' : parseFloat(e.target.value),
                )
              }
              placeholder={t('erpMaster.common.optional')}
            />
          </div>
          <div className="grid gap-2">
            <Label>{t('erpMaster.productEdit.safetyStockUnit')}</Label>
            <Select
              value={form.safetyStockUom}
              onValueChange={(v) => updateField('safetyStockUom', v)}
            >
              <SelectTrigger>
                <SelectValue placeholder={t('erpMaster.packaging.selectUnit')} />
              </SelectTrigger>
              <SelectContent>
                {UOM_CODES.map((code) => (
                  <SelectItem key={code} value={code}>
                    {formatUom(code)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
        <div className="grid grid-cols-2 gap-4">
          <div className="grid gap-2">
            <Label htmlFor="reorderPoint">{t('erpMaster.productDetail.reorderPoint')}</Label>
            <Input
              id="reorderPoint"
              type="number"
              min={0}
              value={form.reorderPoint}
              onChange={(e) =>
                updateField(
                  'reorderPoint',
                  e.target.value === '' ? '' : parseFloat(e.target.value),
                )
              }
              placeholder={t('erpMaster.common.optional')}
            />
          </div>
          <div className="grid gap-2">
            <Label>{t('erpMaster.productEdit.reorderPointUnit')}</Label>
            <Select
              value={form.reorderPointUom}
              onValueChange={(v) => updateField('reorderPointUom', v)}
            >
              <SelectTrigger>
                <SelectValue placeholder={t('erpMaster.packaging.selectUnit')} />
              </SelectTrigger>
              <SelectContent>
                {UOM_CODES.map((code) => (
                  <SelectItem key={code} value={code}>
                    {formatUom(code)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
