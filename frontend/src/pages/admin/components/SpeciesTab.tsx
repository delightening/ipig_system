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
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { Species, CreateSpeciesRequest, UpdateSpeciesRequest } from '@/types/facility'

const EMPTY_FORM: CreateSpeciesRequest = { code: '', name: '', name_en: '', icon: '', sort_order: 0, parent_id: undefined }

export function SpeciesTab({ canManage }: { canManage: boolean }) {
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<Species | null>(null)
  const [form, setForm] = useState<CreateSpeciesRequest>(EMPTY_FORM)

  const { data: species = [], isLoading } = useQuery({
    queryKey: ['species'],
    queryFn: async () => (await facilityApi.listSpecies()).data,
  })

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['species'] })

  const createMutation = useMutation({
    mutationFn: (data: CreateSpeciesRequest) => facilityApi.createSpecies(data),
    onSuccess: () => { invalidate(); dialogs.close('create'); setForm(EMPTY_FORM); toast({ title: '已新增物種' }) },
    onError: (err: unknown) => toast({ title: '新增失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateSpeciesRequest }) => facilityApi.updateSpecies(id, data),
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
    setForm({ code: s.code, name: s.name, name_en: s.name_en ?? '', icon: s.icon ?? '', sort_order: s.sort_order, parent_id: s.parent_id ?? undefined })
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

  const f = (k: keyof CreateSpeciesRequest, v: string | number) => setForm(prev => ({ ...prev, [k]: v }))

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">共 {species.length} 筆</span>
        {canManage && (
          <Button size="sm" onClick={() => { setForm(EMPTY_FORM); dialogs.open('create') }}>
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
            {canManage && <TableHead className="w-24">操作</TableHead>}
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
                <TableCell className="space-x-1">
                  <Button variant="ghost" size="icon" onClick={() => handleEdit(s)}><Pencil className="h-4 w-4" /></Button>
                  <Button variant="ghost" size="icon" onClick={() => handleDelete(s)}><Trash2 className="h-4 w-4 text-destructive" /></Button>
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
          <div className="space-y-3">
            <div><Label>代碼 *</Label><Input value={form.code} onChange={e => f('code', e.target.value)} /></div>
            <div><Label>名稱 *</Label><Input value={form.name} onChange={e => f('name', e.target.value)} /></div>
            <div><Label>英文名稱</Label><Input value={form.name_en ?? ''} onChange={e => f('name_en', e.target.value)} /></div>
            <div>
              <Label>上層物種</Label>
              <Select value={form.parent_id ?? 'none'} onValueChange={v => setForm(prev => ({ ...prev, parent_id: v === 'none' ? undefined : v }))}>
                <SelectTrigger><SelectValue placeholder="無（頂層物種）" /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">無（頂層物種）</SelectItem>
                  {species.filter(s => !s.parent_id).map(s => (
                    <SelectItem key={s.id} value={s.id}>{s.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div><Label>圖示</Label><Input value={form.icon ?? ''} onChange={e => f('icon', e.target.value)} placeholder="emoji 或圖示代碼" /></div>
            <div><Label>排序</Label><Input type="number" value={form.sort_order ?? 0} onChange={e => f('sort_order', parseInt(e.target.value) || 0)} /></div>
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
          <DialogHeader><DialogTitle>編輯物種</DialogTitle></DialogHeader>
          <div className="space-y-3">
            <div><Label>代碼</Label><Input value={form.code} disabled /></div>
            <div><Label>名稱 *</Label><Input value={form.name} onChange={e => f('name', e.target.value)} /></div>
            <div><Label>英文名稱</Label><Input value={form.name_en ?? ''} onChange={e => f('name_en', e.target.value)} /></div>
            <div>
              <Label>上層物種</Label>
              <Select value={form.parent_id ?? 'none'} onValueChange={v => setForm(prev => ({ ...prev, parent_id: v === 'none' ? undefined : v }))}>
                <SelectTrigger><SelectValue placeholder="無（頂層物種）" /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">無（頂層物種）</SelectItem>
                  {species.filter(s => !s.parent_id && s.id !== editing?.id).map(s => (
                    <SelectItem key={s.id} value={s.id}>{s.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div><Label>圖示</Label><Input value={form.icon ?? ''} onChange={e => f('icon', e.target.value)} /></div>
            <div><Label>排序</Label><Input type="number" value={form.sort_order ?? 0} onChange={e => f('sort_order', parseInt(e.target.value) || 0)} /></div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => dialogs.close('edit')}>取消</Button>
            <Button onClick={() => editing && updateMutation.mutate({ id: editing.id, data: { name: form.name, name_en: form.name_en || undefined, icon: form.icon || undefined, parent_id: form.parent_id, sort_order: form.sort_order } })} disabled={updateMutation.isPending || !form.name}>
              {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
