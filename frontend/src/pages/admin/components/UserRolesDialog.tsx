import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Shield, Loader2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { CreateUserData } from '../hooks/useUserManagement'
import type { User, Role } from '@/lib/api'

interface UserRolesDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  selectedUser: User | null
  formData: CreateUserData
  roles: Role[] | undefined
  isPending: boolean
  onSubmit: () => void
  toggleRole: (roleId: string) => void
}

export function UserRolesDialog({
  open,
  onOpenChange,
  selectedUser,
  formData,
  roles,
  isPending,
  onSubmit,
  toggleRole,
}: UserRolesDialogProps) {
  const { t } = useTranslation()
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t('admin.userRolesDialog.title')}</DialogTitle>
          <DialogDescription>
            {t('admin.userRolesDialog.description', { name: selectedUser?.display_name })}
          </DialogDescription>
        </DialogHeader>
        <div className="py-4">
          <div className="space-y-2 max-h-[360px] overflow-y-auto">
            {roles?.map((role) => {
              const selected = formData.role_ids.includes(role.id)
              return (
                <button
                  key={role.id}
                  type="button"
                  role="checkbox"
                  aria-checked={selected}
                  aria-label={t('admin.userRolesDialog.toggleRoleAria', { role: role.name })}
                  className={`w-full text-left p-3 border rounded-md cursor-pointer transition-colors ${
                    selected ? 'border-primary bg-primary/5' : 'hover:bg-muted'
                  }`}
                  onClick={() => toggleRole(role.id)}
                >
                  <div className="flex items-center gap-2">
                    <Shield
                      className={`h-4 w-4 ${
                        selected ? 'text-primary' : 'text-muted-foreground'
                      }`}
                    />
                    <span className="font-medium">{role.name}</span>
                    <span className="text-xs text-muted-foreground">({role.code})</span>
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">
                    {t('admin.userRolesDialog.permissionCount', { count: role.permissions.length })}
                  </p>
                </button>
              )
            })}
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={onSubmit} disabled={isPending}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {t('common.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
