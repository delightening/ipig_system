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
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Plus, Pencil, Trash2, Loader2 } from 'lucide-react'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { Species } from '@/types/facility'

const speciesSchema = z.object({
  code: z.string().min(1, '代碼為必填'),
  name: z.string().min(1, '名稱為必填'),
  name_en: z.string().optional(),
  icon: z.string().optional(),
  sort_order: z.coerce.number().int(),
  parent_id: z.string().optional(),
})

type SpeciesFormData = z.output<typeof speciesSchema>

const EMPTY_FORM: SpeciesFormData = { code: '', name: '', name_en: '', icon: '', sort_order: 0, parent_id: undefined }

export function SpeciesTab({ canManage }: { canManage: boolean }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<Species | null>(null)

  const { register, handleSubmit, reset, setValue, watch, formState: { errors } } = useForm<z.input<typeof speciesSchema>, unknown, SpeciesFormData>({
    resolver: zodResolver(speciesSchema),
    defaultValues: EMPTY_FORM,
  })

  const parentId = watch('parent_id')

  const { data: species = [], isLoading } = useQuery({
    queryKey: ['species'],
    queryFn: async () => (await facilityApi.listSpecies()).data,
  })

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['species'] })

  const createMutation = useMutation({
    mutationFn: (data: SpeciesFormData) => facilityApi.createSpecies(data),
    onSuccess: () => { invalidate(); dialogs.close('create'); toast({ title: '已新增物種' }) },
    onError: (err: unknown) => toast({ title: '新增失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: SpeciesFormData }) => facilityApi.updateSpecies(id, { name: data.name, name_en: data.name_en || undefined, icon: data.icon || undefined, parent_id: data.parent_id, sort_order: data.sort_order }),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: '已更新物種' }) },
    onError: (err: unknown) => toast({ title: '更新失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deleteSpecies(id),
    onSuccess: () => { invalidate(); toast({ title: '已刪除物種' }) },
    onError: (err: unknown) => toast({ title: '刪除失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (s: Species) => {
    setEditing(s)
    reset({ code: s.code, name: s.name, name_en: s.name_en ?? '', icon: s.icon ?? '', sort_order: s.sort_order, parent_id: s.parent_id ?? undefined })
    dialogs.open('edit')
  }

  const handleDelete = async (s: Species) => {
    const ok = await confirm({
      title: '刪除物種',
      description: `確定要刪除「${s.name}」嗎？`,
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
        <span className="text-sm text-muted-foreground">共 {species.length} 筆</span>
        {canManage && (
          <Button size="sm" onClick={() => { reset(EMPTY_FORM); dialogs.open('create') }}>
            <Plus className="h-4 w-4 mr-1" /> 新增物種
          </Button>
        )}
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>代碼</TableHead>
            <TableHead>名稱</TableHead>
            <TableHead>上層物種</TableHead>
            <TableHead>英文名稱</TableHead>
            <TableHead>排序</TableHead>
            <TableHead>狀態</TableHead>
            {canManage && <TableHead className="w-24 text-right">操作</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={7} className="text-center"><Loader2 className="h-4 w-4 animate-spin mx-auto" /></TableCell></TableRow>
          ) : species.length === 0 ? (
            <TableRow><TableCell colSpan={7} className="text-center text-muted-foreground">無資料</TableCell></TableRow>
          ) : species.map(s => {
            const parentName = s.parent_id ? species.find(p => p.id === s.parent_id)?.name : null
            return (
            <TableRow key={s.id}>
              <TableCell className="font-mono">{s.code}</TableCell>
              <TableCell>{parentName ? <span className="pl-4">└ </span> : ''}{s.name}</TableCell>
              <TableCell>{parentName ?? '—'}</TableCell>
              <TableCell>{s.name_en ?? '—'}</TableCell>
              <TableCell>{s.sort_order}</TableCell>
              <TableCell><Badge variant={s.is_active ? 'default' : 'secondary'}>{s.is_active ? '啟用' : '停用'}</Badge></TableCell>
              {canManage && (
                <TableCell>
                  <div className="flex items-center justify-end gap-1">
                    <Button variant="ghost" size="icon" onClick={() => handleEdit(s)} aria-label="編輯"><Pencil className="h-4 w-4" /></Button>
                    <Button variant="ghost" size="icon" onClick={() => handleDelete(s)} aria-label="刪除"><Trash2 className="h-4 w-4 text-destructive" /></Button>
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
          <DialogHeader><DialogTitle>新增物種</DialogTitle></DialogHeader>
          <form onSubmit={onCreateSubmit} className="space-y-3">
            <div>
              <Label>代碼 *</Label>
              <Input {...register('code')} />
              {errors.code && <p className="text-sm text-destructive">{errors.code.message}</p>}
            </div>
            <div>
              <Label>名稱 *</Label>
              <Input {...register('name')} />
              {errors.name && <p className="text-sm text-destructive">{errors.name.message}</p>}
            </div>
            <div><Label>英文名稱</Label><Input {...register('name_en')} /></div>
            <div>
              <Label>上層物種</Label>
              <Select value={parentId ?? 'none'} onValueChange={v => setValue('parent_id', v === 'none' ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder="無（頂層物種）" /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">無（頂層物種）</SelectItem>
                  {species.filter(s => !s.parent_id).map(s => (
                    <SelectItem key={s.id} value={s.id}>{s.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div><Label>圖示</Label><Input {...register('icon')} placeholder="emoji 或圖示代碼" /></div>
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

      {/* 編輯 Dialog */}
      <Dialog open={dialogs.isOpen('edit')} onOpenChange={o => !o && dialogs.close('edit')}>
        <DialogContent>
          <DialogHeader><DialogTitle>編輯物種</DialogTitle></DialogHeader>
          <form onSubmit={onEditSubmit} className="space-y-3">
            <div><Label>代碼</Label><Input {...register('code')} disabled /></div>
            <div>
              <Label>名稱 *</Label>
              <Input {...register('name')} />
              {errors.name && <p className="text-sm text-destructive">{errors.name.message}</p>}
            </div>
            <div><Label>英文名稱</Label><Input {...register('name_en')} /></div>
            <div>
              <Label>上層物種</Label>
              <Select value={parentId ?? 'none'} onValueChange={v => setValue('parent_id', v === 'none' ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder="無（頂層物種）" /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">無（頂層物種）</SelectItem>
                  {species.filter(s => !s.parent_id && s.id !== editing?.id).map(s => (
                    <SelectItem key={s.id} value={s.id}>{s.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div><Label>圖示</Label><Input {...register('icon')} /></div>
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
