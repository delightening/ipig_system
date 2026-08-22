import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

import type { ExternalPiData } from './externalPi'

export function ExternalPiFields({
  value,
  onChange,
}: {
  value: ExternalPiData
  onChange: (v: ExternalPiData) => void
}) {
  const set = (patch: Partial<ExternalPiData>) => onChange({ ...value, ...patch })
  return (
    <div className="grid gap-3 rounded-md border border-dashed p-3">
      <div className="grid gap-2">
        <Label>PI 姓名 *</Label>
        <Input value={value.piName} onChange={(e) => set({ piName: e.target.value })} placeholder="外部計畫主持人姓名" />
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div className="grid gap-2 min-w-0">
          <Label>PI Email *</Label>
          <Input type="email" value={value.piEmail} onChange={(e) => set({ piEmail: e.target.value })} placeholder="例：pi@example.com" />
        </div>
        <div className="grid gap-2 min-w-0">
          <Label>PI 電話 *</Label>
          <Input type="tel" value={value.piPhone} onChange={(e) => set({ piPhone: e.target.value })} placeholder="例：02-1234-5678" />
        </div>
      </div>
      <p className="text-xs text-muted-foreground">委託單位請於下方「研究資料」填寫。</p>
    </div>
  )
}
