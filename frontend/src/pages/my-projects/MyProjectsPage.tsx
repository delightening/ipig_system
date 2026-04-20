import { Link } from 'react-router-dom'
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { ProtocolListItem, ProtocolStatus } from '@/lib/api'
import { useTableSort } from '@/hooks/useTableSort'
import { Badge } from '@/components/ui/badge'
import { PageHeader } from '@/components/ui/page-header'
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
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { formatDate } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'
import { useTranslation } from 'react-i18next'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { RoleWelcomeGuide } from '@/components/dashboard'
import { GuestHide } from '@/components/ui/guest-hide'
import { useAuthStore } from '@/stores/auth'

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
  const isGuest = useAuthStore((s) => s.isGuest)()
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
      <Card className="overflow-hidden">
        <CardHeader>
          <CardTitle>{t('dashboard.widgets.names.my_projects')}</CardTitle>
          <CardDescription>{t('dashboard.widgets.projects.description')}</CardDescription>
        </CardHeader>
        <CardContent>
          {(() => {
            const renderActions = (project: ProtocolListItem) => (
              <>
                <Button variant="outline" size="sm" asChild>
                  <Link to={`/my-projects/${project.id}`}>
                    <Eye className="mr-2 h-4 w-4" />
                    {t('common.view')}
                  </Link>
                </Button>
                <GuestHide>
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
                </GuestHide>
              </>
            )

            return (
              <div className="@container">
                <div className="hidden @[600px]:block overflow-x-auto">
                  <Table>
                    <TableHeader>
                      <TableRow className="bg-muted/50 hover:bg-muted/50">
                        <SortableTableHead sortKey="iacuc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                          {t('protocols.columns.iacucNo')}
                        </SortableTableHead>
                        <SortableTableHead sortKey="status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                          {t('protocols.columns.status')}
                        </SortableTableHead>
                        <SortableTableHead className="hidden @[800px]:table-cell" sortKey="pi_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                          {t('aup.basic.contactPerson')}
                        </SortableTableHead>
                        <SortableTableHead className="hidden @[1100px]:table-cell" sortKey="pi_organization" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                          {t('aup.basic.organizationName')}
                        </SortableTableHead>
                        <TableHead className="hidden @[950px]:table-cell">{t('protocols.columns.status')}</TableHead>
                        <SortableTableHead sortKey="title" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                          {t('protocols.columns.protocolTitle')}
                        </SortableTableHead>
                        <SortableTableHead className="hidden @[1100px]:table-cell" sortKey="start_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                          {t('protocols.columns.period')}
                        </SortableTableHead>
                        <TableHead className="text-right">{t('protocols.columns.actions')}</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {isLoading ? (
                        <TableRow><TableCell colSpan={8} className="p-0"><TableSkeleton rows={5} cols={8} /></TableCell></TableRow>
                      ) : sortedProjects && sortedProjects.length > 0 ? (
                        sortedProjects.map((project) => {
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
                              <TableCell className="hidden @[800px]:table-cell">{project.pi_name}</TableCell>
                              <TableCell className="hidden @[1100px]:table-cell">{project.pi_organization || '-'}</TableCell>
                              <TableCell className="hidden @[950px]:table-cell">{getStatusBadge(project.status)}</TableCell>
                              <TableCell className="max-w-[200px] whitespace-normal break-words">
                                <div title={project.title}>
                                  {project.title}
                                </div>
                              </TableCell>
                              <TableCell className="hidden @[1100px]:table-cell">
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
                                  {renderActions(project)}
                                </div>
                              </TableCell>
                            </TableRow>
                          )
                        })
                      ) : (
                        <TableEmptyRow
                          colSpan={8}
                          icon={FileText}
                          title={t('myProjects.welcome.title', '歡迎使用計畫管理')}
                          description={t('myProjects.welcome.description', '您目前尚無任何計畫書，建立第一個動物使用計畫書以開始使用。')}
                        />
                      )}
                    </TableBody>
                  </Table>
                </div>

                <div className="@[600px]:hidden divide-y rounded-lg border">
                  {isLoading ? (
                    <div className="p-3"><TableSkeleton rows={3} cols={1} /></div>
                  ) : sortedProjects && sortedProjects.length > 0 ? (
                    sortedProjects.map((project) => {
                      const projectStatus = getProjectStatus(project.status)
                      return (
                        <div key={project.id} className="p-3 space-y-2">
                          <div className="flex items-start justify-between gap-2">
                            <div className="min-w-0 flex-1">
                              <div className="font-mono text-sm text-status-warning-text font-semibold">
                                {project.iacuc_no || '-'}
                              </div>
                              <div className="font-medium line-clamp-2" title={project.title}>
                                {project.title}
                              </div>
                            </div>
                            <Badge variant={projectStatus.color} className="shrink-0">{projectStatus.label}</Badge>
                          </div>
                          <div className="text-xs text-muted-foreground space-y-0.5">
                            <div>{project.pi_name}{project.pi_organization && ` · ${project.pi_organization}`}</div>
                            <div className="flex items-center gap-1">
                              狀態：{getStatusBadge(project.status)}
                            </div>
                            {project.start_date && project.end_date && (
                              <div className="flex items-center gap-1">
                                <Calendar className="h-3 w-3" />
                                {formatDate(project.start_date)} ~ {formatDate(project.end_date)}
                              </div>
                            )}
                          </div>
                          <div className="flex flex-wrap justify-end gap-2 pt-1 border-t">
                            {renderActions(project)}
                          </div>
                        </div>
                      )
                    })
                  ) : (
                    <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                      <FileText className="h-8 w-8" />
                      <p className="text-sm font-medium">{t('myProjects.welcome.title', '歡迎使用計畫管理')}</p>
                      <p className="text-xs">{t('myProjects.welcome.description', '您目前尚無任何計畫書，建立第一個動物使用計畫書以開始使用。')}</p>
                    </div>
                  )}
                </div>
              </div>
            )
          })()}
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
