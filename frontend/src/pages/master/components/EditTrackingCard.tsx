import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import type { ProductEditFormReturn } from '../hooks/useProductEditForm'

interface EditTrackingCardProps {
  formReturn: ProductEditFormReturn
}

export function EditTrackingCard({ formReturn }: EditTrackingCardProps) {
  const { form, updateField } = formReturn

  return (
    <Card>
      <CardHeader>
        <CardTitle>追蹤設定</CardTitle>
        <CardDescription>批號、效期追蹤</CardDescription>
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
          <Label htmlFor="trackBatch">追蹤批號</Label>
        </div>
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="trackExpiry"
            checked={form.trackExpiry}
            onChange={(e) => updateField('trackExpiry', e.target.checked)}
            className="rounded"
          />
          <Label htmlFor="trackExpiry">追蹤效期</Label>
        </div>
        {form.trackExpiry && (
          <div className="grid gap-2">
            <Label htmlFor="defaultExpiryDays">預設有效天數</Label>
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
              placeholder="例：365"
            />
          </div>
        )}
      </CardContent>
    </Card>
  )
}
