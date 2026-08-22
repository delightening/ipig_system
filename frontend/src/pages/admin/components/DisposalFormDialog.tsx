/**
 * 報廢申請 Dialog
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
import { useTranslation } from 'react-i18next'

import type { Equipment } from '../types'

export interface DisposalFormData {
  equipment_id: string
  disposal_date: string
  reason: string
  disposal_method: string
  notes: string
}

// eslint-disable-next-line react-refresh/only-export-components
export function emptyDisposalForm(): DisposalFormData {
  return {
    equipment_id: '',
    disposal_date: format(new Date(), 'yyyy-MM-dd'),
    reason: '',
    disposal_method: '',
    notes: '',
  }
}

const DISPOSAL_METHOD_VALUES = ['recycle', 'scrap', 'donate', 'sell', 'other'] as const

interface DisposalFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  form: DisposalFormData
  onFormChange: (form: DisposalFormData) => void
  onSubmit: () => void
  isPending: boolean
  equipmentList: Equipment[]
}

export function DisposalFormDialog({
  open,
  onOpenChange,
  form,
  onFormChange,
  onSubmit,
  isPending,
  equipmentList,
}: DisposalFormDialogProps) {
  const { t } = useTranslation()
  const canSubmit = form.equipment_id && form.reason.trim()

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('admin.disposalFormDialog.title')}</DialogTitle>
          <DialogDescription>{t('admin.disposalFormDialog.description')}</DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          {/* 設備 */}
          <FormField label={t('admin.disposalFormDialog.equipmentLabel')} htmlFor="disposal-equipment" required>
            <Select
              value={form.equipment_id}
              onValueChange={(v) => onFormChange({ ...form, equipment_id: v })}
            >
              <SelectTrigger id="disposal-equipment">
                <SelectValue placeholder={t('admin.disposalFormDialog.equipmentPlaceholder')} />
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

          {/* 日期 + 處理方式 */}
          <div className="grid grid-cols-2 gap-4">
            <FormField label={t('admin.disposalFormDialog.disposalDateLabel')} htmlFor="disposal-date" required>
              <Input
                id="disposal-date"
                type="date"
                value={form.disposal_date}
                onChange={(e) => onFormChange({ ...form, disposal_date: e.target.value })}
              />
            </FormField>

            <FormField label={t('admin.disposalFormDialog.methodLabel')} htmlFor="disposal-method">
              <Select
                value={form.disposal_method}
                onValueChange={(v) => onFormChange({ ...form, disposal_method: v })}
              >
                <SelectTrigger id="disposal-method">
                  <SelectValue placeholder={t('admin.disposalFormDialog.methodPlaceholder')} />
                </SelectTrigger>
                <SelectContent>
                  {DISPOSAL_METHOD_VALUES.map((value) => (
                    <SelectItem key={value} value={value}>
                      {t(`admin.disposalFormDialog.methods.${value}`)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </FormField>
          </div>

          {/* 原因 */}
          <FormField label={t('admin.disposalFormDialog.reasonLabel')} htmlFor="disposal-reason" required>
            <Textarea
              id="disposal-reason"
              value={form.reason}
              onChange={(e) => onFormChange({ ...form, reason: e.target.value })}
              placeholder={t('admin.disposalFormDialog.reasonPlaceholder')}
              rows={3}
            />
          </FormField>

          {/* 備註 */}
          <FormField label={t('admin.disposalFormDialog.notesLabel')} htmlFor="disposal-notes">
            <Textarea
              id="disposal-notes"
              value={form.notes}
              onChange={(e) => onFormChange({ ...form, notes: e.target.value })}
              rows={2}
            />
          </FormField>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button
            onClick={onSubmit}
            disabled={isPending || !canSubmit}
          >
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {t('admin.disposalFormDialog.submit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
