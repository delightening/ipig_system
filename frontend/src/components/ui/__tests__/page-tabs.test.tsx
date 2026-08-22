import { render, screen } from '@testing-library/react'
import { PageTabs } from '../page-tabs'

// Mock react-router-dom's useSearchParams
const mockSetSearchParams = vi.fn()
let mockSearchParams = new URLSearchParams()

vi.mock('react-router-dom', () => ({
  useSearchParams: () => [mockSearchParams, mockSetSearchParams] as const,
}))

// Mock Radix Tabs to simplify testing (avoid Radix internals)
vi.mock('@/components/ui/tabs', () => ({
  Tabs: ({
    children,
    value,
    onValueChange,
  }: {
    children: React.ReactNode
    value: string
    onValueChange: (v: string) => void
    className?: string
  }) => (
    <div data-testid="tabs" data-value={value} data-onvaluechange={String(!!onValueChange)}>
      {children}
    </div>
  ),
  TabsList: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="tabs-list" role="tablist">
      {children}
    </div>
  ),
  TabsTrigger: ({
    children,
    value,
  }: {
    children: React.ReactNode
    value: string
    className?: string
  }) => (
    <button role="tab" data-value={value}>
      {children}
    </button>
  ),
  TabsContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}))

const baseTabs = [
  { value: 'overview', label: '總覽' },
  { value: 'details', label: '細節' },
  { value: 'settings', label: '設定' },
]

describe('PageTabs', () => {
  beforeEach(() => {
    mockSearchParams = new URLSearchParams()
    mockSetSearchParams.mockClear()
  })

  it('renders all visible tab labels', () => {
    render(
      <PageTabs tabs={baseTabs}>
        <div>Content</div>
      </PageTabs>
    )

    expect(screen.getByText('總覽')).toBeInTheDocument()
    expect(screen.getByText('細節')).toBeInTheDocument()
    expect(screen.getByText('設定')).toBeInTheDocument()
  })

  it('hides tabs that have hidden=true', () => {
    const tabsWithHidden = [
      { value: 'overview', label: '總覽' },
      { value: 'admin', label: '管理', hidden: true },
      { value: 'details', label: '細節' },
    ]

    render(
      <PageTabs tabs={tabsWithHidden}>
        <div>Content</div>
      </PageTabs>
    )

    expect(screen.getByText('總覽')).toBeInTheDocument()
    expect(screen.getByText('細節')).toBeInTheDocument()
    expect(screen.queryByText('管理')).not.toBeInTheDocument()
  })

  it('defaults to the first tab when no URL param is set', () => {
    render(
      <PageTabs tabs={baseTabs}>
        <div>Content</div>
      </PageTabs>
    )

    const tabsRoot = screen.getByTestId('tabs')
    expect(tabsRoot).toHaveAttribute('data-value', 'overview')
  })

  it('reads active tab from URL search params', () => {
    mockSearchParams = new URLSearchParams('tab=details')

    render(
      <PageTabs tabs={baseTabs}>
        <div>Content</div>
      </PageTabs>
    )

    const tabsRoot = screen.getByTestId('tabs')
    expect(tabsRoot).toHaveAttribute('data-value', 'details')
  })

  it('falls back to default when URL param has invalid value', () => {
    mockSearchParams = new URLSearchParams('tab=nonexistent')

    render(
      <PageTabs tabs={baseTabs}>
        <div>Content</div>
      </PageTabs>
    )

    const tabsRoot = screen.getByTestId('tabs')
    expect(tabsRoot).toHaveAttribute('data-value', 'overview')
  })

  it('uses custom paramKey to read from URL', () => {
    mockSearchParams = new URLSearchParams('section=settings')

    render(
      <PageTabs tabs={baseTabs} paramKey="section">
        <div>Content</div>
      </PageTabs>
    )

    const tabsRoot = screen.getByTestId('tabs')
    expect(tabsRoot).toHaveAttribute('data-value', 'settings')
  })

  it('uses defaultTab prop when no URL param is set', () => {
    render(
      <PageTabs tabs={baseTabs} defaultTab="details">
        <div>Content</div>
      </PageTabs>
    )

    const tabsRoot = screen.getByTestId('tabs')
    expect(tabsRoot).toHaveAttribute('data-value', 'details')
  })

  it('renders badge count when tab has badge > 0', () => {
    const tabsWithBadge = [
      { value: 'overview', label: '總覽', badge: 5 },
      { value: 'details', label: '細節', badge: 0 },
    ]

    render(
      <PageTabs tabs={tabsWithBadge}>
        <div>Content</div>
      </PageTabs>
    )

    expect(screen.getByText('5')).toBeInTheDocument()
    // badge=0 should not render
    expect(screen.queryByText('0')).not.toBeInTheDocument()
  })

  it('renders children content', () => {
    render(
      <PageTabs tabs={baseTabs}>
        <div>Tab Content Here</div>
      </PageTabs>
    )

    expect(screen.getByText('Tab Content Here')).toBeInTheDocument()
  })
})
