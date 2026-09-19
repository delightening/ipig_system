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
import { useTranslation } from 'react-i18next'

import api from '@/lib/api'

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
  listNonConformances, getNonConformance, createNonConformance,
  updateNonConformance, createCapa, updateCapa,
  type QaNonConformanceWithDetails, type NcSeverity, type NcSource,
  type NcStatus, type CapaActionType, type CapaStatus, type QaCapa,
} from '@/lib/api/qaPlan'

// 值為 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const SEVERITY_LABEL_KEYS: Record<NcSeverity, string> = {
  critical: 'adminGlp.qaNonConformance.severity.critical',
  major: 'adminGlp.qaNonConformance.severity.major',
  minor: 'adminGlp.qaNonConformance.severity.minor',
}

const SEVERITY_VARIANTS: Record<NcSeverity, 'default' | 'secondary' | 'destructive' | 'outline'> = {
  critical: 'destructive', major: 'default', minor: 'secondary',
}

const NC_STATUS_LABEL_KEYS: Record<NcStatus, string> = {
  open: 'adminGlp.qaNonConformance.status.open',
  in_progress: 'adminGlp.qaNonConformance.status.inProgress',
  pending_verification: 'adminGlp.shared.statusLabel.pendingVerification',
  closed: 'adminGlp.shared.statusLabel.closed',
}

const SOURCE_LABEL_KEYS: Record<NcSource, string> = {
  inspection: 'adminGlp.qaNonConformance.source.inspection',
  observation: 'adminGlp.qaNonConformance.source.observation',
  external_audit: 'adminGlp.qaNonConformance.source.externalAudit',
  self_report: 'adminGlp.qaNonConformance.source.selfReport',
}

const CAPA_STATUS_LABEL_KEYS: Record<CapaStatus, string> = {
  open: 'adminGlp.qaNonConformance.status.open',
  in_progress: 'adminGlp.shared.statusLabel.inProgress',
  completed: 'adminGlp.shared.statusLabel.completed',
  verified: 'adminGlp.shared.statusLabel.verified',
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
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
  const canManage = hasPermission('qau.nc.manage')

  const [filterSeverity, setFilterSeverity] = useState('all')
  const [filterStatus, setFilterStatus] = useState('all')
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

  const { data: users = [] } = useQuery({
    queryKey: ['internal-users'],
    queryFn: async () => {
      const res = await api.get<{ id: string; display_name: string; email: string }[]>('/hr/internal-users')
      return res.data
    },
  })

  const { data: ncList = [], isLoading } = useQuery({
    queryKey: ['qa-nc', filterSeverity, filterStatus],
    queryFn: () => listNonConformances({
      severity: filterSeverity !== 'all' ? filterSeverity : undefined,
      status: filterStatus !== 'all' ? filterStatus : undefined,
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
        title={t('adminGlp.qaNonConformance.title')}
        description={t('adminGlp.qaNonConformance.description')}
        actions={canManage ? (
          <Button size="sm" onClick={openCreateNc}>
            <Plus className="h-4 w-4 mr-2" />
            {t('adminGlp.qaNonConformance.create')}
          </Button>
        ) : undefined}
      />

      <div className="flex gap-3">
        <Select value={filterSeverity} onValueChange={setFilterSeverity}>
          <SelectTrigger className="w-32">
            <SelectValue placeholder={t('adminGlp.shared.severity')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t('adminGlp.qaNonConformance.all')}</SelectItem>
            <SelectItem value="critical">{t(SEVERITY_LABEL_KEYS.critical)}</SelectItem>
            <SelectItem value="major">{t(SEVERITY_LABEL_KEYS.major)}</SelectItem>
            <SelectItem value="minor">{t(SEVERITY_LABEL_KEYS.minor)}</SelectItem>
          </SelectContent>
        </Select>
        <Select value={filterStatus} onValueChange={setFilterStatus}>
          <SelectTrigger className="w-36">
            <SelectValue placeholder={t('adminGlp.shared.status')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t('common.allStatus')}</SelectItem>
            {Object.entries(NC_STATUS_LABEL_KEYS).map(([v, labelKey]) => (
              <SelectItem key={v} value={v}>{t(labelKey)}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5" />
            {t('adminGlp.qaNonConformance.listTitle')}
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
                  <TableHead>{t('adminGlp.qaNonConformance.col.ncNumber')}</TableHead>
                  <TableHead>{t('adminGlp.shared.title')}</TableHead>
                  <TableHead>{t('adminGlp.shared.severity')}</TableHead>
                  <TableHead>{t('adminGlp.qaNonConformance.col.source')}</TableHead>
                  <TableHead>{t('adminGlp.shared.owner')}</TableHead>
                  <TableHead>{t('adminGlp.qaNonConformance.col.dueDate')}</TableHead>
                  <TableHead>{t('adminGlp.shared.status')}</TableHead>
                  {canManage && <TableHead className="w-20">{t('common.actions')}</TableHead>}
                </TableRow>
              </TableHeader>
              <TableBody>
                {ncList.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={9} className="text-center text-muted-foreground py-8">
                      {t('adminGlp.qaNonConformance.empty')}
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
                          {t(SEVERITY_LABEL_KEYS[row.severity])}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-sm text-muted-foreground">
                        {t(SOURCE_LABEL_KEYS[row.source])}
                      </TableCell>
                      <TableCell>{row.assignee_name ?? '—'}</TableCell>
                      <TableCell>{row.due_date ?? '—'}</TableCell>
                      <TableCell>
                        <Badge variant="outline">{t(NC_STATUS_LABEL_KEYS[row.status])}</Badge>
                      </TableCell>
                      {canManage && (
                        <TableCell onClick={e => e.stopPropagation()}>
                          <Button size="sm" variant="ghost" onClick={() => openEditNc(row)}>{t('common.edit')}</Button>
                        </TableCell>
                      )}
                    </TableRow>

                    {/* CAPA 展開列 */}
                    {expandedId === row.id && (
                      <TableRow key={`${row.id}-detail`}>
                        <TableCell colSpan={canManage ? 9 : 8} className="bg-muted/30 p-4">
                          <div className="space-y-3">
                            <div className="flex items-center justify-between">
                              <span className="font-medium text-sm">{t('adminGlp.qaNonConformance.capa.title')}</span>
                              {canManage && (
                                <Button size="sm" variant="outline" onClick={openAddCapa}>
                                  <Plus className="h-3 w-3 mr-1" />
                                  {t('adminGlp.qaNonConformance.capa.create')}
                                </Button>
                              )}
                            </div>
                            {!ncDetail?.capa?.length ? (
                              <p className="text-sm text-muted-foreground">{t('adminGlp.qaNonConformance.capa.empty')}</p>
                            ) : (
                              <Table className="w-full text-sm">
                                <TableHeader>
                                  <TableRow className="text-muted-foreground">
                                    <TableHead className="text-left font-normal pb-1">{t('adminGlp.shared.type')}</TableHead>
                                    <TableHead className="text-left font-normal pb-1">{t('adminGlp.shared.description')}</TableHead>
                                    <TableHead className="text-left font-normal pb-1">{t('adminGlp.shared.status')}</TableHead>
                                    <TableHead className="text-left font-normal pb-1">{t('adminGlp.qaNonConformance.col.dueDate')}</TableHead>
                                    {canManage && <TableHead />}
                                  </TableRow>
                                </TableHeader>
                                <TableBody>
                                  {ncDetail.capa.map(capa => (
                                    <TableRow key={capa.id} className="border-t">
                                      <TableCell className="py-1 pr-4">
                                        {capa.action_type === 'corrective'
                                          ? t('adminGlp.qaNonConformance.capa.corrective')
                                          : t('adminGlp.qaNonConformance.capa.preventive')}
                                      </TableCell>
                                      <TableCell className="py-1 pr-4">{capa.description}</TableCell>
                                      <TableCell className="py-1 pr-4">
                                        <Badge variant="outline" className="text-xs">
                                          {t(CAPA_STATUS_LABEL_KEYS[capa.status])}
                                        </Badge>
                                      </TableCell>
                                      <TableCell className="py-1 pr-4">{capa.due_date ?? '—'}</TableCell>
                                      {canManage && (
                                        <TableCell className="py-1">
                                          <Button size="sm" variant="ghost" onClick={() => openEditCapa(capa)}>
                                            {t('common.edit')}
                                          </Button>
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

      {/* NC 對話框 */}
      <Dialog open={ncDialogOpen} onOpenChange={setNcDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editNcId ? t('adminGlp.qaNonConformance.dialog.edit') : t('adminGlp.qaNonConformance.dialog.create')}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-1">
              <Label>{t('adminGlp.shared.title')}</Label>
              <Input value={ncForm.title} onChange={e => setNcForm(f => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-1">
              <Label>{t('adminGlp.shared.description')}</Label>
              <Textarea rows={3} value={ncForm.description} onChange={e => setNcForm(f => ({ ...f, description: e.target.value }))} />
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label>{t('adminGlp.shared.severity')}</Label>
                <Select value={ncForm.severity} onValueChange={v => setNcForm(f => ({ ...f, severity: v as NcSeverity }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="critical">{t(SEVERITY_LABEL_KEYS.critical)}</SelectItem>
                    <SelectItem value="major">{t(SEVERITY_LABEL_KEYS.major)}</SelectItem>
                    <SelectItem value="minor">{t(SEVERITY_LABEL_KEYS.minor)}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label>{t('adminGlp.qaNonConformance.col.source')}</Label>
                <Select value={ncForm.source} onValueChange={v => setNcForm(f => ({ ...f, source: v as NcSource }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="inspection">{t(SOURCE_LABEL_KEYS.inspection)}</SelectItem>
                    <SelectItem value="observation">{t(SOURCE_LABEL_KEYS.observation)}</SelectItem>
                    <SelectItem value="external_audit">{t(SOURCE_LABEL_KEYS.external_audit)}</SelectItem>
                    <SelectItem value="self_report">{t(SOURCE_LABEL_KEYS.self_report)}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label>{t('adminGlp.qaNonConformance.dialog.assignee')}</Label>
                <Select
                  value={ncForm.assignee_id || '__none__'}
                  onValueChange={v => setNcForm(f => ({ ...f, assignee_id: v === '__none__' ? '' : v }))}
                >
                  <SelectTrigger><SelectValue placeholder={t('adminGlp.qaNonConformance.dialog.selectAssignee')} /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__none__">{t('adminGlp.qaNonConformance.dialog.unassigned')}</SelectItem>
                    {users.map(u => (
                      <SelectItem key={u.id} value={u.id}>{u.display_name}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label>{t('adminGlp.qaNonConformance.dialog.dueDate')}</Label>
                <Input type="date" value={ncForm.due_date} onChange={e => setNcForm(f => ({ ...f, due_date: e.target.value }))} />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setNcDialogOpen(false)}>{t('common.cancel')}</Button>
            <Button onClick={() => saveMutation.mutate()} disabled={!ncForm.title || saveMutation.isPending}>
              {saveMutation.isPending ? t('adminGlp.shared.saving') : t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* CAPA 對話框 */}
      <Dialog open={capaDialogOpen} onOpenChange={setCapaDialogOpen}>
        <DialogContent size="sm">
          <DialogHeader>
            <DialogTitle>{editCapaId ? t('adminGlp.qaNonConformance.capa.edit') : t('adminGlp.qaNonConformance.capa.create')}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-1">
              <Label>{t('adminGlp.qaNonConformance.capa.actionType')}</Label>
              <Select value={capaForm.action_type} onValueChange={v => setCapaForm(f => ({ ...f, action_type: v as CapaActionType }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="corrective">{t('adminGlp.qaNonConformance.capa.correctiveAction')}</SelectItem>
                  <SelectItem value="preventive">{t('adminGlp.qaNonConformance.capa.preventiveAction')}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <Label>{t('adminGlp.qaNonConformance.capa.actionDescription')}</Label>
              <Textarea rows={3} value={capaForm.description} onChange={e => setCapaForm(f => ({ ...f, description: e.target.value }))} />
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label>{t('adminGlp.qaNonConformance.dialog.assignee')}</Label>
                <Select
                  value={capaForm.assignee_id || '__none__'}
                  onValueChange={v => setCapaForm(f => ({ ...f, assignee_id: v === '__none__' ? '' : v }))}
                >
                  <SelectTrigger><SelectValue placeholder={t('adminGlp.qaNonConformance.dialog.selectAssignee')} /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__none__">{t('adminGlp.qaNonConformance.dialog.unassigned')}</SelectItem>
                    {users.map(u => (
                      <SelectItem key={u.id} value={u.id}>{u.display_name}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label>{t('adminGlp.qaNonConformance.dialog.dueDate')}</Label>
                <Input type="date" value={capaForm.due_date} onChange={e => setCapaForm(f => ({ ...f, due_date: e.target.value }))} />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCapaDialogOpen(false)}>{t('common.cancel')}</Button>
            <Button onClick={() => saveCapaMutation.mutate()} disabled={!capaForm.description || saveCapaMutation.isPending}>
              {saveCapaMutation.isPending ? t('adminGlp.shared.saving') : t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
