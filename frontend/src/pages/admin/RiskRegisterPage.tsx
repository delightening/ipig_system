import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import { useAuthStore } from '@/stores/auth'
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
import { getApiErrorMessage } from '@/lib/validation'

const CATEGORIES = [
  { value: 'technical', label: '技術' },
  { value: 'operational', label: '營運' },
  { value: 'compliance', label: '法規遵循' },
  { value: 'safety', label: '安全' },
]

const SEVERITY_OPTIONS = [1, 2, 3, 4, 5]

const INITIAL_FORM = {
  title: '',
  category: 'technical',
  severity: '3',
  likelihood: '3',
  description: '',
  mitigation_plan: '',
}

function riskScoreColor(score: number): string {
  if (score <= 6) return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
  if (score <= 14) return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
  return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
}

export function RiskRegisterPage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
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
      toast({ title: '風險已登記' })
    },
    onError: (err: unknown) => toast({ title: '建立失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">風險登記簿</h1>
          <p className="text-muted-foreground">ISO 17025 / ISO 9001 風險管理</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            新增風險
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <div className="flex gap-4">
            <Select value={filterCategory} onValueChange={setFilterCategory}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="所有類別" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">所有類別</SelectItem>
                {CATEGORIES.map((c) => (
                  <SelectItem key={c.value} value={c.value}>{c.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={filterStatus} onValueChange={setFilterStatus}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="所有狀態" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">所有狀態</SelectItem>
                <SelectItem value="identified">已辨識</SelectItem>
                <SelectItem value="mitigated">已緩解</SelectItem>
                <SelectItem value="accepted">已接受</SelectItem>
                <SelectItem value="closed">已結案</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                <TableHead>風險編號</TableHead>
                <TableHead>標題</TableHead>
                <TableHead>類別</TableHead>
                <TableHead>嚴重度</TableHead>
                <TableHead>可能性</TableHead>
                <TableHead>風險分數</TableHead>
                <TableHead>狀態</TableHead>
                <TableHead>負責人</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={8} className="p-0"><TableSkeleton rows={5} cols={8} /></TableCell></TableRow>
              ) : risks.length === 0 ? (
                <TableEmptyRow colSpan={8} icon={ShieldAlert} title="尚無風險紀錄" />
              ) : (
                risks.map((r) => {
                  const score = r.risk_score ?? r.severity * r.likelihood
                  return (
                    <TableRow key={r.id}>
                      <TableCell className="font-mono text-sm">{r.risk_number}</TableCell>
                      <TableCell className="font-medium">{r.title}</TableCell>
                      <TableCell>{CATEGORIES.find((c) => c.value === r.category)?.label ?? r.category ?? '-'}</TableCell>
                      <TableCell>{r.severity}</TableCell>
                      <TableCell>{r.likelihood}</TableCell>
                      <TableCell>
                        <Badge className={riskScoreColor(score)}>
                          {score >= 15 && <AlertTriangle className="mr-1 h-3 w-3 inline" />}
                          {score}
                        </Badge>
                      </TableCell>
                      <TableCell>{r.status}</TableCell>
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
          <DialogHeader><DialogTitle>新增風險</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">標題 *</label>
              <Input value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">類別 *</label>
              <Select value={form.category} onValueChange={(v) => setForm((f) => ({ ...f, category: v }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {CATEGORIES.map((c) => (
                    <SelectItem key={c.value} value={c.value}>{c.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">嚴重度 (1-5) *</label>
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
                <label className="text-sm font-medium">可能性 (1-5) *</label>
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
              <label className="text-sm font-medium">描述</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
                value={form.description}
                onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">緩解計畫</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
                value={form.mitigation_plan}
                onChange={(e) => setForm((f) => ({ ...f, mitigation_plan: e.target.value }))}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>取消</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.title || createMutation.isPending}>
              建立
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
