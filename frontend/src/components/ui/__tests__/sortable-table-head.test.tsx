import { render, screen, fireEvent } from '@testing-library/react'
import { SortableTableHead } from '../sortable-table-head'

/** `<th>` 必須掛在合法的表格結構下，否則 React 會警告 DOM 巢狀不合法 */
function renderHead(props: Partial<React.ComponentProps<typeof SortableTableHead>> = {}) {
  const onSort = props.onSort ?? vi.fn()
  const utils = render(
    <table>
      <thead>
        <tr>
          <SortableTableHead
            sortKey="iacuc_no"
            currentSort={null}
            currentDirection="asc"
            {...props}
            onSort={onSort}
          >
            計畫編號
          </SortableTableHead>
        </tr>
      </thead>
    </table>
  )
  return { ...utils, onSort }
}

describe('SortableTableHead', () => {
  it('以原生 button 呈現，確保鍵盤可啟動（Enter/Space 為 button 原生行為）', () => {
    renderHead()
    const button = screen.getByRole('button', { name: '計畫編號' })
    expect(button).toBeInTheDocument()
    expect(button.tagName).toBe('BUTTON')
    // type="button" 避免在 <form> 內誤觸送出
    expect(button).toHaveAttribute('type', 'button')
  })

  it('button 可被鍵盤聚焦', () => {
    renderHead()
    const button = screen.getByRole('button', { name: '計畫編號' })
    button.focus()
    expect(document.activeElement).toBe(button)
    expect(button).not.toBeDisabled()
  })

  it('未排序時 aria-sort 為 none', () => {
    renderHead({ currentSort: null })
    expect(screen.getByRole('columnheader')).toHaveAttribute('aria-sort', 'none')
  })

  it('升冪時 aria-sort 為 ascending', () => {
    renderHead({ currentSort: 'iacuc_no', currentDirection: 'asc' })
    expect(screen.getByRole('columnheader')).toHaveAttribute('aria-sort', 'ascending')
  })

  it('降冪時 aria-sort 為 descending', () => {
    renderHead({ currentSort: 'iacuc_no', currentDirection: 'desc' })
    expect(screen.getByRole('columnheader')).toHaveAttribute('aria-sort', 'descending')
  })

  it('排序中的是其他欄位時，本欄 aria-sort 仍為 none', () => {
    renderHead({ currentSort: 'pi_name', currentDirection: 'desc' })
    expect(screen.getByRole('columnheader')).toHaveAttribute('aria-sort', 'none')
  })

  it('點擊會以 sortKey 呼叫 onSort', () => {
    const { onSort } = renderHead()
    fireEvent.click(screen.getByRole('button', { name: '計畫編號' }))
    expect(onSort).toHaveBeenCalledTimes(1)
    expect(onSort).toHaveBeenCalledWith('iacuc_no')
  })

  it('vertical 版面同樣提供 button 與 aria-sort', () => {
    renderHead({ vertical: true, currentSort: 'iacuc_no', currentDirection: 'asc' })
    expect(screen.getByRole('button', { name: '計畫編號' })).toBeInTheDocument()
    expect(screen.getByRole('columnheader')).toHaveAttribute('aria-sort', 'ascending')
  })
})
