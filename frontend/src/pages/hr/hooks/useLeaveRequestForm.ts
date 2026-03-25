import { useEffect, useCallback, useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import {
  leaveRequestSchema,
  type LeaveRequestFormData,
} from '@/lib/validation'

/** 以 0.5 小時為單位四捨五入 */
export const roundToHalfHour = (value: number): number =>
  Math.round(value * 2) / 2

const HOURS_PER_DAY = 8

export function useLeaveRequestForm() {
  const form = useForm<LeaveRequestFormData>({
    resolver: zodResolver(leaveRequestSchema),
    defaultValues: {
      leaveType: '',
      startDate: '',
      endDate: '',
      totalHours: '8',
      reason: '',
      proxyUserId: '',
      supportingImages: [],
    },
  })

  const [uploadingImage, setUploadingImage] = useState(false)
  const [lastChangedField, setLastChangedField] = useState<
    'startDate' | 'endDate' | 'totalHours' | null
  >(null)

  const { watch, setValue, getValues, reset: resetRHF } = form

  const startDate = watch('startDate')
  const endDate = watch('endDate')
  const totalHours = watch('totalHours')
  const leaveType = watch('leaveType')
  const proxyUserId = watch('proxyUserId')
  const reason = watch('reason')
  const supportingImages = watch('supportingImages')

  const handleStartDateChange = useCallback((value: string) => {
    setValue('startDate', value)
    setLastChangedField('startDate')
  }, [setValue])

  const handleEndDateChange = useCallback((value: string) => {
    setValue('endDate', value)
    setLastChangedField('endDate')
  }, [setValue])

  const handleTotalHoursChange = useCallback((value: string) => {
    setValue('totalHours', value)
    setLastChangedField('totalHours')
  }, [setValue])

  // 自動計算日期/時數（雙向計算，以 0.5 小時為單位）
  useEffect(() => {
    if (!lastChangedField) return

    if ((lastChangedField === 'startDate' || lastChangedField === 'endDate') && startDate && endDate) {
      const start = new Date(startDate)
      const end = new Date(endDate)
      if (end >= start) {
        const diffTime = end.getTime() - start.getTime()
        const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24)) + 1
        const hours = roundToHalfHour(diffDays * HOURS_PER_DAY)
        setValue('totalHours', String(hours))
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
          setValue('endDate', newEndDate)
        }
      }
    }
  }, [startDate, endDate, totalHours, lastChangedField, setValue])

  const resetForm = useCallback(() => {
    resetRHF()
    setLastChangedField(null)
  }, [resetRHF])

  const addSupportingImages = useCallback((urls: string[]) => {
    const current = getValues('supportingImages') || []
    setValue('supportingImages', [...current, ...urls])
  }, [getValues, setValue])

  const removeSupportingImage = useCallback((index: number) => {
    const current = getValues('supportingImages') || []
    setValue('supportingImages', current.filter((_, i) => i !== index))
  }, [getValues, setValue])

  const isAnnualLeave = leaveType === 'ANNUAL'

  const buildSubmitPayload = useCallback(() => {
    const values = getValues()
    const hours = roundToHalfHour(parseFloat(values.totalHours) || 0)
    return {
      leave_type: values.leaveType,
      start_date: values.startDate,
      end_date: values.endDate,
      total_hours: hours,
      total_days: hours / HOURS_PER_DAY,
      reason: values.reason.trim() || undefined,
      supporting_documents:
        values.supportingImages.length > 0 ? values.supportingImages : undefined,
      proxy_user_id:
        values.proxyUserId && values.proxyUserId !== '__none__' ? values.proxyUserId : undefined,
    }
  }, [getValues])

  return {
    form: { leaveType, startDate, endDate, totalHours, proxyUserId, reason, supportingImages },
    rhf: form,
    updateField: <K extends keyof LeaveRequestFormData>(key: K, value: LeaveRequestFormData[K]) => {
      setValue(key, value as any)
    },
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
