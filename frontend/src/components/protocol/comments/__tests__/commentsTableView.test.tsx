import { render, screen } from '@testing-library/react'
import { CommentsTableView } from '../CommentsTableView'
import type { ReviewCommentResponse } from '@/types/aup'
import type { ReviewerGroup } from '../useCommentsData'

// i18n：回 key 即可，測試斷言的是「用了哪個 key」而非翻譯文字。
// ⚠️ 必須一併導出 initReactI18next——測試 setup 檔會 import 它。
vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (k: string) => k }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

// useTableSort 在測試環境不需要真的排序，直接把資料原樣送回。
vi.mock('@/hooks/useTableSort', () => ({
  useTableSort: (data: unknown[]) => ({
    sortedData: data,
    sort: { column: null, direction: null },
    toggleSort: vi.fn(),
  }),
}))

const NO_REPLY_YET = 'protocols.detail.tables.noReplyYet'
const NO_REPLY_NEEDED = 'protocols.detail.tables.noReplyNeeded'
const NO_OBJECTION_BADGE = 'protocols.detail.tables.noObjection'

function comment(over: Partial<ReviewCommentResponse> = {}): ReviewCommentResponse {
  return {
    id: 'c1',
    protocol_id: 'p1',
    reviewer_id: 'r1',
    reviewer_name: '陳怡均',
    content: '請補充第 4.1 節的麻醉劑量',
    comment_type: 'COMMENT',
    is_resolved: false,
    created_at: '2026-08-25T00:00:00Z',
    updated_at: '2026-08-25T00:00:00Z',
    ...over,
  } as ReviewCommentResponse
}

function groupOf(...comments: ReviewCommentResponse[]): ReviewerGroup {
  return {
    reviewerId: 'r1',
    displayName: '陳怡均',
    questions: comments.map((c) => ({ comment: c, replies: [] })),
  }
}

function setup(c: ReviewCommentResponse, canReply = true) {
  render(
    <CommentsTableView
      preReviewGroups={[groupOf(c)]}
      underReviewGroups={[]}
      canReply={canReply}
      onReply={vi.fn()}
    />
  )
}

/**
 * 表格版與卡片版是兩份各自獨立的 JSX（@container 斷點切換），兩邊都會被 render，
 * 所以同一個字串會出現兩次。要斷言「完全沒有」時用 queryAll().length === 0；
 * 要斷言「有」時用 getAll().length > 0——不能用 getByText，它會因為找到兩個而拋錯。
 */
const countOf = (text: string) => screen.queryAllByText(text).length

/**
 * ⚠️ 不能用 queryAllByRole('button') 抓——排序表頭（SortableTableHead）本身也是
 * button，一律會有 2 個，於是「沒有回覆按鈕」的斷言永遠失敗。要按名稱定位。
 * 這也是為什麼兩處回覆按鈕都補上了 aria-label：原本只有 icon，沒有可存取名稱，
 * 測試抓不到，螢幕閱讀器同樣讀不出來。
 */
const replyButtons = () =>
  screen.queryAllByRole('button', { name: 'protocols.detail.actions.reply' })

describe('CommentsTableView — 一般意見（COMMENT）', () => {
  it('顯示回覆按鈕', () => {
    setup(comment())
    expect(replyButtons().length).toBeGreaterThan(0)
  })

  it('沒有回覆時顯示「尚未回覆」，不是「不需回覆」', () => {
    setup(comment())
    expect(countOf(NO_REPLY_YET)).toBeGreaterThan(0)
    expect(countOf(NO_REPLY_NEEDED)).toBe(0)
  })

  it('不顯示「無意見」標記', () => {
    setup(comment())
    expect(countOf(NO_OBJECTION_BADGE)).toBe(0)
  })
})

describe('CommentsTableView — 無意見（NO_OBJECTION）', () => {
  const noObjection = () =>
    comment({ content: '無意見', comment_type: 'NO_OBJECTION' })

  it('🔴 不顯示回覆按鈕——這是本 PR 的核心需求：申請人不需要回覆', () => {
    setup(noObjection())
    expect(replyButtons()).toHaveLength(0)
  })

  it('顯示「不需回覆」而非「尚未回覆」', () => {
    setup(noObjection())
    expect(countOf(NO_REPLY_NEEDED)).toBeGreaterThan(0)
    expect(countOf(NO_REPLY_YET)).toBe(0)
  })

  it('顯示「無意見」標記', () => {
    setup(noObjection())
    expect(countOf(NO_OBJECTION_BADGE)).toBeGreaterThan(0)
  })
})

describe('CommentsTableView — 判別依據是 comment_type，不是內容文字', () => {
  it('🔴 內容剛好是「無意見」但型別為 COMMENT 時，仍當一般意見處理', () => {
    // migration 005 之後才手打「無意見」四個字的，是一則真的一般意見。
    // 若哪天有人把判斷改回比對 content，這支測試會紅。
    setup(comment({ content: '無意見', comment_type: 'COMMENT' }))
    expect(replyButtons().length).toBeGreaterThan(0)
    expect(countOf(NO_REPLY_YET)).toBeGreaterThan(0)
    expect(countOf(NO_REPLY_NEEDED)).toBe(0)
    expect(countOf(NO_OBJECTION_BADGE)).toBe(0)
  })

  it('comment_type 缺漏時當一般意見（保守側，寧可多要一次回覆）', () => {
    const c = comment()
    delete (c as { comment_type?: string }).comment_type
    setup(c)
    expect(replyButtons().length).toBeGreaterThan(0)
    expect(countOf(NO_REPLY_NEEDED)).toBe(0)
  })
})

describe('CommentsTableView — 與既有 is_resolved 行為不互相干擾', () => {
  it('已結案的一般意見本來就沒有回覆按鈕（改動前的行為不變）', () => {
    setup(comment({ is_resolved: true }))
    expect(replyButtons()).toHaveLength(0)
  })

  it('canReply 為 false 時，一般意見也沒有回覆按鈕', () => {
    setup(comment(), false)
    expect(replyButtons()).toHaveLength(0)
  })
})

describe('CommentsTableView — 同一組內混合兩種型別', () => {
  it('只有一般意見那一則有回覆按鈕', () => {
    render(
      <CommentsTableView
        preReviewGroups={[
          groupOf(
            comment({ id: 'c1', content: '請補充麻醉劑量' }),
            comment({ id: 'c2', content: '無意見', comment_type: 'NO_OBJECTION' })
          ),
        ]}
        underReviewGroups={[]}
        canReply
        onReply={vi.fn()}
      />
    )
    // 表格版 + 卡片版各一個 → 共 2 個，而不是 4 個
    expect(replyButtons()).toHaveLength(2)
    expect(countOf(NO_REPLY_YET)).toBeGreaterThan(0)
    expect(countOf(NO_REPLY_NEEDED)).toBeGreaterThan(0)
  })
})
