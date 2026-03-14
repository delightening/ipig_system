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
import type { ZoneWithBuilding, CreateZoneRequest, UpdateZoneRequest } from '@/types/facility'

const EMPTY_FORM: CreateZoneRequest = { building_id: '', code: '', name: '', color: '', sort_order: 0 }

export function ZoneTab({ canManage }: { canManage: boolean }) {
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const confirm = useConfirmDialog()
  const [editing, setEditing] = useState<ZoneWithBuilding | null>(null)
  const [form, setForm] = useState<CreateZoneRequest>(EMPTY_FORM)

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
    mutationFn: (data: CreateZoneRequest) => facilityApi.createZone(data),
    onSuccess: () => { invalidate(); dialogs.close('create'); setForm(EMPTY_FORM); toast({ title: '已新增區域' }) },
    onError: (err: unknown) => toast({ title: '新增失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateZoneRequest }) => facilityApi.updateZone(id, data),
    onSuccess: () => { invalidate(); dialogs.close('edit'); toast({ title: '已更新區域' }) },
    onError: (err: unknown) => toast({ title: '更新失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => facilityApi.deleteZone(id),
    onSuccess: () => { invalidate(); toast({ title: '已刪除區域' }) },
    onError: (err: unknown) => toast({ title: '刪除失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const handleEdit = (z: ZoneWithBuilding) => {
    setEditing(z)
    setForm({ building_id: z.building_id, code: z.code, name: z.name ?? '', color: z.color ?? '', sort_order: z.sort_order })
    dialogs.open('edit')
  }

  const handleDelete = (z: ZoneWithBuilding) => {
    confirm.open({
      title: '刪除區域',
      description: `確定要刪除「${z.name ?? z.code}」嗎？`,
      variant: 'destructive',
      onConfirm: () => deleteMutation.mutate(z.id),
    })
  }

  const set = (k: keyof CreateZoneRequest, v: string | number) => setForm(prev => ({ ...prev, [k]: v }))

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">共 {zones.length} 筆</span>
        {canManage && (
          <Button size="sm" onClick={() => { setForm(EMPTY_FORM); dialogs.open('create') }}>
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
            <TableHead>狀態</TableHead>
            {canManage && <TableHead className="w-24">操作</TableHead>}
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow><TableCell colSpan={6} className="text-center"><Loader2 className="h-4 w-4 animate-spin mx-auto" /></TableCell></TableRow>
          ) : zones.length === 0 ? (
            <TableRow><TableCell colSpan={6} className="text-center text-muted-foreground">無資料</TableCell></TableRow>
          ) : zones.map(z => (
            <TableRow key={z.id}>
              <TableCell className="font-mono">{z.code}</TableCell>
              <TableCell>{z.name ?? '—'}</TableCell>
              <TableCell className="text-sm text-muted-foreground">{z.building_name} ({z.building_code})</TableCell>
              <TableCell>
                {z.color ? (
                  <span className="flex items-center gap-1">
                    <span className="w-4 h-4 rounded-sm inline-block border" style={{ background: z.color }} />
                    {z.color}
                  </span>
                ) : '—'}
              </TableCell>
              <TableCell><Badge variant={z.is_active ? 'default' : 'secondary'}>{z.is_active ? '啟用' : '停用'}</Badge></TableCell>
              {canManage && (
                <TableCell className="space-x-1">
                  <Button variant="ghost" size="icon" onClick={() => handleEdit(z)}><Pencil className="h-4 w-4" /></Button>
                  <Button variant="ghost" size="icon" onClick={() => handleDelete(z)}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增區域</DialogTitle></DialogHeader>
          <div className="space-y-3">
            <div>
              <Label>所屬棟舍 *</Label>
              <Select value={form.building_id} onValueChange={v => set('building_id', v)}>
                <SelectTrigger><SelectValue placeholder="選擇棟舍" /></SelectTrigger>
                <SelectContent>{buildings.map(b => <SelectItem key={b.id} value={b.id}>{b.name} ({b.facility_code}/{b.code})</SelectItem>)}</SelectContent>
              </Select>
            </div>
            <div><Label>代碼 *</Label><Input value={form.code} onChange={e => set('code', e.target.value)} /></div>
            <div><Label>名稱</Label><Input value={form.name ?? ''} onChange={e => set('name', e.target.value)} /></div>
            <div><Label>顏色</Label><Input value={form.color ?? ''} onChange={e => set('color', e.target.value)} placeholder="#FF0000 或 red" /></div>
            <div><Label>排序</Label><Input type="number" value={form.sort_order ?? 0} onChange={e => set('sort_order', parseInt(e.target.value) || 0)} /></div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => dialogs.close('create')}>取消</Button>
            <Button onClick={() => createMutation.mutate({ ...form, name: form.name || undefined, color: form.color || undefined })} disabled={createMutation.isPending || !form.building_id || !form.code}>
              {createMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 新增
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={dialogs.isOpen('edit')} onOpenChange={o => !o && dialogs.close('edit')}>
        <DialogContent>
          <DialogHeader><DialogTitle>編輯區域</DialogTitle></DialogHeader>
          <div className="space-y-3">
            <div><Label>代碼</Label><Input value={form.code} disabled /></div>
            <div><Label>名稱</Label><Input value={form.name ?? ''} onChange={e => set('name', e.target.value)} /></div>
            <div><Label>顏色</Label><Input value={form.color ?? ''} onChange={e => set('color', e.target.value)} placeholder="#FF0000 或 red" /></div>
            <div><Label>排序</Label><Input type="number" value={form.sort_order ?? 0} onChange={e => set('sort_order', parseInt(e.target.value) || 0)} /></div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => dialogs.close('edit')}>取消</Button>
            <Button onClick={() => editing && updateMutation.mutate({ id: editing.id, data: { name: form.name || undefined, color: form.color || undefined, sort_order: form.sort_order } })} disabled={updateMutation.isPending}>
              {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDialog state={confirm.state} />
    </div>
  )
}
