/**
 * 設備基本資訊卡片 — 顯示於設備履歷頁面上方
 */
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { StatusBadge } from '@/components/ui/status-badge'
import type { StatusVariant } from '@/components/ui/status-badge'
import { Monitor } from 'lucide-react'
import { format, isPast, parseISO } from 'date-fns'
import { useTranslation } from 'react-i18next'

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

function formatDate(d: string | null) {
  if (!d) return null
  try { return format(parseISO(d), 'yyyy/MM/dd') } catch { return d }
}

function isExpired(d: string | null) {
  if (!d) return false
  try { return isPast(parseISO(d)) } catch { return false }
}

export function EquipmentInfoCard({ equipment }: Props) {
  const { t } = useTranslation()
  const warrantyExpired = isExpired(equipment.warranty_expiry)

  const fields = [
    { label: t('admin.equipmentInfoCard.model'), value: equipment.model },
    { label: t('admin.equipmentInfoCard.serialNumber'), value: equipment.serial_number },
    { label: t('admin.equipmentInfoCard.location'), value: equipment.location },
    { label: t('admin.equipmentInfoCard.department'), value: equipment.department },
    {
      label: t('admin.equipmentInfoCard.purchaseDate'),
      value: formatDate(equipment.purchase_date),
    },
    {
      label: t('admin.equipmentInfoCard.warrantyExpiry'),
      value: formatDate(equipment.warranty_expiry),
      highlight: warrantyExpired ? 'error' : undefined,
    },
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
            {t(EQUIPMENT_STATUS_LABELS[equipment.status])}
          </StatusBadge>
        </div>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-3 gap-4 text-sm">
          {fields.map((f) => (
            <div key={f.label}>
              <span className="text-muted-foreground">{f.label}</span>
              <p
                className={
                  f.highlight === 'error'
                    ? 'font-medium text-destructive'
                    : 'font-medium'
                }
              >
                {f.value || '—'}
                {f.highlight === 'error' && f.value
                  ? ` ${t('admin.equipmentInfoCard.expired')}`
                  : ''}
              </p>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
