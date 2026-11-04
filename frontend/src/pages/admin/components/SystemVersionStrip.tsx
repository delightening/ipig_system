import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api from '@/lib/api'
import { uiLocale } from '@/lib/utils'
import {
  formatBuildTime,
  formatMigrationVersion,
  shortSha,
  type SystemVersionInfo,
} from './systemVersion'

const STRIP_CLASS =
  'flex flex-wrap items-center gap-x-6 gap-y-1 rounded-lg bg-status-info-bg px-4 py-2.5 text-sm text-status-info-text'

/**
 * `/admin/settings` 頂端的唯讀「系統版本」資訊帶：短 SHA、建置時間（GMT+8）、migration 版本。
 *
 * 只用既有端點 `GET /admin/internal/version`（系統管理員專用），不放進公開的 /api/health。
 * 後端沒收到 build 參數時回 `unknown`，這裡一律顯示「未知」，不顯示原字串。
 * 查詢失敗與「值未知」分開呈現：失敗時顯示錯誤文案，不假裝成未知版本。
 */
export function SystemVersionStrip() {
  const { t } = useTranslation()
  const { data, isLoading, isError } = useQuery({
    queryKey: ['admin-internal-version'],
    queryFn: async () => {
      const res = await api.get<SystemVersionInfo>('/admin/internal/version')
      return res.data
    },
    staleTime: 300_000,
  })

  if (isLoading) return null

  if (isError || !data) {
    return (
      <div className={STRIP_CLASS} role="status">
        <span className="font-semibold">{t('admin.systemVersion.title')}</span>
        <span>{t('admin.systemVersion.loadError')}</span>
      </div>
    )
  }

  const unknown = t('admin.systemVersion.unknown')
  const sha = shortSha(data.git_sha)
  const builtAt = formatBuildTime(data.build_time, uiLocale())
  const migration = formatMigrationVersion(data.migration_max_version)

  return (
    <div className={STRIP_CLASS}>
      <span className="font-semibold">{t('admin.systemVersion.title')}</span>
      <span>
        {t('admin.systemVersion.sha')}{' '}
        {sha ? (
          <code className="font-mono font-medium">{sha}</code>
        ) : (
          <span className="font-medium">{unknown}</span>
        )}
      </span>
      <span>
        {t('admin.systemVersion.buildTime')}{' '}
        <span className="font-medium">
          {builtAt ? t('admin.systemVersion.buildTimeValue', { time: builtAt }) : unknown}
        </span>
      </span>
      <span>
        {t('admin.systemVersion.migration')}{' '}
        <span className="font-medium">{migration ?? unknown}</span>
      </span>
    </div>
  )
}
