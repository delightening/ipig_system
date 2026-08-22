import type { LucideIcon } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { TableRow, TableCell } from '@/components/ui/table'
import { cn } from '@/lib/utils'

interface EmptyStateAction {
  label: string
  onClick: () => void
  icon?: LucideIcon
}

interface EmptyStateProps {
  icon: LucideIcon
  title: string
  description?: string
  action?: EmptyStateAction
  className?: string
}

export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  className,
}: EmptyStateProps) {
  return (
    <div className={cn('flex flex-col items-center justify-center py-12 px-4 text-center', className)}>
      <Icon className="h-12 w-12 text-muted-foreground/50 mb-4" strokeWidth={1.5} />
      <h3 className="text-lg font-semibold text-foreground mb-1">{title}</h3>
      {description && (
        <p className="text-sm text-muted-foreground max-w-sm mb-4">{description}</p>
      )}
      {action && (
        <Button onClick={action.onClick} size="sm">
          {action.icon && <action.icon className="h-4 w-4 mr-2" />}
          {action.label}
        </Button>
      )}
    </div>
  )
}

interface TableEmptyRowProps {
  colSpan: number
  icon: LucideIcon
  title: string
  description?: string
  action?: EmptyStateAction
}

export function TableEmptyRow({
  colSpan,
  icon: Icon,
  title,
  description,
  action,
}: TableEmptyRowProps) {
  return (
    <TableRow>
      <TableCell colSpan={colSpan} className="text-center py-8">
        <Icon className="h-12 w-12 mx-auto mb-2 text-muted-foreground/50" strokeWidth={1.5} />
        <p className="text-muted-foreground">{title}</p>
        {description && (
          <p className="text-xs text-muted-foreground/70 mt-1">{description}</p>
        )}
        {action && (
          <Button onClick={action.onClick} variant="outline" size="sm" className="mt-4">
            {action.icon && <action.icon className="h-4 w-4 mr-2" />}
            {action.label}
          </Button>
        )}
      </TableCell>
    </TableRow>
  )
}

interface TableErrorRowProps {
  colSpan: number
  /** 覆寫預設訊息；一般情況不必給，共用文案就夠了 */
  message?: string
}

/**
 * 查詢失敗時的表格列。
 *
 * 為什麼不共用 `TableEmptyRow`：查詢失敗與「真的沒有資料」在畫面上長得一樣
 * （兩者的 `data` 都是空陣列），但意義相反——空狀態常會叫使用者去做一件事
 * （「新增第一筆」「指派角色後會自動出現」），而 403／斷線時那件事早就做過了，
 * 照著做只是白忙，真正的失敗原因反而被蓋掉。錯誤分支必須排在空狀態之前。
 *
 * ⚠️ **呼叫端要把條件寫成「`isError` 且手上沒有資料可顯示」**，不是單看 `isError`：
 * TanStack Query v5 在「已成功取過、之後 refetch 失敗」時會讓 `isError` 為 true 而
 * `data` 仍保留上一次的結果（本 repo 實測確認）。只看 `isError` 會在 mutation 後的
 * `invalidateQueries` 重取失敗時，把畫面上原本好好的資料整片換成這一列。
 */
export function TableErrorRow({ colSpan, message }: TableErrorRowProps) {
  const { t } = useTranslation()
  return (
    <TableRow>
      <TableCell colSpan={colSpan} className="py-8 text-center text-status-error-text">
        {message ?? t('common.loadFailed')}
      </TableCell>
    </TableRow>
  )
}
