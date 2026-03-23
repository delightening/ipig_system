import { format } from 'date-fns'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Package, Ruler, AlertTriangle, Wrench } from 'lucide-react'

import type { Equipment, CalibrationWithEquipment } from '../types'
import { EQUIPMENT_STATUS_LABELS } from '../types'

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

function computeEquipmentStats(
  equipmentList: Equipment[],
  allCalibrations: CalibrationWithEquipment[],
) {
  const today = format(new Date(), 'yyyy-MM-dd')

  // 每台設備每種類型的最新校正紀錄
  const latestByKey = new Map<string, CalibrationWithEquipment>()
  const sorted = [...allCalibrations].sort(
    (a, b) => new Date(b.calibrated_at).getTime() - new Date(a.calibrated_at).getTime(),
  )
  for (const c of sorted) {
    const key = `${c.equipment_id}:${c.calibration_type}`
    if (!latestByKey.has(key)) latestByKey.set(key, c)
  }

  const overdueCount = [...latestByKey.values()].filter(
    (c) => c.next_due_at && c.next_due_at < today,
  ).length

  const activeCount = equipmentList.filter((e) => e.status === 'active').length
  const repairCount = equipmentList.filter((e) => e.status === 'under_repair').length

  return {
    totalEquip: equipmentList.length,
    activeCount,
    repairCount,
    totalCalib: allCalibrations.length,
    overdueCount,
  }
}

interface EquipmentStatsCardsProps {
  equipmentList: Equipment[]
  allCalibrations: CalibrationWithEquipment[]
}

export function EquipmentStatsCards({ equipmentList, allCalibrations }: EquipmentStatsCardsProps) {
  const stats = computeEquipmentStats(equipmentList, allCalibrations)

  return (
    <div className="grid gap-4 md:grid-cols-4">
      <StatsCard title="設備總數 (啟用)" value={stats.activeCount} icon={Package} />
      <StatsCard
        title="維修中"
        value={stats.repairCount}
        icon={Wrench}
        iconClassName={stats.repairCount > 0 ? 'text-yellow-500' : 'text-muted-foreground'}
        valueClassName={stats.repairCount > 0 ? 'text-yellow-500' : ''}
      />
      <StatsCard title="校正/確效/查核紀錄" value={stats.totalCalib} icon={Ruler} />
      <StatsCard
        title="逾期待處理"
        value={stats.overdueCount}
        icon={AlertTriangle}
        iconClassName={stats.overdueCount > 0 ? 'text-red-500' : 'text-muted-foreground'}
        valueClassName={stats.overdueCount > 0 ? 'text-red-500' : ''}
      />
    </div>
  )
}
