import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter } from 'react-router-dom'

const apiGet = vi.fn()
vi.mock('@/lib/api', () => ({
  default: { get: (...args: unknown[]) => apiGet(...args) },
}))

vi.mock('@/stores/auth', () => ({
  useAuthUser: () => ({
    display_name: '測試管理員',
    roles: ['admin'],
    permissions: [],
    trainings: [],
  }),
  useAuthHasRole: () => (role: string) => role === 'admin',
}))

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { defaultValue?: string }) => opts?.defaultValue ?? key,
  }),
}))

import { RoleWelcomeGuide } from '@/components/dashboard/RoleWelcomeGuide'

/** mock 的 t() 會回傳 welcomeTitle 的 defaultValue，用它判斷橫幅是否在畫面上 */
const BANNER_TEXT = '歡迎，測試管理員！'

function renderGuide() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <RoleWelcomeGuide />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

describe('RoleWelcomeGuide', () => {
  beforeEach(() => {
    apiGet.mockReset()
    sessionStorage.clear()
  })

  // 這支釘住登入時的版面跳動：show_welcome_guide 偏好尚未回來前若先把橫幅畫出來，
  // 偏好回報「已關閉」時橫幅會整塊消失，把下方的 widget 網格往上拉約 100px。
  it('偏好尚未取回時不渲染橫幅', () => {
    apiGet.mockReturnValue(new Promise(() => {}))

    renderGuide()

    expect(screen.queryByText(BANNER_TEXT)).not.toBeInTheDocument()
  })

  it('偏好為 false（使用者已關閉）時不渲染橫幅', async () => {
    apiGet.mockResolvedValue({ data: { key: 'show_welcome_guide', value: false } })

    renderGuide()

    await waitFor(() => expect(apiGet).toHaveBeenCalled())
    await waitFor(() => expect(screen.queryByText(BANNER_TEXT)).not.toBeInTheDocument())
  })

  it('偏好為 true 時渲染橫幅', async () => {
    apiGet.mockResolvedValue({ data: { key: 'show_welcome_guide', value: true } })

    renderGuide()

    expect(await screen.findByText(BANNER_TEXT)).toBeInTheDocument()
  })
})
