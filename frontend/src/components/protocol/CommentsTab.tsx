import React, { useRef, useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  ProtocolVersion,
  ReviewCommentResponse,
  CreateCommentRequest,
  ReplyCommentRequest,
  VetReviewAssignment,
} from '@/lib/api'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import { logger } from '@/lib/logger'
import { Download, Loader2, MessageSquare } from 'lucide-react'
import { getApiErrorMessage } from '@/lib/validation'
import { ReviewCommentsReport } from '@/components/protocol/ReviewCommentsReport'
import { CommentsTableView } from './comments/CommentsTableView'
import { useCommentsData } from './comments/useCommentsData'

import type { Protocol } from '@/types/aup'

interface CommentsTabProps {
  protocolId: string
  protocol: Protocol
  piName?: string
  vetReview?: VetReviewAssignment
  canAddComment: boolean
  canReply: boolean
  shouldAnonymizeReviewers: boolean
}

export const CommentsTab = React.memo(function CommentsTab({
  protocolId,
  protocol,
  piName,
  vetReview,
  canAddComment,
  canReply,
  shouldAnonymizeReviewers,
}: CommentsTabProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const reportRef = useRef<HTMLDivElement>(null)

  const [showCommentDialog, setShowCommentDialog] = useState(false)
  const [showReplyDialog, setShowReplyDialog] = useState(false)
  const [commentContent, setCommentContent] = useState('')
  const [replyContent, setReplyContent] = useState('')
  const [selectedCommentForReply, setSelectedCommentForReply] = useState<ReviewCommentResponse | null>(null)
  const [isExportingComments, setIsExportingComments] = useState(false)
  const [isExportingResult, setIsExportingResult] = useState(false)

  const { data: versions } = useQuery({
    queryKey: ['protocol-versions', protocolId],
    queryFn: async () => {
      const response = await api.get<ProtocolVersion[]>(`/protocols/${protocolId}/versions`)
      return response.data
    },
    enabled: !!protocolId,
  })

  const { data: comments, isLoading: commentsLoading } = useQuery({
    queryKey: ['protocol-comments', protocolId],
    queryFn: async () => {
      const response = await api.get<ReviewCommentResponse[]>('/reviews/comments', {
        params: { protocol_id: protocolId },
      })
      return response.data
    },
    enabled: !!protocolId,
  })

  const { preReviewGroups, underReviewGroups } = useCommentsData({
    comments: comments || [],
    protocolId,
    protocol,
    piName,
    shouldAnonymizeReviewers,
  })

  const addCommentMutation = useMutation({
    mutationFn: async (data: CreateCommentRequest) => {
      return api.post('/reviews/comments', data)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.dialogs.comment.success') })
      queryClient.invalidateQueries({ queryKey: ['protocol-comments', protocolId] })
      setShowCommentDialog(false)
      setCommentContent('')
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.dialogs.comment.failed')),
        variant: 'destructive',
      })
    },
  })

  const replyCommentMutation = useMutation({
    mutationFn: async (data: ReplyCommentRequest) => {
      return api.post('/reviews/comments/reply', data)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.dialogs.reply.success') })
      queryClient.invalidateQueries({ queryKey: ['protocol-comments', protocolId] })
      setShowReplyDialog(false)
      setReplyContent('')
      setSelectedCommentForReply(null)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.dialogs.reply.failed')),
        variant: 'destructive',
      })
    },
  })

  const handleAddComment = () => {
    if (!commentContent.trim() || !versions || versions.length === 0) return
    addCommentMutation.mutate({
      protocol_version_id: versions[0].id,
      content: commentContent.trim(),
    })
  }

  const handleReply = (comment: ReviewCommentResponse) => {
    setSelectedCommentForReply(comment)
    setShowReplyDialog(true)
  }

  const handleExportPDF = async () => {
    if (isExportingComments) return
    try {
      setIsExportingComments(true)
      const response = await api.get(`/protocols/${protocolId}/export-review-comments`, {
        responseType: 'blob',
      })
      const blob = new Blob([response.data], { type: 'application/pdf' })
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `審查意見回覆表_${protocol.protocol_no || protocolId}.pdf`
      a.style.display = 'none'
      document.body.appendChild(a)
      a.click()
      window.URL.revokeObjectURL(url)
      document.body.removeChild(a)
      toast({ title: t('common.exportSuccess'), variant: 'default' })
    } catch (error) {
      logger.error('PDF export error:', error)
      toast({ title: t('common.exportFailed'), variant: 'destructive' })
    } finally {
      setIsExportingComments(false)
    }
  }

  const handleExportReviewResult = async () => {
    if (isExportingResult) return
    try {
      setIsExportingResult(true)
      const response = await api.get(`/protocols/${protocolId}/export-review-result`, {
        responseType: 'blob',
      })
      const blob = new Blob([response.data], { type: 'application/pdf' })
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `審核結果_${protocol.protocol_no || protocolId}.pdf`
      a.style.display = 'none'
      document.body.appendChild(a)
      a.click()
      window.URL.revokeObjectURL(url)
      document.body.removeChild(a)
      toast({ title: t('common.exportSuccess'), variant: 'default' })
    } catch (error) {
      logger.error('Review result PDF export error:', error)
      toast({ title: t('common.exportFailed'), variant: 'destructive' })
    } finally {
      setIsExportingResult(false)
    }
  }

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>{t('protocols.detail.sections.commentsTitle')}</CardTitle>
            <CardDescription>{t('protocols.detail.sections.commentsDesc')}</CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              onClick={handleExportReviewResult}
              disabled={isExportingResult || commentsLoading}
            >
              {isExportingResult ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Download className="mr-2 h-4 w-4" />}
              審核結果 (PDF)
            </Button>
            <Button
              variant="outline"
              onClick={handleExportPDF}
              disabled={isExportingComments || commentsLoading}
            >
              {isExportingComments ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Download className="mr-2 h-4 w-4" />}
              匯出回覆表 (PDF)
            </Button>
            {canAddComment && protocol.status !== 'DRAFT' && (
              <Button onClick={() => setShowCommentDialog(true)}>
                <MessageSquare className="mr-2 h-4 w-4" />
                {t('protocols.detail.dialogs.comment.title')}
              </Button>
            )}
          </div>
        </CardHeader>
        <CardContent>
          {commentsLoading ? (
            <div className="flex justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : comments && comments.length > 0 ? (
            <CommentsTableView
              preReviewGroups={preReviewGroups}
              underReviewGroups={underReviewGroups}
              canReply={canReply}
              onReply={handleReply}
            />
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              <MessageSquare className="h-12 w-12 mx-auto mb-2" />
              <p>{t('protocols.detail.tables.noComments')}</p>
              {canAddComment && protocol.status !== 'DRAFT' && (
                <Button variant="link" onClick={() => setShowCommentDialog(true)} className="mt-2">
                  {t('protocols.detail.dialogs.comment.firstComment')}
                </Button>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog open={showCommentDialog} onOpenChange={setShowCommentDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('protocols.detail.dialogs.comment.title')}</DialogTitle>
            <DialogDescription>{t('protocols.detail.dialogs.comment.desc')}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.comment.placeholder')}</Label>
              <Textarea
                value={commentContent}
                onChange={(e) => setCommentContent(e.target.value)}
                placeholder={t('protocols.detail.dialogs.comment.placeholder')}
                rows={5}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCommentDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleAddComment}
              disabled={!commentContent.trim() || addCommentMutation.isPending}
            >
              {addCommentMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t('protocols.detail.dialogs.comment.submit')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={showReplyDialog} onOpenChange={(open) => {
        setShowReplyDialog(open)
        if (!open) {
          setReplyContent('')
          setSelectedCommentForReply(null)
        }
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('protocols.detail.dialogs.reply.title')}</DialogTitle>
            <DialogDescription>{t('protocols.detail.dialogs.reply.desc')}</DialogDescription>
          </DialogHeader>
          {selectedCommentForReply && (
            <div className="bg-slate-50 p-3 rounded-lg border mb-4">
              <p className="text-sm font-medium text-slate-600">{t('protocols.detail.dialogs.reply.original')}</p>
              <p className="text-sm text-slate-700 mt-1">{selectedCommentForReply.content}</p>
              <p className="text-xs text-muted-foreground mt-2">
                — {selectedCommentForReply.reviewer_name || selectedCommentForReply.reviewer_email}
              </p>
            </div>
          )}
          <div className="space-y-4">
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.reply.placeholder')}</Label>
              <Textarea
                value={replyContent}
                onChange={(e) => setReplyContent(e.target.value)}
                placeholder={t('protocols.detail.dialogs.reply.placeholder')}
                rows={4}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowReplyDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={() => {
                if (selectedCommentForReply && replyContent.trim()) {
                  replyCommentMutation.mutate({
                    parent_comment_id: selectedCommentForReply.id,
                    content: replyContent.trim(),
                  })
                }
              }}
              disabled={!replyContent.trim() || replyCommentMutation.isPending}
            >
              {replyCommentMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t('protocols.detail.dialogs.reply.submit')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <div className="fixed -left-[9999px] top-0">
        <div ref={reportRef}>
          <ReviewCommentsReport
            protocol={protocol}
            comments={comments || []}
            vet_review={vetReview}
          />
        </div>
      </div>
    </>
  )
})
