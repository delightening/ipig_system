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
import { Plus, Pencil, Trash2, Loader2, Inbox } from 'lucide-react'
import type { BuildingWithFacility } from '@/types/facility'

interface BuildingFormData {
  facility_id: string
  code: string
  name: string
  description?: string
  sort_order: number
}

const EMPTY_FORM: BuildingFormData = { facility_id: '', code: '', name: '', description: '', sort_order: 0 }

export function BuildingTab({ canManage }: { canManage: boolean }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<BuildingWithFacility | null>(null)

  const { register, handleSubmit, reset, setValue, watch, formState: { errors } } = useForm<BuildingFormData>({
    defaultValues: EMPTY_FORM,
  })

  const facilityId = watch('facility_id')

  const { data: buildings = [], isLoading, isError } = useQuery({
    queryKey: ['buildings'],
    queryFn: async () => (await facilityApi.listBuildings()).data,
  })

  const { sortedData: sortedBuildings, sort, toggleSort } = useTableSort(buildings)

  // 操作欄只在可管理時渲染，skeleton／錯誤／空狀態的跨欄數要跟著變
  const columnCount = canManage ? 6 : 5


  const { data: facilities = [] } = useQuery({
    queryKey: ['facilities'],
    queryFn: async () => (await facilityApi.listFacilities()).data,
  })

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ['buildings'] })
    queryClient.invalidateQueries({ queryKey: ['facility-buildings'] })
  }

  const createMutation = useMutation({
    mutationFn: (data: BuildingFormData) => facilityApi.createBuilding({ ...data, description: data.description || undefined }),
    onSuccess: () => { invalidate(); dialogs.close('create'); toast({ title: t('admin.buildingTab.createSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.buildingTab.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: BuildingFormData }) => facilityApi.updateBuilding(id, { name: data.name, description: data.description || undefined, sort_order: data.sort_order }),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: t('admin.buildingTab.updateSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.buildingTab.updateFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deleteBuilding(id),
    onSuccess: () => { invalidate(); toast({ title: t('admin.buildingTab.deleteSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.buildingTab.deleteFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (b: BuildingWithFacility) => {
    setEditing(b)
    reset({ facility_id: b.facility_id, code: b.code, name: b.name, description: b.description ?? '', sort_order: b.sort_order })
    dialogs.open('edit')
  }

  const handleDelete = async (b: BuildingWithFacility) => {
    const ok = await confirm({
      title: t('admin.buildingTab.deleteTitle'),
      description: t('admin.buildingTab.deleteConfirm', { name: b.name }),
      variant: 'destructive',
    })
    if (ok) deleteMutation.mutate(b.id)
  }

  const onCreateSubmit = handleSubmit(data => createMutation.mutate(data))
  const onEditSubmit = handleSubmit(data => {
    if (editing) updateMutation.mutate({ id: editing.id, data })
  })

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">{t('admin.buildingTab.totalCount', { count: buildings.length })}</span>
        {canManage && (
          <Button size="sm" onClick={() => { reset(EMPTY_FORM); dialogs.open('create') }}>
            <Plus className="h-4 w-4 mr-1" /> {t('admin.buildingTab.addBuilding')}
          </Button>
        )}
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <SortableTableHead sortKey="code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.buildingTab.colCode')}</SortableTableHead>
            <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.buildingTab.colName')}</SortableTableHead>
            <SortableTableHead sortKey="facility_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.buildingTab.colFacility')}</SortableTableHead>
            <SortableTableHead sortKey="sort_order" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.buildingTab.colSortOrder')}</SortableTableHead>
            <SortableTableHead sortKey="is_active" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.buildingTab.colStatus')}</SortableTableHead>
            {canManage && <TableHead className="w-24 text-right">{t('common.actions')}</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={columnCount} className="p-0"><TableSkeleton rows={8} cols={columnCount} /></TableCell></TableRow>
          ) : isError && buildings.length === 0 ? (
            <TableErrorRow colSpan={columnCount} />
          ) : sortedBuildings?.length === 0 ? (
            <TableEmptyRow colSpan={columnCount} icon={Inbox} title={t('common.noData')} />
          ) : sortedBuildings?.map(b => (
            <TableRow key={b.id}>
              <TableCell className="font-mono">{b.code}</TableCell>
              <TableCell>{b.name}</TableCell>
              <TableCell className="text-sm text-muted-foreground">{b.facility_name} ({b.facility_code})</TableCell>
              <TableCell>{b.sort_order}</TableCell>
              <TableCell><Badge variant={b.is_active ? 'default' : 'secondary'}>{b.is_active ? t('admin.buildingTab.statusActive') : t('admin.buildingTab.statusInactive')}</Badge></TableCell>
              {canManage && (
                <TableCell>
                  <div className="flex items-center justify-end gap-1">
                    <Button variant="ghost" size="icon" onClick={() => handleEdit(b)} aria-label={t('common.edit')}><Pencil className="h-4 w-4" /></Button>
                    <Button variant="ghost" size="icon" onClick={() => handleDelete(b)} aria-label={t('common.delete')}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                  </div>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('admin.buildingTab.addBuilding')}</DialogTitle></DialogHeader>
          <form onSubmit={onCreateSubmit} className="space-y-3">
            <FormField label={t('admin.buildingTab.colFacility')} required error={errors.facility_id?.message}>
              <input type="hidden" {...register('facility_id', { required: t('admin.buildingTab.facilityRequired') })} />
              <Select value={facilityId} onValueChange={v => setValue('facility_id', v, { shouldValidate: true })}>
                <SelectTrigger><SelectValue placeholder={t('admin.buildingTab.facilityPlaceholder')} /></SelectTrigger>
                <SelectContent>{facilities.map(f => <SelectItem key={f.id} value={f.id}>{f.name} ({f.code})</SelectItem>)}</SelectContent>
              </Select>
            </FormField>
            <FormField label={t('admin.buildingTab.colCode')} required error={errors.code?.message}>
              <Input {...register('code', { required: t('admin.buildingTab.codeRequired') })} />
            </FormField>
            <FormField label={t('admin.buildingTab.colName')} required error={errors.name?.message}>
              <Input {...register('name', { required: t('admin.buildingTab.nameRequired') })} />
            </FormField>
            <FormField label={t('admin.buildingTab.fieldDescription')}>
              <Input {...register('description')} />
            </FormField>
            <FormField label={t('admin.buildingTab.colSortOrder')}>
              <Input type="number" {...register('sort_order', { valueAsNumber: true })} />
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
          <DialogHeader><DialogTitle>{t('admin.buildingTab.editBuilding')}</DialogTitle></DialogHeader>
          <form onSubmit={onEditSubmit} className="space-y-3">
            <FormField label={t('admin.buildingTab.colCode')}>
              <Input {...register('code')} disabled />
            </FormField>
            <FormField label={t('admin.buildingTab.colName')} required error={errors.name?.message}>
              <Input {...register('name', { required: t('admin.buildingTab.nameRequired') })} />
            </FormField>
            <FormField label={t('admin.buildingTab.fieldDescription')}>
              <Input {...register('description')} />
            </FormField>
            <FormField label={t('admin.buildingTab.colSortOrder')}>
              <Input type="number" {...register('sort_order', { valueAsNumber: true })} />
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

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
