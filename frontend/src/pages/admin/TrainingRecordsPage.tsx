/**
 * 人員訓練紀錄管理頁 — GLP 合規
 *
 * 功能：
 * - 列出訓練紀錄（依使用者、課程名稱篩選）
 * - 新增、編輯、刪除訓練紀錄
 */

import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Plus, Pencil, Trash2, Loader2, GraduationCap } from 'lucide-react'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'

interface TrainingRecordWithUser {
  id: string
  user_id: string
  user_email: string
  user_name: string
  course_name: string
  completed_at: string
  expires_at: string | null
  notes: string | null
  created_at: string
}

interface PaginatedResponse<T> {
  data: T[]
  total: number
  page: number
  per_page: number
  total_pages: number
}

interface User {
  id: string
  email: string
  display_name: string
}

export function TrainingRecordsPage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('training.manage')
  const [keyword, setKeyword] = useState('')
  const [userFilter, setUserFilter] = useState<string>('')
  const [page, setPage] = useState(1)
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [showEditDialog, setShowEditDialog] = useState(false)
  const [editingRecord, setEditingRecord] = useState<TrainingRecordWithUser | null>(null)
  const [form, setForm] = useState({
    user_id: '',
    course_name: '',
    completed_at: format(new Date(), 'yyyy-MM-dd'),
    expires_at: '',
    notes: '',
  })

  const { data: users = [] } = useQuery({
    queryKey: ['users-for-training'],
    queryFn: async () => {
      const res = await api.get<User[]>('/users', { params: { per_page: 500 } })
      return res.data
    },
    enabled: showCreateDialog, // 僅在新增表單開啟時載入
  })

  const { data, isLoading } = useQuery({
    queryKey: ['training-records', keyword, userFilter || undefined, page],
    queryFn: async () => {
      const params: Record<string, string | number> = { page, per_page: 20 }
      if (keyword) params.course_name = keyword
      if (userFilter) params.user_id = userFilter
      const res = await api.get<PaginatedResponse<TrainingRecordWithUser>>('/training-records', {
        params,
      })
      return res.data
    },
  })

  const createMutation = useMutation({
    mutationFn: (payload: typeof form) =>
      api.post('/training-records', {
        user_id: payload.user_id,
        course_name: payload.course_name,
        completed_at: payload.completed_at,
        expires_at: payload.expires_at || null,
        notes: payload.notes || null,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['training-records'] })
      setShowCreateDialog(false)
      resetForm()
      toast({ title: '成功', description: '已新增訓練紀錄' })
    },
    onError: (err: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(err, '新增失敗'),
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: Record<string, unknown> }) =>
      api.put(`/training-records/${id}`, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['training-records'] })
      setShowEditDialog(false)
      setEditingRecord(null)
      toast({ title: '成功', description: '已更新訓練紀錄' })
    },
    onError: (err: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(err, '更新失敗'),
        variant: 'destructive',
      })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/training-records/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['training-records'] })
      toast({ title: '成功', description: '已刪除訓練紀錄' })
    },
    onError: (err: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(err, '刪除失敗'),
        variant: 'destructive',
      })
    },
  })

  const resetForm = () => {
    setForm({
      user_id: '',
      course_name: '',
      completed_at: format(new Date(), 'yyyy-MM-dd'),
      expires_at: '',
      notes: '',
    })
  }

  const openEditDialog = (record: TrainingRecordWithUser) => {
    setEditingRecord(record)
    setForm({
      user_id: record.user_id,
      course_name: record.course_name,
      completed_at: record.completed_at,
      expires_at: record.expires_at || '',
      notes: record.notes || '',
    })
    setShowEditDialog(true)
  }

  const handleCreate = () => {
    if (!form.user_id || !form.course_name.trim() || !form.completed_at) {
      toast({ title: '錯誤', description: '請填寫必填欄位（人員、課程名稱、完成日期）', variant: 'destructive' })
      return
    }
    createMutation.mutate(form)
  }

  const handleUpdate = () => {
    if (!editingRecord) return
    if (!form.course_name.trim() || !form.completed_at) {
      toast({ title: '錯誤', description: '請填寫必填欄位', variant: 'destructive' })
      return
    }
    updateMutation.mutate({
      id: editingRecord.id,
      payload: {
        course_name: form.course_name,
        completed_at: form.completed_at,
        expires_at: form.expires_at || null,
        notes: form.notes || null,
      },
    })
  }

  const handleDelete = (record: TrainingRecordWithUser) => {
    if (window.confirm(`確定要刪除「${record.course_name}」的訓練紀錄嗎？`)) {
      deleteMutation.mutate(record.id)
    }
  }

  const records = data?.data ?? []
  const totalPages = data?.total_pages ?? 1

  return (
    <div className="space-y-6 p-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <GraduationCap className="h-7 w-7" />
            人員訓練紀錄
          </h1>
          <p className="text-muted-foreground text-sm mt-1">GLP 合規：管理人員訓練與證照有效期限</p>
        </div>
        {canManage && (
          <Button onClick={() => { resetForm(); setShowCreateDialog(true) }}>
            <Plus className="h-4 w-4 mr-2" />
            新增紀錄
          </Button>
        )}
      </div>

      <div className="flex flex-wrap gap-4">
        <div className="flex-1 min-w-[200px]">
          <Input
            placeholder="搜尋課程名稱..."
            value={keyword}
            onChange={(e) => setKeyword(e.target.value)}
            className="max-w-sm"
          />
        </div>
        <Select value={userFilter} onValueChange={setUserFilter}>
          <SelectTrigger className="w-[220px]">
            <SelectValue placeholder="篩選操作者" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="">全部人員</SelectItem>
            {users.map((u) => (
              <SelectItem key={u.id} value={u.id}>
                {u.display_name || u.email}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div className="rounded-md border">
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : records.length === 0 ? (
          <div className="py-12 text-center text-muted-foreground">尚無訓練紀錄</div>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>人員</TableHead>
                <TableHead>課程名稱</TableHead>
                <TableHead>完成日期</TableHead>
                <TableHead>有效期限</TableHead>
                <TableHead>備註</TableHead>
                <TableHead className="w-[100px]">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {records.map((r) => (
                <TableRow key={r.id}>
                  <TableCell>
                    <div className="font-medium">{r.user_name || r.user_email}</div>
                    <div className="text-xs text-muted-foreground">{r.user_email}</div>
                  </TableCell>
                  <TableCell>{r.course_name}</TableCell>
                  <TableCell>{format(new Date(r.completed_at), 'yyyy/MM/dd', { locale: zhTW })}</TableCell>
                  <TableCell>
                    {r.expires_at
                      ? format(new Date(r.expires_at), 'yyyy/MM/dd', { locale: zhTW })
                      : '—'}
                  </TableCell>
                  <TableCell className="max-w-[200px] truncate">{r.notes || '—'}</TableCell>
                  <TableCell>
                    {canManage && (
                      <div className="flex gap-2">
                        <Button variant="ghost" size="icon" onClick={() => openEditDialog(r)}>
                          <Pencil className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleDelete(r)}
                          className="text-destructive hover:text-destructive"
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </div>

      {totalPages > 1 && (
        <div className="flex justify-center gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={page <= 1}
            onClick={() => setPage((p) => p - 1)}
          >
            上一頁
          </Button>
          <span className="flex items-center px-4 text-sm text-muted-foreground">
            第 {page} / {totalPages} 頁
          </span>
          <Button
            variant="outline"
            size="sm"
            disabled={page >= totalPages}
            onClick={() => setPage((p) => p + 1)}
          >
            下一頁
          </Button>
        </div>
      )}

      {/* 新增 Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新增訓練紀錄</DialogTitle>
            <DialogDescription>填寫課程名稱、完成日期與有效期限（選填）</DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div>
              <Label>人員 *</Label>
              <Select value={form.user_id} onValueChange={(v) => setForm({ ...form, user_id: v })}>
                <SelectTrigger>
                  <SelectValue placeholder="選擇人員" />
                </SelectTrigger>
                <SelectContent>
                  {users.map((u) => (
                    <SelectItem key={u.id} value={u.id}>
                      {u.display_name || u.email}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>課程名稱 *</Label>
              <Input
                value={form.course_name}
                onChange={(e) => setForm({ ...form, course_name: e.target.value })}
                placeholder="例：實驗動物從業人員訓練"
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>完成日期 *</Label>
                <Input
                  type="date"
                  value={form.completed_at}
                  onChange={(e) => setForm({ ...form, completed_at: e.target.value })}
                />
              </div>
              <div>
                <Label>有效期限</Label>
                <Input
                  type="date"
                  value={form.expires_at}
                  onChange={(e) => setForm({ ...form, expires_at: e.target.value })}
                  placeholder="選填"
                />
              </div>
            </div>
            <div>
              <Label>備註</Label>
              <Input
                value={form.notes}
                onChange={(e) => setForm({ ...form, notes: e.target.value })}
                placeholder="選填"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
              取消
            </Button>
            <Button onClick={handleCreate} disabled={createMutation.isPending}>
              {createMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              新增
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 編輯 Dialog */}
      <Dialog open={showEditDialog} onOpenChange={(open) => { if (!open) setEditingRecord(null); setShowEditDialog(open) }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>編輯訓練紀錄</DialogTitle>
            <DialogDescription>修改課程名稱、完成日期與有效期限</DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            {editingRecord && (
              <div className="text-sm text-muted-foreground">
                人員：{editingRecord.user_name || editingRecord.user_email}
              </div>
            )}
            <div>
              <Label>課程名稱 *</Label>
              <Input
                value={form.course_name}
                onChange={(e) => setForm({ ...form, course_name: e.target.value })}
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>完成日期 *</Label>
                <Input
                  type="date"
                  value={form.completed_at}
                  onChange={(e) => setForm({ ...form, completed_at: e.target.value })}
                />
              </div>
              <div>
                <Label>有效期限</Label>
                <Input
                  type="date"
                  value={form.expires_at}
                  onChange={(e) => setForm({ ...form, expires_at: e.target.value })}
                />
              </div>
            </div>
            <div>
              <Label>備註</Label>
              <Input
                value={form.notes}
                onChange={(e) => setForm({ ...form, notes: e.target.value })}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowEditDialog(false)}>
              取消
            </Button>
            <Button onClick={handleUpdate} disabled={updateMutation.isPending}>
              {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
