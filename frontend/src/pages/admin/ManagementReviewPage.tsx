import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import { useAuthStore } from '@/stores/auth'
import {
  listManagementReviews,
  createManagementReview,
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
import { Plus, ClipboardList } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

const STATUS_OPTIONS = [
  { value: 'planned', label: '已規劃' },
  { value: 'in_progress', label: '進行中' },
  { value: 'completed', label: '已完成' },
  { value: 'closed', label: '已結案' },
]

const STATUS_COLORS: Record<string, string> = {
  planned: 'bg-muted text-muted-foreground',
  in_progress: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
  completed: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  closed: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
}

const INITIAL_FORM = { title: '', review_date: '', agenda: '' }

export function ManagementReviewPage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('glp.management_review.manage')

  const [filterStatus, setFilterStatus] = useState<string>('')
  const [showCreate, setShowCreate] = useState(false)
  const [form, setForm] = useState(INITIAL_FORM)

  const { data: reviews = [], isLoading } = useQuery({
    queryKey: ['management-reviews', filterStatus],
    queryFn: () => listManagementReviews({ status: filterStatus || undefined }),
  })

  const createMutation = useMutation({
    mutationFn: () =>
      createManagementReview({
        title: form.title,
        review_date: form.review_date,
        agenda: form.agenda || undefined,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['management-reviews'] })
      setShowCreate(false)
      setForm(INITIAL_FORM)
      toast({ title: '管理審查已建立' })
    },
    onError: (err: unknown) => toast({ title: '建立失敗', description: getApiErrorMessage(err), variant: 'destructive' }),
  })

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">管理審查</h1>
          <p className="text-muted-foreground">ISO 17025 / ISO 9001 管理審查紀錄</p>
        </div>
        {canManage && (
          <Button onClick={() => setShowCreate(true)}>
            <Plus className="mr-2 h-4 w-4" />
            新增審查
          </Button>
        )}
      </div>

      <Card>
        <CardHeader>
          <div className="flex gap-4">
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
                <TableHead>審查編號</TableHead>
                <TableHead>標題</TableHead>
                <TableHead>審查日期</TableHead>
                <TableHead>狀態</TableHead>
                <TableHead>主持人</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={5} className="p-0"><TableSkeleton rows={5} cols={5} /></TableCell></TableRow>
              ) : reviews.length === 0 ? (
                <TableEmptyRow colSpan={5} icon={ClipboardList} title="尚無審查紀錄" />
              ) : (
                reviews.map((r) => (
                  <TableRow key={r.id}>
                    <TableCell className="font-mono text-sm">{r.review_number}</TableCell>
                    <TableCell className="font-medium">{r.title}</TableCell>
                    <TableCell>{r.review_date}</TableCell>
                    <TableCell>
                      <Badge className={STATUS_COLORS[r.status] ?? ''}>
                        {STATUS_OPTIONS.find((s) => s.value === r.status)?.label ?? r.status}
                      </Badge>
                    </TableCell>
                    <TableCell>{r.chaired_by ?? '-'}</TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Dialog open={showCreate} onOpenChange={setShowCreate}>
        <DialogContent>
          <DialogHeader><DialogTitle>新增管理審查</DialogTitle></DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">標題 *</label>
              <Input value={form.title} onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">審查日期 *</label>
              <Input type="date" value={form.review_date} onChange={(e) => setForm((f) => ({ ...f, review_date: e.target.value }))} />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">議程</label>
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                value={form.agenda}
                onChange={(e) => setForm((f) => ({ ...f, agenda: e.target.value }))}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>取消</Button>
            <Button onClick={() => createMutation.mutate()} disabled={!form.title || !form.review_date || createMutation.isPending}>
              建立
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
