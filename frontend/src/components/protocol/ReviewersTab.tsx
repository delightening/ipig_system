import React, { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  ReviewAssignmentResponse,
  ReviewCommentResponse,
  AssignReviewerRequest,
  User,
  ProtocolStatus,
} from '@/lib/api'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import {
  AlertTriangle,
  CheckCircle,
  ClipboardList,
  Loader2,
  UserPlus,
  Users,
} from 'lucide-react'
import { formatDateTime } from '@/lib/utils'
import VetReviewForm from '@/components/protocol/VetReviewForm'

interface ReviewersTabProps {
  protocolId: string
  protocolStatus: ProtocolStatus
  vetReview?: any
  isVetReviewer: boolean
  canAssignReviewer: boolean
}

export function ReviewersTab({
  protocolId,
  protocolStatus,
  vetReview,
  isVetReviewer,
  canAssignReviewer,
}: ReviewersTabProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [showAssignDialog, setShowAssignDialog] = useState(false)
  const [selectedReviewerId, setSelectedReviewerId] = useState('')

  const { data: reviewers, isLoading: reviewersLoading } = useQuery({
    queryKey: ['protocol-reviewers', protocolId],
    queryFn: async () => {
      const response = await api.get<ReviewAssignmentResponse[]>('/reviews/assignments', {
        params: { protocol_id: protocolId },
      })
      return response.data
    },
    enabled: !!protocolId,
  })

  const { data: comments } = useQuery({
    queryKey: ['protocol-comments', protocolId],
    queryFn: async () => {
      const response = await api.get<ReviewCommentResponse[]>('/reviews/comments', {
        params: { protocol_id: protocolId },
      })
      return response.data
    },
    enabled: !!protocolId,
  })

  const { data: availableReviewers } = useQuery({
    queryKey: ['available-reviewers'],
    queryFn: async () => {
      const response = await api.get<User[]>('/users')
      return response.data
        .filter(user => user.roles?.some(role => ['REVIEWER', 'VET'].includes(role)))
        .map(user => ({
          id: user.id,
          email: user.email,
          display_name: user.display_name || user.email,
        }))
    },
    enabled: showAssignDialog,
  })

  const assignReviewerMutation = useMutation({
    mutationFn: async (data: AssignReviewerRequest) => {
      return api.post('/reviews/assignments', data)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.tables.assignSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol-reviewers', protocolId] })
      setShowAssignDialog(false)
      setSelectedReviewerId('')
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.tables.assignFailed'),
        variant: 'destructive',
      })
    },
  })

  const handleAssignReviewer = () => {
    if (!selectedReviewerId || !protocolId) return
    assignReviewerMutation.mutate({
      protocol_id: protocolId,
      reviewer_id: selectedReviewerId,
    })
  }

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>{t('protocols.detail.sections.reviewersTitle')}</CardTitle>
            <CardDescription>{t('protocols.detail.sections.reviewersDesc')}</CardDescription>
          </div>
          {canAssignReviewer && protocolStatus !== 'DRAFT' && protocolStatus !== 'CLOSED' && (
            <Button onClick={() => setShowAssignDialog(true)}>
              <UserPlus className="mr-2 h-4 w-4" />
              {t('protocols.detail.dialogs.assign.title')}
            </Button>
          )}
        </CardHeader>
        <CardContent className="space-y-6">
          {isVetReviewer && (protocolStatus === 'VET_REVIEW' || protocolStatus === 'VET_REVISION_REQUIRED' || protocolStatus === 'UNDER_REVIEW') && (
            <div className="mb-8">
              <div className="flex items-center gap-2 mb-4 text-primary font-semibold border-l-4 border-primary pl-3">
                <ClipboardList className="h-5 w-5" />
                <span>{t('protocols.detail.sections.vetFormFill', '獸醫師線上審查填寫')}</span>
              </div>
              <VetReviewForm
                protocolId={protocolId}
                initialData={vetReview?.review_form}
                isEditable={protocolStatus === 'VET_REVIEW' || protocolStatus === 'VET_REVISION_REQUIRED'}
              />
              <div className="mt-4 p-4 bg-amber-50 border border-amber-100 rounded-lg flex gap-3 text-amber-800 text-sm">
                <AlertTriangle className="h-5 w-5 shrink-0" />
                <p>提示：此表格內容將會自動同步至「審查報告 (PDF)」中。請確保在計畫核准前完成填寫。</p>
              </div>
            </div>
          )}

          {reviewersLoading ? (
            <div className="flex justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : reviewers && reviewers.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t('protocols.detail.tables.reviewer')}</TableHead>
                  <TableHead>{t('protocols.detail.tables.assignedTime')}</TableHead>
                  <TableHead>{t('protocols.detail.tables.assignedBy')}</TableHead>
                  <TableHead>{t('protocols.detail.tables.commentStatus') || '意見狀態'}</TableHead>
                  <TableHead>{t('protocols.detail.tables.completedTime')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {reviewers.map((reviewer) => {
                  const hasComment = comments?.some(
                    c => c.reviewer_id === reviewer.reviewer_id && !c.parent_comment_id
                  ) || false

                  return (
                    <TableRow key={reviewer.id}>
                      <TableCell>
                        <div>
                          <p className="font-medium">{reviewer.reviewer_name || '-'}</p>
                          <p className="text-sm text-muted-foreground">{reviewer.reviewer_email}</p>
                        </div>
                      </TableCell>
                      <TableCell>{formatDateTime(reviewer.assigned_at)}</TableCell>
                      <TableCell>{reviewer.assigned_by_name || '-'}</TableCell>
                      <TableCell>
                        {hasComment ? (
                          <Badge variant="success" className="flex items-center gap-1 w-fit">
                            <CheckCircle className="h-3 w-3" />
                            {t('protocols.detail.tables.commented') || '已發表'}
                          </Badge>
                        ) : reviewer.is_primary_reviewer ? (
                          <Badge variant="destructive" className="flex items-center gap-1 w-fit">
                            <AlertTriangle className="h-3 w-3" />
                            {t('protocols.detail.tables.pendingComment') || '待發表'}
                          </Badge>
                        ) : (
                          <Badge variant="secondary" className="w-fit">
                            {t('protocols.detail.tables.optional') || '選填'}
                          </Badge>
                        )}
                      </TableCell>
                      <TableCell>
                        {reviewer.completed_at ? (
                          <Badge variant="success">{formatDateTime(reviewer.completed_at)}</Badge>
                        ) : (
                          <Badge variant="secondary">{t('protocols.detail.tables.reviewing')}</Badge>
                        )}
                      </TableCell>
                    </TableRow>
                  )
                })}
              </TableBody>
            </Table>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              <Users className="h-12 w-12 mx-auto mb-2" />
              <p>{t('protocols.detail.tables.noReviewers')}</p>
              {canAssignReviewer && protocolStatus !== 'DRAFT' && (
                <Button variant="link" onClick={() => setShowAssignDialog(true)} className="mt-2">
                  {t('protocols.detail.tables.assignFirst')}
                </Button>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog open={showAssignDialog} onOpenChange={setShowAssignDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('protocols.detail.dialogs.assign.title')}</DialogTitle>
            <DialogDescription>{t('protocols.detail.dialogs.assign.desc')}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.assign.placeholder')}</Label>
              <Select value={selectedReviewerId} onValueChange={setSelectedReviewerId}>
                <SelectTrigger>
                  <SelectValue placeholder={t('protocols.detail.dialogs.assign.placeholder')} />
                </SelectTrigger>
                <SelectContent>
                  {availableReviewers?.map((reviewer) => (
                    <SelectItem key={reviewer.id} value={reviewer.id}>
                      {reviewer.display_name || reviewer.email}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAssignDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={handleAssignReviewer}
              disabled={!selectedReviewerId || assignReviewerMutation.isPending}
            >
              {assignReviewerMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t('protocols.detail.dialogs.assign.confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
