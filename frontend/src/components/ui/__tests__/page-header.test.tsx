import { render, screen } from '@testing-library/react'
import { PageHeader } from '../page-header'

describe('PageHeader', () => {
  it('renders the title as an h1 element', () => {
    render(<PageHeader title="動物管理" />)

    const heading = screen.getByRole('heading', { level: 1 })
    expect(heading).toHaveTextContent('動物管理')
  })

  it('renders description when provided', () => {
    render(
      <PageHeader title="動物管理" description="管理所有實驗動物資料" />
    )

    expect(screen.getByText('管理所有實驗動物資料')).toBeInTheDocument()
  })

  it('does not render description when not provided', () => {
    const { container } = render(<PageHeader title="動物管理" />)

    const descriptionP = container.querySelector('p')
    expect(descriptionP).toBeNull()
  })

  it('renders actions slot when provided', () => {
    render(
      <PageHeader
        title="動物管理"
        actions={<button>新增動物</button>}
      />
    )

    expect(screen.getByText('新增動物')).toBeInTheDocument()
  })

  it('does not render actions wrapper when actions is not provided', () => {
    const { container } = render(<PageHeader title="標題" />)

    // Only one child div (the title section)
    const root = container.firstElementChild!
    expect(root.children.length).toBe(1)
  })

  it('renders multiple action buttons', () => {
    render(
      <PageHeader
        title="動物管理"
        actions={
          <>
            <button>匯出</button>
            <button>新增</button>
          </>
        }
      />
    )

    expect(screen.getByText('匯出')).toBeInTheDocument()
    expect(screen.getByText('新增')).toBeInTheDocument()
  })

  it('renders children alongside title and actions', () => {
    render(
      <PageHeader title="標題" actions={<button>操作</button>}>
        <div data-testid="extra-content">Extra</div>
      </PageHeader>
    )

    expect(screen.getByTestId('extra-content')).toBeInTheDocument()
    expect(screen.getByText('操作')).toBeInTheDocument()
  })

  it('applies custom className to the root element', () => {
    const { container } = render(
      <PageHeader title="標題" className="mb-8" />
    )

    expect(container.firstElementChild).toHaveClass('mb-8')
  })
})
