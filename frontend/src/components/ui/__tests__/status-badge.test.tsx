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
    expect(badge).toHaveClass('bg-status-warning-bg')
    expect(badge).toHaveClass('text-status-warning-text')
    expect(badge).toHaveClass('border-status-warning-border')
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
