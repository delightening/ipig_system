import { useState, useCallback } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'

import api, { deleteResource, Partner } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import {
  PartnerFormData,
  PartnerSubmissionData,
  SupplierCategory,
  EMPTY_FORM,
  isValidSupplierCategory,
} from '../constants'

export function usePartnerForm(closeDialog: () => void) {
  const queryClient = useQueryClient()
  const [editingPartner, setEditingPartner] = useState<Partner | null>(null)
  const [formData, setFormData] = useState<PartnerFormData>({ ...EMPTY_FORM })
  const resetForm = () => {
    setFormData({ ...EMPTY_FORM })
    setEditingPartner(null)
  }

  const generateCodeMutation = useMutation({
    mutationFn: async ({ type, category }: { type: 'supplier' | 'customer'; category?: string }) => {
      let url = `/partners/generate-code?partner_type=${type}`
      if (category) url += `&category=${category}`
      const response = await api.get<{ code: string }>(url)
      return response.data.code
    },
    onSuccess: (code) => {
      setFormData(prev => ({ ...prev, code }))
    },
    onError: (error: unknown) => {
      toast({
        title: '生成代碼失敗',
        description: getApiErrorMessage(error, '請重新整理頁面或聯繫管理員'),
        variant: 'destructive',
      })
    },
  })

  const generateCode = useCallback((type: 'supplier' | 'customer', category?: string) => {
    if (editingPartner) return
    generateCodeMutation.mutate({ type, category })
  }, [editingPartner, generateCodeMutation])

  const handleSupplierCategoryChange = (category: SupplierCategory) => {
    setFormData(prev => ({ ...prev, supplier_category: category, code: '' }))
    generateCode('supplier', category)
  }

  const handlePartnerTypeChange = (value: 'supplier' | 'customer') => {
    setFormData(prev => ({
      ...prev,
      partner_type: value,
      supplier_category: '',
      customer_category: '',
      code: '',
    }))
    if (value === 'customer') {
      generateCode('customer')
    }
  }

  const onMutationSuccess = (message: string) => {
    queryClient.invalidateQueries({ queryKey: ['partners'] })
    toast({ title: '成功', description: message })
    closeDialog()
    resetForm()
  }

  const createMutation = useMutation({
    mutationFn: (data: PartnerSubmissionData) => api.post('/partners', data),
    onSuccess: () => onMutationSuccess('夥伴已建立'),
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '建立失敗'),
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: PartnerSubmissionData }) =>
      api.put(`/partners/${id}`, data),
    onSuccess: () => onMutationSuccess('夥伴已更新'),
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '更新失敗'),
        variant: 'destructive',
      })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: ({ id, hard }: { id: string; hard: boolean }) =>
      deleteResource(`/partners/${id}`, { data: { hard } }),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['partners'] })
      toast({
        title: '成功',
        description: variables.hard ? '夥伴已永久刪除' : '夥伴已刪除',
      })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '刪除失敗'),
        variant: 'destructive',
      })
    },
  })

  const handleEdit = (partner: Partner, openDialog: () => void) => {
    setEditingPartner(partner)
    const raw = (partner as Partner & { supplier_category?: string }).supplier_category || ''
    setFormData({
      partner_type: partner.partner_type,
      supplier_category: isValidSupplierCategory(raw) ? raw : '',
      customer_category: partner.customer_category || '',
      code: partner.code,
      name: partner.name,
      tax_id: partner.tax_id || '',
      phone: partner.phone || '',
      phone_ext: partner.phone_ext || '',
      email: partner.email || '',
      address: partner.address || '',
    })
    openDialog()
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()

    if (formData.tax_id.trim() && !/^\d{8}$/.test(formData.tax_id.trim())) {
      toast({ title: '格式錯誤', description: '統編必須為 8 碼數字', variant: 'destructive' })
      return
    }
    if (formData.phone.trim() && !/^\d{9,10}$/.test(formData.phone.trim())) {
      toast({ title: '格式錯誤', description: '電話必須為 9 或 10 碼數字', variant: 'destructive' })
      return
    }
    if (formData.email.trim() && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email.trim())) {
      toast({ title: '格式錯誤', description: 'Email 格式不正確', variant: 'destructive' })
      return
    }

    const submitData: PartnerSubmissionData = {
      ...formData,
      code: formData.code.trim() || null,
      supplier_category: formData.supplier_category || null,
      customer_category: formData.customer_category || null,
      email: formData.email.trim() || null,
      phone: formData.phone.trim() || null,
      phone_ext: formData.phone_ext.trim() || null,
      tax_id: formData.tax_id.trim() || null,
      address: formData.address.trim() || null,
    }

    if (editingPartner) {
      updateMutation.mutate({ id: editingPartner.id, data: submitData })
    } else {
      createMutation.mutate(submitData)
    }
  }

  return {
    formData,
    setFormData,
    editingPartner,
    isGeneratingCode: generateCodeMutation.isPending,
    isPending: createMutation.isPending || updateMutation.isPending,
    resetForm,
    handlePartnerTypeChange,
    handleSupplierCategoryChange,
    handleEdit,
    handleSubmit,
    deleteMutation,
  }
}
