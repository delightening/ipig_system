/**
 * 人員訓練紀錄管理頁 — GLP 合規
 *
 * 版面參考「特休額度管理」：
 * - 統計卡片（人員數、訓練紀錄數、證照即將到期）
 * - Tab 分頁（員工訓練紀錄、證照即將到期）
 * - Card 內搜尋 + 員工標籤選擇
 */

import { useState, useMemo } from 'react'
import { useTabState } from '@/hooks/useTabState'
import { useDialogSet } from '@/hooks/useDialogSet'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
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
import {
  Plus,
  Pencil,
  Trash2,
  Loader2,
  GraduationCap,
  Search,
  User,
  AlertTriangle,
  RefreshCw,
} from 'lucide-react'
import { format, addDays, isBefore, isAfter } from 'date-fns'
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

const EXPIRING_DAYS = 30

export function TrainingRecordsPage() {
  const queryClient = useQueryClient()
  const { hasPermission, user } = useAuthStore()
  const canManage = hasPermission('training.manage') || hasPermission('training.manage_own')
  const canManageAll = hasPermission('training.manage') // 可管理所有人紀錄（admin_staff 審批用）
  const { activeTab, setActiveTab } = useTabState<'records' | 'stats'>('records')
  const dialogs = useDialogSet(['create', 'edit'] as const)
  const [selectedUserId, setSelectedUserId] = useState<string>('')
  const [searchQuery, setSearchQuery] = useState('')
  const [keyword, setKeyword] = useState('')
  const [page, setPage] = useState(1)
  const [editingRecord, setEditingRecord] = useState<TrainingRecordWithUser | null>(null)
  const [form, setForm] = useState({
    user_id: '',
    course_name: '',
    completed_at: format(new Date(), 'yyyy-MM-dd'),
    expires_at: '',
    notes: '',
  })

  // 人員列表（僅 staff，排除 admin；用於統計、員工標籤、新增表單）
  const { data: users = [] } = useQuery({
    queryKey: ['internal-users-for-training'],
    queryFn: async () => {
      const res = await api.get<{ id: string; display_name: string; email: string }[]>(
        '/hr/internal-users'
      )
      return res.data
    },
  })

  // 訓練紀錄（依選擇員工、課程關鍵字篩選）
  const { data, isLoading } = useQuery({
    queryKey: ['training-records', keyword, selectedUserId || undefined, page],
    queryFn: async () => {
      const params: Record<string, string | number> = { page, per_page: 20 }
      if (keyword) params.course_name = keyword
      if (selectedUserId) params.user_id = selectedUserId
      const res = await api.get<PaginatedResponse<TrainingRecordWithUser>>('/training-records', {
        params,
      })
      return res.data
    },
  })

  // 用於統計：取得全部紀錄（含證照即將到期數量）
  const { data: allRecordsData } = useQuery({
    queryKey: ['training-records-stats'],
    queryFn: async () => {
      const res = await api.get<PaginatedResponse<TrainingRecordWithUser>>('/training-records', {
        params: { page: 1, per_page: 500 },
      })
      return res.data
    },
  })

  const expiringSoonCount = useMemo(() => {
    if (!allRecordsData?.data) return 0
    const now = new Date()
    const threshold = addDays(now, EXPIRING_DAYS)
    return allRecordsData.data.filter(
      (r) =>
        r.expires_at &&
        isBefore(new Date(r.expires_at), threshold) &&
        isAfter(new Date(r.expires_at), now)
    ).length
  }, [allRecordsData])

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
      dialogs.close('create')
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
      dialogs.close('edit')
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
    dialogs.open('edit')
  }

  const handleCreate = () => {
    const userId = canManageAll ? form.user_id : user?.id
    if (!userId || !form.course_name.trim() || !form.completed_at) {
      toast({ title: '錯誤', description: '請填寫必填欄位（課程名稱、完成日期）', variant: 'destructive' })
      return
    }
    createMutation.mutate({ ...form, user_id: userId })
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
  const totalRecords = allRecordsData?.total ?? data?.total ?? 0

  // 過濾員工列表（供標籤選擇）
  const filteredUsers = users.filter(
    (u) =>
      (u.display_name || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
      (u.email || '').toLowerCase().includes(searchQuery.toLowerCase())
  )

  // 證照即將到期列表（從 stats 資料篩選）
  const expiringSoonRecords = useMemo(() => {
    if (!allRecordsData?.data) return []
    const now = new Date()
    const threshold = addDays(now, EXPIRING_DAYS)
    return allRecordsData.data
      .filter(
        (r) =>
          r.expires_at &&
          isBefore(new Date(r.expires_at), threshold) &&
          isAfter(new Date(r.expires_at), now)
      )
      .sort((a, b) => (a.expires_at && b.expires_at ? new Date(a.expires_at).getTime() - new Date(b.expires_at).getTime() : 0))
  }, [allRecordsData])

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">
            人員訓練紀錄
          </h1>
          <p className="text-muted-foreground">GLP 合規：管理人員訓練與證照有效期限</p>
        </div>
        {canManage && (
          <Button onClick={() => {
            resetForm()
            if (!canManageAll && user?.id) setForm(f => ({ ...f, user_id: user.id }))
            dialogs.open('create')
          }}>
            <Plus className="h-4 w-4 mr-2" />
            新增訓練紀錄
          </Button>
        )}
      </div>

      {/* 統計卡片（不含人員數：admin 不在訓練範圍，僅 staff） */}
      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">訓練紀錄數</CardTitle>
            <GraduationCap className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{totalRecords}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium">證照即將到期</CardTitle>
            <AlertTriangle className="h-4 w-4 text-orange-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-orange-500">{expiringSoonCount}</div>
          </CardContent>
        </Card>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="records" className="flex items-center gap-2">
            <User className="h-4 w-4" />
            員工訓練紀錄
          </TabsTrigger>
          <TabsTrigger value="expiring" className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4" />
            證照即將到期
            {expiringSoonCount > 0 && (
              <Badge variant="destructive" className="ml-1">
                {expiringSoonCount}
              </Badge>
            )}
          </TabsTrigger>
        </TabsList>

        {/* 員工訓練紀錄 Tab */}
        <TabsContent value="records" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>{canManageAll ? '選擇員工查看訓練紀錄' : '我的訓練紀錄'}</CardTitle>
              <CardDescription>
                {canManageAll ? '選擇員工以查看其訓練課程紀錄與證照有效期限' : '查看與管理您的訓練課程紀錄與證照有效期限'}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* 僅管理全部者可篩選員工 */}
              {canManageAll && (
                <>
                  <div className="relative">
                    <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                    <Input
                      placeholder="搜尋員工姓名或 Email..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="pl-8"
                    />
                  </div>
                  <div className="flex flex-wrap gap-2 max-h-48 overflow-y-auto">
                    <Button
                      variant={!selectedUserId ? 'default' : 'outline'}
                      size="sm"
                      onClick={() => setSelectedUserId('')}
                      className="justify-start"
                    >
                      全部人員
                    </Button>
                    {filteredUsers.map((u) => (
                      <Button
                        key={u.id}
                        variant={selectedUserId === u.id ? 'default' : 'outline'}
                        size="sm"
                        onClick={() => setSelectedUserId(u.id)}
                        className="justify-start"
                      >
                        <User className="h-4 w-4 mr-2" />
                        {u.display_name || u.email}
                      </Button>
                    ))}
                  </div>
                </>
              )}

              {/* 課程名稱篩選 */}
              <Input
                placeholder="搜尋課程名稱..."
                value={keyword}
                onChange={(e) => setKeyword(e.target.value)}
                className="max-w-sm"
              />

              {/* 訓練紀錄表格 */}
              <div className="mt-4">
                {isLoading ? (
                  <div className="flex justify-center py-8">
                    <RefreshCw className="h-6 w-6 animate-spin" />
                  </div>
                ) : records.length === 0 ? (
                  <div className="py-12 text-center text-muted-foreground">尚無訓練紀錄</div>
                ) : (
                  <>
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
                    {totalPages > 1 && (
                      <div className="flex justify-center gap-2 mt-4">
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
                  </>
                )}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* 證照即將到期 Tab */}
        <TabsContent value="expiring" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>證照即將到期</CardTitle>
              <CardDescription>
                列出 30 天內到期之證照，請盡快安排複訓或更新
              </CardDescription>
            </CardHeader>
            <CardContent>
              {expiringSoonRecords.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  目前無即將到期之證照
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>人員</TableHead>
                      <TableHead>課程名稱</TableHead>
                      <TableHead>完成日期</TableHead>
                      <TableHead className="text-orange-500">到期日</TableHead>
                      <TableHead>備註</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {expiringSoonRecords.map((r) => (
                      <TableRow key={r.id}>
                        <TableCell>
                          <div className="font-medium">{r.user_name || r.user_email}</div>
                          <div className="text-xs text-muted-foreground">{r.user_email}</div>
                        </TableCell>
                        <TableCell>{r.course_name}</TableCell>
                        <TableCell>{format(new Date(r.completed_at), 'yyyy/MM/dd', { locale: zhTW })}</TableCell>
                        <TableCell className="font-bold text-orange-500">
                          {r.expires_at
                            ? format(new Date(r.expires_at), 'yyyy/MM/dd', { locale: zhTW })
                            : '—'}
                        </TableCell>
                        <TableCell className="max-w-[200px] truncate">{r.notes || '—'}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* 新增 Dialog */}
      <Dialog open={dialogs.isOpen('create')} onOpenChange={(open) => dialogs.setOpen('create')(open)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新增訓練紀錄</DialogTitle>
            <DialogDescription>填寫課程名稱、完成日期與有效期限（選填）</DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            {canManageAll ? (
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
            ) : (
              <div className="text-sm text-muted-foreground">人員：{user?.display_name || user?.email}</div>
            )}
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
            <Button variant="outline" onClick={() => dialogs.close('create')}>
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
      <Dialog open={dialogs.isOpen('edit')} onOpenChange={(open) => { if (!open) setEditingRecord(null); dialogs.setOpen('edit')(open) }}>
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
            <Button variant="outline" onClick={() => dialogs.close('edit')}>
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
