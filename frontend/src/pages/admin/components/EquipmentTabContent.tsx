/**
 * 設備管理分頁內容：搜尋、表格、分頁
 * 欄位：名稱、型號、序號、位置、狀態、廠商、確效/校正日期、查核日期、操作
 */
import { useState, useMemo } from 'react'
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
  const { records, isLoading, page, totalPages, onPageChange } = tableProps
  const { onEdit, onDelete, onRequestIdle } = actions
  const [supplierDialogOpen, setSupplierDialogOpen] = useState(false)
  const [selectedEquipmentId, setSelectedEquipmentId] = useState<string | null>(null)
  const [sortColumn, setSortColumn] = useState<SortKey | null>(null)
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc')
  const selectedEquipment = records.find((r) => r.id === selectedEquipmentId)

  const { data: suppliers = [] } = useQuery({
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

  const { data: supplierSummary = [] } = useQuery({
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
      { key: 'name', header: sortableHeader('name', '名稱'), cell: (r) => <Link to={`/equipment/${r.id}/history`} className="font-medium text-primary hover:underline">{r.name}</Link> },
      { key: 'model', header: sortableHeader('model', '型號'), cell: (r) => r.model || '—' },
      { key: 'serial', header: sortableHeader('serial_number', '序號'), cell: (r) => r.serial_number || '—' },
      { key: 'location', header: sortableHeader('location', '位置'), cell: (r) => r.location || '—' },
      {
        key: 'status', header: sortableHeader('status', '狀態'),
        cell: (r) => (
          <StatusBadge variant={STATUS_VARIANT[r.status] || 'neutral'}>
            {EQUIPMENT_STATUS_LABELS[r.status]}
          </StatusBadge>
        ),
      },
      {
        key: 'supplier', header: '廠商',
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
        key: 'calDue', header: sortableHeader('calibration_due', '校正/確效到期'),
        cell: (r) => {
          const calType = r.calibration_type
          const calInfo = calType && calType !== 'inspection'
            ? getLatestCalibrationDate(r.id, calType, allCalibrations)
            : { nextDue: null, isOverdue: false }
          if (!calInfo.nextDue) return <span className="text-muted-foreground">—</span>
          return (
            <span className={calInfo.isOverdue ? 'text-destructive font-semibold' : ''}>
              {calType ? CALIBRATION_TYPE_LABELS[calType] : ''} {calInfo.nextDue}
              {calInfo.isOverdue && ' (逾期)'}
            </span>
          )
        },
      },
      {
        key: 'inspDue', header: sortableHeader('inspection_due', '查核到期'),
        cell: (r) => {
          const inspInfo = r.inspection_cycle
            ? getLatestCalibrationDate(r.id, 'inspection', allCalibrations)
            : { nextDue: null, isOverdue: false }
          if (!inspInfo.nextDue) return <span className="text-muted-foreground">—</span>
          return (
            <span className={inspInfo.isOverdue ? 'text-destructive font-semibold' : ''}>
              {inspInfo.nextDue}
              {inspInfo.isOverdue && ' (逾期)'}
            </span>
          )
        },
      },
    ]
    if (canManage) {
      cols.push({
        key: 'actions', header: '操作', className: 'w-[140px] text-right',
        cell: (r) => (
          <div className="flex items-center justify-end gap-1">
            {onRequestIdle && r.status === 'active' && (
              <Button variant="ghost" size="icon" onClick={() => onRequestIdle(r.id, 'idle')} aria-label="申請閒置" title="申請閒置">
                <Pause className="h-4 w-4" />
              </Button>
            )}
            {onRequestIdle && r.status === 'inactive' && (
              <Button variant="ghost" size="icon" onClick={() => onRequestIdle(r.id, 'restore')} aria-label="申請恢復" title="申請恢復啟用">
                <Play className="h-4 w-4" />
              </Button>
            )}
            <Button variant="ghost" size="icon" onClick={() => onEdit(r)} aria-label="編輯">
              <Pencil className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" className="text-destructive hover:text-destructive" onClick={() => onDelete(r.id, r.name)} aria-label="刪除">
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        ),
      })
    }
    return cols
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [canManage, sortColumn, sortDirection, supplierMap, allCalibrations, onEdit, onDelete, onRequestIdle])

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>設備清單</CardTitle>
          <CardDescription>管理實驗室設備，搜尋並維護設備基本資料</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <FilterBar
            search={keyword}
            onSearchChange={onKeywordChange}
            searchPlaceholder="搜尋設備名稱或型號..."
            hasActiveFilters={!!statusFilter}
            onClearFilters={() => onStatusFilterChange('')}
          >
            <select
              value={statusFilter}
              onChange={(e) => onStatusFilterChange(e.target.value)}
              className="h-9 rounded-md border border-input bg-background px-3 text-sm"
            >
              <option value="">全部狀態</option>
              {Object.entries(EQUIPMENT_STATUS_LABELS).map(([value, label]) => (
                <option key={value} value={value}>{label}</option>
              ))}
            </select>
          </FilterBar>
          <DataTable
            columns={columns}
            data={sortedRecords}
            isLoading={isLoading}
            emptyIcon={Building2}
            emptyTitle="尚無設備"
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
              {selectedEquipment?.name} — 廠商資訊
            </DialogTitle>
          </DialogHeader>
          {suppliers.length === 0 ? (
            <p className="text-muted-foreground text-center py-4">尚未關聯廠商</p>
          ) : (
            <div className="space-y-3">
              {suppliers.map((s) => {
                const phone = s.contact_phone || s.partner_phone
                const phoneExt = !s.contact_phone && s.partner_phone_ext ? ` 分機 ${s.partner_phone_ext}` : ''
                const email = s.contact_email || s.partner_email
                const contactPerson = s.contact_person
                const address = s.partner_address
                return (
                  <div key={s.id} className="rounded-lg border p-3 space-y-1">
                    <p className="font-medium">{s.partner_name}</p>
                    {contactPerson && (
                      <p className="text-sm text-muted-foreground">聯絡人：{contactPerson}</p>
                    )}
                    {phone && (
                      <p className="text-sm text-muted-foreground">電話：{phone}{phoneExt}</p>
                    )}
                    {email && (
                      <p className="text-sm text-muted-foreground">Email：{email}</p>
                    )}
                    {address && (
                      <p className="text-sm text-muted-foreground">地址：{address}</p>
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
