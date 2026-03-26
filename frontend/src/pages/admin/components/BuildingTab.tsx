import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
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
import type { BuildingWithFacility } from '@/types/facility'

const buildingSchema = z.object({
  facility_id: z.string().min(1, '請選擇設施'),
  code: z.string().min(1, '代碼為必填'),
  name: z.string().min(1, '名稱為必填'),
  description: z.string().optional(),
  sort_order: z.coerce.number().int(),
})

type BuildingFormData = z.output<typeof buildingSchema>

const EMPTY_FORM: BuildingFormData = { facility_id: '', code: '', name: '', description: '', sort_order: 0 }

export function BuildingTab({ canManage }: { canManage: boolean }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<BuildingWithFacility | null>(null)

  const { register, handleSubmit, reset, setValue, watch, formState: { errors } } = useForm<z.input<typeof buildingSchema>, unknown, BuildingFormData>({
    resolver: zodResolver(buildingSchema),
    defaultValues: EMPTY_FORM,
  })

  const facilityId = watch('facility_id')

  const { data: buildings = [], isLoading } = useQuery({
    queryKey: ['buildings'],
    queryFn: async () => (await facilityApi.listBuildings()).data,
  })

  const { data: facilities = [] } = useQuery({
    queryKey: ['facilities'],
    queryFn: async () => (await facilityApi.listFacilities()).data,
  })

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['buildings'] })

  const createMutation = useMutation({
    mutationFn: (data: BuildingFormData) => facilityApi.createBuilding({ ...data, description: data.description || undefined }),
    onSuccess: () => { invalidate(); dialogs.close('create'); toast({ title: '已新增棟舍' }) },
    onError: (err: unknown) => toast({ title: '新增失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: BuildingFormData }) => facilityApi.updateBuilding(id, { name: data.name, description: data.description || undefined, sort_order: data.sort_order }),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: '已更新棟舍' }) },
    onError: (err: unknown) => toast({ title: '更新失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deleteBuilding(id),
    onSuccess: () => { invalidate(); toast({ title: '已刪除棟舍' }) },
    onError: (err: unknown) => toast({ title: '刪除失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (b: BuildingWithFacility) => {
    setEditing(b)
    reset({ facility_id: b.facility_id, code: b.code, name: b.name, description: b.description ?? '', sort_order: b.sort_order })
    dialogs.open('edit')
  }

  const handleDelete = async (b: BuildingWithFacility) => {
    const ok = await confirm({
      title: '刪除棟舍',
      description: `確定要刪除「${b.name}」嗎？`,
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
        <span className="text-sm text-muted-foreground">共 {buildings.length} 筆</span>
        {canManage && (
          <Button size="sm" onClick={() => { reset(EMPTY_FORM); dialogs.open('create') }}>
            <Plus className="h-4 w-4 mr-1" /> 新增棟舍
          </Button>
        )}
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>代碼</TableHead>
            <TableHead>名稱</TableHead>
            <TableHead>所屬設施</TableHead>
            <TableHead>排序</TableHead>
            <TableHead>狀態</TableHead>
            {canManage && <TableHead className="w-24 text-right">操作</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={6} className="text-center"><Loader2 className="h-4 w-4 animate-spin mx-auto" /></TableCell></TableRow>
          ) : buildings.length === 0 ? (
            <TableRow><TableCell colSpan={6} className="text-center text-muted-foreground">無資料</TableCell></TableRow>
          ) : buildings.map(b => (
            <TableRow key={b.id}>
              <TableCell className="font-mono">{b.code}</TableCell>
              <TableCell>{b.name}</TableCell>
              <TableCell className="text-sm text-muted-foreground">{b.facility_name} ({b.facility_code})</TableCell>
              <TableCell>{b.sort_order}</TableCell>
              <TableCell><Badge variant={b.is_active ? 'default' : 'secondary'}>{b.is_active ? '啟用' : '停用'}</Badge></TableCell>
              {canManage && (
                <TableCell>
                  <div className="flex items-center justify-end gap-1">
                    <Button variant="ghost" size="icon" onClick={() => handleEdit(b)} aria-label="編輯"><Pencil className="h-4 w-4" /></Button>
                    <Button variant="ghost" size="icon" onClick={() => handleDelete(b)} aria-label="刪除"><Trash2 className="h-4 w-4 text-destructive" /></Button>
                  </div>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增棟舍</DialogTitle></DialogHeader>
          <form onSubmit={onCreateSubmit} className="space-y-3">
            <FormField label="所屬設施" required error={errors.facility_id?.message}>
              <Select value={facilityId} onValueChange={v => setValue('facility_id', v, { shouldValidate: true })}>
                <SelectTrigger><SelectValue placeholder="選擇設施" /></SelectTrigger>
                <SelectContent>{facilities.map(f => <SelectItem key={f.id} value={f.id}>{f.name} ({f.code})</SelectItem>)}</SelectContent>
              </Select>
            </FormField>
            <FormField label="代碼" required error={errors.code?.message}>
              <Input {...register('code')} />
            </FormField>
            <FormField label="名稱" required error={errors.name?.message}>
              <Input {...register('name')} />
            </FormField>
            <div><Label>描述</Label><Input {...register('description')} /></div>
            <div>
              <Label>排序</Label>
              <Input type="number" {...register('sort_order', { valueAsNumber: true })} />
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => dialogs.close('create')}>{t('common.cancel')}</Button>
              <Button type="submit" disabled={createMutation.isPending}>
                {createMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 新增
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog open={dialogs.isOpen('edit')} onOpenChange={o => !o && dialogs.close('edit')}>
        <DialogContent>
          <DialogHeader><DialogTitle>編輯棟舍</DialogTitle></DialogHeader>
          <form onSubmit={onEditSubmit} className="space-y-3">
            <div><Label>代碼</Label><Input {...register('code')} disabled /></div>
            <FormField label="名稱" required error={errors.name?.message}>
              <Input {...register('name')} />
            </FormField>
            <div><Label>描述</Label><Input {...register('description')} /></div>
            <div>
              <Label>排序</Label>
              <Input type="number" {...register('sort_order', { valueAsNumber: true })} />
            </div>
            <DialogFooter>
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
