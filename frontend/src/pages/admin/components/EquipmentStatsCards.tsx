import { format } from 'date-fns'

import { StatsCard } from '@/components/ui/stats-card'
import { Package, Ruler, AlertTriangle, Wrench } from 'lucide-react'

import type { Equipment, CalibrationWithEquipment } from '../types'

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
      <StatsCard label="設備總數 (啟用)" value={stats.activeCount} icon={Package} accentColor="info" />
      <StatsCard
        label="待修"
        value={stats.repairCount}
        icon={Wrench}
        accentColor="warning"
        valueClassName={stats.repairCount > 0 ? 'text-status-warning-text' : ''}
      />
      <StatsCard label="校正/確效/查核紀錄" value={stats.totalCalib} icon={Ruler} accentColor="info" />
      <StatsCard
        label="逾期待處理"
        value={stats.overdueCount}
        icon={AlertTriangle}
        accentColor="error"
        valueClassName={stats.overdueCount > 0 ? 'text-destructive' : ''}
      />
    </div>
  )
}
