import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
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
// title 預設值為送往後端存入 DB 的資料（申請須知標題），非 UI 文案，故維持原文不翻譯。
const EMPTY: CreateState = { version_label: '', title: '動物試驗申請須知', content: '', effective_from: '' }

/** 動物試驗申請須知版本登記（admin）：版本號 + 標題 + 正文 + 生效日 + 生效標記。 */
export function ApplicationNoticesTab() {
  const { t } = useTranslation()
  const qc = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()
  const [draft, setDraft] = useState<CreateState | null>(null)

  const { data: notices = [], isLoading, isError } = useQuery({
    queryKey: ['application-notices'],
    queryFn: listApplicationNotices,
  })
  const invalidate = () => qc.invalidateQueries({ queryKey: ['application-notices'] })
  const onErr = (e: unknown, msg: string) => toast({ title: t('common.error'), description: getApiErrorMessage(e, msg), variant: 'destructive' })

  const createMutation = useMutation({
    mutationFn: (s: CreateState) => createApplicationNotice({
      version_label: s.version_label.trim(),
      title: s.title.trim(),
      content: s.content,
      effective_from: s.effective_from,
    }),
    onSuccess: () => { toast({ title: t('common.success'), description: t('protocolPages.applicationNotices.created') }); invalidate(); setDraft(null) },
    onError: (e) => onErr(e, t('protocolPages.applicationNotices.createFailed')),
  })

  const activateMutation = useMutation({
    mutationFn: activateApplicationNotice,
    onSuccess: () => { toast({ title: t('common.success'), description: t('protocolPages.applicationNotices.activated') }); invalidate() },
    onError: (e) => onErr(e, t('protocolPages.applicationNotices.activateFailed')),
  })

  const handleActivate = async (id: string, label: string) => {
    const ok = await confirm({
      title: t('protocolPages.applicationNotices.activateTitle'),
      description: t('protocolPages.applicationNotices.activateConfirm', { label }),
      confirmLabel: t('protocolPages.applicationNotices.activateConfirmLabel'),
    })
    if (ok) activateMutation.mutate(id)
  }

  const handleCreate = () => {
    if (!draft) return
    if (!draft.version_label.trim()) { toast({ title: t('common.error'), description: t('protocolPages.applicationNotices.validation.versionRequired'), variant: 'destructive' }); return }
    if (!draft.title.trim()) { toast({ title: t('common.error'), description: t('protocolPages.applicationNotices.validation.titleRequired'), variant: 'destructive' }); return }
    if (!draft.content.trim()) { toast({ title: t('common.error'), description: t('protocolPages.applicationNotices.validation.contentRequired'), variant: 'destructive' }); return }
    if (!draft.effective_from) { toast({ title: t('common.error'), description: t('protocolPages.applicationNotices.validation.effectiveFromRequired'), variant: 'destructive' }); return }
    createMutation.mutate(draft)
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">{t('protocolPages.applicationNotices.description')}</p>
        <Button size="sm" onClick={() => setDraft({ ...EMPTY })}>
          <Plus className="h-4 w-4 mr-2" />{t('protocolPages.applicationNotices.addVersion')}
        </Button>
      </div>

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <TableHead>{t('protocolPages.applicationNotices.columns.versionLabel')}</TableHead>
              <TableHead>{t('protocolPages.applicationNotices.columns.title')}</TableHead>
              <TableHead>{t('protocolPages.applicationNotices.columns.effectiveFrom')}</TableHead>
              <TableHead>{t('protocolPages.applicationNotices.columns.active')}</TableHead>
              <TableHead>{t('protocolPages.applicationNotices.columns.createdBy')}</TableHead>
              <TableHead className="text-right">{t('common.actions')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow><TableCell colSpan={6} className="p-0"><TableSkeleton rows={8} cols={6} /></TableCell></TableRow>
            ) : isError ? (
              <TableRow><TableCell colSpan={6} className="py-8 text-center text-status-error-text">{t('common.loadFailed')}</TableCell></TableRow>
            ) : notices.length > 0 ? (
              notices.map((n) => (
                <TableRow key={n.id}>
                  <TableCell className="font-medium">{n.version_label}</TableCell>
                  <TableCell className="max-w-[240px] whitespace-normal break-words">{n.title}</TableCell>
                  <TableCell>{n.effective_from ? formatDate(n.effective_from) : '-'}</TableCell>
                  <TableCell>{n.is_active ? <Badge variant="success">{t('protocolPages.applicationNotices.columns.active')}</Badge> : '-'}</TableCell>
                  <TableCell>{n.created_by_name || '-'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-1">
                      {!n.is_active && (
                        <Button variant="ghost" size="icon" title={t('protocolPages.applicationNotices.activate')} aria-label={t('protocolPages.applicationNotices.activate')} disabled={activateMutation.isPending} onClick={() => handleActivate(n.id, n.version_label)}>
                          <CheckCircle2 className="h-4 w-4 text-status-success-solid" />
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={6} icon={FileText} title={t('protocolPages.applicationNotices.empty')} />
            )}
          </TableBody>
        </Table>
      </div>

      {/* 新增版本 */}
      <Dialog open={!!draft} onOpenChange={(o) => !o && setDraft(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('protocolPages.applicationNotices.dialogTitle')}</DialogTitle></DialogHeader>
          {draft && (
            <div className="space-y-3">
              <div className="grid gap-2">
                <Label>{t('protocolPages.applicationNotices.fields.versionLabel')}</Label>
                <Input value={draft.version_label} onChange={(e) => setDraft({ ...draft, version_label: e.target.value })} placeholder={t('protocolPages.applicationNotices.fields.versionLabelPlaceholder')} />
              </div>
              <div className="grid gap-2">
                <Label>{t('protocolPages.applicationNotices.fields.title')}</Label>
                <Input value={draft.title} onChange={(e) => setDraft({ ...draft, title: e.target.value })} placeholder={t('protocolPages.applicationNotices.fields.titlePlaceholder')} />
              </div>
              <div className="grid gap-2">
                <Label>{t('protocolPages.applicationNotices.fields.effectiveFrom')}</Label>
                <Input type="date" value={draft.effective_from} onChange={(e) => setDraft({ ...draft, effective_from: e.target.value })} />
              </div>
              <div className="grid gap-2">
                <Label>{t('protocolPages.applicationNotices.fields.content')}</Label>
                <Textarea rows={8} value={draft.content} onChange={(e) => setDraft({ ...draft, content: e.target.value })} placeholder={t('protocolPages.applicationNotices.fields.contentPlaceholder')} />
              </div>
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setDraft(null)}>{t('common.cancel')}</Button>
            <Button onClick={handleCreate} disabled={createMutation.isPending}>
              {createMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}{t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
