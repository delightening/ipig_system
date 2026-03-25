import type { AnimalStatus } from '@/lib/api'

export type TabType =
  | 'timeline'
  | 'observations'
  | 'surgeries'
  | 'weights'
  | 'vaccinations'
  | 'sacrifice'
  | 'info'
  | 'pathology'
  | 'blood_tests'
  | 'pain_assessment'
  | 'transfer'

export const VALID_TABS: TabType[] = [
  'timeline',
  'observations',
  'surgeries',
  'weights',
  'vaccinations',
  'sacrifice',
  'info',
  'pathology',
  'blood_tests',
  'pain_assessment',
  'transfer',
]

export function parseTabFromUrl(
  tabParam: string | null,
  animalStatus?: AnimalStatus,
): TabType {
  if (!tabParam || !VALID_TABS.includes(tabParam as TabType)) return 'timeline'
  if (
    tabParam === 'transfer' &&
    animalStatus !== 'completed' &&
    animalStatus !== 'transferred'
  ) {
    return 'timeline'
  }
  return tabParam as TabType
}

/** Badge status colors used on the detail page header (solid background + white text) */
export const detailStatusColors: Record<AnimalStatus, string> = {
  unassigned: 'bg-status-neutral-solid',
  in_experiment: 'bg-status-warning-solid',
  completed: 'bg-status-success-solid',
  euthanized: 'bg-status-error-solid',
  sudden_death: 'bg-status-error-strong-solid',
  transferred: 'bg-status-purple-solid',
}

/** List page status colors (light background + dark text) */
export const statusColors: Record<AnimalStatus, string> = {
  unassigned: 'bg-status-neutral-bg text-status-neutral-text',
  in_experiment: 'bg-status-warning-bg text-status-warning-text',
  completed: 'bg-status-success-bg text-status-success-text',
  euthanized: 'bg-status-error-bg text-status-error-text',
  sudden_death: 'bg-status-error-bg text-status-error-text',
  transferred: 'bg-status-purple-bg text-status-purple-text',
}

export const getPenLocationDisplay = (
  animal: { status: AnimalStatus; pen_location?: string | null },
  t: (key: string) => string,
) => {
  if (animal.status === 'completed' && !animal.pen_location) {
    return t('animals.sacrificed')
  }
  return animal.pen_location || '-'
}

// 舊的硬編碼欄位常數已移除，改由 useFacilityLayout hook 從 API 動態取得
