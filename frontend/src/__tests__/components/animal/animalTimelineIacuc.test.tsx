/**
 * 動物時間軸的「IACUC No. 變更」事件：實際日期與登錄時間並列。
 *
 * 要釘住的契約：
 * 1. 事件有實際日期（首次分配才有）且與登錄日不同天 → 以實際日期為主，旁邊附「登錄於 …」。
 * 2. 實際日期與登錄日同一天 → 不重複顯示登錄時間。
 * 3. 沒有實際日期（轉換計畫、歷史事件）→ 只顯示登錄時間，不附註。
 * 4. 「舊 → 新」讀 before/after；舊值為 null 顯示「（無）」，不再兩邊都是「（無）」。
 */
import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'

// t() 回傳 key；帶 date 插值時把值接在後面，才斷言得到顯示的日期
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { date?: string }) => (opts?.date ? `${key}|${opts.date}` : key),
    i18n: { language: 'zh-TW' },
  }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

import { AnimalTimelineView } from '@/components/animal/AnimalTimelineView'
import { uiLocale } from '@/lib/utils'
import type { AnimalEvent } from '@/types/animal'

const noop = () => {}

function renderTimeline(iacucEvents: AnimalEvent[]) {
  return render(
    <AnimalTimelineView
      observations={[]}
      surgeries={[]}
      iacucEvents={iacucEvents}
      onView={noop}
      onEdit={noop}
      onCopy={noop}
      onHistory={noop}
      onVet={noop}
      onDelete={noop}
    />,
  )
}

/** 與元件相同的日期格式（台灣時區），避免測試綁死某個語系的字串。 */
const fmt = (iso: string) => new Date(iso).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })

const NONE = 'animalActions.timeline.none'

describe('AnimalTimelineView IACUC 變更事件', () => {
  it('有實際日期且與登錄日不同天：以實際日期為主，並附登錄時間', () => {
    renderTimeline([
      {
        id: 'e1',
        event_type: 'IACUC_CHANGE',
        actor_name: '系統管理員',
        before_data: { iacuc_no: null },
        after_data: { iacuc_no: 'PIG-114005', experiment_date: '2026-09-01' },
        created_at: '2026-11-12T03:00:00Z',
      },
    ])

    expect(screen.getByText(`${NONE} → PIG-114005`)).toBeTruthy()
    // 實際日期是顯示的日期標籤（同一天會在分組標頭與日期標籤各出現一次）
    expect(screen.getAllByText(fmt('2026-09-01')).length).toBeGreaterThan(0)
    expect(screen.getByText(`animalActions.timeline.recordedOn|${fmt('2026-11-12T03:00:00Z')}`)).toBeTruthy()
  })

  it('實際日期與登錄日同一天：不重複顯示登錄時間', () => {
    renderTimeline([
      {
        id: 'e2',
        event_type: 'IACUC_CHANGE',
        before_data: { iacuc_no: null },
        after_data: { iacuc_no: 'PIG-114005', experiment_date: '2026-11-12' },
        created_at: '2026-11-12T03:00:00Z',
      },
    ])

    expect(screen.getByText(`${NONE} → PIG-114005`)).toBeTruthy()
    expect(screen.queryByText(/animalActions\.timeline\.recordedOn/)).toBeNull()
  })

  it('沒有實際日期（轉換計畫／歷史事件）：只顯示登錄時間，不附註', () => {
    renderTimeline([
      {
        id: 'e3',
        event_type: 'IACUC_CHANGE',
        before_data: { iacuc_no: 'PIG-114004' },
        after_data: { iacuc_no: 'PIG-114005' },
        created_at: '2026-11-12T03:00:00Z',
      },
    ])

    expect(screen.getByText('PIG-114004 → PIG-114005')).toBeTruthy()
    expect(screen.getAllByText(fmt('2026-11-12T03:00:00Z')).length).toBeGreaterThan(0)
    expect(screen.queryByText(/animalActions\.timeline\.recordedOn/)).toBeNull()
  })

  it('before／after 都是空（修正前的壞資料）時才顯示兩邊都是「（無）」', () => {
    renderTimeline([
      {
        id: 'e4',
        event_type: 'IACUC_CHANGE',
        before_data: null,
        after_data: null,
        created_at: '2026-11-12T03:00:00Z',
      },
    ])

    expect(screen.getByText(`${NONE} → ${NONE}`)).toBeTruthy()
  })
})
