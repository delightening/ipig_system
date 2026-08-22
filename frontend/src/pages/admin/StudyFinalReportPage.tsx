import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import { useAuthHasPermission } from '@/stores/auth'
import {
  listStudyReports,
  createStudyReport,
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
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Plus, FileText } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { uiLocale } from '@/lib/utils'

const STATUS_OPTIONS = [
  { value: 'draft', label: '草稿' },
  { value: 'under_review', label: '審查中' },
  { value: 'approved', label: '已核准' },
  { value: 'signed', label: '已簽署' },
]

const STATUS_VARIANTS: Record<
  string,
  'default' | 'secondary' | 'destructive' | 'success' | 'warning'
> = {
  draft: 'secondary',
  under_review: 'warning',
  approved: 'success',
  signed: 'success',
}

const INITIAL_FORM = { protocol_id: '', title: '', summary: '', methods: '' }

export function StudyFinalReportPage() {
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
  const canManage = hasPermission('study.report.manage')

  const [filterStatus, setFilterStatus] = useState<string>('')
  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState(INITIAL_FORM)

  const { data: reports = [], isLoading } = useQuery({
    queryKey: ['study-reports', filterStatus],
    queryFn: () => listStudyReports({ status: filterStatus || undefined }),
  })

  const createMutation = useMutation({
    mutationFn: () =>
      createStudyReport({
        protocol_id: form.protocol_id,
        title: form.title,
        summary: form.summary || undefined,
        methods: form.methods || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['study-reports'] })
      setShowCreate(false)
      setForm(INITIAL_FORM)
      toast({ title: '最終報告已建立' })
    },
    onError: (err: unknown) => toast({ title: '建立失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">研究最終報告</h1>
          <p className="text-muted-foreground">GLP 最終報告管理</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            新增報告
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <div className="flex gap-4">
            <Select value={filterStatus} onValueChange={setFilterStatus}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="所有狀態" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">所有狀態</SelectItem>
                {STATUS_OPTIONS.map((s) => (
                  <SelectItem key={s.value} value={s.value}>{s.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>報告編號</TableHead>
                <TableHead>標題</TableHead>
                <TableHead>狀態</TableHead>
                <TableHead>主持人簽署</TableHead>
                <TableHead>QAU 簽署</TableHead>
                <TableHead>建立時間</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={6} className="p-0"><TableSkeleton rows={8} cols={6} /></TableCell></TableRow>
              ) : reports.length === 0 ? (
                <TableEmptyRow colSpan={6} icon={FileText} title="尚無報告" />
              ) : (
                reports.map((r) => (
                  <TableRow key={r.id}>
                    <TableCell className="font-mono text-sm">{r.report_number}</TableCell>
                    <TableCell className="font-medium">{r.title}</TableCell>
                    <TableCell>
                      <Badge variant={STATUS_VARIANTS[r.status] ?? 'secondary'}>
                        {STATUS_OPTIONS.find((s) => s.value === r.status)?.label ?? r.status}
                      </Badge>
                    </TableCell>
                    <TableCell>{r.signed_by ?? '-'}</TableCell>
                    <TableCell>{r.qau_signed_by ?? '-'}</TableCell>
                    <TableCell>{new Date(r.created_at).toLocaleDateString(uiLocale())}</TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增最終報告</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">實驗計畫 ID *</label>
              <Input value={form.protocol_id} onChange={(e) => setForm((f) => ({ ...f, protocol_id: e.target.value }))} placeholder="Protocol UUID" />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">標題 *</label>
              <Input value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">摘要</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
                value={form.summary}
                onChange={(e) => setForm((f) => ({ ...f, summary: e.target.value }))}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">方法</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
                value={form.methods}
                onChange={(e) => setForm((f) => ({ ...f, methods: e.target.value }))}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>取消</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.protocol_id || !form.title || createMutation.isPending}>
              建立
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
