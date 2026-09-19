import { describe, it, expect, vi, beforeEach } from 'vitest'
// 用 fireEvent/純渲染而非 user-event：@testing-library/user-event 不在專案依賴內
import { render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

// t() 回傳 key 本身（帶 count 時附上 `:count`，好讓「顯示人數」仍能被斷言），
// 斷言比對 key，不受文案改寫與語系影響。
// `lib/utils` 會拉進 `lib/i18n`，後者 `.use(initReactI18next)`——mock 掉整個模組就得補上
// 這個 plugin 形狀，否則 i18next 在 import 階段就炸。
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { count?: number }) =>
      opts && opts.count !== undefined ? `${key}:${opts.count}` : key,
  }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

const K = 'adminUsers.facilities.committeeRoster'

const listCommitteeMembers = vi.fn()
vi.mock('@/lib/api/facility', () => ({
  facilityApi: {
    listCommitteeMembers: (...args: unknown[]) => listCommitteeMembers(...args),
  },
}))

import { CommitteeRoster } from '@/pages/admin/components/CommitteeRoster'
import type { CommitteeMember } from '@/types/facility'

function makeMember(overrides: Partial<CommitteeMember> = {}): CommitteeMember {
  return {
    id: 'u-1',
    display_name: '甲委員',
    department_name: '獸醫部',
    is_internal: true,
    ...overrides,
  }
}

function renderRoster() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <CommitteeRoster />
    </QueryClientProvider>,
  )
}

describe('CommitteeRoster', () => {
  beforeEach(() => {
    listCommitteeMembers.mockReset()
  })

  it('載入中顯示骨架，且不顯示空狀態', () => {
    listCommitteeMembers.mockReturnValue(new Promise(() => {}))

    const { container } = renderRoster()

    expect(container.querySelectorAll('.animate-pulse')).toHaveLength(3)
    expect(screen.queryByText(`${K}.emptyTitle`)).not.toBeInTheDocument()
  })

  it('無委員時顯示空狀態，標題不顯示人數', async () => {
    listCommitteeMembers.mockResolvedValue({ data: [] })

    renderRoster()

    expect(await screen.findByText(`${K}.emptyTitle`)).toBeInTheDocument()
    expect(screen.queryByText(new RegExp(`${K}\\.memberCount`))).not.toBeInTheDocument()
  })

  // 這支釘住名冊的核心契約：委員的部門欄顯示**實際歸屬**而非 IACUC。
  // 若日後有人把名冊改成查「IACUC 部門的成員」，兼任審查委員的獸醫會靜默
  // 消失於名冊——畫面上只是少一列，不會有任何錯誤。
  it('列出委員與實際歸屬部門；無部門者顯示替代文字，外部人員加標籤', async () => {
    listCommitteeMembers.mockResolvedValue({
      data: [
        makeMember({ id: 'u-1', display_name: '甲委員', department_name: '獸醫部', is_internal: true }),
        makeMember({ id: 'u-2', display_name: '乙委員', department_name: null, is_internal: false }),
      ],
    })

    renderRoster()

    expect(await screen.findByText('甲委員')).toBeInTheDocument()
    expect(screen.getByText('獸醫部')).toBeInTheDocument()
    expect(screen.getByText('乙委員')).toBeInTheDocument()
    expect(screen.getByText(`${K}.noDepartment`)).toBeInTheDocument()
    // 外部人員一律加標籤：名單放寬後不標示的話，看不出哪些人不適用人事作業
    expect(screen.getByText('adminUsers.shared.externalStaffTag')).toBeInTheDocument()
    expect(screen.getByText(`${K}.memberCount:2`)).toBeInTheDocument()
  })

  // 這支釘住「載入失敗 ≠ 沒有委員」：兩種情形 members 都是空陣列，只有 isError
  // 分得開。少了它，403／斷線會顯示「尚無委員」並叫管理員去指派角色——而角色
  // 其實早就指派好了，照做只是白忙，真正的失敗原因反而被蓋掉。
  it('載入失敗時顯示錯誤訊息，而非空狀態', async () => {
    listCommitteeMembers.mockRejectedValue(new Error('403'))

    renderRoster()

    expect(await screen.findByText(`${K}.loadFailed`)).toBeInTheDocument()
    expect(screen.queryByText(`${K}.emptyTitle`)).not.toBeInTheDocument()
  })

  it('內部委員不加外部標籤', async () => {
    listCommitteeMembers.mockResolvedValue({ data: [makeMember()] })

    renderRoster()

    expect(await screen.findByText('甲委員')).toBeInTheDocument()
    expect(screen.queryByText('adminUsers.shared.externalStaffTag')).not.toBeInTheDocument()
  })
})
