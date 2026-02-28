import React, { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  AssignCoEditorRequest,
  CoEditorAssignmentResponse,
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
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.tables.assignCoeditorFailed'),
        variant: 'destructive',
      })
    },
  })

  const removeCoEditorMutation = useMutation({
    mutationFn: async (userId: string) => {
      return api.delete(`/protocols/${protocolId}/co-editors/${userId}`)
    },
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.detail.actions.removeCoeditorSuccess') })
      queryClient.invalidateQueries({ queryKey: ['protocol-co-editors', protocolId] })
    },
    onError: (error: any) => {
      toast({
        title: t('common.error'),
        description: error?.response?.data?.error?.message || t('protocols.detail.actions.removeCoeditorFailed'),
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
          {coEditorsLoading ? (
            <div className="flex justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : coEditors && coEditors.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t('protocols.detail.tabs.coeditors')}</TableHead>
                  <TableHead>{t('protocols.detail.tables.assignedTime')}</TableHead>
                  <TableHead>{t('protocols.detail.tables.assignedBy')}</TableHead>
                  <TableHead>{t('protocols.detail.tables.actions')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {coEditors.map((coEditor) => (
                  <TableRow key={coEditor.user_id}>
                    <TableCell>
                      <div>
                        <p className="font-medium">{coEditor.user_name}</p>
                        <p className="text-sm text-muted-foreground">{coEditor.user_email}</p>
                      </div>
                    </TableCell>
                    <TableCell>{formatDateTime(coEditor.granted_at)}</TableCell>
                    <TableCell>{coEditor.granted_by_name || '-'}</TableCell>
                    <TableCell>
                      {canAssignReviewer && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={async () => {
                            const ok = await confirm({
                              title: '移除協作者',
                              description: t('protocols.detail.actions.removeCoeditorConfirm'),
                              variant: 'destructive',
                              confirmLabel: '確認移除',
                            })
                            if (ok) {
                              removeCoEditorMutation.mutate(coEditor.user_id)
                            }
                          }}
                          disabled={removeCoEditorMutation.isPending}
                        >
                          {removeCoEditorMutation.isPending ? (
                            <Loader2 className="h-4 w-4 animate-spin" />
                          ) : (
                            <Trash2 className="h-4 w-4 text-destructive" />
                          )}
                        </Button>
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              <UserPlus className="h-12 w-12 mx-auto mb-2" />
              <p>{t('protocols.detail.tables.noCoeditors')}</p>
              <p className="text-sm mt-2">{t('protocols.detail.tables.coeditorHint')}</p>
            </div>
          )}
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
