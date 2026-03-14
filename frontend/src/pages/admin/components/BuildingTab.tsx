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
import type { BuildingWithFacility, CreateBuildingRequest, UpdateBuildingRequest } from '@/types/facility'

const EMPTY_FORM: CreateBuildingRequest = { facility_id: '', code: '', name: '', description: '', sort_order: 0 }

export function BuildingTab({ canManage }: { canManage: boolean }) {
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const confirm = useConfirmDialog()
  const [editing, setEditing] = useState<BuildingWithFacility | null>(null)
  const [form, setForm] = useState<CreateBuildingRequest>(EMPTY_FORM)

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
    mutationFn: (data: CreateBuildingRequest) => facilityApi.createBuilding(data),
    onSuccess: () => { invalidate(); dialogs.close('create'); setForm(EMPTY_FORM); toast({ title: '已新增棟舍' }) },
    onError: (err: unknown) => toast({ title: '新增失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateBuildingRequest }) => facilityApi.updateBuilding(id, data),
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
    setForm({ facility_id: b.facility_id, code: b.code, name: b.name, description: b.description ?? '', sort_order: b.sort_order })
    dialogs.open('edit')
  }

  const handleDelete = (b: BuildingWithFacility) => {
    confirm.open({
      title: '刪除棟舍',
      description: `確定要刪除「${b.name}」嗎？`,
      variant: 'destructive',
      onConfirm: () => deleteMutation.mutate(b.id),
    })
  }

  const set = (k: keyof CreateBuildingRequest, v: string | number) => setForm(prev => ({ ...prev, [k]: v }))

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">共 {buildings.length} 筆</span>
        {canManage && (
          <Button size="sm" onClick={() => { setForm(EMPTY_FORM); dialogs.open('create') }}>
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
            {canManage && <TableHead className="w-24">操作</TableHead>}
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
                <TableCell className="space-x-1">
                  <Button variant="ghost" size="icon" onClick={() => handleEdit(b)}><Pencil className="h-4 w-4" /></Button>
                  <Button variant="ghost" size="icon" onClick={() => handleDelete(b)}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增棟舍</DialogTitle></DialogHeader>
          <div className="space-y-3">
            <div>
              <Label>所屬設施 *</Label>
              <Select value={form.facility_id} onValueChange={v => set('facility_id', v)}>
                <SelectTrigger><SelectValue placeholder="選擇設施" /></SelectTrigger>
                <SelectContent>{facilities.map(f => <SelectItem key={f.id} value={f.id}>{f.name} ({f.code})</SelectItem>)}</SelectContent>
              </Select>
            </div>
            <div><Label>代碼 *</Label><Input value={form.code} onChange={e => set('code', e.target.value)} /></div>
            <div><Label>名稱 *</Label><Input value={form.name} onChange={e => set('name', e.target.value)} /></div>
            <div><Label>描述</Label><Input value={form.description ?? ''} onChange={e => set('description', e.target.value)} /></div>
            <div><Label>排序</Label><Input type="number" value={form.sort_order ?? 0} onChange={e => set('sort_order', parseInt(e.target.value) || 0)} /></div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => dialogs.close('create')}>取消</Button>
            <Button onClick={() => createMutation.mutate({ ...form, description: form.description || undefined })} disabled={createMutation.isPending || !form.facility_id || !form.code || !form.name}>
              {createMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 新增
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={dialogs.isOpen('edit')} onOpenChange={o => !o && dialogs.close('edit')}>
        <DialogContent>
          <DialogHeader><DialogTitle>編輯棟舍</DialogTitle></DialogHeader>
          <div className="space-y-3">
            <div><Label>代碼</Label><Input value={form.code} disabled /></div>
            <div><Label>名稱 *</Label><Input value={form.name} onChange={e => set('name', e.target.value)} /></div>
            <div><Label>描述</Label><Input value={form.description ?? ''} onChange={e => set('description', e.target.value)} /></div>
            <div><Label>排序</Label><Input type="number" value={form.sort_order ?? 0} onChange={e => set('sort_order', parseInt(e.target.value) || 0)} /></div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => dialogs.close('edit')}>取消</Button>
            <Button onClick={() => editing && updateMutation.mutate({ id: editing.id, data: { name: form.name, description: form.description || undefined, sort_order: form.sort_order } })} disabled={updateMutation.isPending || !form.name}>
              {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDialog state={confirm.state} />
    </div>
  )
}
