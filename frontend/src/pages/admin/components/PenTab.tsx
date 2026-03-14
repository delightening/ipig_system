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
import type { PenDetails, CreatePenRequest, UpdatePenRequest } from '@/types/facility'
import { PEN_STATUS_NAMES } from '@/types/facility'

const EMPTY_FORM: CreatePenRequest = { zone_id: '', code: '', name: '', capacity: 1 }

export function PenTab({ canManage }: { canManage: boolean }) {
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<PenDetails | null>(null)
  const [form, setForm] = useState<CreatePenRequest>(EMPTY_FORM)
  const [editStatus, setEditStatus] = useState('active')

  const { data: pens = [], isLoading } = useQuery({
    queryKey: ['pens'],
    queryFn: async () => (await facilityApi.listPens()).data,
  })

  const { data: zones = [] } = useQuery({
    queryKey: ['zones'],
    queryFn: async () => (await facilityApi.listZones()).data,
  })

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['pens'] })

  const createMutation = useMutation({
    mutationFn: (data: CreatePenRequest) => facilityApi.createPen(data),
    onSuccess: () => { invalidate(); dialogs.close('create'); setForm(EMPTY_FORM); toast({ title: '已新增欄位' }) },
    onError: (err: unknown) => toast({ title: '新增失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdatePenRequest }) => facilityApi.updatePen(id, data),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: '已更新欄位' }) },
    onError: (err: unknown) => toast({ title: '更新失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deletePen(id),
    onSuccess: () => { invalidate(); toast({ title: '已刪除欄位' }) },
    onError: (err: unknown) => toast({ title: '刪除失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (p: PenDetails) => {
    setEditing(p)
    setForm({ zone_id: p.zone_id, code: p.code, name: p.name ?? '', capacity: p.capacity })
    setEditStatus(p.status)
    dialogs.open('edit')
  }

  const handleDelete = async (p: PenDetails) => {
    const ok = await confirm({
      title: '刪除欄位',
      description: `確定要刪除「${p.name ?? p.code}」嗎？`,
      variant: 'destructive',
    })
    if (ok) deleteMutation.mutate(p.id)
  }

  const set = (k: keyof CreatePenRequest, v: string | number) => setForm(prev => ({ ...prev, [k]: v }))

  const statusBadgeVariant = (s: string) => s === 'active' ? 'default' : s === 'empty' ? 'secondary' : 'outline'

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">共 {pens.length} 筆</span>
        {canManage && (
          <Button size="sm" onClick={() => { setForm(EMPTY_FORM); dialogs.open('create') }}>
            <Plus className="h-4 w-4 mr-1" /> 新增欄位
          </Button>
        )}
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>代碼</TableHead>
            <TableHead>名稱</TableHead>
            <TableHead>所屬區域</TableHead>
            <TableHead>容量</TableHead>
            <TableHead>目前數量</TableHead>
            <TableHead>狀態</TableHead>
            {canManage && <TableHead className="w-24">操作</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={7} className="text-center"><Loader2 className="h-4 w-4 animate-spin mx-auto" /></TableCell></TableRow>
          ) : pens.length === 0 ? (
            <TableRow><TableCell colSpan={7} className="text-center text-muted-foreground">無資料</TableCell></TableRow>
          ) : pens.map(p => (
            <TableRow key={p.id}>
              <TableCell className="font-mono">{p.code}</TableCell>
              <TableCell>{p.name ?? '—'}</TableCell>
              <TableCell className="text-sm text-muted-foreground">{p.zone_code} / {p.building_code}</TableCell>
              <TableCell>{p.capacity}</TableCell>
              <TableCell>{p.current_count}</TableCell>
              <TableCell><Badge variant={statusBadgeVariant(p.status)}>{PEN_STATUS_NAMES[p.status] ?? p.status}</Badge></TableCell>
              {canManage && (
                <TableCell className="space-x-1">
                  <Button variant="ghost" size="icon" onClick={() => handleEdit(p)}><Pencil className="h-4 w-4" /></Button>
                  <Button variant="ghost" size="icon" onClick={() => handleDelete(p)}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增欄位</DialogTitle></DialogHeader>
          <div className="space-y-3">
            <div>
              <Label>所屬區域 *</Label>
              <Select value={form.zone_id} onValueChange={v => set('zone_id', v)}>
                <SelectTrigger><SelectValue placeholder="選擇區域" /></SelectTrigger>
                <SelectContent>{zones.map(z => <SelectItem key={z.id} value={z.id}>{z.building_code}/{z.code} {z.name ? `(${z.name})` : ''}</SelectItem>)}</SelectContent>
              </Select>
            </div>
            <div><Label>代碼 *</Label><Input value={form.code} onChange={e => set('code', e.target.value)} /></div>
            <div><Label>名稱</Label><Input value={form.name ?? ''} onChange={e => set('name', e.target.value)} /></div>
            <div><Label>容量</Label><Input type="number" min={1} value={form.capacity ?? 1} onChange={e => set('capacity', parseInt(e.target.value) || 1)} /></div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => dialogs.close('create')}>取消</Button>
            <Button onClick={() => createMutation.mutate({ ...form, name: form.name || undefined })} disabled={createMutation.isPending || !form.zone_id || !form.code}>
              {createMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 新增
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={dialogs.isOpen('edit')} onOpenChange={o => !o && dialogs.close('edit')}>
        <DialogContent>
          <DialogHeader><DialogTitle>編輯欄位</DialogTitle></DialogHeader>
          <div className="space-y-3">
            <div><Label>代碼</Label><Input value={form.code} disabled /></div>
            <div><Label>名稱</Label><Input value={form.name ?? ''} onChange={e => set('name', e.target.value)} /></div>
            <div><Label>容量</Label><Input type="number" min={1} value={form.capacity ?? 1} onChange={e => set('capacity', parseInt(e.target.value) || 1)} /></div>
            <div>
              <Label>狀態</Label>
              <Select value={editStatus} onValueChange={setEditStatus}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>{Object.entries(PEN_STATUS_NAMES).map(([k, v]) => <SelectItem key={k} value={k}>{v}</SelectItem>)}</SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => dialogs.close('edit')}>取消</Button>
            <Button onClick={() => editing && updateMutation.mutate({ id: editing.id, data: { name: form.name || undefined, capacity: form.capacity, status: editStatus } })} disabled={updateMutation.isPending}>
              {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
