import { format } from 'date-fns'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Package, Ruler, AlertTriangle } from 'lucide-react'

import type { Equipment, CalibrationWithEquipment } from '../types'

interface StatsCardProps {
  title: string
  value: number
  icon: React.ComponentType<{ className?: string }>
  iconClassName?: string
  valueClassName?: string
}

function StatsCard({ title, value, icon: Icon, iconClassName, valueClassName }: StatsCardProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className={`h-4 w-4 ${iconClassName ?? 'text-muted-foreground'}`} />
      </CardHeader>
      <CardContent>
        <div className={`text-2xl font-bold ${valueClassName ?? ''}`}>{value}</div>
      </CardContent>
    </Card>
  )
}

/** 計算設備統計資訊 */
function computeEquipmentStats(equipmentList: Equipment[], allCalibrations: CalibrationWithEquipment[]) {
  const today = format(new Date(), 'yyyy-MM-dd')
  const byEquipment = new Map<string, CalibrationWithEquipment>()

  const sorted = [...allCalibrations].sort(
    (a, b) => new Date(b.calibrated_at).getTime() - new Date(a.calibrated_at).getTime(),
  )
  for (const c of sorted) {
    if (!byEquipment.has(c.equipment_id)) byEquipment.set(c.equipment_id, c)
  }

  const overdueCount = [...byEquipment.values()].filter(
    (c) => c.next_due_at && c.next_due_at < today,
  ).length

  return { totalEquip: equipmentList.length, totalCalib: allCalibrations.length, overdueCount }
}

interface EquipmentStatsCardsProps {
  equipmentList: Equipment[]
  allCalibrations: CalibrationWithEquipment[]
}

export function EquipmentStatsCards({ equipmentList, allCalibrations }: EquipmentStatsCardsProps) {
  const stats = computeEquipmentStats(equipmentList, allCalibrations)

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <StatsCard title="設備總數" value={stats.totalEquip} icon={Package} />
      <StatsCard title="校正紀錄總數" value={stats.totalCalib} icon={Ruler} />
      <StatsCard
        title="逾期校正設備數"
        value={stats.overdueCount}
        icon={AlertTriangle}
        iconClassName="text-orange-500"
        valueClassName="text-orange-500"
      />
    </div>
  )
}
