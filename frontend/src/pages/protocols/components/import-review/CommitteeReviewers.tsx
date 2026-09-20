import { useTranslation } from 'react-i18next'

import type { ImportCommitteeReviewer } from '@/lib/api/protocol'
import { Repeater } from '@/components/ui/repeater'
import { Label } from '@/components/ui/label'

import { CommentRows } from './CommentRows'
import { ReviewerSelect } from './ReviewerSelect'
import { REVIEWER, type UserOption } from './users'

/** 補登：各委員意見（每位委員含一審 + 二審）。委員以 REVIEWER 角色下拉 + 其他選擇。 */
export function CommitteeReviewers({
  value,
  onChange,
  users,
  usersLoading,
}: {
  value: ImportCommitteeReviewer[]
  onChange: (v: ImportCommitteeReviewer[]) => void
  users: UserOption[]
  usersLoading: boolean
}) {
  const { t } = useTranslation()
  return (
    <Repeater<ImportCommitteeReviewer>
      value={value}
      onChange={onChange}
      defaultItem={() => ({ reviewer_id: null, reviewer_name: '', first_round: [], second_round: [] })}
      addLabel={t('protocolPages.importReview.committee.addReviewer')}
      maxItems={10}
      renderItem={(item, idx, onItemChange) => (
        <div className="grid gap-3 rounded-md border p-4">
          <ReviewerSelect
            label={t('protocolPages.importReview.committee.reviewerLabel', { index: idx + 1 })}
            role={REVIEWER}
            users={users}
            value={{ reviewer_id: item.reviewer_id ?? null, reviewer_name: item.reviewer_name ?? '' }}
            onChange={(r) => onItemChange({ ...item, reviewer_id: r.reviewer_id, reviewer_name: r.reviewer_name })}
            disabled={usersLoading}
          />
          <div className="grid gap-1">
            <Label className="text-sm text-muted-foreground">{t('protocolPages.importReview.committee.firstRound')}</Label>
            <CommentRows
              value={item.first_round}
              onChange={(v) => onItemChange({ ...item, first_round: v })}
              addLabel={t('protocolPages.importReview.committee.addFirstRound')}
            />
          </div>
          <div className="grid gap-1">
            <Label className="text-sm text-muted-foreground">{t('protocolPages.importReview.committee.secondRound')}</Label>
            <CommentRows
              value={item.second_round}
              onChange={(v) => onItemChange({ ...item, second_round: v })}
              addLabel={t('protocolPages.importReview.committee.addSecondRound')}
            />
          </div>
        </div>
      )}
    />
  )
}
