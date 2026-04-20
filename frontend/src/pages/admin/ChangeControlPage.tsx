import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import { useAuthStore } from '@/stores/auth'
import {
  listChangeRequests,
  createChangeRequest,
  approveChangeRequest,
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
import { Plus, CheckCircle, RefreshCw } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

const CHANGE_TYPES = [
  { value: 'equipment', label: '設備' },
  { value: 'method', label: '方法' },
  { value: 'personnel', label: '人員' },
  { value: 'facility', label: '設施' },
  { value: 'system', label: '系統' },
  { value: 'process', label: '流程' },
]

const STATUS_OPTIONS = [
  { value: 'draft', label: '草稿' },
  { value: 'submitted', label: '已提交' },
  { value: 'under_review', label: '審查中' },
  { value: 'approved', label: '已核准' },
  { value: 'implemented', label: '已實施' },
  { value: 'verified', label: '已驗證' },
  { value: 'rejected', label: '已駁回' },
]

const STATUS_COLORS: Record<string, string> = {
  draft: 'bg-muted text-muted-foreground',
  submitted: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  under_review: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
  approved: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  implemented: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  verified: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  rejected: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
}

const INITIAL_FORM = { title: '', change_type: 'equipment', description: '', justification: '' }

export function ChangeControlPage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('change.request.manage')
  const canApprove = hasPermission('change.request.approve')

  const [filterStatus, setFilterStatus] = useState<string>('')
  const [filterType, setFilterType] = useState<string>('')
  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState(INITIAL_FORM)

  const { data: requests = [], isLoading } = useQuery({
    queryKey: ['change-requests', filterStatus, filterType],
    queryFn: () => listChangeRequests({
      status: filterStatus || undefined,
      change_type: filterType || undefined,
    }),
  })

  const createMutation = useMutation({
    mutationFn: () =>
      createChangeRequest({
        title: form.title,
        change_type: form.change_type,
        description: form.description,
        justification: form.justification || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['change-requests'] })
      setShowCreate(false)
      setForm(INITIAL_FORM)
      toast({ title: '變更申請已建立' })
    },
    onError: (err: unknown) => toast({ title: '建立失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  const approveMutation = useMutation({
    mutationFn: (id: string) => approveChangeRequest(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['change-requests'] })
      toast({ title: '變更已核准' })
    },
    onError: (err: unknown) => toast({ title: '核准失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">變更控制</h1>
          <p className="text-muted-foreground">ISO 9001 變更管理</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            新增變更申請
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
                {CHANGE_TYPES.map((ct) => (
                  <SelectItem key={ct.value} value={ct.value}>{ct.label}</SelectItem>
                ))}
              </SelectContent>
            </Select>
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
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                <TableHead>變更編號</TableHead>
                <TableHead>標題</TableHead>
                <TableHead>類型</TableHead>
                <TableHead>狀態</TableHead>
                <TableHead>申請人</TableHead>
                <TableHead>建立時間</TableHead>
                <TableHead>操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={7} className="p-0"><TableSkeleton rows={5} cols={7} /></TableCell></TableRow>
              ) : requests.length === 0 ? (
                <TableEmptyRow colSpan={7} icon={RefreshCw} title="尚無變更申請" />
              ) : (
                requests.map((cr) => (
                  <TableRow key={cr.id}>
                    <TableCell className="font-mono text-sm">{cr.change_number}</TableCell>
                    <TableCell className="font-medium">{cr.title}</TableCell>
                    <TableCell>{CHANGE_TYPES.find((ct) => ct.value === cr.change_type)?.label ?? cr.change_type}</TableCell>
                    <TableCell>
                      <Badge className={STATUS_COLORS[cr.status] ?? ''}>
                        {STATUS_OPTIONS.find((s) => s.value === cr.status)?.label ?? cr.status}
                      </Badge>
                    </TableCell>
                    <TableCell>{cr.requester_name ?? '-'}</TableCell>
                    <TableCell>{new Date(cr.created_at).toLocaleDateString('zh-TW')}</TableCell>
                    <TableCell>
                      {canApprove && (cr.status === 'submitted' || cr.status === 'under_review') && (
                        <Button variant="outline" size="sm" onClick={() => approveMutation.mutate(cr.id)}>
                          <CheckCircle className="h-3 w-3 mr-1" />核准
                        </Button>
                      )}
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
          <DialogHeader><DialogTitle>新增變更申請</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">標題 *</label>
              <Input value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">變更類型 *</label>
              <Select value={form.change_type} onValueChange={(v) => setForm((f) => ({ ...f, change_type: v }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  {CHANGE_TYPES.map((ct) => (
                    <SelectItem key={ct.value} value={ct.value}>{ct.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">描述 *</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                value={form.description}
                onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">變更理由</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                value={form.justification}
                onChange={(e) => setForm((f) => ({ ...f, justification: e.target.value }))}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>取消</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.title || !form.description || createMutation.isPending}>
              建立
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
