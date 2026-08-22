import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
import { facilityApi } from '@/lib/api/facility'
import { useDialogSet } from '@/hooks/useDialogSet'
import { useTableSort } from '@/hooks/useTableSort'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { FormField } from '@/components/ui/form-field'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow, TableErrorRow } from '@/components/ui/empty-state'
import { Badge } from '@/components/ui/badge'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Plus, Pencil, Trash2, Loader2, LayoutGrid } from 'lucide-react'
import type { PenDetails } from '@/types/facility'
import { PEN_STATUS_NAMES } from '@/types/facility'
import { BatchCreatePenDialog } from './BatchCreatePenDialog'
import { PenLayoutPreview } from './PenLayoutPreview'

interface PenCreateFormData {
  zone_id: string
  code: string
  name?: string
  capacity: number
}

interface PenEditFormData {
  code: string
  name?: string
  capacity: number
  status: string
}

export function PenTab({ canManage }: { canManage: boolean }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<PenDetails | null>(null)
  const [batchOpen, setBatchOpen] = useState(false)

  const createForm = useForm<PenCreateFormData>({
    defaultValues: { zone_id: '', code: '', name: '', capacity: 1 },
  })

  const editForm = useForm<PenEditFormData>({
    defaultValues: { code: '', name: '', capacity: 1, status: 'active' },
  })

  const { data: pens = [], isLoading, isError } = useQuery({
    queryKey: ['pens'],
    queryFn: async () => (await facilityApi.listPens()).data,
  })

  const { sortedData: sortedPens, sort, toggleSort } = useTableSort(pens)

  // 操作欄只在可管理時渲染，skeleton／錯誤／空狀態的跨欄數要跟著變
  const columnCount = canManage ? 7 : 6


  const { data: zones = [] } = useQuery({
    queryKey: ['zones'],
    queryFn: async () => (await facilityApi.listZones()).data,
  })

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ['pens'] })
    queryClient.invalidateQueries({ queryKey: ['facility-pens'] })
  }

  const createMutation = useMutation({
    mutationFn: (data: PenCreateFormData) => facilityApi.createPen({ ...data, name: data.name || undefined }),
    onSuccess: () => { invalidate(); dialogs.close('create'); toast({ title: t('admin.penTab.createSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.penTab.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: PenEditFormData }) => facilityApi.updatePen(id, { name: data.name || undefined, capacity: data.capacity, status: data.status }),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: t('admin.penTab.updateSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.penTab.updateFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deletePen(id),
    onSuccess: () => { invalidate(); toast({ title: t('admin.penTab.deleteSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.penTab.deleteFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (p: PenDetails) => {
    setEditing(p)
    editForm.reset({ code: p.code, name: p.name ?? '', capacity: p.capacity, status: p.status })
    dialogs.open('edit')
  }

  const handleDelete = async (p: PenDetails) => {
    const ok = await confirm({
      title: t('admin.penTab.deleteTitle'),
      description: t('admin.penTab.deleteConfirm', { name: p.name ?? p.code }),
      variant: 'destructive',
    })
    if (ok) deleteMutation.mutate(p.id)
  }

  const onCreateSubmit = createForm.handleSubmit(data => createMutation.mutate(data))
  const onEditSubmit = editForm.handleSubmit(data => {
    if (editing) updateMutation.mutate({ id: editing.id, data })
  })

  const statusBadgeVariant = (s: string) => s === 'active' ? 'default' : s === 'empty' ? 'secondary' : 'outline'

  const createZoneId = createForm.watch('zone_id')
  const editStatus = editForm.watch('status')

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">{t('admin.penTab.totalCount', { count: pens.length })}</span>
        {canManage && (
          <div className="flex gap-2">
            <Button size="sm" variant="outline" onClick={() => setBatchOpen(true)}>
              <LayoutGrid className="h-4 w-4 mr-1" /> {t('admin.penTab.batchCreate')}
            </Button>
            <Button size="sm" onClick={() => { createForm.reset({ zone_id: '', code: '', name: '', capacity: 1 }); dialogs.open('create') }}>
              <Plus className="h-4 w-4 mr-1" /> {t('admin.penTab.addPen')}
            </Button>
          </div>
        )}
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <SortableTableHead sortKey="code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.penTab.colCode')}</SortableTableHead>
            <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.penTab.colName')}</SortableTableHead>
            <SortableTableHead sortKey="building_code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.penTab.colZone')}</SortableTableHead>
            <SortableTableHead sortKey="capacity" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.penTab.colCapacity')}</SortableTableHead>
            <SortableTableHead sortKey="current_count" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.penTab.colCurrentCount')}</SortableTableHead>
            <SortableTableHead sortKey="status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.penTab.colStatus')}</SortableTableHead>
            {canManage && <TableHead className="w-24 text-right">{t('common.actions')}</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={columnCount} className="p-0"><TableSkeleton rows={8} cols={columnCount} /></TableCell></TableRow>
          ) : isError && pens.length === 0 ? (
            <TableErrorRow colSpan={columnCount} />
          ) : sortedPens?.length === 0 ? (
            <TableEmptyRow colSpan={columnCount} icon={LayoutGrid} title={t('common.noData')} />
          ) : sortedPens?.map(p => (
            <TableRow key={p.id}>
              <TableCell className="font-mono">{p.code}</TableCell>
              <TableCell>{p.name ?? '—'}</TableCell>
              <TableCell className="text-sm text-muted-foreground">{t('admin.penTab.zoneLabel', { building: p.building_code, zone: p.zone_code })}</TableCell>
              <TableCell>{p.capacity}</TableCell>
              <TableCell>{p.current_count}</TableCell>
              <TableCell><Badge variant={statusBadgeVariant(p.status)}>{t(PEN_STATUS_NAMES[p.status] ?? p.status)}</Badge></TableCell>
              {canManage && (
                <TableCell>
                  <div className="flex items-center justify-end gap-1">
                    <Button variant="ghost" size="icon" onClick={() => handleEdit(p)} aria-label={t('common.edit')}><Pencil className="h-4 w-4" /></Button>
                    <Button variant="ghost" size="icon" onClick={() => handleDelete(p)} aria-label={t('common.delete')}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                  </div>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <PenLayoutPreview pens={pens} zones={zones} canManage={canManage} />

      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('admin.penTab.addPen')}</DialogTitle></DialogHeader>
          <form onSubmit={onCreateSubmit} className="space-y-3">
            <FormField label={t('admin.penTab.colZone')} required error={createForm.formState.errors.zone_id?.message}>
              <input type="hidden" {...createForm.register('zone_id', { required: t('admin.penTab.zoneRequired') })} />
              <Select value={createZoneId} onValueChange={v => createForm.setValue('zone_id', v, { shouldValidate: true })}>
                <SelectTrigger><SelectValue placeholder={t('admin.penTab.selectZone')} /></SelectTrigger>
                <SelectContent>{zones.map(z => <SelectItem key={z.id} value={z.id}>{t('admin.penTab.zoneLabel', { building: z.building_code, zone: z.code })} {z.name ? `(${z.name})` : ''}</SelectItem>)}</SelectContent>
              </Select>
            </FormField>
            <FormField label={t('admin.penTab.colCode')} required error={createForm.formState.errors.code?.message}>
              <Input {...createForm.register('code', { required: t('admin.penTab.codeRequired') })} />
            </FormField>
            <FormField label={t('admin.penTab.colName')}>
              <Input {...createForm.register('name')} />
            </FormField>
            <FormField label={t('admin.penTab.colCapacity')}>
              <Input type="number" min={1} {...createForm.register('capacity', { valueAsNumber: true, min: { value: 1, message: t('admin.penTab.capacityMin') } })} />
            </FormField>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => dialogs.close('create')}>{t('common.cancel')}</Button>
              <Button type="submit" disabled={createMutation.isPending}>
                {createMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} {t('common.create')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog open={dialogs.isOpen('edit')} onOpenChange={o => !o && dialogs.close('edit')}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('admin.penTab.editPen')}</DialogTitle></DialogHeader>
          <form onSubmit={onEditSubmit} className="space-y-3">
            <FormField label={t('admin.penTab.colCode')}>
              <Input {...editForm.register('code')} disabled />
            </FormField>
            <FormField label={t('admin.penTab.colName')}>
              <Input {...editForm.register('name')} />
            </FormField>
            <FormField label={t('admin.penTab.colCapacity')}>
              <Input type="number" min={1} {...editForm.register('capacity', { valueAsNumber: true, min: { value: 1, message: t('admin.penTab.capacityMin') } })} />
            </FormField>
            <FormField label={t('admin.penTab.colStatus')}>
              <Select value={editStatus} onValueChange={v => editForm.setValue('status', v)}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>{Object.entries(PEN_STATUS_NAMES).map(([k, v]) => <SelectItem key={k} value={k}>{t(v)}</SelectItem>)}</SelectContent>
              </Select>
            </FormField>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => dialogs.close('edit')}>{t('common.cancel')}</Button>
              <Button type="submit" disabled={updateMutation.isPending}>
                {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} {t('common.save')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <BatchCreatePenDialog open={batchOpen} onOpenChange={setBatchOpen} zones={zones} />
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
