/**
 * QA 稽查排程管理頁
 *
 * - 年度稽查計畫列表
 * - 建立年度計畫（含排程項目）
 * - 展開查看各項目，更新執行狀態
 */

import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, Calendar, ChevronDown, ChevronRight } from 'lucide-react'

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
  listSchedules, getSchedule, createSchedule, updateScheduleItem,
  type QaAuditSchedule, type QaInspectionType, type QaScheduleType,
  type QaScheduleItemStatus,
} from '@/lib/api/qaPlan'

const SCHEDULE_STATUS_LABELS: Record<string, string> = {
  planned: '計畫中', in_progress: '進行中', completed: '已完成', cancelled: '已取消',
}

const ITEM_STATUS_LABELS: Record<QaScheduleItemStatus, string> = {
  planned: '計畫中', in_progress: '進行中', completed: '已完成',
  cancelled: '已取消', overdue: '逾期',
}

const ITEM_STATUS_VARIANTS: Record<QaScheduleItemStatus, 'default' | 'secondary' | 'destructive' | 'outline'> = {
  planned: 'secondary', in_progress: 'default', completed: 'outline',
  cancelled: 'outline', overdue: 'destructive',
}

const INSPECTION_TYPE_LABELS: Record<QaInspectionType, string> = {
  protocol: '計畫書', equipment: '設備', facility: '設施', training: '訓練', general: '一般',
}

interface ScheduleItemDraft {
  inspection_type: QaInspectionType
  title: string
  planned_date: string
  notes: string
}

interface ScheduleForm {
  year: number
  title: string
  schedule_type: QaScheduleType
  description: string
  items: ScheduleItemDraft[]
}

const defaultForm = (): ScheduleForm => ({
  year: new Date().getFullYear(),
  title: `${new Date().getFullYear()} 年度稽查計畫`,
  schedule_type: 'annual',
  description: '',
  items: [{ inspection_type: 'general', title: '', planned_date: '', notes: '' }],
})

export function QASchedulePage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('qau.schedule.manage')

  const [filterYear, setFilterYear] = useState<string>(String(new Date().getFullYear()))
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [open, setOpen] = useState(false)
  const [form, setForm] = useState<ScheduleForm>(defaultForm())
  const [updateItemId, setUpdateItemId] = useState<string | null>(null)
  const [itemStatusUpdate, setItemStatusUpdate] = useState<QaScheduleItemStatus>('in_progress')
  const [itemActualDate, setItemActualDate] = useState('')

  const { data: scheduleList = [], isLoading } = useQuery({
    queryKey: ['qa-schedules', filterYear],
    queryFn: () => listSchedules({ year: filterYear ? Number(filterYear) : undefined }),
  })

  const { data: scheduleDetail } = useQuery({
    queryKey: ['qa-schedule-detail', expandedId],
    queryFn: () => getSchedule(expandedId!),
    enabled: !!expandedId,
  })

  const createMutation = useMutation({
    mutationFn: () => createSchedule({
      year: form.year,
      title: form.title,
      schedule_type: form.schedule_type,
      description: form.description || undefined,
      items: form.items
        .filter(i => i.title.trim() && i.planned_date)
        .map(i => ({
          inspection_type: i.inspection_type,
          title: i.title,
          planned_date: i.planned_date,
          notes: i.notes || undefined,
        })),
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['qa-schedules'] })
      setOpen(false)
    },
  })

  const updateItemMutation = useMutation({
    mutationFn: () => updateScheduleItem(expandedId!, updateItemId!, {
      status: itemStatusUpdate,
      actual_date: itemActualDate || undefined,
    }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['qa-schedule-detail', expandedId] })
      setUpdateItemId(null)
    },
  })

  const setItem = (idx: number, field: keyof ScheduleItemDraft, value: string) => {
    setForm(f => ({
      ...f,
      items: f.items.map((item, i) => i === idx ? { ...item, [field]: value } : item),
    }))
  }

  const addItem = () =>
    setForm(f => ({
      ...f,
      items: [...f.items, { inspection_type: 'general', title: '', planned_date: '', notes: '' }],
    }))

  const removeItem = (idx: number) =>
    setForm(f => ({ ...f, items: f.items.filter((_, i) => i !== idx) }))

  const openUpdateItem = (itemId: string, currentStatus: QaScheduleItemStatus) => {
    setUpdateItemId(itemId)
    setItemStatusUpdate(currentStatus)
    setItemActualDate('')
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="稽查排程"
        description="年度 QA 稽查計畫排程與執行追蹤"
        actions={canManage ? (
          <Button size="sm" onClick={() => { setForm(defaultForm()); setOpen(true) }}>
            <Plus className="h-4 w-4 mr-2" />
            建立年度計畫
          </Button>
        ) : undefined}
      />

      {/* 年份篩選 */}
      <div className="flex gap-3">
        <Input
          className="w-28"
          type="number"
          value={filterYear}
          onChange={e => setFilterYear(e.target.value)}
          placeholder="年份"
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Calendar className="h-5 w-5" />
            稽查排程列表
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <Skeleton variant="table" rows={5} />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-8" />
                  <TableHead>年度</TableHead>
                  <TableHead>計畫名稱</TableHead>
                  <TableHead>類型</TableHead>
                  <TableHead>狀態</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {scheduleList.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                      尚無稽查排程
                    </TableCell>
                  </TableRow>
                ) : scheduleList.map((row: QaAuditSchedule) => (
                  <>
                    <TableRow
                      key={row.id}
                      className="cursor-pointer"
                      onClick={() => setExpandedId(expandedId === row.id ? null : row.id)}
                    >
                      <TableCell>
                        {expandedId === row.id
                          ? <ChevronDown className="h-4 w-4" />
                          : <ChevronRight className="h-4 w-4" />
                        }
                      </TableCell>
                      <TableCell>{row.year}</TableCell>
                      <TableCell>{row.title}</TableCell>
                      <TableCell className="text-sm text-muted-foreground">
                        {row.schedule_type === 'annual' ? '年度' : row.schedule_type === 'periodic' ? '定期' : '不定期'}
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline">{SCHEDULE_STATUS_LABELS[row.status] ?? row.status}</Badge>
                      </TableCell>
                    </TableRow>

                    {/* 展開排程項目 */}
                    {expandedId === row.id && (
                      <TableRow key={`${row.id}-detail`}>
                        <TableCell colSpan={5} className="bg-muted/30 p-4">
                          <div className="space-y-3">
                            <span className="font-medium text-sm">排程項目</span>
                            {!scheduleDetail?.items?.length ? (
                              <p className="text-sm text-muted-foreground">尚無排程項目</p>
                            ) : (
                              <table className="w-full text-sm">
                                <thead>
                                  <tr className="text-muted-foreground">
                                    <th className="text-left font-normal pb-1">標題</th>
                                    <th className="text-left font-normal pb-1">類型</th>
                                    <th className="text-left font-normal pb-1">計畫日期</th>
                                    <th className="text-left font-normal pb-1">實際日期</th>
                                    <th className="text-left font-normal pb-1">負責人</th>
                                    <th className="text-left font-normal pb-1">狀態</th>
                                    {canManage && <th />}
                                  </tr>
                                </thead>
                                <tbody>
                                  {scheduleDetail.items.map(item => (
                                    <tr key={item.id} className="border-t">
                                      <td className="py-1 pr-4">{item.title}</td>
                                      <td className="py-1 pr-4">
                                        {INSPECTION_TYPE_LABELS[item.inspection_type]}
                                      </td>
                                      <td className="py-1 pr-4">{item.planned_date}</td>
                                      <td className="py-1 pr-4">{item.actual_date ?? '—'}</td>
                                      <td className="py-1 pr-4">{item.responsible_name ?? '—'}</td>
                                      <td className="py-1 pr-4">
                                        <Badge variant={ITEM_STATUS_VARIANTS[item.status]} className="text-xs">
                                          {ITEM_STATUS_LABELS[item.status]}
                                        </Badge>
                                      </td>
                                      {canManage && (
                                        <td className="py-1">
                                          {updateItemId === item.id ? (
                                            <div className="flex gap-2 items-center">
                                              <Select
                                                value={itemStatusUpdate}
                                                onValueChange={v => setItemStatusUpdate(v as QaScheduleItemStatus)}
                                              >
                                                <SelectTrigger className="h-7 w-28 text-xs">
                                                  <SelectValue />
                                                </SelectTrigger>
                                                <SelectContent>
                                                  {Object.entries(ITEM_STATUS_LABELS).map(([v, l]) => (
                                                    <SelectItem key={v} value={v}>{l}</SelectItem>
                                                  ))}
                                                </SelectContent>
                                              </Select>
                                              <Input
                                                type="date"
                                                className="h-7 w-36 text-xs"
                                                value={itemActualDate}
                                                onChange={e => setItemActualDate(e.target.value)}
                                              />
                                              <Button size="sm" className="h-7 text-xs" onClick={() => updateItemMutation.mutate()}>
                                                確認
                                              </Button>
                                              <Button size="sm" variant="ghost" className="h-7 text-xs" onClick={() => setUpdateItemId(null)}>
                                                取消
                                              </Button>
                                            </div>
                                          ) : (
                                            <Button size="sm" variant="ghost" onClick={() => openUpdateItem(item.id, item.status)}>
                                              更新
                                            </Button>
                                          )}
                                        </td>
                                      )}
                                    </tr>
                                  ))}
                                </tbody>
                              </table>
                            )}
                          </div>
                        </TableCell>
                      </TableRow>
                    )}
                  </>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* 建立年度計畫對話框 */}
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>建立年度稽查計畫</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="grid grid-cols-3 gap-3">
              <div className="space-y-1">
                <Label>年份</Label>
                <Input
                  type="number"
                  value={form.year}
                  onChange={e => setForm(f => ({ ...f, year: Number(e.target.value) }))}
                />
              </div>
              <div className="col-span-2 space-y-1">
                <Label>計畫名稱</Label>
                <Input value={form.title} onChange={e => setForm(f => ({ ...f, title: e.target.value }))} />
              </div>
            </div>
            <div className="space-y-1">
              <Label>說明</Label>
              <Textarea rows={2} value={form.description} onChange={e => setForm(f => ({ ...f, description: e.target.value }))} />
            </div>

            {/* 排程項目 */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label>排程項目</Label>
                <Button size="sm" variant="outline" onClick={addItem}>
                  <Plus className="h-3 w-3 mr-1" />
                  新增項目
                </Button>
              </div>
              {form.items.map((item, idx) => (
                <div key={idx} className="grid grid-cols-12 gap-2 items-end p-2 border rounded-md">
                  <div className="col-span-4 space-y-1">
                    <Label className="text-xs">稽查標題</Label>
                    <Input value={item.title} onChange={e => setItem(idx, 'title', e.target.value)} />
                  </div>
                  <div className="col-span-2 space-y-1">
                    <Label className="text-xs">類型</Label>
                    <Select value={item.inspection_type} onValueChange={v => setItem(idx, 'inspection_type', v)}>
                      <SelectTrigger><SelectValue /></SelectTrigger>
                      <SelectContent>
                        {Object.entries(INSPECTION_TYPE_LABELS).map(([v, l]) => (
                          <SelectItem key={v} value={v}>{l}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="col-span-3 space-y-1">
                    <Label className="text-xs">計畫日期</Label>
                    <Input type="date" value={item.planned_date} onChange={e => setItem(idx, 'planned_date', e.target.value)} />
                  </div>
                  <div className="col-span-2 space-y-1">
                    <Label className="text-xs">備註</Label>
                    <Input value={item.notes} onChange={e => setItem(idx, 'notes', e.target.value)} />
                  </div>
                  <div className="col-span-1">
                    <Button size="icon" variant="ghost" className="h-9 w-9" onClick={() => removeItem(idx)}>
                      ✕
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setOpen(false)}>取消</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.title || createMutation.isPending}>
              {createMutation.isPending ? '建立中…' : '建立計畫'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
