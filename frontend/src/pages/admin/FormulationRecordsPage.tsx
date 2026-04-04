import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import { useAuthStore } from '@/stores/auth'
import {
  listFormulationRecords,
  createFormulationRecord,
  type FormulationRecord,
} from '@/lib/api/glpCompliance'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter,
} from '@/components/ui/dialog'
import { Plus } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

const INITIAL = {
  product_id: '',
  protocol_id: '',
  formulation_date: '',
  batch_number: '',
  concentration: '',
  volume: '',
  expiry_date: '',
  notes: '',
}

export function FormulationRecordsPage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('formulation.record.manage')

  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState(INITIAL)

  const { data: records = [], isLoading } = useQuery({
    queryKey: ['formulation-records'],
    queryFn: () => listFormulationRecords(),
  })

  const createMutation = useMutation({
    mutationFn: () =>
      createFormulationRecord({
        product_id: form.product_id,
        protocol_id: form.protocol_id || undefined,
        formulation_date: form.formulation_date,
        batch_number: form.batch_number || undefined,
        concentration: form.concentration || undefined,
        volume: form.volume || undefined,
        expiry_date: form.expiry_date || undefined,
        notes: form.notes || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['formulation-records'] })
      setShowCreate(false)
      setForm(INITIAL)
      toast({ title: '配製紀錄已建立' })
    },
    onError: (err: unknown) =>
      toast({ title: '建立失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const set = (k: string, v: string) => setForm((f) => ({ ...f, [k]: v }))

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">配製紀錄</h1>
          <p className="text-muted-foreground">GLP 試驗物質配製紀錄管理</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            新增配製紀錄
          </Button>
        )}
      </div>

      <Card>
        <CardHeader />
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>產品</TableHead>
                <TableHead>配製日期</TableHead>
                <TableHead>批號</TableHead>
                <TableHead>濃度</TableHead>
                <TableHead>體積</TableHead>
                <TableHead>配製者</TableHead>
                <TableHead>驗證者</TableHead>
                <TableHead>效期</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow>
                  <TableCell colSpan={8} className="text-center py-8">載入中...</TableCell>
                </TableRow>
              ) : records.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={8} className="text-center py-8 text-muted-foreground">
                    尚無配製紀錄
                  </TableCell>
                </TableRow>
              ) : (
                records.map((r) => (
                  <TableRow key={r.id}>
                    <TableCell className="font-medium">{r.product_name ?? r.product_id}</TableCell>
                    <TableCell>{r.formulation_date}</TableCell>
                    <TableCell className="font-mono text-sm">{r.batch_number ?? '-'}</TableCell>
                    <TableCell>{r.concentration ?? '-'}</TableCell>
                    <TableCell>{r.volume ?? '-'}</TableCell>
                    <TableCell>{r.preparer_name ?? '-'}</TableCell>
                    <TableCell>{r.verified_by ? '已驗證' : '待驗證'}</TableCell>
                    <TableCell>{r.expiry_date ?? '-'}</TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新增配製紀錄</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">產品 ID *</label>
              <Input value={form.product_id} onChange={(e) => set('product_id', e.target.value)} placeholder="輸入產品 UUID" />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">計畫書 ID</label>
              <Input value={form.protocol_id} onChange={(e) => set('protocol_id', e.target.value)} placeholder="選填" />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">配製日期 *</label>
                <Input type="date" value={form.formulation_date} onChange={(e) => set('formulation_date', e.target.value)} />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">批號</label>
                <Input value={form.batch_number} onChange={(e) => set('batch_number', e.target.value)} />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">濃度</label>
                <Input value={form.concentration} onChange={(e) => set('concentration', e.target.value)} placeholder="e.g. 10 mg/mL" />
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">體積</label>
                <Input value={form.volume} onChange={(e) => set('volume', e.target.value)} placeholder="e.g. 500 mL" />
              </div>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">效期</label>
              <Input type="date" value={form.expiry_date} onChange={(e) => set('expiry_date', e.target.value)} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">備註</label>
              <Textarea value={form.notes} onChange={(e) => set('notes', e.target.value)} rows={2} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>取消</Button>
            <Button
              onClick={() => createMutation.mutate()}
              disabled={!form.product_id || !form.formulation_date || createMutation.isPending}
            >
              建立
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
