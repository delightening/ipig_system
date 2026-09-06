import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import { useAuthHasPermission } from '@/stores/auth'
import {
  listStudyReports,
  createStudyReport,
  getStudyReport,
  updateStudyReport,
  signStudyReport,
  updateQauStatement,
  type StudyFinalReport,
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

const textareaClass =
  'flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring'

export function StudyFinalReportPage() {
  const queryClient = useQueryClient()
  const hasPermission = useAuthHasPermission()
  // 2026-09-05：study.report.manage 已改走身分即授權（P0-1）——按鈕永遠顯示，
  // 是否真的能建立／編輯／簽署由後端依「本人是不是該計畫 SD」判定，未授權時 toast 顯示 403 訊息。
  const canWriteQauStatement = hasPermission('qau.report_statement.write')

  const [filterStatus, setFilterStatus] = useState<string>('')
  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState(INITIAL_FORM)
  const [detailId, setDetailId] = useState<string | null>(null)

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
        <Button onClick={() => setShowCreate(true)}>
          <Plus className="mr-2 h-4 w-4" />
          新增報告
        </Button>
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
                  <TableRow
                    key={r.id}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setDetailId(r.id)}
                  >
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
              <p className="text-xs text-muted-foreground">只有該計畫的計劃負責人（Study Director）本人可以建立報告。</p>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">標題 *</label>
              <Input value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">摘要</label>
              <textarea
                className={textareaClass}
                value={form.summary}
                onChange={(e) => setForm((f) => ({ ...f, summary: e.target.value }))}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">方法</label>
              <textarea
                className={textareaClass}
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

      {detailId && (
        <StudyReportDetailDialog
          id={detailId}
          canWriteQauStatement={canWriteQauStatement}
          onClose={() => setDetailId(null)}
        />
      )}
    </div>
  )
}

function StudyReportDetailDialog({
  id,
  canWriteQauStatement,
  onClose,
}: {
  id: string
  canWriteQauStatement: boolean
  onClose: () => void
}) {
  const queryClient = useQueryClient()
  const [edit, setEdit] = useState<{
    title: string
    summary: string
    methods: string
    results: string
    conclusions: string
    deviations: string
  } | null>(null)
  const [qauStatement, setQauStatement] = useState('')
  const [signPassword, setSignPassword] = useState('')

  const { data: report, isLoading } = useQuery({
    queryKey: ['study-report', id],
    queryFn: () => getStudyReport(id),
  })

  const startEdit = (r: StudyFinalReport) =>
    setEdit({
      title: r.title,
      summary: r.summary ?? '',
      methods: r.methods ?? '',
      results: r.results ?? '',
      conclusions: r.conclusions ?? '',
      deviations: r.deviations ?? '',
    })

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ['study-report', id] })
    queryClient.invalidateQueries({ queryKey: ['study-reports'] })
  }

  const saveMutation = useMutation({
    mutationFn: () => {
      if (!edit) throw new Error('no edit state')
      return updateStudyReport(id, edit)
    },
    onSuccess: () => {
      invalidate()
      setEdit(null)
      toast({ title: '報告已更新' })
    },
    onError: (err: unknown) => toast({ title: '更新失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const signMutation = useMutation({
    mutationFn: () => signStudyReport(id, { password: signPassword }),
    onSuccess: () => {
      invalidate()
      setSignPassword('')
      toast({ title: '已簽署最終報告' })
    },
    onError: (err: unknown) => toast({ title: '簽署失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const qauMutation = useMutation({
    mutationFn: () => updateQauStatement(id, qauStatement),
    onSuccess: () => {
      invalidate()
      toast({ title: 'QAU 品保聲明已儲存' })
    },
    onError: (err: unknown) => toast({ title: '儲存失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      {/* size="lg" 即 max-w-2xl（見 components/ui/dialogSize.ts）；
          ⚠️ 不是 size="2xl"——那是 max-w-6xl。寬度一律走 size prop，禁止硬編 max-w-*（DESIGN.md）。 */}
      <DialogContent size="lg">
        <DialogHeader>
          <DialogTitle>{report ? `${report.report_number}｜${report.title}` : '最終報告'}</DialogTitle>
        </DialogHeader>

        {isLoading || !report ? (
          <div className="py-8 text-center text-muted-foreground">載入中…</div>
        ) : (
          <div className="space-y-6">
            <div className="flex items-center gap-2">
              <Badge variant={STATUS_VARIANTS[report.status] ?? 'secondary'}>
                {STATUS_OPTIONS.find((s) => s.value === report.status)?.label ?? report.status}
              </Badge>
              {report.signed_at && (
                <span className="text-xs text-muted-foreground">
                  主持人已於 {new Date(report.signed_at).toLocaleString(uiLocale())} 簽署
                </span>
              )}
            </div>

            {/* 報告本文：身分即授權，只有本計畫 SD 能編輯／簽署，未授權時後端會回 403 */}
            {edit ? (
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium">標題</label>
                  <Input value={edit.title} onChange={(e) => setEdit({ ...edit, title: e.target.value })} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">摘要</label>
                  <textarea className={textareaClass} value={edit.summary} onChange={(e) => setEdit({ ...edit, summary: e.target.value })} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">方法</label>
                  <textarea className={textareaClass} value={edit.methods} onChange={(e) => setEdit({ ...edit, methods: e.target.value })} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">結果</label>
                  <textarea className={textareaClass} value={edit.results} onChange={(e) => setEdit({ ...edit, results: e.target.value })} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">結論</label>
                  <textarea className={textareaClass} value={edit.conclusions} onChange={(e) => setEdit({ ...edit, conclusions: e.target.value })} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">偏離事項</label>
                  <textarea className={textareaClass} value={edit.deviations} onChange={(e) => setEdit({ ...edit, deviations: e.target.value })} />
                </div>
                <div className="flex justify-end gap-2">
                  <Button variant="outline" onClick={() => setEdit(null)}>取消</Button>
                  <Button onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}>儲存</Button>
                </div>
              </div>
            ) : (
              <div className="space-y-3">
                <div className="grid grid-cols-1 gap-2 text-sm">
                  <div><span className="font-medium">摘要：</span>{report.summary || '—'}</div>
                  <div><span className="font-medium">方法：</span>{report.methods || '—'}</div>
                  <div><span className="font-medium">結果：</span>{report.results || '—'}</div>
                  <div><span className="font-medium">結論：</span>{report.conclusions || '—'}</div>
                  <div><span className="font-medium">偏離事項：</span>{report.deviations || '—'}</div>
                </div>
                {report.status !== 'signed' && (
                  <Button variant="outline" size="sm" onClick={() => startEdit(report)}>編輯報告本文</Button>
                )}
              </div>
            )}

            {/* SD 簽署：無 admin 例外，僅本計畫 SD 本人可簽 */}
            {report.status !== 'signed' && (
              <div className="space-y-2 border-t pt-4">
                <label className="text-sm font-medium">簽署最終報告（僅本計畫計劃負責人 Study Director 可簽）</label>
                <div className="flex gap-2">
                  <Input
                    type="password"
                    placeholder="密碼確認身分"
                    value={signPassword}
                    onChange={(e) => setSignPassword(e.target.value)}
                  />
                  <Button
                    onClick={() => signMutation.mutate()}
                    disabled={!signPassword || signMutation.isPending}
                  >
                    簽署
                  </Button>
                </div>
              </div>
            )}

            {/* QAU 品保聲明：與報告本文分開授權，且不得為本計畫 SD 本人（服務層強制） */}
            <div className="space-y-2 border-t pt-4">
              <label className="text-sm font-medium">QAU 品保聲明</label>
              {report.qau_signed_at && (
                <p className="text-xs text-muted-foreground">
                  已於 {new Date(report.qau_signed_at).toLocaleString(uiLocale())} 出具
                </p>
              )}
              {canWriteQauStatement ? (
                <div className="space-y-2">
                  <textarea
                    className={textareaClass}
                    defaultValue={report.qau_statement ?? ''}
                    onChange={(e) => setQauStatement(e.target.value)}
                    placeholder="品保稽核結論…"
                  />
                  <div className="flex justify-end">
                    <Button
                      size="sm"
                      onClick={() => qauMutation.mutate()}
                      disabled={!qauStatement || qauMutation.isPending}
                    >
                      儲存品保聲明
                    </Button>
                  </div>
                </div>
              ) : (
                <p className="text-sm text-muted-foreground">{report.qau_statement || '尚無品保聲明'}</p>
              )}
            </div>
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>關閉</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
