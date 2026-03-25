/**
 * 設備維護管理頁 — 實驗室 GLP 合規
 *
 * 功能：設備 CRUD、校正/確效/查核、維修/保養、報廢、年度計畫
 */

import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import { Plus, Wrench, Ruler, Hammer, Trash2, Calendar } from 'lucide-react'

import { useDialogSet } from '@/hooks/useDialogSet'
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
import { EquipmentTabContent } from './components/EquipmentTabContent'
import { CalibrationTabContent } from './components/CalibrationTabContent'
import { MaintenanceTabContent } from './components/MaintenanceTabContent'
import { DisposalTabContent } from './components/DisposalTabContent'
import AnnualPlanTabContent from './components/AnnualPlanTabContent'
import { EquipmentStatsCards } from './components/EquipmentStatsCards'

export function EquipmentPage() {
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('equipment.manage')
  const canApproveDisposal = hasPermission('equipment.disposal.approve')
  const queryClient = useQueryClient()
  const dialogs = useDialogSet(['equipCreate', 'equipEdit', 'calibCreate', 'calibEdit'] as const)

  const [equipKeyword, setEquipKeyword] = useState('')
  const [equipPage, setEquipPage] = useState(1)
  const [calibEquipmentFilter, setCalibEquipmentFilter] = useState('')
  const [calibPage, setCalibPage] = useState(1)
  const [maintPage, setMaintPage] = useState(1)
  const [disposalPage, setDisposalPage] = useState(1)
  const [planYear, setPlanYear] = useState(new Date().getFullYear())

  const [editingEquip, setEditingEquip] = useState<Equipment | null>(null)
  const [equipForm, setEquipForm] = useState<EquipmentForm>(emptyEquipForm())
  const [selectedPartnerIds, setSelectedPartnerIds] = useState<string[]>([])
  const [editingCalib, setEditingCalib] = useState<CalibrationWithEquipment | null>(null)
  const [calibForm, setCalibForm] = useState<CalibrationForm>(emptyCalibForm())

  /* ── Queries ── */
  const { data: equipmentList = [] } = useQuery({
    queryKey: ['equipment-all'],
    queryFn: async () => {
      const res = await api.get<PaginatedResponse<Equipment>>('/equipment', {
        params: { per_page: 500 },
      })
      return res.data.data
    },
  })

  const { data: allCalibrations = [] } = useQuery({
    queryKey: ['equipment-calibrations-all'],
    queryFn: async () => {
      const res = await api.get<PaginatedResponse<CalibrationWithEquipment>>(
        '/equipment-calibrations',
        { params: { per_page: 500 } },
      )
      return res.data.data
    },
  })

  const { data: equipData, isLoading: equipLoading } = useQuery({
    queryKey: ['equipment', equipKeyword, equipPage],
    queryFn: async () => {
      const params: Record<string, string | number> = { page: equipPage, per_page: 20 }
      if (equipKeyword) params.keyword = equipKeyword
      return (await api.get<PaginatedResponse<Equipment>>('/equipment', { params })).data
    },
  })

  const { data: calibData, isLoading: calibLoading } = useQuery({
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
  })

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

  const { data: plans = [] } = useQuery({
    queryKey: ['equipment-annual-plans', planYear],
    queryFn: async () =>
      (
        await api.get<AnnualPlanWithEquipment[]>('/equipment-annual-plans', {
          params: { year: planYear },
        })
      ).data,
  })

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

  /* ── Handlers ── */
  const handleEditEquip = async (equip: Equipment) => {
    setEditingEquip(equip)
    setEquipForm({
      name: equip.name,
      model: equip.model || '',
      serial_number: equip.serial_number || '',
      location: equip.location || '',
      notes: equip.notes || '',
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
    })
    dialogs.open('calibEdit')
  }

  const handleDeleteCalib = (id: string) => {
    if (window.confirm('確定要刪除此紀錄嗎？')) {
      mutations.deleteCalibMutation.mutate(id)
    }
  }

  const handleDeleteMaint = (id: string) => {
    if (window.confirm('確定要刪除此紀錄嗎？')) {
      deleteMaintMutation.mutate(id)
    }
  }

  const handleApproveDisposal = (id: string, approved: boolean) => {
    const msg = approved ? '確定核准此報廢申請？' : '確定駁回此報廢申請？'
    if (window.confirm(msg)) {
      approveDisposalMutation.mutate({ id, approved })
    }
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="設備維護管理"
        description="實驗室 GLP 合規：設備管理、校正/確效/查核、維修/保養、報廢"
        actions={canManage ? (
          <Button onClick={() => dialogs.open('equipCreate')}>
            <Plus className="h-4 w-4 mr-2" />
            新增設備
          </Button>
        ) : undefined}
      />

      <EquipmentStatsCards equipmentList={equipmentList} allCalibrations={allCalibrations} />

      <PageTabs
        tabs={[
          { value: 'equipment', label: '設備', icon: Wrench },
          { value: 'calibrations', label: '校正/確效/查核', icon: Ruler },
          { value: 'maintenance', label: '維修/保養', icon: Hammer },
          { value: 'disposals', label: '報廢', icon: Trash2 },
          { value: 'annual-plan', label: '年度計畫', icon: Calendar },
        ]}
        defaultTab="equipment"
      >
        <PageTabContent value="equipment" className="space-y-4">
          <EquipmentTabContent
            canManage={canManage}
            keyword={equipKeyword}
            onKeywordChange={setEquipKeyword}
            isLoading={equipLoading}
            records={equipData?.data ?? []}
            page={equipPage}
            totalPages={equipData?.total_pages ?? 1}
            onPageChange={setEquipPage}
            onEdit={handleEditEquip}
            onDelete={handleDeleteEquip}
            allCalibrations={allCalibrations}
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
            records={maintData?.data ?? []}
            isLoading={maintLoading}
            page={maintPage}
            totalPages={maintData?.total_pages ?? 1}
            onPageChange={setMaintPage}
            onDelete={handleDeleteMaint}
          />
        </PageTabContent>

        <PageTabContent value="disposals" className="space-y-4">
          <DisposalTabContent
            canApprove={canApproveDisposal}
            records={disposalData?.data ?? []}
            isLoading={disposalLoading}
            page={disposalPage}
            totalPages={disposalData?.total_pages ?? 1}
            onPageChange={setDisposalPage}
            onApprove={handleApproveDisposal}
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
    </div>
  )
}
