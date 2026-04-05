/**
 * 設備維護管理頁 — 實驗室 GLP 合規
 *
 * 功能：設備 CRUD、校正/確效/查核、維修/保養、報廢、年度計畫
 */

import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import { Plus, Wrench, Ruler, Hammer, Trash2, Calendar, Pause } from 'lucide-react'

import { GuestHide } from '@/components/ui/guest-hide'
import { useDialogSet } from '@/hooks/useDialogSet'
import { useGuestQuery } from '@/hooks/useGuestQuery'
import {
  DEMO_EQUIPMENT_PAGINATED,
  DEMO_EQUIPMENT_ALL,
  DEMO_CALIBRATIONS,
  DEMO_ANNUAL_PLANS,
} from '@/lib/guest-demo'
import api, { deleteResource } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import { useAuthStore } from '@/stores/auth'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import type { PaginatedResponse } from '@/types/common'

import type {
  Equipment,
  AnnualPlanExecutionSummary,
  CalibrationWithEquipment,
  EquipmentForm,
  CalibrationForm,
  MaintenanceRecordWithDetails,
  DisposalWithDetails,
  AnnualPlanWithEquipment,
  EquipmentSupplierWithPartner,
} from './types'

interface Partner {
  id: string
  name: string
}
import { useEquipmentMutations, emptyEquipForm, emptyCalibForm } from './hooks/useEquipmentMutations'
import { EquipmentFormDialog } from './components/EquipmentFormDialog'
import { CalibrationFormDialog } from './components/CalibrationFormDialog'
import { MaintenanceFormDialog, emptyMaintenanceForm, maintenanceFormFromRecord } from './components/MaintenanceFormDialog'
import type { MaintenanceFormData } from './components/MaintenanceFormDialog'
import { DisposalFormDialog, emptyDisposalForm } from './components/DisposalFormDialog'
import type { DisposalFormData } from './components/DisposalFormDialog'
import { EquipmentTabContent } from './components/EquipmentTabContent'
import { CalibrationTabContent } from './components/CalibrationTabContent'
import { MaintenanceTabContent } from './components/MaintenanceTabContent'
import { MaintenanceHistoryDialog } from './components/MaintenanceHistoryDialog'
import { MaintenanceReviewDialog } from './components/MaintenanceReviewDialog'
import { DisposalTabContent } from './components/DisposalTabContent'
import AnnualPlanTabContent from './components/AnnualPlanTabContent'
import { EquipmentStatsCards } from './components/EquipmentStatsCards'
import { IdleTabContent, type IdleRequestWithDetails } from './components/IdleTabContent'

export function EquipmentPage() {
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('equipment.manage')
  const canReview = hasPermission('equipment.maintenance.review') || canManage
  const canApproveDisposal = hasPermission('equipment.disposal.approve')
  const canApproveIdle = hasPermission('equipment.idle.approve') || hasPermission('admin')
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['equipCreate', 'equipEdit', 'calibCreate', 'calibEdit', 'maintCreate', 'maintEdit', 'disposalCreate'] as const)

  const [equipKeyword, setEquipKeyword] = useState('')
  const [equipStatusFilter, setEquipStatusFilter] = useState<string>('')
  const [equipPage, setEquipPage] = useState(1)
  const [calibEquipmentFilter, setCalibEquipmentFilter] = useState('')
  const [calibPage, setCalibPage] = useState(1)
  const [maintPage, setMaintPage] = useState(1)
  const [maintHistoryId, setMaintHistoryId] = useState<string | null>(null)
  const [reviewRecord, setReviewRecord] = useState<MaintenanceRecordWithDetails | null>(null)
  const [reviewMode, setReviewMode] = useState<'approve' | 'reject'>('approve')
  const [disposalPage, setDisposalPage] = useState(1)
  const [idlePage, setIdlePage] = useState(1)
  const [planYear, setPlanYear] = useState(new Date().getFullYear())

  const [editingEquip, setEditingEquip] = useState<Equipment | null>(null)
  const [equipForm, setEquipForm] = useState<EquipmentForm>(emptyEquipForm())
  const [selectedPartnerIds, setSelectedPartnerIds] = useState<string[]>([])
  const [editingCalib, setEditingCalib] = useState<CalibrationWithEquipment | null>(null)
  const [calibForm, setCalibForm] = useState<CalibrationForm>(emptyCalibForm())
  const [editingMaint, setEditingMaint] = useState<MaintenanceRecordWithDetails | null>(null)
  const [maintForm, setMaintForm] = useState<MaintenanceFormData>(emptyMaintenanceForm())
  const [disposalForm, setDisposalForm] = useState<DisposalFormData>(emptyDisposalForm())

  /* ── Queries ── */
  const { data: equipmentList = [] } = useGuestQuery(
    DEMO_EQUIPMENT_ALL.data as unknown as Equipment[],
    {
      queryKey: ['equipment-all'],
      queryFn: async () => {
        const res = await api.get<PaginatedResponse<Equipment>>('/equipment', {
          params: { per_page: 500 },
        })
        return res.data.data
      },
    },
  )

  const { data: allCalibrations = [] } = useGuestQuery(
    DEMO_CALIBRATIONS.data as unknown as CalibrationWithEquipment[],
    {
      queryKey: ['equipment-calibrations-all'],
      queryFn: async () => {
        const res = await api.get<PaginatedResponse<CalibrationWithEquipment>>(
          '/equipment-calibrations',
          { params: { per_page: 500 } },
        )
        return res.data.data
      },
    },
  )

  const { data: equipData, isLoading: equipLoading } = useGuestQuery(
    DEMO_EQUIPMENT_PAGINATED as unknown as PaginatedResponse<Equipment>,
    {
      queryKey: ['equipment', equipKeyword, equipStatusFilter, equipPage],
      queryFn: async () => {
        const params: Record<string, string | number> = { page: equipPage, per_page: 20 }
        if (equipKeyword) params.keyword = equipKeyword
        if (equipStatusFilter) params.status = equipStatusFilter
        return (await api.get<PaginatedResponse<Equipment>>('/equipment', { params })).data
      },
    },
  )

  const { data: calibData, isLoading: calibLoading } = useGuestQuery(
    DEMO_CALIBRATIONS as unknown as PaginatedResponse<CalibrationWithEquipment>,
    {
      queryKey: ['equipment-calibrations', calibEquipmentFilter || undefined, calibPage],
      queryFn: async () => {
        const params: Record<string, string | number> = { page: calibPage, per_page: 20 }
        if (calibEquipmentFilter) params.equipment_id = calibEquipmentFilter
        return (
          await api.get<PaginatedResponse<CalibrationWithEquipment>>('/equipment-calibrations', {
            params,
          })
        ).data
      },
    },
  )

  const { data: maintData, isLoading: maintLoading } = useQuery({
    queryKey: ['equipment-maintenance', maintPage],
    queryFn: async () =>
      (
        await api.get<PaginatedResponse<MaintenanceRecordWithDetails>>('/equipment-maintenance', {
          params: { page: maintPage, per_page: 20 },
        })
      ).data,
  })

  const { data: disposalData, isLoading: disposalLoading } = useQuery({
    queryKey: ['equipment-disposals', disposalPage],
    queryFn: async () =>
      (
        await api.get<PaginatedResponse<DisposalWithDetails>>('/equipment-disposals', {
          params: { page: disposalPage, per_page: 20 },
        })
      ).data,
  })

  const { data: idleData, isLoading: idleLoading } = useQuery({
    queryKey: ['equipment-idle-requests', idlePage],
    queryFn: async () =>
      (
        await api.get<PaginatedResponse<IdleRequestWithDetails>>('/equipment-idle-requests', {
          params: { page: idlePage, per_page: 20 },
        })
      ).data,
  })

  const { data: plans = [] } = useGuestQuery(
    DEMO_ANNUAL_PLANS as unknown as AnnualPlanWithEquipment[],
    {
      queryKey: ['equipment-annual-plans', planYear],
      queryFn: async () =>
        (
          await api.get<AnnualPlanWithEquipment[]>('/equipment-annual-plans', {
            params: { year: planYear },
          })
        ).data,
    },
  )

  const { data: executionSummary = null } = useGuestQuery<AnnualPlanExecutionSummary | null>(
    null,
    {
      queryKey: ['equipment-annual-plans-summary', planYear],
      queryFn: async () =>
        (
          await api.get<AnnualPlanExecutionSummary>('/equipment-annual-plans/execution-summary', {
            params: { year: planYear },
          })
        ).data,
    },
  )

  const { data: partnerOptions = [] } = useQuery({
    queryKey: ['partners-supplier'],
    queryFn: async () => {
      const res = await api.get<Partner[]>('/partners', { params: { partner_type: 'supplier' } })
      return res.data
    },
  })

  /* ── Mutations ── */
  const mutations = useEquipmentMutations({
    closeEquipCreate: () => dialogs.close('equipCreate'),
    closeEquipEdit: () => dialogs.close('equipEdit'),
    closeCalibCreate: () => dialogs.close('calibCreate'),
    closeCalibEdit: () => dialogs.close('calibEdit'),
    clearEditingEquip: () => setEditingEquip(null),
    clearEditingCalib: () => setEditingCalib(null),
    resetEquipForm: () => setEquipForm(emptyEquipForm()),
    resetCalibForm: () => setCalibForm(emptyCalibForm()),
  })

  const generatePlanMutation = useMutation({
    mutationFn: () => api.post('/equipment-annual-plans/generate', { year: planYear }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-annual-plans'] })
      toast({ title: '成功', description: `已產生 ${planYear} 年度計畫` })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '產生失敗'), variant: 'destructive' })
    },
  })

  const createPlanMutation = useMutation({
    mutationFn: (data: Record<string, unknown>) => api.post('/equipment-annual-plans', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-annual-plans'] })
      toast({ title: '成功', description: '已新增年度計畫項目' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '新增失敗'), variant: 'destructive' })
    },
  })

  const updatePlanMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Record<string, unknown> }) =>
      api.put(`/equipment-annual-plans/${id}`, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-annual-plans'] })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
    },
  })

  const deletePlanMutation = useMutation({
    mutationFn: (id: string) => deleteResource(`/equipment-annual-plans/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-annual-plans'] })
      toast({ title: '成功', description: '已刪除年度計畫項目' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '刪除失敗'), variant: 'destructive' })
    },
  })

  const approveDisposalMutation = useMutation({
    mutationFn: ({ id, approved }: { id: string; approved: boolean }) =>
      api.post(`/equipment-disposals/${id}/approve`, {
        approved,
        rejection_reason: approved ? null : '駁回',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-disposals'] })
      queryClient.invalidateQueries({ queryKey: ['equipment-all'] })
      toast({ title: '成功', description: '已處理報廢申請' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '操作失敗'), variant: 'destructive' })
    },
  })

  const restoreEquipmentMutation = useMutation({
    mutationFn: (disposalId: string) =>
      api.post(`/equipment-disposals/${disposalId}/restore`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-disposals'] })
      queryClient.invalidateQueries({ queryKey: ['equipment'] })
      queryClient.invalidateQueries({ queryKey: ['equipment-all'] })
      toast({ title: '成功', description: '設備已恢復為啟用狀態' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '恢復失敗'), variant: 'destructive' })
    },
  })

  const createIdleMutation = useMutation({
    mutationFn: (data: { equipment_id: string; request_type: string; reason: string }) =>
      api.post('/equipment-idle-requests', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-idle-requests'] })
      toast({ title: '成功', description: '已提交閒置申請' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '申請失敗'), variant: 'destructive' })
    },
  })

  const approveIdleMutation = useMutation({
    mutationFn: ({ id, approved }: { id: string; approved: boolean }) =>
      api.post(`/equipment-idle-requests/${id}/approve`, {
        approved,
        rejection_reason: approved ? null : '駁回',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-idle-requests'] })
      queryClient.invalidateQueries({ queryKey: ['equipment'] })
      queryClient.invalidateQueries({ queryKey: ['equipment-all'] })
      toast({ title: '成功', description: '已處理閒置申請' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '操作失敗'), variant: 'destructive' })
    },
  })

  const deleteMaintMutation = useMutation({
    mutationFn: (id: string) => deleteResource(`/equipment-maintenance/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-maintenance'] })
      toast({ title: '成功', description: '已刪除紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '刪除失敗'), variant: 'destructive' })
    },
  })

  const createMaintMutation = useMutation({
    mutationFn: (data: MaintenanceFormData) => {
      const { status: _s, completed_at, repair_content, repair_partner_id, ...rest } = data
      return api.post('/equipment-maintenance', {
        ...rest,
        completed_at: completed_at || null,
        repair_content: repair_content || null,
        repair_partner_id: repair_partner_id || null,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-maintenance'] })
      dialogs.close('maintCreate')
      setMaintForm(emptyMaintenanceForm())
      toast({ title: '成功', description: '已新增維修/保養紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '新增失敗'), variant: 'destructive' })
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
      queryClient.invalidateQueries({ queryKey: ['equipment-maintenance'] })
      dialogs.close('maintEdit')
      setEditingMaint(null)
      setMaintForm(emptyMaintenanceForm())
      toast({ title: '成功', description: '已更新維修/保養紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
    },
  })

  const createDisposalMutation = useMutation({
    mutationFn: (data: DisposalFormData) =>
      api.post('/equipment-disposals', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-disposals'] })
      queryClient.invalidateQueries({ queryKey: ['equipment-all'] })
      dialogs.close('disposalCreate')
      setDisposalForm(emptyDisposalForm())
      toast({ title: '成功', description: '已送出報廢申請' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '申請失敗'), variant: 'destructive' })
    },
  })

  /* ── Handlers ── */
  const handleEditEquip = async (equip: Equipment) => {
    setEditingEquip(equip)
    setEquipForm({
      name: equip.name,
      model: equip.model || '',
      serial_number: equip.serial_number || '',
      location: equip.location || '',
      department: equip.department || '',
      purchase_date: equip.purchase_date || '',
      warranty_expiry: equip.warranty_expiry || '',
      notes: equip.notes || '',
      status: equip.status,
      calibration_type: equip.calibration_type || '',
      calibration_cycle: equip.calibration_cycle || '',
      inspection_cycle: equip.inspection_cycle || '',
    })
    try {
      const res = await api.get<EquipmentSupplierWithPartner[]>(`/equipment/${equip.id}/suppliers`)
      setSelectedPartnerIds(res.data.map((s) => s.partner_id))
    } catch {
      setSelectedPartnerIds([])
    }
    dialogs.open('equipEdit')
  }

  const handleDeleteEquip = (id: string, name: string) => {
    if (window.confirm(`確定要刪除設備「${name}」嗎？`)) {
      mutations.deleteEquipMutation.mutate(id)
    }
  }

  const handleAddCalib = () => {
    setCalibForm({
      equipment_id: calibEquipmentFilter || (equipmentList[0]?.id ?? ''),
      calibration_type: 'calibration',
      calibrated_at: format(new Date(), 'yyyy-MM-dd'),
      next_due_at: '',
      result: '',
      notes: '',
      partner_id: '',
      report_number: '',
      inspector: '',
      certificate_number: '',
      performed_by: '',
      acceptance_criteria: '',
      measurement_uncertainty: '',
      validation_phase: '',
      protocol_number: '',
    })
    dialogs.open('calibCreate')
  }

  const handleEditCalib = (calib: CalibrationWithEquipment) => {
    setEditingCalib(calib)
    setCalibForm({
      equipment_id: calib.equipment_id,
      calibration_type: calib.calibration_type,
      calibrated_at: calib.calibrated_at,
      next_due_at: calib.next_due_at || '',
      result: calib.result || '',
      notes: calib.notes || '',
      partner_id: calib.partner_id || '',
      report_number: calib.report_number || '',
      inspector: calib.inspector || '',
      certificate_number: calib.certificate_number || '',
      performed_by: calib.performed_by || '',
      acceptance_criteria: calib.acceptance_criteria || '',
      measurement_uncertainty: calib.measurement_uncertainty || '',
      validation_phase: calib.validation_phase || '',
      protocol_number: calib.protocol_number || '',
    })
    dialogs.open('calibEdit')
  }

  const handleDeleteCalib = (id: string) => {
    if (window.confirm('確定要刪除此紀錄嗎？')) {
      mutations.deleteCalibMutation.mutate(id)
    }
  }

  const handleAddMaint = () => {
    setMaintForm({
      ...emptyMaintenanceForm(),
      equipment_id: equipmentList[0]?.id ?? '',
    })
    dialogs.open('maintCreate')
  }

  const handleEditMaint = (record: MaintenanceRecordWithDetails) => {
    setEditingMaint(record)
    setMaintForm(maintenanceFormFromRecord(record))
    dialogs.open('maintEdit')
  }

  const handleDeleteMaint = (id: string) => {
    if (window.confirm('確定要刪除此紀錄嗎？')) {
      deleteMaintMutation.mutate(id)
    }
  }

  const handleRequestDisposal = () => {
    setDisposalForm({
      ...emptyDisposalForm(),
      equipment_id: equipmentList[0]?.id ?? '',
    })
    dialogs.open('disposalCreate')
  }

  const handleApproveDisposal = (id: string, approved: boolean) => {
    const msg = approved ? '確定核准此報廢申請？' : '確定駁回此報廢申請？'
    if (window.confirm(msg)) {
      approveDisposalMutation.mutate({ id, approved })
    }
  }

  const handleRestoreEquipment = (id: string) => {
    if (window.confirm('確定要恢復此設備為啟用狀態？')) {
      restoreEquipmentMutation.mutate(id)
    }
  }

  const handleRequestIdle = (equipmentId: string, requestType: 'idle' | 'restore') => {
    const label = requestType === 'idle' ? '閒置' : '恢復'
    const reason = window.prompt(`請輸入${label}原因：`)
    if (reason) {
      createIdleMutation.mutate({ equipment_id: equipmentId, request_type: requestType, reason })
    }
  }

  const handleApproveIdle = (id: string, approved: boolean) => {
    const msg = approved ? '確定核准此閒置申請？' : '確定駁回此閒置申請？'
    if (window.confirm(msg)) {
      approveIdleMutation.mutate({ id, approved })
    }
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="設備維護管理"
        description="實驗室 GLP 合規：設備管理、校正/確效/查核、維修/保養、報廢"
        actions={canManage ? (
          <GuestHide>
            <Button size="sm" onClick={() => dialogs.open('equipCreate')}>
              <Plus className="h-4 w-4 mr-2" />
              新增設備
            </Button>
          </GuestHide>
        ) : undefined}
      />

      <EquipmentStatsCards equipmentList={equipmentList} allCalibrations={allCalibrations} />

      <PageTabs
        tabs={[
          { value: 'equipment', label: '設備', icon: Wrench },
          { value: 'calibrations', label: '校正/確效/查核', icon: Ruler },
          { value: 'maintenance', label: '維修/保養', icon: Hammer },
          { value: 'idle', label: '閒置管理', icon: Pause },
          { value: 'disposals', label: '報廢', icon: Trash2 },
          { value: 'annual-plan', label: '年度計畫', icon: Calendar },
        ]}
        defaultTab="equipment"
        variant="underline"
      >
        <PageTabContent value="equipment" className="space-y-4">
          <EquipmentTabContent
            canManage={canManage}
            keyword={equipKeyword}
            onKeywordChange={setEquipKeyword}
            statusFilter={equipStatusFilter}
            onStatusFilterChange={(v) => { setEquipStatusFilter(v); setEquipPage(1) }}
            allCalibrations={allCalibrations}
            tableProps={{
              records: equipData?.data ?? [],
              isLoading: equipLoading,
              page: equipPage,
              totalPages: equipData?.total_pages ?? 1,
              onPageChange: setEquipPage,
            }}
            actions={{
              onEdit: handleEditEquip,
              onDelete: handleDeleteEquip,
              onRequestIdle: canManage ? handleRequestIdle : undefined,
            }}
          />
        </PageTabContent>

        <PageTabContent value="calibrations" className="space-y-4">
          <CalibrationTabContent
            canManage={canManage}
            equipmentList={equipmentList}
            calibEquipmentFilter={calibEquipmentFilter}
            onFilterChange={setCalibEquipmentFilter}
            isLoading={calibLoading}
            records={calibData?.data ?? []}
            page={calibPage}
            totalPages={calibData?.total_pages ?? 1}
            onPageChange={setCalibPage}
            onAddClick={handleAddCalib}
            onEdit={handleEditCalib}
            onDelete={handleDeleteCalib}
          />
        </PageTabContent>

        <PageTabContent value="maintenance" className="space-y-4">
          <MaintenanceTabContent
            canManage={canManage}
            canReview={canReview}
            records={maintData?.data ?? []}
            isLoading={maintLoading}
            page={maintPage}
            totalPages={maintData?.total_pages ?? 1}
            onPageChange={setMaintPage}
            onDelete={handleDeleteMaint}
            onAdd={handleAddMaint}
            onEdit={handleEditMaint}
            onViewHistory={setMaintHistoryId}
            onReview={(r) => { setReviewRecord(r); setReviewMode('approve') }}
            onReject={(r) => { setReviewRecord(r); setReviewMode('reject') }}
          />
          <MaintenanceHistoryDialog
            open={!!maintHistoryId}
            onOpenChange={(open) => { if (!open) setMaintHistoryId(null) }}
            recordId={maintHistoryId}
          />
          <MaintenanceReviewDialog
            open={!!reviewRecord}
            onOpenChange={(open) => { if (!open) setReviewRecord(null) }}
            record={reviewRecord}
            mode={reviewMode}
          />
        </PageTabContent>

        <PageTabContent value="idle" className="space-y-4">
          <IdleTabContent
            canApprove={canApproveIdle}
            records={idleData?.data ?? []}
            isLoading={idleLoading}
            page={idlePage}
            totalPages={idleData?.total_pages ?? 1}
            onPageChange={setIdlePage}
            onApprove={handleApproveIdle}
          />
        </PageTabContent>

        <PageTabContent value="disposals" className="space-y-4">
          <DisposalTabContent
            canApprove={canApproveDisposal}
            canRequest={canManage}
            records={disposalData?.data ?? []}
            isLoading={disposalLoading}
            page={disposalPage}
            totalPages={disposalData?.total_pages ?? 1}
            onPageChange={setDisposalPage}
            onApprove={handleApproveDisposal}
            onRestore={handleRestoreEquipment}
            onRequestDisposal={handleRequestDisposal}
          />
        </PageTabContent>

        <PageTabContent value="annual-plan" className="space-y-4">
          <AnnualPlanTabContent
            canManage={canManage}
            plans={plans}
            year={planYear}
            onYearChange={setPlanYear}
            onGenerate={() => generatePlanMutation.mutate()}
            isGenerating={generatePlanMutation.isPending}
            equipmentList={equipmentList}
            executionSummary={executionSummary}
            onCreatePlan={(data) => createPlanMutation.mutate(data)}
            onEditPlan={(id, data) => {
              const monthData: Record<string, unknown> = { ...data }
              updatePlanMutation.mutate({ id, data: monthData as Record<string, boolean> })
            }}
            onToggleMonth={(plan, month) => {
              const monthData: Record<string, unknown> = {}
              for (let i = 1; i <= 12; i++) {
                const key = `month_${i}` as keyof AnnualPlanWithEquipment
                monthData[`month_${i}`] = i === month ? !(plan[key] as boolean) : (plan[key] as boolean)
              }
              updatePlanMutation.mutate({ id: plan.id, data: monthData })
            }}
            onDeletePlan={(id) => deletePlanMutation.mutate(id)}
          />
        </PageTabContent>
      </PageTabs>

      {/* Dialogs */}
      <EquipmentFormDialog
        open={dialogs.isOpen('equipCreate')}
        onOpenChange={(open) => {
          if (!open) setSelectedPartnerIds([])
          dialogs.setOpen('equipCreate')(open)
        }}
        mode="create"
        form={equipForm}
        onFormChange={setEquipForm}
        onSubmit={() => mutations.handleCreateEquip(equipForm, selectedPartnerIds)}
        isPending={mutations.equipSaving}
        partnerOptions={partnerOptions}
        selectedPartnerIds={selectedPartnerIds}
        onPartnerIdsChange={setSelectedPartnerIds}
      />
      <EquipmentFormDialog
        open={dialogs.isOpen('equipEdit')}
        onOpenChange={(open) => {
          if (!open) {
            setEditingEquip(null)
            setSelectedPartnerIds([])
          }
          dialogs.setOpen('equipEdit')(open)
        }}
        mode="edit"
        form={equipForm}
        onFormChange={setEquipForm}
        onSubmit={() => editingEquip && mutations.handleUpdateEquip(editingEquip.id, equipForm, selectedPartnerIds)}
        isPending={mutations.equipSaving}
        partnerOptions={partnerOptions}
        selectedPartnerIds={selectedPartnerIds}
        onPartnerIdsChange={setSelectedPartnerIds}
      />
      <CalibrationFormDialog
        open={dialogs.isOpen('calibCreate')}
        onOpenChange={dialogs.setOpen('calibCreate')}
        mode="create"
        form={calibForm}
        onFormChange={setCalibForm}
        onSubmit={() => mutations.handleCreateCalib(calibForm)}
        isPending={mutations.createCalibMutation.isPending}
        equipmentList={equipmentList}
        editingCalib={null}
      />
      <CalibrationFormDialog
        open={dialogs.isOpen('calibEdit')}
        onOpenChange={(open) => {
          if (!open) setEditingCalib(null)
          dialogs.setOpen('calibEdit')(open)
        }}
        mode="edit"
        form={calibForm}
        onFormChange={setCalibForm}
        onSubmit={() => editingCalib && mutations.handleUpdateCalib(editingCalib.id, calibForm)}
        isPending={mutations.updateCalibMutation.isPending}
        equipmentList={equipmentList}
        editingCalib={editingCalib}
      />
      <MaintenanceFormDialog
        open={dialogs.isOpen('maintCreate')}
        onOpenChange={dialogs.setOpen('maintCreate')}
        mode="create"
        form={maintForm}
        onFormChange={setMaintForm}
        onSubmit={() => createMaintMutation.mutate(maintForm)}
        isPending={createMaintMutation.isPending}
        equipmentList={equipmentList}
      />
      <MaintenanceFormDialog
        open={dialogs.isOpen('maintEdit')}
        onOpenChange={(open) => {
          if (!open) setEditingMaint(null)
          dialogs.setOpen('maintEdit')(open)
        }}
        mode="edit"
        form={maintForm}
        onFormChange={setMaintForm}
        onSubmit={() => editingMaint && updateMaintMutation.mutate({ id: editingMaint.id, data: maintForm })}
        onSubmitComplete={() => editingMaint && updateMaintMutation.mutate({
          id: editingMaint.id,
          data: { ...maintForm, status: 'completed', completed_at: maintForm.completed_at || format(new Date(), 'yyyy-MM-dd') },
        })}
        onSubmitUnrepairable={() => editingMaint && updateMaintMutation.mutate({
          id: editingMaint.id,
          data: { ...maintForm, status: 'unrepairable' },
        })}
        isPending={updateMaintMutation.isPending}
        equipmentList={equipmentList}
        partners={partnerOptions}
      />
      <DisposalFormDialog
        open={dialogs.isOpen('disposalCreate')}
        onOpenChange={dialogs.setOpen('disposalCreate')}
        form={disposalForm}
        onFormChange={setDisposalForm}
        onSubmit={() => createDisposalMutation.mutate(disposalForm)}
        isPending={createDisposalMutation.isPending}
        equipmentList={equipmentList}
      />
    </div>
  )
}
