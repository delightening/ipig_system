/**
 * 設備維護管理頁 — 實驗室 GLP 合規
 *
 * 功能：
 * - 設備 CRUD（含狀態、校正/確效類型、週期設定）
 * - 校正/確效/查核紀錄 CRUD
 */

import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { Plus, Wrench, Ruler } from 'lucide-react'

import { useTabState } from '@/hooks/useTabState'
import { useDialogSet } from '@/hooks/useDialogSet'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useAuthStore } from '@/stores/auth'
import type { PaginatedResponse } from '@/types/common'

import type {
  Equipment,
  CalibrationWithEquipment,
  EquipmentForm,
  CalibrationForm,
} from './types'
import { useEquipmentMutations, emptyEquipForm, emptyCalibForm } from './hooks/useEquipmentMutations'
import { EquipmentFormDialog } from './components/EquipmentFormDialog'
import { CalibrationFormDialog } from './components/CalibrationFormDialog'
import { EquipmentTabContent } from './components/EquipmentTabContent'
import { CalibrationTabContent } from './components/CalibrationTabContent'
import { EquipmentStatsCards } from './components/EquipmentStatsCards'

export function EquipmentPage() {
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('equipment.manage')

  const { activeTab, setActiveTab } = useTabState<'equipment' | 'calibrations'>('equipment')
  const dialogs = useDialogSet(['equipCreate', 'equipEdit', 'calibCreate', 'calibEdit'] as const)

  const [equipKeyword, setEquipKeyword] = useState('')
  const [equipPage, setEquipPage] = useState(1)

  const [calibEquipmentFilter, setCalibEquipmentFilter] = useState('')
  const [calibPage, setCalibPage] = useState(1)

  const [editingEquip, setEditingEquip] = useState<Equipment | null>(null)
  const [equipForm, setEquipForm] = useState<EquipmentForm>(emptyEquipForm())

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
      const res = await api.get<PaginatedResponse<Equipment>>('/equipment', { params })
      return res.data
    },
  })

  const { data: calibData, isLoading: calibLoading } = useQuery({
    queryKey: ['equipment-calibrations', calibEquipmentFilter || undefined, calibPage],
    queryFn: async () => {
      const params: Record<string, string | number> = { page: calibPage, per_page: 20 }
      if (calibEquipmentFilter) params.equipment_id = calibEquipmentFilter
      const res = await api.get<PaginatedResponse<CalibrationWithEquipment>>(
        '/equipment-calibrations',
        { params },
      )
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

  const equipRecords = equipData?.data ?? []
  const equipTotalPages = equipData?.total_pages ?? 1
  const calibRecords = calibData?.data ?? []
  const calibTotalPages = calibData?.total_pages ?? 1

  const handleEditEquip = (equip: Equipment) => {
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

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">設備維護管理</h1>
          <p className="text-muted-foreground">
            實驗室 GLP 合規：設備管理、校正/確效/查核紀錄追蹤
          </p>
        </div>
        {canManage && (
          <Button onClick={() => dialogs.open('equipCreate')}>
            <Plus className="h-4 w-4 mr-2" />
            新增設備
          </Button>
        )}
      </div>

      <EquipmentStatsCards equipmentList={equipmentList} allCalibrations={allCalibrations} />

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="equipment" className="flex items-center gap-2">
            <Wrench className="h-4 w-4" />
            設備管理
          </TabsTrigger>
          <TabsTrigger value="calibrations" className="flex items-center gap-2">
            <Ruler className="h-4 w-4" />
            校正/確效/查核
          </TabsTrigger>
        </TabsList>

        <TabsContent value="equipment" className="space-y-4">
          <EquipmentTabContent
            canManage={canManage}
            keyword={equipKeyword}
            onKeywordChange={setEquipKeyword}
            isLoading={equipLoading}
            records={equipRecords}
            page={equipPage}
            totalPages={equipTotalPages}
            onPageChange={setEquipPage}
            onEdit={handleEditEquip}
            onDelete={handleDeleteEquip}
            allCalibrations={allCalibrations}
          />
        </TabsContent>

        <TabsContent value="calibrations" className="space-y-4">
          <CalibrationTabContent
            canManage={canManage}
            equipmentList={equipmentList}
            calibEquipmentFilter={calibEquipmentFilter}
            onFilterChange={setCalibEquipmentFilter}
            isLoading={calibLoading}
            records={calibRecords}
            page={calibPage}
            totalPages={calibTotalPages}
            onPageChange={setCalibPage}
            onAddClick={handleAddCalib}
            onEdit={handleEditCalib}
            onDelete={handleDeleteCalib}
          />
        </TabsContent>
      </Tabs>

      <EquipmentFormDialog
        open={dialogs.isOpen('equipCreate')}
        onOpenChange={dialogs.setOpen('equipCreate')}
        mode="create"
        form={equipForm}
        onFormChange={setEquipForm}
        onSubmit={() => mutations.handleCreateEquip(equipForm)}
        isPending={mutations.createEquipMutation.isPending}
      />
      <EquipmentFormDialog
        open={dialogs.isOpen('equipEdit')}
        onOpenChange={(open) => {
          if (!open) setEditingEquip(null)
          dialogs.setOpen('equipEdit')(open)
        }}
        mode="edit"
        form={equipForm}
        onFormChange={setEquipForm}
        onSubmit={() => editingEquip && mutations.handleUpdateEquip(editingEquip.id, equipForm)}
        isPending={mutations.updateEquipMutation.isPending}
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
