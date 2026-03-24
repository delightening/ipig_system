/**
 * 設備管理分頁內容：搜尋、表格、分頁
 * 欄位：名稱、型號、序號、位置、狀態、廠商、確效/校正日期、查核日期、操作
 */
import { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'
import { Pencil, Trash2, Loader2, Search, Building2, ArrowUpDown } from 'lucide-react'
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

type SortKey = 'name' | 'model' | 'serial_number' | 'location' | 'calibration_due' | 'inspection_due'

interface EquipmentTabContentProps {
  canManage: boolean
  keyword: string
  onKeywordChange: (v: string) => void
  isLoading: boolean
  records: Equipment[]
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  onEdit: (equip: Equipment) => void
  onDelete: (id: string, name: string) => void
  allCalibrations: CalibrationWithEquipment[]
}

const STATUS_COLORS: Record<string, string> = {
  active: 'bg-green-100 text-green-800',
  inactive: 'bg-gray-100 text-gray-800',
  under_repair: 'bg-yellow-100 text-yellow-800',
  decommissioned: 'bg-red-100 text-red-800',
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
  isLoading,
  records,
  page,
  totalPages,
  onPageChange,
  onEdit,
  onDelete,
  allCalibrations,
}: EquipmentTabContentProps) {
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

  const SortableHeader = ({ column, label }: { column: SortKey; label: string }) => (
    <TableHead
      className="cursor-pointer select-none hover:bg-muted/50"
      onClick={() => handleSort(column)}
    >
      <div className="flex items-center gap-1">
        {label}
        <ArrowUpDown className={`h-3 w-3 ${sortColumn === column ? 'text-primary' : 'text-muted-foreground'}`} />
      </div>
    </TableHead>
  )

  const sortedRecords = useMemo(() => {
    if (!sortColumn) return records

    return [...records].sort((a, b) => {
      let aVal: string | number = ''
      let bVal: string | number = ''

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
      } else {
        aVal = (a[sortColumn] ?? '').toLowerCase()
        bVal = (b[sortColumn] ?? '').toLowerCase()
      }

      if (aVal < bVal) return sortDirection === 'asc' ? -1 : 1
      if (aVal > bVal) return sortDirection === 'asc' ? 1 : -1
      return 0
    })
  }, [records, sortColumn, sortDirection, allCalibrations])

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>設備清單</CardTitle>
          <CardDescription>管理實驗室設備，搜尋並維護設備基本資料</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="relative flex-1 max-w-sm">
            <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="搜尋設備名稱或型號..."
              value={keyword}
              onChange={(e) => onKeywordChange(e.target.value)}
              className="pl-8"
            />
          </div>
          <div className="rounded-md border">
            {isLoading ? (
              <div className="flex justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : records.length === 0 ? (
              <div className="py-12 text-center text-muted-foreground">尚無設備</div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <SortableHeader column="name" label="名稱" />
                    <SortableHeader column="model" label="型號" />
                    <SortableHeader column="serial_number" label="序號" />
                    <SortableHeader column="location" label="位置" />
                    <TableHead>狀態</TableHead>
                    <TableHead>廠商</TableHead>
                    <SortableHeader column="calibration_due" label="校正/確效到期" />
                    <SortableHeader column="inspection_due" label="查核到期" />
                    {canManage && <TableHead className="w-[100px] text-right">操作</TableHead>}
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {sortedRecords.map((r) => {
                    const calType = r.calibration_type
                    const calInfo = calType && calType !== 'inspection'
                      ? getLatestCalibrationDate(r.id, calType, allCalibrations)
                      : { nextDue: null, isOverdue: false }
                    const inspInfo = r.inspection_cycle
                      ? getLatestCalibrationDate(r.id, 'inspection', allCalibrations)
                      : { nextDue: null, isOverdue: false }
                    const names = supplierMap.get(r.id)

                    return (
                      <TableRow key={r.id}>
                        <TableCell className="font-medium">{r.name}</TableCell>
                        <TableCell>{r.model || '—'}</TableCell>
                        <TableCell>{r.serial_number || '—'}</TableCell>
                        <TableCell>{r.location || '—'}</TableCell>
                        <TableCell>
                          <Badge variant="outline" className={STATUS_COLORS[r.status] || ''}>
                            {EQUIPMENT_STATUS_LABELS[r.status]}
                          </Badge>
                        </TableCell>
                        <TableCell>
                          {names && names.length > 0 ? (
                            <button
                              type="button"
                              className="text-left text-sm text-primary hover:underline"
                              onClick={() => handleShowSuppliers(r.id)}
                            >
                              {names.join('、')}
                            </button>
                          ) : (
                            <span className="text-muted-foreground">—</span>
                          )}
                        </TableCell>
                        <TableCell>
                          {calInfo.nextDue ? (
                            <span className={calInfo.isOverdue ? 'text-red-600 font-semibold' : ''}>
                              {calType ? CALIBRATION_TYPE_LABELS[calType] : ''}{' '}
                              {calInfo.nextDue}
                              {calInfo.isOverdue && ' (逾期)'}
                            </span>
                          ) : (
                            <span className="text-muted-foreground">—</span>
                          )}
                        </TableCell>
                        <TableCell>
                          {inspInfo.nextDue ? (
                            <span className={inspInfo.isOverdue ? 'text-red-600 font-semibold' : ''}>
                              {inspInfo.nextDue}
                              {inspInfo.isOverdue && ' (逾期)'}
                            </span>
                          ) : (
                            <span className="text-muted-foreground">—</span>
                          )}
                        </TableCell>
                        {canManage && (
                          <TableCell>
                            <div className="flex items-center justify-end gap-1">
                              <Button variant="ghost" size="icon" onClick={() => onEdit(r)}>
                                <Pencil className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                className="text-destructive hover:text-destructive"
                                onClick={() => onDelete(r.id, r.name)}
                              >
                                <Trash2 className="h-4 w-4" />
                              </Button>
                            </div>
                          </TableCell>
                        )}
                      </TableRow>
                    )
                  })}
                </TableBody>
              </Table>
            )}
          </div>
          {totalPages > 1 && (
            <div className="flex justify-center gap-2">
              <Button
                variant="outline"
                size="sm"
                disabled={page <= 1}
                onClick={() => onPageChange(page - 1)}
              >
                上一頁
              </Button>
              <span className="flex items-center px-4 text-sm text-muted-foreground">
                第 {page} / {totalPages} 頁
              </span>
              <Button
                variant="outline"
                size="sm"
                disabled={page >= totalPages}
                onClick={() => onPageChange(page + 1)}
              >
                下一頁
              </Button>
            </div>
          )}
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
              {suppliers.map((s) => (
                <div key={s.id} className="rounded-lg border p-3 space-y-1">
                  <p className="font-medium">{s.partner_name}</p>
                  {s.contact_person && (
                    <p className="text-sm text-muted-foreground">聯絡人：{s.contact_person}</p>
                  )}
                  {s.contact_phone && (
                    <p className="text-sm text-muted-foreground">電話：{s.contact_phone}</p>
                  )}
                  {s.contact_email && (
                    <p className="text-sm text-muted-foreground">Email：{s.contact_email}</p>
                  )}
                </div>
              ))}
            </div>
          )}
        </DialogContent>
      </Dialog>
    </>
  )
}
