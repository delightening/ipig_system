import { reviewerKey } from '@/lib/reviewerKey'
import type {
  ImportReviewComment,
  ImportCommitteeReviewer,
  ImportVetReview,
} from '@/lib/api/protocol'
import type { ReviewCommentResponse, VetReviewAssignment } from '@/types/aup'

import type { ReviewerValue } from './reviewer'

export interface ReconstructedReviews {
  secretary: ReviewerValue
  secretaryComments: ImportReviewComment[]
  committeeReviewers: ImportCommitteeReviewer[]
  vetReview: ImportVetReview
}

/** 取某條意見的申請人回覆（第一條子意見內容） */
function replyOf(all: ReviewCommentResponse[], parentId: string): string {
  return all.find((r) => r.parent_comment_id === parentId)?.content ?? ''
}

function toComment(all: ReviewCommentResponse[], c: ReviewCommentResponse): ImportReviewComment {
  return { content: c.content, reply: replyOf(all, c.id), section_no: c.section_no ?? '' }
}

/** 系統內審查者（有 id）→ 用 id；院外 → 用顯示名稱。 */
function toReviewer(c: ReviewCommentResponse | undefined): ReviewerValue {
  const reviewer_id = c?.reviewer_id ?? null
  return { reviewer_id, reviewer_name: reviewer_id ? '' : (c?.reviewer_name ?? '') }
}

/**
 * 把既有 review_comments + vet_review 還原成補登表單初始狀態，
 * 讓「全量取代」式儲存不會誤刪先前補登的審查資料（Gemini #535）。
 */
export function reconstructReviews(
  comments: ReviewCommentResponse[],
  vet: VetReviewAssignment | undefined,
): ReconstructedReviews {
  const tops = comments.filter((c) => !c.parent_comment_id)

  // 執秘意見（PRE_REVIEW）
  const secretaryTops = tops.filter((c) => c.review_stage === 'PRE_REVIEW')
  const secretary = toReviewer(secretaryTops[0])
  const secretaryComments = secretaryTops.map((c) => toComment(comments, c))

  // 委員意見（UNDER_REVIEW 一審 + FINAL_REVIEW/FINAL 二審），依審查者分組
  const committeeTops = tops.filter((c) =>
    ['UNDER_REVIEW', 'FINAL_REVIEW', 'FINAL'].includes(c.review_stage ?? ''),
  )
  const order: string[] = []
  const byKey = new Map<string, ReviewCommentResponse[]>()
  for (const c of committeeTops) {
    const k = reviewerKey(c)
    if (!byKey.has(k)) {
      byKey.set(k, [])
      order.push(k)
    }
    byKey.get(k)!.push(c)
  }
  const committeeReviewers: ImportCommitteeReviewer[] = order.map((k) => {
    const group = byKey.get(k) ?? []
    const r = toReviewer(group[0])
    return {
      reviewer_id: r.reviewer_id,
      reviewer_name: r.reviewer_name,
      first_round: group
        .filter((c) => c.review_stage === 'UNDER_REVIEW')
        .map((c) => toComment(comments, c)),
      second_round: group
        .filter((c) => c.review_stage === 'FINAL_REVIEW' || c.review_stage === 'FINAL')
        .map((c) => toComment(comments, c)),
    }
  })

  // 獸醫師評比（保留既有 decision / decision_remark，避免全量取代時遺失）
  const vetReview: ImportVetReview = {
    vet_id: vet?.vet_id ?? null,
    vet_name: vet?.vet_id ? '' : (vet?.vet_name ?? ''),
    decision: vet?.decision ?? '',
    decision_remark: vet?.decision_remark ?? '',
    items: (vet?.review_form?.items ?? []).map((i) => ({
      item_name: i.item_name,
      compliance: i.compliance,
      comment: i.comment ?? '',
      pi_reply: i.pi_reply ?? '',
    })),
  }

  return { secretary, secretaryComments, committeeReviewers, vetReview }
}
