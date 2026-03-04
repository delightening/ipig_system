import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Card, CardContent } from '@/components/ui/card'
import { Search, LayoutGrid } from 'lucide-react'

interface AnimalFiltersProps {
  statusFilter: string
  onStatusFilterChange: (value: string) => void
  breedFilter: string
  onBreedFilterChange: (value: string) => void
  search: string
  onSearchChange: (value: string) => void
  onSearchSubmit?: () => void
  allowedStatuses: string[]
  adminOnlyStatuses: string[]
  isPIOrClient: boolean
  isAdmin: boolean
  statusCounts: Record<string, number>
  allAnimalsCount: number
  selectedAnimalsCount: number
  onShowBatchAssign: () => void
}

export function AnimalFilters({
  statusFilter,
  onStatusFilterChange,
  breedFilter,
  onBreedFilterChange,
  search,
  onSearchChange,
  onSearchSubmit,
  allowedStatuses: _allowedStatuses, // intentionally unused
  adminOnlyStatuses,
  isPIOrClient,
  isAdmin,
  statusCounts,
  allAnimalsCount,
  selectedAnimalsCount,
  onShowBatchAssign,
}: AnimalFiltersProps) {
  const { t } = useTranslation()

  const tabs = [
    { value: 'pen', label: t('animals.statusLabels.pen'), count: allAnimalsCount, icon: <LayoutGrid className="h-4 w-4" /> },
    { value: 'unassigned', label: t('animals.statusLabels.unassigned'), count: statusCounts['unassigned'] || 0 },
    { value: 'in_experiment', label: t('animals.statusLabels.in_experiment'), count: statusCounts['in_experiment'] || 0 },
    { value: 'completed', label: t('animals.statusLabels.completed'), count: statusCounts['completed'] || 0 },
    { value: 'euthanized', label: t('animals.statusLabels.euthanized'), count: statusCounts['euthanized'] || 0 },
    { value: 'sudden_death', label: t('animals.statusLabels.sudden_death'), count: statusCounts['sudden_death'] || 0 },
    { value: 'transferred', label: t('animals.statusLabels.transferred'), count: statusCounts['transferred'] || 0 },
    { value: 'all', label: t('animals.statusLabels.all'), count: allAnimalsCount },
  ]

  const visibleTabs = tabs.filter(tab => {
    if (adminOnlyStatuses.includes(tab.value) && !isAdmin) return false
    if (isPIOrClient) return ['in_experiment', 'completed'].includes(tab.value)
    return true
  })

  return (
    <>
      {/* Status Tabs */}
      <div className="flex flex-wrap gap-2 border-b">
        {visibleTabs.map(tab => (
          <button
            key={tab.value}
            onClick={() => onStatusFilterChange(tab.value)}
            className={`px-3 md:px-4 py-2 text-xs md:text-sm font-medium border-b-2 transition-colors flex items-center gap-1.5 ${
              statusFilter === tab.value
                ? 'border-purple-600 text-purple-600'
                : 'border-transparent text-slate-500 hover:text-slate-700'
            }`}
          >
            {'icon' in tab && tab.icon}
            {tab.label} ({tab.count})
          </button>
        ))}
      </div>

      {/* Search & Breed Filter */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-3 md:gap-4">
            <div className="flex flex-col md:flex-row md:items-center gap-3 md:gap-4 flex-1">
              <div className="relative flex-1 max-w-sm flex gap-2">
                <div className="relative flex-1">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                  <Input
                    placeholder={t('animals.searchPlaceholder')}
                    aria-label={t('animals.searchPlaceholder')}
                    value={search}
                    onChange={(e) => onSearchChange(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        e.preventDefault()
                        onSearchSubmit?.()
                      }
                    }}
                    className="pl-9"
                  />
                </div>
                <Button type="button" variant="secondary" onClick={onSearchSubmit} aria-label={t('common.search')}>
                  <Search className="h-4 w-4 md:mr-1.5" />
                  <span className="hidden md:inline">{t('common.search')}</span>
                </Button>
              </div>
              <Select value={breedFilter} onValueChange={onBreedFilterChange}>
                <SelectTrigger className="w-[150px]">
                  <SelectValue placeholder="品種" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('animals.allBreeds')}</SelectItem>
                  <SelectItem value="minipig">{t('animals.breedLabels.minipig')}</SelectItem>
                  <SelectItem value="white">{t('animals.breedLabels.white')}</SelectItem>
                  <SelectItem value="other">{t('animals.breedLabels.other')}</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {selectedAnimalsCount > 0 && (
              <div className="flex items-center gap-2">
                <span className="text-sm text-slate-500">
                  {t('animals.selectedCount', { count: selectedAnimalsCount })}
                </span>
                {statusFilter === 'unassigned' && (
                  <Button variant="outline" size="sm" onClick={onShowBatchAssign}>
                    分配至計畫
                  </Button>
                )}
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </>
  )
}
