/**
 * 設備新增／編輯共用 Dialog
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
import { Loader2 } from 'lucide-react'

import type { EquipmentForm } from '../types'

interface EquipmentFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: 'create' | 'edit'
  form: EquipmentForm
  onFormChange: (form: EquipmentForm) => void
  onSubmit: () => void
  isPending: boolean
}

export function EquipmentFormDialog({
  open,
  onOpenChange,
  mode,
  form,
  onFormChange,
  onSubmit,
  isPending,
}: EquipmentFormDialogProps) {
  const isCreate = mode === 'create'

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
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
          <div>
            <Label>位置</Label>
            <Input
              value={form.location}
              onChange={(e) => onFormChange({ ...form, location: e.target.value })}
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
