import { render, screen, fireEvent } from '@testing-library/react'
import { DataTable, type ColumnDef } from '../data-table'
import { Inbox } from 'lucide-react'

// Mock sub-components to keep tests focused on DataTable logic
vi.mock('@/components/ui/table-skeleton', () => ({
  TableSkeleton: ({ rows, cols }: { rows: number; cols: number }) => (
    <tr data-testid="table-skeleton">
      <td>{`skeleton-${rows}x${cols}`}</td>
    </tr>
  ),
}))

vi.mock('@/components/ui/empty-state', () => ({
  TableEmptyRow: ({
    colSpan,
    title,
    description,
  }: {
    colSpan: number
    icon: unknown
    title: string
    description?: string
  }) => (
    <tr data-testid="empty-state">
      <td colSpan={colSpan}>
        <span>{title}</span>
        {description && <span>{description}</span>}
      </td>
    </tr>
  ),
}))

interface TestRow {
  id: number
  name: string
  status: string
}

const columns: ColumnDef<TestRow>[] = [
  { key: 'id', header: 'ID', cell: (row) => row.id },
  { key: 'name', header: '名稱', cell: (row) => row.name },
  { key: 'status', header: '狀態', cell: (row) => row.status },
]

const sampleData: TestRow[] = [
  { id: 1, name: 'Alpha', status: 'active' },
  { id: 2, name: 'Beta', status: 'inactive' },
]

describe('DataTable', () => {
  it('renders column headers correctly', () => {
    render(
      <DataTable
        columns={columns}
        data={sampleData}
        rowKey={(row) => row.id}
      />
    )

    expect(screen.getByText('ID')).toBeInTheDocument()
    expect(screen.getByText('名稱')).toBeInTheDocument()
    expect(screen.getByText('狀態')).toBeInTheDocument()
  })

  it('renders data rows with correct content', () => {
    render(
      <DataTable
        columns={columns}
        data={sampleData}
        rowKey={(row) => row.id}
      />
    )

    expect(screen.getByText('Alpha')).toBeInTheDocument()
    expect(screen.getByText('Beta')).toBeInTheDocument()
    expect(screen.getByText('active')).toBeInTheDocument()
    expect(screen.getByText('inactive')).toBeInTheDocument()
  })

  it('shows loading skeleton when isLoading is true', () => {
    render(
      <DataTable
        columns={columns}
        data={undefined}
        isLoading={true}
        skeletonRows={3}
        rowKey={(row) => row.id}
      />
    )

    expect(screen.getByTestId('table-skeleton')).toBeInTheDocument()
    expect(screen.getByText('skeleton-3x3')).toBeInTheDocument()
  })

  it('shows default empty state when data is empty and no emptyIcon', () => {
    render(
      <DataTable
        columns={columns}
        data={[]}
        rowKey={(row) => row.id}
      />
    )

    expect(screen.getByText('尚無資料')).toBeInTheDocument()
  })

  it('shows custom empty state with icon when emptyIcon is provided', () => {
    render(
      <DataTable
        columns={columns}
        data={[]}
        rowKey={(row) => row.id}
        emptyIcon={Inbox}
        emptyTitle="沒有紀錄"
        emptyDescription="請新增一筆"
      />
    )

    expect(screen.getByTestId('empty-state')).toBeInTheDocument()
    expect(screen.getByText('沒有紀錄')).toBeInTheDocument()
    expect(screen.getByText('請新增一筆')).toBeInTheDocument()
  })

  it('does not render pagination when totalPages <= 1', () => {
    render(
      <DataTable
        columns={columns}
        data={sampleData}
        rowKey={(row) => row.id}
        page={1}
        totalPages={1}
        onPageChange={vi.fn()}
      />
    )

    // Pagination should not be rendered
    expect(screen.queryByText('1 / 1')).not.toBeInTheDocument()
  })

  it('renders pagination controls and handles page changes', () => {
    const onPageChange = vi.fn()

    render(
      <DataTable
        columns={columns}
        data={sampleData}
        rowKey={(row) => row.id}
        page={2}
        totalPages={5}
        totalItems={50}
        onPageChange={onPageChange}
      />
    )

    // Should show page info
    expect(screen.getByText('2 / 5')).toBeInTheDocument()
    expect(screen.getByText('共 50 筆')).toBeInTheDocument()

    // Click next page
    const buttons = screen.getAllByRole('button')
    const nextButton = buttons[buttons.length - 1]
    fireEvent.click(nextButton)
    expect(onPageChange).toHaveBeenCalledWith(3)

    // Click previous page
    const prevButton = buttons[buttons.length - 2]
    fireEvent.click(prevButton)
    expect(onPageChange).toHaveBeenCalledWith(1)
  })

  it('disables prev button on first page and next button on last page', () => {
    const onPageChange = vi.fn()

    const { rerender } = render(
      <DataTable
        columns={columns}
        data={sampleData}
        rowKey={(row) => row.id}
        page={1}
        totalPages={3}
        onPageChange={onPageChange}
      />
    )

    const buttons = screen.getAllByRole('button')
    const prevButton = buttons[buttons.length - 2]
    expect(prevButton).toBeDisabled()

    rerender(
      <DataTable
        columns={columns}
        data={sampleData}
        rowKey={(row) => row.id}
        page={3}
        totalPages={3}
        onPageChange={onPageChange}
      />
    )

    const buttonsAfter = screen.getAllByRole('button')
    const nextButton = buttonsAfter[buttonsAfter.length - 1]
    expect(nextButton).toBeDisabled()
  })

  it('calls onRowClick when a data row is clicked', () => {
    const onRowClick = vi.fn()

    render(
      <DataTable
        columns={columns}
        data={sampleData}
        rowKey={(row) => row.id}
        onRowClick={onRowClick}
      />
    )

    fireEvent.click(screen.getByText('Alpha'))
    expect(onRowClick).toHaveBeenCalledWith(sampleData[0])
  })
})
