/**
 * `/admin/settings` 的「系統版本」資訊帶。
 *
 * 要釘住的契約：
 * 1. SHA 只顯示前 12 碼（與部署驗收的 `git rev-parse --short=12 HEAD` 對得上）。
 * 2. 建置時間一律換算成台灣時間（GMT+8），不隨瀏覽器時區改變。
 * 3. 後端回 `unknown`（build 時沒收到參數）時顯示「未知」，不顯示原字串。
 * 4. 查詢失敗與「值未知」分開呈現——失敗時不能假裝成未知版本。
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

// t() 回傳 key；有帶 time 插值時把值接在後面，才斷言得到換算後的時間
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { time?: string }) => (opts?.time ? `${key}|${opts.time}` : key),
  }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

const { apiGet } = vi.hoisted(() => ({ apiGet: vi.fn() }))
vi.mock('@/lib/api', () => ({ default: { get: apiGet } }))

import { SystemVersionStrip } from '@/pages/admin/components/SystemVersionStrip'
import {
  formatBuildTime,
  formatMigrationVersion,
  shortSha,
} from '@/pages/admin/components/systemVersion'

function renderStrip() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={client}>
      <SystemVersionStrip />
    </QueryClientProvider>,
  )
}

describe('systemVersion helpers', () => {
  it('shortSha 只留前 12 碼', () => {
    expect(shortSha('aa38782c4e1f0123456789')).toBe('aa38782c4e1f')
    expect(shortSha('aa38782')).toBe('aa38782')
  })

  it('shortSha 對 unknown／空值回 null', () => {
    expect(shortSha('unknown')).toBeNull()
    expect(shortSha('UNKNOWN')).toBeNull()
    expect(shortSha('   ')).toBeNull()
    expect(shortSha(null)).toBeNull()
    expect(shortSha(undefined)).toBeNull()
  })

  it('formatBuildTime 把 UTC 換算成台灣時間（+8）', () => {
    // 14:30 UTC = 22:30 台灣
    const zh = formatBuildTime('2026-11-04T14:30:00Z', 'zh-TW')
    expect(zh).toContain('22:30')
    expect(zh).toContain('2026')
    expect(zh).not.toContain('14:30')
    expect(formatBuildTime('2026-11-04T14:30:00Z', 'en-US')).toContain('22:30')
  })

  it('formatBuildTime 跨日換算：UTC 16:00 是台灣隔天 00:00，且不顯示成 24:00', () => {
    const out = formatBuildTime('2026-11-04T16:00:00Z', 'en-US')
    expect(out).toContain('00:00')
    expect(out).not.toContain('24:00')
    expect(out).toContain('05')
  })

  it('formatBuildTime 對 unknown／空值／無法解析回 null', () => {
    expect(formatBuildTime('unknown', 'zh-TW')).toBeNull()
    expect(formatBuildTime('', 'zh-TW')).toBeNull()
    expect(formatBuildTime('not-a-date', 'zh-TW')).toBeNull()
    expect(formatBuildTime(undefined, 'zh-TW')).toBeNull()
  })

  it('formatMigrationVersion：正整數顯示，0 視為未知', () => {
    expect(formatMigrationVersion(59)).toBe('59')
    expect(formatMigrationVersion(0)).toBeNull()
    expect(formatMigrationVersion(Number.NaN)).toBeNull()
    expect(formatMigrationVersion(null)).toBeNull()
  })
})

describe('SystemVersionStrip', () => {
  beforeEach(() => {
    apiGet.mockReset()
  })

  it('顯示 12 碼 SHA、台灣時間的建置時間與 migration 版本', async () => {
    apiGet.mockResolvedValue({
      data: {
        git_sha: 'aa38782c4e1f0123456789',
        build_time: '2026-11-04T14:30:00Z',
        migration_max_version: 59,
        rust_version: 'rustc 1.99.0',
      },
    })
    renderStrip()

    expect(await screen.findByText('aa38782c4e1f')).toBeTruthy()
    expect(screen.queryByText('aa38782c4e1f0123456789')).toBeNull()
    expect(screen.getByText(/admin\.systemVersion\.buildTimeValue\|.*22:30/)).toBeTruthy()
    expect(screen.getByText('59')).toBeTruthy()
    expect(apiGet).toHaveBeenCalledWith('/admin/internal/version')
  })

  it('後端回 unknown 時顯示「未知」，不顯示原字串', async () => {
    apiGet.mockResolvedValue({
      data: {
        git_sha: 'unknown',
        build_time: 'unknown',
        migration_max_version: 0,
        rust_version: 'unknown',
      },
    })
    renderStrip()

    await waitFor(() => {
      expect(screen.getAllByText('admin.systemVersion.unknown')).toHaveLength(3)
    })
    expect(screen.queryByText('unknown')).toBeNull()
  })

  it('查詢失敗時顯示錯誤文案，且不假裝成「未知」', async () => {
    apiGet.mockRejectedValue(new Error('403'))
    renderStrip()

    expect(await screen.findByText('admin.systemVersion.loadError')).toBeTruthy()
    expect(screen.queryByText('admin.systemVersion.unknown')).toBeNull()
  })
})
