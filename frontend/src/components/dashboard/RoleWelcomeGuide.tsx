import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { AlertCircle, Sparkles, X as XIcon } from 'lucide-react'

import { useAuthUser, useAuthHasRole } from '@/stores/auth'
import { useWelcomeGuidePref } from '@/hooks/useWelcomeGuidePref'
import { TRAINING_ROLES, hasAnyRole } from '@/lib/constants'
import { getGuidesForRoles } from './roleGuideConfig'

const SESSION_KEY = 'dashboard-welcome-dismissed'

/**
 * 角色歡迎指引 Banner
 *
 * - 根據使用者角色顯示對應的指引內容與頁面連結
 * - 多角色時合併顯示，以「身為 XX」前綴區分
 * - admin 不顯示角色指引（顯示儀表板操作說明）
 * - 支援 sessionStorage 單次關閉 + preference 永久關閉
 */
export function RoleWelcomeGuide() {
  const { t } = useTranslation()
  const user = useAuthUser()
  const hasRole = useAuthHasRole()
  const [sessionDismissed, setSessionDismissed] = useState(
    () => !!sessionStorage.getItem(SESSION_KEY),
  )

  // 使用者是否永久關閉歡迎指引（預設顯示，可在設定中關閉）
  const { enabled: prefEnabled, isPending: isPrefPending } = useWelcomeGuidePref()

  // 偏好未到手前一律不渲染：先畫再收掉會把下方的 widget 網格往上拉，造成登入時的版面跳動。
  // DashboardPage 另以同一個 pending 狀態壓住網格（顯示骨架），讓橫幅與網格同時定案、只繪製一次。
  if (!user || sessionDismissed || isPrefPending || !prefEnabled) return null

  const isAdmin = hasRole('admin')
  const userRoles = user.roles || []
  const guides = getGuidesForRoles(userRoles)

  // 訓練/資格提醒：需填訓練的角色（審查相關＋staff）且尚未填任何訓練時，
  // 於歡迎橫幅內提醒前往帳號設定填寫。沿用歡迎指引開關（上方 prefEnabled）。
  const needsTrainingReminder =
    hasAnyRole(userRoles, TRAINING_ROLES) && (user.trainings?.length ?? 0) === 0
  const trainingReminder = needsTrainingReminder ? (
    <p className="mt-2 flex items-start gap-1.5 text-sm text-status-warning-text">
      <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" />
      <span>
        {t('dashboard.welcome.trainingReminder')}{' '}
        <Link
          to="/profile/settings"
          className="font-medium underline underline-offset-2 hover:opacity-80"
        >
          {t('dashboard.welcome.trainingReminderLink')}
        </Link>
      </span>
    </p>
  ) : null

  const dismiss = () => {
    setSessionDismissed(true)
    sessionStorage.setItem(SESSION_KEY, '1')
  }

  const welcomeTitle = (
    <p className="font-medium text-foreground">
      {t('dashboard.welcome.title', {
        name: user.display_name,
        defaultValue: `歡迎，${user.display_name}！`,
      })}
    </p>
  )

  // admin 顯示儀表板操作說明（維持原有行為）
  if (isAdmin && guides.length === 0) {
    return (
      <GuideBanner onDismiss={dismiss}>
        {welcomeTitle}
        <p className="text-sm text-muted-foreground mt-1">
          {t('dashboard.welcome.description')}
        </p>
        {trainingReminder}
      </GuideBanner>
    )
  }

  // 非 admin 且無匹配角色指引：僅在需要訓練提醒時仍顯示橫幅
  if (guides.length === 0) {
    if (!trainingReminder) return null
    return (
      <GuideBanner onDismiss={dismiss}>
        {welcomeTitle}
        {trainingReminder}
      </GuideBanner>
    )
  }

  const showRolePrefix = guides.length > 1

  return (
    <GuideBanner onDismiss={dismiss}>
      {welcomeTitle}
      <div className="mt-1.5 space-y-1">
        {guides.map((guide) => {
          const roleName = t(`dashboard.welcome.roles.${guide.i18nKey}.name`)
          const desc = t(`dashboard.welcome.roles.${guide.i18nKey}.description`)

          return (
            <p key={guide.role} className="text-sm text-muted-foreground">
              {showRolePrefix && (
                <span className="font-medium text-foreground">
                  {t('dashboard.welcome.asRole', { role: roleName })}
                </span>
              )}
              <GuideDescription
                description={desc}
                links={guide.links}
              />
            </p>
          )
        })}
      </div>
      {trainingReminder}
    </GuideBanner>
  )
}

/** Banner 外殼 */
function GuideBanner({
  children,
  onDismiss,
}: {
  children: React.ReactNode
  onDismiss: () => void
}) {
  return (
    <div className="relative p-4 bg-primary/5 border border-primary/20 rounded-lg">
      <button
        onClick={onDismiss}
        className="absolute top-3 right-3 text-muted-foreground hover:text-foreground"
      >
        <XIcon className="h-4 w-4" />
      </button>
      <div className="flex items-start gap-3">
        <Sparkles className="h-5 w-5 text-primary mt-0.5 shrink-0" />
        <div>{children}</div>
      </div>
    </div>
  )
}

/**
 * 指引描述：將描述文字中的 {{linkN}} 佔位符替換為實際連結
 *
 * 描述格式範例：
 * "您可以在 {{link0}} 查看動物紀錄、在 {{link1}} 處理單據"
 */
function GuideDescription({
  description,
  links,
}: {
  description: string
  links: { labelKey: string; href: string }[]
}) {
  const { t } = useTranslation()

  // 分割描述，將 {{linkN}} 替換為 Link 元件
  const parts = description.split(/({{link\d+}})/)

  return (
    <>
      {parts.map((part, idx) => {
        const match = part.match(/^{{link(\d+)}}$/)
        if (!match) return <span key={idx}>{part}</span>

        const linkIndex = parseInt(match[1], 10)
        const link = links[linkIndex]
        if (!link) return <span key={idx}>{part}</span>

        const label = t(`dashboard.welcome.linkLabels.${link.labelKey}`)
        return (
          <Link
            key={idx}
            to={link.href}
            className="font-medium text-primary underline underline-offset-2 hover:text-primary/80"
          >
            {label}
          </Link>
        )
      })}
    </>
  )
}
