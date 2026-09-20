/**
 * QA SOP 文件管理頁
 *
 * - SOP 列表（篩選狀態、類別）
 * - 新增 / 更新 SOP 文件
 * - 閱讀確認（Acknowledge）功能
 */

import { useState, useRef } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, BookOpen, CheckCircle2, Download, Upload } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import api from '@/lib/api'
import { useToast } from '@/components/ui/use-toast'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table, TableBody, TableCell, TableHead, TableHeader, TableRow,
} from '@/components/ui/table'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { Skeleton } from '@/components/ui/skeleton'
import { useAuthHasPermission } from '@/stores/auth'
import {
  listSopDocuments, createSopDocument, updateSopDocument, acknowledgeSop,
  type SopStatus,
} from '@/lib/api/qaPlan'

// 值為 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const STATUS_LABEL_KEYS: Record<SopStatus, string> = {
  draft: 'adminGlp.shared.statusLabel.draft',
  active: 'adminGlp.shared.statusLabel.active',
  obsolete: 'adminGlp.shared.statusLabel.obsolete',
}

const STATUS_VARIANTS: Record<SopStatus, 'default' | 'secondary' | 'destructive' | 'outline'> = {
  draft: 'secondary', active: 'default', obsolete: 'outline',
}

interface SopForm {
  title: string
  version: string
  category: string
  file_path: string
  effective_date: string
  review_date: string
  description: string
  status: SopStatus
}

const defaultForm = (): SopForm => ({
  title: '', version: '1.0', category: '', file_path: '',
  effective_date: '', review_date: '', description: '', status: 'draft',
})

export function QASopPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { toast } = useToast()
  const hasPermission = useAuthHasPermission()
  const canManage = hasPermission('qau.sop.manage')

  const [filterStatus, setFilterStatus] = useState('all')
  const [filterCategory, setFilterCategory] = useState('')
  const [open, setOpen] = useState(false)
  const [editId, setEditId] = useState<string | null>(null)
  const [form, setForm] = useState<SopForm>(defaultForm())
  const [uploadFile, setUploadFile] = useState<File | null>(null)
  const [uploading, setUploading] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const { data: sopList = [], isLoading } = useQuery({
    queryKey: ['qa-sop', filterStatus, filterCategory],
    queryFn: () => listSopDocuments({
      status: filterStatus !== 'all' ? filterStatus : undefined,
      category: filterCategory || undefined,
    }),
  })

  const saveMutation = useMutation({
    mutationFn: async () => {
      const payload = {
        title: form.title,
        version: form.version,
        category: form.category || undefined,
        file_path: form.file_path || undefined,
        effective_date: form.effective_date || undefined,
        review_date: form.review_date || undefined,
        description: form.description || undefined,
        status: editId ? form.status : undefined,
      }
      const result = editId
        ? await updateSopDocument(editId, payload)
        : await createSopDocument(payload)

      // 儲存成功後上傳檔案
      if (uploadFile) {
        const sopId = editId ?? result.id
        setUploading(true)
        try {
          const formData = new FormData()
          formData.append('file', uploadFile)
          await api.post(`/qau/sop/${sopId}/upload`, formData)
        } finally {
          setUploading(false)
        }
      }

      return result
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['qa-sop'] })
      setUploadFile(null)
      setOpen(false)
    },
  })

  const ackMutation = useMutation({
    mutationFn: (id: string) => acknowledgeSop(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['qa-sop'] }),
  })

  const openCreate = () => {
    setEditId(null)
    setForm(defaultForm())
    setUploadFile(null)
    setOpen(true)
  }

  const openEdit = (row: typeof sopList[number]) => {
    setEditId(row.id)
    setForm({
      title: row.title,
      version: row.version,
      category: row.category ?? '',
      file_path: row.file_path ?? '',
      effective_date: row.effective_date ?? '',
      review_date: row.review_date ?? '',
      description: row.description ?? '',
      status: row.status,
    })
    setUploadFile(null)
    setOpen(true)
  }

  const handleDownload = async (id: string, title: string) => {
    try {
      const res = await api.get(`/qau/sop/${id}/download`, { responseType: 'blob' })
      const contentDisposition = res.headers['content-disposition'] ?? ''
      const filenameMatch = contentDisposition.match(/filename\*?=(?:UTF-8'')?["']?([^"';\n]+)/)
      const filename = filenameMatch ? decodeURIComponent(filenameMatch[1]) : `${title}.pdf`
      const url = URL.createObjectURL(res.data)
      const a = document.createElement('a')
      a.href = url
      a.download = filename
      a.click()
      URL.revokeObjectURL(url)
    } catch {
      toast({ variant: 'destructive', title: t('common.downloadFailed') })
    }
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('adminGlp.qaSop.title')}
        description={t('adminGlp.qaSop.description')}
        actions={canManage ? (
          <Button size="sm" onClick={openCreate}>
            <Plus className="h-4 w-4 mr-2" />
            {t('adminGlp.qaSop.create')}
          </Button>
        ) : undefined}
      />

      <div className="flex gap-3">
        <Select value={filterStatus} onValueChange={setFilterStatus}>
          <SelectTrigger className="w-32">
            <SelectValue placeholder={t('adminGlp.shared.status')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t('common.allStatus')}</SelectItem>
            <SelectItem value="draft">{t(STATUS_LABEL_KEYS.draft)}</SelectItem>
            <SelectItem value="active">{t(STATUS_LABEL_KEYS.active)}</SelectItem>
            <SelectItem value="obsolete">{t(STATUS_LABEL_KEYS.obsolete)}</SelectItem>
          </SelectContent>
        </Select>
        <Input
          className="w-40"
          placeholder={t('adminGlp.qaSop.categoryFilter')}
          value={filterCategory}
          onChange={e => setFilterCategory(e.target.value)}
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <BookOpen className="h-5 w-5" />
            {t('adminGlp.qaSop.listTitle')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <Skeleton variant="table" rows={5} />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t('adminGlp.shared.documentNumber')}</TableHead>
                  <TableHead>{t('adminGlp.shared.title')}</TableHead>
                  <TableHead>{t('adminGlp.shared.version')}</TableHead>
                  <TableHead>{t('adminGlp.shared.category')}</TableHead>
                  <TableHead>{t('adminGlp.qaSop.col.effectiveDate')}</TableHead>
                  <TableHead>{t('adminGlp.qaSop.col.reviewDate')}</TableHead>
                  <TableHead>{t('adminGlp.shared.status')}</TableHead>
                  <TableHead>{t('adminGlp.qaSop.col.acknowledgments')}</TableHead>
                  <TableHead className="w-32">{t('common.actions')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sopList.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={9} className="text-center text-muted-foreground py-8">
                      {t('adminGlp.qaSop.empty')}
                    </TableCell>
                  </TableRow>
                ) : sopList.map(row => (
                  <TableRow key={row.id}>
                    <TableCell className="font-mono text-sm">{row.document_number}</TableCell>
                    <TableCell>{row.title}</TableCell>
                    <TableCell>{row.version}</TableCell>
                    <TableCell>{row.category ?? '—'}</TableCell>
                    <TableCell>{row.effective_date ?? '—'}</TableCell>
                    <TableCell>{row.review_date ?? '—'}</TableCell>
                    <TableCell>
                      <Badge variant={STATUS_VARIANTS[row.status]}>
                        {t(STATUS_LABEL_KEYS[row.status])}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <span className="flex items-center gap-1 text-sm">
                        {row.acknowledged_by_me && (
                          <CheckCircle2 className="h-3.5 w-3.5 text-green-600" />
                        )}
                        {t('adminGlp.qaSop.ackPeople', { count: row.ack_count })}
                      </span>
                    </TableCell>
                    <TableCell>
                      <div className="flex gap-1">
                        {row.file_path && (
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => handleDownload(row.id, row.title)}
                          >
                            <Download className="h-3.5 w-3.5 mr-1" />
                            {t('adminGlp.qaSop.download')}
                          </Button>
                        )}
                        {!row.acknowledged_by_me && row.status === 'active' && (
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => ackMutation.mutate(row.id)}
                            disabled={ackMutation.isPending}
                          >
                            {t('adminGlp.qaSop.acknowledge')}
                          </Button>
                        )}
                        {canManage && (
                          <Button size="sm" variant="ghost" onClick={() => openEdit(row)}>
                            {t('common.edit')}
                          </Button>
                        )}
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* 新增/編輯對話框 */}
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editId ? t('adminGlp.qaSop.dialog.edit') : t('adminGlp.qaSop.dialog.create')}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-1">
              <Label>{t('adminGlp.shared.title')}</Label>
              <Input value={form.title} onChange={e => setForm(f => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label>{t('adminGlp.shared.version')}</Label>
                <Input value={form.version} onChange={e => setForm(f => ({ ...f, version: e.target.value }))} />
              </div>
              <div className="space-y-1">
                <Label>{t('adminGlp.shared.category')}</Label>
                <Input value={form.category} onChange={e => setForm(f => ({ ...f, category: e.target.value }))} />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label>{t('adminGlp.qaSop.dialog.effectiveDate')}</Label>
                <Input type="date" value={form.effective_date} onChange={e => setForm(f => ({ ...f, effective_date: e.target.value }))} />
              </div>
              <div className="space-y-1">
                <Label>{t('adminGlp.qaSop.dialog.nextReviewDate')}</Label>
                <Input type="date" value={form.review_date} onChange={e => setForm(f => ({ ...f, review_date: e.target.value }))} />
              </div>
            </div>
            <div className="space-y-1">
              <Label>{t('adminGlp.qaSop.dialog.file')}</Label>
              <input
                ref={fileInputRef}
                type="file"
                accept=".pdf,.doc,.docx"
                className="hidden"
                onChange={e => {
                  const file = e.target.files?.[0]
                  if (file) {
                    if (file.size > 30 * 1024 * 1024) {
                      toast({ variant: 'destructive', title: t('adminGlp.qaSop.dialog.fileTooLarge') })
                      return
                    }
                    setUploadFile(file)
                  }
                }}
              />
              <div
                className="flex items-center gap-3 rounded-md border border-input px-3 py-2 cursor-pointer hover:bg-muted/50"
                onClick={() => fileInputRef.current?.click()}
              >
                <Upload className="h-4 w-4 text-muted-foreground shrink-0" />
                <span className="text-sm truncate">
                  {uploadFile
                    ? uploadFile.name
                    : form.file_path
                      ? t('adminGlp.qaSop.dialog.uploaded', { name: form.file_path.split('/').pop() })
                      : t('adminGlp.qaSop.dialog.selectFile')}
                </span>
              </div>
              {uploadFile && (
                <p className="text-xs text-muted-foreground">
                  {t('adminGlp.qaSop.dialog.uploadOnSave', { size: (uploadFile.size / 1024 / 1024).toFixed(1) })}
                </p>
              )}
            </div>
            {editId && (
              <div className="space-y-1">
                <Label>{t('adminGlp.shared.status')}</Label>
                <Select value={form.status} onValueChange={v => setForm(f => ({ ...f, status: v as SopStatus }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="draft">{t(STATUS_LABEL_KEYS.draft)}</SelectItem>
                    <SelectItem value="active">{t(STATUS_LABEL_KEYS.active)}</SelectItem>
                    <SelectItem value="obsolete">{t(STATUS_LABEL_KEYS.obsolete)}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            )}
            <div className="space-y-1">
              <Label>{t('adminGlp.shared.description')}</Label>
              <Textarea rows={2} value={form.description} onChange={e => setForm(f => ({ ...f, description: e.target.value }))} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setOpen(false)}>{t('common.cancel')}</Button>
            <Button onClick={() => saveMutation.mutate()} disabled={!form.title || saveMutation.isPending || uploading}>
              {uploading ? t('adminGlp.qaSop.dialog.uploading') : saveMutation.isPending ? t('adminGlp.shared.saving') : t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
