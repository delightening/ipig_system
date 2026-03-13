import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'

import type { ReviewCommentResponse, Protocol } from '@/types/aup'

export interface ReviewerQuestion {
  comment: ReviewCommentResponse
  replies: ReviewCommentResponse[]
}

export interface ReviewerGroup {
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
    const uniqueReviewerIds = Array.from(
      new Set(comments.map(c => c.reviewer_id).filter(Boolean))
    )
    const seed = protocolId?.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0) || 0
    const shuffled = [...uniqueReviewerIds].sort((a, b) => {
      const hashA = ((seed * a.charCodeAt(0)) % 26)
      const hashB = ((seed * b.charCodeAt(0)) % 26)
      return hashA - hashB
    })
    const map = new Map<string, string>()
    shuffled.forEach((reviewerId, index) => {
      const letter = String.fromCharCode(65 + index)
      map.set(reviewerId, t(`protocols.detail.actions.reviewer${letter}`))
    })
    return map
  }, [comments, protocolId, t])

  const getReviewerDisplayName = (comment: ReviewCommentResponse) => {
    if (comment.parent_comment_id && (comment.replied_by === protocol.pi_user_id || comment.replied_by_name)) {
      return comment.replied_by_name || piName || t('common.user')
    }
    if (!shouldAnonymizeReviewers) {
      return comment.reviewer_name || comment.reviewer_email || comment.replied_by_name || comment.replied_by_email
    }
    return reviewerAnonymousMap.get(comment.reviewer_id) || t('protocols.detail.actions.reviewer')
  }

  const groupCommentsByReviewer = (stage: string): ReviewerGroup[] => {
    const topComments = comments.filter(c => !c.parent_comment_id && c.review_stage === stage)
    const reviewerIds = Array.from(new Set(topComments.map(c => c.reviewer_id)))

    return reviewerIds.map(reviewerId => {
      const reviewerComments = topComments.filter(c => c.reviewer_id === reviewerId)
      const sampleComment = reviewerComments[0]

      return {
        reviewerId,
        displayName: sampleComment ? getReviewerDisplayName(sampleComment) ?? '' : '',
        questions: reviewerComments.map(c => ({
          comment: c,
          replies: comments.filter(r => r.parent_comment_id === c.id),
        })),
      }
    })
  }

  const preReviewGroups = useMemo(() => groupCommentsByReviewer('PRE_REVIEW'), [comments])
  const underReviewGroups = useMemo(() => groupCommentsByReviewer('UNDER_REVIEW'), [comments])

  return {
    getReviewerDisplayName,
    preReviewGroups,
    underReviewGroups,
  }
}
