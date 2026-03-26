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

import type { Equipment } from '../types'

export interface DisposalFormData {
  equipment_id: string
  disposal_date: string
  reason: string
  disposal_method: string
  notes: string
}

export function emptyDisposalForm(): DisposalFormData {
  return {
    equipment_id: '',
    disposal_date: format(new Date(), 'yyyy-MM-dd'),
    reason: '',
    disposal_method: '',
    notes: '',
  }
}

const DISPOSAL_METHODS = [
  { value: 'recycle', label: '回收' },
  { value: 'scrap', label: '報廢銷毀' },
  { value: 'donate', label: '捐贈' },
  { value: 'sell', label: '變賣' },
  { value: 'other', label: '其他' },
] as const

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
  const canSubmit = form.equipment_id && form.reason.trim()

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>申請報廢</DialogTitle>
          <DialogDescription>填寫設備報廢申請資訊，送出後需管理員核准</DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          {/* 設備 */}
          <FormField label="設備" htmlFor="disposal-equipment" required>
            <Select
              value={form.equipment_id}
              onValueChange={(v) => onFormChange({ ...form, equipment_id: v })}
            >
              <SelectTrigger id="disposal-equipment">
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

          {/* 日期 + 處理方式 */}
          <div className="grid grid-cols-2 gap-4">
            <FormField label="報廢日期" htmlFor="disposal-date" required>
              <Input
                id="disposal-date"
                type="date"
                value={form.disposal_date}
                onChange={(e) => onFormChange({ ...form, disposal_date: e.target.value })}
              />
            </FormField>

            <FormField label="處理方式" htmlFor="disposal-method">
              <Select
                value={form.disposal_method}
                onValueChange={(v) => onFormChange({ ...form, disposal_method: v })}
              >
                <SelectTrigger id="disposal-method">
                  <SelectValue placeholder="選擇方式" />
                </SelectTrigger>
                <SelectContent>
                  {DISPOSAL_METHODS.map((m) => (
                    <SelectItem key={m.value} value={m.value}>
                      {m.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </FormField>
          </div>

          {/* 原因 */}
          <FormField label="報廢原因" htmlFor="disposal-reason" required>
            <Textarea
              id="disposal-reason"
              value={form.reason}
              onChange={(e) => onFormChange({ ...form, reason: e.target.value })}
              placeholder="說明設備報廢原因"
              rows={3}
            />
          </FormField>

          {/* 備註 */}
          <FormField label="備註" htmlFor="disposal-notes">
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
            取消
          </Button>
          <Button
            onClick={onSubmit}
            disabled={isPending || !canSubmit}
          >
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            送出申請
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
