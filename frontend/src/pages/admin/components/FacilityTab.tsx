import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
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
import type { Facility, CreateFacilityRequest, UpdateFacilityRequest } from '@/types/facility'

const EMPTY_FORM: CreateFacilityRequest = { code: '', name: '', address: '', phone: '', contact_person: '' }

export function FacilityTab({ canManage }: { canManage: boolean }) {
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<Facility | null>(null)
  const [form, setForm] = useState<CreateFacilityRequest>(EMPTY_FORM)

  const { data: facilities = [], isLoading } = useQuery({
    queryKey: ['facilities'],
    queryFn: async () => (await facilityApi.listFacilities()).data,
  })

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['facilities'] })

  const createMutation = useMutation({
    mutationFn: (data: CreateFacilityRequest) => facilityApi.createFacility(data),
    onSuccess: () => { invalidate(); dialogs.close('create'); setForm(EMPTY_FORM); toast({ title: '已新增設施' }) },
    onError: (err: unknown) => toast({ title: '新增失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateFacilityRequest }) => facilityApi.updateFacility(id, data),
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
    setForm({ code: f.code, name: f.name, address: f.address ?? '', phone: f.phone ?? '', contact_person: f.contact_person ?? '' })
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

  const set = (k: keyof CreateFacilityRequest, v: string) => setForm(prev => ({ ...prev, [k]: v }))

  const FormFields = ({ disabled = false }: { disabled?: boolean }) => (
    <div className="space-y-3">
      <div><Label>代碼 *</Label><Input value={form.code} onChange={e => set('code', e.target.value)} disabled={disabled} /></div>
      <div><Label>名稱 *</Label><Input value={form.name} onChange={e => set('name', e.target.value)} /></div>
      <div><Label>地址</Label><Input value={form.address ?? ''} onChange={e => set('address', e.target.value)} /></div>
      <div><Label>電話</Label><Input value={form.phone ?? ''} onChange={e => set('phone', e.target.value)} /></div>
      <div><Label>聯絡人</Label><Input value={form.contact_person ?? ''} onChange={e => set('contact_person', e.target.value)} /></div>
    </div>
  )

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">共 {facilities.length} 筆</span>
        {canManage && (
          <Button size="sm" onClick={() => { setForm(EMPTY_FORM); dialogs.open('create') }}>
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
            {canManage && <TableHead className="w-24">操作</TableHead>}
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
                <TableCell className="space-x-1">
                  <Button variant="ghost" size="icon" onClick={() => handleEdit(f)}><Pencil className="h-4 w-4" /></Button>
                  <Button variant="ghost" size="icon" onClick={() => handleDelete(f)}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增設施</DialogTitle></DialogHeader>
          <FormFields />
          <DialogFooter>
            <Button variant="outline" onClick={() => dialogs.close('create')}>取消</Button>
            <Button onClick={() => createMutation.mutate({ ...form, address: form.address || undefined, phone: form.phone || undefined, contact_person: form.contact_person || undefined })} disabled={createMutation.isPending || !form.code || !form.name}>
              {createMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 新增
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={dialogs.isOpen('edit')} onOpenChange={o => !o && dialogs.close('edit')}>
        <DialogContent>
          <DialogHeader><DialogTitle>編輯設施</DialogTitle></DialogHeader>
          <FormFields disabled />
          <DialogFooter>
            <Button variant="outline" onClick={() => dialogs.close('edit')}>取消</Button>
            <Button onClick={() => editing && updateMutation.mutate({ id: editing.id, data: { name: form.name, address: form.address || undefined, phone: form.phone || undefined, contact_person: form.contact_person || undefined } })} disabled={updateMutation.isPending || !form.name}>
              {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
