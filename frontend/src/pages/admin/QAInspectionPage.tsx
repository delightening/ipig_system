/**
 * QA 稽查報告管理頁
 *
 * - 列表：稽查報告、篩選（類型、狀態）
 * - 新增 / 編輯報告（含稽查項目）
 * - 送出 / 關閉報告
 */

import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, ClipboardCheck, XCircle } from 'lucide-react'
import { useTranslation } from 'react-i18next'

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
import { useAuthHasPermission } from '@/stores/auth'
import {
  listInspections, getInspection, createInspection, updateInspection,
  type QaInspectionWithInspector, type QaInspectionType, type QaItemResult,
  type QaInspectionStatus,
} from '@/lib/api/qaPlan'

// 值為 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const INSPECTION_TYPE_LABEL_KEYS: Record<string, string> = {
  protocol: 'adminGlp.shared.inspectionType.protocol',
  equipment: 'adminGlp.shared.equipment',
  facility: 'adminGlp.shared.facility',
  training: 'adminGlp.shared.inspectionType.training',
  general: 'adminGlp.shared.inspectionType.general',
}

const STATUS_LABEL_KEYS: Record<string, string> = {
  draft: 'adminGlp.shared.statusLabel.draft',
  submitted: 'adminGlp.shared.statusLabel.submitted',
  closed: 'adminGlp.qaInspection.status.closed',
}

const STATUS_VARIANTS: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
  draft: 'secondary', submitted: 'default', closed: 'outline',
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
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
  const canManage = hasPermission('qau.inspection.manage')

  const [open, setOpen] = useState(false)
  const [editId, setEditId] = useState<string | null>(null)
  const [form, setForm] = useState<InspectionForm>(defaultForm())
  const [filterType, setFilterType] = useState('all')
  const [filterStatus, setFilterStatus] = useState('all')

  const { data: inspections = [], isLoading } = useQuery({
    queryKey: ['qa-inspections', filterType, filterStatus],
    queryFn: () => listInspections({
      inspection_type: filterType !== 'all' ? filterType : undefined,
      status: filterStatus !== 'all' ? filterStatus : undefined,
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

  const changeStatusMutation = useMutation({
    mutationFn: ({ id, status }: { id: string; status: QaInspectionStatus }) =>
      updateInspection(id, { status }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['qa-inspections'] }),
  })

  const openCreate = () => {
    setEditId(null)
    setForm(defaultForm())
    setOpen(true)
  }

  const openEdit = async (row: QaInspectionWithInspector) => {
    const detail = await getInspection(row.id)
    setEditId(row.id)
    setForm({
      title: row.title,
      inspection_type: row.inspection_type,
      inspection_date: row.inspection_date,
      findings: row.findings ?? '',
      conclusion: row.conclusion ?? '',
      items: detail.items.length > 0
        ? detail.items.map(i => ({ description: i.description, result: i.result, remarks: i.remarks ?? '' }))
        : [{ description: '', result: 'not_applicable' as QaItemResult, remarks: '' }],
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
        title={t('adminGlp.qaInspection.title')}
        description={t('adminGlp.qaInspection.description')}
        actions={canManage ? (
          <Button size="sm" onClick={openCreate}>
            <Plus className="h-4 w-4 mr-2" />
            {t('adminGlp.qaInspection.create')}
          </Button>
        ) : undefined}
      />

      {/* 篩選 */}
      <div className="flex gap-3">
        <Select value={filterType} onValueChange={setFilterType}>
          <SelectTrigger className="w-36">
            <SelectValue placeholder={t('adminGlp.qaInspection.inspectionType')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t('adminGlp.qaInspection.allTypes')}</SelectItem>
            {Object.entries(INSPECTION_TYPE_LABEL_KEYS).map(([v, labelKey]) => (
              <SelectItem key={v} value={v}>{t(labelKey)}</SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select value={filterStatus} onValueChange={setFilterStatus}>
          <SelectTrigger className="w-32">
            <SelectValue placeholder={t('adminGlp.shared.status')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t('common.allStatus')}</SelectItem>
            {Object.entries(STATUS_LABEL_KEYS).map(([v, labelKey]) => (
              <SelectItem key={v} value={v}>{t(labelKey)}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {/* 列表 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ClipboardCheck className="h-5 w-5" />
            {t('adminGlp.qaInspection.listTitle')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <Skeleton variant="table" rows={5} />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t('adminGlp.shared.reportNumber')}</TableHead>
                  <TableHead>{t('adminGlp.shared.title')}</TableHead>
                  <TableHead>{t('adminGlp.shared.type')}</TableHead>
                  <TableHead>{t('adminGlp.qaInspection.inspectionDate')}</TableHead>
                  <TableHead>{t('adminGlp.qaInspection.inspector')}</TableHead>
                  <TableHead>{t('adminGlp.shared.status')}</TableHead>
                  {canManage && <TableHead className="w-32">{t('common.actions')}</TableHead>}
                </TableRow>
              </TableHeader>
              <TableBody>
                {inspections.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={7} className="text-center text-muted-foreground py-8">
                      {t('adminGlp.qaInspection.empty')}
                    </TableCell>
                  </TableRow>
                ) : inspections.map(row => (
                  <TableRow key={row.id}>
                    <TableCell className="font-mono text-sm">{row.inspection_number}</TableCell>
                    <TableCell>{row.title}</TableCell>
                    <TableCell>
                      {INSPECTION_TYPE_LABEL_KEYS[row.inspection_type]
                        ? t(INSPECTION_TYPE_LABEL_KEYS[row.inspection_type])
                        : row.inspection_type}
                    </TableCell>
                    <TableCell>{row.inspection_date}</TableCell>
                    <TableCell>{row.inspector_name}</TableCell>
                    <TableCell>
                      <Badge variant={STATUS_VARIANTS[row.status] ?? 'secondary'}>
                        {STATUS_LABEL_KEYS[row.status] ? t(STATUS_LABEL_KEYS[row.status]) : row.status}
                      </Badge>
                    </TableCell>
                    {canManage && (
                      <TableCell>
                        <div className="flex gap-1">
                          {row.status === 'draft' && (
                            <>
                              <Button size="sm" variant="ghost" onClick={() => openEdit(row)}>{t('common.edit')}</Button>
                              <Button size="sm" variant="ghost" onClick={() => changeStatusMutation.mutate({ id: row.id, status: 'submitted' })}>{t('adminGlp.qaInspection.submit')}</Button>
                            </>
                          )}
                          {row.status === 'submitted' && (
                            <Button size="sm" variant="ghost" onClick={() => changeStatusMutation.mutate({ id: row.id, status: 'closed' })}>{t('adminGlp.qaInspection.close')}</Button>
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
        <DialogContent size="lg" className="max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{editId ? t('adminGlp.qaInspection.edit') : t('adminGlp.qaInspection.create')}</DialogTitle>
          </DialogHeader>

          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="col-span-2 space-y-1">
                <Label>{t('adminGlp.shared.title')}</Label>
                <Input value={form.title} onChange={e => setForm(f => ({ ...f, title: e.target.value }))} />
              </div>
              <div className="space-y-1">
                <Label>{t('adminGlp.qaInspection.inspectionType')}</Label>
                <Select
                  value={form.inspection_type}
                  onValueChange={v => setForm(f => ({ ...f, inspection_type: v as QaInspectionType }))}
                >
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {Object.entries(INSPECTION_TYPE_LABEL_KEYS).map(([v, labelKey]) => (
                      <SelectItem key={v} value={v}>{t(labelKey)}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label>{t('adminGlp.qaInspection.inspectionDate')}</Label>
                <Input
                  type="date"
                  value={form.inspection_date}
                  onChange={e => setForm(f => ({ ...f, inspection_date: e.target.value }))}
                />
              </div>
              <div className="col-span-2 space-y-1">
                <Label>{t('adminGlp.qaInspection.findings')}</Label>
                <Textarea
                  rows={3}
                  value={form.findings}
                  onChange={e => setForm(f => ({ ...f, findings: e.target.value }))}
                />
              </div>
              <div className="col-span-2 space-y-1">
                <Label>{t('adminGlp.qaInspection.conclusion')}</Label>
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
                <Label>{t('adminGlp.qaInspection.items')}</Label>
                <Button size="sm" variant="outline" onClick={addItem}>
                  <Plus className="h-3 w-3 mr-1" />
                  {t('common.addItem')}
                </Button>
              </div>
              {form.items.map((item, idx) => (
                <div key={idx} className="grid grid-cols-12 gap-2 items-start p-2 border rounded-md">
                  <div className="col-span-5 space-y-1">
                    <Label className="text-xs">{t('adminGlp.qaInspection.itemDescription')}</Label>
                    <Input
                      placeholder={t('adminGlp.qaInspection.itemPlaceholder')}
                      value={item.description}
                      onChange={e => setItem(idx, 'description', e.target.value)}
                    />
                  </div>
                  <div className="col-span-3 space-y-1">
                    <Label className="text-xs">{t('adminGlp.qaInspection.result')}</Label>
                    <Select value={item.result} onValueChange={v => setItem(idx, 'result', v)}>
                      <SelectTrigger><SelectValue /></SelectTrigger>
                      <SelectContent>
                        <SelectItem value="pass">{t('adminGlp.qaInspection.resultPass')}</SelectItem>
                        <SelectItem value="fail">{t('adminGlp.qaInspection.resultFail')}</SelectItem>
                        <SelectItem value="not_applicable">{t('adminGlp.qaInspection.resultNotApplicable')}</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="col-span-3 space-y-1">
                    <Label className="text-xs">{t('adminGlp.shared.remarks')}</Label>
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
            <Button variant="outline" onClick={() => setOpen(false)}>{t('common.cancel')}</Button>
            <Button
              onClick={() => saveMutation.mutate()}
              disabled={!form.title || saveMutation.isPending}
            >
              {saveMutation.isPending ? t('adminGlp.shared.saving') : t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
