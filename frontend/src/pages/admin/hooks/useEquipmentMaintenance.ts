/**
 * 維修/保養分頁的完整資料層：篩選/分頁 state、清單查詢、CRUD mutations。
 *
 * 頁面只負責把回傳值接到 UI 上，不再自己管 queryKey 與失效範圍。
 */
import { useCallback, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, { deleteResource } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import type { PaginatedResponse } from '@/types/common'

import type { MaintenanceRecordWithDetails } from '../types'
import {
  emptyMaintenanceForm,
  type MaintenanceFormData,
} from '../components/MaintenanceFormDialog'

const MAINT_KEYS = {
  list: ['equipment-maintenance'],
  recent: ['recent-maintenance'],
} as const

const EQUIP_KEYS = {
  list: ['equipment'],
  all: ['equipment-all'],
} as const

interface UseEquipmentMaintenanceOptions {
  closeMaintCreate: () => void
  closeMaintEdit: () => void
}

export function useEquipmentMaintenance(options: UseEquipmentMaintenanceOptions) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()

  /* ── 篩選 / 分頁 ── */
  const [maintPage, setMaintPage] = useState(1)
  const [maintType, setMaintType] = useState('')
  const [maintStatus, setMaintStatus] = useState('')
  const [maintSort, setMaintSort] = useState<{ field: string | null; order: 'asc' | 'desc' }>({
    field: null,
    order: 'desc',
  })

  // 改變任一篩選/排序都回到第 1 頁，否則會停在不存在的頁碼上
  const handleMaintTypeChange = useCallback((v: string) => { setMaintType(v); setMaintPage(1) }, [])
  const handleMaintStatusChange = useCallback((v: string) => { setMaintStatus(v); setMaintPage(1) }, [])
  const handleMaintSortChange = useCallback((field: string) => {
    setMaintSort((prev) =>
      prev.field === field
        ? { field, order: prev.order === 'asc' ? 'desc' : 'asc' }
        : { field, order: 'desc' }
    )
    setMaintPage(1)
  }, [])

  /* ── 編輯 / 檢視 state ── */
  const [editingMaint, setEditingMaint] = useState<MaintenanceRecordWithDetails | null>(null)
  const [maintForm, setMaintForm] = useState<MaintenanceFormData>(emptyMaintenanceForm())
  const [maintHistoryId, setMaintHistoryId] = useState<string | null>(null)
  const [reviewRecord, setReviewRecord] = useState<MaintenanceRecordWithDetails | null>(null)
  const [reviewMode, setReviewMode] = useState<'approve' | 'reject'>('approve')

  /* ── Query ── */
  const { data: maintData, isLoading: maintLoading } = useQuery({
    queryKey: [...MAINT_KEYS.list, maintPage, maintType, maintStatus, maintSort.field, maintSort.order],
    queryFn: async () => {
      const params: Record<string, string | number> = { page: maintPage, per_page: 20 }
      if (maintType) params.maintenance_type = maintType
      if (maintStatus) params.status = maintStatus
      if (maintSort.field) {
        params.sort_by = maintSort.field
        params.sort_order = maintSort.order
      }
      return (
        await api.get<PaginatedResponse<MaintenanceRecordWithDetails>>('/equipment-maintenance', {
          params,
        })
      ).data
    },
  })

  /* ── Mutations ── */
  // 維修紀錄的異動會連帶改設備狀態（backend `services/equipment/maintenance.rs`）：
  // 建立時轉 under_repair + is_active=false、標記完成時轉回 active，
  // 故三個 mutation 都要一併失效設備 query，否則設備分頁與統計卡會停在舊狀態。
  const invalidateMaint = () => {
    queryClient.invalidateQueries({ queryKey: MAINT_KEYS.list })
    queryClient.invalidateQueries({ queryKey: MAINT_KEYS.recent })
    queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.list })
    queryClient.invalidateQueries({ queryKey: EQUIP_KEYS.all })
  }

  const deleteMaintMutation = useMutation({
    mutationFn: (id: string) => deleteResource(`/equipment-maintenance/${id}`),
    onSuccess: () => {
      invalidateMaint()
      toast({ title: t('common.success'), description: t('adminOps.equipment.toast.recordDeleted') })
    },
    onError: (err: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.deleteFailed')), variant: 'destructive' })
    },
  })

  const createMaintMutation = useMutation({
    mutationFn: (data: MaintenanceFormData) => {
      return api.post('/equipment-maintenance', {
        equipment_id: data.equipment_id,
        maintenance_type: data.maintenance_type,
        reported_at: data.reported_at,
        problem_description: data.problem_description,
        maintenance_items: data.maintenance_items,
        performed_by: data.performed_by,
        notes: data.notes,
        completed_at: data.completed_at || null,
        repair_content: data.repair_content || null,
        repair_partner_id: data.repair_partner_id || null,
      })
    },
    onSuccess: () => {
      invalidateMaint()
      options.closeMaintCreate()
      setMaintForm(emptyMaintenanceForm())
      toast({ title: t('common.success'), description: t('adminOps.equipment.toast.maintenanceCreated') })
    },
    onError: (err: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.createFailed')), variant: 'destructive' })
    },
  })

  const updateMaintMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: MaintenanceFormData }) => {
      return api.put(`/equipment-maintenance/${id}`, {
        ...data,
        status: data.status || null,
        completed_at: data.completed_at || null,
        repair_content: data.repair_content || null,
        repair_partner_id: data.repair_partner_id || null,
      })
    },
    onSuccess: () => {
      invalidateMaint()
      options.closeMaintEdit()
      setEditingMaint(null)
      setMaintForm(emptyMaintenanceForm())
      toast({ title: t('common.success'), description: t('adminOps.equipment.toast.maintenanceUpdated') })
    },
    onError: (err: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(err, t('adminOps.shared.updateFailed')), variant: 'destructive' })
    },
  })

  return {
    // 查詢結果
    maintData,
    maintLoading,
    // 篩選 / 分頁
    maintPage,
    setMaintPage,
    maintType,
    maintStatus,
    maintSort,
    handleMaintTypeChange,
    handleMaintStatusChange,
    handleMaintSortChange,
    // 編輯 / 檢視 state
    editingMaint,
    setEditingMaint,
    maintForm,
    setMaintForm,
    maintHistoryId,
    setMaintHistoryId,
    reviewRecord,
    setReviewRecord,
    reviewMode,
    setReviewMode,
    // mutations
    deleteMaintMutation,
    createMaintMutation,
    updateMaintMutation,
  }
}
