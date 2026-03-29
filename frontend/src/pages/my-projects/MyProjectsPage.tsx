import { Link } from 'react-router-dom'
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { ProtocolListItem, ProtocolStatus } from '@/lib/api'
import { useTableSort } from '@/hooks/useTableSort'
import { Badge } from '@/components/ui/badge'
import { PageHeader } from '@/components/ui/page-header'
import { EmptyState } from '@/components/ui/empty-state'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useToast } from '@/components/ui/use-toast'
import { Eye, Loader2, FileText, Calendar, X } from 'lucide-react'
import { formatDate } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'
import { useTranslation } from 'react-i18next'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { RoleWelcomeGuide } from '@/components/dashboard'

const statusColors: Record<ProtocolStatus, 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline'> = {
  DRAFT: 'secondary',
  SUBMITTED: 'default',
  PRE_REVIEW: 'default',
  PRE_REVIEW_REVISION_REQUIRED: 'destructive',
  VET_REVIEW: 'warning',
  VET_REVISION_REQUIRED: 'destructive',
  UNDER_REVIEW: 'warning',
  REVISION_REQUIRED: 'destructive',
  RESUBMITTED: 'default',
  APPROVED: 'success',
  APPROVED_WITH_CONDITIONS: 'success',
  DEFERRED: 'secondary',
  REJECTED: 'destructive',
  SUSPENDED: 'destructive',
  CLOSED: 'outline',
  DELETED: 'destructive',
}

export function MyProjectsPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { toast } = useToast()
  const { dialogState, confirm } = useConfirmDialog()
  const [closeDialogOpen, setCloseDialogOpen] = useState(false)
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null)

  const { data: projects, isLoading } = useQuery({
    queryKey: ['my-projects'],
    queryFn: async () => {
      const response = await api.get<ProtocolListItem[]>('/my-projects')
      return response.data
    },
  })

  // 結案 mutation
  const closeProtocolMutation = useMutation({
    mutationFn: async (projectId: string) => {
      return api.post(`/protocols/${projectId}/status`, {
        to_status: 'CLOSED',
        remark: '計畫結案',
      })
    },
    onSuccess: () => {
      toast({
        title: t('common.success'),
        description: t('common.saved'),
      })
      queryClient.invalidateQueries({ queryKey: ['my-projects'] })
      setCloseDialogOpen(false)
      setSelectedProjectId(null)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('common.error')),
        variant: 'destructive',
      })
    },
  })

  // 刪除 mutation
  const deleteProtocolMutation = useMutation({
    mutationFn: async (projectId: string) => {
      return api.post(`/protocols/${projectId}/status`, {
        to_status: 'DELETED',
        remark: '使用者刪除計畫',
      })
    },
    onSuccess: () => {
      toast({
        title: t('common.success'),
        description: t('common.deleted'),
      })
      queryClient.invalidateQueries({ queryKey: ['my-projects'] })
      setSelectedProjectId(null)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('common.error')),
        variant: 'destructive',
      })
    },
  })

  const handleCloseClick = (projectId: string) => {
    setSelectedProjectId(projectId)
    setCloseDialogOpen(true)
  }

  const handleDeleteClick = async (projectId: string) => {
    const project = projects?.find(p => p.id === projectId)
    const ok = await confirm({ title: t('protocols.deleteTitle'), description: t('protocols.deleteConfirm', { title: project?.title || '' }), variant: 'destructive', confirmLabel: t('common.delete') })
    if (ok) deleteProtocolMutation.mutate(projectId)
  }

  const handleConfirmClose = () => {
    if (selectedProjectId) {
      closeProtocolMutation.mutate(selectedProjectId)
    }
  }

  const getStatusBadge = (status: ProtocolStatus) => {
    return (
      <Badge variant={statusColors[status]}>
        {t(`protocols.status.${status}`)}
      </Badge>
    )
  }

  // 計算計畫狀態：申請中、進行中、已結案
  const getProjectStatus = (status: ProtocolStatus): { label: string; color: 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline' } => {
    if (status === 'CLOSED') {
      return { label: t('dashboard.projectStats.closed'), color: 'outline' }
    } else if (status === 'APPROVED' || status === 'APPROVED_WITH_CONDITIONS') {
      return { label: t('dashboard.projectStats.ongoing'), color: 'success' }
    } else {
      return { label: t('dashboard.projectStats.applying'), color: 'warning' }
    }
  }

  // 排序函數：申請中(1) -> 進行中(2) -> 已結案(3)
  const getStatusSortOrder = (status: ProtocolStatus): number => {
    if (status === 'CLOSED') {
      return 3 // 已結案
    } else if (status === 'APPROVED' || status === 'APPROVED_WITH_CONDITIONS') {
      return 2 // 進行中
    } else {
      return 1 // 申請中
    }
  }

  // 對計劃列表進行排序（預設按狀態分組）
  const statusSortedProjects = projects ? [...projects].sort((a, b) => {
    const orderA = getStatusSortOrder(a.status)
    const orderB = getStatusSortOrder(b.status)
    return orderA - orderB
  }) : undefined

  const { sortedData: sortedProjects, sort, toggleSort } = useTableSort(statusSortedProjects)

  // 統計數據
  const stats = {
    total: projects?.length || 0,
    approved: projects?.filter(p => p.status === 'APPROVED' || p.status === 'APPROVED_WITH_CONDITIONS').length || 0,
    underReview: projects?.filter(p => ['SUBMITTED', 'PRE_REVIEW', 'UNDER_REVIEW', 'RESUBMITTED'].includes(p.status)).length || 0,
    draft: projects?.filter(p => p.status === 'DRAFT').length || 0,
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('nav.myProjects')}
        description={t('dashboard.widgets.projects.description')}
      />

      <RoleWelcomeGuide />

      {/* 統計卡片 */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">{t('dashboard.widgets.projects.total')}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-status-success-text">{t('protocols.status.APPROVED')}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-status-success-text">{stats.approved}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-status-warning-text">{t('dashboard.widgets.projects.reviewing')}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-status-warning-text">{stats.underReview}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">{t('protocols.status.DRAFT')}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-muted-foreground">{stats.draft}</div>
          </CardContent>
        </Card>
      </div>

      {/* 計劃列表 */}
      <Card>
        <CardHeader>
          <CardTitle>{t('dashboard.widgets.names.my_projects')}</CardTitle>
          <CardDescription>{t('dashboard.widgets.projects.description')}</CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : sortedProjects && sortedProjects.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <SortableTableHead sortKey="iacuc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                    {t('protocols.columns.iacucNo')}
                  </SortableTableHead>
                  <SortableTableHead sortKey="status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                    {t('protocols.columns.status')}
                  </SortableTableHead>
                  <SortableTableHead sortKey="pi_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                    {t('aup.basic.contactPerson')}
                  </SortableTableHead>
                  <SortableTableHead sortKey="pi_organization" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                    {t('aup.basic.organizationName')}
                  </SortableTableHead>
                  <TableHead>{t('protocols.columns.status')}</TableHead>
                  <SortableTableHead sortKey="title" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                    {t('protocols.columns.protocolTitle')}
                  </SortableTableHead>
                  <SortableTableHead sortKey="start_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                    {t('protocols.columns.period')}
                  </SortableTableHead>
                  <TableHead className="text-right">{t('protocols.columns.actions')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sortedProjects.map((project) => {
                  const projectStatus = getProjectStatus(project.status)

                  return (
                    <TableRow key={project.id}>
                      <TableCell className="font-mono text-status-warning-text font-semibold">
                        {project.iacuc_no || '-'}
                      </TableCell>
                      <TableCell>
                        <Badge variant={projectStatus.color}>
                          {projectStatus.label}
                        </Badge>
                      </TableCell>
                      <TableCell>{project.pi_name}</TableCell>
                      <TableCell>{project.pi_organization || '-'}</TableCell>
                      <TableCell>{getStatusBadge(project.status)}</TableCell>
                      <TableCell className="max-w-[200px]">
                        <div className="truncate" title={project.title}>
                          {project.title}
                        </div>
                      </TableCell>
                      <TableCell>
                        {project.start_date && project.end_date ? (
                          <div className="flex items-center gap-1 text-sm">
                            <Calendar className="h-3 w-3" />
                            {formatDate(project.start_date)} ~ {formatDate(project.end_date)}
                          </div>
                        ) : (
                          '-'
                        )}
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex items-center justify-end gap-2">
                          <Button variant="outline" size="sm" asChild>
                            <Link to={`/my-projects/${project.id}`}>
                              <Eye className="mr-2 h-4 w-4" />
                              {t('common.view')}
                            </Link>
                          </Button>
                          {/* 結案按鈕: 審查委員不應有結案功能 */}
                          {/* 只有草稿或需修訂狀態可刪除 */}
                          {['DRAFT', 'REVISION_REQUIRED'].includes(project.status) && (
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => handleDeleteClick(project.id)}
                              disabled={deleteProtocolMutation.isPending}
                            >
                              <X className="mr-2 h-4 w-4" />
                              {t('common.delete')}
                            </Button>
                          )}
                          {/* 只有核准狀態可結案 */}
                          {(project.status === 'APPROVED' || project.status === 'APPROVED_WITH_CONDITIONS') && (
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => handleCloseClick(project.id)}
                              disabled={closeProtocolMutation.isPending}
                            >
                              <X className="mr-2 h-4 w-4" />
                              {t('common.close')}
                            </Button>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  )
                })}
              </TableBody>
            </Table>
          ) : (
            <EmptyState
              icon={FileText}
              title={t('myProjects.welcome.title', '歡迎使用計畫管理')}
              description={t('myProjects.welcome.description', '您目前尚無任何計畫書，建立第一個動物使用計畫書以開始使用。')}
              action={{
                label: t('myProjects.welcome.createFirst', '建立計畫書'),
                onClick: () => window.location.href = '/protocols/new',
              }}
            />
          )}
        </CardContent>
      </Card>

      {/* 結案確認對話框 */}
      <Dialog open={closeDialogOpen} onOpenChange={setCloseDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('common.closeTitle')}</DialogTitle>
            <DialogDescription>
              {t('common.closeDescription')}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setCloseDialogOpen(false)}
              disabled={closeProtocolMutation.isPending}
            >
              {t('common.cancel')}
            </Button>
            <Button
              variant="default"
              onClick={handleConfirmClose}
              disabled={closeProtocolMutation.isPending}
            >
              {closeProtocolMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  {t('common.processed')}
                </>
              ) : (
                t('common.confirm')
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
