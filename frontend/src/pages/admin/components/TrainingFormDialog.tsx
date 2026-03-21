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
import type { TrainingRecordWithUser, TrainingForm, TrainingUser } from '../types/training'

interface TrainingFormDialogProps {
  mode: 'create' | 'edit'
  open: boolean
  onOpenChange: (open: boolean) => void
  form: TrainingForm
  setForm: (updater: TrainingForm | ((prev: TrainingForm) => TrainingForm)) => void
  onSubmit: () => void
  isPending: boolean
  canManageAll: boolean
  users: TrainingUser[]
  /** 目前登入使用者資訊（非管理員時顯示） */
  currentUser?: { display_name?: string; email: string } | null
  /** 編輯模式下的原始紀錄 */
  editingRecord?: TrainingRecordWithUser | null
}

export function TrainingFormDialog({
  mode,
  open,
  onOpenChange,
  form,
  setForm,
  onSubmit,
  isPending,
  canManageAll,
  users,
  currentUser,
  editingRecord,
}: TrainingFormDialogProps) {
  const isCreate = mode === 'create'
  const title = isCreate ? '新增訓練紀錄' : '編輯訓練紀錄'
  const description = isCreate
    ? '填寫課程名稱、完成日期與有效期限（選填）'
    : '修改課程名稱、完成日期與有效期限'
  const submitLabel = isCreate ? '新增' : '儲存'

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          {/* 人員欄位 */}
          {isCreate && canManageAll ? (
            <div>
              <Label>人員 *</Label>
              <Select
                value={form.user_id}
                onValueChange={(v) => setForm({ ...form, user_id: v })}
              >
                <SelectTrigger>
                  <SelectValue placeholder="選擇人員" />
                </SelectTrigger>
                <SelectContent>
                  {users.map((u) => (
                    <SelectItem key={u.id} value={u.id}>
                      {u.display_name || u.email}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          ) : isCreate ? (
            <div className="text-sm text-muted-foreground">
              人員：{currentUser?.display_name || currentUser?.email}
            </div>
          ) : editingRecord ? (
            <div className="text-sm text-muted-foreground">
              人員：{editingRecord.user_name || editingRecord.user_email}
            </div>
          ) : null}

          <div>
            <Label>課程名稱 *</Label>
            <Input
              value={form.course_name}
              onChange={(e) => setForm({ ...form, course_name: e.target.value })}
              placeholder={isCreate ? '例：實驗動物從業人員訓練' : undefined}
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>完成日期 *</Label>
              <Input
                type="date"
                value={form.completed_at}
                onChange={(e) => setForm({ ...form, completed_at: e.target.value })}
              />
            </div>
            <div>
              <Label>有效期限</Label>
              <Input
                type="date"
                value={form.expires_at}
                onChange={(e) => setForm({ ...form, expires_at: e.target.value })}
                placeholder={isCreate ? '選填' : undefined}
              />
            </div>
          </div>
          <div>
            <Label>備註</Label>
            <Input
              value={form.notes}
              onChange={(e) => setForm({ ...form, notes: e.target.value })}
              placeholder={isCreate ? '選填' : undefined}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button onClick={onSubmit} disabled={isPending}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {submitLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
