import { useRef } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Download, Loader2, Trash2, Upload } from 'lucide-react'

import api from '@/lib/api'
import {
  listTemplateVersionDocuments,
  uploadTemplateVersionDocument,
  deleteTemplateVersionDocument,
  type TemplateVersionDocument,
} from '@/lib/api/protocolTemplateVersions'
import { Button } from '@/components/ui/button'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'

/** 某計畫書範本版本的現行 SOP / 表單文件：列表 + 上傳 + 下載 + 刪除。 */
export function TemplateVersionDocuments({ versionId }: { versionId: string }) {
  const qc = useQueryClient()
  const fileRef = useRef<HTMLInputElement>(null)
  const { dialogState, confirm } = useConfirmDialog()
  const key = ['template-version-docs', versionId]

  const { data: docs, isLoading, isError } = useQuery({
    queryKey: key,
    queryFn: () => listTemplateVersionDocuments(versionId),
  })

  const uploadMutation = useMutation({
    mutationFn: (file: File) => uploadTemplateVersionDocument(versionId, file),
    onSuccess: () => {
      toast({ title: '成功', description: '已上傳文件' })
      qc.invalidateQueries({ queryKey: key })
    },
    onError: (e: unknown) => toast({ title: '錯誤', description: getApiErrorMessage(e, '上傳失敗'), variant: 'destructive' }),
  })

  const deleteMutation = useMutation({
    mutationFn: (docId: string) => deleteTemplateVersionDocument(versionId, docId),
    onSuccess: () => qc.invalidateQueries({ queryKey: key }),
    onError: (e: unknown) => toast({ title: '錯誤', description: getApiErrorMessage(e, '刪除失敗'), variant: 'destructive' }),
  })

  const downloadMutation = useMutation({
    mutationFn: async (doc: TemplateVersionDocument) => {
      const res = await api.get(`/attachments/${doc.id}/download`, { responseType: 'blob' })
      const url = window.URL.createObjectURL(new Blob([res.data]))
      const link = document.createElement('a')
      link.href = url
      link.setAttribute('download', doc.file_name)
      document.body.appendChild(link)
      link.click()
      link.remove()
      window.URL.revokeObjectURL(url)
    },
    onError: (e: unknown) => toast({ title: '錯誤', description: getApiErrorMessage(e, '下載失敗'), variant: 'destructive' }),
  })

  const onFile = (e: React.ChangeEvent<HTMLInputElement>) => {
    const f = e.target.files?.[0]
    if (f) uploadMutation.mutate(f)
    if (fileRef.current) fileRef.current.value = ''
  }

  const handleDeleteDoc = async (doc: TemplateVersionDocument) => {
    const ok = await confirm({
      title: '刪除文件',
      description: `確定刪除「${doc.file_name}」？此操作會永久刪除實體檔案，無法復原。`,
      variant: 'destructive',
      confirmLabel: '確認刪除',
    })
    if (ok) deleteMutation.mutate(doc.id)
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">現行 SOP 與表單文件（PDF / Word / Excel / 圖片）</p>
        <input ref={fileRef} type="file" className="hidden" onChange={onFile}
          accept=".pdf,.doc,.docx,.xls,.xlsx,.png,.jpg,.jpeg" />
        <Button variant="outline" size="sm" disabled={uploadMutation.isPending} onClick={() => fileRef.current?.click()}>
          {uploadMutation.isPending ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <Upload className="h-4 w-4 mr-2" />}
          上傳文件
        </Button>
      </div>
      {isLoading ? (
        <Loader2 className="h-4 w-4 animate-spin" />
      ) : isError ? (
        <p className="text-sm text-status-error-text">文件載入失敗，請稍後再試。</p>
      ) : !docs || docs.length === 0 ? (
        <p className="text-sm text-muted-foreground">尚無文件。</p>
      ) : (
        <ul className="divide-y rounded-md border">
          {docs.map((doc) => (
            <li key={doc.id} className="flex items-center justify-between gap-2 px-3 py-2">
              <span className="truncate text-sm">{doc.file_name}</span>
              <div className="flex shrink-0 gap-1">
                <Button variant="ghost" size="icon" title="下載" aria-label={`下載 ${doc.file_name}`} disabled={downloadMutation.isPending}
                  onClick={() => downloadMutation.mutate(doc)}>
                  <Download className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon" title="刪除" aria-label={`刪除 ${doc.file_name}`} disabled={deleteMutation.isPending}
                  onClick={() => handleDeleteDoc(doc)}>
                  <Trash2 className="h-4 w-4 text-destructive" />
                </Button>
              </div>
            </li>
          ))}
        </ul>
      )}
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
