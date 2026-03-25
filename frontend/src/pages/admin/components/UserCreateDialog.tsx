import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
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
import { Badge } from '@/components/ui/badge'
import { Loader2 } from 'lucide-react'
import {
  createUserSchema,
  type CreateUserFormData,
} from '@/lib/validation'
import type { CreateUserData } from '../hooks/useUserManagement'
import type { Role } from '@/lib/api'

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
  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors },
  } = useForm<CreateUserFormData>({
    resolver: zodResolver(createUserSchema),
    defaultValues: {
      email: '',
      password: '',
      display_name: '',
      phone: '',
      organization: '',
      role_ids: [],
      ...defaultValues,
    },
  })

  const roleIds = watch('role_ids')

  useEffect(() => {
    if (open) {
      reset({
        email: '',
        password: '',
        display_name: '',
        phone: '',
        organization: '',
        role_ids: [],
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
          <DialogTitle>新增使用者</DialogTitle>
          <DialogDescription>創建新的系統使用者帳號</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onValid)} className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="email">Email *</Label>
            <Input
              id="email"
              type="email"
              {...register('email')}
              placeholder="user@example.com"
            />
            {errors.email && (
              <p className="text-sm text-destructive">{errors.email.message}</p>
            )}
          </div>
          <div className="space-y-2">
            <Label htmlFor="password">密碼 *</Label>
            <Input
              id="password"
              type="password"
              {...register('password')}
              placeholder="至少 10 個字元"
            />
            {errors.password && (
              <p className="text-sm text-destructive">{errors.password.message}</p>
            )}
          </div>
          <div className="space-y-2">
            <Label htmlFor="display_name">顯示名稱 *</Label>
            <Input
              id="display_name"
              {...register('display_name')}
              placeholder="使用者名稱"
            />
            {errors.display_name && (
              <p className="text-sm text-destructive">{errors.display_name.message}</p>
            )}
          </div>
          <div className="space-y-2">
            <Label>指派角色</Label>
            <div className="flex flex-wrap gap-2 p-3 border rounded-md">
              {roles?.map((role) => (
                <Badge
                  key={role.id}
                  variant={roleIds.includes(role.id) ? 'default' : 'outline'}
                  className="cursor-pointer"
                  onClick={() => toggleRole(role.id)}
                >
                  {role.name}
                </Badge>
              ))}
            </div>
            {errors.role_ids && (
              <p className="text-sm text-destructive">{errors.role_ids.message}</p>
            )}
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              創建
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
