import { memo } from 'react'
import { act, render, screen } from '@testing-library/react'
import i18n from '@/lib/i18n'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/components/ui/dialog'

/**
 * 對話框右上角「關閉」鈕的螢幕報讀文字。
 *
 * 釘住：對話框開著、外層完全不重繪時切換語言，文字仍要跟著換。
 * 直接在 render 裡呼叫 i18n.t() 不會訂閱語言切換，外層不重繪就會停在舊語言。
 */

// memo 且無 props：語言切換時不會因為外層而重繪，只剩 DialogContent 內部自己的訂閱
const StaticDialog = memo(function StaticDialog() {
  return (
    <Dialog open>
      <DialogContent>
        <DialogTitle>title</DialogTitle>
        <DialogDescription>description</DialogDescription>
      </DialogContent>
    </Dialog>
  )
})

let previousLanguage: string

beforeAll(() => {
  previousLanguage = i18n.language
})

afterAll(async () => {
  await i18n.changeLanguage(previousLanguage)
})

it('對話框開著時切換語言，關閉鈕的 sr-only 文字跟著更新', async () => {
  await i18n.changeLanguage('zh-TW')
  render(<StaticDialog />)
  expect(screen.getByText('關閉')).toBeInTheDocument()

  await act(async () => {
    await i18n.changeLanguage('en')
  })
  expect(screen.getByText('Close')).toBeInTheDocument()
  expect(screen.queryByText('關閉')).not.toBeInTheDocument()
})
