import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import type { User } from '@/lib/api'
import { uiLocale } from '@/lib/utils'
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
import { Users, Pencil, Trash2, Shield, UserCheck, UserX, Key, ArrowUpDown, ArrowUp, ArrowDown, LogIn, ChevronLeft, ChevronRight, MoreHorizontal } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

interface UserTableActions {
  onEdit: (user: User) => void
  onManageRoles: (user: User) => void
  onResetPassword: (user: User) => void
  onToggleActive: (user: User) => void
  onDelete: (user: User) => void
  onImpersonate: (user: User) => void
}

interface UserTableSorting {
  sortRole: 'asc' | 'desc' | null
  sortStatus: 'asc' | 'desc' | null
  sortLastLogin: 'asc' | 'desc' | null
  onToggleSortRole: () => void
  onToggleSortStatus: () => void
  onToggleSortLastLogin: () => void
}

interface UserTablePagination {
  currentPage: number
  totalPages: number
  sortedUsersLength: number
  onPrevPage: () => void
  onNextPage: () => void
}

interface UserTableProps {
  users: User[]
  isLoading: boolean
  currentUserId?: string
  actions: UserTableActions
  sorting: UserTableSorting
  pagination: UserTablePagination
}

function getSortIcon(sort: 'asc' | 'desc' | null) {
  if (sort === 'asc') return <ArrowUp className="h-4 w-4" />
  if (sort === 'desc') return <ArrowDown className="h-4 w-4" />
  return <ArrowUpDown className="h-4 w-4 text-muted-foreground" />
}

/**
 * R35-18: 格式化「最後登入」欄位 — relative time + dormant 高亮。
 * - null → 「從未登入」灰色
 * - <1 小時 → 「剛剛」
 * - <24 小時 → 「N 小時前」
 * - <30 天 → 「N 天前」
 * - <90 天 → 完整日期
 * - ≥90 天 → 完整日期 + 紅色 dormant 標記；`note` 折至第 2 行（日期 ↵ 未登入註記），
 *   讓欄寬由較短的註記決定、把水平空間讓給 Email 欄。
 */
function formatLastLogin(
  iso: string | null | undefined,
  t: TFunction,
): { text: string; note?: string; tone: 'muted' | 'normal' | 'warn' } {
  if (!iso) return { text: t('admin.userTable.neverLoggedIn'), tone: 'muted' }
  const time = new Date(iso).getTime()
  if (!Number.isFinite(time)) return { text: '-', tone: 'muted' }
  const diffMs = Date.now() - time
  const minutes = Math.floor(diffMs / 60_000)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  if (minutes < 60) {
    return { text: minutes < 1 ? t('common.justNow') : t('common.minutesAgo', { count: minutes }), tone: 'normal' }
  }
  if (hours < 24) return { text: t('common.hoursAgo', { count: hours }), tone: 'normal' }
  if (days < 30) return { text: t('common.daysAgo', { count: days }), tone: 'normal' }
  const dateStr = new Date(iso).toLocaleDateString(uiLocale())
  if (days >= 90) return { text: dateStr, note: t('admin.userTable.dormantNote', { days }), tone: 'warn' }
  return { text: dateStr, tone: 'normal' }
}

export function UserTable({
  users,
  isLoading,
  currentUserId,
  actions,
  sorting,
  pagination,
}: UserTableProps) {
  const { t } = useTranslation()
  const { onEdit, onManageRoles, onResetPassword, onToggleActive, onDelete, onImpersonate } = actions
  // sortStatus 不解構：狀態欄依需求不顯示排序箭頭（點整格即切換），僅需 onToggleSortStatus
  const { sortRole, sortLastLogin, onToggleSortRole, onToggleSortStatus, onToggleSortLastLogin } = sorting
  const { currentPage, totalPages, sortedUsersLength, onPrevPage, onNextPage } = pagination
  return (
    <div className="@container rounded-lg border bg-card overflow-hidden">
      <Table>
        <TableHeader>
          <TableRow className="bg-muted/50 hover:bg-muted/50">
            {/* Email：彈性欄，吸收剩餘寬度 */}
            <TableHead>{t('common.email')}</TableHead>
            {/* 名稱：nowrap，永遠完整顯示、不折行 */}
            <TableHead className="whitespace-nowrap">{t('admin.userTable.name')}</TableHead>
            {/* 角色：@[720px] 起顯示（第 2 個隱藏） */}
            <TableHead className="hidden @[720px]:table-cell">
              <button
                className="flex items-center gap-1 hover:text-foreground transition-colors"
                onClick={onToggleSortRole}
              >
                {t('admin.userTable.role')}
                {getSortIcon(sortRole)}
              </button>
            </TableHead>
            {/* 狀態：@[900px] 起顯示（第 1 個隱藏）；直書、點整格切換排序、不顯示排序箭頭 */}
            <TableHead className="hidden @[900px]:table-cell w-[1%] text-center p-0">
              <button
                className="flex h-full w-full items-center justify-center px-4 py-3 hover:text-foreground transition-colors select-none"
                onClick={onToggleSortStatus}
                title={t('admin.userTable.status')}
              >
                {/* 直書字序修正：writing-mode 掛在內層 span（非 flex button），
                    避免 flex 容器 + vertical-rl 造成的字序反轉（顯示成「態狀」） */}
                <span className="[writing-mode:vertical-rl] [text-orientation:upright] tracking-widest">
                  {t('admin.userTable.status')}
                </span>
              </button>
            </TableHead>
            {/* 最後登入：@[560px] 起顯示（最後隱藏）；可排序 */}
            <TableHead className="hidden @[560px]:table-cell w-[1%] whitespace-nowrap">
              <button
                className="flex items-center gap-1 hover:text-foreground transition-colors"
                onClick={onToggleSortLastLogin}
              >
                {t('admin.userTable.lastLogin')}
                {getSortIcon(sortLastLogin)}
              </button>
            </TableHead>
            {/* 操作：縮到內容寬，靠右 */}
            <TableHead className="w-[1%] whitespace-nowrap text-right">{t('common.actions')}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow>
              <TableCell colSpan={6} className="p-0">
                <TableSkeleton rows={8} cols={6} />
              </TableCell>
            </TableRow>
          ) : users.length > 0 ? (
            users.map((user) => (
              <TableRow key={user.id} className="[&_td]:py-1.5">{/* 列高緊湊：僅資料列縮 cell 垂直 padding，不動 skeleton/空狀態 */}
                <TableCell className="font-medium">{user.email}</TableCell>
                <TableCell className="whitespace-nowrap">{user.display_name}</TableCell>
                <TableCell className="hidden @[720px]:table-cell">
                  <div className="flex flex-col items-start gap-1">
                    {user.roles.length > 0 ? (
                      user.roles.map((role) => (
                        <Badge key={role} variant="secondary">
                          {role}
                        </Badge>
                      ))
                    ) : (
                      <span className="text-muted-foreground text-sm">{t('admin.userTable.noRoles')}</span>
                    )}
                  </div>
                </TableCell>
                <TableCell className="hidden @[900px]:table-cell w-[1%] text-center">
                  {user.is_active ? (
                    <Badge variant="success" className="[writing-mode:vertical-rl] [text-orientation:upright] px-1 py-1.5 tracking-widest">{t('admin.userTable.active')}</Badge>
                  ) : (
                    <Badge variant="destructive" className="[writing-mode:vertical-rl] [text-orientation:upright] px-1 py-1.5 tracking-widest">{t('admin.userTable.inactive')}</Badge>
                  )}
                </TableCell>
                <TableCell className="hidden @[560px]:table-cell w-[1%] whitespace-nowrap">
                  {(() => {
                    const { text, note, tone } = formatLastLogin(user.last_login_at, t)
                    const cls = tone === 'warn'
                      ? 'text-status-error-text font-medium'
                      : tone === 'muted'
                        ? 'text-muted-foreground'
                        : ''
                    // dormant（≥90 天）時 note 折至第 2 行，欄寬由較短的註記決定；
                    // 明確 <br> 換行，與 cell 的 whitespace-nowrap 不衝突（僅擋自動換行）
                    return (
                      <span className={cls} title={user.last_login_at ?? t('admin.userTable.neverLoggedIn')}>
                        {text}
                        {note && <br />}
                        {note}
                      </span>
                    )
                  })()}
                </TableCell>
                <TableCell className="w-[1%] whitespace-nowrap text-right">
                  {(() => {
                    // 單一動作清單（grid 與 ⋯ 下拉共用，避免重複邏輯）
                    const isSelf = user.id === currentUserId
                    const items = [
                      ...(isSelf
                        ? []
                        : [{
                            key: 'impersonate',
                            icon: <LogIn className="h-4 w-4 text-primary" />,
                            label: t('admin.userTable.impersonateTitle'),
                            onClick: () => onImpersonate(user),
                          }]),
                      { key: 'roles', icon: <Shield className="h-4 w-4" />, label: t('admin.userTable.manageRoles'), onClick: () => onManageRoles(user) },
                      ...(isSelf
                        ? []
                        : [{
                            key: 'reset',
                            icon: <Key className="h-4 w-4 text-status-warning-text" />,
                            label: t('admin.userTable.resetPassword'),
                            onClick: () => onResetPassword(user),
                          }]),
                      { key: 'edit', icon: <Pencil className="h-4 w-4" />, label: t('common.edit'), onClick: () => onEdit(user) },
                      {
                        key: 'toggle',
                        icon: user.is_active ? <UserX className="h-4 w-4 text-destructive" /> : <UserCheck className="h-4 w-4 text-status-success-text" />,
                        label: user.is_active ? t('admin.userTable.deactivate') : t('admin.userTable.activate'),
                        onClick: () => onToggleActive(user),
                      },
                      { key: 'delete', icon: <Trash2 className="h-4 w-4 text-destructive" />, label: t('common.delete'), onClick: () => onDelete(user) },
                    ]
                    return (
                      <>
                        {/* 手機（容器 <640px）：操作收進 ⋯ 下拉選單 */}
                        <div className="@[640px]:hidden">
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                              <Button variant="ghost" size="icon" aria-label={t('common.actions')}>
                                <MoreHorizontal className="h-4 w-4" />
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end">
                              {items.map((a) => (
                                <DropdownMenuItem key={a.key} onSelect={a.onClick}>
                                  {a.icon}
                                  <span className="ml-2">{a.label}</span>
                                </DropdownMenuItem>
                              ))}
                            </DropdownMenuContent>
                          </DropdownMenu>
                        </div>
                        {/* 電腦（容器 ≥640px）：2×3 網格，間距 gap-x-10 gap-y-0（水平放寬、垂直兩排貼合）；本人 4 顆 → 2×2 */}
                        <div className={`hidden @[640px]:grid gap-x-10 gap-y-0 w-fit ml-auto justify-items-center ${isSelf ? 'grid-cols-2' : 'grid-cols-3'}`}>
                          {items.map((a) => (
                            <Button key={a.key} variant="ghost" size="icon" className="h-7 w-7" onClick={a.onClick} title={a.label} aria-label={a.label}>
                              {a.icon}
                            </Button>
                          ))}
                        </div>
                      </>
                    )
                  })()}
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableEmptyRow colSpan={6} icon={Users} title={t('admin.userTable.empty')} />
          )}
        </TableBody>
      </Table>
      {totalPages > 0 && (
        <div className="flex flex-col md:flex-row items-center justify-between px-4 py-3 border-t gap-2">
          <p className="text-sm text-muted-foreground">
            {t('admin.userTable.pageInfo', { total: sortedUsersLength, current: currentPage, totalPages })}
          </p>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={currentPage <= 1}
              onClick={onPrevPage}
            >
              <ChevronLeft className="h-4 w-4 mr-1" />
              {t('admin.userTable.prevPage')}
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={currentPage >= totalPages}
              onClick={onNextPage}
            >
              {t('admin.userTable.nextPage')}
              <ChevronRight className="h-4 w-4 ml-1" />
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}
