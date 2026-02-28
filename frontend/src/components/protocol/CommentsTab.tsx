import React, { useMemo, useRef, useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { AxiosError } from 'axios'
import api, {
  ProtocolVersion,
  ReviewCommentResponse,
  CreateCommentRequest,
  ReplyCommentRequest,
  ProtocolStatus,
  VetReviewAssignment,
} from '@/lib/api'
import type { ApiErrorPayload } from '@/types/error'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
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
import {
  CheckCircle,
  CheckCircle2,
  Download,
  Loader2,
  MessageSquare,
  Reply,
  User as UserIcon,
} from 'lucide-react'
import { formatDateTime } from '@/lib/utils'
import { useAuthStore } from '@/stores/auth'
import { ReviewCommentsReport } from '@/components/protocol/ReviewCommentsReport'
// jsPDF & html2canvas loaded lazily at PDF export time

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
  const { user } = useAuthStore()
  const reportRef = useRef<HTMLDivElement>(null)

  const [showCommentDialog, setShowCommentDialog] = useState(false)
  const [showReplyDialog, setShowReplyDialog] = useState(false)
  const [commentContent, setCommentContent] = useState('')
  const [replyContent, setReplyContent] = useState('')
  const [selectedCommentForReply, setSelectedCommentForReply] = useState<ReviewCommentResponse | null>(null)
  const [isExportingComments, setIsExportingComments] = useState(false)

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
    onError: (error: AxiosError<ApiErrorPayload>) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.dialogs.comment.failed'),
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
    onError: (error: AxiosError<ApiErrorPayload>) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.dialogs.reply.failed'),
        variant: 'destructive',
      })
    },
  })

  const resolveCommentMutation = useMutation({
    mutationFn: async (commentId: string) => {
      return api.post(`/reviews/comments/${commentId}/resolve`)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.actions.resolved') })
      queryClient.invalidateQueries({ queryKey: ['protocol-comments', protocolId] })
    },
    onError: (error: AxiosError<ApiErrorPayload>) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('common.error'),
        variant: 'destructive',
      })
    },
  })

  const reviewerAnonymousMap = useMemo(() => {
    if (!comments) return new Map<string, string>()
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
  }, [comments, protocolId])

  const getReviewerDisplayName = (comment: ReviewCommentResponse) => {
    if (comment.parent_comment_id && (comment.replied_by === protocol.pi_user_id || comment.replied_by_name)) {
      return comment.replied_by_name || piName || t('common.user')
    }
    if (!shouldAnonymizeReviewers) {
      return comment.reviewer_name || comment.reviewer_email || comment.replied_by_name || comment.replied_by_email
    }
    return reviewerAnonymousMap.get(comment.reviewer_id) || t('protocols.detail.actions.reviewer')
  }

  const handleAddComment = () => {
    if (!commentContent.trim() || !versions || versions.length === 0) return
    addCommentMutation.mutate({
      protocol_version_id: versions[0].id,
      content: commentContent.trim(),
    })
  }

  const handleExportPDF = async () => {
    if (!reportRef.current || isExportingComments) return
    try {
      setIsExportingComments(true)
      const [{ default: html2canvas }, { default: jsPDF }] = await Promise.all([
        import('html2canvas'),
        import('jspdf'),
      ])
      const canvas = await html2canvas(reportRef.current, { scale: 2, useCORS: true })
      const imgData = canvas.toDataURL('image/png')
      const pdf = new jsPDF('p', 'mm', 'a4')
      const imgWidth = 210
      const imgHeight = (canvas.height * imgWidth) / canvas.width
      pdf.addImage(imgData, 'PNG', 0, 0, imgWidth, imgHeight)
      pdf.save(`審查意見回覆表_${protocol.protocol_no || protocolId}.pdf`)
      toast({ title: t('common.exportSuccess'), variant: 'default' })
    } catch (error) {
      logger.error('PDF export error:', error)
      toast({ title: t('common.exportFailed'), variant: 'destructive' })
    } finally {
      setIsExportingComments(false)
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
            <div className="space-y-4">
              {comments
                .filter(c => !c.parent_comment_id)
                .map((comment) => (
                  <div key={comment.id} className="space-y-2">
                    <div
                      className={`p-4 rounded-lg border max-w-[85%] mr-auto ${comment.is_resolved ? 'bg-green-50 border-green-200' : 'bg-white border-slate-200'}`}
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex items-center gap-2">
                          <div className="h-8 w-8 rounded-full bg-blue-100 flex items-center justify-center">
                            <UserIcon className="h-4 w-4 text-blue-600" />
                          </div>
                          <div>
                            <p className="font-medium">{getReviewerDisplayName(comment)}</p>
                            <p className="text-xs text-muted-foreground">{formatDateTime(comment.created_at)}</p>
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          {canReply && !comment.is_resolved && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => {
                                setSelectedCommentForReply(comment)
                                setShowReplyDialog(true)
                              }}
                            >
                              <Reply className="mr-1 h-4 w-4" />
                              {t('protocols.detail.actions.reply')}
                            </Button>
                          )}
                          {comment.is_resolved ? (
                            <Badge variant="success" className="flex items-center gap-1">
                              <CheckCircle2 className="h-3 w-3" />
                              {t('protocols.detail.actions.resolved')}
                            </Badge>
                          ) : (
                            user?.id === comment.reviewer_id && (
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() => resolveCommentMutation.mutate(comment.id)}
                                disabled={resolveCommentMutation.isPending}
                              >
                                {resolveCommentMutation.isPending ? (
                                  <Loader2 className="h-4 w-4 animate-spin" />
                                ) : (
                                  <>
                                    <CheckCircle className="mr-1 h-4 w-4" />
                                    {t('protocols.detail.actions.markResolved')}
                                  </>
                                )}
                              </Button>
                            )
                          )}
                        </div>
                      </div>
                      <p className="mt-3 text-sm text-slate-700 whitespace-pre-wrap">{comment.content}</p>
                    </div>

                    {comments
                      .filter(reply => reply.parent_comment_id === comment.id)
                      .map(reply => (
                        <div key={reply.id} className="ml-auto max-w-[85%] p-4 rounded-lg border bg-slate-50 border-slate-200">
                          <div className="flex items-center gap-2 justify-end">
                            <div className="text-right">
                              <p className="font-medium text-sm">{getReviewerDisplayName(reply)}</p>
                              <p className="text-xs text-muted-foreground">{formatDateTime(reply.created_at)}</p>
                            </div>
                            <div className="h-6 w-6 rounded-full bg-green-100 flex items-center justify-center">
                              <Reply className="h-3 w-3 text-green-600" />
                            </div>
                          </div>
                          <p className="mt-2 text-sm text-slate-700 whitespace-pre-wrap text-right">{reply.content}</p>
                        </div>
                      ))}
                  </div>
                ))}
            </div>
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
