import { useState, useEffect, useCallback } from 'react'

/** 以 0.5 小時為單位四捨五入 */
export const roundToHalfHour = (value: number): number =>
  Math.round(value * 2) / 2

const HOURS_PER_DAY = 8

export interface LeaveRequestFormData {
  leaveType: string
  startDate: string
  endDate: string
  totalHours: string
  reason: string
  proxyUserId: string
  supportingImages: string[]
}

const initialForm: LeaveRequestFormData = {
  leaveType: '',
  startDate: '',
  endDate: '',
  totalHours: '8',
  reason: '',
  proxyUserId: '',
  supportingImages: [],
}

export function useLeaveRequestForm() {
  const [form, setForm] = useState<LeaveRequestFormData>(initialForm)
  const [uploadingImage, setUploadingImage] = useState(false)
  const [lastChangedField, setLastChangedField] = useState<
    'startDate' | 'endDate' | 'totalHours' | null
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

  const handleTotalHoursChange = useCallback((value: string) => {
    setForm((prev) => ({ ...prev, totalHours: value }))
    setLastChangedField('totalHours')
  }, [])

  // 自動計算日期/時數（雙向計算，以 0.5 小時為單位）
  useEffect(() => {
    if (!lastChangedField) return
    const { startDate, endDate, totalHours } = form

    if ((lastChangedField === 'startDate' || lastChangedField === 'endDate') && startDate && endDate) {
      const start = new Date(startDate)
      const end = new Date(endDate)
      if (end >= start) {
        const diffTime = end.getTime() - start.getTime()
        const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24)) + 1
        const hours = roundToHalfHour(diffDays * HOURS_PER_DAY)
        setForm((prev) => ({ ...prev, totalHours: String(hours) }))
      }
    }

    if (lastChangedField === 'totalHours' && startDate && totalHours) {
      const hours = parseFloat(totalHours)
      if (hours >= 0.5) {
        const days = Math.ceil(hours / HOURS_PER_DAY)
        const start = new Date(startDate)
        const end = new Date(start.getTime() + (days - 1) * 24 * 60 * 60 * 1000)
        const newEndDate = end.toISOString().split('T')[0]
        if (newEndDate !== endDate) {
          setForm((prev) => ({ ...prev, endDate: newEndDate }))
        }
      }
    }
  }, [form.startDate, form.endDate, form.totalHours, lastChangedField])

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

  const buildSubmitPayload = useCallback(() => {
    const totalHours = roundToHalfHour(parseFloat(form.totalHours) || 0)
    return {
      leave_type: form.leaveType,
      start_date: form.startDate,
      end_date: form.endDate,
      total_hours: totalHours,
      total_days: totalHours / HOURS_PER_DAY,
      reason: form.reason.trim() || undefined,
      supporting_documents:
        form.supportingImages.length > 0 ? form.supportingImages : undefined,
      proxy_user_id:
        form.proxyUserId && form.proxyUserId !== '__none__' ? form.proxyUserId : undefined,
    }
  }, [form])

  return {
    form,
    setForm,
    updateField,
    uploadingImage,
    setUploadingImage,
    handleStartDateChange,
    handleEndDateChange,
    handleTotalHoursChange,
    resetForm,
    addSupportingImages,
    removeSupportingImage,
    isAnnualLeave,
    buildSubmitPayload,
  }
}
