import { useEffect, useRef, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'

import api from '@/lib/api'
import {
  recordImportReviews,
  type ImportReviewComment,
  type ImportCommitteeReviewer,
  type ImportReviewsRequest,
  type ImportVetReview,
} from '@/lib/api/protocol'
import { getApiErrorMessage } from '@/lib/apiError'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import type { ReviewCommentResponse, VetReviewAssignment } from '@/types/aup'

import { CollapsibleCard } from './CollapsibleCard'
import { CommentRows } from './CommentRows'
import { CommitteeReviewers } from './CommitteeReviewers'
import { VetReviewSection } from './VetReviewSection'
import { ReviewerSelect } from './ReviewerSelect'
import { EMPTY_REVIEWER, type ReviewerValue } from './reviewer'
import { IACUC_STAFF, type UserOption } from './users'
import { reconstructReviews } from './reconstruct'

const EMPTY_VET: ImportVetReview = { vet_name: '', decision: '', items: [] }

/** 去除空白意見列（content 為空者不送出） */
function pruneComments(rows: ImportReviewComment[]): ImportReviewComment[] {
  return rows
    .filter((c) => c.content.trim().length > 0)
    .map((c) => ({ ...c, content: c.content.trim(), reply: c.reply?.trim() || null }))
}

interface FormState {
  secretary: ReviewerValue
  secretaryComments: ImportReviewComment[]
  committeeReviewers: ImportCommitteeReviewer[]
  vetReview: ImportVetReview
}

/** 審查者身分（系統內 id 或院外姓名）至少其一。 */
function hasReviewer(reviewer_id: string | null | undefined, name: string | null | undefined): boolean {
  return !!reviewer_id || !!(name && name.trim())
}

/** 驗證 + 正規化補登審查表單 → 送出 payload；回傳錯誤訊息或 payload。 */
function buildReviewsPayload(s: FormState): { error: string } | { payload: ImportReviewsRequest } {
  const secretary = pruneComments(s.secretaryComments)
  if (secretary.length > 0 && !hasReviewer(s.secretary.reviewer_id, s.secretary.reviewer_name)) {
    return { error: '有執秘意見時請選擇執秘或填寫姓名' }
  }
  const committee = s.committeeReviewers
    .map((r) => ({
      reviewer_id: r.reviewer_id ?? null,
      reviewer_name: r.reviewer_name?.trim() || null,
      first_round: pruneComments(r.first_round),
      second_round: pruneComments(r.second_round),
    }))
    .filter((r) => r.first_round.length > 0 || r.second_round.length > 0)
  if (committee.some((r) => !hasReviewer(r.reviewer_id, r.reviewer_name))) {
    return { error: '每位委員都需選擇委員或填寫姓名' }
  }
  const vetItems = s.vetReview.items.filter((i) => i.item_name.trim().length > 0)
  const hasVet = hasReviewer(s.vetReview.vet_id, s.vetReview.vet_name) || vetItems.length > 0
  const vet_review = hasVet
    ? { ...s.vetReview, vet_name: s.vetReview.vet_name?.trim() || null, items: vetItems, signed_at: null }
    : null
  if (vet_review && !hasReviewer(vet_review.vet_id, vet_review.vet_name)) {
    return { error: '有獸醫評比時請選擇獸醫師或填寫姓名' }
  }
  return {
    payload: {
      secretary_comments: secretary.map((c) => ({
        ...c,
        reviewer_id: s.secretary.reviewer_id,
        reviewer_name: s.secretary.reviewer_name.trim() || null,
      })),
      committee_reviewers: committee,
      vet_review,
    },
  }
}

/**
 * 補登審查文件表單：執秘 / 委員 / 獸醫師意見，一次送出（全量取代）。
 * 審查者皆以「角色過濾下拉 + 其他填姓名」選擇。主席核准同意函於上層頁面以附件上傳。
 */
export function ImportReviewArtifactsForm({
  protocolId,
  initialVetReview,
  onSaved,
}: {
  protocolId: string
  initialVetReview?: VetReviewAssignment
  onSaved?: () => void
}) {
  const [secretary, setSecretary] = useState<ReviewerValue>(EMPTY_REVIEWER)
  const [secretaryComments, setSecretaryComments] = useState<ImportReviewComment[]>([])
  const [committeeReviewers, setCommitteeReviewers] = useState<ImportCommitteeReviewer[]>([])
  const [vetReview, setVetReview] = useState<ImportVetReview>(EMPTY_VET)
  const populatedRef = useRef(false)
  const queryClient = useQueryClient()

  // 補登審查者下拉（執秘 IACUC_STAFF / 委員 REVIEWER / 獸醫 VET）取自 assignable-users
  // （門檻＝匯入權限，非 admin.user.view），讓 EXPERIMENT_STAFF 也能選系統內該角色，而非只能選「其他」。
  const { data: users = [], isLoading: usersLoading } = useQuery({
    queryKey: ['protocols', 'assignable-users'],
    queryFn: async () => (await api.get<UserOption[]>('/protocols/assignable-users')).data,
  })

  const { data: existingComments } = useQuery({
    queryKey: ['protocol-comments', protocolId],
    queryFn: async () =>
      (await api.get<ReviewCommentResponse[]>('/reviews/comments', {
        params: { protocol_id: protocolId },
      })).data,
    enabled: !!protocolId,
  })

  useEffect(() => {
    if (populatedRef.current || existingComments === undefined) return
    populatedRef.current = true
    const r = reconstructReviews(existingComments, initialVetReview)
    if (r.secretaryComments.length || r.committeeReviewers.length || r.vetReview.items.length || r.vetReview.vet_name) {
      setSecretary(r.secretary)
      setSecretaryComments(r.secretaryComments)
      setCommitteeReviewers(r.committeeReviewers)
      setVetReview(r.vetReview)
    }
  }, [existingComments, initialVetReview])

  const saveMutation = useMutation({
    mutationFn: (payload: ImportReviewsRequest) => recordImportReviews(protocolId, payload),
    onSuccess: () => {
      toast({ title: '成功', description: '已記錄補登審查文件' })
      queryClient.invalidateQueries({ queryKey: ['protocol-comments', protocolId] })
      onSaved?.()
    },
    onError: (err: unknown) =>
      toast({ title: '錯誤', description: getApiErrorMessage(err, '記錄失敗'), variant: 'destructive' }),
  })

  const handleSave = () => {
    const result = buildReviewsPayload({ secretary, secretaryComments, committeeReviewers, vetReview })
    if ('error' in result) {
      toast({ title: '錯誤', description: result.error, variant: 'destructive' })
      return
    }
    saveMutation.mutate(result.payload)
  }

  return (
    <div className="space-y-4">
      <CollapsibleCard title="執秘意見（行政預審）">
        <ReviewerSelect
          label="執行秘書"
          role={IACUC_STAFF}
          users={users}
          value={secretary}
          onChange={setSecretary}
          disabled={usersLoading}
        />
        <CommentRows value={secretaryComments} onChange={setSecretaryComments} addLabel="新增執秘意見" />
      </CollapsibleCard>

      <CollapsibleCard title="獸醫師評比與意見">
        <VetReviewSection value={vetReview} onChange={setVetReview} users={users} usersLoading={usersLoading} />
      </CollapsibleCard>

      <CollapsibleCard title="委員意見">
        <CommitteeReviewers value={committeeReviewers} onChange={setCommitteeReviewers} users={users} usersLoading={usersLoading} />
      </CollapsibleCard>

      <div className="flex items-center justify-end gap-3">
        <p className="text-sm text-muted-foreground">儲存會以目前表單內容取代先前補登的審查文件。</p>
        <Button onClick={handleSave} disabled={saveMutation.isPending}>
          {saveMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
          儲存補登審查文件
        </Button>
      </div>
    </div>
  )
}
