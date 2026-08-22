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
import type { ProductEditFormReturn } from '../hooks/useProductEditForm'

const UOM_OPTIONS = Object.entries(UOM_MAP).map(([code, name]) => ({ code, name }))

interface EditInventoryCardProps {
  formReturn: ProductEditFormReturn
}

export function EditInventoryCard({ formReturn }: EditInventoryCardProps) {
  const { form, updateField } = formReturn

  return (
    <Card>
      <CardHeader>
        <CardTitle>庫存管理設定</CardTitle>
        <CardDescription>安全庫存與補貨點</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div className="grid gap-2">
            <Label htmlFor="safetyStock">安全庫存</Label>
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
              placeholder="選填"
            />
          </div>
          <div className="grid gap-2">
            <Label>安全庫存單位</Label>
            <Select
              value={form.safetyStockUom}
              onValueChange={(v) => updateField('safetyStockUom', v)}
            >
              <SelectTrigger>
                <SelectValue placeholder="選擇單位" />
              </SelectTrigger>
              <SelectContent>
                {UOM_OPTIONS.map((u) => (
                  <SelectItem key={u.code} value={u.code}>
                    {u.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
        <div className="grid grid-cols-2 gap-4">
          <div className="grid gap-2">
            <Label htmlFor="reorderPoint">補貨點</Label>
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
              placeholder="選填"
            />
          </div>
          <div className="grid gap-2">
            <Label>補貨點單位</Label>
            <Select
              value={form.reorderPointUom}
              onValueChange={(v) => updateField('reorderPointUom', v)}
            >
              <SelectTrigger>
                <SelectValue placeholder="選擇單位" />
              </SelectTrigger>
              <SelectContent>
                {UOM_OPTIONS.map((u) => (
                  <SelectItem key={u.code} value={u.code}>
                    {u.name}
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
