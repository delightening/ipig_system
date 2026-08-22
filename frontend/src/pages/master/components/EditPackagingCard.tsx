import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { UOM_MAP } from '@/lib/utils'
import { cn } from '@/lib/utils'
import type { ProductEditFormReturn } from '../hooks/useProductEditForm'

const PACKAGING_UNITS = {
  outer: [
    { code: 'CTN', name: '箱' },
    { code: 'BX', name: '盒' },
    { code: 'PK', name: '包' },
    { code: 'CASE', name: '件' },
  ],
  inner: [
    { code: 'BX', name: '盒' },
    { code: 'PK', name: '包' },
    { code: 'EA', name: '個' },
    { code: 'PC', name: '支' },
    { code: 'PR', name: '雙' },
    { code: 'BT', name: '瓶' },
    { code: 'RL', name: '卷' },
    { code: 'SET', name: '組' },
    { code: 'TB', name: '錠' },
    { code: 'CP', name: '膠囊' },
  ],
  base: [
    { code: 'EA', name: '個' },
    { code: 'PC', name: '支' },
    { code: 'PR', name: '雙' },
    { code: 'BT', name: '瓶' },
    { code: 'BX', name: '盒' },
    { code: 'PK', name: '包' },
    { code: 'RL', name: '卷' },
    { code: 'SET', name: '組' },
    { code: 'TB', name: '錠' },
    { code: 'CP', name: '膠囊' },
  ],
}

interface UnitOption {
  code: string
  name: string
}

interface UnitChipRowProps {
  units: UnitOption[]
  selectedCode: string
  onSelect: (code: string) => void
}

function UnitChipRow({ units, selectedCode, onSelect }: UnitChipRowProps) {
  return (
    <div className="flex flex-wrap gap-2">
      {units.map((u) => (
        <button
          key={u.code}
          type="button"
          onClick={() => onSelect(u.code)}
          className={cn(
            'flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 transition-all',
            selectedCode === u.code
              ? 'border-primary bg-primary/10'
              : 'border-border hover:border-primary/50',
          )}
        >
          <span className="font-mono text-sm font-semibold">{u.code}</span>
          <span className="text-xs text-muted-foreground">{u.name}</span>
        </button>
      ))}
    </div>
  )
}

interface EditPackagingCardProps {
  formReturn: ProductEditFormReturn
}

export function EditPackagingCard({ formReturn }: EditPackagingCardProps) {
  const { form, updateField } = formReturn

  return (
    <Card>
      <CardHeader>
        <CardTitle>包裝結構</CardTitle>
        <CardDescription>
          外層→內層→基礎單位（消耗單位），編輯時可檢視與修改
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Layer count toggle */}
        <div className="space-y-3">
          <Label className="text-sm font-medium">包裝層數</Label>
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
              兩層包裝
              <span className="block text-xs mt-1 text-muted-foreground">
                外層 → 內層（消耗每內層）
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
              三層包裝
              <span className="block text-xs mt-1 text-muted-foreground">
                外層 → 內層 → 基礎（消耗每基礎）
              </span>
            </button>
          </div>
        </div>

        <div className="space-y-4">
          {/* Outer layer */}
          <div className="space-y-2">
            <Label className="text-sm font-medium">外層包裝</Label>
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
                  {UOM_MAP[form.outerUnitCode] || form.outerUnitCode}
                </span>
              </div>
            )}
          </div>

          {/* Inner layer */}
          <div className="space-y-2">
            <Label className="text-sm font-medium">
              內層包裝
              {form.packagingLayers === 2 && (
                <span className="text-xs text-muted-foreground ml-2">（消耗單位）</span>
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
                      ? `一${UOM_MAP[form.outerUnitCode] || form.outerUnitCode}`
                      : '一'}
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
                    {UOM_MAP[form.innerUnitCode] || form.innerUnitCode}
                  </span>
                </div>
              </div>
            )}
          </div>

          {/* Base unit (3-layer only) */}
          {form.packagingLayers === 3 && (
            <div className="space-y-2">
              <Label className="text-sm font-medium">
                基礎單位（消耗單位，庫存管理）
              </Label>
              <UnitChipRow
                units={PACKAGING_UNITS.base}
                selectedCode={form.baseUnitCode}
                onSelect={(code) => updateField('baseUnitCode', code)}
              />
              {form.innerUnitCode && form.baseUnitCode && (
                <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-muted border border-border w-fit">
                  <span className="text-sm text-muted-foreground">
                    一{UOM_MAP[form.innerUnitCode] || form.innerUnitCode}
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
                    {UOM_MAP[form.baseUnitCode] || form.baseUnitCode}
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
