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
  const { t } = useTranslation()
  const isCreate = mode === 'create'
  const title = isCreate
    ? t('admin.trainingFormDialog.createTitle')
    : t('admin.trainingFormDialog.editTitle')
  const description = isCreate
    ? t('admin.trainingFormDialog.createDescription')
    : t('admin.trainingFormDialog.editDescription')
  const submitLabel = isCreate ? t('common.create') : t('common.save')

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
              <Label>{t('admin.trainingFormDialog.personRequired')}</Label>
              <Select
                value={form.user_id}
                onValueChange={(v) => setForm({ ...form, user_id: v })}
              >
                <SelectTrigger>
                  <SelectValue placeholder={t('admin.trainingFormDialog.personPlaceholder')} />
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
              {t('admin.trainingFormDialog.personLabel')}
              {currentUser?.display_name || currentUser?.email}
            </div>
          ) : editingRecord ? (
            <div className="text-sm text-muted-foreground">
              {t('admin.trainingFormDialog.personLabel')}
              {editingRecord.user_name || editingRecord.user_email}
            </div>
          ) : null}

          <div>
            <Label>{t('admin.trainingFormDialog.courseNameRequired')}</Label>
            <Input
              value={form.course_name}
              onChange={(e) => setForm({ ...form, course_name: e.target.value })}
              placeholder={isCreate ? t('admin.trainingFormDialog.courseNamePlaceholder') : undefined}
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>{t('admin.trainingFormDialog.completedAtRequired')}</Label>
              <Input
                type="date"
                value={form.completed_at}
                onChange={(e) => setForm({ ...form, completed_at: e.target.value })}
              />
            </div>
            <div>
              <Label>{t('admin.trainingFormDialog.expiresAt')}</Label>
              <Input
                type="date"
                value={form.expires_at}
                onChange={(e) => setForm({ ...form, expires_at: e.target.value })}
                placeholder={isCreate ? t('admin.trainingFormDialog.optional') : undefined}
              />
            </div>
          </div>
          <div>
            <Label>{t('admin.trainingFormDialog.notes')}</Label>
            <Input
              value={form.notes}
              onChange={(e) => setForm({ ...form, notes: e.target.value })}
              placeholder={isCreate ? t('admin.trainingFormDialog.optional') : undefined}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
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
