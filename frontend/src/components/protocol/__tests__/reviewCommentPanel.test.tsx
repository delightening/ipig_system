import { render, screen, fireEvent } from '@testing-library/react'
import { ReviewCommentPanel } from '../ReviewCommentPanel'

// i18n：元件只用 t() 取送出鈕文字，測試不需要真實翻譯，回 key 即可。
// ⚠️ 必須一併導出 initReactI18next——測試 setup 檔會 import 它，
// 少了會整支測試檔載入失敗（訊息是 No "initReactI18next" export is defined）。
vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (k: string) => k }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

function setup(overrides: Partial<Parameters<typeof ReviewCommentPanel>[0]> = {}) {
  const onSubmit = vi.fn()
  const onClose = vi.fn()
  render(
    <ReviewCommentPanel
      onClose={onClose}
      onSubmit={onSubmit}
      isSubmitting={false}
      sectionOptions={['4.1 實驗設計', '5.2 麻醉']}
      {...overrides}
    />
  )
  return { onSubmit, onClose }
}

const noObjectionCheckbox = () => screen.getByRole('checkbox')
const submitButton = () => screen.getByRole('button', { name: /送出|submit/i })

describe('ReviewCommentPanel — 一般意見', () => {
  it('內容為空時送出鈕停用（避免送出空意見）', () => {
    setup()
    expect(submitButton()).toBeDisabled()
  })

  it('送出一般意見時 commentType 為 COMMENT', () => {
    const { onSubmit } = setup()
    fireEvent.change(screen.getByPlaceholderText('請輸入審查意見...'), {
      target: { value: '請補充第 4.1 節的麻醉劑量' },
    })
    fireEvent.click(submitButton())
    expect(onSubmit).toHaveBeenCalledWith(
      '請補充第 4.1 節的麻醉劑量',
      'COMMENT'
    )
  })

  it('選了章節時內容會帶章節前綴', () => {
    const { onSubmit } = setup()
    fireEvent.change(screen.getByRole('combobox'), {
      target: { value: '4.1 實驗設計' },
    })
    fireEvent.change(screen.getByPlaceholderText('請輸入審查意見...'), {
      target: { value: '請補充麻醉劑量' },
    })
    fireEvent.click(submitButton())
    expect(onSubmit).toHaveBeenCalledWith(
      '[4.1 實驗設計] 請補充麻醉劑量',
      'COMMENT'
    )
  })
})

describe('ReviewCommentPanel — 無意見', () => {
  it('勾選「無意見」後，即使內容為空也能送出', () => {
    setup()
    expect(submitButton()).toBeDisabled()
    fireEvent.click(noObjectionCheckbox())
    expect(submitButton()).toBeEnabled()
  })

  it('送出「無意見」時 commentType 為 NO_OBJECTION、內容為固定字串', () => {
    const { onSubmit } = setup()
    fireEvent.click(noObjectionCheckbox())
    fireEvent.click(submitButton())
    expect(onSubmit).toHaveBeenCalledWith('無意見', 'NO_OBJECTION')
  })

  it('勾選後章節選擇與內容欄都停用', () => {
    setup()
    fireEvent.click(noObjectionCheckbox())
    expect(screen.getByRole('combobox')).toBeDisabled()
    expect(screen.getByPlaceholderText('請輸入審查意見...')).toBeDisabled()
  })

  it('「無意見」不掛章節前綴——它是對整份計畫書表態，不是針對某一節', () => {
    const { onSubmit } = setup()
    // 先選章節、再勾無意見：章節不該混進去
    fireEvent.change(screen.getByRole('combobox'), {
      target: { value: '5.2 麻醉' },
    })
    fireEvent.click(noObjectionCheckbox())
    fireEvent.click(submitButton())
    expect(onSubmit).toHaveBeenCalledWith('無意見', 'NO_OBJECTION')
    // 確認完全沒有前綴形式的呼叫
    expect(onSubmit).not.toHaveBeenCalledWith(
      expect.stringContaining('['),
      expect.anything()
    )
  })

  it('取消勾選後回到一般意見模式，欄位重新啟用且需要內容', () => {
    setup()
    fireEvent.click(noObjectionCheckbox())
    fireEvent.click(noObjectionCheckbox())
    expect(screen.getByPlaceholderText('請輸入審查意見...')).toBeEnabled()
    expect(submitButton()).toBeDisabled()
  })

  it('送出中時按鈕停用（避免重複送出）', () => {
    setup({ isSubmitting: true })
    expect(submitButton()).toBeDisabled()
  })

  it('送出中時「無意見」勾選框也停用——否則送出途中改主意會讓畫面與已送出的內容不一致', () => {
    setup({ isSubmitting: true })
    expect(noObjectionCheckbox()).toBeDisabled()
  })
})
