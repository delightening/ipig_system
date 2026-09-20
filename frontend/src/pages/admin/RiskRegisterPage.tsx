import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { useAuthHasPermission } from '@/stores/auth'
import {
  listRisks,
  createRisk,
} from '@/lib/api/glpCompliance'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter,
} from '@/components/ui/dialog'
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { Badge } from '@/components/ui/badge'
import { Plus, AlertTriangle, ShieldAlert } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'

// labelKey 是 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const CATEGORIES = [
  { value: 'technical', labelKey: 'adminGlp.riskRegister.category.technical' },
  { value: 'operational', labelKey: 'adminGlp.riskRegister.category.operational' },
  { value: 'compliance', labelKey: 'adminGlp.riskRegister.category.compliance' },
  { value: 'safety', labelKey: 'adminGlp.riskRegister.category.safety' },
]

const SEVERITY_OPTIONS = [1, 2, 3, 4, 5]

const RISK_STATUS_LABEL_KEYS: Record<string, string> = {
  identified: 'adminGlp.riskRegister.status.identified',
  mitigated: 'adminGlp.riskRegister.status.mitigated',
  accepted: 'adminGlp.riskRegister.status.accepted',
  closed: 'adminGlp.shared.statusLabel.closed',
}

const INITIAL_FORM = {
  title: '',
  category: 'technical',
  severity: '3',
  likelihood: '3',
  description: '',
  mitigation_plan: '',
}

function riskScoreVariant(score: number): 'success' | 'warning' | 'destructive' {
  if (score <= 6) return 'success'
  if (score <= 14) return 'warning'
  return 'destructive'
}

export function RiskRegisterPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
  const canManage = hasPermission('risk.register.manage')

  const [filterCategory, setFilterCategory] = useState<string>('')
  const [filterStatus, setFilterStatus] = useState<string>('')
  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState(INITIAL_FORM)

  const { data: risks = [], isLoading } = useQuery({
    queryKey: ['risks', filterCategory, filterStatus],
    queryFn: () => listRisks({
      category: filterCategory || undefined,
      status: filterStatus || undefined,
    }),
  })

  const createMutation = useMutation({
    mutationFn: () =>
      createRisk({
        title: form.title,
        category: form.category,
        severity: Number(form.severity),
        likelihood: Number(form.likelihood),
        description: form.description || undefined,
        mitigation_plan: form.mitigation_plan || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['risks'] })
      setShowCreate(false)
      setForm(INITIAL_FORM)
      toast({ title: t('adminGlp.riskRegister.toast.created') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.shared.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t('adminGlp.riskRegister.title')}</h1>
          <p className="text-muted-foreground">{t('adminGlp.riskRegister.subtitle')}</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            {t('adminGlp.riskRegister.create')}
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <div className="flex gap-4">
            <Select value={filterCategory} onValueChange={setFilterCategory}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder={t('adminGlp.riskRegister.allCategories')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">{t('adminGlp.riskRegister.allCategories')}</SelectItem>
                {CATEGORIES.map((c) => (
                  <SelectItem key={c.value} value={c.value}>{t(c.labelKey)}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={filterStatus} onValueChange={setFilterStatus}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder={t('adminGlp.shared.allStatuses')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">{t('adminGlp.shared.allStatuses')}</SelectItem>
                <SelectItem value="identified">{t(RISK_STATUS_LABEL_KEYS.identified)}</SelectItem>
                <SelectItem value="mitigated">{t(RISK_STATUS_LABEL_KEYS.mitigated)}</SelectItem>
                <SelectItem value="accepted">{t(RISK_STATUS_LABEL_KEYS.accepted)}</SelectItem>
                <SelectItem value="closed">{t(RISK_STATUS_LABEL_KEYS.closed)}</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                <TableHead>{t('adminGlp.riskRegister.col.riskNumber')}</TableHead>
                <TableHead>{t('adminGlp.shared.title')}</TableHead>
                <TableHead>{t('adminGlp.shared.category')}</TableHead>
                <TableHead>{t('adminGlp.shared.severity')}</TableHead>
                <TableHead>{t('adminGlp.riskRegister.col.likelihood')}</TableHead>
                <TableHead>{t('adminGlp.riskRegister.col.riskScore')}</TableHead>
                <TableHead>{t('adminGlp.shared.status')}</TableHead>
                <TableHead>{t('adminGlp.shared.owner')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={8} className="p-0"><TableSkeleton rows={5} cols={8} /></TableCell></TableRow>
              ) : risks.length === 0 ? (
                <TableEmptyRow colSpan={8} icon={ShieldAlert} title={t('adminGlp.riskRegister.empty')} />
              ) : (
                risks.map((r) => {
                  const score = r.risk_score ?? r.severity * r.likelihood
                  const category = CATEGORIES.find((c) => c.value === r.category)
                  return (
                    <TableRow key={r.id}>
                      <TableCell className="font-mono text-sm">{r.risk_number}</TableCell>
                      <TableCell className="font-medium">{r.title}</TableCell>
                      <TableCell>{category ? t(category.labelKey) : r.category ?? '-'}</TableCell>
                      <TableCell>{r.severity}</TableCell>
                      <TableCell>{r.likelihood}</TableCell>
                      <TableCell>
                        <Badge variant={riskScoreVariant(score)}>
                          {score >= 15 && <AlertTriangle className="mr-1 h-3 w-3 inline" />}
                          {score}
                        </Badge>
                      </TableCell>
                      <TableCell>{RISK_STATUS_LABEL_KEYS[r.status] ? t(RISK_STATUS_LABEL_KEYS[r.status]) : r.status}</TableCell>
                      <TableCell>{r.owner_name ?? '-'}</TableCell>
                    </TableRow>
                  )
                })
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('adminGlp.riskRegister.create')}</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.shared.titleRequired')}</label>
              <Input value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.riskRegister.dialog.categoryRequired')}</label>
              <Select value={form.category} onValueChange={(v) => setForm((f) => ({ ...f, category: v }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {CATEGORIES.map((c) => (
                    <SelectItem key={c.value} value={c.value}>{t(c.labelKey)}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">{t('adminGlp.riskRegister.dialog.severityRange')}</label>
                <Select value={form.severity} onValueChange={(v) => setForm((f) => ({ ...f, severity: v }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {SEVERITY_OPTIONS.map((n) => (
                      <SelectItem key={n} value={String(n)}>{n}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <label className="text-sm font-medium">{t('adminGlp.riskRegister.dialog.likelihoodRange')}</label>
                <Select value={form.likelihood} onValueChange={(v) => setForm((f) => ({ ...f, likelihood: v }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {SEVERITY_OPTIONS.map((n) => (
                      <SelectItem key={n} value={String(n)}>{n}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.shared.description')}</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
                value={form.description}
                onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.riskRegister.dialog.mitigationPlan')}</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
                value={form.mitigation_plan}
                onChange={(e) => setForm((f) => ({ ...f, mitigation_plan: e.target.value }))}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>{t('common.cancel')}</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.title || createMutation.isPending}>
              {t('adminGlp.shared.create')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
