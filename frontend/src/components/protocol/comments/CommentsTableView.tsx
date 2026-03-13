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
import { formatDateTime } from '@/lib/utils'
import type { ReviewCommentResponse } from '@/types/aup'
import type { ReviewerGroup } from './useCommentsData'

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

  return (
    <div className="mb-6">
      <h3 className="font-semibold text-base mb-2">{title}</h3>
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/50">
            <TableHead className="w-16 text-center">{t('protocols.detail.tables.itemNo')}</TableHead>
            <TableHead className="w-[42%]">{t('protocols.detail.tables.reviewOpinion')}</TableHead>
            <TableHead className="w-[42%]">{t('protocols.detail.tables.applicantReply')}</TableHead>
            {canReply && <TableHead className="w-20 text-center">{t('protocols.detail.tables.actions')}</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {group.questions.map((q, i) => (
            <TableRow
              key={q.comment.id}
              className={q.comment.is_resolved ? 'bg-green-50' : ''}
            >
              <TableCell className="text-center align-top font-medium">{i + 1}</TableCell>
              <TableCell className="align-top">
                <p className="whitespace-pre-wrap text-sm">{q.comment.content}</p>
                <p className="text-xs text-muted-foreground mt-1">
                  {formatDateTime(q.comment.created_at)}
                </p>
              </TableCell>
              <TableCell className="align-top">
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
                  <span className="text-sm text-muted-foreground italic">
                    {t('protocols.detail.tables.noReplyYet')}
                  </span>
                )}
                {q.comment.is_resolved && (
                  <Badge variant="success" className="mt-2">
                    {t('protocols.detail.actions.resolved')}
                  </Badge>
                )}
              </TableCell>
              {canReply && (
                <TableCell className="text-center align-top">
                  {!q.comment.is_resolved && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => onReply(q.comment)}
                    >
                      <Reply className="h-4 w-4" />
                    </Button>
                  )}
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>
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
