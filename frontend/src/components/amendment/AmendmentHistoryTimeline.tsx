import { useTranslation } from 'react-i18next'

import { Badge } from '@/components/ui/badge'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { History } from 'lucide-react'
import { formatDateTime } from '@/lib/utils'
import { amendmentStatusColors } from '@/types/amendment'
import type { AmendmentStatusHistory } from '@/types/amendment'

interface AmendmentHistoryTimelineProps {
  history?: AmendmentStatusHistory[]
  isLoading: boolean
}

/** 狀態歷程時間軸（反向時間序，新→舊）。資料來源 GET /amendments/:id/history。 */
export function AmendmentHistoryTimeline({ history, isLoading }: AmendmentHistoryTimelineProps) {
  const { t } = useTranslation()

  if (isLoading) return <TableSkeleton rows={4} cols={1} />

  if (!history || history.length === 0) {
    return (
      <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
        <History className="h-8 w-8" />
        <p className="text-sm">{t('amendments.decision.history.empty')}</p>
      </div>
    )
  }

  const ordered = [...history].sort(
    (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
  )

  return (
    <ol className="space-y-3">
      {ordered.map((h) => (
        <li key={h.id} className="rounded-lg border bg-card p-3 space-y-1">
          <div className="flex flex-wrap items-center gap-2">
            {h.from_status && (
              <>
                <Badge variant={amendmentStatusColors[h.from_status]}>
                  {t(`amendments.status.${h.from_status}`)}
                </Badge>
                <span className="text-muted-foreground">→</span>
              </>
            )}
            <Badge variant={amendmentStatusColors[h.to_status]}>
              {t(`amendments.status.${h.to_status}`)}
            </Badge>
            <span className="text-xs text-muted-foreground ml-auto">
              {formatDateTime(h.created_at)}
            </span>
          </div>
          {h.remark && <p className="text-sm text-muted-foreground break-words">{h.remark}</p>}
        </li>
      ))}
    </ol>
  )
}
