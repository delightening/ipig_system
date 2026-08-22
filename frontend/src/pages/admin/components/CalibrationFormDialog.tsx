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
import { useTranslation } from 'react-i18next'

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
  const { t } = useTranslation()
  const isCreate = mode === 'create'
  const isInspection = form.calibration_type === 'inspection'
  const isValidation = form.calibration_type === 'validation'
  const isCalibration = form.calibration_type === 'calibration'

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {isCreate ? t('common.create') : t('common.edit')}
            {t(CALIBRATION_TYPE_LABELS[form.calibration_type])}
            {t('admin.calibrationFormDialog.recordSuffix')}
          </DialogTitle>
          {isCreate && <DialogDescription>{t('admin.calibrationFormDialog.createDescription')}</DialogDescription>}
          {!isCreate && editingCalib && (
            <DialogDescription>
              {t('admin.calibrationFormDialog.equipmentLabel')}{editingCalib.equipment_name}
              {editingCalib.equipment_serial_number && ` (${editingCalib.equipment_serial_number})`}
            </DialogDescription>
          )}
        </DialogHeader>
        <div className="grid gap-4 py-4">
          {/* 新增模式：選設備和類型 */}
          {isCreate && (
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>{t('admin.calibrationFormDialog.equipmentRequired')}</Label>
                <Select
                  value={form.equipment_id}
                  onValueChange={(v) => onFormChange({ ...form, equipment_id: v })}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('admin.calibrationFormDialog.equipmentPlaceholder')} />
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
                <Label>{t('admin.calibrationFormDialog.typeRequired')}</Label>
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
                    {calTypeOptions.map((ct) => (
                      <SelectItem key={ct} value={ct}>
                        {t(CALIBRATION_TYPE_LABELS[ct])}
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
              <Label>{t('admin.calibrationFormDialog.calibratedAtRequired')}</Label>
              <Input
                type="date"
                value={form.calibrated_at}
                onChange={(e) => onFormChange({ ...form, calibrated_at: e.target.value })}
              />
            </div>
            <div>
              <Label>{t('admin.calibrationFormDialog.nextDueAt')}</Label>
              <Input
                type="date"
                value={form.next_due_at}
                onChange={(e) => onFormChange({ ...form, next_due_at: e.target.value })}
              />
            </div>
          </div>

          {/* 結果 */}
          <div>
            <Label>{t('admin.calibrationFormDialog.result')}</Label>
            <Input
              value={form.result}
              onChange={(e) => onFormChange({ ...form, result: e.target.value })}
              placeholder={t('admin.calibrationFormDialog.resultPlaceholder')}
            />
          </div>

          {/* ── 校正（calibration）特有欄位 ── */}
          {isCalibration && (
            <>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>{t('admin.calibrationFormDialog.certificateNumber')}</Label>
                  <Input
                    value={form.certificate_number}
                    onChange={(e) => onFormChange({ ...form, certificate_number: e.target.value })}
                    placeholder="ISO 17025 §7.8.4"
                  />
                </div>
                <div>
                  <Label>{t('admin.calibrationFormDialog.reportNumber')}</Label>
                  <Input
                    value={form.report_number}
                    onChange={(e) => onFormChange({ ...form, report_number: e.target.value })}
                    placeholder={t('admin.calibrationFormDialog.calibrationReportNumberPlaceholder')}
                  />
                </div>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>{t('admin.calibrationFormDialog.performedBy')}</Label>
                  <Input
                    value={form.performed_by}
                    onChange={(e) => onFormChange({ ...form, performed_by: e.target.value })}
                    placeholder={t('admin.calibrationFormDialog.performedByPlaceholder')}
                  />
                </div>
                <div>
                  <Label>{t('admin.calibrationFormDialog.measurementUncertainty')}</Label>
                  <Input
                    value={form.measurement_uncertainty}
                    onChange={(e) => onFormChange({ ...form, measurement_uncertainty: e.target.value })}
                    placeholder={t('admin.calibrationFormDialog.measurementUncertaintyPlaceholder')}
                  />
                </div>
              </div>
              <div>
                <Label>{t('admin.calibrationFormDialog.acceptanceCriteria')}</Label>
                <Input
                  value={form.acceptance_criteria}
                  onChange={(e) => onFormChange({ ...form, acceptance_criteria: e.target.value })}
                  placeholder={t('admin.calibrationFormDialog.acceptanceCriteriaPlaceholder')}
                />
              </div>
            </>
          )}

          {/* ── 確效（validation）特有欄位 ── */}
          {isValidation && (
            <>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>{t('admin.calibrationFormDialog.validationPhase')}</Label>
                  <Select
                    value={form.validation_phase || '_none'}
                    onValueChange={(v) =>
                      onFormChange({ ...form, validation_phase: v === '_none' ? '' : (v as ValidationPhase) })
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder={t('admin.calibrationFormDialog.validationPhasePlaceholder')} />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="_none">{t('admin.calibrationFormDialog.unspecified')}</SelectItem>
                      {validationPhaseOptions.map((p) => (
                        <SelectItem key={p} value={p}>
                          {t(VALIDATION_PHASE_LABELS[p])}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <Label>{t('admin.calibrationFormDialog.protocolNumber')}</Label>
                  <Input
                    value={form.protocol_number}
                    onChange={(e) => onFormChange({ ...form, protocol_number: e.target.value })}
                    placeholder={t('admin.calibrationFormDialog.protocolNumberPlaceholder')}
                  />
                </div>
              </div>
              <div>
                <Label>{t('admin.calibrationFormDialog.reportNumber')}</Label>
                <Input
                  value={form.report_number}
                  onChange={(e) => onFormChange({ ...form, report_number: e.target.value })}
                  placeholder={t('admin.calibrationFormDialog.validationReportNumberPlaceholder')}
                />
              </div>
            </>
          )}

          {/* ── 查核（inspection）特有欄位 ── */}
          {isInspection && (
            <div>
              <Label>{t('admin.calibrationFormDialog.inspector')}</Label>
              <Input
                value={form.inspector}
                onChange={(e) => onFormChange({ ...form, inspector: e.target.value })}
                placeholder={t('admin.calibrationFormDialog.inspectorPlaceholder')}
              />
            </div>
          )}

          <div>
            <Label>{t('admin.calibrationFormDialog.notes')}</Label>
            <Input
              value={form.notes}
              onChange={(e) => onFormChange({ ...form, notes: e.target.value })}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={onSubmit} disabled={isPending}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {isCreate ? t('common.create') : t('common.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
