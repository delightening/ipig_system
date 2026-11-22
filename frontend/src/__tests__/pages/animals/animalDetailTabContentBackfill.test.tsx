/**
 * 動物詳細頁 TabContent：補登 IACUC 事件日期對話框的「選取對象」要跟著動物走。
 *
 * 背景：路由從動物 A 換到動物 B 時，TabContent 不會卸載（B 已在快取就沒有載入畫面），
 * 若只記「哪個事件」，對話框會拿 B 的 animalId 配 A 的事件送出（CodeRabbit 對 #255 的建議）。
 *
 * 要釘住的契約：
 * 1. 開啟補登後，對話框拿到的是「目前這隻動物」與「被點的事件」。
 * 2. 換動物後，對話框消失，且任何一次 render 都不會出現「新動物 + 舊事件」的組合。
 * 3. 換回原本的動物，舊的選取不會自己跑回來。
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

// 有補登權限；useAuthHasPermission 回傳的是「判斷函式」
vi.mock('@/stores/auth', () => ({
  useAuthHasPermission: () => () => true,
}))

const dialogRenders: Array<{ animalId: string; eventId: string }> = []

vi.mock('@/components/animal/AnimalTimelineView', () => ({
  AnimalTimelineView: ({
    iacucEvents,
    onBackfillIacucDate,
  }: {
    iacucEvents: Array<{ id: string }>
    onBackfillIacucDate?: (event: { id: string }) => void
  }) => (
    <button type="button" onClick={() => onBackfillIacucDate?.(iacucEvents[0])}>
      pick-event
    </button>
  ),
}))

vi.mock('@/components/animal/BackfillIacucEventDateDialog', () => ({
  BackfillIacucEventDateDialog: ({
    animalId,
    event,
    onOpenChange,
  }: {
    animalId: string
    event: { id: string }
    onOpenChange: (open: boolean) => void
  }) => {
    dialogRenders.push({ animalId, eventId: event.id })
    return (
      <div data-testid="backfill-dialog">
        {animalId}/{event.id}
        <button type="button" onClick={() => onOpenChange(false)}>
          close
        </button>
      </div>
    )
  },
}))

import { TabContent } from '@/pages/animals/components/AnimalDetailTabContent'
import type { Animal, AnimalEvent } from '@/lib/api'

const eventOf = (id: string): AnimalEvent => ({
  id,
  event_type: 'IACUC_CHANGE',
  before_data: { iacuc_no: null },
  after_data: { iacuc_no: 'PIG-114005' },
  created_at: '2026-11-12T03:00:00Z',
})

function ui(animalId: string, eventId: string) {
  return (
    <TabContent
      activeTab="timeline"
      animal={{ id: animalId, ear_tag: 'T-001', status: 'in_experiment' } as unknown as Animal}
      animalId={animalId}
      data={{
        afterParam: '',
        observations: [],
        surgeries: [],
        weights: [],
        vaccinations: [],
        sacrifice: undefined,
        suddenDeath: undefined,
        transfers: [],
        iacucEvents: [eventOf(eventId)],
      }}
      hasAdminRole
      developerMode={false}
      toggleDeveloperMode={() => {}}
      setActiveTab={() => {}}
    />
  )
}

describe('TabContent 補登對話框的選取對象', () => {
  beforeEach(() => {
    dialogRenders.length = 0
  })

  it('開啟後對話框拿到目前動物與被點的事件', async () => {
    render(ui('animal-1', 'evt-1'))
    fireEvent.click(await screen.findByText('pick-event'))
    expect(await screen.findByTestId('backfill-dialog')).toHaveTextContent('animal-1/evt-1')
  })

  it('換動物後對話框消失，且不曾出現「新動物 + 舊事件」', async () => {
    const { rerender } = render(ui('animal-1', 'evt-1'))
    fireEvent.click(await screen.findByText('pick-event'))
    await screen.findByTestId('backfill-dialog')

    rerender(ui('animal-2', 'evt-2'))

    expect(screen.queryByTestId('backfill-dialog')).toBeNull()
    expect(dialogRenders.length).toBeGreaterThan(0)
    // 每一次 render 的 (動物, 事件) 都必須是一起選的那一對
    expect(dialogRenders.every((r) => r.animalId === 'animal-1' && r.eventId === 'evt-1')).toBe(true)
  })

  it('換回原本的動物，舊的選取不會自己跑回來', async () => {
    const { rerender } = render(ui('animal-1', 'evt-1'))
    fireEvent.click(await screen.findByText('pick-event'))
    await screen.findByTestId('backfill-dialog')

    rerender(ui('animal-2', 'evt-2'))
    rerender(ui('animal-1', 'evt-1'))

    expect(screen.queryByTestId('backfill-dialog')).toBeNull()
  })

  it('關閉對話框會清掉選取', async () => {
    render(ui('animal-1', 'evt-1'))
    fireEvent.click(await screen.findByText('pick-event'))
    fireEvent.click(await screen.findByText('close'))
    expect(screen.queryByTestId('backfill-dialog')).toBeNull()
  })
})
