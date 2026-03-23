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

/** Badge status colors used on the detail page header */
export const detailStatusColors: Record<AnimalStatus, string> = {
  unassigned: 'bg-gray-500',
  in_experiment: 'bg-orange-500',
  completed: 'bg-green-500',
  euthanized: 'bg-red-500',
  sudden_death: 'bg-rose-600',
  transferred: 'bg-indigo-500',
}

/** List page status colors */
export const statusColors: Record<AnimalStatus, string> = {
  unassigned: 'bg-gray-100 text-gray-800',
  in_experiment: 'bg-orange-100 text-orange-800',
  completed: 'bg-green-100 text-green-800',
  euthanized: 'bg-red-100 text-red-800',
  sudden_death: 'bg-rose-100 text-rose-800',
  transferred: 'bg-indigo-100 text-indigo-800',
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
