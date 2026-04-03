/**
 * 設備新增／編輯共用 Dialog
 */
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Loader2 } from 'lucide-react'

import type { EquipmentForm, CalibrationType, CalibrationCycle } from '../types'
import { CALIBRATION_TYPE_LABELS, CALIBRATION_CYCLE_LABELS } from '../types'

interface PartnerOption {
  id: string
  name: string
}

interface EquipmentFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: 'create' | 'edit'
  form: EquipmentForm
  onFormChange: (form: EquipmentForm) => void
  onSubmit: () => void
  isPending: boolean
  partnerOptions: PartnerOption[]
  selectedPartnerIds: string[]
  onPartnerIdsChange: (ids: string[]) => void
}

const calTypeOptions: CalibrationType[] = ['calibration', 'validation']
const cycleOptions: CalibrationCycle[] = ['monthly', 'quarterly', 'semi_annual', 'annual']

export function EquipmentFormDialog({
  open,
  onOpenChange,
  mode,
  form,
  onFormChange,
  onSubmit,
  isPending,
  partnerOptions,
  selectedPartnerIds,
  onPartnerIdsChange,
}: EquipmentFormDialogProps) {
  const isCreate = mode === 'create'

  const togglePartner = (partnerId: string) => {
    if (selectedPartnerIds.includes(partnerId)) {
      onPartnerIdsChange(selectedPartnerIds.filter((id) => id !== partnerId))
    } else {
      onPartnerIdsChange([...selectedPartnerIds, partnerId])
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{isCreate ? '新增設備' : '編輯設備'}</DialogTitle>
          {isCreate && <DialogDescription>填寫設備基本資料</DialogDescription>}
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div>
            <Label>名稱 *</Label>
            <Input
              value={form.name}
              onChange={(e) => onFormChange({ ...form, name: e.target.value })}
              placeholder={isCreate ? '例：電子天平' : undefined}
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>型號</Label>
              <Input
                value={form.model}
                onChange={(e) => onFormChange({ ...form, model: e.target.value })}
              />
            </div>
            <div>
              <Label>序號</Label>
              <Input
                value={form.serial_number}
                onChange={(e) => onFormChange({ ...form, serial_number: e.target.value })}
              />
            </div>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>位置</Label>
              <Input
                value={form.location}
                onChange={(e) => onFormChange({ ...form, location: e.target.value })}
              />
            </div>
            <div>
              <Label>部門</Label>
              <Input
                value={form.department}
                onChange={(e) => onFormChange({ ...form, department: e.target.value })}
                placeholder="責任部門/保管單位"
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>購買日期</Label>
              <Input
                type="date"
                value={form.purchase_date}
                onChange={(e) => onFormChange({ ...form, purchase_date: e.target.value })}
              />
            </div>
            <div>
              <Label>保固到期日</Label>
              <Input
                type="date"
                value={form.warranty_expiry}
                onChange={(e) => onFormChange({ ...form, warranty_expiry: e.target.value })}
              />
            </div>
          </div>

          {/* 廠商選擇 */}
          <div className="border-t pt-4 space-y-2">
            <Label>廠商</Label>
            {partnerOptions.length === 0 ? (
              <p className="text-sm text-muted-foreground">尚無供應商資料</p>
            ) : (
              <div className="max-h-32 overflow-y-auto rounded-md border p-2 space-y-1">
                {partnerOptions.map((p) => (
                  <label
                    key={p.id}
                    className="flex items-center gap-2 px-1 py-0.5 rounded hover:bg-muted/50 cursor-pointer text-sm"
                  >
                    <input
                      type="checkbox"
                      checked={selectedPartnerIds.includes(p.id)}
                      onChange={() => togglePartner(p.id)}
                      className="rounded border-border"
                    />
                    {p.name}
                  </label>
                ))}
              </div>
            )}
          </div>

          {/* 校正/確效設定 */}
          <div className="border-t pt-4 space-y-4">
            <p className="text-sm font-medium text-muted-foreground">校正/確效設定</p>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>校正/確效類型</Label>
                <Select
                  value={form.calibration_type || '_none'}
                  onValueChange={(v) =>
                    onFormChange({
                      ...form,
                      calibration_type: v === '_none' ? '' : (v as CalibrationType),
                    })
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder="不適用" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="_none">不適用</SelectItem>
                    {calTypeOptions.map((t) => (
                      <SelectItem key={t} value={t}>
                        {CALIBRATION_TYPE_LABELS[t]}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label>校正/確效週期</Label>
                <Select
                  value={form.calibration_cycle || '_none'}
                  onValueChange={(v) =>
                    onFormChange({
                      ...form,
                      calibration_cycle: v === '_none' ? '' : (v as CalibrationCycle),
                    })
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder="不適用" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="_none">不適用</SelectItem>
                    {cycleOptions.map((c) => (
                      <SelectItem key={c} value={c}>
                        {CALIBRATION_CYCLE_LABELS[c]}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>查核週期</Label>
                <Select
                  value={form.inspection_cycle || '_none'}
                  onValueChange={(v) =>
                    onFormChange({
                      ...form,
                      inspection_cycle: v === '_none' ? '' : (v as CalibrationCycle),
                    })
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder="不適用" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="_none">不適用</SelectItem>
                    {cycleOptions.map((c) => (
                      <SelectItem key={c} value={c}>
                        {CALIBRATION_CYCLE_LABELS[c]}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>

          <div>
            <Label>備註</Label>
            <Input
              value={form.notes}
              onChange={(e) => onFormChange({ ...form, notes: e.target.value })}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button onClick={onSubmit} disabled={isPending}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {isCreate ? '新增' : '儲存'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
