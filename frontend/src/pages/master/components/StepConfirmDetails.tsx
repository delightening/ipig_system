import { useMemo } from 'react'
import { ArrowLeft, Loader2, Check } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent } from '@/components/ui/card'
import { cn } from '@/lib/utils'
import { UNITS } from '../constants'
import type { ProductFormData } from '../constants'
import { UnitButtonGroup } from './UnitButtonGroup'
import { BasicInfoCard } from './BasicInfoCard'
import { InventorySettingsCard } from './InventorySettingsCard'
import type { CreateProductFormReturn } from '../hooks/useCreateProductForm'

interface StepConfirmDetailsProps {
  form: CreateProductFormReturn
}

export function StepConfirmDetails({ form }: StepConfirmDetailsProps) {
  const {
    formData, setFormData, isCreated, isCreating, skuStatus,
    isOuterCustom, setIsOuterCustom, customOuter, setCustomOuter,
    isInnerCustom, setIsInnerCustom, customInner, setCustomInner,
    isBaseCustom, setIsBaseCustom, customBase, setCustomBase,
    handleBack, handleNext,
  } = form

  // Build base unit list for three-layer mode
  const baseDisplayUnits = useMemo(() => {
    const selectedUnits = [formData.outerUnit, formData.innerUnit]
      .filter(Boolean)
      .map(u => {
        const found = [...UNITS.outer, ...UNITS.inner, ...UNITS.base].find(
          unit => unit.name === u || unit.code === u
        )
        return found ? { code: found.code, name: found.name } : null
      })
      .filter(Boolean) as Array<{ code: string; name: string }>

    const uniqueSelected = Array.from(new Map(selectedUnits.map(u => [u.code, u])).values())
    const otherBase = UNITS.base.filter(u => !uniqueSelected.some(su => su.code === u.code))
    return { units: [...uniqueSelected, ...otherBase], highlightedCodes: uniqueSelected.map(u => u.code) }
  }, [formData.outerUnit, formData.innerUnit])

  return (
    <div className="space-y-6 animate-slide-in-right">
      <BasicInfoCard form={form} />

      {/* Packaging Units */}
      <Card>
        <CardContent className="pt-6">
          <h3 className="text-lg font-semibold mb-4">包裝單位</h3>
          <div className="space-y-6">
            <PackagingLayerToggle formData={formData} setFormData={setFormData} disabled={isCreated} />

            {/* Outer unit */}
            <OuterUnitSection
              formData={formData} setFormData={setFormData} disabled={isCreated}
              isCustom={isOuterCustom} setIsCustom={setIsOuterCustom}
              customValue={customOuter} setCustomValue={setCustomOuter}
            />

            {/* Inner unit */}
            <InnerUnitSection
              formData={formData} setFormData={setFormData} disabled={isCreated}
              isCustom={isInnerCustom} setIsCustom={setIsInnerCustom}
              customValue={customInner} setCustomValue={setCustomInner}
            />

            {/* Base unit (three-layer only) */}
            {formData.packagingLayers === 3 && (
              <BaseUnitSection
                formData={formData} setFormData={setFormData} disabled={isCreated}
                isCustom={isBaseCustom} setIsCustom={setIsBaseCustom}
                customValue={customBase} setCustomValue={setCustomBase}
                displayUnits={baseDisplayUnits.units}
                highlightedCodes={baseDisplayUnits.highlightedCodes}
              />
            )}
          </div>
        </CardContent>
      </Card>

      <InventorySettingsCard formData={formData} setFormData={setFormData} disabled={isCreated} />

      {/* Action Buttons */}
      <div className="flex justify-between">
        <Button variant="outline" onClick={handleBack} disabled={isCreating}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          上一步
        </Button>
        <Button
          onClick={handleNext}
          disabled={!formData.baseUnit || isCreating || skuStatus !== 'S3'}
          size="lg"
        >
          {isCreating ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              建立中...
            </>
          ) : (
            <>
              建立產品
              <Check className="ml-2 h-4 w-4" />
            </>
          )}
        </Button>
      </div>
    </div>
  )
}

// --- Sub-components ---

function PackagingLayerToggle({
  formData, setFormData, disabled,
}: {
  formData: ProductFormData
  setFormData: React.Dispatch<React.SetStateAction<ProductFormData>>
  disabled: boolean
}) {
  return (
    <div className="space-y-3">
      <Label className="text-sm font-medium">包裝層數</Label>
      <div className="flex gap-3">
        <button
          type="button"
          onClick={() => setFormData(prev => ({
            ...prev, packagingLayers: 2, outerUnit: '',
            baseUnit: prev.innerUnit || prev.baseUnit, baseQty: 1,
          }))}
          disabled={disabled}
          className={cn(
            "flex-1 p-3 rounded-lg border-2 transition-all",
            formData.packagingLayers === 2
              ? "border-primary bg-primary/10 text-primary font-medium"
              : "border-slate-200 dark:border-slate-700 hover:border-primary/50"
          )}
        >
          兩層包裝
          <span className="block text-xs mt-1 text-slate-500">外層 → 內層（消耗每內層）</span>
        </button>
        <button
          type="button"
          onClick={() => setFormData(prev => ({ ...prev, packagingLayers: 3 }))}
          disabled={disabled}
          className={cn(
            "flex-1 p-3 rounded-lg border-2 transition-all",
            formData.packagingLayers === 3
              ? "border-primary bg-primary/10 text-primary font-medium"
              : "border-slate-200 dark:border-slate-700 hover:border-primary/50"
          )}
        >
          三層包裝
          <span className="block text-xs mt-1 text-slate-500">外層 → 內層 → 基礎（消耗每基礎）</span>
        </button>
      </div>
    </div>
  )
}

interface UnitSectionProps {
  formData: ProductFormData
  setFormData: React.Dispatch<React.SetStateAction<ProductFormData>>
  disabled: boolean
  isCustom: boolean
  setIsCustom: (v: boolean) => void
  customValue: string
  setCustomValue: (v: string) => void
}

function OuterUnitSection({ formData, setFormData, disabled, isCustom, setIsCustom, customValue, setCustomValue }: UnitSectionProps) {
  return (
    <div className="space-y-3">
      <Label className="text-sm font-medium">外層包裝</Label>
      <UnitButtonGroup
        units={UNITS.outer}
        selectedUnit={formData.outerUnit}
        onSelect={(unit) => {
          setIsCustom(false)
          setFormData(prev => ({ ...prev, outerUnit: prev.outerUnit === unit.name ? '' : unit.name }))
        }}
        isCustom={isCustom}
        onCustomToggle={() => {
          setIsCustom(true)
          setFormData(prev => ({ ...prev, outerUnit: customValue }))
        }}
        customValue={customValue}
        onCustomChange={(v) => {
          setCustomValue(v)
          setFormData(prev => ({ ...prev, outerUnit: v }))
        }}
        onCustomClear={() => {
          setIsCustom(false)
          setCustomValue('')
          if (formData.outerUnit === customValue) {
            setFormData(prev => ({ ...prev, outerUnit: '' }))
          }
        }}
        disabled={disabled}
      />
      {formData.outerUnit && (
        <UnitDisplay label="1" unit={formData.outerUnit} />
      )}
    </div>
  )
}

function InnerUnitSection({ formData, setFormData, disabled, isCustom, setIsCustom, customValue, setCustomValue }: UnitSectionProps) {
  const isTwoLayer = formData.packagingLayers === 2

  const applyInnerUnit = (unitName: string) => {
    const updates: Partial<ProductFormData> = { innerUnit: unitName }
    if (isTwoLayer) {
      updates.baseUnit = unitName
      updates.baseQty = 1
    }
    return updates
  }

  return (
    <div className="space-y-3">
      <Label className="text-sm font-medium">
        內層包裝
        {isTwoLayer && <span className="text-xs text-slate-400 ml-2">（消耗單位）</span>}
      </Label>
      <UnitButtonGroup
        units={UNITS.inner}
        selectedUnit={formData.innerUnit}
        onSelect={(unit) => {
          setIsCustom(false)
          setFormData(prev => ({ ...prev, ...applyInnerUnit(unit.name) }))
        }}
        isCustom={isCustom}
        onCustomToggle={() => {
          setIsCustom(true)
          setFormData(prev => ({ ...prev, ...applyInnerUnit(customValue) }))
        }}
        customValue={customValue}
        onCustomChange={(v) => {
          setCustomValue(v)
          setFormData(prev => ({ ...prev, ...applyInnerUnit(v) }))
        }}
        disabled={disabled}
      />
      {formData.innerUnit && (
        <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 whitespace-nowrap ml-auto">
          <span className="text-sm text-slate-500">
            {formData.outerUnit ? `一${formData.outerUnit}` : '一'}
          </span>
          <Input
            type="number" min={1}
            value={formData.innerQty}
            onChange={(e) => setFormData(prev => ({ ...prev, innerQty: parseInt(e.target.value) || 1 }))}
            className="w-16 h-8 text-center"
            disabled={disabled}
          />
          <span className="text-sm text-slate-500">{formData.innerUnit}</span>
        </div>
      )}
    </div>
  )
}

function BaseUnitSection({
  formData, setFormData, disabled,
  isCustom, setIsCustom, customValue, setCustomValue,
  displayUnits, highlightedCodes,
}: UnitSectionProps & {
  displayUnits: Array<{ code: string; name: string }>
  highlightedCodes: string[]
}) {
  const applyBaseUnit = (unitName: string) => ({
    baseUnit: unitName,
    safetyStockUnit: unitName,
    currentStockUnit: unitName,
    reorderPointUnit: unitName,
  })

  return (
    <div className="space-y-3">
      <Label className="text-sm font-medium">
        基礎單位（消耗單位）
        <span className="text-xs text-slate-400 ml-2">(庫存管理)</span>
      </Label>
      <UnitButtonGroup
        units={displayUnits}
        selectedUnit={formData.baseUnit}
        onSelect={(unit) => {
          setIsCustom(false)
          setFormData(prev => ({ ...prev, ...applyBaseUnit(unit.name) }))
        }}
        isCustom={isCustom}
        onCustomToggle={() => {
          setIsCustom(true)
          setFormData(prev => ({ ...prev, ...applyBaseUnit(customValue) }))
        }}
        customValue={customValue}
        onCustomChange={(v) => {
          setCustomValue(v)
          setFormData(prev => ({ ...prev, ...applyBaseUnit(v) }))
        }}
        disabled={disabled}
        highlightedCodes={highlightedCodes}
      />
      {formData.innerUnit && formData.baseUnit && (
        <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 whitespace-nowrap ml-auto">
          <span className="text-sm text-slate-500">一{formData.innerUnit}</span>
          <Input
            type="number" min={1}
            value={formData.baseQty}
            onChange={(e) => setFormData(prev => ({ ...prev, baseQty: parseInt(e.target.value) || 1 }))}
            className="w-16 h-8 text-center"
            disabled={disabled}
          />
          <span className="text-sm text-slate-500">{formData.baseUnit}</span>
        </div>
      )}
    </div>
  )
}

function UnitDisplay({ label, unit }: { label: string; unit: string }) {
  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-slate-50 dark:bg-slate-800/50 border border-slate-200 dark:border-slate-700 whitespace-nowrap ml-auto">
      <span className="text-sm text-slate-500">{label}</span>
      <span className="text-sm text-slate-700 dark:text-slate-200">{unit}</span>
    </div>
  )
}
