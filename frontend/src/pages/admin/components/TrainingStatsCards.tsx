import { StatsCard } from '@/components/ui/stats-card'
import { GraduationCap, AlertTriangle } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface TrainingStatsCardsProps {
  totalRecords: number
  expiringSoonCount: number
}

export function TrainingStatsCards({ totalRecords, expiringSoonCount }: TrainingStatsCardsProps) {
  const { t } = useTranslation()
  return (
    <div className="grid gap-4 md:grid-cols-2">
      <StatsCard label={t('admin.trainingStatsCards.totalRecords')} value={totalRecords} icon={GraduationCap} />
      <StatsCard
        label={t('admin.trainingStatsCards.expiringSoon')}
        value={expiringSoonCount}
        icon={AlertTriangle}
        iconClassName="text-status-warning-text"
        valueClassName="text-status-warning-text"
      />
    </div>
  )
}
