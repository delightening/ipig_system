import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import { useAuthStore } from '@/stores/auth'
import {
  listControlledDocuments,
  createControlledDocument,
  updateControlledDocument,
  approveControlledDocument,
  acknowledgeDocument,
  type ControlledDocument,
} from '@/lib/api/glpCompliance'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
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
import { useTranslation } from 'react-i18next'
import { Plus, FileCheck, CheckCircle } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

const DOC_TYPES = [
  { value: 'quality_manual', label: '品質手冊' },
  { value: 'sop', label: 'SOP' },
  { value: 'form', label: '表單' },
  { value: 'external', label: '外部文件' },
  { value: 'policy', label: '政策' },
  { value: 'report', label: '報告' },
]

const STATUS_COLORS: Record<string, string> = {
  draft: 'bg-muted text-muted-foreground',
  under_review: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
  approved: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  active: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  obsolete: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
}

export function DocumentControlPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('dms.document.manage')
  const canApprove = hasPermission('dms.document.approve')

  const [filterType, setFilterType] = useState<string>('')
  const [filterStatus, setFilterStatus] = useState<string>('')
  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState({ title: '', doc_type: 'sop', category: '', retention_years: '' })

  const { data: documents = [], isLoading } = useQuery({
    queryKey: ['controlled-documents', filterType, filterStatus],
    queryFn: () =>
      listControlledDocuments({
        doc_type: filterType || undefined,
        status: filterStatus || undefined,
      }),
  })

  const createMutation = useMutation({
    mutationFn: () =>
      createControlledDocument({
        title: form.title,
        doc_type: form.doc_type,
        category: form.category || undefined,
        retention_years: form.retention_years ? Number(form.retention_years) : undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['controlled-documents'] })
      setShowCreate(false)
      setForm({ title: '', doc_type: 'sop', category: '', retention_years: '' })
      toast({ title: '文件已建立' })
    },
    onError: (err: unknown) => toast({ title: '建立失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const approveMutation = useMutation({
    mutationFn: (id: string) => approveControlledDocument(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['controlled-documents'] })
      toast({ title: '文件已核准' })
    },
    onError: (err: unknown) => toast({ title: '核准失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const acknowledgeMutation = useMutation({
    mutationFn: (id: string) => acknowledgeDocument(id),
    onSuccess: () => toast({ title: '已簽收' }),
    onError: (err: unknown) => toast({ title: '簽收失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">文件控制系統</h1>
          <p className="text-muted-foreground">ISO 17025 / ISO 9001 受控文件管理</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            新增文件
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <div className="flex gap-4">
            <Select value={filterType} onValueChange={setFilterType}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="所有類型" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">所有類型</SelectItem>
                {DOC_TYPES.map((dt) => (
                  <SelectItem key={dt.value} value={dt.value}>{dt.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={filterStatus} onValueChange={setFilterStatus}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="所有狀態" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">所有狀態</SelectItem>
                <SelectItem value="draft">草稿</SelectItem>
                <SelectItem value="under_review">審查中</SelectItem>
                <SelectItem value="approved">已核准</SelectItem>
                <SelectItem value="active">生效中</SelectItem>
                <SelectItem value="obsolete">已廢止</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>文件編號</TableHead>
                <TableHead>標題</TableHead>
                <TableHead>類型</TableHead>
                <TableHead>版本</TableHead>
                <TableHead>狀態</TableHead>
                <TableHead>負責人</TableHead>
                <TableHead>操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={7} className="text-center py-8">載入中...</TableCell></TableRow>
              ) : documents.length === 0 ? (
                <TableRow><TableCell colSpan={7} className="text-center py-8 text-muted-foreground">尚無文件</TableCell></TableRow>
              ) : (
                documents.map((doc) => (
                  <TableRow key={doc.id}>
                    <TableCell className="font-mono text-sm">{doc.doc_number}</TableCell>
                    <TableCell className="font-medium">{doc.title}</TableCell>
                    <TableCell>{DOC_TYPES.find((d) => d.value === doc.doc_type)?.label ?? doc.doc_type}</TableCell>
                    <TableCell>v{doc.current_version}</TableCell>
                    <TableCell>
                      <Badge className={STATUS_COLORS[doc.status] ?? ''}>{doc.status}</Badge>
                    </TableCell>
                    <TableCell>{doc.owner_name ?? '-'}</TableCell>
                    <TableCell>
                      <div className="flex gap-1">
                        {canApprove && (doc.status === 'draft' || doc.status === 'under_review') && (
                          <Button variant="outline" size="sm" onClick={() => approveMutation.mutate(doc.id)}>
                            <FileCheck className="h-3 w-3 mr-1" />核准
                          </Button>
                        )}
                        <Button variant="ghost" size="sm" onClick={() => acknowledgeMutation.mutate(doc.id)}>
                          <CheckCircle className="h-3 w-3 mr-1" />簽收
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增受控文件</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">標題 *</label>
              <Input value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">文件類型 *</label>
              <Select value={form.doc_type} onValueChange={(v) => setForm((f) => ({ ...f, doc_type: v }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {DOC_TYPES.map((dt) => (
                    <SelectItem key={dt.value} value={dt.value}>{dt.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">分類</label>
              <Input value={form.category} onChange={(e) => setForm((f) => ({ ...f, category: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">保存年限</label>
              <Input type="number" value={form.retention_years} onChange={(e) => setForm((f) => ({ ...f, retention_years: e.target.value }))} />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>取消</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.title || createMutation.isPending}>
              建立
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
