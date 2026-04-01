/**
 * 設備基本資訊卡片 — 顯示於設備履歷頁面上方
 */
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { StatusBadge } from '@/components/ui/status-badge'
import type { StatusVariant } from '@/components/ui/status-badge'
import { Monitor } from 'lucide-react'

import type { Equipment } from '../types'
import { EQUIPMENT_STATUS_LABELS } from '../types'

const STATUS_VARIANT: Record<string, StatusVariant> = {
  active: 'success',
  inactive: 'neutral',
  under_repair: 'warning',
  decommissioned: 'error',
}

interface Props {
  equipment: Equipment
}

export function EquipmentInfoCard({ equipment }: Props) {
  const fields = [
    { label: '型號', value: equipment.model },
    { label: '序號', value: equipment.serial_number },
    { label: '位置', value: equipment.location },
  ]

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Monitor className="h-5 w-5" />
            {equipment.name}
          </CardTitle>
          <StatusBadge variant={STATUS_VARIANT[equipment.status] || 'neutral'}>
            {EQUIPMENT_STATUS_LABELS[equipment.status]}
          </StatusBadge>
        </div>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-3 gap-4 text-sm">
          {fields.map((f) => (
            <div key={f.label}>
              <span className="text-muted-foreground">{f.label}</span>
              <p className="font-medium">{f.value || '—'}</p>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
