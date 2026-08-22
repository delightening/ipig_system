import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
import { facilityApi } from '@/lib/api/facility'
import { useInternalUsersBrief } from '@/hooks/useInternalUsersBrief'
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
import { Badge } from '@/components/ui/badge'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { Plus, Pencil, Trash2, Loader2, Building2, UsersRound } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow, TableErrorRow } from '@/components/ui/empty-state'
import type { DepartmentWithManager } from '@/types/facility'
import { DepartmentMembersDialog } from './DepartmentMembersDialog'

const NONE_VALUE = '__none__'

interface DepartmentFormData {
  code: string
  name: string
  parent_id?: string
  manager_id?: string
  sort_order: number
}

const EMPTY_FORM: DepartmentFormData = { code: '', name: '', parent_id: undefined, manager_id: undefined, sort_order: 0 }

export function DepartmentTab({ canManage, canAssignMembers }: { canManage: boolean; canAssignMembers: boolean }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<DepartmentWithManager | null>(null)
  const [membersDept, setMembersDept] = useState<DepartmentWithManager | null>(null)

  const { register, handleSubmit, reset, setValue, watch, formState: { errors } } = useForm<DepartmentFormData>({
    defaultValues: EMPTY_FORM,
  })

  const parentId = watch('parent_id')
  const managerId = watch('manager_id')

  const { data: departments = [], isLoading, isError } = useQuery({
    queryKey: ['departments'],
    queryFn: async () => (await facilityApi.listDepartments()).data,
  })

  const { sortedData: sortedDepartments, sort, toggleSort } = useTableSort(departments)

  const { data: users = [] } = useInternalUsersBrief()

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['departments'] })

  const createMutation = useMutation({
    mutationFn: (data: DepartmentFormData) => facilityApi.createDepartment(data),
    onSuccess: () => { invalidate(); dialogs.close('create'); toast({ title: t('admin.departmentTab.createSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.departmentTab.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: DepartmentFormData }) => facilityApi.updateDepartment(id, { name: data.name, parent_id: data.parent_id, manager_id: data.manager_id, sort_order: data.sort_order }),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: t('admin.departmentTab.updateSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.departmentTab.updateFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deleteDepartment(id),
    onSuccess: () => { invalidate(); toast({ title: t('admin.departmentTab.deleteSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.departmentTab.deleteFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (d: DepartmentWithManager) => {
    setEditing(d)
    reset({ code: d.code, name: d.name, parent_id: d.parent_id ?? undefined, manager_id: d.manager_id ?? undefined, sort_order: d.sort_order })
    dialogs.open('edit')
  }

  const handleDelete = async (d: DepartmentWithManager) => {
    const ok = await confirm({
      title: t('admin.departmentTab.deleteTitle'),
      description: t('admin.departmentTab.deleteConfirm', { name: d.name }),
      variant: 'destructive',
    })
    if (ok) deleteMutation.mutate(d.id)
  }

  const parentOptions = departments.filter(d => !editing || d.id !== editing.id)

  const onCreateSubmit = handleSubmit(data => createMutation.mutate(data))
  const onEditSubmit = handleSubmit(data => {
    if (editing) updateMutation.mutate({ id: editing.id, data })
  })

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">{t('admin.departmentTab.totalCount', { count: departments.length })}</span>
        {canManage && (
          <Button size="sm" onClick={() => { reset(EMPTY_FORM); dialogs.open('create') }}>
            <Plus className="h-4 w-4 mr-1" /> {t('admin.departmentTab.createDepartment')}
          </Button>
        )}
      </div>
      <div className="rounded-lg border bg-card overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/50 hover:bg-muted/50">
            <SortableTableHead sortKey="code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.departmentTab.colCode')}</SortableTableHead>
            <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.departmentTab.colName')}</SortableTableHead>
            <SortableTableHead sortKey="parent_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.departmentTab.colParent')}</SortableTableHead>
            <SortableTableHead sortKey="manager_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.departmentTab.colManager')}</SortableTableHead>
            <SortableTableHead sortKey="sort_order" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.departmentTab.colSortOrder')}</SortableTableHead>
            <SortableTableHead sortKey="is_active" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.departmentTab.colStatus')}</SortableTableHead>
            <TableHead className="w-32 text-right">{t('common.actions')}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={7} className="p-0"><TableSkeleton rows={5} cols={7} /></TableCell></TableRow>
          ) : isError && departments.length === 0 ? (
            <TableErrorRow colSpan={7} />
          ) : sortedDepartments?.length === 0 ? (
            <TableEmptyRow colSpan={7} icon={Building2} title={t('admin.departmentTab.emptyTitle')} />
          ) : sortedDepartments?.map(d => (
            <TableRow key={d.id}>
              <TableCell className="font-mono">{d.code}</TableCell>
              <TableCell>{d.name}</TableCell>
              <TableCell className="text-sm text-muted-foreground">{d.parent_name ?? '—'}</TableCell>
              <TableCell className="text-sm">{d.manager_name ?? '—'}</TableCell>
              <TableCell>{d.sort_order}</TableCell>
              <TableCell><Badge variant={d.is_active ? 'default' : 'secondary'}>{d.is_active ? t('admin.departmentTab.statusActive') : t('admin.departmentTab.statusInactive')}</Badge></TableCell>
              <TableCell>
                <div className="flex items-center justify-end gap-1">
                  <Button variant="ghost" size="icon" onClick={() => setMembersDept(d)} aria-label={t('admin.departmentTab.members.manageAria', { name: d.name })}><UsersRound className="h-4 w-4" /></Button>
                  {canManage && (
                    <>
                      <Button variant="ghost" size="icon" onClick={() => handleEdit(d)} aria-label={t('common.edit')}><Pencil className="h-4 w-4" /></Button>
                      <Button variant="ghost" size="icon" onClick={() => handleDelete(d)} aria-label={t('common.delete')}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                    </>
                  )}
                </div>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
      </div>

      {/* 新增 Dialog */}
      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('admin.departmentTab.createDepartment')}</DialogTitle></DialogHeader>
          <form onSubmit={onCreateSubmit} className="space-y-3">
            <FormField label={t('admin.departmentTab.colCode')} required error={errors.code?.message}>
              <Input {...register('code', { required: t('admin.departmentTab.codeRequired') })} />
            </FormField>
            <FormField label={t('admin.departmentTab.colName')} required error={errors.name?.message}>
              <Input {...register('name', { required: t('admin.departmentTab.nameRequired') })} />
            </FormField>
            <FormField label={t('admin.departmentTab.colParent')}>
              <Select value={parentId ?? NONE_VALUE} onValueChange={v => setValue('parent_id', v === NONE_VALUE ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder={t('admin.departmentTab.none')} /></SelectTrigger>
                <SelectContent>
                  <SelectItem value={NONE_VALUE}>{t('admin.departmentTab.none')}</SelectItem>
                  {departments.map(d => <SelectItem key={d.id} value={d.id}>{d.name}</SelectItem>)}
                </SelectContent>
              </Select>
            </FormField>
            <FormField label={t('admin.departmentTab.colManager')}>
              <Select value={managerId ?? NONE_VALUE} onValueChange={v => setValue('manager_id', v === NONE_VALUE ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder={t('admin.departmentTab.none')} /></SelectTrigger>
                <SelectContent>
                  <SelectItem value={NONE_VALUE}>{t('admin.departmentTab.none')}</SelectItem>
                  {users.map(u => <SelectItem key={u.id} value={u.id}>{u.display_name}</SelectItem>)}
                </SelectContent>
              </Select>
            </FormField>
            <FormField label={t('admin.departmentTab.colSortOrder')}>
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

      {/* 編輯 Dialog */}
      <Dialog open={dialogs.isOpen('edit')} onOpenChange={o => !o && dialogs.close('edit')}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('admin.departmentTab.editDepartment')}</DialogTitle></DialogHeader>
          <form onSubmit={onEditSubmit} className="space-y-3">
            <FormField label={t('admin.departmentTab.colCode')}>
              <Input {...register('code')} disabled />
            </FormField>
            <FormField label={t('admin.departmentTab.colName')} required error={errors.name?.message}>
              <Input {...register('name', { required: t('admin.departmentTab.nameRequired') })} />
            </FormField>
            <FormField label={t('admin.departmentTab.colParent')}>
              <Select value={parentId ?? NONE_VALUE} onValueChange={v => setValue('parent_id', v === NONE_VALUE ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder={t('admin.departmentTab.none')} /></SelectTrigger>
                <SelectContent>
                  <SelectItem value={NONE_VALUE}>{t('admin.departmentTab.none')}</SelectItem>
                  {parentOptions.map(d => <SelectItem key={d.id} value={d.id}>{d.name}</SelectItem>)}
                </SelectContent>
              </Select>
            </FormField>
            <FormField label={t('admin.departmentTab.colManager')}>
              <Select value={managerId ?? NONE_VALUE} onValueChange={v => setValue('manager_id', v === NONE_VALUE ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder={t('admin.departmentTab.none')} /></SelectTrigger>
                <SelectContent>
                  <SelectItem value={NONE_VALUE}>{t('admin.departmentTab.none')}</SelectItem>
                  {users.map(u => <SelectItem key={u.id} value={u.id}>{u.display_name}</SelectItem>)}
                </SelectContent>
              </Select>
            </FormField>
            <FormField label={t('admin.departmentTab.colSortOrder')}>
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

      {membersDept && (
        <DepartmentMembersDialog
          open={!!membersDept}
          onOpenChange={o => !o && setMembersDept(null)}
          department={membersDept}
          canManage={canAssignMembers}
        />
      )}

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
