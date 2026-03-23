/**
 * 設備管理分頁內容：搜尋、表格、分頁
 * 欄位：名稱、型號、序號、位置、狀態、廠商、確效/校正日期、查核日期、操作
 */
import { useState } from 'react'
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
import { Pencil, Trash2, Loader2, Search, Building2 } from 'lucide-react'
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

  const handleShowSuppliers = (equipmentId: string) => {
    setSelectedEquipmentId(equipmentId)
    setSupplierDialogOpen(true)
  }

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
                    <TableHead>名稱</TableHead>
                    <TableHead>型號</TableHead>
                    <TableHead>序號</TableHead>
                    <TableHead>位置</TableHead>
                    <TableHead>狀態</TableHead>
                    <TableHead>廠商</TableHead>
                    <TableHead>校正/確效到期</TableHead>
                    <TableHead>查核到期</TableHead>
                    {canManage && <TableHead className="w-[100px]">操作</TableHead>}
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {records.map((r) => {
                    const calType = r.calibration_type
                    const calInfo = calType && calType !== 'inspection'
                      ? getLatestCalibrationDate(r.id, calType, allCalibrations)
                      : { nextDue: null, isOverdue: false }
                    const inspInfo = r.inspection_cycle
                      ? getLatestCalibrationDate(r.id, 'inspection', allCalibrations)
                      : { nextDue: null, isOverdue: false }

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
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-7 px-2 text-xs"
                            onClick={() => handleShowSuppliers(r.id)}
                          >
                            <Building2 className="h-3 w-3 mr-1" />
                            查看
                          </Button>
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
                            <div className="flex gap-2">
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
