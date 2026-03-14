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
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Plus, Pencil, Trash2, Loader2 } from 'lucide-react'
import type { DepartmentWithManager, CreateDepartmentRequest, UpdateDepartmentRequest } from '@/types/facility'

const EMPTY_FORM: CreateDepartmentRequest = { code: '', name: '', parent_id: undefined, manager_id: undefined, sort_order: 0 }
const NONE_VALUE = '__none__'

export function DepartmentTab({ canManage }: { canManage: boolean }) {
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<DepartmentWithManager | null>(null)
  const [form, setForm] = useState<CreateDepartmentRequest>(EMPTY_FORM)

  const { data: departments = [], isLoading } = useQuery({
    queryKey: ['departments'],
    queryFn: async () => (await facilityApi.listDepartments()).data,
  })

  const { data: users = [] } = useQuery({
    queryKey: ['internal-users-brief'],
    queryFn: async () => {
      const res = await import('@/lib/api').then(m => m.default.get<{ id: string; display_name: string }[]>('/hr/internal-users'))
      return res.data
    },
  })

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['departments'] })

  const createMutation = useMutation({
    mutationFn: (data: CreateDepartmentRequest) => facilityApi.createDepartment(data),
    onSuccess: () => { invalidate(); dialogs.close('create'); setForm(EMPTY_FORM); toast({ title: '已新增部門' }) },
    onError: (err: unknown) => toast({ title: '新增失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateDepartmentRequest }) => facilityApi.updateDepartment(id, data),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: '已更新部門' }) },
    onError: (err: unknown) => toast({ title: '更新失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deleteDepartment(id),
    onSuccess: () => { invalidate(); toast({ title: '已刪除部門' }) },
    onError: (err: unknown) => toast({ title: '刪除失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (d: DepartmentWithManager) => {
    setEditing(d)
    setForm({ code: d.code, name: d.name, parent_id: d.parent_id ?? undefined, manager_id: d.manager_id ?? undefined, sort_order: d.sort_order })
    dialogs.open('edit')
  }

  const handleDelete = async (d: DepartmentWithManager) => {
    const ok = await confirm({
      title: '刪除部門',
      description: `確定要刪除「${d.name}」嗎？`,
      variant: 'destructive',
    })
    if (ok) deleteMutation.mutate(d.id)
  }

  const set = (k: keyof CreateDepartmentRequest, v: string | number | undefined) => setForm(prev => ({ ...prev, [k]: v }))

  const parentOptions = departments.filter(d => !editing || d.id !== editing.id)

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">共 {departments.length} 筆</span>
        {canManage && (
          <Button size="sm" onClick={() => { setForm(EMPTY_FORM); dialogs.open('create') }}>
            <Plus className="h-4 w-4 mr-1" /> 新增部門
          </Button>
        )}
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>代碼</TableHead>
            <TableHead>名稱</TableHead>
            <TableHead>上層部門</TableHead>
            <TableHead>主管</TableHead>
            <TableHead>排序</TableHead>
            <TableHead>狀態</TableHead>
            {canManage && <TableHead className="w-24">操作</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={7} className="text-center"><Loader2 className="h-4 w-4 animate-spin mx-auto" /></TableCell></TableRow>
          ) : departments.length === 0 ? (
            <TableRow><TableCell colSpan={7} className="text-center text-muted-foreground">無資料</TableCell></TableRow>
          ) : departments.map(d => (
            <TableRow key={d.id}>
              <TableCell className="font-mono">{d.code}</TableCell>
              <TableCell>{d.name}</TableCell>
              <TableCell className="text-sm text-muted-foreground">{d.parent_name ?? '—'}</TableCell>
              <TableCell className="text-sm">{d.manager_name ?? '—'}</TableCell>
              <TableCell>{d.sort_order}</TableCell>
              <TableCell><Badge variant={d.is_active ? 'default' : 'secondary'}>{d.is_active ? '啟用' : '停用'}</Badge></TableCell>
              {canManage && (
                <TableCell className="space-x-1">
                  <Button variant="ghost" size="icon" onClick={() => handleEdit(d)}><Pencil className="h-4 w-4" /></Button>
                  <Button variant="ghost" size="icon" onClick={() => handleDelete(d)}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      {/* 新增 Dialog */}
      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增部門</DialogTitle></DialogHeader>
          <div className="space-y-3">
            <div><Label>代碼 *</Label><Input value={form.code} onChange={e => set('code', e.target.value)} /></div>
            <div><Label>名稱 *</Label><Input value={form.name} onChange={e => set('name', e.target.value)} /></div>
            <div>
              <Label>上層部門</Label>
              <Select value={form.parent_id ?? NONE_VALUE} onValueChange={v => set('parent_id', v === NONE_VALUE ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder="（無）" /></SelectTrigger>
                <SelectContent>
                  <SelectItem value={NONE_VALUE}>（無）</SelectItem>
                  {departments.map(d => <SelectItem key={d.id} value={d.id}>{d.name}</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>主管</Label>
              <Select value={form.manager_id ?? NONE_VALUE} onValueChange={v => set('manager_id', v === NONE_VALUE ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder="（無）" /></SelectTrigger>
                <SelectContent>
                  <SelectItem value={NONE_VALUE}>（無）</SelectItem>
                  {users.map(u => <SelectItem key={u.id} value={u.id}>{u.display_name}</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div><Label>排序</Label><Input type="number" value={form.sort_order ?? 0} onChange={e => set('sort_order', parseInt(e.target.value) || 0)} /></div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => dialogs.close('create')}>取消</Button>
            <Button onClick={() => createMutation.mutate(form)} disabled={createMutation.isPending || !form.code || !form.name}>
              {createMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 新增
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 編輯 Dialog */}
      <Dialog open={dialogs.isOpen('edit')} onOpenChange={o => !o && dialogs.close('edit')}>
        <DialogContent>
          <DialogHeader><DialogTitle>編輯部門</DialogTitle></DialogHeader>
          <div className="space-y-3">
            <div><Label>代碼</Label><Input value={form.code} disabled /></div>
            <div><Label>名稱 *</Label><Input value={form.name} onChange={e => set('name', e.target.value)} /></div>
            <div>
              <Label>上層部門</Label>
              <Select value={form.parent_id ?? NONE_VALUE} onValueChange={v => set('parent_id', v === NONE_VALUE ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder="（無）" /></SelectTrigger>
                <SelectContent>
                  <SelectItem value={NONE_VALUE}>（無）</SelectItem>
                  {parentOptions.map(d => <SelectItem key={d.id} value={d.id}>{d.name}</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>主管</Label>
              <Select value={form.manager_id ?? NONE_VALUE} onValueChange={v => set('manager_id', v === NONE_VALUE ? undefined : v)}>
                <SelectTrigger><SelectValue placeholder="（無）" /></SelectTrigger>
                <SelectContent>
                  <SelectItem value={NONE_VALUE}>（無）</SelectItem>
                  {users.map(u => <SelectItem key={u.id} value={u.id}>{u.display_name}</SelectItem>)}
                </SelectContent>
              </Select>
            </div>
            <div><Label>排序</Label><Input type="number" value={form.sort_order ?? 0} onChange={e => set('sort_order', parseInt(e.target.value) || 0)} /></div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => dialogs.close('edit')}>取消</Button>
            <Button onClick={() => editing && updateMutation.mutate({ id: editing.id, data: { name: form.name, parent_id: form.parent_id, manager_id: form.manager_id, sort_order: form.sort_order } })} disabled={updateMutation.isPending || !form.name}>
              {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
