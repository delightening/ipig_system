import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { vi } from 'vitest'
import i18n from '@/lib/i18n'

/**
 * QAU 儀表板的狀態欄。
 *
 * 後端 display_name 固定中文（backend/src/services/qau.rs），頁面改依 status code 走語言包。
 * 釘住三件事：
 * 1. en 下顯示英文（計畫書 `protocols.status.*`、動物 `animals.statusLabels.*`）。
 * 2. zh-TW 下顯示語言包措辭，不是後端的 display_name。
 * 3. 語言包沒有的 code 退回後端 display_name，不顯示鍵名。
 */

const DASHBOARD = {
  protocol_status_summary: [
    { status: 'APPROVED', display_name: '已核准', count: 8 },
    { status: 'PRE_REVIEW', display_name: '行政預審', count: 2 },
    { status: 'FUTURE_STATUS', display_name: '未來狀態', count: 1 },
  ],
  review_progress: { status_changes_last_7_days: 0, protocols_in_review: 0, protocols_pending_pi_response: 0 },
  audit_summary: [],
  animal_summary: {
    total: 3,
    by_status: [{ status: 'in_experiment', display_name: '實驗中', count: 3 }],
    in_experiment: 3,
    euthanized: 0,
    completed: 0,
  },
  qa_plan_summary: {
    open_nc_count: 0,
    overdue_nc_count: 0,
    active_sop_count: 0,
    inspection_by_status: [],
    schedule_items_by_status: [],
  },
}

vi.mock('@/hooks/useGuestQuery', () => ({
  useGuestQuery: () => ({ data: DASHBOARD, isLoading: false, error: null }),
}))

const { QAUDashboardPage } = await import('../QAUDashboardPage')

let previousLanguage: string

beforeAll(() => {
  previousLanguage = i18n.language
})

afterAll(async () => {
  await i18n.changeLanguage(previousLanguage)
})

// 摘要卡片也會出現「實驗中」這類字，只看表格儲存格才量得到狀態欄本身
function renderCells(): string[] {
  render(
    <MemoryRouter>
      <QAUDashboardPage />
    </MemoryRouter>,
  )
  return screen.getAllByRole('cell').map((c) => c.textContent?.trim() ?? '')
}

it('en：狀態欄顯示英文，不顯示後端中文', async () => {
  await i18n.changeLanguage('en')
  const cells = renderCells()
  expect(cells).toEqual(expect.arrayContaining(['Approved', 'Pre-Review', 'In Experiment']))
  expect(cells).not.toContain('已核准')
  expect(cells).not.toContain('實驗中')
})

it('zh-TW：狀態欄用語言包措辭', async () => {
  await i18n.changeLanguage('zh-TW')
  const cells = renderCells()
  expect(cells).toEqual(expect.arrayContaining(['已核准', '行政預審中', '實驗中']))
  // 後端原字「行政預審」不應再出現
  expect(cells).not.toContain('行政預審')
})

it('語言包沒有的 code 退回後端 display_name，不顯示鍵名', async () => {
  await i18n.changeLanguage('en')
  const cells = renderCells()
  expect(cells).toContain('未來狀態')
  expect(cells.some((c) => c.startsWith('protocols.status.'))).toBe(false)
})
