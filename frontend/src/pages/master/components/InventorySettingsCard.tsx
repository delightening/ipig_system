import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent } from '@/components/ui/card'
import { PackagingUnitSelect } from './PackagingUnitSelect'
import type { ProductFormData } from '../constants'

interface InventorySettingsCardProps {
  formData: ProductFormData
  setFormData: React.Dispatch<React.SetStateAction<ProductFormData>>
  disabled: boolean
}

export function InventorySettingsCard({ formData, setFormData, disabled }: InventorySettingsCardProps) {
  const includeBaseInSelect = formData.packagingLayers === 3

  return (
    <Card>
      <CardContent className="pt-6">
        <h3 className="text-lg font-semibold mb-4">庫存設定</h3>
        <div className="space-y-6">
          {/* Tracking toggles */}
          <div className="flex gap-6">
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={formData.trackBatch}
                onChange={(e) => setFormData(prev => ({ ...prev, trackBatch: e.target.checked }))}
                disabled={disabled}
                className="rounded"
              />
              <span className="text-sm">追蹤批號</span>
            </label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={formData.trackExpiry}
                onChange={(e) => setFormData(prev => ({ ...prev, trackExpiry: e.target.checked }))}
                disabled={disabled}
                className="rounded"
              />
              <span className="text-sm">追蹤效期</span>
            </label>
          </div>
          {formData.category === 'DRG' && (
            <p className="text-xs text-status-warning-text dark:text-amber-400">
              藥品建議開啟批號和效期追蹤
            </p>
          )}

          {/* Current Stock */}
          <div className="space-y-2">
            <Label>當前值（單位）</Label>
            <div className="flex items-center gap-3">
              <Input
                type="number"
                min={0}
                value={formData.currentStock}
                onChange={(e) => setFormData(prev => ({ ...prev, currentStock: parseFloat(e.target.value) || 0 }))}
                className="w-32"
                disabled={disabled}
                placeholder="0"
              />
              <PackagingUnitSelect
                value={formData.currentStockUnit}
                onChange={(v) => setFormData(prev => ({ ...prev, currentStockUnit: v }))}
                formData={formData}
                disabled={disabled}
                includeBase={includeBaseInSelect}
                className="w-32"
              />
            </div>
          </div>

          {/* Safety Stock */}
          <div className="space-y-2">
            <Label>安全庫存</Label>
            <div className="flex items-center gap-3">
              <Input
                type="number"
                min={0}
                value={formData.safetyStock}
                onChange={(e) => setFormData(prev => ({ ...prev, safetyStock: parseFloat(e.target.value) || 0 }))}
                className="w-32"
                disabled={disabled}
                placeholder="0"
              />
              <PackagingUnitSelect
                value={formData.safetyStockUnit}
                onChange={(v) => setFormData(prev => ({ ...prev, safetyStockUnit: v }))}
                formData={formData}
                disabled={disabled}
                includeBase={includeBaseInSelect}
                className="w-32"
              />
            </div>
          </div>

          {/* Reorder Point */}
          <div className="space-y-2">
            <Label>補貨提醒點</Label>
            <div className="flex items-center gap-3 flex-wrap">
              <span className="text-sm text-muted-foreground">當庫存低於</span>
              <Input
                type="number"
                min={0}
                value={formData.reorderPoint}
                onChange={(e) => setFormData(prev => ({ ...prev, reorderPoint: parseFloat(e.target.value) || 0 }))}
                className="w-24"
                disabled={disabled}
              />
              <PackagingUnitSelect
                value={formData.reorderPointUnit}
                onChange={(v) => setFormData(prev => ({ ...prev, reorderPointUnit: v }))}
                formData={formData}
                disabled={disabled}
                includeBase
                className="w-40"
              />
              <span className="text-sm text-muted-foreground">時，發送補貨提醒</span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
