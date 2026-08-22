import { render, screen, fireEvent } from '@testing-library/react'
import { vi } from 'vitest'

import { AutoGrowTextarea } from '../autoGrowTextarea'

// 註：jsdom 不計算 scrollHeight（恆為 0），自動長高的實際高度需瀏覽器驗證；
// 此處驗證可測行為：值傳遞、onChange、收合按鈕切換、disabled 時隱藏按鈕。

describe('AutoGrowTextarea', () => {
  it('將 value 傳給底層 textarea 並轉發 onChange', () => {
    const onChange = vi.fn()
    render(<AutoGrowTextarea value="hello" onChange={onChange} />)
    const ta = screen.getByDisplayValue('hello') as HTMLTextAreaElement
    expect(ta.tagName).toBe('TEXTAREA')
    fireEvent.change(ta, { target: { value: 'world' } })
    expect(onChange).toHaveBeenCalledTimes(1)
  })

  it('收合按鈕點擊後在「收合」與「展開」間切換', () => {
    render(<AutoGrowTextarea value="x" onChange={() => {}} />)
    const btn = screen.getByLabelText('收合欄位')
    fireEvent.click(btn)
    expect(screen.getByLabelText('展開欄位')).toBeInTheDocument()
    fireEvent.click(screen.getByLabelText('展開欄位'))
    expect(screen.getByLabelText('收合欄位')).toBeInTheDocument()
  })

  it('disabled 時不顯示收合按鈕', () => {
    render(<AutoGrowTextarea value={'N/A'} disabled />)
    expect(screen.queryByLabelText('收合欄位')).not.toBeInTheDocument()
    expect(screen.queryByLabelText('展開欄位')).not.toBeInTheDocument()
  })
})
