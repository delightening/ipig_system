import { useTranslation } from 'react-i18next'

import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { cn, formatUom } from '@/lib/utils'
import type { ProductEditFormReturn } from '../hooks/useProductEditForm'

// 只放單位代碼；顯示名稱在渲染時走 formatUom（i18n `uom.<code>`）
const PACKAGING_UNITS = {
  outer: ['CTN', 'BX', 'PK', 'CASE'],
  inner: ['BX', 'PK', 'EA', 'PC', 'PR', 'BT', 'RL', 'SET', 'TB', 'CP'],
  base: ['EA', 'PC', 'PR', 'BT', 'BX', 'PK', 'RL', 'SET', 'TB', 'CP'],
}

interface UnitChipRowProps {
  units: string[]
  selectedCode: string
  onSelect: (code: string) => void
}

function UnitChipRow({ units, selectedCode, onSelect }: UnitChipRowProps) {
  return (
    <div className="flex flex-wrap gap-2">
      {units.map((code) => (
        <button
          key={code}
          type="button"
          onClick={() => onSelect(code)}
          className={cn(
            'flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 transition-all',
            selectedCode === code
              ? 'border-primary bg-primary/10'
              : 'border-border hover:border-primary/50',
          )}
        >
          <span className="font-mono text-sm font-semibold">{code}</span>
          <span className="text-xs text-muted-foreground">{formatUom(code)}</span>
        </button>
      ))}
    </div>
  )
}

interface EditPackagingCardProps {
  formReturn: ProductEditFormReturn
}

export function EditPackagingCard({ formReturn }: EditPackagingCardProps) {
  const { t } = useTranslation()
  const { form, updateField } = formReturn

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('erpMaster.productEdit.packaging.title')}</CardTitle>
        <CardDescription>
          {t('erpMaster.productEdit.packaging.description')}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Layer count toggle */}
        <div className="space-y-3">
          <Label className="text-sm font-medium">{t('erpMaster.packaging.layerCount')}</Label>
          <div className="flex gap-3">
            <button
              type="button"
              onClick={() => updateField('packagingLayers', 2)}
              className={cn(
                'flex-1 p-3 rounded-lg border-2 transition-all text-left',
                form.packagingLayers === 2
                  ? 'border-primary bg-primary/10 text-primary font-medium'
                  : 'border-border hover:border-primary/50',
              )}
            >
              {t('erpMaster.packaging.twoLayer')}
              <span className="block text-xs mt-1 text-muted-foreground">
                {t('erpMaster.packaging.twoLayerDescription')}
              </span>
            </button>
            <button
              type="button"
              onClick={() => updateField('packagingLayers', 3)}
              className={cn(
                'flex-1 p-3 rounded-lg border-2 transition-all text-left',
                form.packagingLayers === 3
                  ? 'border-primary bg-primary/10 text-primary font-medium'
                  : 'border-border hover:border-primary/50',
              )}
            >
              {t('erpMaster.packaging.threeLayer')}
              <span className="block text-xs mt-1 text-muted-foreground">
                {t('erpMaster.packaging.threeLayerDescription')}
              </span>
            </button>
          </div>
        </div>

        <div className="space-y-4">
          {/* Outer layer */}
          <div className="space-y-2">
            <Label className="text-sm font-medium">{t('erpMaster.packaging.outerLayer')}</Label>
            <UnitChipRow
              units={PACKAGING_UNITS.outer}
              selectedCode={form.outerUnitCode}
              onSelect={(code) =>
                updateField(
                  'outerUnitCode',
                  form.outerUnitCode === code ? '' : code,
                )
              }
            />
            {form.outerUnitCode && (
              <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-muted border border-border w-fit">
                <span className="text-sm text-muted-foreground">1</span>
                <span className="text-sm">
                  {formatUom(form.outerUnitCode)}
                </span>
              </div>
            )}
          </div>

          {/* Inner layer */}
          <div className="space-y-2">
            <Label className="text-sm font-medium">
              {t('erpMaster.packaging.innerLayer')}
              {form.packagingLayers === 2 && (
                <span className="text-xs text-muted-foreground ml-2">{t('erpMaster.packaging.consumptionUnitNote')}</span>
              )}
            </Label>
            <UnitChipRow
              units={PACKAGING_UNITS.inner}
              selectedCode={form.innerUnitCode}
              onSelect={(code) => {
                updateField('innerUnitCode', code)
                if (form.packagingLayers === 2) updateField('baseUnitCode', code)
              }}
            />
            {form.innerUnitCode && (
              <div className="flex items-center gap-2 flex-wrap">
                <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-muted border border-border w-fit">
                  <span className="text-sm text-muted-foreground">
                    {form.outerUnitCode
                      ? t('erpMaster.packaging.oneUnit', { unit: formatUom(form.outerUnitCode) })
                      : t('erpMaster.packaging.one')}
                  </span>
                  <Input
                    type="number"
                    min={1}
                    className="w-16 h-8 text-center"
                    value={form.innerQty}
                    onChange={(e) =>
                      updateField(
                        'innerQty',
                        Math.max(1, parseInt(e.target.value, 10) || 1),
                      )
                    }
                  />
                  <span className="text-sm">
                    {formatUom(form.innerUnitCode)}
                  </span>
                </div>
              </div>
            )}
          </div>

          {/* Base unit (3-layer only) */}
          {form.packagingLayers === 3 && (
            <div className="space-y-2">
              <Label className="text-sm font-medium">
                {t('erpMaster.productEdit.packaging.baseUnitLabel')}
              </Label>
              <UnitChipRow
                units={PACKAGING_UNITS.base}
                selectedCode={form.baseUnitCode}
                onSelect={(code) => updateField('baseUnitCode', code)}
              />
              {form.innerUnitCode && form.baseUnitCode && (
                <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-muted border border-border w-fit">
                  <span className="text-sm text-muted-foreground">
                    {t('erpMaster.packaging.oneUnit', { unit: formatUom(form.innerUnitCode) })}
                  </span>
                  <Input
                    type="number"
                    min={1}
                    className="w-16 h-8 text-center"
                    value={form.baseQty}
                    onChange={(e) =>
                      updateField(
                        'baseQty',
                        Math.max(1, parseInt(e.target.value, 10) || 1),
                      )
                    }
                  />
                  <span className="text-sm">
                    {formatUom(form.baseUnitCode)}
                  </span>
                </div>
              )}
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
