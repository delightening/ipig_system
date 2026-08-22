import { useMemo, useCallback } from 'react'
import { useTranslation } from 'react-i18next'

import { reviewerKey } from '@/lib/reviewerKey'
import type { ReviewCommentResponse, Protocol } from '@/types/aup'

export interface ReviewerQuestion {
  comment: ReviewCommentResponse
  replies: ReviewCommentResponse[]
}

export interface ReviewerGroup {
  /** 分組鍵：系統內審查者用 id、院外審查者（補登）用 name 派生鍵 */
  reviewerId: string
  displayName: string
  questions: ReviewerQuestion[]
}

interface UseCommentsDataParams {
  comments: ReviewCommentResponse[]
  protocolId: string
  protocol: Protocol
  piName?: string
  shouldAnonymizeReviewers: boolean
}

export function useCommentsData({
  comments,
  protocolId,
  protocol,
  piName,
  shouldAnonymizeReviewers,
}: UseCommentsDataParams) {
  const { t } = useTranslation()

  const reviewerAnonymousMap = useMemo(() => {
    const uniqueReviewerKeys = Array.from(new Set(comments.map(reviewerKey)))
    const seed = protocolId?.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0) || 0
    const shuffled = [...uniqueReviewerKeys].sort((a, b) => {
      const hashA = ((seed * a.charCodeAt(0)) % 26)
      const hashB = ((seed * b.charCodeAt(0)) % 26)
      return hashA - hashB
    })
    const map = new Map<string, string>()
    shuffled.forEach((key, index) => {
      const letter = String.fromCharCode(65 + index)
      map.set(key, t(`protocols.detail.actions.reviewer${letter}`))
    })
    return map
  }, [comments, protocolId, t])

  const getReviewerDisplayName = useCallback((comment: ReviewCommentResponse) => {
    if (comment.parent_comment_id && (comment.replied_by === protocol.pi_user_id || comment.replied_by_name)) {
      return comment.replied_by_name || piName || t('common.user')
    }
    if (!shouldAnonymizeReviewers) {
      return comment.reviewer_name || comment.reviewer_email || comment.replied_by_name || comment.replied_by_email
    }
    return reviewerAnonymousMap.get(reviewerKey(comment)) || t('protocols.detail.actions.reviewer')
  }, [protocol, piName, shouldAnonymizeReviewers, reviewerAnonymousMap, t])

  const groupCommentsByReviewer = useCallback((stage: string): ReviewerGroup[] => {
    const topComments = comments.filter(c => !c.parent_comment_id && c.review_stage === stage)
    const reviewerKeys = Array.from(new Set(topComments.map(reviewerKey)))

    return reviewerKeys.map(key => {
      const reviewerComments = topComments.filter(c => reviewerKey(c) === key)
      const sampleComment = reviewerComments[0]

      return {
        reviewerId: key,
        displayName: sampleComment ? getReviewerDisplayName(sampleComment) ?? '' : '',
        questions: reviewerComments.map(c => ({
          comment: c,
          replies: comments.filter(r => r.parent_comment_id === c.id),
        })),
      }
    })
  }, [comments, getReviewerDisplayName])

  const preReviewGroups = useMemo(() => groupCommentsByReviewer('PRE_REVIEW'), [groupCommentsByReviewer])
  const underReviewGroups = useMemo(() => groupCommentsByReviewer('UNDER_REVIEW'), [groupCommentsByReviewer])

  return {
    getReviewerDisplayName,
    preReviewGroups,
    underReviewGroups,
  }
}
