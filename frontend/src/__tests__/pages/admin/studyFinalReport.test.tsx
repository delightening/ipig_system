/**
 * P0-1：最終報告的三條授權路徑在 UI 上各自獨立，不能互相取代。
 *
 * 後端把授權拆成三塊（見 `docs/reviews/2026-09-03-code-side-issues.md` P0-1）：
 * 報告本文＝身分即授權（SD）、簽署＝身分即授權且無 admin 例外、QAU 品保聲明＝
 * 獨立權限碼且不得由該計畫 SD 填寫。前端對應的形狀是：
 *
 * - 本文編輯與簽署**刻意不做前端權限判斷**——SD 是「該計畫的」身分，列表頁無從得知，
 *   一律顯示、由後端回 403。所以測試要證明它們**確實會發出請求**，
 *   而不是被前端悄悄擋掉（前端擋掉的話，SD 本人也會點不到）。
 * - QAU 品保聲明**有**對應的權限碼，故前端依 `qau.report_statement.write`
 *   顯示可編輯區塊或唯讀文字。
 * - 已簽署（status = signed）的報告不得再出現編輯與簽署入口。
 *
 * harness 沿用鄰檔 `queryErrorStates.test.tsx` 的形狀（i18n mock + QueryClient +
 * MemoryRouter），差別只在這裡 mock 的是 glpCompliance 那組 API。
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
// 用 fireEvent 而非 @testing-library/user-event：後者不在專案依賴內
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter } from 'react-router-dom'

// t() 回傳 key 本身；同鄰檔，避免文案改寫影響斷言，也擋掉 lib/i18n 的 init 副作用
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { defaultValue?: string }) => opts?.defaultValue ?? key,
  }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

const {
  listStudyReports,
  getStudyReport,
  createStudyReport,
  updateStudyReport,
  signStudyReport,
  updateQauStatement,
  hasPermission,
  toast,
} = vi.hoisted(() => ({
  listStudyReports: vi.fn(),
  getStudyReport: vi.fn(),
  createStudyReport: vi.fn(),
  updateStudyReport: vi.fn(),
  signStudyReport: vi.fn(),
  updateQauStatement: vi.fn(),
  hasPermission: vi.fn(),
  toast: vi.fn(),
}))

vi.mock('@/lib/api/glpCompliance', () => ({
  listStudyReports,
  getStudyReport,
  createStudyReport,
  updateStudyReport,
  signStudyReport,
  updateQauStatement,
}))
vi.mock('@/stores/auth', () => ({ useAuthHasPermission: () => hasPermission }))
vi.mock('@/components/ui/use-toast', () => ({ toast }))

import { StudyFinalReportPage } from '@/pages/admin/StudyFinalReportPage'
// 型別匯入會在編譯期抹除，不受上面的 vi.mock 影響；標上它才不會讓
// `signed_at: null` 這種欄位把 fixture 的型別窄化成「只能是 null」
import type { StudyFinalReport } from '@/lib/api/glpCompliance'

const DRAFT_REPORT: StudyFinalReport = {
  id: '11111111-2222-3333-4444-555555555555',
  report_number: 'RPT-2026-0001',
  protocol_id: '99999999-8888-7777-6666-555555555555',
  title: '第一份最終報告',
  status: 'draft',
  summary: '摘要內容',
  methods: '方法內容',
  results: null,
  conclusions: null,
  deviations: null,
  signed_by: null,
  signed_at: null,
  qau_statement: null,
  qau_signed_by: null,
  qau_signed_at: null,
  created_at: '2026-09-01T00:00:00Z',
  updated_at: '2026-09-01T00:00:00Z',
}

function renderPage() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <StudyFinalReportPage />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

/** 開啟某份報告的詳情對話框，等它把內容載進來 */
async function openDetail(report: StudyFinalReport = DRAFT_REPORT) {
  getStudyReport.mockResolvedValue(report)
  fireEvent.click(await screen.findByText(report.title))
  // 標題是 `${report_number}｜${title}`，出現代表 getStudyReport 已回來
  return screen.findByText(`${report.report_number}｜${report.title}`)
}

beforeEach(() => {
  vi.clearAllMocks()
  listStudyReports.mockResolvedValue([DRAFT_REPORT])
  getStudyReport.mockResolvedValue(DRAFT_REPORT)
  updateStudyReport.mockResolvedValue(DRAFT_REPORT)
  signStudyReport.mockResolvedValue({ ...DRAFT_REPORT, status: 'signed' })
  updateQauStatement.mockResolvedValue(DRAFT_REPORT)
  hasPermission.mockReturnValue(false)
})

describe('最終報告列表', () => {
  it('列出後端回傳的報告', async () => {
    renderPage()
    expect(await screen.findByText('RPT-2026-0001')).toBeInTheDocument()
    expect(screen.getByText('第一份最終報告')).toBeInTheDocument()
  })

  it('「新增報告」不做前端權限判斷——建立權來自「是不是該計畫的 SD」，前端無從得知', async () => {
    hasPermission.mockReturnValue(false)
    renderPage()
    expect(await screen.findByText('新增報告')).toBeInTheDocument()
  })
})

describe('報告詳情對話框', () => {
  it('點列開啟詳情，顯示本文欄位', async () => {
    renderPage()
    await openDetail()
    expect(screen.getByText('摘要內容', { exact: false })).toBeInTheDocument()
    expect(getStudyReport).toHaveBeenCalledWith(DRAFT_REPORT.id)
  })

  it('編輯本文後儲存，送出 updateStudyReport 且 payload 不含 qau_statement', async () => {
    renderPage()
    await openDetail()

    fireEvent.click(screen.getByText('編輯報告本文'))
    fireEvent.click(screen.getByText('儲存'))

    await waitFor(() => expect(updateStudyReport).toHaveBeenCalledTimes(1))
    expect(updateStudyReport.mock.calls[0][0]).toBe(DRAFT_REPORT.id)
    // qau_statement 不得混在本文編輯的 payload 裡——那正是 P0-1 拆開的東西
    expect(updateStudyReport.mock.calls[0][1]).not.toHaveProperty('qau_statement')
  })

  it('簽署要帶密碼，且前端不擋——是不是該計畫 SD 由後端判定', async () => {
    renderPage()
    await openDetail()

    fireEvent.change(screen.getByPlaceholderText('密碼確認身分'), {
      target: { value: 'pw' },
    })
    fireEvent.click(screen.getByText('簽署'))

    await waitFor(() => expect(signStudyReport).toHaveBeenCalledTimes(1))
    expect(signStudyReport.mock.calls[0][1]).toEqual({ password: 'pw' })
  })

  it('已簽署的報告不再出現編輯與簽署入口', async () => {
    const signed = { ...DRAFT_REPORT, status: 'signed', signed_at: '2026-09-05T10:00:00Z' }
    listStudyReports.mockResolvedValue([signed])
    renderPage()
    await openDetail(signed)

    expect(screen.queryByText('編輯報告本文')).not.toBeInTheDocument()
    expect(screen.queryByText('簽署')).not.toBeInTheDocument()
  })
})

describe('QAU 品保聲明與本文分開授權', () => {
  it('沒有 qau.report_statement.write 時只讀不能寫', async () => {
    hasPermission.mockReturnValue(false)
    renderPage()
    await openDetail()

    expect(screen.getByText('尚無品保聲明')).toBeInTheDocument()
    expect(screen.queryByText('儲存品保聲明')).not.toBeInTheDocument()
  })

  it('有該權限時可填寫並送出 updateQauStatement，且不走本文那條路', async () => {
    hasPermission.mockImplementation((code: string) => code === 'qau.report_statement.write')
    renderPage()
    await openDetail()

    fireEvent.change(screen.getByPlaceholderText('品保稽核結論…'), {
      target: { value: '已完成品保稽核' },
    })
    fireEvent.click(screen.getByText('儲存品保聲明'))

    await waitFor(() => expect(updateQauStatement).toHaveBeenCalledTimes(1))
    expect(updateQauStatement).toHaveBeenCalledWith(DRAFT_REPORT.id, '已完成品保稽核')
    expect(updateStudyReport).not.toHaveBeenCalled()
  })

  /**
   * 迴歸：**已經有品保聲明的報告，打開後必須直接可以再存一次**。
   *
   * 舊寫法 textarea 用 `defaultValue`、按鈕看 `disabled={!qauStatement}`，
   * 而 `qauStatement` 初始是空字串——於是畫面上看得到既有內容、按鈕卻是灰的，
   * 非得先打一個字才能存（CodeRabbit 於 #102 指出）。
   *
   * 這支對舊寫法會紅：按鈕 disabled，click 不觸發 mutation，
   * `updateQauStatement` 收不到呼叫。
   */
  it('已有品保聲明時不必先打字就能重存，且送出的是畫面上那份內容', async () => {
    const withStatement = { ...DRAFT_REPORT, qau_statement: '既有的品保聲明' }
    listStudyReports.mockResolvedValue([withStatement])
    hasPermission.mockImplementation((code: string) => code === 'qau.report_statement.write')

    renderPage()
    await openDetail(withStatement)

    // 受控欄位：畫面上看得到既有內容。⚠️ 這一條不是鑑別點——舊寫法用
    // defaultValue 也看得到；鑑別點是下面那個 click。
    expect(screen.getByDisplayValue('既有的品保聲明')).toBeInTheDocument()

    fireEvent.click(screen.getByText('儲存品保聲明'))

    await waitFor(() => expect(updateQauStatement).toHaveBeenCalledTimes(1))
    expect(updateQauStatement).toHaveBeenCalledWith(withStatement.id, '既有的品保聲明')
  })
})
