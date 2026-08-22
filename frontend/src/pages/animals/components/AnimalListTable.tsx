import { Link } from 'react-router-dom'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useTranslation } from 'react-i18next'
import { animalSpeciesLabel } from '@/lib/animalSpecies'
import type { AnimalListItem } from '@/lib/api'
import { uiLocale } from '@/lib/utils'
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
import { Edit2, AlertCircle, ArrowUpDown, ArrowUp, ArrowDown } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { statusColors, getPenLocationDisplay } from '../constants'

interface AnimalListPagination {
  page: number
  totalPages: number
  totalAnimals: number
  perPage: number
  onPageChange: (page: number) => void
}

interface AnimalListSorting {
  sortColumn: string | null
  sortDirection: 'asc' | 'desc'
  onSort: (column: string) => void
}

interface AnimalListSelection {
  selectedAnimals: string[]
  onToggleSelection: (id: string) => void
  onToggleAll: () => void
}

interface AnimalListTableProps {
  animals: AnimalListItem[]
  isLoading: boolean
  onQuickEdit: (animalId: string) => void
  selection: AnimalListSelection
  sorting: AnimalListSorting
  pagination: AnimalListPagination
}

// 提到 module scope 避免每次父元件 render 產生新元件 type 造成 TableHead unmount/remount。
// asc / desc 改用 ArrowUp / ArrowDown 圖示，未排序欄位用中性 ArrowUpDown。
function SortableHeader({
  column,
  label,
  className,
  sortColumn,
  sortDirection,
  onSort,
}: {
  column: string
  label: string
  className?: string
  sortColumn: string | null
  sortDirection: 'asc' | 'desc'
  onSort: (column: string) => void
}) {
  const isActive = sortColumn === column
  const Icon = isActive
    ? sortDirection === 'desc' ? ArrowDown : ArrowUp
    : ArrowUpDown
  return (
    <TableHead
      className={`cursor-pointer hover:bg-muted select-none${className ? ` ${className}` : ''}`}
      onClick={() => onSort(column)}
    >
      <div className="flex items-center gap-1">
        {label}
        <Icon className={`h-3 w-3 ${isActive ? 'text-primary' : 'text-muted-foreground'}`} />
      </div>
    </TableHead>
  )
}

export function AnimalListTable({
  animals,
  isLoading,
  onQuickEdit,
  selection,
  sorting,
  pagination,
}: AnimalListTableProps) {
  const { selectedAnimals, onToggleSelection, onToggleAll } = selection
  const { sortColumn, sortDirection, onSort } = sorting
  const { page, totalPages, totalAnimals, perPage, onPageChange } = pagination
  const { t } = useTranslation()

  // R35-5: client-side sort 已移除，由後端 `sort_by` / `sort_order` 處理；
  // SortableHeader 提到 module scope，每次 caller 傳入 sortColumn/sortDirection/onSort。

  return (
    <div className="rounded-lg border bg-card overflow-hidden">
      {isLoading ? (
        <TableSkeleton rows={8} cols={12} />
      ) : animals.length === 0 ? (
        <Table>
          <TableBody>
            <TableEmptyRow
              colSpan={12}
              icon={AlertCircle}
              title={t('animals.noAnimalsFound')}
              description={t('animals.noAnimalsFoundDescription', '嘗試調整篩選條件，或新增第一筆動物紀錄')}
            />
          </TableBody>
        </Table>
      ) : (
        <>
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                  <TableHead className="w-12">
                    <input
                      type="checkbox"
                      aria-label="全選動物"
                      checked={selectedAnimals.length === animals.length && animals.length > 0}
                      onChange={onToggleAll}
                      className="rounded border-border"
                    />
                  </TableHead>
                  <SortableHeader column="ear_tag" label={t('animals.earTag')} sortColumn={sortColumn} sortDirection={sortDirection} onSort={onSort} />
                  <SortableHeader column="pen_location" label={t('animals.pen')} sortColumn={sortColumn} sortDirection={sortDirection} onSort={onSort} />
                  <SortableHeader column="iacuc_no" label={t('animals.iacucNo')} sortColumn={sortColumn} sortDirection={sortDirection} onSort={onSort} />
                  <TableHead>{t('animals.status')}</TableHead>
                  <TableHead className="hidden md:table-cell">{t('animals.breed')}</TableHead>
                  <TableHead className="hidden md:table-cell">{t('animals.gender')}</TableHead>
                  <TableHead className="hidden lg:table-cell">{t('animals.onMedicationShort')}</TableHead>
                  <SortableHeader column="entry_date" label={t('animals.entryDate')} sortColumn={sortColumn} sortDirection={sortDirection} onSort={onSort} />
                  <SortableHeader column="latest_weight" label={t('animals.currentWeight')} className="hidden md:table-cell" sortColumn={sortColumn} sortDirection={sortDirection} onSort={onSort} />
                  <TableHead className="text-right">{t('animals.actions')}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {animals.map((animal) => (
                  <TableRow
                    key={animal.id}
                    className={animal.has_abnormal_record ? 'bg-status-warning-bg' : ''}
                  >
                    <TableCell>
                      <input
                        type="checkbox"
                        aria-label={`選取動物 ${animal.ear_tag || animal.id}`}
                        checked={selectedAnimals.includes(animal.id)}
                        onChange={() => onToggleSelection(animal.id)}
                        className="rounded border-border"
                      />
                    </TableCell>
                    <TableCell>
                      <Link
                        to={`/animals/${animal.id}`}
                        className="text-status-warning-text hover:text-status-warning-text/80 hover:underline font-medium cursor-pointer block"
                        title={`點擊進入動物詳情 · 系統號: ${animal.id}`}
                      >
                        {animal.ear_tag}
                      </Link>
                    </TableCell>
                    <TableCell>{getPenLocationDisplay(animal, t)}</TableCell>
                    <TableCell>
                      {animal.iacuc_no || (
                        <span className="text-muted-foreground">未分配</span>
                      )}
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap items-center gap-1">
                        <Badge className={statusColors[animal.status]}>
                          {t(`animals.statusLabels.${animal.status}`)}
                        </Badge>
                        {/* issue #180：轉讓簽核期間動物仍在原欄、狀態維持「實驗完成」，
                            另掛一枚 chip 標示尚有未結案的轉讓申請。 */}
                        {animal.pending_transfer_status && (
                          <Badge
                            variant="outline"
                            className="text-xs"
                            title={t('animals.pendingTransferHint')}
                          >
                            {t('animals.pendingTransfer')}
                          </Badge>
                        )}
                      </div>
                    </TableCell>
                    <TableCell className="hidden md:table-cell">
                      {animalSpeciesLabel(animal, (b) => t(`animals.breedLabels.${b}`))}
                    </TableCell>
                    <TableCell className="hidden md:table-cell">{t(`animals.genderLabels.${animal.gender}`)}</TableCell>
                    <TableCell className="hidden lg:table-cell">
                      {animal.is_on_medication ? (
                        <Badge variant="destructive" className="text-xs">{t('animals.onMedication')}</Badge>
                      ) : (
                        <span className="text-muted-foreground">{t('animals.notOnMedication')}</span>
                      )}
                    </TableCell>
                    <TableCell>
                      {new Date(animal.entry_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })}
                    </TableCell>
                    <TableCell className="hidden md:table-cell">
                      {animal.latest_weight ? (
                        <span
                          className="text-sm text-foreground font-medium"
                          title={animal.latest_weight_date ? `量測日期: ${new Date(animal.latest_weight_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })}` : undefined}
                        >
                          {animal.latest_weight} kg
                        </span>
                      ) : (
                        <span className="text-muted-foreground">-</span>
                      )}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Can permission={PERMISSIONS.ANIMAL_ANIMAL_EDIT}>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => onQuickEdit(animal.id)}
                            title="快速編輯"
                            aria-label="快速編輯"
                          >
                            <Edit2 className="h-4 w-4" />
                          </Button>
                        </Can>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </>
        )}

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-between border-t px-4 py-3">
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
    </div>
  )
}
