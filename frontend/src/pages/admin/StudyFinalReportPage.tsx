import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'

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

// labelKey 是 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const STATUS_OPTIONS = [
  { value: 'draft', labelKey: 'adminGlp.shared.statusLabel.draft' },
  { value: 'under_review', labelKey: 'adminGlp.shared.statusLabel.underReview' },
  { value: 'approved', labelKey: 'adminGlp.shared.statusLabel.approved' },
  { value: 'signed', labelKey: 'adminGlp.studyFinalReport.status.signed' },
]

function getStatusLabel(status: string, t: TFunction): string {
  const option = STATUS_OPTIONS.find((s) => s.value === status)
  return option ? t(option.labelKey) : status
}

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
  const { t } = useTranslation()
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
      toast({ title: t('adminGlp.studyFinalReport.toast.created') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.shared.createFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t('adminGlp.studyFinalReport.title')}</h1>
          <p className="text-muted-foreground">{t('adminGlp.studyFinalReport.subtitle')}</p>
        </div>
        <Button onClick={() => setShowCreate(true)}>
          <Plus className="mr-2 h-4 w-4" />
          {t('adminGlp.studyFinalReport.create')}
        </Button>
      </div>

      <Card>
        <CardHeader>
          <div className="flex gap-4">
            <Select value={filterStatus} onValueChange={setFilterStatus}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder={t('adminGlp.shared.allStatuses')} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">{t('adminGlp.shared.allStatuses')}</SelectItem>
                {STATUS_OPTIONS.map((s) => (
                  <SelectItem key={s.value} value={s.value}>{t(s.labelKey)}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('adminGlp.shared.reportNumber')}</TableHead>
                <TableHead>{t('adminGlp.shared.title')}</TableHead>
                <TableHead>{t('adminGlp.shared.status')}</TableHead>
                <TableHead>{t('adminGlp.studyFinalReport.col.directorSignature')}</TableHead>
                <TableHead>{t('adminGlp.studyFinalReport.col.qauSignature')}</TableHead>
                <TableHead>{t('adminGlp.shared.createdAt')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={6} className="p-0"><TableSkeleton rows={8} cols={6} /></TableCell></TableRow>
              ) : reports.length === 0 ? (
                <TableEmptyRow colSpan={6} icon={FileText} title={t('adminGlp.studyFinalReport.empty')} />
              ) : (
                reports.map((r) => (
                  <TableRow
                    key={r.id}
                    // 整列是開啟詳情的唯一入口，所以它必須能用鍵盤操作：
                    // 只掛 onClick 的 <tr> 不在 tab 順序內，鍵盤與讀屏使用者完全打不開詳情。
                    // a11y 是 CLAUDE.md §精簡的底線 明列不可省的項目。
                    role="button"
                    tabIndex={0}
                    className="cursor-pointer hover:bg-muted/50"
                    onClick={() => setDetailId(r.id)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault() // 空白鍵預設會捲動頁面
                        setDetailId(r.id)
                      }
                    }}
                  >
                    <TableCell className="font-mono text-sm">{r.report_number}</TableCell>
                    <TableCell className="font-medium">{r.title}</TableCell>
                    <TableCell>
                      <Badge variant={STATUS_VARIANTS[r.status] ?? 'secondary'}>
                        {getStatusLabel(r.status, t)}
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
          <DialogHeader><DialogTitle>{t('adminGlp.studyFinalReport.dialog.create')}</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.studyFinalReport.dialog.protocolId')}</label>
              <Input value={form.protocol_id} onChange={(e) => setForm((f) => ({ ...f, protocol_id: e.target.value }))} placeholder="Protocol UUID" />
              <p className="text-xs text-muted-foreground">{t('adminGlp.studyFinalReport.dialog.protocolIdHint')}</p>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.shared.titleRequired')}</label>
              <Input value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.studyFinalReport.summary')}</label>
              <textarea
                className={textareaClass}
                value={form.summary}
                onChange={(e) => setForm((f) => ({ ...f, summary: e.target.value }))}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">{t('adminGlp.studyFinalReport.methods')}</label>
              <textarea
                className={textareaClass}
                value={form.methods}
                onChange={(e) => setForm((f) => ({ ...f, methods: e.target.value }))}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>{t('common.cancel')}</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.protocol_id || !form.title || createMutation.isPending}>
              {t('adminGlp.shared.create')}
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

/**
 * 最終報告詳情對話框：本文編輯、SD 簽署、QAU 品保聲明三塊在同一個畫面，
 * 但**授權來源三者各異**（P0-1），所以顯示條件不共用：
 *
 * - 本文編輯與簽署：身分即授權（該計畫的 SD）。SD 是「該計畫的」身分，前端無從得知，
 *   故**刻意不做前端權限判斷**，一律顯示、由後端回 403。前端若擅自擋，SD 本人也會點不到。
 * - QAU 品保聲明：有對應權限碼，所以靠 `canWriteQauStatement` 決定可編輯或唯讀。
 * - 已簽署（`status === 'signed'`）：本文編輯與簽署入口一律不顯示，後端另有守衛擋。
 */
function StudyReportDetailDialog({
  id,
  canWriteQauStatement,
  onClose,
}: {
  id: string
  canWriteQauStatement: boolean
  onClose: () => void
}) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [edit, setEdit] = useState<{
    title: string
    summary: string
    methods: string
    results: string
    conclusions: string
    deviations: string
  } | null>(null)
  // null = 使用者還沒動過這個欄位，畫面與送出都回落到報告現值；
  // 空字串是「使用者刻意清空」，與 null 不同義，所以不能用 '' 當初始值。
  const [qauStatement, setQauStatement] = useState<string | null>(null)
  const [signPassword, setSignPassword] = useState('')

  const { data: report, isLoading, isError, error } = useQuery({
    queryKey: ['study-report', id],
    queryFn: () => getStudyReport(id),
  })

  /**
   * 進入編輯模式：把報告現值複製進表單狀態。
   * `?? ''` 是刻意的——後端這些欄位可為 `null`，而受控 textarea 吃到 `null`
   * 會被 React 當成非受控而噴警告，且游標行為會亂掉。
   */
  const startEdit = (r: StudyFinalReport) =>
    setEdit({
      title: r.title,
      summary: r.summary ?? '',
      methods: r.methods ?? '',
      results: r.results ?? '',
      conclusions: r.conclusions ?? '',
      deviations: r.deviations ?? '',
    })

  /**
   * 三個 mutation 成功後都呼叫這支。**兩個 key 都要失效**：
   * 詳情（`['study-report', id]`）與列表（`['study-reports']`）——
   * 簽署會同時改變詳情內容與列表上的狀態徽章／簽署人欄，
   * 只失效其一會讓另一邊停在舊值。
   */
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
      toast({ title: t('adminGlp.studyFinalReport.toast.updated') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.studyFinalReport.toast.updateFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const signMutation = useMutation({
    mutationFn: () => signStudyReport(id, { password: signPassword }),
    onSuccess: () => {
      invalidate()
      setSignPassword('')
      toast({ title: t('adminGlp.studyFinalReport.toast.signed') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.studyFinalReport.toast.signFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const qauMutation = useMutation({
    // 與 textarea 的 value、與按鈕的 disabled 用同一個 resolved 值：
    // 三處若各自解讀 null，就會出現「畫面顯示 A、送出 B」的分歧。
    mutationFn: () => updateQauStatement(id, qauStatement ?? report?.qau_statement ?? ''),
    onSuccess: () => {
      invalidate()
      toast({ title: t('adminGlp.studyFinalReport.toast.qauSaved') })
    },
    onError: (err: unknown) => toast({ title: t('adminGlp.studyFinalReport.toast.saveFailed'), description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      {/* size="lg" 即 max-w-2xl（見 components/ui/dialogSize.ts）；
          ⚠️ 不是 size="2xl"——那是 max-w-6xl。寬度一律走 size prop，禁止硬編 max-w-*（DESIGN.md）。 */}
      <DialogContent size="lg">
        <DialogHeader>
          <DialogTitle>{report ? `${report.report_number}｜${report.title}` : t('adminGlp.studyFinalReport.detail.fallbackTitle')}</DialogTitle>
        </DialogHeader>

        {/* ⚠️ 失敗必須有自己的分支。原本只有 `isLoading || !report`：查詢 reject 時
            isLoading 會變 false 而 report 仍是 undefined，於是落進同一格，
            對話框**永遠停在「載入中…」**——使用者看到的是無限載入而不是錯誤。 */}
        {isError ? (
          <div className="py-8 text-center text-destructive">
            {getApiErrorMessage(error, t('adminGlp.studyFinalReport.detail.loadFailed'))}
          </div>
        ) : isLoading || !report ? (
          <div className="py-8 text-center text-muted-foreground">{t('adminGlp.shared.loading')}</div>
        ) : (
          <div className="space-y-6">
            <div className="flex items-center gap-2">
              <Badge variant={STATUS_VARIANTS[report.status] ?? 'secondary'}>
                {getStatusLabel(report.status, t)}
              </Badge>
              {report.signed_at && (
                <span className="text-xs text-muted-foreground">
                  {t('adminGlp.studyFinalReport.detail.signedAt', { time: new Date(report.signed_at).toLocaleString(uiLocale()) })}
                </span>
              )}
            </div>

            {/* 報告本文：身分即授權，只有本計畫 SD 能編輯／簽署，未授權時後端會回 403 */}
            {edit ? (
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium">{t('adminGlp.shared.title')}</label>
                  <Input value={edit.title} onChange={(e) => setEdit({ ...edit, title: e.target.value })} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">{t('adminGlp.studyFinalReport.summary')}</label>
                  <textarea className={textareaClass} value={edit.summary} onChange={(e) => setEdit({ ...edit, summary: e.target.value })} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">{t('adminGlp.studyFinalReport.methods')}</label>
                  <textarea className={textareaClass} value={edit.methods} onChange={(e) => setEdit({ ...edit, methods: e.target.value })} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">{t('adminGlp.studyFinalReport.results')}</label>
                  <textarea className={textareaClass} value={edit.results} onChange={(e) => setEdit({ ...edit, results: e.target.value })} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">{t('adminGlp.studyFinalReport.conclusions')}</label>
                  <textarea className={textareaClass} value={edit.conclusions} onChange={(e) => setEdit({ ...edit, conclusions: e.target.value })} />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-medium">{t('adminGlp.studyFinalReport.deviations')}</label>
                  <textarea className={textareaClass} value={edit.deviations} onChange={(e) => setEdit({ ...edit, deviations: e.target.value })} />
                </div>
                <div className="flex justify-end gap-2">
                  <Button variant="outline" onClick={() => setEdit(null)}>{t('common.cancel')}</Button>
                  <Button onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}>{t('common.save')}</Button>
                </div>
              </div>
            ) : (
              <div className="space-y-3">
                <div className="grid grid-cols-1 gap-2 text-sm">
                  <div><span className="font-medium">{t('adminGlp.studyFinalReport.view.summary')}</span>{report.summary || '—'}</div>
                  <div><span className="font-medium">{t('adminGlp.studyFinalReport.view.methods')}</span>{report.methods || '—'}</div>
                  <div><span className="font-medium">{t('adminGlp.studyFinalReport.view.results')}</span>{report.results || '—'}</div>
                  <div><span className="font-medium">{t('adminGlp.studyFinalReport.view.conclusions')}</span>{report.conclusions || '—'}</div>
                  <div><span className="font-medium">{t('adminGlp.studyFinalReport.view.deviations')}</span>{report.deviations || '—'}</div>
                </div>
                {report.status !== 'signed' && (
                  <Button variant="outline" size="sm" onClick={() => startEdit(report)}>{t('adminGlp.studyFinalReport.detail.editBody')}</Button>
                )}
              </div>
            )}

            {/* SD 簽署：無 admin 例外，僅本計畫 SD 本人可簽 */}
            {report.status !== 'signed' && (
              <div className="space-y-2 border-t pt-4">
                <label className="text-sm font-medium">{t('adminGlp.studyFinalReport.detail.signSection')}</label>
                <div className="flex gap-2">
                  <Input
                    type="password"
                    placeholder={t('adminGlp.studyFinalReport.detail.passwordPlaceholder')}
                    value={signPassword}
                    onChange={(e) => setSignPassword(e.target.value)}
                  />
                  <Button
                    onClick={() => signMutation.mutate()}
                    disabled={!signPassword || signMutation.isPending}
                  >
                    {t('adminGlp.studyFinalReport.detail.sign')}
                  </Button>
                </div>
              </div>
            )}

            {/* QAU 品保聲明：與報告本文分開授權，且不得為本計畫 SD 本人（服務層強制） */}
            <div className="space-y-2 border-t pt-4">
              <label className="text-sm font-medium">{t('adminGlp.studyFinalReport.detail.qauStatement')}</label>
              {report.qau_signed_at && (
                <p className="text-xs text-muted-foreground">
                  {t('adminGlp.studyFinalReport.detail.qauIssuedAt', { time: new Date(report.qau_signed_at).toLocaleString(uiLocale()) })}
                </p>
              )}
              {canWriteQauStatement ? (
                <div className="space-y-2">
                  {/* 受控欄位 + 「草稿為 null 時回落到報告現值」（CodeRabbit 於 #102 指出）。
                      改掉的是兩件事：
                      ① `defaultValue` 只在首次掛載生效，報告在對話框開著時被 refetch
                         （儲存後 invalidate 就會）不會反映到畫面；
                      ② 舊寫法的 disabled 看的是空的草稿 state，於是**已經有品保聲明的報告
                         打開後按鈕是灰的**，非得先打一個字才能存。 */}
                  <textarea
                    className={textareaClass}
                    value={qauStatement ?? report.qau_statement ?? ''}
                    onChange={(e) => setQauStatement(e.target.value)}
                    placeholder={t('adminGlp.studyFinalReport.detail.qauPlaceholder')}
                  />
                  <div className="flex justify-end">
                    <Button
                      size="sm"
                      onClick={() => qauMutation.mutate()}
                      disabled={!(qauStatement ?? report.qau_statement) || qauMutation.isPending}
                    >
                      {t('adminGlp.studyFinalReport.detail.saveQauStatement')}
                    </Button>
                  </div>
                </div>
              ) : (
                <p className="text-sm text-muted-foreground">{report.qau_statement || t('adminGlp.studyFinalReport.detail.noQauStatement')}</p>
              )}
            </div>
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>{t('common.closeDialog')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
