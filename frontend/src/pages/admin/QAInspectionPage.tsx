/**
 * QA 稽查報告管理頁
 *
 * - 列表：稽查報告、篩選（類型、狀態）
 * - 新增 / 編輯報告（含稽查項目）
 * - 送出 / 關閉報告
 */

import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, ClipboardCheck, CheckCircle2, XCircle, MinusCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { Skeleton } from '@/components/ui/skeleton'
import { useAuthStore } from '@/stores/auth'
import {
  listInspections, createInspection, updateInspection,
  type QaInspectionWithInspector, type QaInspectionType, type QaItemResult,
} from '@/lib/api/qaPlan'

const INSPECTION_TYPE_LABELS: Record<string, string> = {
  protocol: '計畫書', equipment: '設備', facility: '設施',
  training: '訓練', general: '一般',
}

const STATUS_LABELS: Record<string, string> = {
  draft: '草稿', submitted: '已提交', closed: '已關閉',
}

const STATUS_VARIANTS: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
  draft: 'secondary', submitted: 'default', closed: 'outline',
}

const RESULT_ICONS = {
  pass: <CheckCircle2 className="h-4 w-4 text-green-600" />,
  fail: <XCircle className="h-4 w-4 text-destructive" />,
  not_applicable: <MinusCircle className="h-4 w-4 text-muted-foreground" />,
}

interface ItemRow {
  description: string
  result: QaItemResult
  remarks: string
}

interface InspectionForm {
  title: string
  inspection_type: QaInspectionType
  inspection_date: string
  findings: string
  conclusion: string
  items: ItemRow[]
}

const defaultForm = (): InspectionForm => ({
  title: '',
  inspection_type: 'general',
  inspection_date: new Date().toISOString().split('T')[0],
  findings: '',
  conclusion: '',
  items: [{ description: '', result: 'not_applicable', remarks: '' }],
})

export function QAInspectionPage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('qau.inspection.manage')

  const [open, setOpen] = useState(false)
  const [editId, setEditId] = useState<string | null>(null)
  const [form, setForm] = useState<InspectionForm>(defaultForm())
  const [filterType, setFilterType] = useState('')
  const [filterStatus, setFilterStatus] = useState('')

  const { data: inspections = [], isLoading } = useQuery({
    queryKey: ['qa-inspections', filterType, filterStatus],
    queryFn: () => listInspections({
      inspection_type: filterType || undefined,
      status: filterStatus || undefined,
    }),
  })

  const saveMutation = useMutation({
    mutationFn: () => {
      const payload = {
        title: form.title,
        inspection_type: form.inspection_type,
        inspection_date: form.inspection_date,
        findings: form.findings || undefined,
        conclusion: form.conclusion || undefined,
        items: form.items
          .filter(i => i.description.trim())
          .map((i, idx) => ({
            item_order: idx + 1,
            description: i.description,
            result: i.result,
            remarks: i.remarks || undefined,
          })),
      }
      return editId
        ? updateInspection(editId, payload)
        : createInspection(payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['qa-inspections'] })
      setOpen(false)
    },
  })

  const submitMutation = useMutation({
    mutationFn: (id: string) => updateInspection(id, { status: 'submitted' }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['qa-inspections'] }),
  })

  const openCreate = () => {
    setEditId(null)
    setForm(defaultForm())
    setOpen(true)
  }

  const openEdit = (row: QaInspectionWithInspector) => {
    setEditId(row.id)
    setForm({
      title: row.title,
      inspection_type: row.inspection_type,
      inspection_date: row.inspection_date,
      findings: row.findings ?? '',
      conclusion: row.conclusion ?? '',
      items: [{ description: '', result: 'not_applicable', remarks: '' }],
    })
    setOpen(true)
  }

  const setItem = (idx: number, field: keyof ItemRow, value: string) => {
    setForm(f => ({
      ...f,
      items: f.items.map((item, i) => i === idx ? { ...item, [field]: value } : item),
    }))
  }

  const addItem = () =>
    setForm(f => ({ ...f, items: [...f.items, { description: '', result: 'not_applicable', remarks: '' }] }))

  const removeItem = (idx: number) =>
    setForm(f => ({ ...f, items: f.items.filter((_, i) => i !== idx) }))

  return (
    <div className="space-y-6">
      <PageHeader
        title="稽查報告"
        description="GLP 合規：建立與管理 QAU 稽查報告"
        actions={canManage ? (
          <Button size="sm" onClick={openCreate}>
            <Plus className="h-4 w-4 mr-2" />
            新增稽查報告
          </Button>
        ) : undefined}
      />

      {/* 篩選 */}
      <div className="flex gap-3">
        <Select value={filterType} onValueChange={setFilterType}>
          <SelectTrigger className="w-36">
            <SelectValue placeholder="稽查類型" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="">全部類型</SelectItem>
            {Object.entries(INSPECTION_TYPE_LABELS).map(([v, l]) => (
              <SelectItem key={v} value={v}>{l}</SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select value={filterStatus} onValueChange={setFilterStatus}>
          <SelectTrigger className="w-32">
            <SelectValue placeholder="狀態" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="">全部狀態</SelectItem>
            {Object.entries(STATUS_LABELS).map(([v, l]) => (
              <SelectItem key={v} value={v}>{l}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {/* 列表 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ClipboardCheck className="h-5 w-5" />
            稽查報告列表
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <Skeleton variant="table" rows={5} />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>報告編號</TableHead>
                  <TableHead>標題</TableHead>
                  <TableHead>類型</TableHead>
                  <TableHead>稽查日期</TableHead>
                  <TableHead>稽查人員</TableHead>
                  <TableHead>狀態</TableHead>
                  {canManage && <TableHead className="w-32">操作</TableHead>}
                </TableRow>
              </TableHeader>
              <TableBody>
                {inspections.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={7} className="text-center text-muted-foreground py-8">
                      尚無稽查報告
                    </TableCell>
                  </TableRow>
                ) : inspections.map(row => (
                  <TableRow key={row.id}>
                    <TableCell className="font-mono text-sm">{row.inspection_number}</TableCell>
                    <TableCell>{row.title}</TableCell>
                    <TableCell>{INSPECTION_TYPE_LABELS[row.inspection_type] ?? row.inspection_type}</TableCell>
                    <TableCell>{row.inspection_date}</TableCell>
                    <TableCell>{row.inspector_name}</TableCell>
                    <TableCell>
                      <Badge variant={STATUS_VARIANTS[row.status] ?? 'secondary'}>
                        {STATUS_LABELS[row.status] ?? row.status}
                      </Badge>
                    </TableCell>
                    {canManage && (
                      <TableCell>
                        <div className="flex gap-1">
                          {row.status === 'draft' && (
                            <>
                              <Button size="sm" variant="ghost" onClick={() => openEdit(row)}>編輯</Button>
                              <Button size="sm" variant="ghost" onClick={() => submitMutation.mutate(row.id)}>送出</Button>
                            </>
                          )}
                          {row.status === 'submitted' && (
                            <Button size="sm" variant="ghost" onClick={() => submitMutation.mutate(row.id)}>關閉</Button>
                          )}
                        </div>
                      </TableCell>
                    )}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* 新增/編輯對話框 */}
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{editId ? '編輯稽查報告' : '新增稽查報告'}</DialogTitle>
          </DialogHeader>

          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="col-span-2 space-y-1">
                <Label>標題</Label>
                <Input value={form.title} onChange={e => setForm(f => ({ ...f, title: e.target.value }))} />
              </div>
              <div className="space-y-1">
                <Label>稽查類型</Label>
                <Select
                  value={form.inspection_type}
                  onValueChange={v => setForm(f => ({ ...f, inspection_type: v as QaInspectionType }))}
                >
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {Object.entries(INSPECTION_TYPE_LABELS).map(([v, l]) => (
                      <SelectItem key={v} value={v}>{l}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label>稽查日期</Label>
                <Input
                  type="date"
                  value={form.inspection_date}
                  onChange={e => setForm(f => ({ ...f, inspection_date: e.target.value }))}
                />
              </div>
              <div className="col-span-2 space-y-1">
                <Label>稽查發現</Label>
                <Textarea
                  rows={3}
                  value={form.findings}
                  onChange={e => setForm(f => ({ ...f, findings: e.target.value }))}
                />
              </div>
              <div className="col-span-2 space-y-1">
                <Label>結論</Label>
                <Textarea
                  rows={2}
                  value={form.conclusion}
                  onChange={e => setForm(f => ({ ...f, conclusion: e.target.value }))}
                />
              </div>
            </div>

            {/* 稽查項目 */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label>稽查項目</Label>
                <Button size="sm" variant="outline" onClick={addItem}>
                  <Plus className="h-3 w-3 mr-1" />
                  新增項目
                </Button>
              </div>
              {form.items.map((item, idx) => (
                <div key={idx} className="grid grid-cols-12 gap-2 items-start p-2 border rounded-md">
                  <div className="col-span-5 space-y-1">
                    <Label className="text-xs">稽查事項</Label>
                    <Input
                      placeholder="稽查事項描述"
                      value={item.description}
                      onChange={e => setItem(idx, 'description', e.target.value)}
                    />
                  </div>
                  <div className="col-span-3 space-y-1">
                    <Label className="text-xs">結果</Label>
                    <Select value={item.result} onValueChange={v => setItem(idx, 'result', v)}>
                      <SelectTrigger><SelectValue /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="pass">符合</SelectItem>
                        <SelectItem value="fail">不符合</SelectItem>
                        <SelectItem value="not_applicable">不適用</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="col-span-3 space-y-1">
                    <Label className="text-xs">備註</Label>
                    <Input value={item.remarks} onChange={e => setItem(idx, 'remarks', e.target.value)} />
                  </div>
                  <div className="col-span-1 pt-6">
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-8 w-8"
                      onClick={() => removeItem(idx)}
                    >
                      <XCircle className="h-4 w-4 text-muted-foreground" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setOpen(false)}>取消</Button>
            <Button
              onClick={() => saveMutation.mutate()}
              disabled={!form.title || saveMutation.isPending}
            >
              {saveMutation.isPending ? '儲存中…' : '儲存'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
