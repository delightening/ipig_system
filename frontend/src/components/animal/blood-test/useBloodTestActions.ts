/**
 * Blood test CRUD mutations and form handlers
 */
import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import {
  CreateBloodTestRequest,
  UpdateBloodTestRequest,
  bloodTestApi,
} from '@/lib/api'
import { toast } from '@/components/ui/use-toast'

import { LAB_OPTIONS } from './constants'
import type { BloodTestFormData } from './BloodTestFormDialog'

const INITIAL_FORM_DATA: BloodTestFormData = {
  test_date: new Date().toISOString().split('T')[0],
  lab_name: '',
  remark: '',
  items: [],
}

export function useBloodTestActions(animalId: string) {
  const queryClient = useQueryClient()
  const [showFormDialog, setShowFormDialog] = useState(false)
  const [editingId, setEditingId] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; date: string } | null>(null)
  const [labNameOption, setLabNameOption] = useState<string>('')
  const [formData, setFormData] = useState<BloodTestFormData>({ ...INITIAL_FORM_DATA })

  const resetForm = () => {
    setFormData({ ...INITIAL_FORM_DATA, test_date: new Date().toISOString().split('T')[0] })
    setLabNameOption('')
  }

  const createMutation = useMutation({
    mutationFn: (data: CreateBloodTestRequest) => bloodTestApi.create(animalId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-blood-tests', animalId] })
      setShowFormDialog(false)
      resetForm()
      toast({ title: '成功', description: '血液檢查紀錄已建立' })
    },
    onError: () => {
      toast({ title: '錯誤', description: '建立失敗', variant: 'destructive' })
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateBloodTestRequest }) =>
      bloodTestApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-blood-tests', animalId] })
      queryClient.invalidateQueries({ queryKey: ['blood-test-detail'] })
      setShowFormDialog(false)
      setEditingId(null)
      resetForm()
      toast({ title: '成功', description: '血液檢查紀錄已更新' })
    },
    onError: () => {
      toast({ title: '錯誤', description: '更新失敗', variant: 'destructive' })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: ({ id, reason }: { id: string; reason: string }) =>
      bloodTestApi.delete(id, reason),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-blood-tests', animalId] })
      setDeleteTarget(null)
      toast({ title: '成功', description: '血液檢查紀錄已刪除' })
    },
    onError: () => {
      toast({ title: '錯誤', description: '刪除失敗', variant: 'destructive' })
    },
  })

  const openCreateForm = () => {
    resetForm()
    setEditingId(null)
    setShowFormDialog(true)
  }

  const openEditForm = async (id: string) => {
    try {
      const res = await bloodTestApi.getById(id)
      const detail = res.data
      setFormData({
        test_date: detail.blood_test.test_date,
        lab_name: detail.blood_test.lab_name || '',
        remark: detail.blood_test.remark || '',
        items: detail.items.map((item) => ({
          template_id: item.template_id || undefined,
          item_name: item.item_name,
          result_value: item.result_value || '',
          result_unit: item.result_unit || '',
          reference_range: item.reference_range || '',
          is_abnormal: item.is_abnormal,
          remark: item.remark || '',
          sort_order: item.sort_order,
        })),
      })
      const loadedLabName = detail.blood_test.lab_name || ''
      if (LAB_OPTIONS.includes(loadedLabName as typeof LAB_OPTIONS[number])) {
        setLabNameOption(loadedLabName)
      } else if (loadedLabName) {
        setLabNameOption('__other__')
      } else {
        setLabNameOption('')
      }
      setEditingId(id)
      setShowFormDialog(true)
    } catch {
      toast({ title: '錯誤', description: '載入資料失敗', variant: 'destructive' })
    }
  }

  const handleSubmit = () => {
    if (!formData.test_date) {
      toast({ title: '錯誤', description: '請填寫檢查日期', variant: 'destructive' })
      return
    }
    if (formData.items.length === 0) {
      toast({ title: '錯誤', description: '至少需要一個檢查項目', variant: 'destructive' })
      return
    }
    if (formData.items.some((item) => !item.item_name.trim())) {
      toast({ title: '錯誤', description: '所有檢查項目都需要填寫名稱', variant: 'destructive' })
      return
    }

    const payload = {
      test_date: formData.test_date,
      lab_name: formData.lab_name || undefined,
      remark: formData.remark || undefined,
      items: formData.items,
    }

    if (editingId) {
      updateMutation.mutate({ id: editingId, data: payload })
    } else {
      createMutation.mutate(payload)
    }
  }

  const handleFormClose = () => {
    setShowFormDialog(false)
    setEditingId(null)
    resetForm()
  }

  return {
    showFormDialog, setShowFormDialog,
    editingId,
    deleteTarget, setDeleteTarget,
    labNameOption, setLabNameOption,
    formData, setFormData,
    isPending: createMutation.isPending || updateMutation.isPending,
    deleteMutation,
    openCreateForm, openEditForm, handleSubmit, handleFormClose,
  }
}
