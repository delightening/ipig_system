/**
 * QA 不符合事項（NC）管理頁
 *
 * - NC 列表，篩選嚴重度、狀態
 * - 新增 / 更新 NC
 * - 點開 NC 可查看 CAPA 並新增 / 更新
 */

import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, AlertTriangle, ChevronDown, ChevronRight } from 'lucide-react'

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
  listNonConformances, getNonConformance, createNonConformance,
  updateNonConformance, createCapa, updateCapa,
  type QaNonConformanceWithDetails, type NcSeverity, type NcSource,
  type NcStatus, type CapaActionType, type CapaStatus, type QaCapa,
} from '@/lib/api/qaPlan'

const SEVERITY_LABELS: Record<NcSeverity, string> = {
  critical: '重大', major: '主要', minor: '輕微',
}

const SEVERITY_VARIANTS: Record<NcSeverity, 'default' | 'secondary' | 'destructive' | 'outline'> = {
  critical: 'destructive', major: 'default', minor: 'secondary',
}

const NC_STATUS_LABELS: Record<NcStatus, string> = {
  open: '開啟', in_progress: '處理中', pending_verification: '待驗證', closed: '已結案',
}

const SOURCE_LABELS: Record<NcSource, string> = {
  inspection: '稽查發現', observation: '觀察', external_audit: '外部稽核', self_report: '自主回報',
}

const CAPA_STATUS_LABELS: Record<CapaStatus, string> = {
  open: '開啟', in_progress: '進行中', completed: '已完成', verified: '已驗證',
}

interface NcForm {
  title: string
  description: string
  severity: NcSeverity
  source: NcSource
  assignee_id: string
  due_date: string
}

interface CapaForm {
  action_type: CapaActionType
  description: string
  assignee_id: string
  due_date: string
}

export function QANonConformancePage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('qau.nc.manage')

  const [filterSeverity, setFilterSeverity] = useState('')
  const [filterStatus, setFilterStatus] = useState('')
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [ncDialogOpen, setNcDialogOpen] = useState(false)
  const [capaDialogOpen, setCapaDialogOpen] = useState(false)
  const [editNcId, setEditNcId] = useState<string | null>(null)
  const [editCapaId, setEditCapaId] = useState<string | null>(null)
  const [ncForm, setNcForm] = useState<NcForm>({
    title: '', description: '', severity: 'minor', source: 'observation', assignee_id: '', due_date: '',
  })
  const [capaForm, setCapaForm] = useState<CapaForm>({
    action_type: 'corrective', description: '', assignee_id: '', due_date: '',
  })

  const { data: ncList = [], isLoading } = useQuery({
    queryKey: ['qa-nc', filterSeverity, filterStatus],
    queryFn: () => listNonConformances({
      severity: filterSeverity || undefined,
      status: filterStatus || undefined,
    }),
  })

  const { data: ncDetail } = useQuery({
    queryKey: ['qa-nc-detail', expandedId],
    queryFn: () => getNonConformance(expandedId!),
    enabled: !!expandedId,
  })

  const saveMutation = useMutation({
    mutationFn: () => editNcId
      ? updateNonConformance(editNcId, ncForm)
      : createNonConformance(ncForm),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['qa-nc'] })
      setNcDialogOpen(false)
    },
  })

  const saveCapaMutation = useMutation({
    mutationFn: () => {
      const payload = {
        action_type: capaForm.action_type,
        description: capaForm.description,
        assignee_id: capaForm.assignee_id || undefined,
        due_date: capaForm.due_date || undefined,
      }
      return editCapaId
        ? updateCapa(expandedId!, editCapaId, payload)
        : createCapa(expandedId!, payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['qa-nc-detail', expandedId] })
      setCapaDialogOpen(false)
    },
  })

  const openCreateNc = () => {
    setEditNcId(null)
    setNcForm({ title: '', description: '', severity: 'minor', source: 'observation', assignee_id: '', due_date: '' })
    setNcDialogOpen(true)
  }

  const openEditNc = (row: QaNonConformanceWithDetails) => {
    setEditNcId(row.id)
    setNcForm({
      title: row.title,
      description: row.description,
      severity: row.severity,
      source: row.source,
      assignee_id: row.assignee_id ?? '',
      due_date: row.due_date ?? '',
    })
    setNcDialogOpen(true)
  }

  const openAddCapa = () => {
    setEditCapaId(null)
    setCapaForm({ action_type: 'corrective', description: '', assignee_id: '', due_date: '' })
    setCapaDialogOpen(true)
  }

  const openEditCapa = (capa: QaCapa) => {
    setEditCapaId(capa.id)
    setCapaForm({
      action_type: capa.action_type,
      description: capa.description,
      assignee_id: capa.assignee_id ?? '',
      due_date: capa.due_date ?? '',
    })
    setCapaDialogOpen(true)
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="不符合事項（NC）"
        description="記錄與追蹤不符合事項及矯正預防行動（CAPA）"
        actions={canManage ? (
          <Button size="sm" onClick={openCreateNc}>
            <Plus className="h-4 w-4 mr-2" />
            新增 NC
          </Button>
        ) : undefined}
      />

      <div className="flex gap-3">
        <Select value={filterSeverity} onValueChange={setFilterSeverity}>
          <SelectTrigger className="w-32">
            <SelectValue placeholder="嚴重度" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="">全部</SelectItem>
            <SelectItem value="critical">重大</SelectItem>
            <SelectItem value="major">主要</SelectItem>
            <SelectItem value="minor">輕微</SelectItem>
          </SelectContent>
        </Select>
        <Select value={filterStatus} onValueChange={setFilterStatus}>
          <SelectTrigger className="w-36">
            <SelectValue placeholder="狀態" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="">全部狀態</SelectItem>
            {Object.entries(NC_STATUS_LABELS).map(([v, l]) => (
              <SelectItem key={v} value={v}>{l}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5" />
            不符合事項列表
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
                  <TableHead>NC 編號</TableHead>
                  <TableHead>標題</TableHead>
                  <TableHead>嚴重度</TableHead>
                  <TableHead>來源</TableHead>
                  <TableHead>負責人</TableHead>
                  <TableHead>截止日</TableHead>
                  <TableHead>狀態</TableHead>
                  {canManage && <TableHead className="w-20">操作</TableHead>}
                </TableRow>
              </TableHeader>
              <TableBody>
                {ncList.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={9} className="text-center text-muted-foreground py-8">
                      尚無不符合事項
                    </TableCell>
                  </TableRow>
                ) : ncList.map(row => (
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
                      <TableCell className="font-mono text-sm">{row.nc_number}</TableCell>
                      <TableCell>{row.title}</TableCell>
                      <TableCell>
                        <Badge variant={SEVERITY_VARIANTS[row.severity]}>
                          {SEVERITY_LABELS[row.severity]}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-sm text-muted-foreground">
                        {SOURCE_LABELS[row.source]}
                      </TableCell>
                      <TableCell>{row.assignee_name ?? '—'}</TableCell>
                      <TableCell>{row.due_date ?? '—'}</TableCell>
                      <TableCell>
                        <Badge variant="outline">{NC_STATUS_LABELS[row.status]}</Badge>
                      </TableCell>
                      {canManage && (
                        <TableCell onClick={e => e.stopPropagation()}>
                          <Button size="sm" variant="ghost" onClick={() => openEditNc(row)}>編輯</Button>
                        </TableCell>
                      )}
                    </TableRow>

                    {/* CAPA 展開列 */}
                    {expandedId === row.id && (
                      <TableRow key={`${row.id}-detail`}>
                        <TableCell colSpan={canManage ? 9 : 8} className="bg-muted/30 p-4">
                          <div className="space-y-3">
                            <div className="flex items-center justify-between">
                              <span className="font-medium text-sm">矯正預防行動（CAPA）</span>
                              {canManage && (
                                <Button size="sm" variant="outline" onClick={openAddCapa}>
                                  <Plus className="h-3 w-3 mr-1" />
                                  新增 CAPA
                                </Button>
                              )}
                            </div>
                            {!ncDetail?.capa?.length ? (
                              <p className="text-sm text-muted-foreground">尚無 CAPA</p>
                            ) : (
                              <table className="w-full text-sm">
                                <thead>
                                  <tr className="text-muted-foreground">
                                    <th className="text-left font-normal pb-1">類型</th>
                                    <th className="text-left font-normal pb-1">描述</th>
                                    <th className="text-left font-normal pb-1">狀態</th>
                                    <th className="text-left font-normal pb-1">截止日</th>
                                    {canManage && <th />}
                                  </tr>
                                </thead>
                                <tbody>
                                  {ncDetail.capa.map(capa => (
                                    <tr key={capa.id} className="border-t">
                                      <td className="py-1 pr-4">
                                        {capa.action_type === 'corrective' ? '矯正' : '預防'}
                                      </td>
                                      <td className="py-1 pr-4">{capa.description}</td>
                                      <td className="py-1 pr-4">
                                        <Badge variant="outline" className="text-xs">
                                          {CAPA_STATUS_LABELS[capa.status]}
                                        </Badge>
                                      </td>
                                      <td className="py-1 pr-4">{capa.due_date ?? '—'}</td>
                                      {canManage && (
                                        <td className="py-1">
                                          <Button size="sm" variant="ghost" onClick={() => openEditCapa(capa)}>
                                            編輯
                                          </Button>
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

      {/* NC 對話框 */}
      <Dialog open={ncDialogOpen} onOpenChange={setNcDialogOpen}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>{editNcId ? '編輯不符合事項' : '新增不符合事項'}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-1">
              <Label>標題</Label>
              <Input value={ncForm.title} onChange={e => setNcForm(f => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-1">
              <Label>描述</Label>
              <Textarea rows={3} value={ncForm.description} onChange={e => setNcForm(f => ({ ...f, description: e.target.value }))} />
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label>嚴重度</Label>
                <Select value={ncForm.severity} onValueChange={v => setNcForm(f => ({ ...f, severity: v as NcSeverity }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="critical">重大</SelectItem>
                    <SelectItem value="major">主要</SelectItem>
                    <SelectItem value="minor">輕微</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label>來源</Label>
                <Select value={ncForm.source} onValueChange={v => setNcForm(f => ({ ...f, source: v as NcSource }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="inspection">稽查發現</SelectItem>
                    <SelectItem value="observation">觀察</SelectItem>
                    <SelectItem value="external_audit">外部稽核</SelectItem>
                    <SelectItem value="self_report">自主回報</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="space-y-1">
              <Label>截止日期</Label>
              <Input type="date" value={ncForm.due_date} onChange={e => setNcForm(f => ({ ...f, due_date: e.target.value }))} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setNcDialogOpen(false)}>取消</Button>
            <Button onClick={() => saveMutation.mutate()} disabled={!ncForm.title || saveMutation.isPending}>
              {saveMutation.isPending ? '儲存中…' : '儲存'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* CAPA 對話框 */}
      <Dialog open={capaDialogOpen} onOpenChange={setCapaDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{editCapaId ? '編輯 CAPA' : '新增 CAPA'}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-1">
              <Label>行動類型</Label>
              <Select value={capaForm.action_type} onValueChange={v => setCapaForm(f => ({ ...f, action_type: v as CapaActionType }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="corrective">矯正行動（CA）</SelectItem>
                  <SelectItem value="preventive">預防行動（PA）</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <Label>行動描述</Label>
              <Textarea rows={3} value={capaForm.description} onChange={e => setCapaForm(f => ({ ...f, description: e.target.value }))} />
            </div>
            <div className="space-y-1">
              <Label>截止日期</Label>
              <Input type="date" value={capaForm.due_date} onChange={e => setCapaForm(f => ({ ...f, due_date: e.target.value }))} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCapaDialogOpen(false)}>取消</Button>
            <Button onClick={() => saveCapaMutation.mutate()} disabled={!capaForm.description || saveCapaMutation.isPending}>
              {saveCapaMutation.isPending ? '儲存中…' : '儲存'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
