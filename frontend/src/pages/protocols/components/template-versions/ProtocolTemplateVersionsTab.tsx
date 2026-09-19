import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Plus, Loader2, Pencil, Trash2, CheckCircle2, FileText } from 'lucide-react'

import {
  listTemplateVersions,
  createTemplateVersion,
  updateTemplateVersion,
  setCurrentTemplateVersion,
  deleteTemplateVersion,
  type ProtocolTemplateVersion,
} from '@/lib/api/protocolTemplateVersions'
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
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { formatDate } from '@/lib/utils'
import { PROTOCOL_FORM_VERSIONS, PROTOCOL_FORM_VERSION_LABELS } from '@/lib/constants/protocolVersionManifests'

import { TemplateVersionDocuments } from './TemplateVersionDocuments'

interface EditState {
  id: string | null
  version_label: string
  effective_date: string
  notes: string
}
const EMPTY_EDIT: EditState = { id: null, version_label: '', effective_date: '', notes: '' }

/** 計畫書範本版本登記（admin）：版本號 + 生效日 + 現行標記 + 每版本 SOP/表單文件。 */
export function ProtocolTemplateVersionsTab() {
  const { t } = useTranslation()
  const qc = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()
  const [edit, setEdit] = useState<EditState | null>(null)
  const [docsFor, setDocsFor] = useState<ProtocolTemplateVersion | null>(null)

  const { data: versions = [], isLoading, isError } = useQuery({
    queryKey: ['protocol-template-versions'],
    queryFn: listTemplateVersions,
  })
  const invalidate = () => qc.invalidateQueries({ queryKey: ['protocol-template-versions'] })
  const onErr = (e: unknown, msg: string) => toast({ title: t('common.error'), description: getApiErrorMessage(e, msg), variant: 'destructive' })

  const saveMutation = useMutation({
    mutationFn: (s: EditState) => {
      const payload = {
        version_label: s.version_label.trim(),
        effective_date: s.effective_date || null,
        notes: s.notes.trim() || null,
      }
      return s.id ? updateTemplateVersion(s.id, payload) : createTemplateVersion(payload)
    },
    onSuccess: () => { toast({ title: t('common.success'), description: t('protocolPages.templateVersions.saved') }); invalidate(); setEdit(null) },
    onError: (e) => onErr(e, t('protocolPages.templateVersions.saveFailed')),
  })

  const setCurrentMutation = useMutation({
    mutationFn: setCurrentTemplateVersion,
    onSuccess: () => { toast({ title: t('common.success'), description: t('protocolPages.templateVersions.setCurrentSuccess') }); invalidate() },
    onError: (e) => onErr(e, t('protocolPages.templateVersions.setCurrentFailed')),
  })

  const deleteMutation = useMutation({
    mutationFn: deleteTemplateVersion,
    onSuccess: () => { toast({ title: t('common.success'), description: t('protocolPages.templateVersions.deleted') }); invalidate() },
    onError: (e) => onErr(e, t('protocols.deleteFailed')),
  })

  const handleDelete = async (v: ProtocolTemplateVersion) => {
    const ok = await confirm({ title: t('protocolPages.templateVersions.deleteTitle'), description: t('protocolPages.templateVersions.deleteConfirm', { label: v.version_label }), variant: 'destructive', confirmLabel: t('common.confirmDelete') })
    if (ok) deleteMutation.mutate(v.id)
  }

  const handleSave = () => {
    if (!edit?.version_label.trim()) { toast({ title: t('common.error'), description: t('protocolPages.templateVersions.versionRequired'), variant: 'destructive' }); return }
    saveMutation.mutate(edit)
  }

  return (
    <div className="space-y-4">
      {/* 補登版本名冊：系統依版本鍵渲染補登欄位（功能真相源為程式常數 VERSION_MANIFESTS，此處為對照說明） */}
      <div className="rounded-lg border bg-muted/30 p-4 text-sm space-y-2">
        <p className="font-medium">{t('protocolPages.templateVersions.manifestTitle')}</p>
        <ul className="space-y-1 text-muted-foreground">
          {PROTOCOL_FORM_VERSIONS.map((v) => (
            <li key={v}>· {PROTOCOL_FORM_VERSION_LABELS[v]}</li>
          ))}
        </ul>
        <p className="text-xs text-muted-foreground">
          {t('protocolPages.templateVersions.manifestHint')}
        </p>
      </div>
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">{t('protocolPages.templateVersions.description')}</p>
        <Button size="sm" onClick={() => setEdit({ ...EMPTY_EDIT })}>
          <Plus className="h-4 w-4 mr-2" />{t('protocolPages.templateVersions.addVersion')}
        </Button>
      </div>

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <TableHead>{t('protocolPages.templateVersions.columns.versionLabel')}</TableHead>
              <TableHead>{t('protocolPages.templateVersions.columns.effectiveDate')}</TableHead>
              <TableHead>{t('protocolPages.templateVersions.columns.current')}</TableHead>
              <TableHead>{t('protocolPages.shared.remark')}</TableHead>
              <TableHead>{t('protocolPages.templateVersions.columns.createdBy')}</TableHead>
              <TableHead className="text-right">{t('common.actions')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow><TableCell colSpan={6} className="py-8 text-center"><Loader2 className="h-5 w-5 animate-spin inline" /></TableCell></TableRow>
            ) : isError ? (
              <TableRow><TableCell colSpan={6} className="py-8 text-center text-status-error-text">{t('common.loadFailed')}</TableCell></TableRow>
            ) : versions.length > 0 ? (
              versions.map((v) => (
                <TableRow key={v.id}>
                  <TableCell className="font-medium">{v.version_label}</TableCell>
                  <TableCell>{v.effective_date ? formatDate(v.effective_date) : '-'}</TableCell>
                  <TableCell>{v.is_current ? <Badge variant="success">{t('protocolPages.templateVersions.columns.current')}</Badge> : '-'}</TableCell>
                  <TableCell className="max-w-[240px] whitespace-normal break-words">{v.notes || '-'}</TableCell>
                  <TableCell>{v.created_by_name || '-'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-1">
                      <Button variant="ghost" size="icon" title={t('protocolPages.templateVersions.manageDocuments')} aria-label={t('protocolPages.templateVersions.manageDocuments')} onClick={() => setDocsFor(v)}><FileText className="h-4 w-4" /></Button>
                      {!v.is_current && (
                        <Button variant="ghost" size="icon" title={t('protocolPages.templateVersions.setCurrent')} aria-label={t('protocolPages.templateVersions.setCurrent')} disabled={setCurrentMutation.isPending} onClick={() => setCurrentMutation.mutate(v.id)}>
                          <CheckCircle2 className="h-4 w-4 text-status-success-solid" />
                        </Button>
                      )}
                      <Button variant="ghost" size="icon" title={t('common.edit')} aria-label={t('common.edit')} onClick={() => setEdit({ id: v.id, version_label: v.version_label, effective_date: v.effective_date ?? '', notes: v.notes ?? '' })}><Pencil className="h-4 w-4" /></Button>
                      <Button variant="ghost" size="icon" title={t('common.delete')} aria-label={t('common.delete')} onClick={() => handleDelete(v)}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={6} icon={FileText} title={t('protocolPages.templateVersions.empty')} />
            )}
          </TableBody>
        </Table>
      </div>

      {/* 新增 / 編輯版本 */}
      <Dialog open={!!edit} onOpenChange={(o) => !o && setEdit(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>{edit?.id ? t('protocolPages.templateVersions.dialogEditTitle') : t('protocolPages.templateVersions.dialogCreateTitle')}</DialogTitle></DialogHeader>
          {edit && (
            <div className="space-y-3">
              <div className="grid gap-2">
                <Label>{t('protocolPages.templateVersions.fields.versionLabel')}</Label>
                <Input value={edit.version_label} onChange={(e) => setEdit({ ...edit, version_label: e.target.value })} placeholder={t('protocolPages.templateVersions.fields.versionLabelPlaceholder')} />
              </div>
              <div className="grid gap-2">
                <Label>{t('protocolPages.templateVersions.columns.effectiveDate')}</Label>
                <Input type="date" value={edit.effective_date} onChange={(e) => setEdit({ ...edit, effective_date: e.target.value })} />
              </div>
              <div className="grid gap-2">
                <Label>{t('protocolPages.shared.remark')}</Label>
                <Textarea rows={2} value={edit.notes} onChange={(e) => setEdit({ ...edit, notes: e.target.value })} placeholder={t('protocolPages.templateVersions.fields.notesPlaceholder')} />
              </div>
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setEdit(null)}>{t('common.cancel')}</Button>
            <Button onClick={handleSave} disabled={saveMutation.isPending}>
              {saveMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}{t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 管理版本文件 */}
      <Dialog open={!!docsFor} onOpenChange={(o) => !o && setDocsFor(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>{t('protocolPages.templateVersions.documentsDialogTitle', { label: docsFor?.version_label })}</DialogTitle></DialogHeader>
          {docsFor && <TemplateVersionDocuments versionId={docsFor.id} />}
        </DialogContent>
      </Dialog>

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
