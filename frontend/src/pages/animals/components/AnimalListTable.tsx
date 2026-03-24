import { useMemo } from 'react'
import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import type { AnimalListItem } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Card, CardContent } from '@/components/ui/card'
import { Eye, Edit2, AlertCircle, ArrowUpDown } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { EmptyState } from '@/components/ui/empty-state'
import { statusColors, getPenLocationDisplay } from '../constants'

interface AnimalListTableProps {
  animals: AnimalListItem[]
  isLoading: boolean
  selectedAnimals: string[]
  onToggleSelection: (id: string) => void
  onToggleAll: () => void
  onQuickEdit: (animalId: string) => void
  sortColumn: string | null
  sortDirection: 'asc' | 'desc'
  onSort: (column: string) => void
  page: number
  totalPages: number
  totalAnimals: number
  perPage: number
  onPageChange: (page: number) => void
}

export function AnimalListTable({
  animals,
  isLoading,
  selectedAnimals,
  onToggleSelection,
  onToggleAll,
  onQuickEdit,
  sortColumn,
  sortDirection,
  onSort,
  page,
  totalPages,
  totalAnimals,
  perPage,
  onPageChange,
}: AnimalListTableProps) {
  const { t } = useTranslation()

  const sortedAnimals = useMemo(() => {
    if (!sortColumn) return animals
    return [...animals].sort((a, b) => {
      let aVal: string | number = String((a as unknown as Record<string, unknown>)[sortColumn] ?? '')
      let bVal: string | number = String((b as unknown as Record<string, unknown>)[sortColumn] ?? '')
      if (sortColumn === 'entry_date') {
        aVal = aVal ? new Date(aVal as string).getTime() : 0
        bVal = bVal ? new Date(bVal as string).getTime() : 0
      } else if (sortColumn === 'latest_weight') {
        aVal = aVal !== null && aVal !== undefined ? Number(aVal) : (sortDirection === 'asc' ? Infinity : -Infinity)
        bVal = bVal !== null && bVal !== undefined ? Number(bVal) : (sortDirection === 'asc' ? Infinity : -Infinity)
      } else if (typeof aVal === 'string') aVal = aVal.toLowerCase()
      if (typeof bVal === 'string' && sortColumn !== 'latest_weight') bVal = bVal.toLowerCase()

      if (aVal < bVal) return sortDirection === 'asc' ? -1 : 1
      if (aVal > bVal) return sortDirection === 'asc' ? 1 : -1
      return 0
    })
  }, [animals, sortColumn, sortDirection])

  const SortableHeader = ({ column, label }: { column: string; label: string }) => (
    <TableHead
      className="cursor-pointer hover:bg-slate-100 select-none"
      onClick={() => onSort(column)}
    >
      <div className="flex items-center gap-1">
        {label}
        <ArrowUpDown className={`h-3 w-3 ${sortColumn === column ? 'text-purple-600' : 'text-slate-400'}`} />
      </div>
    </TableHead>
  )

  return (
    <Card>
      <CardContent className="p-0">
        {isLoading ? (
          <TableSkeleton rows={10} cols={8} />
        ) : animals.length === 0 ? (
          <EmptyState
            icon={AlertCircle}
            title={t('animals.noAnimalsFound')}
            description={t('animals.noAnimalsFoundDescription', '嘗試調整篩選條件，或新增第一筆動物紀錄')}
          />
        ) : (
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-12">
                    <input
                      type="checkbox"
                      aria-label="全選動物"
                      checked={selectedAnimals.length === animals.length && animals.length > 0}
                      onChange={onToggleAll}
                      className="rounded border-slate-300"
                    />
                  </TableHead>
                  <SortableHeader column="ear_tag" label={t('animals.earTag')} />
                  <SortableHeader column="pen_location" label={t('animals.pen')} />
                  <SortableHeader column="iacuc_no" label={t('animals.iacucNo')} />
                  <TableHead>{t('animals.status')}</TableHead>
                  <TableHead>{t('animals.breed')}</TableHead>
                  <TableHead>{t('animals.gender')}</TableHead>
                  <TableHead>{t('animals.onMedicationShort')}</TableHead>
                  <TableHead>{t('animals.vetRecommendation')}</TableHead>
                  <SortableHeader column="entry_date" label={t('animals.entryDate')} />
                  <SortableHeader column="latest_weight" label={t('animals.currentWeight')} />
                  <TableHead className="text-right">{t('animals.actions')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sortedAnimals.map((animal) => (
                  <TableRow
                    key={animal.id}
                    className={animal.has_abnormal_record ? 'bg-yellow-50' : ''}
                  >
                    <TableCell>
                      <input
                        type="checkbox"
                        aria-label={`選取動物 ${animal.ear_tag || animal.id}`}
                        checked={selectedAnimals.includes(animal.id)}
                        onChange={() => onToggleSelection(animal.id)}
                        className="rounded border-slate-300"
                      />
                    </TableCell>
                    <TableCell>
                      <Link
                        to={`/animals/${animal.id}`}
                        className="text-orange-600 hover:text-orange-700 hover:underline font-medium cursor-pointer block"
                        title={`點擊進入動物詳情 · 系統號: ${animal.id}`}
                      >
                        {animal.ear_tag}
                      </Link>
                    </TableCell>
                    <TableCell>{getPenLocationDisplay(animal, t)}</TableCell>
                    <TableCell>
                      {animal.iacuc_no || (
                        <span className="text-slate-400">未分配</span>
                      )}
                    </TableCell>
                    <TableCell>
                      <Badge className={statusColors[animal.status]}>
                        {t(`animals.statusLabels.${animal.status}`)}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      {animal.breed === 'other'
                        ? (animal.breed_other || t('animals.breedLabels.other'))
                        : t(`animals.breedLabels.${animal.breed}`)}
                    </TableCell>
                    <TableCell>{t(`animals.genderLabels.${animal.gender}`)}</TableCell>
                    <TableCell>
                      {animal.is_on_medication ? (
                        <Badge variant="destructive" className="text-xs">{t('animals.onMedication')}</Badge>
                      ) : (
                        <span className="text-slate-400">{t('animals.notOnMedication')}</span>
                      )}
                    </TableCell>
                    <TableCell>
                      {animal.vet_recommendation_date ? (
                        <span className="text-sm text-slate-600">
                          {new Date(animal.vet_recommendation_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })}
                        </span>
                      ) : (
                        <span className="text-slate-400">-</span>
                      )}
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <span>{new Date(animal.entry_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })}</span>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-6 w-6"
                          onClick={() => onQuickEdit(animal.id)}
                          title="快速編輯"
                        >
                          <Edit2 className="h-3 w-3" />
                        </Button>
                      </div>
                    </TableCell>
                    <TableCell>
                      {animal.latest_weight ? (
                        <span
                          className="text-sm text-slate-700 font-medium"
                          title={animal.latest_weight_date ? `量測日期: ${new Date(animal.latest_weight_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })}` : undefined}
                        >
                          {animal.latest_weight} kg
                        </span>
                      ) : (
                        <span className="text-slate-400">-</span>
                      )}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button variant="ghost" size="icon" asChild title={t('common.view')}>
                          <Link to={`/animals/${animal.id}`}>
                            <Eye className="h-4 w-4" />
                          </Link>
                        </Button>
                        <Button variant="ghost" size="icon" asChild title={t('common.edit')}>
                          <Link to={`/animals/${animal.id}/edit`}>
                            <Edit2 className="h-4 w-4" />
                          </Link>
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        )}

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-between pt-4">
            <p className="text-sm text-muted-foreground">
              {t('common.showingOf', '顯示第 {{from}}–{{to}} 筆，共 {{total}} 筆', {
                from: (page - 1) * perPage + 1,
                to: Math.min(page * perPage, totalAnimals),
                total: totalAnimals,
              })}
            </p>
            <div className="flex gap-1">
              <Button
                variant="outline"
                size="sm"
                disabled={page <= 1}
                onClick={() => onPageChange(Math.max(1, page - 1))}
              >
                {t('common.previous', '上一頁')}
              </Button>
              {Array.from({ length: Math.min(totalPages, 7) }, (_, i) => {
                let pageNum: number
                if (totalPages <= 7) {
                  pageNum = i + 1
                } else if (page <= 4) {
                  pageNum = i + 1
                } else if (page >= totalPages - 3) {
                  pageNum = totalPages - 6 + i
                } else {
                  pageNum = page - 3 + i
                }
                return (
                  <Button
                    key={pageNum}
                    variant={pageNum === page ? 'default' : 'outline'}
                    size="sm"
                    className="min-w-[36px]"
                    onClick={() => onPageChange(pageNum)}
                  >
                    {pageNum}
                  </Button>
                )
              })}
              <Button
                variant="outline"
                size="sm"
                disabled={page >= totalPages}
                onClick={() => onPageChange(Math.min(totalPages, page + 1))}
              >
                {t('common.next', '下一頁')}
              </Button>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
