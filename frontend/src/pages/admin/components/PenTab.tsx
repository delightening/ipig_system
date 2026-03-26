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
import { FormField } from '@/components/ui/form-field'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Plus, Pencil, Trash2, Loader2, LayoutGrid } from 'lucide-react'
import type { PenDetails } from '@/types/facility'
import { PEN_STATUS_NAMES } from '@/types/facility'
import { BatchCreatePenDialog } from './BatchCreatePenDialog'
import { PenLayoutPreview } from './PenLayoutPreview'

const penCreateSchema = z.object({
  zone_id: z.string().min(1, '請選擇區域'),
  code: z.string().min(1, '代碼為必填'),
  name: z.string().optional(),
  capacity: z.coerce.number().int().min(1),
})

const penEditSchema = z.object({
  code: z.string().min(1, '代碼為必填'),
  name: z.string().optional(),
  capacity: z.coerce.number().int().min(1),
  status: z.string(),
})

type PenCreateFormData = z.output<typeof penCreateSchema>
type PenEditFormData = z.output<typeof penEditSchema>

export function PenTab({ canManage }: { canManage: boolean }) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const { dialogState, confirm } = useConfirmDialog()
  const [editing, setEditing] = useState<PenDetails | null>(null)
  const [batchOpen, setBatchOpen] = useState(false)

  const createForm = useForm<z.input<typeof penCreateSchema>, unknown, PenCreateFormData>({
    resolver: zodResolver(penCreateSchema),
    defaultValues: { zone_id: '', code: '', name: '', capacity: 1 },
  })

  const editForm = useForm<z.input<typeof penEditSchema>, unknown, PenEditFormData>({
    resolver: zodResolver(penEditSchema),
    defaultValues: { code: '', name: '', capacity: 1, status: 'active' },
  })

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
    mutationFn: (data: PenCreateFormData) => facilityApi.createPen({ ...data, name: data.name || undefined }),
    onSuccess: () => { invalidate(); dialogs.close('create'); toast({ title: '已新增欄位' }) },
    onError: (err: unknown) => toast({ title: '新增失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: PenEditFormData }) => facilityApi.updatePen(id, { name: data.name || undefined, capacity: data.capacity, status: data.status }),
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
    editForm.reset({ code: p.code, name: p.name ?? '', capacity: p.capacity, status: p.status })
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

  const onCreateSubmit = createForm.handleSubmit(data => createMutation.mutate(data))
  const onEditSubmit = editForm.handleSubmit(data => {
    if (editing) updateMutation.mutate({ id: editing.id, data })
  })

  const statusBadgeVariant = (s: string) => s === 'active' ? 'default' : s === 'empty' ? 'secondary' : 'outline'

  const createZoneId = createForm.watch('zone_id')
  const editStatus = editForm.watch('status')

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-sm text-muted-foreground">共 {pens.length} 筆</span>
        {canManage && (
          <div className="flex gap-2">
            <Button size="sm" variant="outline" onClick={() => setBatchOpen(true)}>
              <LayoutGrid className="h-4 w-4 mr-1" /> 批次建立
            </Button>
            <Button size="sm" onClick={() => { createForm.reset({ zone_id: '', code: '', name: '', capacity: 1 }); dialogs.open('create') }}>
              <Plus className="h-4 w-4 mr-1" /> 新增欄位
            </Button>
          </div>
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
            {canManage && <TableHead className="w-24 text-right">操作</TableHead>}
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
              <TableCell className="text-sm text-muted-foreground">{p.building_code}棟 {p.zone_code}區</TableCell>
              <TableCell>{p.capacity}</TableCell>
              <TableCell>{p.current_count}</TableCell>
              <TableCell><Badge variant={statusBadgeVariant(p.status)}>{PEN_STATUS_NAMES[p.status] ?? p.status}</Badge></TableCell>
              {canManage && (
                <TableCell>
                  <div className="flex items-center justify-end gap-1">
                    <Button variant="ghost" size="icon" onClick={() => handleEdit(p)} aria-label="編輯"><Pencil className="h-4 w-4" /></Button>
                    <Button variant="ghost" size="icon" onClick={() => handleDelete(p)} aria-label="刪除"><Trash2 className="h-4 w-4 text-destructive" /></Button>
                  </div>
                </TableCell>
              )}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <PenLayoutPreview pens={pens} zones={zones} canManage={canManage} />

      <Dialog open={dialogs.isOpen('create')} onOpenChange={o => !o && dialogs.close('create')}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增欄位</DialogTitle></DialogHeader>
          <form onSubmit={onCreateSubmit} className="space-y-3">
            <FormField label="所屬區域" required error={createForm.formState.errors.zone_id?.message}>
              <Select value={createZoneId} onValueChange={v => createForm.setValue('zone_id', v, { shouldValidate: true })}>
                <SelectTrigger><SelectValue placeholder="選擇區域" /></SelectTrigger>
                <SelectContent>{zones.map(z => <SelectItem key={z.id} value={z.id}>{z.building_code}棟 {z.code}區 {z.name ? `(${z.name})` : ''}</SelectItem>)}</SelectContent>
              </Select>
            </FormField>
            <FormField label="代碼" required error={createForm.formState.errors.code?.message}>
              <Input {...createForm.register('code')} />
            </FormField>
            <FormField label="名稱">
              <Input {...createForm.register('name')} />
            </FormField>
            <FormField label="容量">
              <Input type="number" min={1} {...createForm.register('capacity', { valueAsNumber: true })} />
            </FormField>
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
          <DialogHeader><DialogTitle>編輯欄位</DialogTitle></DialogHeader>
          <form onSubmit={onEditSubmit} className="space-y-3">
            <FormField label="代碼">
              <Input {...editForm.register('code')} disabled />
            </FormField>
            <FormField label="名稱">
              <Input {...editForm.register('name')} />
            </FormField>
            <FormField label="容量">
              <Input type="number" min={1} {...editForm.register('capacity', { valueAsNumber: true })} />
            </FormField>
            <FormField label="狀態">
              <Select value={editStatus} onValueChange={v => editForm.setValue('status', v)}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>{Object.entries(PEN_STATUS_NAMES).map(([k, v]) => <SelectItem key={k} value={k}>{v}</SelectItem>)}</SelectContent>
              </Select>
            </FormField>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => dialogs.close('edit')}>{t('common.cancel')}</Button>
              <Button type="submit" disabled={updateMutation.isPending}>
                {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-1 animate-spin" />} 儲存
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <BatchCreatePenDialog open={batchOpen} onOpenChange={setBatchOpen} zones={zones} />
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
