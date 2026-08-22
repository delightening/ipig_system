import { useTranslation } from 'react-i18next'

import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Users } from 'lucide-react'
import { formatDateTime } from '@/lib/utils'
import type { AmendmentReviewAssignment } from '@/types/amendment'

type BadgeVariant = 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline'

/** 決定值 → Badge variant（pending = 尚未投票）。對齊後端 APPROVE/REJECT/REVISION。 */
const decisionVariant = (decision?: string): BadgeVariant => {
  switch (decision) {
    case 'APPROVE':
      return 'success'
    case 'REJECT':
      return 'destructive'
    case 'REVISION':
      return 'warning'
    default:
      return 'secondary'
  }
}

interface AmendmentVoteRosterProps {
  assignments?: AmendmentReviewAssignment[]
  isLoading: boolean
}

/**
 * 委員投票名冊（含未投票列）。投票結算規則：全員投完才結算，
 * precedence REVISION > REJECT > APPROVE（見 services/amendment/workflow.rs）。
 */
export function AmendmentVoteRoster({ assignments, isLoading }: AmendmentVoteRosterProps) {
  const { t } = useTranslation()

  const decisionLabel = (decision?: string) =>
    decision
      ? t(`amendments.decision.vote.${decision}`)
      : t('amendments.decision.pending')

  return (
    <div className="@container">
      <div className="hidden @[600px]:block overflow-x-auto">
        <Table className="w-full" style={{ minWidth: 520 }}>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <TableHead style={{ minWidth: 180 }}>{t('amendments.decision.roster.reviewer')}</TableHead>
              <TableHead style={{ width: 90 }}>{t('amendments.decision.roster.vote')}</TableHead>
              <TableHead style={{ width: 160 }} className="hidden @[780px]:table-cell">{t('amendments.decision.roster.decidedAt')}</TableHead>
              <TableHead style={{ minWidth: 160 }}>{t('amendments.decision.roster.comment')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={4} className="p-0">
                  <TableSkeleton rows={3} cols={4} />
                </TableCell>
              </TableRow>
            ) : assignments && assignments.length > 0 ? (
              assignments.map((a) => (
                <TableRow key={a.id}>
                  <TableCell style={{ minWidth: 180 }}>
                    <p className="font-medium">{a.reviewer_name || '-'}</p>
                    <p className="text-sm text-muted-foreground break-all">{a.reviewer_email || ''}</p>
                  </TableCell>
                  <TableCell style={{ width: 90 }}>
                    <Badge variant={decisionVariant(a.decision)}>{decisionLabel(a.decision)}</Badge>
                  </TableCell>
                  <TableCell style={{ width: 160 }} className="text-xs text-muted-foreground hidden @[780px]:table-cell">
                    {a.decided_at ? formatDateTime(a.decided_at) : '-'}
                  </TableCell>
                  <TableCell style={{ minWidth: 160 }} className="whitespace-normal break-words text-sm">
                    {a.comment || <span className="text-muted-foreground">-</span>}
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={4} icon={Users} title={t('amendments.decision.roster.empty')} />
            )}
          </TableBody>
        </Table>
      </div>

      <div className="@[600px]:hidden space-y-3 py-1">
        {isLoading ? (
          <TableSkeleton rows={3} cols={1} />
        ) : assignments && assignments.length > 0 ? (
          assignments.map((a) => (
            <div key={a.id} className="rounded-lg border bg-card p-3 space-y-2">
              <div className="flex items-start justify-between gap-2">
                <div>
                  <p className="text-sm font-medium text-foreground">{a.reviewer_name || '-'}</p>
                  <p className="text-xs text-muted-foreground break-all">{a.reviewer_email || ''}</p>
                </div>
                <Badge variant={decisionVariant(a.decision)}>{decisionLabel(a.decision)}</Badge>
              </div>
              {a.comment && (
                <p className="text-sm text-muted-foreground leading-snug break-words border-t pt-2">{a.comment}</p>
              )}
              {a.decided_at && (
                <p className="text-xs text-muted-foreground">{formatDateTime(a.decided_at)}</p>
              )}
            </div>
          ))
        ) : (
          <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
            <Users className="h-8 w-8" />
            <p className="text-sm">{t('amendments.decision.roster.empty')}</p>
          </div>
        )}
      </div>
    </div>
  )
}
