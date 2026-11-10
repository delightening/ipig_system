/**
 * 通知中心頁：沒有落點的通知點擊後展開全文。
 *
 * 背景：備份月度健康檢查（related_entity_type = backup_routine_check）在路由表沒有對應頁面，
 * 列表又把內文截成兩行，點下去什麼都不會發生，第三項檢查項目永遠看不到。
 *
 * 要釘住的契約：
 * 1. 沒有落點、有內文 → 點擊展開全文（aria-expanded），再點收合；不導頁。
 * 2. 有落點的通知 → 點擊仍導頁，不進入展開狀態（沒有 aria-expanded）。
 * 3. 沒落點也沒內文 → 不可展開。
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter } from 'react-router-dom'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

const { apiGet, apiPost, navigateMock } = vi.hoisted(() => ({
  apiGet: vi.fn(),
  apiPost: vi.fn(),
  navigateMock: vi.fn(),
}))
vi.mock('@/lib/api', () => ({ default: { get: apiGet, post: apiPost } }))

vi.mock('react-router-dom', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router-dom')>()
  return { ...actual, useNavigate: () => navigateMock }
})

// 型別檔的 label map 依賴 i18n 初始化，與本測試無關
vi.mock('@/types/notification', () => ({ notificationTypeNames: {} }))

import NotificationsPage from '@/pages/NotificationsPage'

const BACKUP_BODY =
  '請確認：\n1) Grafana backup_last_success_timestamp_seconds 持續更新中（< 25h）\n2) 抽 R2 / NAS-B 任一處最新 backup 確認檔案存在\n3) 詳見 runbook §7'

const base = {
  type: 'system_alert',
  is_read: true,
  created_at: '2026-11-01T09:00:00Z',
  kind: 'info',
}

function renderPage(items: unknown[]) {
  apiGet.mockResolvedValue({ data: { data: items, total: items.length, page: 1, per_page: 20 } })
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={client}>
      <MemoryRouter>
        <NotificationsPage />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

describe('NotificationsPage 無落點通知展開', () => {
  beforeEach(() => {
    apiGet.mockReset()
    apiPost.mockReset()
    navigateMock.mockReset()
  })

  it('沒有落點、有內文：點擊展開全文，再點收合，且不導頁', async () => {
    renderPage([
      {
        ...base,
        id: 'n1',
        title: '[Backup] 月度健康檢查',
        content: BACKUP_BODY,
        related_entity_type: 'backup_routine_check',
      },
    ])

    const row = (await screen.findByText('[Backup] 月度健康檢查')).closest('button')!
    expect(row.getAttribute('aria-expanded')).toBe('false')
    const body = screen.getByText(/3\) 詳見 runbook/)
    expect(body.className).toContain('line-clamp-2')

    fireEvent.click(row)
    expect(row.getAttribute('aria-expanded')).toBe('true')
    expect(body.className).toContain('whitespace-pre-line')
    expect(body.className).not.toContain('line-clamp-2')

    fireEvent.click(row)
    expect(row.getAttribute('aria-expanded')).toBe('false')
    expect(body.className).toContain('line-clamp-2')
    expect(navigateMock).not.toHaveBeenCalled()
  })

  it('一次只展開一則：展開第二則時第一則自動收合', async () => {
    renderPage([
      { ...base, id: 'a', title: '通知甲', content: '甲內文', related_entity_type: 'backup_routine_check' },
      { ...base, id: 'b', title: '通知乙', content: '乙內文', related_entity_type: 'backup_routine_check' },
    ])

    const rowA = (await screen.findByText('通知甲')).closest('button')!
    const rowB = screen.getByText('通知乙').closest('button')!
    fireEvent.click(rowA)
    expect(rowA.getAttribute('aria-expanded')).toBe('true')
    fireEvent.click(rowB)
    expect(rowA.getAttribute('aria-expanded')).toBe('false')
    expect(rowB.getAttribute('aria-expanded')).toBe('true')
  })

  it('有落點的通知：點擊導頁，不可展開', async () => {
    renderPage([
      {
        ...base,
        id: 'n2',
        type: 'low_stock',
        title: '低庫存預警',
        content: '採血針 庫存: 200',
        related_entity_type: 'low_stock',
      },
    ])

    const row = (await screen.findByText('低庫存預警')).closest('button')!
    expect(row.hasAttribute('aria-expanded')).toBe(false)
    fireEvent.click(row)
    expect(navigateMock).toHaveBeenCalledWith('/inventory?filter=low_stock')
  })

  it('沒落點也沒內文：不可展開', async () => {
    renderPage([{ ...base, id: 'n3', title: '只有標題', related_entity_type: 'backup_routine_check' }])

    const row = (await screen.findByText('只有標題')).closest('button')!
    expect(row.hasAttribute('aria-expanded')).toBe(false)
    fireEvent.click(row)
    expect(navigateMock).not.toHaveBeenCalled()
  })
})
