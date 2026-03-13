/**
 * 設備與校正紀錄的 mutation hooks
 */
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { format } from 'date-fns'

import type { EquipmentForm, CalibrationForm } from '../types'

/** 設備相關 query keys */
const EQUIP_KEYS = {
  list: ['equipment'],
  all: ['equipment-all'],
} as const

/** 校正紀錄相關 query keys */
const CALIB_KEYS = {
  list: ['equipment-calibrations'],
  all: ['equipment-calibrations-all'],
} as const

/** 將空字串轉為 null */
const emptyToNull = (v: string) => v || null

interface UseEquipmentMutationsOptions {
  closeEquipCreate: () => void
  closeEquipEdit: () => void
  closeCalibCreate: () => void
  closeCalibEdit: () => void
  clearEditingEquip: () => void
  clearEditingCalib: () => void
  resetEquipForm: () => void
  resetCalibForm: () => void
}

export function useEquipmentMutations(options: UseEquipmentMutationsOptions) {
  const queryClient = useQueryClient()

  const invalidateEquip = () => {
    queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.list })
    queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.all })
  }

  const invalidateCalib = () => {
    queryClient.invalidateQueries({ queryKey: CALIB_KEYS.list })
    queryClient.invalidateQueries({ queryKey: CALIB_KEYS.all })
  }

  const createEquipMutation = useMutation({
    mutationFn: (payload: EquipmentForm) =>
      api.post('/equipment', {
        name: payload.name,
        model: emptyToNull(payload.model),
        serial_number: emptyToNull(payload.serial_number),
        location: emptyToNull(payload.location),
        notes: emptyToNull(payload.notes),
      }),
    onSuccess: () => {
      invalidateEquip()
      options.closeEquipCreate()
      options.resetEquipForm()
      toast({ title: '成功', description: '已新增設備' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '新增失敗'), variant: 'destructive' })
    },
  })

  const updateEquipMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: Record<string, unknown> }) =>
      api.put(`/equipment/${id}`, payload),
    onSuccess: () => {
      invalidateEquip()
      options.closeEquipEdit()
      options.clearEditingEquip()
      toast({ title: '成功', description: '已更新設備' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
    },
  })

  const deleteEquipMutation = useMutation({
    mutationFn: (id: string) => deleteResource(`/equipment/${id}`),
    onSuccess: () => {
      invalidateEquip()
      invalidateCalib()
      toast({ title: '成功', description: '已刪除設備' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '刪除失敗'), variant: 'destructive' })
    },
  })

  const createCalibMutation = useMutation({
    mutationFn: (payload: CalibrationForm) =>
      api.post('/equipment-calibrations', {
        equipment_id: payload.equipment_id,
        calibrated_at: payload.calibrated_at,
        next_due_at: emptyToNull(payload.next_due_at),
        result: emptyToNull(payload.result),
        notes: emptyToNull(payload.notes),
      }),
    onSuccess: () => {
      invalidateCalib()
      options.closeCalibCreate()
      options.resetCalibForm()
      toast({ title: '成功', description: '已新增校正紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '新增失敗'), variant: 'destructive' })
    },
  })

  const updateCalibMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: Record<string, unknown> }) =>
      api.put(`/equipment-calibrations/${id}`, payload),
    onSuccess: () => {
      invalidateCalib()
      options.closeCalibEdit()
      options.clearEditingCalib()
      toast({ title: '成功', description: '已更新校正紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
    },
  })

  const deleteCalibMutation = useMutation({
    mutationFn: (id: string) => deleteResource(`/equipment-calibrations/${id}`),
    onSuccess: () => {
      invalidateCalib()
      toast({ title: '成功', description: '已刪除校正紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '刪除失敗'), variant: 'destructive' })
    },
  })

  /** 新增設備（含驗證） */
  const handleCreateEquip = (form: EquipmentForm) => {
    if (!form.name.trim()) {
      toast({ title: '錯誤', description: '設備名稱為必填', variant: 'destructive' })
      return
    }
    createEquipMutation.mutate(form)
  }

  /** 更新設備（含驗證） */
  const handleUpdateEquip = (id: string, form: EquipmentForm) => {
    if (!form.name.trim()) {
      toast({ title: '錯誤', description: '設備名稱為必填', variant: 'destructive' })
      return
    }
    updateEquipMutation.mutate({
      id,
      payload: {
        name: form.name,
        model: emptyToNull(form.model),
        serial_number: emptyToNull(form.serial_number),
        location: emptyToNull(form.location),
        notes: emptyToNull(form.notes),
      },
    })
  }

  /** 新增校正紀錄（含驗證） */
  const handleCreateCalib = (form: CalibrationForm) => {
    if (!form.equipment_id || !form.calibrated_at) {
      toast({ title: '錯誤', description: '請選擇設備並填寫校正日期', variant: 'destructive' })
      return
    }
    createCalibMutation.mutate(form)
  }

  /** 更新校正紀錄（含驗證） */
  const handleUpdateCalib = (id: string, form: CalibrationForm) => {
    if (!form.calibrated_at) {
      toast({ title: '錯誤', description: '校正日期為必填', variant: 'destructive' })
      return
    }
    updateCalibMutation.mutate({
      id,
      payload: {
        calibrated_at: form.calibrated_at,
        next_due_at: emptyToNull(form.next_due_at),
        result: emptyToNull(form.result),
        notes: emptyToNull(form.notes),
      },
    })
  }

  return {
    createEquipMutation,
    updateEquipMutation,
    deleteEquipMutation,
    createCalibMutation,
    updateCalibMutation,
    deleteCalibMutation,
    handleCreateEquip,
    handleUpdateEquip,
    handleCreateCalib,
    handleUpdateCalib,
  }
}

/** 建立空白設備表單 */
export const emptyEquipForm = (): EquipmentForm => ({
  name: '',
  model: '',
  serial_number: '',
  location: '',
  notes: '',
})

/** 建立空白校正紀錄表單 */
export const emptyCalibForm = (): CalibrationForm => ({
  equipment_id: '',
  calibrated_at: format(new Date(), 'yyyy-MM-dd'),
  next_due_at: '',
  result: '',
  notes: '',
})
