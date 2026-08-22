import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { FormField } from '@/components/ui/form-field'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { AUP_ROLE_OPTIONS } from '@/lib/constants'
import { onActivateKey } from '@/lib/a11y'
import { Loader2 } from 'lucide-react'

export interface EditUserFormData {
  email: string
  display_name: string
  /**
   * 內部員工旗標。
   *
   * 決定這個人算不算「場內同仁」——影響部門成員可選名單（`GET /hr/internal-users`
   * 硬性過濾 `is_internal = true`）、特休/加班/訓練等 HR 清單、以及入職日期欄位。
   * 建立帳號時後端預設 true，但經邀請流程進來的外部人員會是 false；
   * 若這個人後來轉為正職，先前**沒有任何 UI 可以改回來**，本欄位就是為此而加。
   */
  is_internal: boolean
  entry_date: string
  aup_roles: string[]
  trainings: { code: string; certificate_no: string; received_date: string }[]
}

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

interface UserEditDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  initialData: EditUserFormData
  isPending: boolean
  onSubmit: (data: EditUserFormData) => void
}

export function UserEditDialog({
  open,
  onOpenChange,
  initialData,
  isPending,
  onSubmit,
}: UserEditDialogProps) {
  const { t } = useTranslation()
  const {
    register,
    handleSubmit,
    watch,
    setValue,
    getValues,
    reset,
    formState: { errors },
  } = useForm<EditUserFormData>({
    defaultValues: initialData,
  })

  const trainings = watch('trainings') || []
  const aupRoles = watch('aup_roles') || []
  // 後端 UserResponse 一定會給 is_internal，但型別上是 optional；
  // 讀不到時以 true 為準（與後端建立帳號時的預設一致），避免把既有內部員工
  // 靜默降級成外部人員。
  const isInternal = watch('is_internal') ?? true

  const toggleAupRole = (value: string) => {
    const next = aupRoles.includes(value)
      ? aupRoles.filter((r) => r !== value)
      : [...aupRoles, value]
    setValue('aup_roles', next)
  }

  useEffect(() => {
    if (open) {
      reset(initialData)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, reset])

  const toggleTraining = (code: string) => {
    const exists = trainings.some((t) => t.code === code)
    const next = exists
      ? trainings.filter((t) => t.code !== code)
      : [...trainings, { code, certificate_no: '', received_date: '' }]
    setValue('trainings', next)
  }

  const updateTrainingField = (
    idx: number,
    field: 'certificate_no' | 'received_date',
    value: string,
  ) => {
    const updated = [...trainings]
    updated[idx] = { ...updated[idx], [field]: value }
    setValue('trainings', updated)
  }

  const onValid = () => {
    onSubmit(getValues())
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('admin.userEditDialog.title')}</DialogTitle>
          <DialogDescription>{t('admin.userEditDialog.description')}</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onValid)} className="space-y-4 py-4">
          <FormField label="Email" error={errors.email?.message} htmlFor="edit-email">
            <Input id="edit-email" type="email" {...register('email', {
              required: t('admin.userEditDialog.emailRequired'),
              pattern: { value: EMAIL_PATTERN, message: t('admin.userEditDialog.emailRequired') },
            })} />
          </FormField>
          <FormField label={t('admin.userEditDialog.displayNameLabel')} error={errors.display_name?.message} htmlFor="edit-display_name">
            <Input id="edit-display_name" {...register('display_name', { required: t('admin.userEditDialog.displayNameRequired') })} />
          </FormField>
          {/* Switch 是受控元件，RHF 的 register 綁不上，沿用本檔 aup_roles 的 watch/setValue 寫法 */}
          <div className="flex items-center justify-between gap-4 rounded-md border p-3">
            <div className="space-y-0.5">
              <Label htmlFor="edit-is_internal">{t('admin.userEditDialog.isInternalLabel')}</Label>
              <p className="text-xs text-muted-foreground">
                {t('admin.userEditDialog.isInternalHint')}
              </p>
            </div>
            <Switch
              id="edit-is_internal"
              checked={isInternal}
              onCheckedChange={(checked) => setValue('is_internal', checked)}
            />
          </div>
          <div className="border-t pt-4 mt-4">
            <h4 className="font-medium mb-3">{t('admin.userEditDialog.aupSectionTitle')}</h4>
            <FormField label={t('admin.userEditDialog.entryDateLabel')} htmlFor="edit-entry_date">
              <Input id="edit-entry_date" type="date" {...register('entry_date')} />
            </FormField>
            <div className="space-y-2 mt-3">
              <Label>{t('admin.userEditDialog.aupRolesLabel')}</Label>
              <div className="flex flex-wrap gap-2 p-3 border rounded-md">
                {AUP_ROLE_OPTIONS.map((role) => {
                  const selected = aupRoles.includes(role.value)
                  return (
                    <Badge
                      key={role.value}
                      role="checkbox"
                      tabIndex={0}
                      aria-checked={selected}
                      variant={selected ? 'default' : 'outline'}
                      className="cursor-pointer"
                      onClick={() => toggleAupRole(role.value)}
                      onKeyDown={onActivateKey(() => toggleAupRole(role.value))}
                    >
                      {t(`profile.roles.${role.key}`)}
                    </Badge>
                  )
                })}
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                {t('admin.userEditDialog.aupRolesHint')}
              </p>
            </div>
            <div className="space-y-2 mt-3">
              <Label>{t('admin.userEditDialog.trainingsLabel')}</Label>
              <div className="flex flex-wrap gap-2 p-3 border rounded-md">
                {['A', 'B', 'C', 'D', 'E', 'F'].map((code) => {
                  const selected = trainings.some((t) => t.code === code)
                  return (
                    <Badge
                      key={code}
                      role="checkbox"
                      tabIndex={0}
                      aria-checked={selected}
                      aria-label={t('admin.userEditDialog.toggleTrainingAria', { code })}
                      variant={selected ? 'default' : 'outline'}
                      className="cursor-pointer"
                      onClick={() => toggleTraining(code)}
                      onKeyDown={onActivateKey(() => toggleTraining(code))}
                    >
                      {code}
                    </Badge>
                  )
                })}
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                {t('admin.userEditDialog.trainingsHint')}
              </p>
              {trainings.length > 0 && (
                <div className="space-y-2 mt-3">
                  {trainings.map((training, idx) => (
                    <div key={training.code} className="flex gap-2 items-center">
                      <Badge variant="secondary">{training.code}</Badge>
                      <Input
                        placeholder={t('admin.userEditDialog.certificateNoPlaceholder')}
                        value={training.certificate_no}
                        onChange={(e) => updateTrainingField(idx, 'certificate_no', e.target.value)}
                        className="w-32"
                      />
                      <Input
                        type="date"
                        value={training.received_date}
                        onChange={(e) => updateTrainingField(idx, 'received_date', e.target.value)}
                        className="w-36"
                      />
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('common.save')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
