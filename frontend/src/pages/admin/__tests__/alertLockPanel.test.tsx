import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { vi } from 'vitest'
import i18n from '@/lib/i18n'
import type { AlertLockStatus } from '@/lib/api/alertLock'

/**
 * 安全警報詳情的解鎖面板。
 *
 * 釘住三件容易在重構中走鐘的行為：
 * 1. 解除原因必填——原因會寫進 HMAC 稽核鏈，空白按鈕就不能按。
 * 2. 帳號解鎖與 IP 解封是兩顆獨立按鈕，各自呼叫各自的 API，不互相牽動。
 * 3. 帳號查得到但沒被鎖時仍要顯示一行「未鎖定」，而不是整塊消失——
 *    否則管理員會以為功能壞了，而不是「本來就進得去」。
 */

const ALERT_ID = 'alert-1'

const clearAccountLockout = vi.fn().mockResolvedValue({ data: { ok: true } })
const unblock = vi.fn().mockResolvedValue({ data: { ok: true } })
const status = vi.fn()

vi.mock('@/lib/api/alertLock', () => ({
  alertLockApi: {
    status: (...args: unknown[]) => status(...args),
    clearAccountLockout: (...args: unknown[]) => clearAccountLockout(...args),
  },
}))

vi.mock('@/lib/api/ipBlocklist', () => ({
  ipBlocklistApi: {
    unblock: (...args: unknown[]) => unblock(...args),
  },
}))

vi.mock('@/components/ui/use-toast', () => ({
  toast: vi.fn(),
}))

// 於 vi.mock 之後 import，確保元件拿到的是 mock 版本
const { AlertLockPanel } = await import('../components/AlertLockPanel')

let previousLanguage: string

beforeAll(async () => {
  previousLanguage = i18n.language
  await i18n.changeLanguage('zh-TW')
})

afterAll(async () => {
  await i18n.changeLanguage(previousLanguage)
})

beforeEach(() => {
  clearAccountLockout.mockClear()
  unblock.mockClear()
  status.mockReset()
})

const LOCKED_ACCOUNT: AlertLockStatus = {
  account: {
    email: 'vet@example.com',
    user_exists: true,
    locked: true,
    fail_count: 5,
    max_attempts: 5,
    unlock_at: '2026-07-29T07:27:00Z',
    cleared_at: null,
  },
  ip: null,
}

const BLOCKED_IP: AlertLockStatus = {
  account: null,
  ip: {
    id: 'block-1',
    ip_address: '203.0.113.9',
    reason: 'IDOR 探測',
    source: 'R22-6_idor',
    blocked_until: null,
  },
}

function renderPanel(lock: AlertLockStatus) {
  status.mockResolvedValue({ data: lock })
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  render(
    <QueryClientProvider client={queryClient}>
      <AlertLockPanel alertId={ALERT_ID} />
    </QueryClientProvider>
  )
}

function fillReason(reason: string) {
  fireEvent.change(screen.getByLabelText(/解除原因/), { target: { value: reason } })
}

describe('安全警報 — 解鎖面板', () => {
  it('原因空白時解鎖按鈕不可按，填入後才會送出並帶上 email 與原因', async () => {
    renderPanel(LOCKED_ACCOUNT)

    const button = await screen.findByRole('button', { name: /解除帳號鎖定/ })
    expect(button).toBeDisabled()

    fillReason('已與本人確認為忘記密碼')
    expect(button).not.toBeDisabled()

    fireEvent.click(button)
    await waitFor(() => {
      expect(clearAccountLockout).toHaveBeenCalledWith(
        'vet@example.com',
        '已與本人確認為忘記密碼'
      )
    })
    // 帳號解鎖不應牽動 IP 封鎖
    expect(unblock).not.toHaveBeenCalled()
  })

  it('IP 封鎖中會顯示該 IP，解封送出 blocklist id 與原因', async () => {
    renderPanel(BLOCKED_IP)

    expect(await screen.findByText('203.0.113.9')).toBeInTheDocument()
    const button = screen.getByRole('button', { name: /解除 IP 封鎖/ })
    expect(button).toBeDisabled()

    fillReason('誤判，已確認為院內固定 IP')
    fireEvent.click(button)
    await waitFor(() => {
      expect(unblock).toHaveBeenCalledWith('block-1', '誤判，已確認為院內固定 IP')
    })
    expect(clearAccountLockout).not.toHaveBeenCalled()
  })

  it('帳號存在但未被鎖時，仍顯示「未鎖定」且不給解鎖按鈕', async () => {
    renderPanel({
      account: { ...LOCKED_ACCOUNT.account!, locked: false, fail_count: 1 },
      ip: null,
    })

    expect(await screen.findByText('帳號未鎖定')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /解除帳號鎖定/ })).not.toBeInTheDocument()
    // 沒有任何可解除的對象 → 不顯示原因輸入框
    expect(screen.queryByLabelText(/解除原因/)).not.toBeInTheDocument()
  })
})
