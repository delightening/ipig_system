import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import { vi } from 'vitest'

import i18n from '@/lib/i18n'
import type { ProtocolStatus } from '@/types'

/**
 * 代理徽章的顯示條件（CodeRabbit #53 第六輪）。
 *
 * 徽章原本嵌在「建立修正案」按鈕的條件裡（`canCreateAmendment && canManageAmendment`），
 * 但送審既有 DRAFT 的按鈕只看 `canManageAmendment`——`canCreateAmendment` 講的是
 * 「計畫狀態允許新增」。兩者綁在一起的結果：計畫離開 APPROVED 之後，代理人仍能送審，
 * 畫面卻不再顯示自己是以代理身分在操作。這個徽章的用途正是讓操作者知道自己
 * 以誰的名義行事，掉了不會有任何型別或既有測試擋住。
 */

const apiGet = vi.fn()
const hasPermission = vi.fn(() => false)

vi.mock('@/lib/api', async () => {
  const actual = await vi.importActual<typeof import('@/lib/api')>('@/lib/api')
  return {
    ...actual,
    default: { get: (...a: unknown[]) => apiGet(...a), post: vi.fn() },
  }
})

vi.mock('@/components/ui/use-toast', () => ({ toast: vi.fn() }))

vi.mock('@/stores/auth', () => ({
  useAuthStore: () => ({ hasPermission }),
}))

const { AmendmentsTab } = await import('../AmendmentsTab')

function renderTab(
  protocolStatus: ProtocolStatus,
  opts: { isPiDelegate?: boolean; canWriteAmendment?: boolean } = {}
) {
  apiGet.mockResolvedValue({ data: [] })
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <AmendmentsTab
        protocolId="protocol-1"
        protocolStatus={protocolStatus}
        isImported={false}
        isStudyDirector={false}
        canWriteAmendment={opts.canWriteAmendment ?? true}
        isPiDelegate={opts.isPiDelegate ?? true}
      />
    </QueryClientProvider>
  )
}

let previousLanguage: string

beforeAll(async () => {
  previousLanguage = i18n.language
  await i18n.changeLanguage('zh-TW')
})

afterAll(async () => {
  await i18n.changeLanguage(previousLanguage)
})

beforeEach(() => {
  apiGet.mockReset()
  hasPermission.mockReturnValue(false)
})

const BADGE = '以 PI 代理人身分操作'

describe('修正案分頁的 PI 代理徽章', () => {
  it('計畫可新增修正案時顯示徽章', async () => {
    renderTab('APPROVED')
    expect(await screen.findByText(BADGE)).toBeInTheDocument()
  })

  // 這條就是修正的核心：SUSPENDED 不在可新增名單內，但代理人仍可送審既有 DRAFT。
  it('計畫已不可新增、但仍可送審既有草稿時，徽章不得消失', async () => {
    renderTab('SUSPENDED')
    expect(await screen.findByText(BADGE)).toBeInTheDocument()
  })

  it('不是代理人就不顯示徽章', async () => {
    renderTab('APPROVED', { isPiDelegate: false })
    await waitFor(() => expect(apiGet).toHaveBeenCalled())
    expect(screen.queryByText(BADGE)).not.toBeInTheDocument()
  })

  it('沒有修正案寫入權就不顯示徽章（徽章不該暗示自己有權限）', async () => {
    renderTab('APPROVED', { canWriteAmendment: false })
    await waitFor(() => expect(apiGet).toHaveBeenCalled())
    expect(screen.queryByText(BADGE)).not.toBeInTheDocument()
  })
})
