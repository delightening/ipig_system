import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
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
  const qc = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()
  const [edit, setEdit] = useState<EditState | null>(null)
  const [docsFor, setDocsFor] = useState<ProtocolTemplateVersion | null>(null)

  const { data: versions = [], isLoading, isError } = useQuery({
    queryKey: ['protocol-template-versions'],
    queryFn: listTemplateVersions,
  })
  const invalidate = () => qc.invalidateQueries({ queryKey: ['protocol-template-versions'] })
  const onErr = (e: unknown, msg: string) => toast({ title: '錯誤', description: getApiErrorMessage(e, msg), variant: 'destructive' })

  const saveMutation = useMutation({
    mutationFn: (s: EditState) => {
      const payload = {
        version_label: s.version_label.trim(),
        effective_date: s.effective_date || null,
        notes: s.notes.trim() || null,
      }
      return s.id ? updateTemplateVersion(s.id, payload) : createTemplateVersion(payload)
    },
    onSuccess: () => { toast({ title: '成功', description: '已儲存版本' }); invalidate(); setEdit(null) },
    onError: (e) => onErr(e, '儲存失敗'),
  })

  const setCurrentMutation = useMutation({
    mutationFn: setCurrentTemplateVersion,
    onSuccess: () => { toast({ title: '成功', description: '已設為現行版本' }); invalidate() },
    onError: (e) => onErr(e, '設定失敗'),
  })

  const deleteMutation = useMutation({
    mutationFn: deleteTemplateVersion,
    onSuccess: () => { toast({ title: '成功', description: '已刪除版本' }); invalidate() },
    onError: (e) => onErr(e, '刪除失敗'),
  })

  const handleDelete = async (v: ProtocolTemplateVersion) => {
    const ok = await confirm({ title: '刪除版本', description: `確定刪除範本版本「${v.version_label}」？`, variant: 'destructive', confirmLabel: '確認刪除' })
    if (ok) deleteMutation.mutate(v.id)
  }

  const handleSave = () => {
    if (!edit?.version_label.trim()) { toast({ title: '錯誤', description: '版本號為必填', variant: 'destructive' }); return }
    saveMutation.mutate(edit)
  }

  return (
    <div className="space-y-4">
      {/* 補登版本名冊：系統依版本鍵渲染補登欄位（功能真相源為程式常數 VERSION_MANIFESTS，此處為對照說明） */}
      <div className="rounded-lg border bg-muted/30 p-4 text-sm space-y-2">
        <p className="font-medium">補登計畫書版本對照（版本名冊）</p>
        <ul className="space-y-1 text-muted-foreground">
          {PROTOCOL_FORM_VERSIONS.map((v) => (
            <li key={v}>· {PROTOCOL_FORM_VERSION_LABELS[v]}</li>
          ))}
        </ul>
        <p className="text-xs text-muted-foreground">
          補登匯入時「先選版本」即依此決定顯示欄位；C 與 D 欄位結構相同，E/F 逐版增修。
        </p>
      </div>
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">院區層級計畫書範本 / 申請表單 + SOP 的版本登記，僅系統管理員可維護。</p>
        <Button size="sm" onClick={() => setEdit({ ...EMPTY_EDIT })}>
          <Plus className="h-4 w-4 mr-2" />新增版本
        </Button>
      </div>

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <TableHead>版本號</TableHead>
              <TableHead>生效日</TableHead>
              <TableHead>現行</TableHead>
              <TableHead>備註</TableHead>
              <TableHead>建立者</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow><TableCell colSpan={6} className="py-8 text-center"><Loader2 className="h-5 w-5 animate-spin inline" /></TableCell></TableRow>
            ) : isError ? (
              <TableRow><TableCell colSpan={6} className="py-8 text-center text-status-error-text">載入失敗，請稍後再試。</TableCell></TableRow>
            ) : versions.length > 0 ? (
              versions.map((v) => (
                <TableRow key={v.id}>
                  <TableCell className="font-medium">{v.version_label}</TableCell>
                  <TableCell>{v.effective_date ? formatDate(v.effective_date) : '-'}</TableCell>
                  <TableCell>{v.is_current ? <Badge variant="success">現行</Badge> : '-'}</TableCell>
                  <TableCell className="max-w-[240px] whitespace-normal break-words">{v.notes || '-'}</TableCell>
                  <TableCell>{v.created_by_name || '-'}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-1">
                      <Button variant="ghost" size="icon" title="管理文件" aria-label="管理文件" onClick={() => setDocsFor(v)}><FileText className="h-4 w-4" /></Button>
                      {!v.is_current && (
                        <Button variant="ghost" size="icon" title="設為現行" aria-label="設為現行" disabled={setCurrentMutation.isPending} onClick={() => setCurrentMutation.mutate(v.id)}>
                          <CheckCircle2 className="h-4 w-4 text-status-success-solid" />
                        </Button>
                      )}
                      <Button variant="ghost" size="icon" title="編輯" aria-label="編輯" onClick={() => setEdit({ id: v.id, version_label: v.version_label, effective_date: v.effective_date ?? '', notes: v.notes ?? '' })}><Pencil className="h-4 w-4" /></Button>
                      <Button variant="ghost" size="icon" title="刪除" aria-label="刪除" onClick={() => handleDelete(v)}><Trash2 className="h-4 w-4 text-destructive" /></Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={6} icon={FileText} title="尚無範本版本" />
            )}
          </TableBody>
        </Table>
      </div>

      {/* 新增 / 編輯版本 */}
      <Dialog open={!!edit} onOpenChange={(o) => !o && setEdit(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>{edit?.id ? '編輯範本版本' : '新增範本版本'}</DialogTitle></DialogHeader>
          {edit && (
            <div className="space-y-3">
              <div className="grid gap-2">
                <Label>版本號 *</Label>
                <Input value={edit.version_label} onChange={(e) => setEdit({ ...edit, version_label: e.target.value })} placeholder="例：v3.0" />
              </div>
              <div className="grid gap-2">
                <Label>生效日</Label>
                <Input type="date" value={edit.effective_date} onChange={(e) => setEdit({ ...edit, effective_date: e.target.value })} />
              </div>
              <div className="grid gap-2">
                <Label>備註</Label>
                <Textarea rows={2} value={edit.notes} onChange={(e) => setEdit({ ...edit, notes: e.target.value })} placeholder="版本說明（選填）" />
              </div>
            </div>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setEdit(null)}>取消</Button>
            <Button onClick={handleSave} disabled={saveMutation.isPending}>
              {saveMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 管理版本文件 */}
      <Dialog open={!!docsFor} onOpenChange={(o) => !o && setDocsFor(null)}>
        <DialogContent>
          <DialogHeader><DialogTitle>範本版本文件：{docsFor?.version_label}</DialogTitle></DialogHeader>
          {docsFor && <TemplateVersionDocuments versionId={docsFor.id} />}
        </DialogContent>
      </Dialog>

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
