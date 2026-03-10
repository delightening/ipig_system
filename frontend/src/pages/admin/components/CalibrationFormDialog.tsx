/**
 * 校正紀錄新增／編輯共用 Dialog
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

import type { Equipment, CalibrationForm, CalibrationWithEquipment } from '../types'

interface CalibrationFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: 'create' | 'edit'
  form: CalibrationForm
  onFormChange: (form: CalibrationForm) => void
  onSubmit: () => void
  isPending: boolean
  /** 設備清單，供新增時選擇 */
  equipmentList: Equipment[]
  /** 編輯模式下顯示的設備名稱 */
  editingCalib: CalibrationWithEquipment | null
}

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

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isCreate ? '新增校正紀錄' : '編輯校正紀錄'}</DialogTitle>
          {isCreate && <DialogDescription>填寫校正日期與結果</DialogDescription>}
          {!isCreate && editingCalib && (
            <DialogDescription>設備：{editingCalib.equipment_name}</DialogDescription>
          )}
        </DialogHeader>
        <div className="grid gap-4 py-4">
          {/* 新增模式才顯示設備選擇 */}
          {isCreate && (
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
                      {e.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}
          <div className={isCreate ? 'grid grid-cols-2 gap-4' : undefined}>
            <div>
              <Label>校正日期 *</Label>
              <Input
                type="date"
                value={form.calibrated_at}
                onChange={(e) => onFormChange({ ...form, calibrated_at: e.target.value })}
              />
            </div>
            <div>
              <Label>下次校正</Label>
              <Input
                type="date"
                value={form.next_due_at}
                onChange={(e) => onFormChange({ ...form, next_due_at: e.target.value })}
              />
            </div>
          </div>
          <div>
            <Label>結果</Label>
            <Input
              value={form.result}
              onChange={(e) => onFormChange({ ...form, result: e.target.value })}
              placeholder={isCreate ? '例：合格' : undefined}
            />
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
