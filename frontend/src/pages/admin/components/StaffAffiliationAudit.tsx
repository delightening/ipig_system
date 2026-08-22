/**
 * 「身分旗標與角色矛盾」稽核清單。
 *
 * 為什麼需要：`users.is_internal` 曾經由兩條建立路徑各自**猜**——使用者管理
 * 一律 true、邀請流程硬編 false，兩邊都不問。實測造成 9 位在職同仁分類錯誤，
 * 其中一位因此無法被加入任何部門、假單找不到單位主管簽核。
 *
 * 那兩條路徑已改為必選（見 `StaffAffiliationField`），但**既有資料不會自己
 * 修好**，而且錯誤是靜默的——沒有任何畫面會告訴你某個人被分錯了。
 *
 * ⚠️ **刻意不自動修正**。矛盾不等於錯誤：REVIEWER / VET / IACUC_CHAIR 標為
 * 外部多半是對的（他們確實是外聘），只有具 EXPERIMENT_STAFF 卻標外部這種
 * 才明顯有問題。哪一位獸醫是外聘、哪一位是自己人，只有使用者知道。
 */
import { useMemo } from 'react'
import { AlertTriangle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { deriveAffiliation } from '@/lib/staffAffiliation'
import type { User } from '@/lib/api'

interface StaffAffiliationAuditProps {
  users: User[]
  onEdit: (user: User) => void
}

export function StaffAffiliationAudit({ users, onEdit }: StaffAffiliationAuditProps) {
  const mismatched = useMemo(
    () =>
      users.filter(u => {
        if (!u.is_active) return false
        const derived = deriveAffiliation(u.roles)
        // derived === null 代表角色本身無法判定（REVIEWER/VET/…），不算矛盾
        if (derived === null) return false
        return derived !== (u.is_internal ?? true)
      }),
    [users],
  )

  if (mismatched.length === 0) return null

  return (
    <div className="rounded-lg border border-amber-300 bg-amber-50 p-4 dark:border-amber-800 dark:bg-amber-950/30">
      <div className="flex items-start gap-2">
        <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-amber-600 dark:text-amber-400" />
        <div className="min-w-0 flex-1 space-y-2">
          <div>
            <div className="text-sm font-medium">
              有 {mismatched.length} 位使用者的身分與其角色不一致
            </div>
            <p className="mt-1 text-xs text-muted-foreground">
              下列人員的「本場受僱人員 / 外部人員」設定與其持有的角色相矛盾。
              這不一定是錯的——外聘獸醫、外部審查委員本來就會這樣；
              但具試驗工作人員等內部職能卻被標為外部的人，
              <strong className="font-medium">預設不會出現在部門成員的候選名單裡</strong>
              （需在部門成員對話框明確開啟「一併顯示外部人員」才列得出來、才能指派），
              也不適用請假、加班等人事作業——連帶讓他的假單找不到單位主管簽核。請逐一確認。
            </p>
          </div>
          <ul className="space-y-1">
            {mismatched.map(u => (
              <li key={u.id} className="flex flex-wrap items-center gap-2 text-sm">
                <span className="font-medium">{u.display_name}</span>
                <Badge variant="outline" className="text-xs">
                  目前：{(u.is_internal ?? true) ? '本場受僱人員' : '外部人員'}
                </Badge>
                <span className="text-xs text-muted-foreground">
                  角色：{u.roles.join('、') || '（無）'}
                </span>
                <Button
                  variant="link"
                  size="sm"
                  className="h-auto p-0 text-xs"
                  onClick={() => onEdit(u)}
                >
                  前往修改
                </Button>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  )
}
