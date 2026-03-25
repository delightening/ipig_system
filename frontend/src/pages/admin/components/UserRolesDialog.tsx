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
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>管理角色</DialogTitle>
          <DialogDescription>
            為 {selectedUser?.display_name} 指派角色
          </DialogDescription>
        </DialogHeader>
        <div className="py-4">
          <div className="space-y-2 max-h-[360px] overflow-y-auto">
            {roles?.map((role) => (
              <div
                key={role.id}
                className={`p-3 border rounded-md cursor-pointer transition-colors ${
                  formData.role_ids.includes(role.id)
                    ? 'border-primary bg-primary/5'
                    : 'hover:bg-muted'
                }`}
                onClick={() => toggleRole(role.id)}
              >
                <div className="flex items-center gap-2">
                  <Shield
                    className={`h-4 w-4 ${
                      formData.role_ids.includes(role.id) ? 'text-primary' : 'text-muted-foreground'
                    }`}
                  />
                  <span className="font-medium">{role.name}</span>
                  <span className="text-xs text-muted-foreground">({role.code})</span>
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  {role.permissions.length} 個權限
                </p>
              </div>
            ))}
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button onClick={onSubmit} disabled={isPending}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            儲存
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
