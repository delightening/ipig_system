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
import { Trans, useTranslation } from 'react-i18next'
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
  const { t } = useTranslation()
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
              {t('adminUsers.users.affiliationAudit.title', { count: mismatched.length })}
            </div>
            <p className="mt-1 text-xs text-muted-foreground">
              <Trans
                i18nKey="adminUsers.users.affiliationAudit.body"
                components={{ strong: <strong className="font-medium" /> }}
              />
            </p>
          </div>
          <ul className="space-y-1">
            {mismatched.map(u => (
              <li key={u.id} className="flex flex-wrap items-center gap-2 text-sm">
                <span className="font-medium">{u.display_name}</span>
                <Badge variant="outline" className="text-xs">
                  {t('adminUsers.users.affiliationAudit.current', {
                    status: (u.is_internal ?? true) ? t('adminUsers.shared.affiliation.internal') : t('adminUsers.shared.affiliation.external'),
                  })}
                </Badge>
                <span className="text-xs text-muted-foreground">
                  {t('adminUsers.users.affiliationAudit.roles', {
                    roles: u.roles.join(t('adminUsers.shared.listSeparator')) || t('admin.departmentTab.none'),
                  })}
                </span>
                <Button
                  variant="link"
                  size="sm"
                  className="h-auto p-0 text-xs"
                  onClick={() => onEdit(u)}
                >
                  {t('adminUsers.users.affiliationAudit.goEdit')}
                </Button>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  )
}
