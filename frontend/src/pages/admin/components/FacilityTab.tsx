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
import type { Facility } from '@/types/facility'

const facilitySchema = z.object({
  code: z.string().min(1, '代碼為必填'),
  name: z.string().min(1, '名稱為必填'),
  address: z.string().optional(),
  phone: z.string().optional(),
  contact_person: z.string().optional(),
})

type FacilityFormData = z.infer<typeof facilitySchema>

const EMPTY_FORM: FacilityFormData = { code: '', name: '', address: '', phone: '', contact_person: '' }

export function FacilityTab({ canManage }: { canManage: boolean }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<Facility | null>(null)

  const { register, handleSubmit, reset, formState: { errors } } = useForm<FacilityFormData>({
    resolver: zodResolver(facilitySchema),
    defaultValues: EMPTY_FORM,
  })

  const { data: facilities = [], isLoading } = useQuery({
    queryKey: ['facilities'],
    queryFn: async () => (await facilityApi.listFacilities()).data,
  })

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['facilities'] })

  const createMutation = useMutation({
    mutationFn: (data: FacilityFormData) => facilityApi.createFacility({ ...data, address: data.address || undefined, phone: data.phone || undefined, contact_person: data.contact_person || undefined }),
    onSuccess: () => { invalidate(); dialogs.close('create'); toast({ title: '已新增設施' }) },
    onError: (err: unknown) => toast({ title: '新增失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: FacilityFormData }) => facilityApi.updateFacility(id, { name: data.name, address: data.address || undefined, phone: data.phone || undefined, contact_person: data.contact_person || undefined }),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: '已更新設施' }) },
    onError: (err: unknown) => toast({ title: '更新失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deleteFacility(id),
    onSuccess: () => { invalidate(); toast({ title: '已刪除設施' }) },
    onError: (err: unknown) => toast({ title: '刪除失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (f: Facility) => {
    setEditing(f)
    reset({ code: f.code, name: f.name, address: f.address ?? '', phone: f.phone ?? '', contact_person: f.contact_person ?? '' })
    dialogs.open('edit')
  }

  const handleDelete = async (f: Facility) => {
    const ok = await confirm({
      title: '刪除設施',
      description: `確定要刪除「${f.name}」嗎？`,
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
        <span className="text-sm text-muted-foreground">共 {facilities.length} 筆</span>
        {canManage && (
          <Button size="sm" onClick={() => { reset(EMPTY_FORM); dialogs.open('create') }}>
            <Plus className="h-4 w-4 mr-1" /> 新增設施
          </Button>
        )}
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>代碼</TableHead>
            <TableHead>名稱</TableHead>
            <TableHead>聯絡人</TableHead>
            <TableHead>電話</TableHead>
            <TableHead>狀態</TableHead>
            {canManage && <TableHead className="w-24 text-right">操作</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={6} className="text-center"><Loader2 className="h-4 w-4 animate-spin mx-auto" /></TableCell></TableRow>
          ) : facilities.length === 0 ? (
            <TableRow><TableCell colSpan={6} className="text-center text-muted-foreground">無資料</TableCell></TableRow>
          ) : facilities.map(f => (
            <TableRow key={f.id}>
              <TableCell className="font-mono">{f.code}</TableCell>
              <TableCell>{f.name}</TableCell>
              <TableCell>{f.contact_person ?? '—'}</TableCell>
              <TableCell>{f.phone ?? '—'}</TableCell>
              <TableCell><Badge variant={f.is_active ? 'default' : 'secondary'}>{f.is_active ? '啟用' : '停用'}</Badge></TableCell>
              {canManage && (
                <TableCell>
                  <div className="flex items-center justify-end gap-1">
                    <Button variant="ghost" size="icon" onClick={() => handleEdit(f)} aria-label="編輯"><Pencil className="h-4 w-4" /></Button>
                    <Button variant="ghost" size="icon" onClick={() => handleDelete(f)} aria-label="刪除"><Trash2 className="h-4 w-4 text-destructive" /></Button>
                  </div>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增設施</DialogTitle></DialogHeader>
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
            <div><Label>地址</Label><Input {...register('address')} /></div>
            <div><Label>電話</Label><Input {...register('phone')} /></div>
            <div><Label>聯絡人</Label><Input {...register('contact_person')} /></div>
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
          <DialogHeader><DialogTitle>編輯設施</DialogTitle></DialogHeader>
          <form onSubmit={onEditSubmit} className="space-y-3">
            <div><Label>代碼</Label><Input {...register('code')} disabled /></div>
            <div>
              <Label>名稱 *</Label>
              <Input {...register('name')} />
              {errors.name && <p className="text-sm text-destructive">{errors.name.message}</p>}
            </div>
            <div><Label>地址</Label><Input {...register('address')} /></div>
            <div><Label>電話</Label><Input {...register('phone')} /></div>
            <div><Label>聯絡人</Label><Input {...register('contact_person')} /></div>
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
