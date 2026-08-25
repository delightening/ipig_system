import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { ReviewCommentPanel } from '../ReviewCommentPanel'

// i18n：回 key 即可，測試斷言的是「用了哪個 key」而非翻譯文字。
// ⚠️ 必須一併導出 initReactI18next——測試 setup 檔會 import 它，
// 少了會整支測試檔載入失敗（訊息是 No "initReactI18next" export is defined）。
vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (k: string) => k }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

const K = 'protocols.detail.dialogs.comment.panel'
const PLACEHOLDER = `${K}.placeholder`

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
const submitButton = () => screen.getByRole('button', { name: /submit/i })
const contentInput = () => screen.getByPlaceholderText(PLACEHOLDER)

describe('ReviewCommentPanel — 一般意見', () => {
  it('內容為空時送出鈕停用（避免送出空意見）', () => {
    setup()
    expect(submitButton()).toBeDisabled()
  })

  it('送出一般意見時 commentType 為 COMMENT', () => {
    const { onSubmit } = setup()
    fireEvent.change(contentInput(), {
      target: { value: '請補充第 4.1 節的麻醉劑量' },
    })
    fireEvent.click(submitButton())
    expect(onSubmit).toHaveBeenCalledWith('請補充第 4.1 節的麻醉劑量', 'COMMENT')
  })

  it('選了章節時內容會帶章節前綴', () => {
    const { onSubmit } = setup()
    fireEvent.change(screen.getByRole('combobox'), {
      target: { value: '4.1 實驗設計' },
    })
    fireEvent.change(contentInput(), { target: { value: '請補充麻醉劑量' } })
    fireEvent.click(submitButton())
    expect(onSubmit).toHaveBeenCalledWith('[4.1 實驗設計] 請補充麻醉劑量', 'COMMENT')
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
    expect(contentInput()).toBeDisabled()
  })

  it('「無意見」不掛章節前綴——它是對整份計畫書表態，不是針對某一節', () => {
    const { onSubmit } = setup()
    // 先選章節、再勾無意見：章節不該混進去
    fireEvent.change(screen.getByRole('combobox'), { target: { value: '5.2 麻醉' } })
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
    expect(contentInput()).toBeEnabled()
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

/**
 * 送出失敗時的行為。onSubmit 底下是 React Query 的 mutation，
 * 面板要等它 resolve 才能清空欄位——否則請求一失敗，使用者剛打完的意見就沒了。
 */
describe('ReviewCommentPanel — 送出失敗時保留使用者的輸入', () => {
  it('🔴 一般意見送出失敗，內容不被清空', async () => {
    const onSubmit = vi.fn().mockRejectedValue(new Error('網路斷線'))
    setup({ onSubmit })
    fireEvent.change(contentInput(), { target: { value: '請補充麻醉劑量' } })
    fireEvent.click(submitButton())
    await waitFor(() => expect(onSubmit).toHaveBeenCalled())
    expect(contentInput()).toHaveValue('請補充麻醉劑量')
  })

  it('🔴 「無意見」送出失敗，勾選狀態不被還原', async () => {
    const onSubmit = vi.fn().mockRejectedValue(new Error('網路斷線'))
    setup({ onSubmit })
    fireEvent.click(noObjectionCheckbox())
    fireEvent.click(submitButton())
    await waitFor(() => expect(onSubmit).toHaveBeenCalled())
    expect(noObjectionCheckbox()).toBeChecked()
  })

  it('送出成功則照常清空，並回到一般意見模式', async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined)
    setup({ onSubmit })
    fireEvent.click(noObjectionCheckbox())
    fireEvent.click(submitButton())
    await waitFor(() => expect(noObjectionCheckbox()).not.toBeChecked())
    expect(contentInput()).toHaveValue('')
  })

  it('onSubmit 回傳 void（非 Promise）時維持舊行為：送出即清空', async () => {
    const onSubmit = vi.fn().mockReturnValue(undefined)
    setup({ onSubmit })
    fireEvent.change(contentInput(), { target: { value: '請補充麻醉劑量' } })
    fireEvent.click(submitButton())
    await waitFor(() => expect(contentInput()).toHaveValue(''))
  })
})

/**
 * 無障礙：每個可見的 Label 都要真的關聯到它的控制項。
 * 沒有 htmlFor/id 這組關聯，螢幕閱讀器讀到的是一個沒有名稱的下拉選單／文字區。
 */
describe('ReviewCommentPanel — 表單控制項有可存取名稱', () => {
  it('章節下拉選單抓得到對應的 Label', () => {
    setup()
    expect(
      screen.getByLabelText(`${K}.targetSection`)
    ).toBe(screen.getByRole('combobox'))
  })

  it('意見內容文字區抓得到對應的 Label', () => {
    setup()
    expect(screen.getByLabelText(`${K}.contentLabel`)).toBe(contentInput())
  })

  /**
   * ⚠️ 一定要把兩種狀態都掃過。
   *
   * 第一版只掃未勾選的狀態，於是「送出鈕文案改回寫死中文」的 mutation 它抓不到——
   * 那句中文只在 noObjection 為 true 時才 render。當時是靠 submitButton() 用
   * /submit/i 定位失敗、連帶 5 支測試變紅才發現，而不是靠這支測試本身。
   * 綠燈不等於測到了。
   */
  it.each([
    ['未勾選無意見', false],
    ['已勾選無意見', true],
  ])('沒有任何寫死的中文字串留在元件裡（%s）', (_label, checkNoObjection) => {
    setup()
    if (checkNoObjection) fireEvent.click(noObjectionCheckbox())
    // t() 被 mock 成回傳 key，所以畫面上不該出現中文——
    // 唯一的例外是 sectionOptions，那是呼叫端傳進來的資料而非 UI 文案。
    const texts = Array.from(document.querySelectorAll('h3, label, option, button'))
      .map((el) => el.textContent ?? '')
      .filter((s) => /[一-鿿]/.test(s))
      .filter((s) => !/實驗設計|麻醉/.test(s))
    expect(texts).toEqual([])
  })
})
