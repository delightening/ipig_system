import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'

/**
 * 撤銷對話框：原因必填，空白或過短不可送出；送出的是 trim 後的原因。
 *
 * i18n：t() 回 key 本身，斷言比對 key 不比對文案。
 * ⚠️ 必須一併導出 initReactI18next——`lib/utils` 會拉進 `lib/i18n`。
 */
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

const { RevokeDocumentDialog } = await import('../components/RevokeDocumentDialog')
const { ReversalStateBadge } = await import('../components/ReversalStateBadge')
const { MIN_REVOKE_REASON_CHARS } = await import('../documentActionState')

const CONFIRM_KEY = 'erpDocs.documents.detail.revokeDialog.confirm'
const REASON_LABEL_KEY = 'erpDocs.documents.detail.revokeDialog.reasonLabel'

function renderDialog(overrides: Partial<React.ComponentProps<typeof RevokeDocumentDialog>> = {}) {
  const onConfirm = vi.fn()
  const onOpenChange = vi.fn()
  const utils = render(
    <RevokeDocumentDialog
      open
      onOpenChange={onOpenChange}
      docNo="PO-TEST-01"
      isPending={false}
      onConfirm={onConfirm}
      {...overrides}
    />,
  )
  return { onConfirm, onOpenChange, ...utils }
}

describe('RevokeDocumentDialog', () => {
  it('原因為空時不可送出', () => {
    renderDialog()
    expect(screen.getByRole('button', { name: CONFIRM_KEY })).toBeDisabled()
  })

  it('只有空白、或不足下限時不可送出', () => {
    renderDialog()
    const box = screen.getByLabelText(REASON_LABEL_KEY)

    fireEvent.change(box, { target: { value: '      ' } })
    expect(screen.getByRole('button', { name: CONFIRM_KEY })).toBeDisabled()

    fireEvent.change(box, { target: { value: '太短' } })
    expect(screen.getByRole('button', { name: CONFIRM_KEY })).toBeDisabled()
  })

  it('原因達下限後可送出，且送出 trim 後的原因', () => {
    const { onConfirm } = renderDialog()
    const reason = '一'.repeat(MIN_REVOKE_REASON_CHARS)
    fireEvent.change(screen.getByLabelText(REASON_LABEL_KEY), {
      target: { value: `  ${reason}  ` },
    })

    const confirm = screen.getByRole('button', { name: CONFIRM_KEY })
    expect(confirm).toBeEnabled()
    fireEvent.click(confirm)
    expect(onConfirm).toHaveBeenCalledWith(reason)
  })

  it('送出中不可重複送出', () => {
    renderDialog({ isPending: true })
    fireEvent.change(screen.getByLabelText(REASON_LABEL_KEY), {
      target: { value: '一'.repeat(MIN_REVOKE_REASON_CHARS) },
    })
    expect(screen.getByRole('button', { name: CONFIRM_KEY })).toBeDisabled()
  })

  it('關閉後重開，不會帶著上一次的原因', () => {
    const { rerender, onConfirm, onOpenChange } = renderDialog()
    fireEvent.change(screen.getByLabelText(REASON_LABEL_KEY), {
      target: { value: '一'.repeat(MIN_REVOKE_REASON_CHARS) },
    })

    const props = { onOpenChange, docNo: 'PO-TEST-01', isPending: false, onConfirm }
    rerender(<RevokeDocumentDialog open={false} {...props} />)
    rerender(<RevokeDocumentDialog open {...props} />)

    expect(screen.getByLabelText(REASON_LABEL_KEY)).toHaveValue('')
    expect(screen.getByRole('button', { name: CONFIRM_KEY })).toBeDisabled()
  })
})

describe('ReversalStateBadge', () => {
  it('已沖銷與沖銷中各顯示對應文案，無狀態時不渲染', () => {
    const { rerender, container } = render(<ReversalStateBadge state="reversed" />)
    expect(screen.getByText('erpDocs.documents.reversal.badgeReversed')).toBeInTheDocument()

    rerender(<ReversalStateBadge state="reversing" />)
    expect(screen.getByText('erpDocs.documents.reversal.badgeReversing')).toBeInTheDocument()

    rerender(<ReversalStateBadge state={null} />)
    expect(container).toBeEmptyDOMElement()
  })
})
