import type { AnimalStatus } from '@/lib/api'

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

const buildPenNumbers = (count: number) =>
  Array.from({ length: count }, (_, index) => String(index + 1).padStart(2, '0'))

export const penBuildings = [
  { value: 'A', label: 'A 棟 (ACD)' },
  { value: 'B', label: 'B 棟 (BEFG)' },
]

export const penZonesByBuilding: Record<string, string[]> = {
  A: ['A', 'C', 'D'],
  B: ['B', 'E', 'F', 'G'],
}

export const penNumbersByZone: Record<string, string[]> = {
  A: buildPenNumbers(20),
  B: buildPenNumbers(20),
  C: buildPenNumbers(20),
  D: buildPenNumbers(33),
  E: buildPenNumbers(25),
  F: buildPenNumbers(6),
  G: buildPenNumbers(6),
}

export const penZoneColors: Record<string, { bg: string; border: string; header: string; text: string }> = {
  A: { bg: 'bg-blue-50', border: 'border-blue-300', header: 'bg-blue-500', text: 'text-blue-700' },
  B: { bg: 'bg-orange-50', border: 'border-orange-300', header: 'bg-orange-500', text: 'text-orange-700' },
  C: { bg: 'bg-yellow-50', border: 'border-yellow-300', header: 'bg-yellow-500', text: 'text-yellow-700' },
  D: { bg: 'bg-cyan-50', border: 'border-cyan-300', header: 'bg-cyan-500', text: 'text-cyan-700' },
  E: { bg: 'bg-purple-50', border: 'border-purple-300', header: 'bg-purple-500', text: 'text-purple-700' },
  F: { bg: 'bg-amber-50', border: 'border-amber-300', header: 'bg-amber-500', text: 'text-amber-700' },
  G: { bg: 'bg-green-50', border: 'border-green-300', header: 'bg-green-500', text: 'text-green-700' },
}
