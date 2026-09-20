import React, { useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { GuestHide } from '@/components/ui/guest-hide'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource, AnimalWeight } from '@/lib/api'
import { uiLocale } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Plus, Edit2, Trash2, Scale, Loader2 } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'

interface WeightsTabProps {
  animalId: string
  earTag: string
  afterParam: string
  weights: AnimalWeight[] | undefined
  hasAdminRole: boolean
  developerMode: boolean
  toggleDeveloperMode: () => void
}

export const WeightsTab = React.memo(function WeightsTab({
  animalId, earTag, afterParam: _afterParam, weights,
  hasAdminRole, developerMode, toggleDeveloperMode,
}: WeightsTabProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { sortedData, sort, toggleSort } = useTableSort(weights)

  const [showAddDialog, setShowAddDialog] = useState(false)
  const [newWeight, setNewWeight] = useState({ measure_date: new Date().toISOString().split('T')[0], weight: '' })
  const [deleteTarget, setDeleteTarget] = useState<number | null>(null)
  const [editTarget, setEditTarget] = useState<AnimalWeight | null>(null)
  const [editForm, setEditForm] = useState({ measure_date: '', weight: '' })

  const openEdit = useCallback((weight: AnimalWeight) => {
    setEditTarget(weight)
    setEditForm({
      measure_date: weight.measure_date.split('T')[0],
      weight: String(weight.weight),
    })
  }, [])

  const addMutation = useMutation({
    mutationFn: async (data: typeof newWeight) => {
      return api.post(`/animals/${animalId}/weights`, {
        measure_date: data.measure_date,
        weight: parseFloat(data.weight),
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-weights', animalId] })
      toast({ title: t('common.success'), description: t('animalRecords.weights.added') })
      setShowAddDialog(false)
      setNewWeight({ measure_date: new Date().toISOString().split('T')[0], weight: '' })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalRecords.shared.createFailed')),
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: number; data: typeof editForm }) => {
      const weight = Number(data.weight)
      if (!Number.isFinite(weight) || weight < 0) {
        throw new Error(t('animalRecords.weights.invalidWeight'))
      }
      return api.put(`/weights/${id}`, {
        measure_date: data.measure_date,
        weight,
      })
    },
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['animal-weights', animalId] })
      toast({ title: t('common.success'), description: t('animalRecords.weights.updated') })
      // 使用者可能在請求進行中切去編輯另一筆；只有目前開啟的仍是同一筆時才清空 dialog
      if (editTarget?.id === variables.id) {
        setEditTarget(null)
        setEditForm({ measure_date: '', weight: '' })
      }
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalRecords.shared.updateFailed')),
        variant: 'destructive',
      })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: number; reason: string }) => {
      return deleteResource(`/weights/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-weights', animalId] })
      toast({ title: t('common.success'), description: t('animalRecords.weights.deleted') })
      setDeleteTarget(null)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('animalRecords.shared.deleteFailed')),
        variant: 'destructive',
      })
    },
  })

  return (
    <>
      <Card className="overflow-hidden">
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>{t('animalDetail.tabs.weights')}</CardTitle>
            <CardDescription>{t('animalRecords.weights.description')}</CardDescription>
          </div>
          <div className="flex items-center gap-2">
            {hasAdminRole && (
              <label className="flex items-center gap-2 text-sm text-muted-foreground cursor-pointer">
                <input
                  type="checkbox"
                  checked={developerMode}
                  onChange={() => toggleDeveloperMode()}
                  className="rounded"
                />
                {t('animalRecords.weights.showSystemNo')}
              </label>
            )}
            <GuestHide>
              <Can permission={PERMISSIONS.ANIMAL_RECORD_CREATE}>
                <Button className="bg-status-purple-solid hover:bg-status-purple-solid/90" onClick={() => setShowAddDialog(true)}>
                  <Plus className="h-4 w-4 mr-2" />
                  {t('animalRecords.shared.addRecord')}
                </Button>
              </Can>
            </GuestHide>
          </div>
        </CardHeader>
        <CardContent>
          <div className="@container">

            {/* Table view: container ≥ 600px */}
            <div className="hidden @[600px]:block overflow-x-auto">
              <Table className="w-full" style={{ minWidth: 380 }}>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    {developerMode && <TableHead style={{ width: 80 }} className="hidden @[620px]:table-cell">{t('animals.systemNo')}</TableHead>}
                    <SortableTableHead style={{ width: 100 }} sortKey="measure_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('animalRecords.weights.measureDate')}</SortableTableHead>
                    <SortableTableHead style={{ width: 90 }} sortKey="weight" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('animalRecords.weights.weightKg')}</SortableTableHead>
                    <TableHead style={{ width: 100 }}>{t('animalRecords.shared.recorder')}</TableHead>
                    <SortableTableHead style={{ width: 160 }} sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="hidden @[620px]:table-cell">{t('animalRecords.shared.createdAt')}</SortableTableHead>
                    <TableHead style={{ width: 90 }} className="text-right">{t('common.actions')}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {!weights || weights.length === 0 ? (
                    <TableEmptyRow colSpan={developerMode ? 6 : 5} icon={Scale} title={t('animalRecords.weights.emptyTitle')} />
                  ) : (
                    sortedData?.map((weight) => (
                      <TableRow key={weight.id} data-record-id={weight.id}>
                        {developerMode && <TableCell style={{ width: 80 }} className="font-mono text-xs text-muted-foreground hidden @[620px]:table-cell">{weight.id}</TableCell>}
                        <TableCell style={{ width: 100 }} className="whitespace-nowrap">{new Date(weight.measure_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })}</TableCell>
                        <TableCell style={{ width: 90 }} className="font-medium">{weight.weight}</TableCell>
                        <TableCell style={{ width: 100 }} className="whitespace-normal break-words">{weight.created_by_name || '-'}</TableCell>
                        <TableCell style={{ width: 160 }} className="text-xs text-muted-foreground hidden @[620px]:table-cell">{new Date(weight.created_at).toLocaleString(uiLocale(), { timeZone: 'Asia/Taipei' })}</TableCell>
                        <TableCell style={{ width: 90 }} className="text-right">
                          <div className="flex items-center justify-end gap-1">
                            <GuestHide>
                              <Can permission={PERMISSIONS.ANIMAL_RECORD_EDIT}>
                                <Button variant="ghost" size="icon" onClick={() => openEdit(weight)} title={t('animalRecords.weights.editTooltip', { id: weight.id })}>
                                  <Edit2 className="h-4 w-4" />
                                </Button>
                              </Can>
                              <Can permission={PERMISSIONS.ANIMAL_RECORD_DELETE}>
                                <Button
                                  variant="ghost"
                                  size="icon"
                                  onClick={() => setDeleteTarget(weight.id)}
                                  title={t('animalRecords.weights.deleteTooltip', { id: weight.id })}
                                >
                                  <Trash2 className="h-4 w-4 text-status-error-solid" />
                                </Button>
                              </Can>
                            </GuestHide>
                          </div>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>

            {/* Card view: container < 600px */}
            <div className="@[600px]:hidden space-y-3 py-1">
              {!weights || weights.length === 0 ? (
                <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                  <Scale className="h-8 w-8" />
                  <p className="text-sm">{t('animalRecords.weights.emptyTitle')}</p>
                </div>
              ) : (
                sortedData?.map((weight) => (
                  <div key={weight.id} data-record-id={weight.id} className="rounded-lg border bg-card p-3">
                    <div className="text-xs text-muted-foreground mb-0.5">
                      {new Date(weight.measure_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })}
                    </div>
                    <div className="text-lg font-bold text-foreground mb-2">
                      ⚖️ {weight.weight} kg
                    </div>
                    <div className="flex items-center justify-between gap-2 pt-1 border-t">
                      <span className="text-xs text-muted-foreground">{weight.created_by_name || '-'}</span>
                      <div className="flex gap-0.5">
                        <GuestHide>
                          <Can permission={PERMISSIONS.ANIMAL_RECORD_EDIT}>
                            <Button variant="ghost" size="icon" onClick={() => openEdit(weight)} title={t('common.edit')}>
                              <Edit2 className="h-4 w-4" />
                            </Button>
                          </Can>
                          <Can permission={PERMISSIONS.ANIMAL_RECORD_DELETE}>
                            <Button variant="ghost" size="icon" onClick={() => setDeleteTarget(weight.id)} title={t('common.delete')}>
                              <Trash2 className="h-4 w-4 text-status-error-solid" />
                            </Button>
                          </Can>
                        </GuestHide>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>

          </div>
        </CardContent>
      </Card>

      <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('animalRecords.weights.addTitle')}</DialogTitle>
            <DialogDescription>{t('animalRecords.shared.earTagLine', { earTag })}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="weight_date">{t('animalRecords.weights.measureDateRequired')}</Label>
              <Input
                id="weight_date"
                type="date"
                value={newWeight.measure_date}
                onChange={(e) => setNewWeight({ ...newWeight, measure_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="weight_value">{t('animalRecords.weights.weightKgRequired')}</Label>
              <Input
                id="weight_value"
                type="number"
                step="0.1"
                value={newWeight.weight}
                onChange={(e) => setNewWeight({ ...newWeight, weight: e.target.value })}
                placeholder={t('animalRecords.weights.enterWeight')}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAddDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              onClick={() => addMutation.mutate(newWeight)}
              disabled={addMutation.isPending || !newWeight.weight || !newWeight.measure_date}
              className="bg-status-success-solid hover:bg-status-success-solid/90"
            >
              {addMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={editTarget !== null} onOpenChange={(open) => {
        if (!open) {
          setEditTarget(null)
          setEditForm({ measure_date: '', weight: '' })
          updateMutation.reset()
        }
      }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('animalRecords.weights.editTitle')}</DialogTitle>
            <DialogDescription>{t('animalRecords.shared.earTagLine', { earTag })}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="edit_weight_date">{t('animalRecords.weights.measureDateRequired')}</Label>
              <Input
                id="edit_weight_date"
                type="date"
                required
                value={editForm.measure_date}
                onChange={(e) => setEditForm({ ...editForm, measure_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit_weight_value">{t('animalRecords.weights.weightKgRequired')}</Label>
              <Input
                id="edit_weight_value"
                type="number"
                step="0.1"
                required
                value={editForm.weight}
                onChange={(e) => setEditForm({ ...editForm, weight: e.target.value })}
                placeholder={t('animalRecords.weights.enterWeight')}
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setEditTarget(null)
                setEditForm({ measure_date: '', weight: '' })
                updateMutation.reset()
              }}
            >
              {t('common.cancel')}
            </Button>
            <Button
              onClick={() => editTarget && updateMutation.mutate({ id: editTarget.id, data: editForm })}
              disabled={updateMutation.isPending || !editForm.weight || !editForm.measure_date}
              className="bg-status-success-solid hover:bg-status-success-solid/90"
            >
              {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <DeleteReasonDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        copy={{ title: t('animalRecords.weights.deleteTitle'), description: t('animalRecords.shared.deleteRecordDescription') }}
        onConfirm={(reason) => deleteMutation.mutate({ id: deleteTarget!, reason })}
        isPending={deleteMutation.isPending}
      />
    </>
  )
})
