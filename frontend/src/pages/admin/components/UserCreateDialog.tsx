import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
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
import { FormField } from '@/components/ui/form-field'
import { badgeVariants } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import { Loader2 } from 'lucide-react'
import { StaffAffiliationField } from '@/components/admin/StaffAffiliationField'
import type { CreateUserData } from '../hooks/useUserManagement'
import type { Role } from '@/lib/api'

interface CreateUserFormData {
  email: string
  password: string
  display_name: string
  phone: string
  organization: string
  role_ids: string[]
  /** null = 尚未選擇；送出前必填 */
  is_internal: boolean | null
}

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

interface UserCreateDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  roles: Role[] | undefined
  isPending: boolean
  onSubmit: (data: CreateUserData) => void
  defaultValues?: Partial<CreateUserData>
}

export function UserCreateDialog({
  open,
  onOpenChange,
  roles,
  isPending,
  onSubmit,
  defaultValues,
}: UserCreateDialogProps) {
  const { t } = useTranslation()
  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors },
  } = useForm<CreateUserFormData>({
    defaultValues: {
      email: '',
      password: '',
      display_name: '',
      phone: '',
      organization: '',
      role_ids: [],
      is_internal: null,
      ...defaultValues,
    },
  })

  const roleIds = watch('role_ids')
  const isInternal = watch('is_internal')

  // 已選角色的 code，供身分欄位做預選判斷
  const selectedRoleCodes = (roles ?? [])
    .filter(r => (roleIds ?? []).includes(r.id))
    .map(r => r.code)

  useEffect(() => {
    if (open) {
      reset({
        email: '',
        password: '',
        display_name: '',
        phone: '',
        organization: '',
        role_ids: [],
        is_internal: null,
        ...defaultValues,
      })
    }
  }, [open, reset, defaultValues])

  const toggleRole = (roleId: string) => {
    const current = roleIds || []
    const next = current.includes(roleId)
      ? current.filter((id) => id !== roleId)
      : [...current, roleId]
    setValue('role_ids', next, { shouldValidate: true })
  }

  const onValid = (data: CreateUserFormData) => {
    onSubmit({
      ...data,
      role_ids: data.role_ids,
      // 斷言而非 `?? true`：**刻意不補預設值**——補了等於把「靜默猜測」留一條
      // 後路，而且與 InvitationCreateDialog 原本的 `?? false` 方向相反，正是本 PR
      // 要修的「兩條建立路徑做出相反假設」。不變式由兩道防線保證：① 上方 validate
      // 要求 typeof === 'boolean'；② 後端 `CreateUserRequest.is_internal` 無 serde
      // default，真的漏帶會回 400。加執行期 if 只會多一條走不到、也量不到的分支。
      is_internal: data.is_internal as boolean,
      entry_date: '',
      position: '',
      aup_roles: [],
      years_experience: 0,
      trainings: [],
    })
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('admin.userCreateDialog.title')}</DialogTitle>
          <DialogDescription>{t('admin.userCreateDialog.description')}</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onValid)} className="space-y-4 py-4">
          <FormField label={t('admin.userCreateDialog.emailLabel')} required error={errors.email?.message} htmlFor="email">
            <Input
              id="email"
              type="email"
              {...register('email', {
                required: t('admin.userCreateDialog.emailInvalid'),
                pattern: { value: EMAIL_PATTERN, message: t('admin.userCreateDialog.emailInvalid') },
              })}
              placeholder="user@example.com"
            />
          </FormField>
          <FormField label={t('admin.userCreateDialog.passwordLabel')} required error={errors.password?.message} htmlFor="password">
            <Input
              id="password"
              type="password"
              {...register('password', {
                required: t('admin.userCreateDialog.passwordMinLength'),
                minLength: { value: 10, message: t('admin.userCreateDialog.passwordMinLength') },
              })}
              placeholder={t('admin.userCreateDialog.passwordPlaceholder')}
            />
          </FormField>
          <FormField label={t('admin.userCreateDialog.displayNameLabel')} required error={errors.display_name?.message} htmlFor="display_name">
            <Input
              id="display_name"
              {...register('display_name', { required: t('admin.userCreateDialog.displayNameRequired') })}
              placeholder={t('admin.userCreateDialog.displayNamePlaceholder')}
            />
          </FormField>
          <FormField label={t('admin.userCreateDialog.rolesLabel')} error={errors.role_ids?.message}>
            <input type="hidden" {...register('role_ids', {
              validate: v => (v && v.length > 0) || t('admin.userCreateDialog.rolesRequired'),
            })} />
            <div className="flex flex-wrap gap-2 p-3 border rounded-md">
              {roles?.map((role) => {
                const selected = roleIds.includes(role.id)
                return (
                  <button
                    key={role.id}
                    type="button"
                    className={cn(
                      badgeVariants({ variant: selected ? 'default' : 'outline' }),
                      'cursor-pointer',
                    )}
                    aria-pressed={selected}
                    onClick={() => toggleRole(role.id)}
                  >
                    {role.name}
                  </button>
                )
              })}
            </div>
          </FormField>
          {/* 身分二擇一。後端建立使用者時 is_internal 預設 true，先前沒有任何
              UI 會問，所有從這裡建的帳號一律變成內部人員——這裡補上必選。 */}
          <input
            type="hidden"
            {...register('is_internal', {
              // 用 typeof 而非 `v !== null`：後者放行 undefined，而下方送出時
              // 原本的 `?? true` 會把它靜默補成「內部人員」——與 InvitationCreateDialog
              // 原本的 `?? false` 方向相反，正是本 PR 要修的那個「兩條路徑相反假設」。
              validate: v =>
                typeof v === 'boolean' || '請選擇這個人是本場受僱人員或外部人員',
            })}
          />
          <StaffAffiliationField
            value={isInternal ?? null}
            onChange={next => setValue('is_internal', next, { shouldValidate: true })}
            selectedRoleCodes={selectedRoleCodes}
            error={errors.is_internal?.message}
          />
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              {t('common.cancel')}
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('common.create')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
