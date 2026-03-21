import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { format, addDays, isBefore, isAfter } from 'date-fns'

import api, { deleteResource } from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { useDialogSet } from '@/hooks/useDialogSet'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import type { PaginatedResponse } from '@/types/common'
import type { TrainingRecordWithUser, TrainingForm } from '../types/training'
import { EXPIRING_DAYS } from '../types/training'

function createEmptyForm(): TrainingForm {
  return {
    user_id: '',
    course_name: '',
    completed_at: format(new Date(), 'yyyy-MM-dd'),
    expires_at: '',
    notes: '',
  }
}

export function useTrainingRecords() {
  const queryClient = useQueryClient()
  const { hasPermission, user } = useAuthStore()
  const canManage = hasPermission('training.manage') || hasPermission('training.manage_own')
  const canManageAll = hasPermission('training.manage')

  const dialogs = useDialogSet(['create', 'edit'] as const)
  const [selectedUserId, setSelectedUserId] = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [keyword, setKeyword] = useState('')
  const [page, setPage] = useState(1)
  const [editingRecord, setEditingRecord] = useState<TrainingRecordWithUser | null>(null)
  const [form, setForm] = useState<TrainingForm>(createEmptyForm)

  // 人員列表
  const { data: users = [] } = useQuery({
    queryKey: ['internal-users-for-training'],
    queryFn: async () => {
      const res = await api.get<{ id: string; display_name: string; email: string }[]>(
        '/hr/internal-users'
      )
      return res.data
    },
  })

  // 訓練紀錄（分頁、篩選）
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

  // 統計用：取全部紀錄
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
      .sort((a, b) =>
        a.expires_at && b.expires_at
          ? new Date(a.expires_at).getTime() - new Date(b.expires_at).getTime()
          : 0
      )
  }, [allRecordsData])

  const resetForm = () => setForm(createEmptyForm())

  const createMutation = useMutation({
    mutationFn: (payload: TrainingForm) =>
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
      toast({ title: '錯誤', description: getApiErrorMessage(err, '新增失敗'), variant: 'destructive' })
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
      toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteResource(`/training-records/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['training-records'] })
      toast({ title: '成功', description: '已刪除訓練紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '刪除失敗'), variant: 'destructive' })
    },
  })

  const openCreateDialog = () => {
    resetForm()
    if (!canManageAll && user?.id) setForm((f) => ({ ...f, user_id: user.id }))
    dialogs.open('create')
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

  const filteredUsers = users.filter(
    (u) =>
      (u.display_name || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
      (u.email || '').toLowerCase().includes(searchQuery.toLowerCase())
  )

  return {
    // permissions
    canManage,
    canManageAll,
    user,
    // dialogs
    dialogs,
    // data
    users,
    filteredUsers,
    records,
    totalPages,
    totalRecords,
    isLoading,
    expiringSoonCount,
    expiringSoonRecords,
    // form
    form,
    setForm,
    editingRecord,
    // filters
    selectedUserId,
    setSelectedUserId,
    searchQuery,
    setSearchQuery,
    keyword,
    setKeyword,
    page,
    setPage,
    // actions
    openCreateDialog,
    openEditDialog,
    handleCreate,
    handleUpdate,
    handleDelete,
    createMutation,
    updateMutation,
  }
}
