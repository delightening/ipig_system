import type { PaginatedResponse } from '@/types/common'

export interface DemoAnimalListItem {
  id: string
  ear_tag: string
  status: string
  breed: string
  gender: string
  entry_date: string
  pen_location?: string
  species_name?: string
  latest_weight?: number
  latest_weight_date?: string
  is_on_medication?: boolean
  has_abnormal_record?: boolean
  iacuc_no?: string
  created_at: string
  updated_at: string
}

const demoAnimals: DemoAnimalListItem[] = [
  {
    id: 'demo-a1', ear_tag: 'D-001', status: 'in_experiment', breed: 'minipig',
    gender: 'female', entry_date: '2025-11-15', pen_location: 'A-01',
    species_name: '迷你豬', latest_weight: 28.5, latest_weight_date: '2026-03-28',
    is_on_medication: false, has_abnormal_record: false, iacuc_no: 'IACUC-2025-001',
    created_at: '2025-11-15T08:00:00Z', updated_at: '2026-03-28T10:00:00Z',
  },
  {
    id: 'demo-a2', ear_tag: 'D-002', status: 'in_experiment', breed: 'minipig',
    gender: 'male', entry_date: '2025-11-15', pen_location: 'A-02',
    species_name: '迷你豬', latest_weight: 32.1, latest_weight_date: '2026-03-28',
    is_on_medication: true, has_abnormal_record: false, iacuc_no: 'IACUC-2025-001',
    created_at: '2025-11-15T08:00:00Z', updated_at: '2026-03-28T10:00:00Z',
  },
  {
    id: 'demo-a3', ear_tag: 'D-003', status: 'unassigned', breed: 'lyd',
    gender: 'female', entry_date: '2026-01-10', pen_location: 'B-03',
    species_name: 'LYD', latest_weight: 45.0, latest_weight_date: '2026-03-25',
    is_on_medication: false, has_abnormal_record: false,
    created_at: '2026-01-10T08:00:00Z', updated_at: '2026-03-25T10:00:00Z',
  },
  {
    id: 'demo-a4', ear_tag: 'D-004', status: 'completed', breed: 'white',
    gender: 'male', entry_date: '2025-09-01', pen_location: 'C-01',
    species_name: '白豬', latest_weight: 55.2, latest_weight_date: '2026-02-20',
    is_on_medication: false, has_abnormal_record: true, iacuc_no: 'IACUC-2024-012',
    created_at: '2025-09-01T08:00:00Z', updated_at: '2026-02-20T10:00:00Z',
  },
  {
    id: 'demo-a5', ear_tag: 'D-005', status: 'in_experiment', breed: 'minipig',
    gender: 'female', entry_date: '2026-02-01', pen_location: 'A-03',
    species_name: '迷你豬', latest_weight: 22.8, latest_weight_date: '2026-03-30',
    is_on_medication: false, has_abnormal_record: false, iacuc_no: 'IACUC-2025-003',
    created_at: '2026-02-01T08:00:00Z', updated_at: '2026-03-30T10:00:00Z',
  },
]

export const DEMO_ANIMALS_PAGINATED: PaginatedResponse<DemoAnimalListItem> = {
  data: demoAnimals,
  total: 5,
  page: 1,
  per_page: 20,
  total_pages: 1,
}

export const DEMO_ANIMALS_ALL = demoAnimals

export const DEMO_ANIMAL_STATS = {
  status_counts: {
    in_experiment: 3,
    unassigned: 1,
    completed: 1,
  } as Record<string, number>,
  pen_animals_count: 5,
  total: 5,
}

export const DEMO_ANIMALS_BY_PEN = [
  {
    pen_location: 'A-01',
    animals: [demoAnimals[0]],
  },
  {
    pen_location: 'A-02',
    animals: [demoAnimals[1]],
  },
  {
    pen_location: 'A-03',
    animals: [demoAnimals[4]],
  },
  {
    pen_location: 'B-03',
    animals: [demoAnimals[2]],
  },
  {
    pen_location: 'C-01',
    animals: [demoAnimals[3]],
  },
]
