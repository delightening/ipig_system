/**
 * 設備維護管理的 mutation hooks
 */
import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { format } from 'date-fns'

import type { Equipment, EquipmentForm, CalibrationForm, EquipmentSupplierWithPartner } from '../types'

const EQUIP_KEYS = {
  list: ['equipment'],
  all: ['equipment-all'],
} as const

const CALIB_KEYS = {
  list: ['equipment-calibrations'],
  all: ['equipment-calibrations-all'],
} as const

const SUPPLIER_KEYS = {
  summary: ['equipment-suppliers-summary'],
} as const

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
  const [equipSaving, setEquipSaving] = useState(false)

  const invalidateEquip = () => {
    queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.list })
    queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.all })
    queryClient.invalidateQueries({ queryKey: SUPPLIER_KEYS.summary })
  }

  const invalidateCalib = () => {
    queryClient.invalidateQueries({ queryKey: CALIB_KEYS.list })
    queryClient.invalidateQueries({ queryKey: CALIB_KEYS.all })
  }

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
        calibration_type: payload.calibration_type,
        calibrated_at: payload.calibrated_at,
        next_due_at: emptyToNull(payload.next_due_at),
        result: emptyToNull(payload.result),
        notes: emptyToNull(payload.notes),
        partner_id: emptyToNull(payload.partner_id),
        report_number: emptyToNull(payload.report_number),
        inspector: emptyToNull(payload.inspector),
      }),
    onSuccess: () => {
      invalidateCalib()
      options.closeCalibCreate()
      options.resetCalibForm()
      toast({ title: '成功', description: '已新增紀錄' })
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
      toast({ title: '成功', description: '已更新紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
    },
  })

  const deleteCalibMutation = useMutation({
    mutationFn: (id: string) => deleteResource(`/equipment-calibrations/${id}`),
    onSuccess: () => {
      invalidateCalib()
      toast({ title: '成功', description: '已刪除紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '刪除失敗'), variant: 'destructive' })
    },
  })

  const syncSuppliers = async (equipmentId: string, partnerIds: string[]) => {
    const res = await api.get<EquipmentSupplierWithPartner[]>(`/equipment/${equipmentId}/suppliers`)
    const existing = res.data

    const existingPartnerIds = existing.map((s) => s.partner_id)
    const toAdd = partnerIds.filter((pid) => !existingPartnerIds.includes(pid))
    const toRemove = existing.filter((s) => !partnerIds.includes(s.partner_id))

    await Promise.all([
      ...toAdd.map((pid) => api.post(`/equipment/${equipmentId}/suppliers`, { partner_id: pid })),
      ...toRemove.map((s) => deleteResource(`/equipment-suppliers/${s.id}`)),
    ])
  }

  const handleCreateEquip = async (form: EquipmentForm, partnerIds: string[]) => {
    if (!form.name.trim()) {
      toast({ title: '錯誤', description: '設備名稱為必填', variant: 'destructive' })
      return
    }
    setEquipSaving(true)
    try {
      const res = await api.post<Equipment>('/equipment', {
        name: form.name,
        model: emptyToNull(form.model),
        serial_number: emptyToNull(form.serial_number),
        location: emptyToNull(form.location),
        notes: emptyToNull(form.notes),
        calibration_type: form.calibration_type || null,
        calibration_cycle: form.calibration_cycle || null,
        inspection_cycle: form.inspection_cycle || null,
      })
      const created = res.data
      if (partnerIds.length > 0) {
        await Promise.all(
          partnerIds.map((pid) => api.post(`/equipment/${created.id}/suppliers`, { partner_id: pid })),
        )
      }
      invalidateEquip()
      options.closeEquipCreate()
      options.resetEquipForm()
      toast({ title: '成功', description: '已新增設備' })
    } catch (err: unknown) {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '新增失敗'), variant: 'destructive' })
    } finally {
      setEquipSaving(false)
    }
  }

  const handleUpdateEquip = async (id: string, form: EquipmentForm, partnerIds: string[]) => {
    if (!form.name.trim()) {
      toast({ title: '錯誤', description: '設備名稱為必填', variant: 'destructive' })
      return
    }
    setEquipSaving(true)
    try {
      await api.put(`/equipment/${id}`, {
        name: form.name,
        model: emptyToNull(form.model),
        serial_number: emptyToNull(form.serial_number),
        location: emptyToNull(form.location),
        notes: emptyToNull(form.notes),
        calibration_type: form.calibration_type || null,
        calibration_cycle: form.calibration_cycle || null,
        inspection_cycle: form.inspection_cycle || null,
      })
      await syncSuppliers(id, partnerIds)
      invalidateEquip()
      options.closeEquipEdit()
      options.clearEditingEquip()
      toast({ title: '成功', description: '已更新設備' })
    } catch (err: unknown) {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
    } finally {
      setEquipSaving(false)
    }
  }

  const handleCreateCalib = (form: CalibrationForm) => {
    if (!form.equipment_id || !form.calibrated_at) {
      toast({ title: '錯誤', description: '請選擇設備並填寫執行日期', variant: 'destructive' })
      return
    }
    createCalibMutation.mutate(form)
  }

  const handleUpdateCalib = (id: string, form: CalibrationForm) => {
    if (!form.calibrated_at) {
      toast({ title: '錯誤', description: '執行日期為必填', variant: 'destructive' })
      return
    }
    updateCalibMutation.mutate({
      id,
      payload: {
        calibration_type: form.calibration_type,
        calibrated_at: form.calibrated_at,
        next_due_at: emptyToNull(form.next_due_at),
        result: emptyToNull(form.result),
        notes: emptyToNull(form.notes),
        partner_id: emptyToNull(form.partner_id),
        report_number: emptyToNull(form.report_number),
        inspector: emptyToNull(form.inspector),
      },
    })
  }

  return {
    equipSaving,
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

export const emptyEquipForm = (): EquipmentForm => ({
  name: '',
  model: '',
  serial_number: '',
  location: '',
  notes: '',
  calibration_type: '',
  calibration_cycle: '',
  inspection_cycle: '',
})

export const emptyCalibForm = (): CalibrationForm => ({
  equipment_id: '',
  calibration_type: 'calibration',
  calibrated_at: format(new Date(), 'yyyy-MM-dd'),
  next_due_at: '',
  result: '',
  notes: '',
  partner_id: '',
  report_number: '',
  inspector: '',
})
