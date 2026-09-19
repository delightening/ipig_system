import { useTranslation } from 'react-i18next'

import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

/**
 * 計畫起訖日。起始日 = 計畫核准通過日（唯讀，自里程碑帶入，不另填）；結束日必填。
 * 手機單欄避免日期框 overflow。
 */
export function DateRangePicker({
  startDate,
  endDate,
  onEndChange,
}: {
  /** 計畫核准通過日（= 起始日，唯讀） */
  startDate: string
  endDate: string
  onEndChange: (v: string) => void
}) {
  const { t } = useTranslation()
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <div className="grid gap-2 min-w-0">
        <Label>{t('protocolPages.importReview.dateRange.startDate')}</Label>
        <Input type="date" className="w-full" value={startDate} readOnly disabled />
        {!startDate && (
          <p className="text-xs text-muted-foreground">{t('protocolPages.importReview.dateRange.startDateHint')}</p>
        )}
      </div>
      <div className="grid gap-2 min-w-0">
        <Label>{t('protocolPages.importReview.dateRange.endDate')}</Label>
        <Input type="date" className="w-full" value={endDate} onChange={(e) => onEndChange(e.target.value)} />
      </div>
    </div>
  )
}
