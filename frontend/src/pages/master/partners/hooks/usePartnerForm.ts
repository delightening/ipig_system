import { useCallback, useState } from 'react'
import { useForm, type RegisterOptions, type UseFormRegister } from 'react-hook-form'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api, { deleteResource, Partner } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import {
  PartnerFormData,
  PartnerSubmissionData,
  SupplierCategory,
  EMPTY_FORM,
  isValidSupplierCategory,
} from '../constants'

// R57-2: 改 React Hook Form 原生 validation rules（避開 Zod 4 CSP eval probe）
// 對齊原 partnerFormZodSchema：name 必填、tax_id 8 碼、phone 9-10 碼、email 格式
// ⚠️ 下方 message 是 **i18n 鍵**（不是顯示文字），由 PartnerFormDialog 的 FieldError 在顯示時 t()。
const TAX_ID_PATTERN = /^\d{8}$/
const PHONE_PATTERN = /^\d{9,10}$/
const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

const FIELD_RULES: Partial<Record<keyof PartnerFormData, RegisterOptions<PartnerFormData>>> = {
  name: { required: 'erpMaster.partners.validation.nameRequired' },
  tax_id: {
    validate: (v) =>
      typeof v !== 'string' || v === '' || TAX_ID_PATTERN.test(v) || 'validation.taxId',
  },
  phone: {
    validate: (v) =>
      typeof v !== 'string' || v === '' || PHONE_PATTERN.test(v) || 'erpMaster.partners.validation.phone',
  },
  email: {
    validate: (v) =>
      typeof v !== 'string' || v === '' || EMAIL_PATTERN.test(v) || 'validation.email',
  },
}

export function usePartnerForm(closeDialog: () => void) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [editingPartner, setEditingPartner] = useState<Partner | null>(null)

  const rhf = useForm<PartnerFormData>({
    defaultValues: { ...EMPTY_FORM },
  })

  // 包一層 register，依欄位名稱自動套上 FIELD_RULES（呼叫端不需逐個傳 rules）。
  // 用 `as never` 繞過 RHF generic per-field constraint — runtime 行為等價 baseRegister。
  const baseRegister = rhf.register
  const register: UseFormRegister<PartnerFormData> = (name, options) => {
    const baseRules = FIELD_RULES[name]
    const merged = { ...(baseRules ?? {}), ...(options ?? {}) }
    return baseRegister(name, merged as never)
  }

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
        title: t('erpMaster.partners.toast.generateCodeFailed'),
        description: getApiErrorMessage(error, t('erpMaster.partners.toast.generateCodeFailedDescription')),
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
    toast({ title: t('common.success'), description: message })
    closeDialog()
    resetForm()
  }

  const createMutation = useMutation({
    mutationFn: (data: PartnerSubmissionData) => api.post('/partners', data),
    onSuccess: () => onMutationSuccess(t('erpMaster.partners.toast.created')),
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpMaster.partners.toast.createFailed')),
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: PartnerSubmissionData }) =>
      api.put(`/partners/${id}`, data),
    onSuccess: () => onMutationSuccess(t('erpMaster.partners.toast.updated')),
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpMaster.partners.toast.updateFailed')),
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
        title: t('common.success'),
        description: variables.hard ? t('erpMaster.partners.toast.hardDeleted') : t('erpMaster.partners.toast.deleted'),
      })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpMaster.partners.toast.deleteFailed')),
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
    register,
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
