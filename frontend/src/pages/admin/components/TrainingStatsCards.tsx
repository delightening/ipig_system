import { StatsCard } from '@/components/ui/stats-card'
import { GraduationCap, AlertTriangle } from 'lucide-react'

interface TrainingStatsCardsProps {
  totalRecords: number
  expiringSoonCount: number
}

export function TrainingStatsCards({ totalRecords, expiringSoonCount }: TrainingStatsCardsProps) {
  return (
    <div className="grid gap-4 md:grid-cols-2">
      <StatsCard label="訓練紀錄數" value={totalRecords} icon={GraduationCap} />
      <StatsCard
        label="證照即將到期"
        value={expiringSoonCount}
        icon={AlertTriangle}
        iconClassName="text-status-warning-text"
        valueClassName="text-status-warning-text"
      />
    </div>
  )
}
