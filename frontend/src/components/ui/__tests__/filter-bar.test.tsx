import { render, screen, fireEvent } from '@testing-library/react'
import { FilterBar } from '../filter-bar'
import i18n from '@/lib/i18n'

describe('FilterBar', () => {
  it('renders search input when onSearchChange is provided', () => {
    render(
      <FilterBar search="" onSearchChange={vi.fn()} />
    )

    expect(screen.getByPlaceholderText(i18n.t('common.searchEllipsis'))).toBeInTheDocument()
  })

  it('does not render search input when onSearchChange is undefined', () => {
    render(<FilterBar />)

    expect(screen.queryByPlaceholderText(i18n.t('common.searchEllipsis'))).not.toBeInTheDocument()
  })

  it('uses custom searchPlaceholder', () => {
    render(
      <FilterBar
        search=""
        onSearchChange={vi.fn()}
        searchPlaceholder="搜尋動物..."
      />
    )

    expect(screen.getByPlaceholderText('搜尋動物...')).toBeInTheDocument()
  })

  it('displays the current search value', () => {
    render(
      <FilterBar search="hello" onSearchChange={vi.fn()} />
    )

    const input = screen.getByPlaceholderText(i18n.t('common.searchEllipsis')) as HTMLInputElement
    expect(input.value).toBe('hello')
  })

  it('calls onSearchChange when input value changes', () => {
    const onSearchChange = vi.fn()

    render(
      <FilterBar search="" onSearchChange={onSearchChange} />
    )

    const input = screen.getByPlaceholderText(i18n.t('common.searchEllipsis'))
    fireEvent.change(input, { target: { value: 'test' } })

    expect(onSearchChange).toHaveBeenCalledWith('test')
  })

  it('shows clear button when hasActiveFilters is true and onClearFilters is provided', () => {
    const onClearFilters = vi.fn()

    render(
      <FilterBar hasActiveFilters={true} onClearFilters={onClearFilters} />
    )

    const clearButton = screen.getByText(i18n.t('common.clearFilters'))
    expect(clearButton).toBeInTheDocument()
  })

  it('does not show clear button when hasActiveFilters is false', () => {
    render(
      <FilterBar hasActiveFilters={false} onClearFilters={vi.fn()} />
    )

    expect(screen.queryByText(i18n.t('common.clearFilters'))).not.toBeInTheDocument()
  })

  it('does not show clear button when onClearFilters is undefined', () => {
    render(
      <FilterBar hasActiveFilters={true} />
    )

    expect(screen.queryByText(i18n.t('common.clearFilters'))).not.toBeInTheDocument()
  })

  it('calls onClearFilters when clear button is clicked', () => {
    const onClearFilters = vi.fn()

    render(
      <FilterBar hasActiveFilters={true} onClearFilters={onClearFilters} />
    )

    fireEvent.click(screen.getByText(i18n.t('common.clearFilters')))
    expect(onClearFilters).toHaveBeenCalledTimes(1)
  })

  it('renders children (filter slots)', () => {
    render(
      <FilterBar>
        <select data-testid="custom-filter">
          <option>All</option>
        </select>
      </FilterBar>
    )

    expect(screen.getByTestId('custom-filter')).toBeInTheDocument()
  })

  it('applies custom className', () => {
    const { container } = render(
      <FilterBar className="my-custom-class" />
    )

    expect(container.firstElementChild).toHaveClass('my-custom-class')
  })
})
