import { useState, useEffect, useCallback } from 'react'

export interface LeaveRequestFormData {
  leaveType: string
  startDate: string
  endDate: string
  totalDays: string
  reason: string
  proxyUserId: string
  supportingImages: string[]
}

const initialForm: LeaveRequestFormData = {
  leaveType: '',
  startDate: '',
  endDate: '',
  totalDays: '1',
  reason: '',
  proxyUserId: '',
  supportingImages: [],
}

export function useLeaveRequestForm() {
  const [form, setForm] = useState<LeaveRequestFormData>(initialForm)
  const [uploadingImage, setUploadingImage] = useState(false)
  const [lastChangedField, setLastChangedField] = useState<
    'startDate' | 'endDate' | 'totalDays' | null
  >(null)

  const updateField = useCallback(<K extends keyof LeaveRequestFormData>(
    key: K,
    value: LeaveRequestFormData[K]
  ) => {
    setForm((prev) => ({ ...prev, [key]: value }))
  }, [])

  const handleStartDateChange = useCallback((value: string) => {
    setForm((prev) => ({ ...prev, startDate: value }))
    setLastChangedField('startDate')
  }, [])

  const handleEndDateChange = useCallback((value: string) => {
    setForm((prev) => ({ ...prev, endDate: value }))
    setLastChangedField('endDate')
  }, [])

  const handleTotalDaysChange = useCallback((value: string) => {
    setForm((prev) => ({ ...prev, totalDays: value }))
    setLastChangedField('totalDays')
  }, [])

  // 自動計算日期/天數（雙向計算）
  useEffect(() => {
    if (!lastChangedField) return
    const { startDate, endDate, totalDays } = form

    if ((lastChangedField === 'startDate' || lastChangedField === 'endDate') && startDate && endDate) {
      const start = new Date(startDate)
      const end = new Date(endDate)
      if (end >= start) {
        const diffTime = end.getTime() - start.getTime()
        const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24)) + 1
        setForm((prev) => ({ ...prev, totalDays: String(diffDays) }))
      }
    }

    if (lastChangedField === 'totalDays' && startDate && totalDays) {
      const days = parseFloat(totalDays)
      if (days >= 0.5) {
        const start = new Date(startDate)
        const end = new Date(start.getTime() + (Math.ceil(days) - 1) * 24 * 60 * 60 * 1000)
        const newEndDate = end.toISOString().split('T')[0]
        if (newEndDate !== endDate) {
          setForm((prev) => ({ ...prev, endDate: newEndDate }))
        }
      }
    }
  }, [form.startDate, form.endDate, form.totalDays, lastChangedField])

  const resetForm = useCallback(() => {
    setForm(initialForm)
    setLastChangedField(null)
  }, [])

  const addSupportingImages = useCallback((urls: string[]) => {
    setForm((prev) => ({
      ...prev,
      supportingImages: [...prev.supportingImages, ...urls],
    }))
  }, [])

  const removeSupportingImage = useCallback((index: number) => {
    setForm((prev) => ({
      ...prev,
      supportingImages: prev.supportingImages.filter((_, i) => i !== index),
    }))
  }, [])

  const isAnnualLeave = form.leaveType === 'ANNUAL'

  const buildSubmitPayload = useCallback(() => ({
    leave_type: form.leaveType,
    start_date: form.startDate,
    end_date: form.endDate,
    total_days: parseFloat(form.totalDays),
    reason: form.reason.trim() || undefined,
    supporting_documents:
      form.supportingImages.length > 0 ? form.supportingImages : undefined,
    proxy_user_id:
      form.proxyUserId && form.proxyUserId !== '__none__' ? form.proxyUserId : undefined,
  }), [form])

  return {
    form,
    setForm,
    updateField,
    uploadingImage,
    setUploadingImage,
    handleStartDateChange,
    handleEndDateChange,
    handleTotalDaysChange,
    resetForm,
    addSupportingImages,
    removeSupportingImage,
    isAnnualLeave,
    buildSubmitPayload,
  }
}
