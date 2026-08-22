import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { vi } from 'vitest'
import i18n from '@/lib/i18n'
import type { SecurityAlert } from '@/types/hr'

// 本檔的受測對象是「來源摘要」。對話框內另有 #1090 的解鎖面板（AlertLockPanel），
// 它掛載即發 useQuery（需要 QueryClientProvider + API mock），行為已由
// alertLockPanel.test.tsx 完整覆蓋——這裡 mock 成空殼，讓兩個功能的測試各守各的。
vi.mock('../components/AlertLockPanel', () => ({
  AlertLockPanel: () => null,
}))

// 於 vi.mock 之後 import，確保對話框拿到的是 mock 版本
const { AuditAlertDetailDialog } = await import('../components/AuditAlertDetailDialog')

/**
 * 安全警報詳情的「來源摘要」。
 *
 * 這塊存在的理由：unusual_login 警報的描述只寫得出「來自新的 IP 位置」，
 * 管理員看不到究竟是哪個 IP，也就無從判斷是本人換網路還是他人盜用。
 * 摘要把 context_data 裡最關鍵的幾個欄位提到前面，不必從下方 JSON 自己撈。
 */

let previousLanguage: string

beforeAll(async () => {
  previousLanguage = i18n.language
  await i18n.changeLanguage('zh-TW')
})

afterAll(async () => {
  await i18n.changeLanguage(previousLanguage)
})

const BASE_ALERT: SecurityAlert = {
  id: 'alert-1',
  alert_type: 'unusual_login',
  severity: 'warning',
  title: '偵測到異常登入 (vet@example.com)',
  description: '帳號 vet@example.com 的登入觸發異常偵測：使用新裝置',
  user_id: 'user-1',
  context_data: null,
  status: 'open',
  resolved_by: null,
  resolved_at: null,
  resolution_notes: null,
  created_at: '2026-07-29T07:12:00Z',
}

function renderDialog(alert: SecurityAlert) {
  render(
    <MemoryRouter>
      <AuditAlertDetailDialog
        alert={alert}
        open
        onOpenChange={() => {}}
        onResolve={vi.fn()}
        isResolving={false}
      />
    </MemoryRouter>
  )
}

describe('安全警報詳情 — 來源摘要', () => {
  it('帶出來源 IP、裝置與前次登入來源，供比對是不是同一個人', () => {
    renderDialog({
      ...BASE_ALERT,
      context_data: {
        email: 'vet@example.com',
        ip: '203.0.113.9',
        triggers: ['new_device'],
        browser: 'Chrome',
        os: 'Windows',
        previous_login_ip: '198.51.100.4',
        previous_login_at: '2026-07-20T11:03:00Z',
      },
    })

    expect(screen.getByText('來源 IP')).toBeInTheDocument()
    expect(screen.getByText('203.0.113.9')).toBeInTheDocument()
    expect(screen.getByText('Chrome / Windows')).toBeInTheDocument()
    expect(screen.getByText('前次登入來源')).toBeInTheDocument()
    // 錨在開頭：下方的 context_data JSON 區塊也含這個 IP，
    // 「IP + 括號時間」的組合只有摘要那一行會產生
    expect(screen.getByText(/^198\.51\.100\.4 \(/)).toBeInTheDocument()
  })

  it('GeoIP 未安裝時 geo 欄位整個省略，不顯示空殼欄位', () => {
    renderDialog({
      ...BASE_ALERT,
      context_data: {
        email: 'vet@example.com',
        ip: '203.0.113.9',
        triggers: ['new_device'],
      },
    })

    expect(screen.getByText('來源 IP')).toBeInTheDocument()
    expect(screen.queryByText('地理位置')).not.toBeInTheDocument()
    expect(screen.queryByText('裝置')).not.toBeInTheDocument()
    expect(screen.queryByText('前次登入來源')).not.toBeInTheDocument()
  })

  it('沒有任何來源資訊時整塊摘要不渲染（舊警報不會多出空區塊）', () => {
    renderDialog({ ...BASE_ALERT, context_data: { email: 'vet@example.com' } })

    expect(screen.queryByText('來源 IP')).not.toBeInTheDocument()
    // 但既有的「詳細上下文資料」JSON 區塊仍在，不影響舊行為
    expect(screen.getByText('詳細上下文資料')).toBeInTheDocument()
  })
})
