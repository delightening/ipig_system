import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Plus, Loader2, CheckCircle2, FileText } from 'lucide-react'

import {
  listApplicationNotices,
  createApplicationNotice,
  activateApplicationNotice,
} from '@/lib/api/applicationNotices'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter,
} from '@/components/ui/dialog'
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { formatDate } from '@/lib/utils'

interface CreateState {
  version_label: string
  title: string
  content: string
  effective_from: string
}
const EMPTY: CreateState = { version_label: '', title: '動物試驗申請須知', content: '', effective_from: '' }

/** 動物試驗申請須知版本登記（admin）：版本號 + 標題 + 正文 + 生效日 + 生效標記。 */
export function ApplicationNoticesTab() {
  const qc = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()
  const [draft, setDraft] = useState<CreateState | null>(null)

  const { data: notices = [], isLoading, isError } = useQuery({
    queryKey: ['application-notices'],
    queryFn: listApplicationNotices,
  })
  const invalidate = () => qc.invalidateQueries({ queryKey: ['application-notices'] })
  const onErr = (e: unknown, msg: string) => toast({ title: '錯誤', description: getApiErrorMessage(e, msg), variant: 'destructive' })

  const createMutation = useMutation({
    mutationFn: (s: CreateState) => createApplicationNotice({
      version_label: s.version_label.trim(),
      title: s.title.trim(),
      content: s.content,
      effective_from: s.effective_from,
    }),
    onSuccess: () => { toast({ title: '成功', description: '已新增須知版本' }); invalidate(); setDraft(null) },
    onError: (e) => onErr(e, '新增失敗'),
  })

  const activateMutation = useMutation({
    mutationFn: activateApplicationNotice,
    onSuccess: () => { toast({ title: '成功', description: '已設為生效版本' }); invalidate() },
    onError: (e) => onErr(e, '設定失敗'),
  })

  const handleActivate = async (id: string, label: string) => {
    const ok = await confirm({
      title: '設為生效版本',
      description: `將「${label}」設為當前生效須知？申請人填表/送審時將以此版本為準（停用其他版本）。`,
      confirmLabel: '確認設為生效',
    })
    if (ok) activateMutation.mutate(id)
  }

  const handleCreate = () => {
    if (!draft) return
    if (!draft.version_label.trim()) { toast({ title: '錯誤', description: '版本號為必填', variant: 'destructive' }); return }
    if (!draft.title.trim()) { toast({ title: '錯誤', description: '標題為必填', variant: 'destructive' }); return }
    if (!draft.content.trim()) { toast({ title: '錯誤', description: '須知正文為必填', variant: 'destructive' }); return }
    if (!draft.effective_from) { toast({ title: '錯誤', description: '生效日為必填', variant: 'destructive' }); return }
    createMutation.mutate(draft)
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">院區層級動物試驗申請須知的版次登記，僅系統管理員可維護；設為生效後申請人送審前須簽署。</p>
        <Button size="sm" onClick={() => setDraft({ ...EMPTY })}>
          <Plus className="h-4 w-4 mr-2" />新增版本
        </Button>
      </div>

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <TableHead>版本號</TableHead>
              <TableHead>標題</TableHead>
              <TableHead>生效日</TableHead>
              <TableHead>生效中</TableHead>
              <TableHead>建立者</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow><TableCell colSpan={6} className="p-0"><TableSkeleton rows={8} cols={6} /></TableCell></TableRow>
            ) : isError ? (
              <TableRow><TableCell colSpan={6} className="py-8 text-center text-status-error-text">載入失敗，請稍後再試。</TableCell></TableRow>
            ) : notices.length > 0 ? (
              notices.map((n) => (
                <TableRow key={n.id}>
                  <TableCell className="font-medium">{n.version_label}</TableCell>
                  <TableCell className="max-w-[240px] whitespace-normal break-words">{n.title}</TableCell>
                  <TableCell>{n.effective_from ? formatDate(n.effective_from) : '-'}</TableCell>
                  <TableCell>{n.is_active ? <Badge variant="success">生效中</Badge> : '-'}</TableCell>
                  <TableCell>{n.created_by_name || '-'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-1">
                      {!n.is_active && (
                        <Button variant="ghost" size="icon" title="設為生效" aria-label="設為生效" disabled={activateMutation.isPending} onClick={() => handleActivate(n.id, n.version_label)}>
                          <CheckCircle2 className="h-4 w-4 text-status-success-solid" />
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={6} icon={FileText} title="尚無須知版本" />
            )}
          </TableBody>
        </Table>
      </div>

      {/* 新增版本 */}
      <Dialog open={!!draft} onOpenChange={(o) => !o && setDraft(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增申請須知版本</DialogTitle></DialogHeader>
          {draft && (
            <div className="space-y-3">
              <div className="grid gap-2">
                <Label>版本號 *</Label>
                <Input value={draft.version_label} onChange={(e) => setDraft({ ...draft, version_label: e.target.value })} placeholder="例：2025年9月修訂版" />
              </div>
              <div className="grid gap-2">
                <Label>標題 *</Label>
                <Input value={draft.title} onChange={(e) => setDraft({ ...draft, title: e.target.value })} placeholder="動物試驗申請須知" />
              </div>
              <div className="grid gap-2">
                <Label>生效日 *</Label>
                <Input type="date" value={draft.effective_from} onChange={(e) => setDraft({ ...draft, effective_from: e.target.value })} />
              </div>
              <div className="grid gap-2">
                <Label>須知正文 *（支援 markdown，顯示給申請人閱讀）</Label>
                <Textarea rows={8} value={draft.content} onChange={(e) => setDraft({ ...draft, content: e.target.value })} placeholder="貼上須知正文…（歷史紙本版本可填「（紙本版本，詳見附件 PDF）」）" />
              </div>
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setDraft(null)}>取消</Button>
            <Button onClick={handleCreate} disabled={createMutation.isPending}>
              {createMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
