import React, { useState } from 'react'
import { useDialogSet } from '@/hooks/useDialogSet'
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
import { getApiErrorMessage } from '@/lib/validation'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import VetReviewForm from '@/components/protocol/VetReviewForm'
import type { VetReviewAssignment } from '@/types/aup'

interface ReviewersTabProps {
  protocolId: string
  protocolStatus: ProtocolStatus
  vetReview?: VetReviewAssignment
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
  const dialogs = useDialogSet(['assign'] as const)
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

  const { sortedData: sortedReviewers, sort, toggleSort } = useTableSort(reviewers)

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
    enabled: dialogs.isOpen('assign'),
  })

  const assignReviewerMutation = useMutation({
    mutationFn: async (data: AssignReviewerRequest) => {
      return api.post('/reviews/assignments', data)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.tables.assignSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol-reviewers', protocolId] })
      dialogs.close('assign')
      setSelectedReviewerId('')
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.tables.assignFailed')),
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
            <Button onClick={() => dialogs.open('assign')}>
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
              <div className="mt-4 p-4 bg-status-warning-bg border border-amber-100 rounded-lg flex gap-3 text-status-warning-text text-sm">
                <AlertTriangle className="h-5 w-5 shrink-0" />
                <p>提示：此表格內容將會自動同步至「審查報告 (PDF)」中。請確保在計畫核准前完成填寫。</p>
              </div>
            </div>
          )}

          <div className="@container">
            <div className="hidden @[600px]:block overflow-x-auto">
              <Table className="w-full" style={{ minWidth: 470 }}>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <SortableTableHead style={{ minWidth: 180 }} sortKey="reviewer_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.detail.tables.reviewer')}</SortableTableHead>
                    <SortableTableHead style={{ width: 160 }} sortKey="assigned_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="hidden @[780px]:table-cell">{t('protocols.detail.tables.assignedTime')}</SortableTableHead>
                    <TableHead style={{ width: 100 }} className="hidden @[780px]:table-cell">{t('protocols.detail.tables.assignedBy')}</TableHead>
                    <TableHead style={{ width: 110 }}>{t('protocols.detail.tables.commentStatus') || '意見狀態'}</TableHead>
                    <SortableTableHead style={{ width: 180 }} sortKey="completed_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="hidden @[780px]:table-cell">{t('protocols.detail.tables.completedTime')}</SortableTableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {reviewersLoading ? (
                    <TableRow>
                      <TableCell colSpan={5} className="p-0">
                        <TableSkeleton rows={3} cols={5} />
                      </TableCell>
                    </TableRow>
                  ) : sortedReviewers && sortedReviewers.length > 0 ? (
                    sortedReviewers.map((reviewer) => {
                      const hasComment = comments?.some(
                        c => c.reviewer_id === reviewer.reviewer_id && !c.parent_comment_id
                      ) || false

                      return (
                        <TableRow key={reviewer.id}>
                          <TableCell style={{ minWidth: 180 }}>
                            <div>
                              <p className="font-medium">{reviewer.reviewer_name || '-'}</p>
                              <p className="text-sm text-muted-foreground break-all">{reviewer.reviewer_email}</p>
                            </div>
                          </TableCell>
                          <TableCell style={{ width: 160 }} className="text-xs text-muted-foreground hidden @[780px]:table-cell">{formatDateTime(reviewer.assigned_at)}</TableCell>
                          <TableCell style={{ width: 100 }} className="whitespace-normal break-words hidden @[780px]:table-cell">{reviewer.assigned_by_name || '-'}</TableCell>
                          <TableCell style={{ width: 110 }}>
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
                          <TableCell style={{ width: 180 }} className="hidden @[780px]:table-cell">
                            {reviewer.completed_at ? (
                              <Badge variant="success">{formatDateTime(reviewer.completed_at)}</Badge>
                            ) : (
                              <Badge variant="secondary">{t('protocols.detail.tables.reviewing')}</Badge>
                            )}
                          </TableCell>
                        </TableRow>
                      )
                    })
                  ) : (
                    <TableEmptyRow colSpan={5} icon={Users} title={t('protocols.detail.tables.noReviewers')} />
                  )}
                </TableBody>
              </Table>
            </div>

            <div className="@[600px]:hidden space-y-3 py-1">
              {reviewersLoading ? (
                <TableSkeleton rows={3} cols={1} />
              ) : sortedReviewers && sortedReviewers.length > 0 ? (
                sortedReviewers.map((reviewer) => {
                  const hasComment = comments?.some(
                    c => c.reviewer_id === reviewer.reviewer_id && !c.parent_comment_id
                  ) || false
                  return (
                    <div key={reviewer.id} className="rounded-lg border bg-card p-3 space-y-2">
                      <div>
                        <p className="text-sm font-medium text-foreground">{reviewer.reviewer_name || '-'}</p>
                        <p className="text-xs text-muted-foreground break-all">{reviewer.reviewer_email}</p>
                      </div>
                      <div className="flex flex-wrap items-center gap-2 pt-1 border-t">
                        {hasComment ? (
                          <Badge variant="success" className="flex items-center gap-1">
                            <CheckCircle className="h-3 w-3" />
                            {t('protocols.detail.tables.commented') || '已發表'}
                          </Badge>
                        ) : reviewer.is_primary_reviewer ? (
                          <Badge variant="destructive" className="flex items-center gap-1">
                            <AlertTriangle className="h-3 w-3" />
                            {t('protocols.detail.tables.pendingComment') || '待發表'}
                          </Badge>
                        ) : (
                          <Badge variant="secondary">{t('protocols.detail.tables.optional') || '選填'}</Badge>
                        )}
                        {reviewer.completed_at ? (
                          <Badge variant="success">{formatDateTime(reviewer.completed_at)}</Badge>
                        ) : (
                          <Badge variant="secondary">{t('protocols.detail.tables.reviewing')}</Badge>
                        )}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        指派：{formatDateTime(reviewer.assigned_at)}
                        {reviewer.assigned_by_name && <> · {reviewer.assigned_by_name}</>}
                      </div>
                    </div>
                  )
                })
              ) : (
                <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                  <Users className="h-8 w-8" />
                  <p className="text-sm">{t('protocols.detail.tables.noReviewers')}</p>
                </div>
              )}
            </div>

          </div>
        </CardContent>
      </Card>

      <Dialog open={dialogs.isOpen('assign')} onOpenChange={dialogs.setOpen('assign')}>
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
            <Button variant="outline" onClick={() => dialogs.close('assign')}>
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
