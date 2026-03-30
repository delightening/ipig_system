import React, { useRef, useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  ProtocolVersion,
  ReviewCommentResponse,
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
import { ReviewCommentPanel } from '@/components/protocol/ReviewCommentPanel'
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
  sectionOptions: string[]
}

export const CommentsTab = React.memo(function CommentsTab({
  protocolId,
  protocol,
  piName,
  vetReview,
  canAddComment,
  canReply,
  shouldAnonymizeReviewers,
  sectionOptions,
}: CommentsTabProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const reportRef = useRef<HTMLDivElement>(null)

  const [showCommentPanel, setShowCommentPanel] = useState(false)
  const [showReplyDialog, setShowReplyDialog] = useState(false)
  const [replyContent, setReplyContent] = useState('')
  const [selectedCommentForReply, setSelectedCommentForReply] = useState<ReviewCommentResponse | null>(null)

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
    mutationFn: async (content: string) => {
      if (!versions || versions.length === 0) throw new Error('No version found')
      return api.post('/reviews/comments', {
        protocol_version_id: versions[0].id,
        content,
      })
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.dialogs.comment.success') })
      queryClient.invalidateQueries({ queryKey: ['protocol-comments', protocolId] })
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

  const handleReply = (comment: ReviewCommentResponse) => {
    setSelectedCommentForReply(comment)
    setShowReplyDialog(true)
  }

  const exportCommentsPDFMutation = useMutation({
    mutationFn: async () => {
      const response = await api.get(`/protocols/${protocolId}/export-review-comments`, {
        responseType: 'blob',
        _silentError: true,
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
    },
    onSuccess: () => {
      toast({ title: t('common.exportSuccess'), variant: 'default' })
    },
    onError: (error) => {
      logger.error('PDF export error:', error)
      toast({ title: t('common.exportFailed'), variant: 'destructive' })
    },
  })

  const exportReviewResultMutation = useMutation({
    mutationFn: async () => {
      const response = await api.get(`/protocols/${protocolId}/export-review-result`, {
        responseType: 'blob',
        _silentError: true,
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
    },
    onSuccess: () => {
      toast({ title: t('common.exportSuccess'), variant: 'default' })
    },
    onError: (error) => {
      logger.error('Review result PDF export error:', error)
      toast({ title: t('common.exportFailed'), variant: 'destructive' })
    },
  })

  return (
    <>
      <div className="flex gap-4">
        {/* 左側：審查意見列表 */}
        <div className={showCommentPanel ? 'flex-1 min-w-0' : 'w-full'}>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>{t('protocols.detail.sections.commentsTitle')}</CardTitle>
                <CardDescription>{t('protocols.detail.sections.commentsDesc')}</CardDescription>
              </div>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  onClick={() => exportReviewResultMutation.mutate()}
                  disabled={exportReviewResultMutation.isPending || commentsLoading}
                >
                  {exportReviewResultMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Download className="mr-2 h-4 w-4" />}
                  審核結果 (PDF)
                </Button>
                <Button
                  variant="outline"
                  onClick={() => exportCommentsPDFMutation.mutate()}
                  disabled={exportCommentsPDFMutation.isPending || commentsLoading}
                >
                  {exportCommentsPDFMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Download className="mr-2 h-4 w-4" />}
                  匯出回覆表 (PDF)
                </Button>
                {canAddComment && protocol.status !== 'DRAFT' && (
                  <Button onClick={() => setShowCommentPanel(prev => !prev)}>
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
                    <Button variant="link" onClick={() => setShowCommentPanel(true)} className="mt-2">
                      {t('protocols.detail.dialogs.comment.firstComment')}
                    </Button>
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        </div>

        {/* 右側：浮動審查意見面板 */}
        {showCommentPanel && (
          <div className="w-72 shrink-0 sticky top-4 self-start">
            <ReviewCommentPanel
              onClose={() => setShowCommentPanel(false)}
              onSubmit={(content) => addCommentMutation.mutate(content)}
              isSubmitting={addCommentMutation.isPending}
              sectionOptions={sectionOptions}
            />
          </div>
        )}
      </div>

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
            <div className="bg-muted p-3 rounded-lg border mb-4">
              <p className="text-sm font-medium text-muted-foreground">{t('protocols.detail.dialogs.reply.original')}</p>
              <p className="text-sm text-foreground mt-1">{selectedCommentForReply.content}</p>
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
