/**
 * 校正/確效/查核紀錄新增／編輯共用 Dialog
 */
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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

import type { Equipment, CalibrationForm, CalibrationWithEquipment, CalibrationType, ValidationPhase } from '../types'
import { CALIBRATION_TYPE_LABELS, VALIDATION_PHASE_LABELS } from '../types'

interface CalibrationFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: 'create' | 'edit'
  form: CalibrationForm
  onFormChange: (form: CalibrationForm) => void
  onSubmit: () => void
  isPending: boolean
  equipmentList: Equipment[]
  editingCalib: CalibrationWithEquipment | null
}

const calTypeOptions: CalibrationType[] = ['calibration', 'validation', 'inspection']
const validationPhaseOptions: ValidationPhase[] = ['IQ', 'OQ', 'PQ']

export function CalibrationFormDialog({
  open,
  onOpenChange,
  mode,
  form,
  onFormChange,
  onSubmit,
  isPending,
  equipmentList,
  editingCalib,
}: CalibrationFormDialogProps) {
  const isCreate = mode === 'create'
  const isInspection = form.calibration_type === 'inspection'
  const isValidation = form.calibration_type === 'validation'
  const isCalibration = form.calibration_type === 'calibration'

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {isCreate ? '新增' : '編輯'}
            {CALIBRATION_TYPE_LABELS[form.calibration_type]}紀錄
          </DialogTitle>
          {isCreate && <DialogDescription>填寫執行日期與結果</DialogDescription>}
          {!isCreate && editingCalib && (
            <DialogDescription>
              設備：{editingCalib.equipment_name}
              {editingCalib.equipment_serial_number && ` (${editingCalib.equipment_serial_number})`}
            </DialogDescription>
          )}
        </DialogHeader>
        <div className="grid gap-4 py-4">
          {/* 新增模式：選設備和類型 */}
          {isCreate && (
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>設備 *</Label>
                <Select
                  value={form.equipment_id}
                  onValueChange={(v) => onFormChange({ ...form, equipment_id: v })}
                >
                  <SelectTrigger>
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
              </div>
              <div>
                <Label>類型 *</Label>
                <Select
                  value={form.calibration_type}
                  onValueChange={(v) =>
                    onFormChange({ ...form, calibration_type: v as CalibrationType })
                  }
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {calTypeOptions.map((t) => (
                      <SelectItem key={t} value={t}>
                        {CALIBRATION_TYPE_LABELS[t]}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          )}

          {/* 日期 */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>執行日期 *</Label>
              <Input
                type="date"
                value={form.calibrated_at}
                onChange={(e) => onFormChange({ ...form, calibrated_at: e.target.value })}
              />
            </div>
            <div>
              <Label>下次到期日</Label>
              <Input
                type="date"
                value={form.next_due_at}
                onChange={(e) => onFormChange({ ...form, next_due_at: e.target.value })}
              />
            </div>
          </div>

          {/* 結果 */}
          <div>
            <Label>結果</Label>
            <Input
              value={form.result}
              onChange={(e) => onFormChange({ ...form, result: e.target.value })}
              placeholder="例：合格 / 不合格"
            />
          </div>

          {/* ── 校正（calibration）特有欄位 ── */}
          {isCalibration && (
            <>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>校正證書編號</Label>
                  <Input
                    value={form.certificate_number}
                    onChange={(e) => onFormChange({ ...form, certificate_number: e.target.value })}
                    placeholder="ISO 17025 §7.8.4"
                  />
                </div>
                <div>
                  <Label>報告編號</Label>
                  <Input
                    value={form.report_number}
                    onChange={(e) => onFormChange({ ...form, report_number: e.target.value })}
                    placeholder="校正報告編號"
                  />
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>執行人員</Label>
                  <Input
                    value={form.performed_by}
                    onChange={(e) => onFormChange({ ...form, performed_by: e.target.value })}
                    placeholder="執行校正之人員"
                  />
                </div>
                <div>
                  <Label>量測不確定度</Label>
                  <Input
                    value={form.measurement_uncertainty}
                    onChange={(e) => onFormChange({ ...form, measurement_uncertainty: e.target.value })}
                    placeholder="例：±0.01 mg (k=2)"
                  />
                </div>
              </div>
              <div>
                <Label>合格判定標準</Label>
                <Input
                  value={form.acceptance_criteria}
                  onChange={(e) => onFormChange({ ...form, acceptance_criteria: e.target.value })}
                  placeholder="例：示值誤差 ≤ ±0.5%"
                />
              </div>
            </>
          )}

          {/* ── 確效（validation）特有欄位 ── */}
          {isValidation && (
            <>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>確效階段</Label>
                  <Select
                    value={form.validation_phase || '_none'}
                    onValueChange={(v) =>
                      onFormChange({ ...form, validation_phase: v === '_none' ? '' : (v as ValidationPhase) })
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="選擇確效階段" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="_none">未指定</SelectItem>
                      {validationPhaseOptions.map((p) => (
                        <SelectItem key={p} value={p}>
                          {VALIDATION_PHASE_LABELS[p]}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <Label>確效方案編號</Label>
                  <Input
                    value={form.protocol_number}
                    onChange={(e) => onFormChange({ ...form, protocol_number: e.target.value })}
                    placeholder="VP 方案編號"
                  />
                </div>
              </div>
              <div>
                <Label>報告編號</Label>
                <Input
                  value={form.report_number}
                  onChange={(e) => onFormChange({ ...form, report_number: e.target.value })}
                  placeholder="確效報告編號"
                />
              </div>
            </>
          )}

          {/* ── 查核（inspection）特有欄位 ── */}
          {isInspection && (
            <div>
              <Label>查核人員</Label>
              <Input
                value={form.inspector}
                onChange={(e) => onFormChange({ ...form, inspector: e.target.value })}
                placeholder="查核人員姓名"
              />
            </div>
          )}

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
