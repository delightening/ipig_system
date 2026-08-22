import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'

/** 每個測試自行決定「這個使用者有哪些權限」 */
const granted = new Set<string>()
vi.mock('@/stores/auth', () => ({
  useAuthHasPermission: () => (p: string) => granted.has(p),
}))

import { Can } from '@/components/auth/Can'
import { PERMISSIONS } from '@/lib/permissions.generated'

const CREATE = PERMISSIONS.ANIMAL_RECORD_CREATE
const EDIT = PERMISSIONS.ANIMAL_RECORD_EDIT

describe('Can', () => {
  beforeEach(() => granted.clear())

  it('有權限時渲染 children', () => {
    granted.add(CREATE)
    render(
      <Can permission={CREATE}>
        <button>新增</button>
      </Can>,
    )
    expect(screen.getByRole('button', { name: '新增' })).toBeInTheDocument()
  })

  // 使用者 2026-08-07 裁定：無權限「完全隱藏」，不是 disabled。
  // 這是本元件存在的理由，退化成 disabled 會把全部功能清單公開給每個使用者。
  it('無權限時什麼都不渲染（不是 disabled）', () => {
    const { container } = render(
      <Can permission={CREATE}>
        <button>新增</button>
      </Can>,
    )
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
    expect(container).toBeEmptyDOMElement()
  })

  it('無權限且有 fallback 時渲染 fallback', () => {
    render(
      <Can permission={CREATE} fallback={<span>需要權限</span>}>
        <button>新增</button>
      </Can>,
    )
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
    expect(screen.getByText('需要權限')).toBeInTheDocument()
  })

  it('anyOf：任一符合即渲染', () => {
    granted.add(EDIT)
    render(
      <Can anyOf={[CREATE, EDIT]}>
        <button>動作</button>
      </Can>,
    )
    expect(screen.getByRole('button')).toBeInTheDocument()
  })

  it('anyOf：全不符合則不渲染', () => {
    render(
      <Can anyOf={[CREATE, EDIT]}>
        <button>動作</button>
      </Can>,
    )
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
  })

  it('allOf：缺一則不渲染', () => {
    granted.add(CREATE)
    render(
      <Can allOf={[CREATE, EDIT]}>
        <button>動作</button>
      </Can>,
    )
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
  })

  it('allOf：全部符合才渲染', () => {
    granted.add(CREATE)
    granted.add(EDIT)
    render(
      <Can allOf={[CREATE, EDIT]}>
        <button>動作</button>
      </Can>,
    )
    expect(screen.getByRole('button')).toBeInTheDocument()
  })

  it('未指定任何條件時無條件渲染', () => {
    render(
      <Can>
        <button>動作</button>
      </Can>,
    )
    expect(screen.getByRole('button')).toBeInTheDocument()
  })

  // permission 與 anyOf 併用時是 AND —— 避免呼叫端誤以為多寫一個條件會「放寬」。
  it('permission 與 anyOf 併用時取交集', () => {
    granted.add(EDIT)
    render(
      <Can permission={CREATE} anyOf={[EDIT]}>
        <button>動作</button>
      </Can>,
    )
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
  })
})
