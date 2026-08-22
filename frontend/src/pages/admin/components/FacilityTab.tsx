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
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow, TableErrorRow } from '@/components/ui/empty-state'
import { Badge } from '@/components/ui/badge'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Plus, Pencil, Trash2, Loader2, Inbox } from 'lucide-react'
import type { Facility } from '@/types/facility'

interface FacilityFormData {
  code: string
  name: string
  address?: string
  phone?: string
  contact_person?: string
}

const EMPTY_FORM: FacilityFormData = { code: '', name: '', address: '', phone: '', contact_person: '' }

export function FacilityTab({ canManage }: { canManage: boolean }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<Facility | null>(null)

  const { register, handleSubmit, reset, formState: { errors } } = useForm<FacilityFormData>({
    defaultValues: EMPTY_FORM,
  })

  const { data: facilities = [], isLoading, isError } = useQuery({
    queryKey: ['facilities'],
    queryFn: async () => (await facilityApi.listFacilities()).data,
  })

  const { sortedData: sortedFacilities, sort, toggleSort } = useTableSort(facilities)

  // 操作欄只在可管理時渲染，skeleton／錯誤／空狀態的跨欄數要跟著變
  const columnCount = canManage ? 6 : 5


  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ['facilities'] })
    queryClient.invalidateQueries({ queryKey: ['facility-buildings'] })
  }

  const createMutation = useMutation({
    mutationFn: (data: FacilityFormData) => facilityApi.createFacility({ ...data, address: data.address || undefined, phone: data.phone || undefined, contact_person: data.contact_person || undefined }),
    onSuccess: () => { invalidate(); dialogs.close('create'); toast({ title: t('admin.facilityTab.createSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.facilityTab.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: FacilityFormData }) => facilityApi.updateFacility(id, { name: data.name, address: data.address || undefined, phone: data.phone || undefined, contact_person: data.contact_person || undefined }),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: t('admin.facilityTab.updateSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.facilityTab.updateFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deleteFacility(id),
    onSuccess: () => { invalidate(); toast({ title: t('admin.facilityTab.deleteSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.facilityTab.deleteFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (f: Facility) => {
    setEditing(f)
    reset({ code: f.code, name: f.name, address: f.address ?? '', phone: f.phone ?? '', contact_person: f.contact_person ?? '' })
    dialogs.open('edit')
  }

  const handleDelete = async (f: Facility) => {
    const ok = await confirm({
      title: t('admin.facilityTab.deleteTitle'),
      description: t('admin.facilityTab.deleteConfirm', { name: f.name }),
      variant: 'destructive',
    })
    if (ok) deleteMutation.mutate(f.id)
  }

  const onCreateSubmit = handleSubmit(data => createMutation.mutate(data))
  const onEditSubmit = handleSubmit(data => {
    if (editing) updateMutation.mutate({ id: editing.id, data })
  })

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">{t('admin.facilityTab.totalCount', { count: facilities.length })}</span>
        {canManage && (
          <Button size="sm" onClick={() => { reset(EMPTY_FORM); dialogs.open('create') }}>
            <Plus className="h-4 w-4 mr-1" /> {t('admin.facilityTab.createFacility')}
          </Button>
        )}
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <SortableTableHead sortKey="code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.facilityTab.code')}</SortableTableHead>
            <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.facilityTab.name')}</SortableTableHead>
            <SortableTableHead sortKey="contact_person" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.facilityTab.contactPerson')}</SortableTableHead>
            <SortableTableHead sortKey="phone" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.facilityTab.phone')}</SortableTableHead>
            <SortableTableHead sortKey="is_active" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.facilityTab.status')}</SortableTableHead>
            {canManage && <TableHead className="w-24 text-right">{t('common.actions')}</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={columnCount} className="p-0"><TableSkeleton rows={8} cols={columnCount} /></TableCell></TableRow>
          ) : isError && facilities.length === 0 ? (
            <TableErrorRow colSpan={columnCount} />
          ) : sortedFacilities?.length === 0 ? (
            <TableEmptyRow colSpan={columnCount} icon={Inbox} title={t('common.noData')} />
          ) : sortedFacilities?.map(f => (
            <TableRow key={f.id}>
              <TableCell className="font-mono">{f.code}</TableCell>
              <TableCell>{f.name}</TableCell>
              <TableCell>{f.contact_person ?? '—'}</TableCell>
              <TableCell>{f.phone ?? '—'}</TableCell>
              <TableCell><Badge variant={f.is_active ? 'default' : 'secondary'}>{f.is_active ? t('admin.facilityTab.statusActive') : t('admin.facilityTab.statusInactive')}</Badge></TableCell>
              {canManage && (
                <TableCell>
                  <div className="flex items-center justify-end gap-1">
                    <Button variant="ghost" size="icon" onClick={() => handleEdit(f)} aria-label={t('common.edit')}><Pencil className="h-4 w-4" /></Button>
                    <Button variant="ghost" size="icon" onClick={() => handleDelete(f)} aria-label={t('common.delete')}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                  </div>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('admin.facilityTab.createFacility')}</DialogTitle></DialogHeader>
          <form onSubmit={onCreateSubmit} className="space-y-3">
            <FormField label={t('admin.facilityTab.code')} required error={errors.code?.message}>
              <Input {...register('code', { required: t('admin.facilityTab.codeRequired') })} />
            </FormField>
            <FormField label={t('admin.facilityTab.name')} required error={errors.name?.message}>
              <Input {...register('name', { required: t('admin.facilityTab.nameRequired') })} />
            </FormField>
            <FormField label={t('admin.facilityTab.address')}>
              <Input {...register('address')} />
            </FormField>
            <FormField label={t('admin.facilityTab.phone')}>
              <Input {...register('phone')} />
            </FormField>
            <FormField label={t('admin.facilityTab.contactPerson')}>
              <Input {...register('contact_person')} />
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
          <DialogHeader><DialogTitle>{t('admin.facilityTab.editFacility')}</DialogTitle></DialogHeader>
          <form onSubmit={onEditSubmit} className="space-y-3">
            <FormField label={t('admin.facilityTab.code')}>
              <Input {...register('code')} disabled />
            </FormField>
            <FormField label={t('admin.facilityTab.name')} required error={errors.name?.message}>
              <Input {...register('name', { required: t('admin.facilityTab.nameRequired') })} />
            </FormField>
            <FormField label={t('admin.facilityTab.address')}>
              <Input {...register('address')} />
            </FormField>
            <FormField label={t('admin.facilityTab.phone')}>
              <Input {...register('phone')} />
            </FormField>
            <FormField label={t('admin.facilityTab.contactPerson')}>
              <Input {...register('contact_person')} />
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
