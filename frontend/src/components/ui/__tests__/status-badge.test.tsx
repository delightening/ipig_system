import { render, screen } from '@testing-library/react'
import { StatusBadge, type StatusVariant } from '../status-badge'

const variants: StatusVariant[] = ['success', 'warning', 'error', 'info', 'neutral', 'purple']

describe('StatusBadge', () => {
  it.each(variants)('renders with variant "%s"', (variant) => {
    render(<StatusBadge variant={variant}>{variant}</StatusBadge>)
    expect(screen.getByText(variant)).toBeInTheDocument()
  })

  it('renders children text content', () => {
    render(<StatusBadge variant="success">已核准</StatusBadge>)
    expect(screen.getByText('已核准')).toBeInTheDocument()
  })

  it('does not render dot by default', () => {
    const { container } = render(
      <StatusBadge variant="success">Active</StatusBadge>
    )
    // The outer span has one child: the text. No dot span.
    const badge = container.firstElementChild!
    expect(badge.children.length).toBe(0) // text node, no child elements
  })

  it('renders a dot indicator when dot prop is true', () => {
    const { container } = render(
      <StatusBadge variant="error" dot>
        Error
      </StatusBadge>
    )
    const badge = container.firstElementChild!
    // With dot=true, there should be a child span element for the dot
    const dotElement = badge.querySelector('span')
    expect(dotElement).toBeInTheDocument()
    expect(dotElement).toHaveClass('rounded-full')
  })

  it('applies variant-specific classes to the outer element', () => {
    const { container } = render(
      <StatusBadge variant="warning">Warning</StatusBadge>
    )
    const badge = container.firstElementChild!
    // 2026-07-03 全域統一：StatusBadge 預設 solid（實心 + 白字）
    expect(badge).toHaveClass('bg-status-warning-solid')
    expect(badge).toHaveClass('text-white')
    expect(badge).toHaveClass('border-transparent')
  })

  it('defaults to solid tone when tone prop is omitted', () => {
    const { container } = render(<StatusBadge variant="success">OK</StatusBadge>)
    const badge = container.firstElementChild!
    expect(badge).toHaveClass('bg-status-success-solid')
    expect(badge).toHaveClass('text-white')
  })

  it('applies soft-tier classes when tone="soft" (DESIGN.md §3.4)', () => {
    const { container } = render(
      <StatusBadge variant="success" tone="soft">
        實驗中
      </StatusBadge>
    )
    const badge = container.firstElementChild!
    // soft = 淺底 + 深字 + 淺框（表格成群密集標籤）
    expect(badge).toHaveClass('bg-status-success-bg')
    expect(badge).toHaveClass('text-status-success-text')
    expect(badge).toHaveClass('border-status-success-border')
    expect(badge).not.toHaveClass('text-white')
  })

  it('renders a soft-tone dot with follow-text classes and aria-hidden', () => {
    const { container } = render(
      <StatusBadge variant="success" tone="soft" dot>
        實驗中
      </StatusBadge>
    )
    const badge = container.firstElementChild!
    const dotElement = badge.querySelector('span')
    expect(dotElement).toBeInTheDocument()
    // soft 深字 → dot 跟隨文字色
    expect(dotElement).toHaveClass('bg-current')
    expect(dotElement).toHaveClass('opacity-70')
    // 純裝飾，不朗讀
    expect(dotElement).toHaveAttribute('aria-hidden', 'true')
  })

  it('renders a solid-tone dot with white fill', () => {
    const { container } = render(
      <StatusBadge variant="error" dot>
        錯誤
      </StatusBadge>
    )
    const dotElement = container.firstElementChild!.querySelector('span')
    expect(dotElement).toHaveClass('bg-white/85')
  })

  it('applies custom className via className prop', () => {
    const { container } = render(
      <StatusBadge variant="info" className="ml-4">
        Info
      </StatusBadge>
    )
    const badge = container.firstElementChild!
    expect(badge).toHaveClass('ml-4')
  })

  it('renders as an inline-flex span element', () => {
    const { container } = render(
      <StatusBadge variant="neutral">Neutral</StatusBadge>
    )
    const badge = container.firstElementChild!
    expect(badge.tagName).toBe('SPAN')
    expect(badge).toHaveClass('inline-flex')
  })
})
