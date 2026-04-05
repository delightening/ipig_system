import { useState, useCallback } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQueryClient } from '@tanstack/react-query'

import api, { deleteResource, Partner } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage, partnerFormZodSchema } from '@/lib/validation'
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

  const rhf = useForm<PartnerFormData>({
    resolver: zodResolver(partnerFormZodSchema),
    defaultValues: { ...EMPTY_FORM },
  })

  const { setValue, reset, watch, handleSubmit: rhfHandleSubmit, formState } = rhf

  /** Targeted watches for fields consumed by PartnerFormDialog */
  const partnerType = watch('partner_type')
  const supplierCategory = watch('supplier_category')
  const customerCategory = watch('customer_category')
  const code = watch('code')
  const formData = { partner_type: partnerType, supplier_category: supplierCategory, customer_category: customerCategory, code }

  const resetForm = () => {
    reset({ ...EMPTY_FORM })
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
      setValue('code', code)
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
    setValue('supplier_category', category)
    setValue('code', '')
    generateCode('supplier', category)
  }

  const handlePartnerTypeChange = (value: 'supplier' | 'customer') => {
    setValue('partner_type', value)
    setValue('supplier_category', '')
    setValue('customer_category', '')
    setValue('code', '')
    if (value === 'customer') {
      generateCode('customer')
    }
  }

  const invalidatePartnerRelated = () => {
    queryClient.invalidateQueries({ queryKey: ['partners'] })
    queryClient.invalidateQueries({ queryKey: ['partners-supplier'] })
    queryClient.invalidateQueries({ queryKey: ['partners-customer-list'] })
    queryClient.invalidateQueries({ queryKey: ['equipment-suppliers'] })
    queryClient.invalidateQueries({ queryKey: ['equipment-suppliers-summary'] })
  }

  const onMutationSuccess = (message: string) => {
    invalidatePartnerRelated()
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
      invalidatePartnerRelated()
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
    reset({
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

  const onValid = (data: PartnerFormData) => {
    const submitData: PartnerSubmissionData = {
      ...data,
      code: data.code.trim() || null,
      supplier_category: data.supplier_category || null,
      customer_category: data.customer_category || null,
      email: data.email.trim() || null,
      phone: data.phone.trim() || null,
      phone_ext: data.phone_ext.trim() || null,
      tax_id: data.tax_id.trim() || null,
      address: data.address.trim() || null,
    }

    if (editingPartner) {
      updateMutation.mutate({ id: editingPartner.id, data: submitData })
    } else {
      createMutation.mutate(submitData)
    }
  }

  const handleSubmit = rhfHandleSubmit(onValid)

  return {
    formData,
    register: rhf.register,
    setValue,
    errors: formState.errors,
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
