/**
 * 維修/保養紀錄新增／編輯 Dialog
 */
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { FormField } from '@/components/ui/form-field'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Loader2 } from 'lucide-react'
import { format } from 'date-fns'

import type { Equipment, MaintenanceType, MaintenanceRecordWithDetails } from '../types'
import { MAINTENANCE_TYPE_LABELS } from '../types'

export interface MaintenanceFormData {
  equipment_id: string
  maintenance_type: MaintenanceType
  reported_at: string
  problem_description: string
  maintenance_items: string
  performed_by: string
  notes: string
}

// eslint-disable-next-line react-refresh/only-export-components
export function emptyMaintenanceForm(): MaintenanceFormData {
  return {
    equipment_id: '',
    maintenance_type: 'repair',
    reported_at: format(new Date(), 'yyyy-MM-dd'),
    problem_description: '',
    maintenance_items: '',
    performed_by: '',
    notes: '',
  }
}

// eslint-disable-next-line react-refresh/only-export-components
export function maintenanceFormFromRecord(r: MaintenanceRecordWithDetails): MaintenanceFormData {
  return {
    equipment_id: r.equipment_id,
    maintenance_type: r.maintenance_type,
    reported_at: r.reported_at ? r.reported_at.slice(0, 10) : format(new Date(), 'yyyy-MM-dd'),
    problem_description: r.problem_description || '',
    maintenance_items: r.maintenance_items || '',
    performed_by: r.performed_by || '',
    notes: r.notes || '',
  }
}

const typeOptions: MaintenanceType[] = ['repair', 'maintenance']

interface MaintenanceFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: 'create' | 'edit'
  form: MaintenanceFormData
  onFormChange: (form: MaintenanceFormData) => void
  onSubmit: () => void
  isPending: boolean
  equipmentList: Equipment[]
}

export function MaintenanceFormDialog({
  open,
  onOpenChange,
  mode,
  form,
  onFormChange,
  onSubmit,
  isPending,
  equipmentList,
}: MaintenanceFormDialogProps) {
  const isCreate = mode === 'create'
  const isRepair = form.maintenance_type === 'repair'

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>
            {isCreate ? '新增' : '編輯'}維修/保養紀錄
          </DialogTitle>
          <DialogDescription>
            {isCreate ? '填寫維修或定期保養的相關資訊' : '修改紀錄內容'}
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          {/* 設備 + 類型 */}
          <div className="grid grid-cols-2 gap-4">
            <FormField label="設備" htmlFor="maint-equipment" required>
              <Select
                value={form.equipment_id}
                onValueChange={(v) => onFormChange({ ...form, equipment_id: v })}
                disabled={!isCreate}
              >
                <SelectTrigger id="maint-equipment">
                  <SelectValue placeholder="選擇設備" />
                </SelectTrigger>
                <SelectContent>
                  {equipmentList.map((e) => (
                    <SelectItem key={e.id} value={e.id}>
                      {e.name}{e.serial_number ? ` (${e.serial_number})` : ''}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </FormField>

            <FormField label="類型" htmlFor="maint-type" required>
              <Select
                value={form.maintenance_type}
                onValueChange={(v) => onFormChange({ ...form, maintenance_type: v as MaintenanceType })}
              >
                <SelectTrigger id="maint-type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {typeOptions.map((t) => (
                    <SelectItem key={t} value={t}>
                      {MAINTENANCE_TYPE_LABELS[t]}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </FormField>
          </div>

          {/* 報修/報保日期 */}
          <FormField label="報修日期" htmlFor="maint-reported-at" required>
            <Input
              id="maint-reported-at"
              type="date"
              value={form.reported_at}
              onChange={(e) => onFormChange({ ...form, reported_at: e.target.value })}
            />
          </FormField>

          {/* 維修：問題描述 / 保養：保養項目 */}
          {isRepair ? (
            <FormField label="問題描述" htmlFor="maint-problem" required>
              <Textarea
                id="maint-problem"
                value={form.problem_description}
                onChange={(e) => onFormChange({ ...form, problem_description: e.target.value })}
                placeholder="描述設備故障或異常情況"
                rows={3}
              />
            </FormField>
          ) : (
            <FormField label="保養項目" htmlFor="maint-items" required>
              <Textarea
                id="maint-items"
                value={form.maintenance_items}
                onChange={(e) => onFormChange({ ...form, maintenance_items: e.target.value })}
                placeholder="列出保養項目與內容"
                rows={3}
              />
            </FormField>
          )}

          {/* 執行人員 */}
          <FormField label="執行人員" htmlFor="maint-performer">
            <Input
              id="maint-performer"
              value={form.performed_by}
              onChange={(e) => onFormChange({ ...form, performed_by: e.target.value })}
              placeholder="維修/保養執行人員"
            />
          </FormField>

          {/* 備註 */}
          <FormField label="備註" htmlFor="maint-notes">
            <Textarea
              id="maint-notes"
              value={form.notes}
              onChange={(e) => onFormChange({ ...form, notes: e.target.value })}
              rows={2}
            />
          </FormField>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button
            onClick={onSubmit}
            disabled={isPending || !form.equipment_id}
          >
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {isCreate ? '新增' : '儲存'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
