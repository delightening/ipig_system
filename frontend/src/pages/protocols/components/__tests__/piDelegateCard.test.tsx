import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { vi } from 'vitest'

import i18n from '@/lib/i18n'
import type { ProtocolResponse } from '@/types'

/**
 * PI 代理授權卡片（外部 PI 尚未開通帳號前，由 SD 核准的代簽人）。
 *
 * 釘住四件在重構中最容易走鐘、而且走鐘後不會有任何紅燈的行為：
 * 1. `pi_is_external=false` 的計畫**整張卡片不得出現**——內部 PI 沒有借位問題，
 *    出現這張卡等於在鼓勵不必要的代簽。
 * 2. 沒有生效授權時要明講「尚未核准代理人」，而不是整塊留白（留白會被讀成
 *    「這個計畫不支援代理」，與事實相反）。
 * 3. 撤銷按鈕只給 SD／執秘看得到；無關人員連按鈕都不該有。這層只是方便使用者，
 *    真正把關在後端 `services/protocol/pi_delegate.rs`。
 * 4. 目前使用者就是代理人時要掛「以代理人身分操作中」徽章——這是稽核可歸責性的
 *    視覺提示，掉了不會有任何測試或型別擋住。
 */

const PROTOCOL_ID = 'protocol-1'
const SD_ID = 'user-sd'
const DELEGATE_ID = 'user-delegate'
const OUTSIDER_ID = 'user-outsider'

const apiGet = vi.fn()
const authorizePiDelegate = vi.fn().mockResolvedValue({})
const revokePiDelegate = vi.fn().mockResolvedValue(undefined)
const currentUser = vi.fn()

vi.mock('@/lib/api', () => ({
  default: {
    get: (...args: unknown[]) => apiGet(...args),
  },
}))

vi.mock('@/lib/api/protocol', () => ({
  authorizePiDelegate: (...args: unknown[]) => authorizePiDelegate(...args),
  revokePiDelegate: (...args: unknown[]) => revokePiDelegate(...args),
}))

vi.mock('@/components/ui/use-toast', () => ({
  toast: vi.fn(),
}))

vi.mock('@/stores/auth', () => ({
  useAuthUser: () => currentUser(),
}))

// 於 vi.mock 之後 import，確保元件拿到的是 mock 版本
const { PiDelegateCard } = await import('../PiDelegateCard')

type ProtocolResponseFixture = Pick<ProtocolResponse, 'protocol' | 'pi_delegate' | 'is_pi_delegate'>

const DELEGATE_INFO = {
  id: 'delegation-1',
  delegate_user_id: DELEGATE_ID,
  delegate_name: '代簽人小陳',
  authorized_by: SD_ID,
  authorized_by_name: '負責人小李',
  authorized_at: '2026-08-30T02:00:00Z',
  reason: null,
}

function protocolResponse(
  overrides: Partial<ProtocolResponseFixture['protocol']> = {},
  rest: Partial<Omit<ProtocolResponseFixture, 'protocol'>> = {}
) {
  return {
    protocol: {
      id: PROTOCOL_ID,
      pi_is_external: true,
      study_director_user_id: SD_ID,
      ...overrides,
    },
    pi_delegate: null,
    is_pi_delegate: false,
    ...rest,
  }
}

function renderCard(response: ReturnType<typeof protocolResponse>) {
  apiGet.mockImplementation((url: string) =>
    url === '/protocols/assignable-users'
      ? Promise.resolve({ data: [{ id: DELEGATE_ID, display_name: '代簽人小陳' }] })
      : Promise.resolve({ data: response })
  )
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <PiDelegateCard protocolId={PROTOCOL_ID} />
    </QueryClientProvider>
  )
}

// 元件的字串全部走 i18n；斷言用的是 zh-TW 的值，所以把語系釘死，
// 不讓 LanguageDetector 依執行環境（localStorage / navigator）飄移。
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
  authorizePiDelegate.mockClear()
  revokePiDelegate.mockClear()
  currentUser.mockReturnValue({ id: SD_ID, roles: ['EXPERIMENT_STAFF'] })
})

describe('PI 代理授權卡片', () => {
  it('PI 非外部人員時整張卡片不顯示', async () => {
    const { container } = renderCard(protocolResponse({ pi_is_external: false }))
    await waitFor(() => expect(apiGet).toHaveBeenCalled())
    expect(container).toBeEmptyDOMElement()
  })

  it('外部 PI 且尚無授權時，SD 看得到說明與核准按鈕（未選人前不可按）', async () => {
    renderCard(protocolResponse())

    expect(await screen.findByText('尚未核准代理人。')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /核准為代理人/ })).toBeDisabled()
    expect(authorizePiDelegate).not.toHaveBeenCalled()
  })

  it('已有代理人時顯示核可證據，SD 按撤銷會呼叫撤銷 API', async () => {
    renderCard(protocolResponse({}, { pi_delegate: DELEGATE_INFO }))

    expect(await screen.findByText('代簽人小陳')).toBeInTheDocument()
    expect(screen.getByText(/由 負責人小李 於/)).toBeInTheDocument()
    // 已有生效授權時不再提供「再核准一位」的表單（後端同時只允許一筆）
    expect(screen.queryByRole('button', { name: /核准為代理人/ })).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: /撤銷/ }))
    await waitFor(() => expect(revokePiDelegate).toHaveBeenCalledWith(PROTOCOL_ID))
  })

  it('非 SD 也非執秘的使用者看得到現況，但沒有撤銷按鈕', async () => {
    currentUser.mockReturnValue({ id: OUTSIDER_ID, roles: ['PI'] })
    renderCard(protocolResponse({}, { pi_delegate: DELEGATE_INFO }))

    expect(await screen.findByText('代簽人小陳')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /撤銷/ })).not.toBeInTheDocument()
  })

  it('目前使用者本人就是代理人時掛上「以代理人身分操作中」徽章', async () => {
    currentUser.mockReturnValue({ id: DELEGATE_ID, roles: [] })
    renderCard(protocolResponse({}, { pi_delegate: DELEGATE_INFO, is_pi_delegate: true }))

    expect(await screen.findByText('以代理人身分操作中')).toBeInTheDocument()
  })

  it('執行秘書即使不是 SD 也能撤銷（鏡像後端的 escalation 規則）', async () => {
    currentUser.mockReturnValue({ id: OUTSIDER_ID, roles: ['IACUC_STAFF'] })
    renderCard(protocolResponse({}, { pi_delegate: DELEGATE_INFO }))

    fireEvent.click(await screen.findByRole('button', { name: /撤銷/ }))
    await waitFor(() => expect(revokePiDelegate).toHaveBeenCalledWith(PROTOCOL_ID))
  })
})
