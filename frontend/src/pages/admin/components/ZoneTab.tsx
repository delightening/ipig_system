import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useForm, type UseFormRegister, type UseFormSetValue, type UseFormWatch } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { facilityApi } from '@/lib/api/facility'
import { useDialogSet } from '@/hooks/useDialogSet'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { FormField } from '@/components/ui/form-field'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Plus, Pencil, Trash2, Loader2 } from 'lucide-react'
import type { ZoneWithBuilding } from '@/types/facility'

const zoneSchema = z.object({
  building_id: z.string().min(1, '請選擇棟舍'),
  code: z.string().min(1, '代碼為必填'),
  name: z.string().optional(),
  color: z.string().optional(),
  sort_order: z.coerce.number().int(),
  display_group: z.string().optional(),
  group_position: z.string().optional(),
  group_order: z.coerce.number().int(),
})

type ZoneFormData = z.output<typeof zoneSchema>

const EMPTY_FORM: ZoneFormData = {
  building_id: '', code: '', name: '', color: '', sort_order: 0,
  display_group: '', group_position: '', group_order: 0,
}
type ZoneFormInput = z.input<typeof zoneSchema>

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

  const { register, handleSubmit, reset, setValue, watch, formState: { errors } } = useForm<ZoneFormInput, unknown, ZoneFormData>({
    resolver: zodResolver(zoneSchema),
    defaultValues: EMPTY_FORM,
  })

  const { data: zones = [], isLoading } = useQuery({
    queryKey: ['zones'],
    queryFn: async () => (await facilityApi.listZones()).data,
  })

  const { data: buildings = [] } = useQuery({
    queryKey: ['buildings'],
    queryFn: async () => (await facilityApi.listBuildings()).data,
  })

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['zones'] })

  const createMutation = useMutation({
    mutationFn: (data: ZoneFormData) => facilityApi.createZone({
      building_id: data.building_id, code: data.code,
      name: data.name || undefined, color: data.color || undefined,
      sort_order: data.sort_order, layout_config: buildLayoutConfig(data),
    }),
    onSuccess: () => { invalidate(); dialogs.close('create'); toast({ title: '已新增區域' }) },
    onError: (err: unknown) => toast({ title: '新增失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: ZoneFormData }) => facilityApi.updateZone(id, {
      name: data.name || undefined, color: data.color || undefined,
      sort_order: data.sort_order, layout_config: buildLayoutConfig(data),
    }),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: '已更新區域' }) },
    onError: (err: unknown) => toast({ title: '更新失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deleteZone(id),
    onSuccess: () => { invalidate(); toast({ title: '已刪除區域' }) },
    onError: (err: unknown) => toast({ title: '刪除失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
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
    const ok = await confirm({ title: '刪除區域', description: `確定要刪除「${z.name ?? z.code}」嗎？`, variant: 'destructive' })
    if (ok) deleteMutation.mutate(z.id)
  }

  const onCreateSubmit = handleSubmit(data => createMutation.mutate(data))
  const onEditSubmit = handleSubmit(data => {
    if (editing) updateMutation.mutate({ id: editing.id, data })
  })

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">共 {zones.length} 筆</span>
        {canManage && (
          <Button size="sm" onClick={() => { reset(EMPTY_FORM); dialogs.open('create') }}>
            <Plus className="h-4 w-4 mr-1" /> 新增區域
          </Button>
        )}
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>代碼</TableHead>
            <TableHead>名稱</TableHead>
            <TableHead>所屬棟舍</TableHead>
            <TableHead>顏色</TableHead>
            <TableHead>合併群組</TableHead>
            <TableHead>狀態</TableHead>
            {canManage && <TableHead className="w-24 text-right">操作</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={7} className="text-center"><Loader2 className="h-4 w-4 animate-spin mx-auto" /></TableCell></TableRow>
          ) : zones.length === 0 ? (
            <TableRow><TableCell colSpan={7} className="text-center text-muted-foreground">無資料</TableCell></TableRow>
          ) : zones.map(z => {
            const lc = parseLayoutConfig(z.layout_config)
            return (
              <TableRow key={z.id}>
                <TableCell className="font-mono">{z.code}</TableCell>
                <TableCell>{z.name ?? '—'}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{z.building_code}棟</TableCell>
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
                <TableCell><Badge variant={z.is_active ? 'default' : 'secondary'}>{z.is_active ? '啟用' : '停用'}</Badge></TableCell>
                {canManage && (
                  <TableCell>
                    <div className="flex items-center justify-end gap-1">
                      <Button variant="ghost" size="icon" onClick={() => handleEdit(z)} aria-label="編輯"><Pencil className="h-4 w-4" /></Button>
                      <Button variant="ghost" size="icon" onClick={() => handleDelete(z)} aria-label="刪除"><Trash2 className="h-4 w-4 text-destructive" /></Button>
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
          <DialogHeader><DialogTitle>新增區域</DialogTitle></DialogHeader>
          <form onSubmit={onCreateSubmit}>
            <ZoneFormFields register={register} setValue={setValue} watch={watch} errors={errors} buildings={buildings} isCreate />
            <DialogFooter className="mt-4">
              <Button type="button" variant="outline" onClick={() => dialogs.close('create')}>{t('common.cancel')}</Button>
              <Button type="submit" disabled={createMutation.isPending}>
                {createMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 新增
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* 編輯 Dialog */}
      <Dialog open={dialogs.isOpen('edit')} onOpenChange={o => !o && dialogs.close('edit')}>
        <DialogContent>
          <DialogHeader><DialogTitle>編輯區域</DialogTitle></DialogHeader>
          <form onSubmit={onEditSubmit}>
            <ZoneFormFields register={register} setValue={setValue} watch={watch} errors={errors} buildings={buildings} isCreate={false} />
            <DialogFooter className="mt-4">
              <Button type="button" variant="outline" onClick={() => dialogs.close('edit')}>{t('common.cancel')}</Button>
              <Button type="submit" disabled={updateMutation.isPending}>
                {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 儲存
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
  register: UseFormRegister<ZoneFormInput>
  setValue: UseFormSetValue<ZoneFormInput>
  watch: UseFormWatch<ZoneFormInput>
  errors: Record<string, { message?: string }>
  buildings: { id: string; code: string; name: string; facility_code: string }[]
  isCreate: boolean
}) {
  const buildingId = watch('building_id')
  const color = watch('color')
  const groupPosition = watch('group_position')

  return (
    <div className="space-y-3">
      {isCreate && (
        <FormField label="所屬棟舍" required error={errors.building_id?.message}>
          <Select value={buildingId} onValueChange={v => setValue('building_id', v, { shouldValidate: true })}>
            <SelectTrigger><SelectValue placeholder="選擇棟舍" /></SelectTrigger>
            <SelectContent>{buildings.map(b => <SelectItem key={b.id} value={b.id}>{b.name} ({b.facility_code}/{b.code})</SelectItem>)}</SelectContent>
          </Select>
        </FormField>
      )}
      <div className="grid grid-cols-2 gap-3">
        <FormField label={isCreate ? '代碼' : '代碼'} required={isCreate} error={errors.code?.message}>
          <Input {...register('code')} disabled={!isCreate} />
        </FormField>
        <div><Label>名稱</Label><Input {...register('name')} /></div>
      </div>
      <div className="grid grid-cols-2 gap-3">
        <div>
          <Label>顏色</Label>
          <div className="flex gap-2">
            <Input {...register('color')} placeholder="#FF0000" />
            {color && <span className="w-10 h-10 rounded border shrink-0" style={{ background: color }} />}
          </div>
        </div>
        <div>
          <Label>排序</Label>
          <Input type="number" {...register('sort_order', { valueAsNumber: true })} />
        </div>
      </div>

      {/* 合併顯示設定 */}
      <div className="border-t pt-3 mt-3">
        <Label className="text-sm font-semibold">合併顯示設定（選填）</Label>
        <p className="text-xs text-muted-foreground mb-2">
          設定後，同群組的區域會在動物列表中合併為一張卡片（如 EFG）
        </p>
        <div className="grid grid-cols-3 gap-3">
          <div>
            <Label className="text-xs">群組名稱</Label>
            <Input {...register('display_group')} placeholder="如 EFG、RS" />
          </div>
          <div>
            <Label className="text-xs">位置</Label>
            <Select value={groupPosition || '_none'} onValueChange={v => setValue('group_position', v === '_none' ? '' : v)}>
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="_none">不設定</SelectItem>
                <SelectItem value="left">左欄</SelectItem>
                <SelectItem value="right">右欄</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div>
            <Label className="text-xs">右欄排序</Label>
            <Input type="number" {...register('group_order', { valueAsNumber: true })} placeholder="0" />
          </div>
        </div>
      </div>
    </div>
  )
}
