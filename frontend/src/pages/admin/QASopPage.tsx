/**
 * QA SOP 文件管理頁
 *
 * - SOP 列表（篩選狀態、類別）
 * - 新增 / 更新 SOP 文件
 * - 閱讀確認（Acknowledge）功能
 */

import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Plus, BookOpen, CheckCircle2 } from 'lucide-react'

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
import { useAuthStore } from '@/stores/auth'
import {
  listSopDocuments, createSopDocument, updateSopDocument, acknowledgeSop,
  type SopStatus,
} from '@/lib/api/qaPlan'

const STATUS_LABELS: Record<SopStatus, string> = {
  draft: '草稿', active: '生效中', obsolete: '已廢止',
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
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('qau.sop.manage')

  const [filterStatus, setFilterStatus] = useState('')
  const [filterCategory, setFilterCategory] = useState('')
  const [open, setOpen] = useState(false)
  const [editId, setEditId] = useState<string | null>(null)
  const [form, setForm] = useState<SopForm>(defaultForm())

  const { data: sopList = [], isLoading } = useQuery({
    queryKey: ['qa-sop', filterStatus, filterCategory],
    queryFn: () => listSopDocuments({
      status: filterStatus || undefined,
      category: filterCategory || undefined,
    }),
  })

  const saveMutation = useMutation({
    mutationFn: () => {
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
      return editId ? updateSopDocument(editId, payload) : createSopDocument(payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['qa-sop'] })
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
    setOpen(true)
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="SOP 文件管理"
        description="標準作業程序文件版本控制與閱讀確認"
        actions={canManage ? (
          <Button size="sm" onClick={openCreate}>
            <Plus className="h-4 w-4 mr-2" />
            新增 SOP
          </Button>
        ) : undefined}
      />

      <div className="flex gap-3">
        <Select value={filterStatus} onValueChange={setFilterStatus}>
          <SelectTrigger className="w-32">
            <SelectValue placeholder="狀態" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="">全部狀態</SelectItem>
            <SelectItem value="draft">草稿</SelectItem>
            <SelectItem value="active">生效中</SelectItem>
            <SelectItem value="obsolete">已廢止</SelectItem>
          </SelectContent>
        </Select>
        <Input
          className="w-40"
          placeholder="類別篩選"
          value={filterCategory}
          onChange={e => setFilterCategory(e.target.value)}
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <BookOpen className="h-5 w-5" />
            SOP 文件列表
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <Skeleton variant="table" rows={5} />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>文件編號</TableHead>
                  <TableHead>標題</TableHead>
                  <TableHead>版本</TableHead>
                  <TableHead>類別</TableHead>
                  <TableHead>生效日</TableHead>
                  <TableHead>審查日</TableHead>
                  <TableHead>狀態</TableHead>
                  <TableHead>確認數</TableHead>
                  <TableHead className="w-32">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sopList.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={9} className="text-center text-muted-foreground py-8">
                      尚無 SOP 文件
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
                        {STATUS_LABELS[row.status]}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <span className="flex items-center gap-1 text-sm">
                        {row.acknowledged_by_me && (
                          <CheckCircle2 className="h-3.5 w-3.5 text-green-600" />
                        )}
                        {row.ack_count} 人
                      </span>
                    </TableCell>
                    <TableCell>
                      <div className="flex gap-1">
                        {!row.acknowledged_by_me && row.status === 'active' && (
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => ackMutation.mutate(row.id)}
                            disabled={ackMutation.isPending}
                          >
                            確認閱讀
                          </Button>
                        )}
                        {canManage && (
                          <Button size="sm" variant="ghost" onClick={() => openEdit(row)}>
                            編輯
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
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>{editId ? '編輯 SOP 文件' : '新增 SOP 文件'}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3">
            <div className="space-y-1">
              <Label>標題</Label>
              <Input value={form.title} onChange={e => setForm(f => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label>版本</Label>
                <Input value={form.version} onChange={e => setForm(f => ({ ...f, version: e.target.value }))} />
              </div>
              <div className="space-y-1">
                <Label>類別</Label>
                <Input value={form.category} onChange={e => setForm(f => ({ ...f, category: e.target.value }))} />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label>生效日期</Label>
                <Input type="date" value={form.effective_date} onChange={e => setForm(f => ({ ...f, effective_date: e.target.value }))} />
              </div>
              <div className="space-y-1">
                <Label>下次審查日期</Label>
                <Input type="date" value={form.review_date} onChange={e => setForm(f => ({ ...f, review_date: e.target.value }))} />
              </div>
            </div>
            <div className="space-y-1">
              <Label>檔案路徑 / URL</Label>
              <Input value={form.file_path} onChange={e => setForm(f => ({ ...f, file_path: e.target.value }))} placeholder="例：/docs/SOP-0001.pdf" />
            </div>
            {editId && (
              <div className="space-y-1">
                <Label>狀態</Label>
                <Select value={form.status} onValueChange={v => setForm(f => ({ ...f, status: v as SopStatus }))}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="draft">草稿</SelectItem>
                    <SelectItem value="active">生效中</SelectItem>
                    <SelectItem value="obsolete">已廢止</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            )}
            <div className="space-y-1">
              <Label>描述</Label>
              <Textarea rows={2} value={form.description} onChange={e => setForm(f => ({ ...f, description: e.target.value }))} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setOpen(false)}>取消</Button>
            <Button onClick={() => saveMutation.mutate()} disabled={!form.title || saveMutation.isPending}>
              {saveMutation.isPending ? '儲存中…' : '儲存'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
