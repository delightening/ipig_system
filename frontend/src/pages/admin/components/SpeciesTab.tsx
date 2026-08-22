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
import { Plus, Pencil, Trash2, Loader2, Inbox, RotateCcw } from 'lucide-react'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { Species, SpeciesListItem } from '@/types/facility'

interface SpeciesFormData {
  code: string
  name: string
  name_en?: string
  icon?: string
  sort_order: number
  parent_id?: string
}

const EMPTY_FORM: SpeciesFormData = { code: '', name: '', name_en: '', icon: '', sort_order: 0, parent_id: undefined }

export function SpeciesTab({ canManage }: { canManage: boolean }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<Species | null>(null)

  const { register, handleSubmit, reset, setValue, watch, formState: { errors } } = useForm<SpeciesFormData>({
    defaultValues: EMPTY_FORM,
  })

  const parentId = watch('parent_id')

  // 物種代碼全域唯一且不重用（見 backend ensure_species_code_available）：
  // 停用的物種若看不到，它佔用的代碼就等於被永久鎖死，使用者只會看到「代碼已存在」。
  const [showInactive, setShowInactive] = useState(false)

  const { data: species = [], isLoading, isError } = useQuery({
    queryKey: ['species', { showInactive }],
    queryFn: async () => (await facilityApi.listSpecies(showInactive)).data,
  })

  const { sortedData: sortedSpecies, sort, toggleSort } = useTableSort(species)

  // 操作欄只在可管理時渲染，skeleton 與空狀態的跨欄數要跟著變
  const columnCount = canManage ? 8 : 7

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ['species'] })
    queryClient.invalidateQueries({ queryKey: ['facility-species'] })
  }

  const reactivateMutation = useMutation({
    mutationFn: (id: string) => facilityApi.updateSpecies(id, { is_active: true }),
    onSuccess: () => { invalidate(); toast({ title: t('admin.speciesTab.reactivateSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.speciesTab.reactivateFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const createMutation = useMutation({
    mutationFn: (data: SpeciesFormData) => facilityApi.createSpecies(data),
    onSuccess: () => { invalidate(); dialogs.close('create'); toast({ title: t('admin.speciesTab.createSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.speciesTab.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: SpeciesFormData }) => facilityApi.updateSpecies(id, { name: data.name, name_en: data.name_en || undefined, icon: data.icon || undefined, parent_id: data.parent_id, sort_order: data.sort_order }),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: t('admin.speciesTab.updateSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.speciesTab.updateFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deleteSpecies(id),
    onSuccess: () => { invalidate(); toast({ title: t('admin.speciesTab.deleteSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.speciesTab.deleteFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (s: Species) => {
    setEditing(s)
    reset({ code: s.code, name: s.name, name_en: s.name_en ?? '', icon: s.icon ?? '', sort_order: s.sort_order, parent_id: s.parent_id ?? undefined })
    dialogs.open('edit')
  }

  const handleDelete = async (s: SpeciesListItem) => {
    const ok = await confirm({
      title: t('admin.speciesTab.deleteTitle'),
      description: t('admin.speciesTab.deleteConfirm', { name: s.name }),
      variant: 'destructive',
    })
    if (ok) deleteMutation.mutate(s.id)
  }

  const onCreateSubmit = handleSubmit(data => createMutation.mutate(data))
  const onEditSubmit = handleSubmit(data => {
    if (editing) updateMutation.mutate({ id: editing.id, data })
  })

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-4">
          <span className="text-sm text-muted-foreground">{t('admin.speciesTab.totalCount', { count: species.length })}</span>
          <label className="flex items-center gap-2 text-sm text-muted-foreground">
            <input
              type="checkbox"
              className="rounded border-border"
              checked={showInactive}
              onChange={e => setShowInactive(e.target.checked)}
            />
            {t('admin.speciesTab.showInactive')}
          </label>
        </div>
        {canManage && (
          <Button size="sm" onClick={() => { reset(EMPTY_FORM); dialogs.open('create') }}>
            <Plus className="h-4 w-4 mr-1" /> {t('admin.speciesTab.addSpecies')}
          </Button>
        )}
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <SortableTableHead sortKey="code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.speciesTab.colCode')}</SortableTableHead>
            <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.speciesTab.colName')}</SortableTableHead>
            <SortableTableHead sortKey="parent_id" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.speciesTab.colParent')}</SortableTableHead>
            <SortableTableHead sortKey="name_en" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.speciesTab.colNameEn')}</SortableTableHead>
            <SortableTableHead sortKey="sort_order" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.speciesTab.colSortOrder')}</SortableTableHead>
            <SortableTableHead sortKey="animal_count" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.speciesTab.colInUse')}</SortableTableHead>
            <SortableTableHead sortKey="is_active" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.speciesTab.colStatus')}</SortableTableHead>
            {canManage && <TableHead className="w-24 text-right">{t('common.actions')}</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={columnCount} className="p-0"><TableSkeleton rows={8} cols={columnCount} /></TableCell></TableRow>
          ) : isError && species.length === 0 ? (
            <TableErrorRow colSpan={columnCount} />
          ) : sortedSpecies?.length === 0 ? (
            <TableEmptyRow colSpan={columnCount} icon={Inbox} title={t('common.noData')} />
          ) : sortedSpecies?.map(s => {
            const parentName = s.parent_id ? species.find(p => p.id === s.parent_id)?.name : null
            return (
            <TableRow key={s.id}>
              <TableCell className="font-mono">{s.code}</TableCell>
              <TableCell>{parentName ? <span className="pl-4">└ </span> : ''}{s.name}</TableCell>
              <TableCell>{parentName ?? '—'}</TableCell>
              <TableCell>{s.name_en ?? '—'}</TableCell>
              <TableCell>{s.sort_order}</TableCell>
              <TableCell>
                {s.animal_count > 0
                  ? <Badge variant="outline">{t('admin.speciesTab.inUseCount', { count: s.animal_count })}</Badge>
                  : <span className="text-muted-foreground">—</span>}
              </TableCell>
              <TableCell><Badge variant={s.is_active ? 'default' : 'secondary'}>{s.is_active ? t('admin.speciesTab.statusActive') : t('admin.speciesTab.statusInactive')}</Badge></TableCell>
              {canManage && (
                <TableCell>
                  <div className="flex items-center justify-end gap-1">
                    {s.is_active ? (
                      <>
                        <Button variant="ghost" size="icon" onClick={() => handleEdit(s)} aria-label={t('admin.speciesTab.editAria', { name: s.name })}><Pencil className="h-4 w-4" /></Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleDelete(s)}
                          disabled={s.animal_count > 0}
                          title={s.animal_count > 0 ? t('admin.speciesTab.deleteBlocked') : undefined}
                          aria-label={t('admin.speciesTab.deleteAria', { name: s.name })}
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      </>
                    ) : (
                      // 停用物種仍佔用其代碼（代碼全域唯一、不重用），重新啟用是拿回代碼的唯一途徑
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => reactivateMutation.mutate(s.id)}
                        disabled={reactivateMutation.isPending}
                        aria-label={t('admin.speciesTab.reactivateAria', { name: s.name })}
                      >
                        <RotateCcw className="h-4 w-4 mr-1" />
                        {t('admin.speciesTab.reactivate')}
                      </Button>
                    )}
                  </div>
                </TableCell>
              )}
            </TableRow>
          )})}
        </TableBody>
      </Table>

      {/* 新增 Dialog */}
      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('admin.speciesTab.createTitle')}</DialogTitle></DialogHeader>
          <form onSubmit={onCreateSubmit} className="space-y-3">
            <FormField label={t('admin.speciesTab.colCode')} required error={errors.code?.message}>
              <Input {...register('code', { required: t('admin.speciesTab.codeRequired') })} />
            </FormField>
            <FormField label={t('admin.speciesTab.colName')} required error={errors.name?.message}>
              <Input {...register('name', { required: t('admin.speciesTab.nameRequired') })} />
            </FormField>
            <FormField label={t('admin.speciesTab.colNameEn')}>
              <Input {...register('name_en')} />
            </FormField>
            <FormField label={t('admin.speciesTab.colParent')}>
              <Select value={parentId ?? 'none'} onValueChange={v => setValue('parent_id', v === 'none' ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder={t('admin.speciesTab.parentNone')} /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">{t('admin.speciesTab.parentNone')}</SelectItem>
                  {species.filter(s => !s.parent_id).map(s => (
                    <SelectItem key={s.id} value={s.id}>{s.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </FormField>
            <FormField label={t('admin.speciesTab.colIcon')}>
              <Input {...register('icon')} placeholder={t('admin.speciesTab.iconPlaceholder')} />
            </FormField>
            <FormField label={t('admin.speciesTab.colSortOrder')}>
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
          <DialogHeader><DialogTitle>{t('admin.speciesTab.editTitle')}</DialogTitle></DialogHeader>
          <form onSubmit={onEditSubmit} className="space-y-3">
            <FormField label={t('admin.speciesTab.colCode')}>
              <Input {...register('code')} disabled />
            </FormField>
            <FormField label={t('admin.speciesTab.colName')} required error={errors.name?.message}>
              <Input {...register('name', { required: t('admin.speciesTab.nameRequired') })} />
            </FormField>
            <FormField label={t('admin.speciesTab.colNameEn')}>
              <Input {...register('name_en')} />
            </FormField>
            <FormField label={t('admin.speciesTab.colParent')}>
              <Select value={parentId ?? 'none'} onValueChange={v => setValue('parent_id', v === 'none' ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder={t('admin.speciesTab.parentNone')} /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">{t('admin.speciesTab.parentNone')}</SelectItem>
                  {species.filter(s => !s.parent_id && s.id !== editing?.id).map(s => (
                    <SelectItem key={s.id} value={s.id}>{s.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </FormField>
            <FormField label={t('admin.speciesTab.colIcon')}>
              <Input {...register('icon')} />
            </FormField>
            <FormField label={t('admin.speciesTab.colSortOrder')}>
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
