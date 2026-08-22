/**
 * 設備管理分頁內容：搜尋、表格、分頁
 * 欄位：名稱、型號、序號、位置、狀態、廠商、確效/校正日期、查核日期、操作
 */
import { useState, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { useQuery } from '@tanstack/react-query'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { StatusBadge } from '@/components/ui/status-badge'
import type { StatusVariant } from '@/components/ui/status-badge'
import { FilterBar } from '@/components/ui/filter-bar'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { Pencil, Trash2, Building2, ArrowUpDown, Pause, Play } from 'lucide-react'
import { Link } from 'react-router-dom'
import api from '@/lib/api'

import type {
  Equipment,
  CalibrationWithEquipment,
  EquipmentSupplierWithPartner,
} from '../types'
import {
  EQUIPMENT_STATUS_LABELS,
  CALIBRATION_TYPE_LABELS,
} from '../types'

interface SupplierSummaryRow {
  equipment_id: string
  partner_name: string
}

type SortKey = 'name' | 'model' | 'serial_number' | 'location' | 'status' | 'calibration_due' | 'inspection_due'

interface EquipmentTableProps {
  records: Equipment[]
  isLoading: boolean
  page: number
  totalPages: number
  onPageChange: (page: number) => void
}

interface EquipmentActions {
  onEdit: (equip: Equipment) => void
  onDelete: (id: string, name: string) => void
  onRequestIdle?: (equipmentId: string, requestType: 'idle' | 'restore') => void
}

interface EquipmentTabContentProps {
  canManage: boolean
  keyword: string
  onKeywordChange: (v: string) => void
  statusFilter: string
  onStatusFilterChange: (v: string) => void
  allCalibrations: CalibrationWithEquipment[]
  tableProps: EquipmentTableProps
  actions: EquipmentActions
}

const STATUS_VARIANT: Record<string, StatusVariant> = {
  active: 'success',
  inactive: 'neutral',
  under_repair: 'warning',
  decommissioned: 'error',
}

function getLatestCalibrationDate(
  equipmentId: string,
  type: 'calibration' | 'validation' | 'inspection',
  allCalibrations: CalibrationWithEquipment[],
): { nextDue: string | null; isOverdue: boolean } {
  const records = allCalibrations
    .filter((c) => c.equipment_id === equipmentId && c.calibration_type === type)
    .sort((a, b) => b.calibrated_at.localeCompare(a.calibrated_at))

  const latest = records[0]
  if (!latest?.next_due_at) return { nextDue: null, isOverdue: false }

  const isOverdue = new Date(latest.next_due_at) < new Date()
  return { nextDue: latest.next_due_at, isOverdue }
}

export function EquipmentTabContent({
  canManage,
  keyword,
  onKeywordChange,
  statusFilter,
  onStatusFilterChange,
  allCalibrations,
  tableProps,
  actions,
}: EquipmentTabContentProps) {
  const { t } = useTranslation()
  const { records, isLoading, page, totalPages, onPageChange } = tableProps
  const { onEdit, onDelete, onRequestIdle } = actions
  const [supplierDialogOpen, setSupplierDialogOpen] = useState(false)
  const [selectedEquipmentId, setSelectedEquipmentId] = useState<string | null>(null)
  const [sortColumn, setSortColumn] = useState<SortKey | null>(null)
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc')
  const selectedEquipment = records.find((r) => r.id === selectedEquipmentId)

  const { data: suppliers = [], isError: suppliersError } = useQuery({
    queryKey: ['equipment-suppliers', selectedEquipmentId],
    queryFn: async () => {
      if (!selectedEquipmentId) return []
      const res = await api.get<EquipmentSupplierWithPartner[]>(
        `/equipment/${selectedEquipmentId}/suppliers`,
      )
      return res.data
    },
    enabled: !!selectedEquipmentId && supplierDialogOpen,
  })

  const { data: supplierSummary = [], isError: supplierSummaryError } = useQuery({
    queryKey: ['equipment-suppliers-summary'],
    queryFn: async () => {
      const res = await api.get<SupplierSummaryRow[]>('/equipment-suppliers/summary')
      return res.data
    },
  })

  const supplierMap = useMemo(() => {
    const map = new Map<string, string[]>()
    for (const row of supplierSummary) {
      const list = map.get(row.equipment_id) ?? []
      list.push(row.partner_name)
      map.set(row.equipment_id, list)
    }
    return map
  }, [supplierSummary])

  const handleShowSuppliers = (equipmentId: string) => {
    setSelectedEquipmentId(equipmentId)
    setSupplierDialogOpen(true)
  }

  const handleSort = (column: SortKey) => {
    if (sortColumn === column) {
      setSortDirection((prev) => (prev === 'asc' ? 'desc' : 'asc'))
    } else {
      setSortColumn(column)
      setSortDirection('asc')
    }
  }

  const sortableHeader = (column: SortKey, label: string) => (
    <button
      type="button"
      className="flex items-center gap-1 cursor-pointer select-none"
      onClick={() => handleSort(column)}
    >
      {label}
      <ArrowUpDown className={`h-3 w-3 ${sortColumn === column ? 'text-primary' : 'text-muted-foreground'}`} />
    </button>
  )

  const sortedRecords = useMemo(() => {
    if (!sortColumn) return records

    return [...records].sort((a, b) => {
      let aVal: string | number
      let bVal: string | number

      if (sortColumn === 'calibration_due') {
        const aType = a.calibration_type
        const aCal = aType && aType !== 'inspection'
          ? getLatestCalibrationDate(a.id, aType, allCalibrations)
          : { nextDue: null, isOverdue: false }
        const bType = b.calibration_type
        const bCal = bType && bType !== 'inspection'
          ? getLatestCalibrationDate(b.id, bType, allCalibrations)
          : { nextDue: null, isOverdue: false }
        aVal = aCal.nextDue ? new Date(aCal.nextDue).getTime() : (sortDirection === 'asc' ? Infinity : -Infinity)
        bVal = bCal.nextDue ? new Date(bCal.nextDue).getTime() : (sortDirection === 'asc' ? Infinity : -Infinity)
      } else if (sortColumn === 'inspection_due') {
        const aInsp = a.inspection_cycle
          ? getLatestCalibrationDate(a.id, 'inspection', allCalibrations)
          : { nextDue: null, isOverdue: false }
        const bInsp = b.inspection_cycle
          ? getLatestCalibrationDate(b.id, 'inspection', allCalibrations)
          : { nextDue: null, isOverdue: false }
        aVal = aInsp.nextDue ? new Date(aInsp.nextDue).getTime() : (sortDirection === 'asc' ? Infinity : -Infinity)
        bVal = bInsp.nextDue ? new Date(bInsp.nextDue).getTime() : (sortDirection === 'asc' ? Infinity : -Infinity)
      } else if (sortColumn === 'status') {
        const order: Record<string, number> = { active: 0, under_repair: 1, decommissioned: 2 }
        aVal = order[a.status] ?? 99
        bVal = order[b.status] ?? 99
      } else {
        aVal = (a[sortColumn] ?? '').toLowerCase()
        bVal = (b[sortColumn] ?? '').toLowerCase()
      }

      if (aVal < bVal) return sortDirection === 'asc' ? -1 : 1
      if (aVal > bVal) return sortDirection === 'asc' ? 1 : -1
      return 0
    })
  }, [records, sortColumn, sortDirection, allCalibrations])

  const columns = useMemo<ColumnDef<Equipment>[]>(() => {
    const cols: ColumnDef<Equipment>[] = [
      { key: 'name', header: sortableHeader('name', t('admin.equipmentTabContent.columns.name')), cell: (r) => <Link to={`/equipment/${r.id}/history`} className="font-medium text-primary hover:underline">{r.name}</Link> },
      { key: 'model', header: sortableHeader('model', t('admin.equipmentTabContent.columns.model')), cell: (r) => r.model || '—' },
      { key: 'serial', header: sortableHeader('serial_number', t('admin.equipmentTabContent.columns.serialNumber')), cell: (r) => r.serial_number || '—' },
      { key: 'location', header: sortableHeader('location', t('admin.equipmentTabContent.columns.location')), cell: (r) => r.location || '—' },
      {
        key: 'status', header: sortableHeader('status', t('admin.equipmentTabContent.columns.status')),
        cell: (r) => (
          <StatusBadge variant={STATUS_VARIANT[r.status] || 'neutral'}>
            {t(EQUIPMENT_STATUS_LABELS[r.status])}
          </StatusBadge>
        ),
      },
      {
        key: 'supplier', header: t('admin.equipmentTabContent.columns.supplier'),
        cell: (r) => {
          const names = supplierMap.get(r.id)
          return names && names.length > 0 ? (
            <button type="button" className="text-left text-sm text-primary hover:underline" onClick={() => handleShowSuppliers(r.id)}>
              {names.join('、')}
            </button>
          ) : <span className="text-muted-foreground">—</span>
        },
      },
      {
        key: 'calDue', header: sortableHeader('calibration_due', t('admin.equipmentTabContent.columns.calibrationDue')),
        cell: (r) => {
          const calType = r.calibration_type
          const calInfo = calType && calType !== 'inspection'
            ? getLatestCalibrationDate(r.id, calType, allCalibrations)
            : { nextDue: null, isOverdue: false }
          if (!calInfo.nextDue) return <span className="text-muted-foreground">—</span>
          return (
            <span className={calInfo.isOverdue ? 'text-destructive font-semibold' : ''}>
              {calType ? t(CALIBRATION_TYPE_LABELS[calType]) : ''} {calInfo.nextDue}
              {calInfo.isOverdue && ` (${t('admin.equipmentTabContent.overdue')})`}
            </span>
          )
        },
      },
      {
        key: 'inspDue', header: sortableHeader('inspection_due', t('admin.equipmentTabContent.columns.inspectionDue')),
        cell: (r) => {
          const inspInfo = r.inspection_cycle
            ? getLatestCalibrationDate(r.id, 'inspection', allCalibrations)
            : { nextDue: null, isOverdue: false }
          if (!inspInfo.nextDue) return <span className="text-muted-foreground">—</span>
          return (
            <span className={inspInfo.isOverdue ? 'text-destructive font-semibold' : ''}>
              {inspInfo.nextDue}
              {inspInfo.isOverdue && ` (${t('admin.equipmentTabContent.overdue')})`}
            </span>
          )
        },
      },
    ]
    if (canManage) {
      cols.push({
        key: 'actions', header: t('common.actions'), className: 'w-[140px] text-right',
        cell: (r) => (
          <div className="flex items-center justify-end gap-1">
            {onRequestIdle && r.status === 'active' && (
              <Button variant="ghost" size="icon" onClick={() => onRequestIdle(r.id, 'idle')} aria-label={t('admin.equipmentTabContent.requestIdle')} title={t('admin.equipmentTabContent.requestIdle')}>
                <Pause className="h-4 w-4" />
              </Button>
            )}
            {onRequestIdle && r.status === 'inactive' && (
              <Button variant="ghost" size="icon" onClick={() => onRequestIdle(r.id, 'restore')} aria-label={t('admin.equipmentTabContent.requestRestore')} title={t('admin.equipmentTabContent.requestRestoreTitle')}>
                <Play className="h-4 w-4" />
              </Button>
            )}
            <Button variant="ghost" size="icon" onClick={() => onEdit(r)} aria-label={t('common.edit')}>
              <Pencil className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" className="text-destructive hover:text-destructive" onClick={() => onDelete(r.id, r.name)} aria-label={t('common.delete')}>
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        ),
      })
    }
    return cols
  // t 加入 deps：語系切換時 react-i18next 的 t identity 變更 → 表頭/標籤即時重新翻譯
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [canManage, sortColumn, sortDirection, supplierMap, allCalibrations, onEdit, onDelete, onRequestIdle, t])

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>{t('admin.equipmentTabContent.cardTitle')}</CardTitle>
          <CardDescription>{t('admin.equipmentTabContent.cardDescription')}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <FilterBar
            search={keyword}
            onSearchChange={onKeywordChange}
            searchPlaceholder={t('admin.equipmentTabContent.searchPlaceholder')}
            hasActiveFilters={!!statusFilter}
            onClearFilters={() => onStatusFilterChange('')}
          >
            <select
              value={statusFilter}
              onChange={(e) => onStatusFilterChange(e.target.value)}
              className="h-9 rounded-md border border-input bg-background px-3 text-sm"
              aria-label={t('admin.equipmentTabContent.statusFilterLabel')}
            >
              <option value="">{t('common.allStatus')}</option>
              {Object.entries(EQUIPMENT_STATUS_LABELS).map(([value, label]) => (
                <option key={value} value={value}>{t(label)}</option>
              ))}
            </select>
          </FilterBar>
          {/* summary 失敗時「廠商」欄每一列都會退成「—」，跟「這台設備沒登錄廠商」
              完全同形。逐列標記太吵，改在表格上方講一次，讓使用者知道那些破折號
              不是事實。 */}
          {supplierSummaryError && supplierSummary.length === 0 && (
            <p className="text-sm text-status-error-text">
              {t('admin.equipmentTabContent.supplierSummaryLoadFailed')}
            </p>
          )}
          <DataTable
            columns={columns}
            data={sortedRecords}
            isLoading={isLoading}
            emptyIcon={Building2}
            emptyTitle={t('admin.equipmentTabContent.emptyTitle')}
            rowKey={(r) => r.id}
            page={page}
            totalPages={totalPages}
            onPageChange={onPageChange}
          />
        </CardContent>
      </Card>

      {/* 廠商詳細資訊 Dialog */}
      <Dialog open={supplierDialogOpen} onOpenChange={setSupplierDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {t('admin.equipmentTabContent.supplierDialogTitle', { name: selectedEquipment?.name ?? '' })}
            </DialogTitle>
          </DialogHeader>
          {suppliersError && suppliers.length === 0 ? (
            <p className="text-status-error-text text-center py-4">{t('common.loadFailed')}</p>
          ) : suppliers.length === 0 ? (
            <p className="text-muted-foreground text-center py-4">{t('admin.equipmentTabContent.noSuppliers')}</p>
          ) : (
            <div className="space-y-3">
              {suppliers.map((s) => {
                const phone = s.contact_phone || s.partner_phone
                const phoneExt = !s.contact_phone && s.partner_phone_ext ? ` ${t('admin.equipmentTabContent.phoneExt', { ext: s.partner_phone_ext })}` : ''
                const email = s.contact_email || s.partner_email
                const contactPerson = s.contact_person
                const address = s.partner_address
                return (
                  <div key={s.id} className="rounded-lg border p-3 space-y-1">
                    <p className="font-medium">{s.partner_name}</p>
                    {contactPerson && (
                      <p className="text-sm text-muted-foreground">{t('admin.equipmentTabContent.contactPerson')}：{contactPerson}</p>
                    )}
                    {phone && (
                      <p className="text-sm text-muted-foreground">{t('admin.equipmentTabContent.phone')}：{phone}{phoneExt}</p>
                    )}
                    {email && (
                      <p className="text-sm text-muted-foreground">{t('common.email')}：{email}</p>
                    )}
                    {address && (
                      <p className="text-sm text-muted-foreground">{t('admin.equipmentTabContent.address')}：{address}</p>
                    )}
                  </div>
                )
              })}
            </div>
          )}
        </DialogContent>
      </Dialog>
    </>
  )
}
