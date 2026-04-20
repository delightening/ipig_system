import { useTranslation } from 'react-i18next'
import { Reply } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { useTableSort } from '@/hooks/useTableSort'
import { formatDateTime } from '@/lib/utils'
import type { ReviewCommentResponse } from '@/types/aup'
import type { ReviewerGroup } from './useCommentsData'

function parseCommentContent(content: string): { section: string | null; text: string } {
  const match = content.match(/^\[(.+?)\]\s*(.*)/)
  if (match) {
    return { section: match[1], text: match[2] }
  }
  return { section: null, text: content }
}

interface CommentsTableViewProps {
  preReviewGroups: ReviewerGroup[]
  underReviewGroups: ReviewerGroup[]
  canReply: boolean
  onReply: (comment: ReviewCommentResponse) => void
}

function ReviewerSection({
  title,
  group,
  canReply,
  onReply,
}: {
  title: string
  group: ReviewerGroup
  canReply: boolean
  onReply: (comment: ReviewCommentResponse) => void
}) {
  const { t } = useTranslation()
  const { sortedData, sort, toggleSort } = useTableSort(group.questions)

  return (
    <div className="mb-6 @container">
      <h3 className="font-semibold text-base mb-2">{title}</h3>

      {/* Table view: container ≥ 600px */}
      <div className="hidden @[600px]:block overflow-x-auto">
        <Table className="w-full" style={{ minWidth: 510 }}>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <TableHead style={{ width: 90 }} className="text-center">{t('protocols.detail.tables.targetSection')}</TableHead>
              <SortableTableHead style={{ minWidth: 180 }} sortKey="comment.created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.detail.tables.reviewOpinion')}</SortableTableHead>
              <SortableTableHead style={{ minWidth: 180 }} sortKey="comment.is_resolved" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.detail.tables.applicantReply')}</SortableTableHead>
              {canReply && <TableHead style={{ width: 60 }} className="text-center">{t('protocols.detail.tables.actions')}</TableHead>}
            </TableRow>
          </TableHeader>
          <TableBody>
            {(sortedData ?? group.questions).map((q) => {
              const parsed = parseCommentContent(q.comment.content)
              return (
                <TableRow key={q.comment.id} className={q.comment.is_resolved ? 'bg-status-success-bg' : ''}>
                  <TableCell style={{ width: 90 }} className="text-center align-top font-medium text-sm">
                    {parsed.section ?? '-'}
                  </TableCell>
                  <TableCell style={{ minWidth: 180 }} className="align-top">
                    <p className="whitespace-pre-wrap text-sm">{parsed.text}</p>
                    <p className="text-xs text-muted-foreground mt-1">{formatDateTime(q.comment.created_at)}</p>
                  </TableCell>
                  <TableCell style={{ minWidth: 180 }} className="align-top">
                    {q.replies.length > 0 ? (
                      <div className="space-y-2">
                        {q.replies.map(r => (
                          <div key={r.id}>
                            <p className="whitespace-pre-wrap text-sm">{r.content}</p>
                            <p className="text-xs text-muted-foreground mt-1">
                              {r.replied_by_name && `${r.replied_by_name} · `}
                              {formatDateTime(r.created_at)}
                            </p>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <span className="text-sm text-muted-foreground italic">{t('protocols.detail.tables.noReplyYet')}</span>
                    )}
                    {q.comment.is_resolved && (
                      <Badge variant="success" className="mt-2">{t('protocols.detail.actions.resolved')}</Badge>
                    )}
                  </TableCell>
                  {canReply && (
                    <TableCell style={{ width: 60 }} className="text-center align-top">
                      {!q.comment.is_resolved && (
                        <Button variant="ghost" size="sm" onClick={() => onReply(q.comment)}>
                          <Reply className="h-4 w-4" />
                        </Button>
                      )}
                    </TableCell>
                  )}
                </TableRow>
              )
            })}
          </TableBody>
        </Table>
      </div>

      {/* Card view: container < 600px */}
      <div className="@[600px]:hidden space-y-3 py-1">
        {(sortedData ?? group.questions).map((q) => {
          const parsed = parseCommentContent(q.comment.content)
          return (
            <div key={q.comment.id} className={`rounded-lg border p-3 space-y-2 ${q.comment.is_resolved ? 'bg-status-success-bg' : 'bg-card'}`}>
              <div className="flex items-center justify-between gap-2">
                <span className="text-xs font-medium text-muted-foreground">{parsed.section ?? '-'}</span>
                {canReply && !q.comment.is_resolved && (
                  <Button variant="ghost" size="icon" onClick={() => onReply(q.comment)} title="回覆">
                    <Reply className="h-4 w-4" />
                  </Button>
                )}
              </div>
              <div>
                <div className="text-xs text-muted-foreground uppercase mb-0.5">{t('protocols.detail.tables.reviewOpinion')}</div>
                <p className="whitespace-pre-wrap text-sm">{parsed.text}</p>
                <p className="text-xs text-muted-foreground mt-1">{formatDateTime(q.comment.created_at)}</p>
              </div>
              <div>
                <div className="text-xs text-muted-foreground uppercase mb-0.5">{t('protocols.detail.tables.applicantReply')}</div>
                {q.replies.length > 0 ? (
                  <div className="space-y-2">
                    {q.replies.map(r => (
                      <div key={r.id}>
                        <p className="whitespace-pre-wrap text-sm">{r.content}</p>
                        <p className="text-xs text-muted-foreground mt-1">
                          {r.replied_by_name && `${r.replied_by_name} · `}
                          {formatDateTime(r.created_at)}
                        </p>
                      </div>
                    ))}
                  </div>
                ) : (
                  <span className="text-sm text-muted-foreground italic">{t('protocols.detail.tables.noReplyYet')}</span>
                )}
                {q.comment.is_resolved && (
                  <Badge variant="success" className="mt-2">{t('protocols.detail.actions.resolved')}</Badge>
                )}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}

export function CommentsTableView({
  preReviewGroups,
  underReviewGroups,
  canReply,
  onReply,
}: CommentsTableViewProps) {
  const { t } = useTranslation()

  const hasPreReview = preReviewGroups.length > 0
  const hasUnderReview = underReviewGroups.length > 0

  if (!hasPreReview && !hasUnderReview) {
    return null
  }

  return (
    <div className="space-y-8">
      {hasPreReview && (
        <section>
          <h2 className="font-bold text-lg border-b pb-2 mb-4">
            {t('protocols.detail.tables.executiveSecretary')}
          </h2>
          {preReviewGroups.map(group => (
            <ReviewerSection
              key={group.reviewerId}
              title={group.displayName}
              group={group}
              canReply={canReply}
              onReply={onReply}
            />
          ))}
        </section>
      )}

      {hasUnderReview && (
        <section>
          <h2 className="font-bold text-lg border-b pb-2 mb-4">
            {t('protocols.detail.sections.reviewersTitle')}
          </h2>
          {underReviewGroups.map((group, idx) => (
            <ReviewerSection
              key={group.reviewerId}
              title={`${t('protocols.detail.tables.reviewer')} ${idx + 1}`}
              group={group}
              canReply={canReply}
              onReply={onReply}
            />
          ))}
        </section>
      )}
    </div>
  )
}
