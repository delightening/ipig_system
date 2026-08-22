import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useForm, type UseFormRegister, type UseFormSetValue, type UseFormWatch } from 'react-hook-form'
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
import type { ZoneWithBuilding } from '@/types/facility'

interface ZoneFormData {
  building_id: string
  code: string
  name?: string
  color?: string
  sort_order: number
  display_group?: string
  group_position?: string
  group_order: number
}

const EMPTY_FORM: ZoneFormData = {
  building_id: '', code: '', name: '', color: '', sort_order: 0,
  display_group: '', group_position: '', group_order: 0,
}

function parseLayoutConfig(cfg: Record<string, unknown> | null): { display_group: string; group_position: string; group_order: number } {
  return {
    display_group: (cfg?.display_group as string) || '',
    group_position: (cfg?.group_position as string) || '',
    group_order: (cfg?.group_order as number) || 0,
  }
}

function buildLayoutConfig(form: ZoneFormData): Record<string, unknown> | null {
  if (!form.display_group) return null
  const cfg: Record<string, unknown> = { display_group: form.display_group }
  if (form.group_position) cfg.group_position = form.group_position
  if (form.group_order) cfg.group_order = form.group_order
  return cfg
}

export function ZoneTab({ canManage }: { canManage: boolean }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<ZoneWithBuilding | null>(null)

  const { register, handleSubmit, reset, setValue, watch, formState: { errors } } = useForm<ZoneFormData>({
    defaultValues: EMPTY_FORM,
  })

  const { data: zones = [], isLoading, isError } = useQuery({
    queryKey: ['zones'],
    queryFn: async () => (await facilityApi.listZones()).data,
  })

  const { sortedData: sortedZones, sort, toggleSort } = useTableSort(zones)

  // 操作欄只在可管理時渲染，skeleton／錯誤／空狀態的跨欄數要跟著變
  const columnCount = canManage ? 7 : 6


  const { data: buildings = [] } = useQuery({
    queryKey: ['buildings'],
    queryFn: async () => (await facilityApi.listBuildings()).data,
  })

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ['zones'] })
    queryClient.invalidateQueries({ queryKey: ['facility-zones'] })
  }

  const createMutation = useMutation({
    mutationFn: (data: ZoneFormData) => facilityApi.createZone({
      building_id: data.building_id, code: data.code,
      name: data.name || undefined, color: data.color || undefined,
      sort_order: data.sort_order, layout_config: buildLayoutConfig(data),
    }),
    onSuccess: () => { invalidate(); dialogs.close('create'); toast({ title: t('admin.zoneTab.createSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.zoneTab.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: ZoneFormData }) => facilityApi.updateZone(id, {
      name: data.name || undefined, color: data.color || undefined,
      sort_order: data.sort_order, layout_config: buildLayoutConfig(data),
    }),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: t('admin.zoneTab.updateSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.zoneTab.updateFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deleteZone(id),
    onSuccess: () => { invalidate(); toast({ title: t('admin.zoneTab.deleteSuccess') }) },
    onError: (err: unknown) => toast({ title: t('admin.zoneTab.deleteFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (z: ZoneWithBuilding) => {
    const lc = parseLayoutConfig(z.layout_config)
    setEditing(z)
    reset({
      building_id: z.building_id, code: z.code, name: z.name ?? '', color: z.color ?? '',
      sort_order: z.sort_order, ...lc,
    })
    dialogs.open('edit')
  }

  const handleDelete = async (z: ZoneWithBuilding) => {
    const ok = await confirm({ title: t('admin.zoneTab.deleteTitle'), description: t('admin.zoneTab.deleteConfirm', { name: z.name ?? z.code }), variant: 'destructive' })
    if (ok) deleteMutation.mutate(z.id)
  }

  const onCreateSubmit = handleSubmit(data => createMutation.mutate(data))
  const onEditSubmit = handleSubmit(data => {
    if (editing) updateMutation.mutate({ id: editing.id, data })
  })

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">{t('admin.zoneTab.totalCount', { count: zones.length })}</span>
        {canManage && (
          <Button size="sm" onClick={() => { reset(EMPTY_FORM); dialogs.open('create') }}>
            <Plus className="h-4 w-4 mr-1" /> {t('admin.zoneTab.addZone')}
          </Button>
        )}
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <SortableTableHead sortKey="code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.zoneTab.colCode')}</SortableTableHead>
            <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.zoneTab.colName')}</SortableTableHead>
            <SortableTableHead sortKey="building_code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.zoneTab.colBuilding')}</SortableTableHead>
            <SortableTableHead sortKey="color" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.zoneTab.colColor')}</SortableTableHead>
            <TableHead>{t('admin.zoneTab.colMergeGroup')}</TableHead>
            <SortableTableHead sortKey="is_active" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.zoneTab.colStatus')}</SortableTableHead>
            {canManage && <TableHead className="w-24 text-right">{t('common.actions')}</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={columnCount} className="p-0"><TableSkeleton rows={8} cols={columnCount} /></TableCell></TableRow>
          ) : isError && zones.length === 0 ? (
            <TableErrorRow colSpan={columnCount} />
          ) : sortedZones?.length === 0 ? (
            <TableEmptyRow colSpan={columnCount} icon={Inbox} title={t('common.noData')} />
          ) : sortedZones?.map(z => {
            const lc = parseLayoutConfig(z.layout_config)
            return (
              <TableRow key={z.id}>
                <TableCell className="font-mono">{z.code}</TableCell>
                <TableCell>{z.name ?? '—'}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{t('admin.zoneTab.buildingSuffix', { code: z.building_code })}</TableCell>
                <TableCell>
                  {z.color ? (
                    <span className="flex items-center gap-1">
                      <span className="w-4 h-4 rounded-sm inline-block border" style={{ background: z.color }} />
                      {z.color}
                    </span>
                  ) : '—'}
                </TableCell>
                <TableCell>
                  {lc.display_group ? (
                    <Badge variant="outline">{lc.display_group} / {lc.group_position || '-'}</Badge>
                  ) : '—'}
                </TableCell>
                <TableCell><Badge variant={z.is_active ? 'default' : 'secondary'}>{z.is_active ? t('admin.zoneTab.statusActive') : t('admin.zoneTab.statusInactive')}</Badge></TableCell>
                {canManage && (
                  <TableCell>
                    <div className="flex items-center justify-end gap-1">
                      <Button variant="ghost" size="icon" onClick={() => handleEdit(z)} aria-label={t('common.edit')}><Pencil className="h-4 w-4" /></Button>
                      <Button variant="ghost" size="icon" onClick={() => handleDelete(z)} aria-label={t('common.delete')}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                    </div>
                  </TableCell>
                )}
              </TableRow>
            )
          })}
        </TableBody>
      </Table>

      {/* 新增 Dialog */}
      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('admin.zoneTab.addZone')}</DialogTitle></DialogHeader>
          <form onSubmit={onCreateSubmit}>
            <ZoneFormFields register={register} setValue={setValue} watch={watch} errors={errors} buildings={buildings} isCreate />
            <DialogFooter className="mt-4">
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
          <DialogHeader><DialogTitle>{t('admin.zoneTab.editZone')}</DialogTitle></DialogHeader>
          <form onSubmit={onEditSubmit}>
            <ZoneFormFields register={register} setValue={setValue} watch={watch} errors={errors} buildings={buildings} isCreate={false} />
            <DialogFooter className="mt-4">
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

function ZoneFormFields({ register, setValue, watch, errors, buildings, isCreate }: {
  register: UseFormRegister<ZoneFormData>
  setValue: UseFormSetValue<ZoneFormData>
  watch: UseFormWatch<ZoneFormData>
  errors: Record<string, { message?: string }>
  buildings: { id: string; code: string; name: string; facility_code: string }[]
  isCreate: boolean
}) {
  const { t } = useTranslation()
  const buildingId = watch('building_id')
  const color = watch('color')
  const groupPosition = watch('group_position')

  return (
    <div className="space-y-3">
      {isCreate && (
        <FormField label={t('admin.zoneTab.colBuilding')} required error={errors.building_id?.message}>
          <input type="hidden" {...register('building_id', { required: t('admin.zoneTab.buildingRequired') })} />
          <Select value={buildingId} onValueChange={v => setValue('building_id', v, { shouldValidate: true })}>
            <SelectTrigger><SelectValue placeholder={t('admin.zoneTab.selectBuilding')} /></SelectTrigger>
            <SelectContent>{buildings.map(b => <SelectItem key={b.id} value={b.id}>{b.name} ({b.facility_code}/{b.code})</SelectItem>)}</SelectContent>
          </Select>
        </FormField>
      )}
      <div className="grid grid-cols-2 gap-3">
        <FormField label={t('admin.zoneTab.colCode')} required={isCreate} error={errors.code?.message}>
          <Input {...register('code', isCreate ? { required: t('admin.zoneTab.codeRequired') } : {})} disabled={!isCreate} />
        </FormField>
        <FormField label={t('admin.zoneTab.colName')}>
          <Input {...register('name')} />
        </FormField>
      </div>
      <div className="grid grid-cols-2 gap-3">
        <FormField label={t('admin.zoneTab.colColor')}>
          <div className="flex gap-2">
            <Input {...register('color')} placeholder="#FF0000" />
            {color && <span className="w-10 h-10 rounded border shrink-0" style={{ background: color }} />}
          </div>
        </FormField>
        <FormField label={t('admin.zoneTab.sortOrder')}>
          <Input type="number" {...register('sort_order', { valueAsNumber: true })} />
        </FormField>
      </div>

      {/* 合併顯示設定 */}
      <div className="border-t pt-3 mt-3">
        <span className="text-sm font-semibold">{t('admin.zoneTab.mergeSettingsTitle')}</span>
        <p className="text-xs text-muted-foreground mb-2">
          {t('admin.zoneTab.mergeSettingsHint')}
        </p>
        <div className="grid grid-cols-3 gap-3">
          <FormField label={t('admin.zoneTab.groupName')}>
            <Input {...register('display_group')} placeholder={t('admin.zoneTab.groupNamePlaceholder')} />
          </FormField>
          <FormField label={t('admin.zoneTab.groupPosition')}>
            <Select value={groupPosition || '_none'} onValueChange={v => setValue('group_position', v === '_none' ? '' : v)}>
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="_none">{t('admin.zoneTab.positionNone')}</SelectItem>
                <SelectItem value="left">{t('admin.zoneTab.positionLeft')}</SelectItem>
                <SelectItem value="right">{t('admin.zoneTab.positionRight')}</SelectItem>
              </SelectContent>
            </Select>
          </FormField>
          <FormField label={t('admin.zoneTab.groupOrder')}>
            <Input type="number" {...register('group_order', { valueAsNumber: true })} placeholder="0" />
          </FormField>
        </div>
      </div>
    </div>
  )
}
