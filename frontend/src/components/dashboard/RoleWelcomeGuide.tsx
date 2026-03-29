import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Sparkles, X as XIcon } from 'lucide-react'

import { useAuthStore } from '@/stores/auth'
import api from '@/lib/api'
import { STALE_TIME } from '@/lib/query'
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
  const { user, hasRole } = useAuthStore()
  const [sessionDismissed, setSessionDismissed] = useState(
    () => !!sessionStorage.getItem(SESSION_KEY),
  )

  // 查詢使用者是否永久關閉歡迎指引
  const { data: prefData } = useQuery({
    queryKey: ['user-preferences', 'show_welcome_guide'],
    queryFn: async () => {
      const res = await api.get<{ key: string; value: boolean }>(
        '/me/preferences/show_welcome_guide',
      )
      return res.data.value
    },
    staleTime: STALE_TIME.SETTINGS,
  })

  // 預設為 true（顯示），使用者可在設定中關閉
  const prefEnabled = prefData ?? true

  if (!user || sessionDismissed || !prefEnabled) return null

  const isAdmin = hasRole('admin')
  const userRoles = user.roles || []
  const guides = getGuidesForRoles(userRoles)

  // admin 顯示儀表板操作說明（維持原有行為）
  if (isAdmin && guides.length === 0) {
    return (
      <GuideBanner
        onDismiss={() => {
          setSessionDismissed(true)
          sessionStorage.setItem(SESSION_KEY, '1')
        }}
      >
        <p className="font-medium text-foreground">
          {t('dashboard.welcome.title', {
            name: user.display_name,
            defaultValue: `歡迎，${user.display_name}！`,
          })}
        </p>
        <p className="text-sm text-muted-foreground mt-1">
          {t('dashboard.welcome.description')}
        </p>
      </GuideBanner>
    )
  }

  // 非 admin 但也沒有匹配的角色指引
  if (guides.length === 0) return null

  const showRolePrefix = guides.length > 1

  return (
    <GuideBanner
      onDismiss={() => {
        setSessionDismissed(true)
        sessionStorage.setItem(SESSION_KEY, '1')
      }}
    >
      <p className="font-medium text-foreground">
        {t('dashboard.welcome.title', {
          name: user.display_name,
          defaultValue: `歡迎，${user.display_name}！`,
        })}
      </p>
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
