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
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'

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
  listSchedules, getSchedule, createSchedule, updateScheduleItem,
  type QaAuditSchedule, type QaInspectionType, type QaScheduleType,
  type QaScheduleItemStatus,
} from '@/lib/api/qaPlan'

// 值為 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const SCHEDULE_STATUS_LABEL_KEYS: Record<string, string> = {
  planned: 'adminGlp.qaSchedule.status.planned',
  in_progress: 'adminGlp.shared.statusLabel.inProgress',
  completed: 'adminGlp.shared.statusLabel.completed',
  cancelled: 'adminGlp.shared.statusLabel.cancelled',
}

const ITEM_STATUS_LABEL_KEYS: Record<QaScheduleItemStatus, string> = {
  planned: 'adminGlp.qaSchedule.status.planned',
  in_progress: 'adminGlp.shared.statusLabel.inProgress',
  completed: 'adminGlp.shared.statusLabel.completed',
  cancelled: 'adminGlp.shared.statusLabel.cancelled',
  overdue: 'adminGlp.shared.statusLabel.overdue',
}

const ITEM_STATUS_VARIANTS: Record<QaScheduleItemStatus, 'default' | 'secondary' | 'destructive' | 'outline'> = {
  planned: 'secondary', in_progress: 'default', completed: 'outline',
  cancelled: 'outline', overdue: 'destructive',
}

const INSPECTION_TYPE_LABEL_KEYS: Record<QaInspectionType, string> = {
  protocol: 'adminGlp.shared.inspectionType.protocol',
  equipment: 'adminGlp.shared.equipment',
  facility: 'adminGlp.shared.facility',
  training: 'adminGlp.shared.inspectionType.training',
  general: 'adminGlp.shared.inspectionType.general',
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

const defaultForm = (t: TFunction): ScheduleForm => ({
  year: new Date().getFullYear(),
  // 預設標題會寫進資料庫，固定中文（使用者裁定 2026-09-19）
  title: t('adminGlp.qaSchedule.defaultTitle', { lng: 'zh-TW', year: new Date().getFullYear() }),
  schedule_type: 'annual',
  description: '',
  items: [{ inspection_type: 'general', title: '', planned_date: '', notes: '' }],
})

export function QASchedulePage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
  const canManage = hasPermission('qau.schedule.manage')

  const [filterYear, setFilterYear] = useState<string>(String(new Date().getFullYear()))
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [open, setOpen] = useState(false)
  const [form, setForm] = useState<ScheduleForm>(() => defaultForm(t))
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
        title={t('adminGlp.qaSchedule.title')}
        description={t('adminGlp.qaSchedule.description')}
        actions={canManage ? (
          <Button size="sm" onClick={() => { setForm(defaultForm(t)); setOpen(true) }}>
            <Plus className="h-4 w-4 mr-2" />
            {t('adminGlp.qaSchedule.create')}
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
          placeholder={t('adminGlp.qaSchedule.year')}
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Calendar className="h-5 w-5" />
            {t('adminGlp.qaSchedule.listTitle')}
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
                  <TableHead>{t('adminGlp.qaSchedule.col.year')}</TableHead>
                  <TableHead>{t('adminGlp.qaSchedule.col.planName')}</TableHead>
                  <TableHead>{t('adminGlp.shared.type')}</TableHead>
                  <TableHead>{t('adminGlp.shared.status')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {scheduleList.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={5} className="text-center text-muted-foreground py-8">
                      {t('adminGlp.qaSchedule.empty')}
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
                        {row.schedule_type === 'annual'
                          ? t('adminGlp.qaSchedule.scheduleType.annual')
                          : row.schedule_type === 'periodic'
                            ? t('adminGlp.qaSchedule.scheduleType.periodic')
                            : t('adminGlp.qaSchedule.scheduleType.adhoc')}
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline">
                          {SCHEDULE_STATUS_LABEL_KEYS[row.status] ? t(SCHEDULE_STATUS_LABEL_KEYS[row.status]) : row.status}
                        </Badge>
                      </TableCell>
                    </TableRow>

                    {/* 展開排程項目 */}
                    {expandedId === row.id && (
                      <TableRow key={`${row.id}-detail`}>
                        <TableCell colSpan={5} className="bg-muted/30 p-4">
                          <div className="space-y-3">
                            <span className="font-medium text-sm">{t('adminGlp.qaSchedule.items')}</span>
                            {!scheduleDetail?.items?.length ? (
                              <p className="text-sm text-muted-foreground">{t('adminGlp.qaSchedule.noItems')}</p>
                            ) : (
                              <Table className="w-full text-sm">
                                <TableHeader>
                                  <TableRow className="text-muted-foreground">
                                    <TableHead className="text-left font-normal pb-1">{t('adminGlp.shared.title')}</TableHead>
                                    <TableHead className="text-left font-normal pb-1">{t('adminGlp.shared.type')}</TableHead>
                                    <TableHead className="text-left font-normal pb-1">{t('adminGlp.qaSchedule.col.plannedDate')}</TableHead>
                                    <TableHead className="text-left font-normal pb-1">{t('adminGlp.qaSchedule.col.actualDate')}</TableHead>
                                    <TableHead className="text-left font-normal pb-1">{t('adminGlp.shared.owner')}</TableHead>
                                    <TableHead className="text-left font-normal pb-1">{t('adminGlp.shared.status')}</TableHead>
                                    {canManage && <TableHead />}
                                  </TableRow>
                                </TableHeader>
                                <TableBody>
                                  {scheduleDetail.items.map(item => (
                                    <TableRow key={item.id} className="border-t">
                                      <TableCell className="py-1 pr-4">{item.title}</TableCell>
                                      <TableCell className="py-1 pr-4">
                                        {INSPECTION_TYPE_LABEL_KEYS[item.inspection_type]
                                          ? t(INSPECTION_TYPE_LABEL_KEYS[item.inspection_type])
                                          : undefined}
                                      </TableCell>
                                      <TableCell className="py-1 pr-4">{item.planned_date}</TableCell>
                                      <TableCell className="py-1 pr-4">{item.actual_date ?? '—'}</TableCell>
                                      <TableCell className="py-1 pr-4">{item.responsible_name ?? '—'}</TableCell>
                                      <TableCell className="py-1 pr-4">
                                        <Badge variant={ITEM_STATUS_VARIANTS[item.status]} className="text-xs">
                                          {t(ITEM_STATUS_LABEL_KEYS[item.status])}
                                        </Badge>
                                      </TableCell>
                                      {canManage && (
                                        <TableCell className="py-1">
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
                                                  {Object.entries(ITEM_STATUS_LABEL_KEYS).map(([v, labelKey]) => (
                                                    <SelectItem key={v} value={v}>{t(labelKey)}</SelectItem>
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
                                                {t('common.confirm')}
                                              </Button>
                                              <Button size="sm" variant="ghost" className="h-7 text-xs" onClick={() => setUpdateItemId(null)}>
                                                {t('common.cancel')}
                                              </Button>
                                            </div>
                                          ) : (
                                            <Button size="sm" variant="ghost" onClick={() => openUpdateItem(item.id, item.status)}>
                                              {t('common.update')}
                                            </Button>
                                          )}
                                        </TableCell>
                                      )}
                                    </TableRow>
                                  ))}
                                </TableBody>
                              </Table>
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
        <DialogContent size="lg" className="max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{t('adminGlp.qaSchedule.dialog.title')}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="grid grid-cols-3 gap-3">
              <div className="space-y-1">
                <Label>{t('adminGlp.qaSchedule.year')}</Label>
                <Input
                  type="number"
                  value={form.year}
                  onChange={e => setForm(f => ({ ...f, year: Number(e.target.value) }))}
                />
              </div>
              <div className="col-span-2 space-y-1">
                <Label>{t('adminGlp.qaSchedule.col.planName')}</Label>
                <Input value={form.title} onChange={e => setForm(f => ({ ...f, title: e.target.value }))} />
              </div>
            </div>
            <div className="space-y-1">
              <Label>{t('adminGlp.qaSchedule.dialog.explanation')}</Label>
              <Textarea rows={2} value={form.description} onChange={e => setForm(f => ({ ...f, description: e.target.value }))} />
            </div>

            {/* 排程項目 */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label>{t('adminGlp.qaSchedule.items')}</Label>
                <Button size="sm" variant="outline" onClick={addItem}>
                  <Plus className="h-3 w-3 mr-1" />
                  {t('common.addItem')}
                </Button>
              </div>
              {form.items.map((item, idx) => (
                <div key={idx} className="grid grid-cols-12 gap-2 items-end p-2 border rounded-md">
                  <div className="col-span-4 space-y-1">
                    <Label className="text-xs">{t('adminGlp.qaSchedule.dialog.inspectionTitle')}</Label>
                    <Input value={item.title} onChange={e => setItem(idx, 'title', e.target.value)} />
                  </div>
                  <div className="col-span-2 space-y-1">
                    <Label className="text-xs">{t('adminGlp.shared.type')}</Label>
                    <Select value={item.inspection_type} onValueChange={v => setItem(idx, 'inspection_type', v)}>
                      <SelectTrigger><SelectValue /></SelectTrigger>
                      <SelectContent>
                        {Object.entries(INSPECTION_TYPE_LABEL_KEYS).map(([v, labelKey]) => (
                          <SelectItem key={v} value={v}>{t(labelKey)}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="col-span-3 space-y-1">
                    <Label className="text-xs">{t('adminGlp.qaSchedule.col.plannedDate')}</Label>
                    <Input type="date" value={item.planned_date} onChange={e => setItem(idx, 'planned_date', e.target.value)} />
                  </div>
                  <div className="col-span-2 space-y-1">
                    <Label className="text-xs">{t('adminGlp.shared.remarks')}</Label>
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
            <Button variant="outline" onClick={() => setOpen(false)}>{t('common.cancel')}</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.title || createMutation.isPending}>
              {createMutation.isPending ? t('adminGlp.qaSchedule.dialog.creating') : t('adminGlp.qaSchedule.dialog.createPlan')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
