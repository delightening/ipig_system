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
import { Search } from 'lucide-react'
import { BuildingTabs } from './BuildingTabs'
import { StatusTabs } from './StatusTabs'

interface AnimalFilterState {
  statusFilter: string
  onStatusFilterChange: (value: string) => void
  /** 欄位檢視選中的棟舍 code（null → BuildingTabs fallback 到第一棟） */
  activeBuildingCode: string | null
  onBuildingChange: (buildingCode: string) => void
  breedFilter: string
  onBreedFilterChange: (value: string) => void
  search: string
  onSearchChange: (value: string) => void
  onSearchSubmit?: () => void
}

interface AnimalFilterCounts {
  statusCounts: Record<string, number>
  allAnimalsCount: number
  /** 各棟在欄動物數（key = building.code），各棟加總 == 舊的 pen_animals_count */
  penCountsByBuilding: Record<string, number>
  selectedAnimalsCount: number
}

interface AnimalFiltersProps {
  filters: AnimalFilterState
  counts: AnimalFilterCounts
  adminOnlyStatuses: string[]
  isPIOrClient: boolean
  isAdmin: boolean
  onShowBatchAssign: () => void
}

export function AnimalFilters({
  filters,
  counts,
  adminOnlyStatuses,
  isPIOrClient,
  isAdmin,
  onShowBatchAssign,
}: AnimalFiltersProps) {
  const { statusFilter, onStatusFilterChange, activeBuildingCode, onBuildingChange, breedFilter, onBreedFilterChange, search, onSearchChange, onSearchSubmit } = filters
  const { statusCounts, allAnimalsCount, penCountsByBuilding, selectedAnimalsCount } = counts
  const { t } = useTranslation()

  const tabs = [
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
      {/* 兩列分群：上＝欄位檢視（依棟舍），下＝動物狀態。兩列互斥，任一時刻只有一個 tag 選中 */}
      <div className="space-y-4">
        {/* PI / CLIENT 的 allowedStatuses 不含 'pen'，欄位檢視對他們不開放 */}
        {!isPIOrClient && (
          <div>
            <p className="text-xs font-medium text-muted-foreground mb-1.5">
              {t('animals.tabGroups.penView')}
            </p>
            <BuildingTabs
              activeBuildingCode={activeBuildingCode}
              onBuildingChange={onBuildingChange}
              penCountsByBuilding={penCountsByBuilding}
              isPenView={statusFilter === 'pen'}
            />
          </div>
        )}

        <div>
          <p className="text-xs font-medium text-muted-foreground mb-1.5">
            {t('animals.tabGroups.status')}
          </p>
          <StatusTabs
            tabs={visibleTabs}
            statusFilter={statusFilter}
            onStatusFilterChange={onStatusFilterChange}
          />
        </div>
      </div>

      {/* Search & Breed Filter */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-3 md:gap-4">
            <div className="flex flex-col md:flex-row md:items-center gap-3 md:gap-4 flex-1">
              <div className="relative flex-1 max-w-sm">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
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
                <span className="text-sm text-muted-foreground">
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
