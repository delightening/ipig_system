import React, { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  AssignCoEditorRequest,
  CoEditorAssignmentResponse,
  deleteResource,
  User,
} from '@/lib/api'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
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
import { Loader2, Trash2, UserPlus } from 'lucide-react'
import { formatDateTime } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

interface CoEditorsTabProps {
  protocolId: string
  canAssignReviewer: boolean
}

export function CoEditorsTab({ protocolId, canAssignReviewer }: CoEditorsTabProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()
  const [showCoEditorDialog, setShowCoEditorDialog] = useState(false)
  const [selectedCoEditorId, setSelectedCoEditorId] = useState('')

  const { data: coEditors, isLoading: coEditorsLoading } = useQuery({
    queryKey: ['protocol-co-editors', protocolId],
    queryFn: async () => {
      const response = await api.get<CoEditorAssignmentResponse[]>(`/protocols/${protocolId}/co-editors`)
      return response.data
    },
    enabled: !!protocolId,
  })

  const { sortedData: sortedCoEditors, sort, toggleSort } = useTableSort(coEditors)

  const { data: availableExperimentStaff } = useQuery({
    queryKey: ['available-experiment-staff'],
    queryFn: async () => {
      const response = await api.get<User[]>('/users')
      return response.data
        .filter(user => user.roles?.includes('EXPERIMENT_STAFF'))
        .map(user => ({
          id: user.id,
          email: user.email,
          display_name: user.display_name || user.email,
        }))
    },
    enabled: showCoEditorDialog,
  })

  const assignCoEditorMutation = useMutation({
    mutationFn: async (data: AssignCoEditorRequest) => {
      return api.post(`/protocols/${protocolId}/co-editors`, data)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.tables.assignCoeditorSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol-co-editors', protocolId] })
      setShowCoEditorDialog(false)
      setSelectedCoEditorId('')
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.tables.assignCoeditorFailed')),
        variant: 'destructive',
      })
    },
  })

  const removeCoEditorMutation = useMutation({
    mutationFn: async (userId: string) => {
      return deleteResource(`/protocols/${protocolId}/co-editors/${userId}`)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.actions.removeCoeditorSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol-co-editors', protocolId] })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.detail.actions.removeCoeditorFailed')),
        variant: 'destructive',
      })
    },
  })

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>{t('protocols.detail.tabs.coeditors')}</CardTitle>
            <CardDescription>{t('protocols.detail.sections.coeditorsDesc')}</CardDescription>
          </div>
          {canAssignReviewer && (
            <Button onClick={() => setShowCoEditorDialog(true)}>
              <UserPlus className="mr-2 h-4 w-4" />
              {t('protocols.detail.tables.addCoeditor')}
            </Button>
          )}
        </CardHeader>
        <CardContent>
          <div className="@container">
            <div className="hidden @[600px]:block overflow-x-auto">
              <Table className="w-full" style={{ minWidth: 520 }}>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    <SortableTableHead style={{ minWidth: 180 }} sortKey="user_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.detail.tabs.coeditors')}</SortableTableHead>
                    <SortableTableHead style={{ width: 160 }} sortKey="granted_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.detail.tables.assignedTime')}</SortableTableHead>
                    <TableHead style={{ width: 100 }}>{t('protocols.detail.tables.assignedBy')}</TableHead>
                    <TableHead style={{ width: 80 }} className="text-right">{t('protocols.detail.tables.actions')}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {coEditorsLoading ? (
                    <TableRow>
                      <TableCell colSpan={4} className="p-0">
                        <TableSkeleton rows={3} cols={4} />
                      </TableCell>
                    </TableRow>
                  ) : sortedCoEditors && sortedCoEditors.length > 0 ? (
                    sortedCoEditors.map((coEditor) => (
                      <TableRow key={coEditor.user_id}>
                        <TableCell style={{ minWidth: 180 }}>
                          <div>
                            <p className="font-medium">{coEditor.user_name}</p>
                            <p className="text-sm text-muted-foreground break-all">{coEditor.user_email}</p>
                          </div>
                        </TableCell>
                        <TableCell style={{ width: 160 }} className="text-xs text-muted-foreground">{formatDateTime(coEditor.granted_at)}</TableCell>
                        <TableCell style={{ width: 100 }} className="whitespace-normal break-words">{coEditor.granted_by_name || '-'}</TableCell>
                        <TableCell style={{ width: 80 }}>
                          <div className="flex items-center justify-end gap-1">
                            {canAssignReviewer && (
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={async () => {
                                  const ok = await confirm({
                                    title: '移除協作者',
                                    description: t('protocols.detail.actions.removeCoeditorConfirm'),
                                    variant: 'destructive',
                                    confirmLabel: '確認移除',
                                  })
                                  if (ok) removeCoEditorMutation.mutate(coEditor.user_id)
                                }}
                                disabled={removeCoEditorMutation.isPending}
                                title="移除"
                              >
                                {removeCoEditorMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Trash2 className="h-4 w-4 text-destructive" />}
                              </Button>
                            )}
                          </div>
                        </TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <TableEmptyRow colSpan={4} icon={UserPlus} title={t('protocols.detail.tables.noCoeditors')} />
                  )}
                </TableBody>
              </Table>
            </div>

            <div className="@[600px]:hidden space-y-3 py-1">
              {coEditorsLoading ? (
                <TableSkeleton rows={3} cols={1} />
              ) : sortedCoEditors && sortedCoEditors.length > 0 ? (
                sortedCoEditors.map((coEditor) => (
                  <div key={coEditor.user_id} className="rounded-lg border bg-card p-3 space-y-2">
                    <div>
                      <p className="text-sm font-medium text-foreground">{coEditor.user_name}</p>
                      <p className="text-xs text-muted-foreground break-all">{coEditor.user_email}</p>
                    </div>
                    <div className="flex items-center justify-between gap-2 pt-1 border-t">
                      <span className="text-xs text-muted-foreground">
                        {formatDateTime(coEditor.granted_at)}
                        {coEditor.granted_by_name && <> · {coEditor.granted_by_name}</>}
                      </span>
                      {canAssignReviewer && (
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={async () => {
                            const ok = await confirm({
                              title: '移除協作者',
                              description: t('protocols.detail.actions.removeCoeditorConfirm'),
                              variant: 'destructive',
                              confirmLabel: '確認移除',
                            })
                            if (ok) removeCoEditorMutation.mutate(coEditor.user_id)
                          }}
                          disabled={removeCoEditorMutation.isPending}
                          title="移除"
                        >
                          {removeCoEditorMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : <Trash2 className="h-4 w-4 text-destructive" />}
                        </Button>
                      )}
                    </div>
                  </div>
                ))
              ) : (
                <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                  <UserPlus className="h-8 w-8" />
                  <p className="text-sm">{t('protocols.detail.tables.noCoeditors')}</p>
                </div>
              )}
            </div>

          </div>
        </CardContent>
      </Card>

      <Dialog open={showCoEditorDialog} onOpenChange={setShowCoEditorDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('protocols.detail.dialogs.coeditor.title')}</DialogTitle>
            <DialogDescription>{t('protocols.detail.dialogs.coeditor.desc')}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('protocols.detail.dialogs.coeditor.placeholder')}</Label>
              <Select value={selectedCoEditorId} onValueChange={setSelectedCoEditorId}>
                <SelectTrigger>
                  <SelectValue placeholder={t('protocols.detail.dialogs.coeditor.placeholder')} />
                </SelectTrigger>
                <SelectContent>
                  {availableExperimentStaff?.map((staff) => (
                    <SelectItem key={staff.id} value={staff.id}>
                      {staff.display_name || staff.email}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => {
              setShowCoEditorDialog(false)
              setSelectedCoEditorId('')
            }}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={() => {
                if (selectedCoEditorId && protocolId) {
                  assignCoEditorMutation.mutate({
                    protocol_id: protocolId,
                    user_id: selectedCoEditorId,
                  })
                }
              }}
              disabled={!selectedCoEditorId || assignCoEditorMutation.isPending}
            >
              {assignCoEditorMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t('protocols.detail.dialogs.assign.confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDialog state={dialogState} />
    </>
  )
}
