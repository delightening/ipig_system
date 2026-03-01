import type { User } from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Users, Pencil, Trash2, Shield, UserCheck, UserX, Key, ArrowUpDown, ArrowUp, ArrowDown, LogIn, ChevronLeft, ChevronRight } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'

interface UserTableProps {
  users: User[]
  isLoading: boolean
  sortRole: 'asc' | 'desc' | null
  sortStatus: 'asc' | 'desc' | null
  currentPage: number
  totalPages: number
  sortedUsersLength: number
  currentUserId?: string
  onToggleSortRole: () => void
  onToggleSortStatus: () => void
  onPrevPage: () => void
  onNextPage: () => void
  onEdit: (user: User) => void
  onManageRoles: (user: User) => void
  onResetPassword: (user: User) => void
  onToggleActive: (user: User) => void
  onDelete: (user: User) => void
  onImpersonate: (user: User) => void
}

function getSortIcon(sort: 'asc' | 'desc' | null) {
  if (sort === 'asc') return <ArrowUp className="h-4 w-4" />
  if (sort === 'desc') return <ArrowDown className="h-4 w-4" />
  return <ArrowUpDown className="h-4 w-4 text-muted-foreground" />
}

export function UserTable({
  users,
  isLoading,
  sortRole,
  sortStatus,
  currentPage,
  totalPages,
  sortedUsersLength,
  currentUserId,
  onToggleSortRole,
  onToggleSortStatus,
  onPrevPage,
  onNextPage,
  onEdit,
  onManageRoles,
  onResetPassword,
  onToggleActive,
  onDelete,
  onImpersonate,
}: UserTableProps) {
  return (
    <div className="rounded-md border bg-white">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Email</TableHead>
            <TableHead>名稱</TableHead>
            <TableHead>
              <button
                className="flex items-center gap-1 hover:text-foreground transition-colors"
                onClick={onToggleSortRole}
              >
                角色
                {getSortIcon(sortRole)}
              </button>
            </TableHead>
            <TableHead>
              <button
                className="flex items-center gap-1 hover:text-foreground transition-colors"
                onClick={onToggleSortStatus}
              >
                狀態
                {getSortIcon(sortStatus)}
              </button>
            </TableHead>
            <TableHead className="text-right">操作</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow>
              <TableCell colSpan={5} className="p-0">
                <TableSkeleton rows={8} cols={5} />
              </TableCell>
            </TableRow>
          ) : users.length > 0 ? (
            users.map((user) => (
              <TableRow key={user.id}>
                <TableCell className="font-medium">{user.email}</TableCell>
                <TableCell>{user.display_name}</TableCell>
                <TableCell>
                  <div className="flex gap-1 flex-wrap">
                    {user.roles.length > 0 ? (
                      user.roles.map((role) => (
                        <Badge key={role} variant="secondary">
                          {role}
                        </Badge>
                      ))
                    ) : (
                      <span className="text-muted-foreground text-sm">無角色</span>
                    )}
                  </div>
                </TableCell>
                <TableCell>
                  {user.is_active ? (
                    <Badge variant="success">啟用</Badge>
                  ) : (
                    <Badge variant="destructive">停用</Badge>
                  )}
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-2">
                    {user.id !== currentUserId && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => onImpersonate(user)}
                        title="模擬登入 (Login As)"
                      >
                        <LogIn className="h-4 w-4 text-blue-500" />
                      </Button>
                    )}
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => onManageRoles(user)}
                      title="管理角色"
                    >
                      <Shield className="h-4 w-4" />
                    </Button>
                    {user.id !== currentUserId && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => onResetPassword(user)}
                        title="重設密碼"
                      >
                        <Key className="h-4 w-4 text-orange-500" />
                      </Button>
                    )}
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => onEdit(user)}
                      title="編輯"
                    >
                      <Pencil className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => onToggleActive(user)}
                      title={user.is_active ? '停用' : '啟用'}
                    >
                      {user.is_active ? (
                        <UserX className="h-4 w-4 text-red-500" />
                      ) : (
                        <UserCheck className="h-4 w-4 text-green-500" />
                      )}
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => onDelete(user)}
                      title="刪除"
                    >
                      <Trash2 className="h-4 w-4 text-red-500" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={5} className="text-center py-8">
                <Users className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                <p className="text-muted-foreground">尚無使用者資料</p>
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
      {totalPages > 0 && (
        <div className="flex flex-col md:flex-row items-center justify-between px-4 py-3 border-t gap-2">
          <p className="text-sm text-muted-foreground">
            共 {sortedUsersLength} 筆，第 {currentPage} / {totalPages} 頁
          </p>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={currentPage <= 1}
              onClick={onPrevPage}
            >
              <ChevronLeft className="h-4 w-4 mr-1" />
              上一頁
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={currentPage >= totalPages}
              onClick={onNextPage}
            >
              下一頁
              <ChevronRight className="h-4 w-4 ml-1" />
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}
