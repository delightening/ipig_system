/**
 * 維修/保養紀錄新增／編輯 Dialog
 *
 * 新增模式：設備、類型、報修日期、問題描述/保養項目、執行人員、備註
 * 編輯模式：額外顯示完修日期、維修內容、維修廠商 + 完成回報/無法維修按鈕
 */
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { FormField } from '@/components/ui/form-field'
import { StatusBadge } from '@/components/ui/status-badge'
import type { StatusVariant } from '@/components/ui/status-badge'
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
import { AlertTriangle, CheckCircle2, Loader2 } from 'lucide-react'
import { format } from 'date-fns'

import type { Equipment, MaintenanceType, MaintenanceStatus, MaintenanceRecordWithDetails } from '../types'
import { MAINTENANCE_TYPE_LABELS, MAINTENANCE_STATUS_LABELS } from '../types'

export interface MaintenanceFormData {
  equipment_id: string
  maintenance_type: MaintenanceType
  reported_at: string
  problem_description: string
  maintenance_items: string
  performed_by: string
  notes: string
  status: MaintenanceStatus | ''
  completed_at: string
  repair_content: string
  repair_partner_id: string
}

interface Partner {
  id: string
  name: string
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
    status: '',
    completed_at: '',
    repair_content: '',
    repair_partner_id: '',
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
    status: r.status,
    completed_at: r.completed_at ? r.completed_at.slice(0, 10) : '',
    repair_content: r.repair_content || '',
    repair_partner_id: r.repair_partner_id || '',
  }
}

const typeOptions: MaintenanceType[] = ['repair', 'maintenance']

const STATUS_VARIANT: Record<string, StatusVariant> = {
  pending: 'neutral',
  in_progress: 'warning',
  pending_review: 'info',
  completed: 'success',
  unrepairable: 'error',
}

interface MaintenanceFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: 'create' | 'edit'
  form: MaintenanceFormData
  onFormChange: (form: MaintenanceFormData) => void
  onSubmit: () => void
  /** 完成回報（狀態 → completed，後端攔截為 pending_review） */
  onSubmitComplete?: () => void
  /** 標記無法維修 */
  onSubmitUnrepairable?: () => void
  isPending: boolean
  equipmentList: Equipment[]
  partners?: Partner[]
}

export function MaintenanceFormDialog({
  open,
  onOpenChange,
  mode,
  form,
  onFormChange,
  onSubmit,
  onSubmitComplete,
  onSubmitUnrepairable,
  isPending,
  equipmentList,
  partners = [],
}: MaintenanceFormDialogProps) {
  const isCreate = mode === 'create'
  const isRepair = form.maintenance_type === 'repair'
  const canReport = !isCreate && form.status === 'pending'

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {isCreate ? '新增' : '編輯'}維修/保養紀錄
          </DialogTitle>
          <DialogDescription>
            {isCreate ? '填寫維修或定期保養的相關資訊' : '修改紀錄內容，或完成回報送交驗收'}
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          {/* 目前狀態（編輯模式顯示） */}
          {!isCreate && form.status && (
            <div className="flex items-center gap-2 text-sm">
              <span className="text-muted-foreground">目前狀態：</span>
              <StatusBadge variant={STATUS_VARIANT[form.status] || 'neutral'}>
                {MAINTENANCE_STATUS_LABELS[form.status as MaintenanceStatus] ?? form.status}
              </StatusBadge>
            </div>
          )}

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
                disabled={!isCreate}
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

          {/* 報修日期 + 完修日期 */}
          <div className={!isCreate ? 'grid grid-cols-2 gap-4' : ''}>
            <FormField label="報修日期" htmlFor="maint-reported-at" required>
              <Input
                id="maint-reported-at"
                type="date"
                value={form.reported_at}
                onChange={(e) => onFormChange({ ...form, reported_at: e.target.value })}
              />
            </FormField>

            {!isCreate && (
              <FormField label="完修日期" htmlFor="maint-completed-at">
                <Input
                  id="maint-completed-at"
                  type="date"
                  value={form.completed_at}
                  onChange={(e) => onFormChange({ ...form, completed_at: e.target.value })}
                />
              </FormField>
            )}
          </div>

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

          {/* 編輯模式 + 維修類型：維修內容 + 廠商 */}
          {!isCreate && isRepair && (
            <>
              <FormField label="維修內容" htmlFor="maint-repair-content">
                <Textarea
                  id="maint-repair-content"
                  value={form.repair_content}
                  onChange={(e) => onFormChange({ ...form, repair_content: e.target.value })}
                  placeholder="實際維修內容、更換零件等"
                  rows={3}
                />
              </FormField>

              {partners.length > 0 && (
                <FormField label="維修廠商" htmlFor="maint-partner">
                  <Select
                    value={form.repair_partner_id || '_none'}
                    onValueChange={(v) => onFormChange({ ...form, repair_partner_id: v === '_none' ? '' : v })}
                  >
                    <SelectTrigger id="maint-partner">
                      <SelectValue placeholder="選擇廠商（選填）" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="_none">— 無 —</SelectItem>
                      {partners.map((p) => (
                        <SelectItem key={p.id} value={p.id}>{p.name}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </FormField>
              )}
            </>
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

        <DialogFooter className="flex-col sm:flex-row gap-2">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <div className="flex gap-2 ml-auto">
            {canReport && onSubmitUnrepairable && (
              <Button
                variant="destructive"
                onClick={onSubmitUnrepairable}
                disabled={isPending}
              >
                <AlertTriangle className="h-4 w-4 mr-1" />
                無法維修
              </Button>
            )}
            <Button
              variant="outline"
              onClick={onSubmit}
              disabled={isPending || !form.equipment_id}
            >
              {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {isCreate ? '新增' : '儲存'}
            </Button>
            {canReport && onSubmitComplete && (
              <Button
                onClick={onSubmitComplete}
                disabled={isPending || !form.equipment_id}
              >
                <CheckCircle2 className="h-4 w-4 mr-1" />
                完成回報
              </Button>
            )}
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
