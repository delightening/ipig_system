/**
 * 維修/保養紀錄分頁內容：表格、分頁、新增/編輯/刪除
 */
import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { StatusBadge } from '@/components/ui/status-badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { ArrowDown, ArrowUp, ArrowUpDown, Check, History, Pencil, Plus, Trash2, Wrench, X } from 'lucide-react'
import { format } from 'date-fns'
import { cn, getDateFnsLocale } from '@/lib/utils'

import type { MaintenanceRecordWithDetails, MaintenanceStatus, MaintenanceType } from '../types'
import { getMaintenanceBadge, MAINTENANCE_STATUS_LABELS, MAINTENANCE_TYPE_LABELS } from '../types'

/** 篩選 + 排序控制（由 EquipmentPage 持有狀態，server-side 套用）。 */
export interface MaintenanceFilterSort {
  /** 類型篩選，'' = 全部 */
  type: string
  /** 狀態篩選，'' = 全部 */
  status: string
  /** 目前排序欄位（後端白名單 key），null = 預設 */
  sortField: string | null
  sortOrder: 'asc' | 'desc'
  onTypeChange: (value: string) => void
  onStatusChange: (value: string) => void
  onSortChange: (field: string) => void
}

const FILTER_ALL = 'all'
const TYPE_OPTIONS: MaintenanceType[] = ['repair', 'maintenance']
const STATUS_OPTIONS: MaintenanceStatus[] = ['pending', 'in_progress', 'pending_review', 'completed', 'unrepairable']

interface MaintenanceTabContentProps {
  canManage: boolean
  canReview?: boolean
  records: MaintenanceRecordWithDetails[]
  isLoading: boolean
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  onDelete: (id: string) => void
  filterSort: MaintenanceFilterSort
  onAdd?: () => void
  onEdit?: (record: MaintenanceRecordWithDetails) => void
  onViewHistory?: (id: string) => void
  onReview?: (record: MaintenanceRecordWithDetails) => void
  onReject?: (record: MaintenanceRecordWithDetails) => void
}

/** 可排序表頭內容（置於 DataTable 既有 TableHead 內，故僅渲染可點擊內容）。 */
function SortHeader({
  label,
  field,
  sortField,
  sortOrder,
  onSort,
}: {
  label: string
  field: string
  sortField: string | null
  sortOrder: 'asc' | 'desc'
  onSort: (field: string) => void
}) {
  const active = sortField === field
  const Icon = active ? (sortOrder === 'asc' ? ArrowUp : ArrowDown) : ArrowUpDown
  return (
    <button
      type="button"
      onClick={() => onSort(field)}
      className="flex items-center gap-1 hover:text-foreground"
    >
      <span>{label}</span>
      <Icon className={cn('h-3 w-3 shrink-0', active ? 'text-foreground' : 'text-muted-foreground/50')} />
    </button>
  )
}

export function MaintenanceTabContent({
  canManage,
  canReview,
  records,
  isLoading,
  page,
  totalPages,
  onPageChange,
  onDelete,
  filterSort,
  onAdd,
  onEdit,
  onViewHistory,
  onReview,
  onReject,
}: MaintenanceTabContentProps) {
  const { t } = useTranslation()
  const columns = useMemo<ColumnDef<MaintenanceRecordWithDetails>[]>(() => {
    const cols: ColumnDef<MaintenanceRecordWithDetails>[] = [
      {
        key: 'equipment',
        header: <SortHeader label={t('admin.maintenanceTabContent.colEquipment')} field="equipment_name" sortField={filterSort.sortField} sortOrder={filterSort.sortOrder} onSort={filterSort.onSortChange} />,
        cell: (r) => <span className="font-medium">{r.equipment_name}</span>,
      },
      {
        key: 'status',
        header: <SortHeader label={t('admin.maintenanceTabContent.colStatus')} field="status" sortField={filterSort.sortField} sortOrder={filterSort.sortOrder} onSort={filterSort.onSortChange} />,
        cell: (r) => {
          const badge = getMaintenanceBadge(r.maintenance_type, r.status)
          return <StatusBadge variant={badge.variant}>{t(badge.labelKey)}</StatusBadge>
        },
      },
      {
        key: 'reported',
        header: <SortHeader label={t('admin.maintenanceTabContent.colReported')} field="reported_at" sortField={filterSort.sortField} sortOrder={filterSort.sortOrder} onSort={filterSort.onSortChange} />,
        cell: (r) => format(new Date(r.reported_at), 'yyyy/MM/dd', { locale: getDateFnsLocale() }),
      },
      {
        key: 'completed',
        header: <SortHeader label={t('admin.maintenanceTabContent.colCompleted')} field="completed_at" sortField={filterSort.sortField} sortOrder={filterSort.sortOrder} onSort={filterSort.onSortChange} />,
        cell: (r) => r.completed_at ? format(new Date(r.completed_at), 'yyyy/MM/dd', { locale: getDateFnsLocale() }) : '—',
      },
      {
        key: 'description', header: t('admin.maintenanceTabContent.colDescription'), className: 'max-w-[200px] whitespace-normal break-words',
        cell: (r) => r.maintenance_type === 'repair' ? r.problem_description : r.maintenance_items,
      },
    ]
    if (canManage || canReview || onViewHistory) {
      cols.push({
        key: 'actions', header: t('common.actions'), className: 'w-[150px] text-right',
        cell: (r) => (
          <div className="flex items-center justify-end gap-1">
            {canReview && r.status === 'pending_review' && onReview && (
              <Button variant="ghost" size="icon" className="text-status-success-text hover:text-status-success-text/80" onClick={() => onReview(r)} aria-label={t('admin.maintenanceTabContent.actionApprove')}>
                <Check className="h-4 w-4" />
              </Button>
            )}
            {canReview && r.status === 'pending_review' && onReject && (
              <Button variant="ghost" size="icon" className="text-destructive hover:text-destructive" onClick={() => onReject(r)} aria-label={t('admin.maintenanceTabContent.actionReject')}>
                <X className="h-4 w-4" />
              </Button>
            )}
            {onViewHistory && (
              <Button variant="ghost" size="icon" onClick={() => onViewHistory(r.id)} aria-label={t('admin.maintenanceTabContent.actionHistory')}>
                <History className="h-4 w-4" />
              </Button>
            )}
            {canManage && onEdit && (
              <Button variant="ghost" size="icon" onClick={() => onEdit(r)} aria-label={t('common.edit')}>
                <Pencil className="h-4 w-4" />
              </Button>
            )}
            {canManage && (
              <Button variant="ghost" size="icon" className="text-destructive hover:text-destructive" onClick={() => onDelete(r.id)} aria-label={t('common.delete')}>
                <Trash2 className="h-4 w-4" />
              </Button>
            )}
          </div>
        ),
      })
    }
    return cols
  }, [t, canManage, canReview, onDelete, onEdit, onViewHistory, onReview, onReject, filterSort.sortField, filterSort.sortOrder, filterSort.onSortChange])

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>{t('admin.maintenanceTabContent.cardTitle')}</CardTitle>
            <CardDescription>{t('admin.maintenanceTabContent.cardDescription')}</CardDescription>
          </div>
          {canManage && onAdd && (
            <Button size="sm" onClick={onAdd}>
              <Plus className="h-4 w-4 mr-2" />
              {t('admin.maintenanceTabContent.addButton')}
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex flex-wrap items-center gap-3">
          <Select
            value={filterSort.type || FILTER_ALL}
            onValueChange={(v) => filterSort.onTypeChange(v === FILTER_ALL ? '' : v)}
          >
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={FILTER_ALL}>{t('admin.maintenanceTabContent.filterAllTypes')}</SelectItem>
              {TYPE_OPTIONS.map((mt) => (
                <SelectItem key={mt} value={mt}>{t(MAINTENANCE_TYPE_LABELS[mt])}</SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select
            value={filterSort.status || FILTER_ALL}
            onValueChange={(v) => filterSort.onStatusChange(v === FILTER_ALL ? '' : v)}
          >
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={FILTER_ALL}>{t('admin.maintenanceTabContent.filterAllStatuses')}</SelectItem>
              {STATUS_OPTIONS.map((s) => (
                <SelectItem key={s} value={s}>{t(MAINTENANCE_STATUS_LABELS[s])}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <DataTable
          columns={columns}
          data={records}
          isLoading={isLoading}
          emptyIcon={Wrench}
          emptyTitle={t('admin.maintenanceTabContent.emptyTitle')}
          rowKey={(r) => r.id}
          page={page}
          totalPages={totalPages}
          onPageChange={onPageChange}
        />
      </CardContent>
    </Card>
  )
}
