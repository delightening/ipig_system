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
import { Shield } from 'lucide-react'
import { Loader2, Key, AlertTriangle } from 'lucide-react'
import { ConfirmPasswordModal } from '@/components/auth/ConfirmPasswordModal'
import type { CreateUserData } from '../hooks/useUserManagement'
import type { User, Role } from '@/lib/api'

interface UserCreateDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  formData: CreateUserData
  setFormData: (data: CreateUserData | ((prev: CreateUserData) => CreateUserData)) => void
  roles: Role[] | undefined
  isPending: boolean
  onSubmit: () => void
  toggleRole: (roleId: string) => void
}

export function UserCreateDialog({
  open,
  onOpenChange,
  formData,
  setFormData,
  roles,
  isPending,
  onSubmit,
  toggleRole,
}: UserCreateDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>新增使用者</DialogTitle>
          <DialogDescription>創建新的系統使用者帳號</DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="email">Email *</Label>
            <Input
              id="email"
              type="email"
              value={formData.email}
              onChange={(e) => setFormData({ ...formData, email: e.target.value })}
              placeholder="user@example.com"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="password">密碼 *</Label>
            <Input
              id="password"
              type="password"
              value={formData.password}
              onChange={(e) => setFormData({ ...formData, password: e.target.value })}
              placeholder="至少 6 個字元"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="display_name">顯示名稱 *</Label>
            <Input
              id="display_name"
              value={formData.display_name}
              onChange={(e) => setFormData({ ...formData, display_name: e.target.value })}
              placeholder="使用者名稱"
            />
          </div>
          <div className="space-y-2">
            <Label>指派角色</Label>
            <div className="flex flex-wrap gap-2 p-3 border rounded-md">
              {roles?.map((role) => (
                <Badge
                  key={role.id}
                  variant={formData.role_ids.includes(role.id) ? 'default' : 'outline'}
                  className="cursor-pointer"
                  onClick={() => toggleRole(role.id)}
                >
                  {role.name}
                </Badge>
              ))}
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button onClick={onSubmit} disabled={isPending}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            創建
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

interface UserEditDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  formData: CreateUserData
  setFormData: (data: CreateUserData | ((prev: CreateUserData) => CreateUserData)) => void
  isPending: boolean
  onSubmit: () => void
}

export function UserEditDialog({
  open,
  onOpenChange,
  formData,
  setFormData,
  isPending,
  onSubmit,
}: UserEditDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>編輯使用者</DialogTitle>
          <DialogDescription>修改使用者資訊</DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="edit-email">Email</Label>
            <Input
              id="edit-email"
              type="email"
              value={formData.email}
              onChange={(e) => setFormData({ ...formData, email: e.target.value })}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-display_name">顯示名稱</Label>
            <Input
              id="edit-display_name"
              value={formData.display_name}
              onChange={(e) => setFormData({ ...formData, display_name: e.target.value })}
            />
          </div>
          <div className="border-t pt-4 mt-4">
            <h4 className="font-medium mb-3">AUP 人員資料</h4>
            <div className="space-y-2 mb-4">
              <Label htmlFor="edit-entry_date">入職日期 (Entry Date)</Label>
              <Input
                id="edit-entry_date"
                type="date"
                value={formData.entry_date || ''}
                onChange={(e) => setFormData({ ...formData, entry_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>訓練/資格 (Trainings)</Label>
              <div className="flex flex-wrap gap-2 p-3 border rounded-md">
                {['A', 'B', 'C', 'D', 'E', 'F'].map((code) => (
                  <Badge
                    key={code}
                    variant={formData.trainings.some((t) => t.code === code) ? 'default' : 'outline'}
                    className="cursor-pointer"
                    onClick={() => {
                      const exists = formData.trainings.some((t) => t.code === code)
                      const newTrainings = exists
                        ? formData.trainings.filter((t) => t.code !== code)
                        : [
                            ...formData.trainings,
                            { code, certificate_no: '', received_date: '' },
                          ]
                      setFormData({ ...formData, trainings: newTrainings })
                    }}
                  >
                    {code}
                  </Badge>
                ))}
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                A.IACUC訓練班 B.IACUC研討會 C.輻射安全 D.生醫產業研習 E.動物法規管理班 F.其他
              </p>
              {formData.trainings.length > 0 && (
                <div className="space-y-2 mt-3">
                  {formData.trainings.map((training, idx) => (
                    <div key={training.code} className="flex gap-2 items-center">
                      <Badge variant="secondary">{training.code}</Badge>
                      <Input
                        placeholder="證書編號"
                        value={training.certificate_no}
                        onChange={(e) => {
                          const newTrainings = [...formData.trainings]
                          newTrainings[idx] = {
                            ...training,
                            certificate_no: e.target.value,
                          }
                          setFormData({ ...formData, trainings: newTrainings })
                        }}
                        className="w-32"
                      />
                      <Input
                        type="date"
                        value={training.received_date}
                        onChange={(e) => {
                          const newTrainings = [...formData.trainings]
                          newTrainings[idx] = {
                            ...training,
                            received_date: e.target.value,
                          }
                          setFormData({ ...formData, trainings: newTrainings })
                        }}
                        className="w-36"
                      />
                    </div>
                  ))}
                </div>
              )}
            </div>
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

interface UserDeleteDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  userToDelete: User | null
  showReauth: boolean
  onReauthOpenChange: (open: boolean) => void
  onConfirmDelete: () => void
  onReauthSubmit: (password: string) => Promise<void>
}

export function UserDeleteDialog({
  open,
  onOpenChange,
  userToDelete,
  showReauth,
  onReauthOpenChange,
  onConfirmDelete,
  onReauthSubmit,
}: UserDeleteDialogProps) {
  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-red-500" />
              確認刪除使用者
            </DialogTitle>
            <DialogDescription>
              此操作無法復原。確定要刪除使用者{' '}
              <span className="font-medium">{userToDelete?.display_name}</span>（
              {userToDelete?.email}）嗎？
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <div className="p-3 bg-red-50 border border-red-200 rounded-md">
              <p className="text-sm text-red-800">
                刪除後，該使用者將無法再登入系統。如果只是暫時停用，建議使用「停用」功能。
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={onConfirmDelete}
            >
              確認刪除
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <ConfirmPasswordModal
        open={showReauth}
        onOpenChange={onReauthOpenChange}
        title="確認刪除使用者"
        description={
          userToDelete
            ? `確定要刪除使用者「${userToDelete.display_name}」？此操作無法復原，請輸入您的登入密碼以確認。`
            : ''
        }
        onSubmit={onReauthSubmit}
      />
    </>
  )
}

interface UserResetPasswordDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  userToResetPassword: User | null
  reauthPassword: string
  setReauthPassword: (v: string) => void
  newPassword: string
  setNewPassword: (v: string) => void
  confirmNewPassword: string
  setConfirmNewPassword: (v: string) => void
  isPending: boolean
  onSubmit: () => void
  onClose: () => void
}

export function UserResetPasswordDialog({
  open,
  onOpenChange,
  userToResetPassword,
  reauthPassword,
  setReauthPassword,
  newPassword,
  setNewPassword,
  confirmNewPassword,
  setConfirmNewPassword,
  isPending,
  onSubmit,
  onClose,
}: UserResetPasswordDialogProps) {
  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        onOpenChange(o)
        if (!o) onClose()
      }}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Key className="h-5 w-5 text-orange-500" />
            重設使用者密碼
          </DialogTitle>
          <DialogDescription>
            為使用者{' '}
            <span className="font-medium">{userToResetPassword?.display_name}</span>（
            {userToResetPassword?.email}）設定新密碼。
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="p-3 bg-orange-50 border border-orange-200 rounded-md">
            <p className="text-sm text-orange-800">
              重設密碼後，該使用者需要使用新密碼重新登入。建議通知該使用者密碼已變更。
            </p>
          </div>
          <div className="space-y-2">
            <Label htmlFor="reset-reauth-password">您的登入密碼（確認身份）</Label>
            <Input
              id="reset-reauth-password"
              type="password"
              value={reauthPassword}
              onChange={(e) => setReauthPassword(e.target.value)}
              placeholder="請輸入您的密碼以確認此操作"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="reset-new-password">新密碼</Label>
            <Input
              id="reset-new-password"
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              placeholder="至少 6 個字元"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="reset-confirm-password">確認新密碼</Label>
            <Input
              id="reset-confirm-password"
              type="password"
              value={confirmNewPassword}
              onChange={(e) => setConfirmNewPassword(e.target.value)}
              placeholder="再次輸入新密碼"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isPending}>
            取消
          </Button>
          <Button onClick={onSubmit} disabled={isPending}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            確認重設
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
