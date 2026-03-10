/**
 * 校正紀錄分頁內容：設備篩選、表格、分頁
 */
import { useState } from 'react'
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
import { Plus, Pencil, Trash2, Loader2, Wrench, Ruler, Search } from 'lucide-react'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'

import type { Equipment, CalibrationWithEquipment } from '../types'

interface CalibrationTabContentProps {
  canManage: boolean
  equipmentList: Equipment[]
  calibEquipmentFilter: string
  onFilterChange: (id: string) => void
  isLoading: boolean
  records: CalibrationWithEquipment[]
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  onAddClick: () => void
  onEdit: (calib: CalibrationWithEquipment) => void
  onDelete: (id: string) => void
}

export function CalibrationTabContent({
  canManage,
  equipmentList,
  calibEquipmentFilter,
  onFilterChange,
  isLoading,
  records,
  page,
  totalPages,
  onPageChange,
  onAddClick,
  onEdit,
  onDelete,
}: CalibrationTabContentProps) {
  const [searchKeyword, setSearchKeyword] = useState('')

  const filteredEquip = equipmentList.filter((e) =>
    e.name.toLowerCase().includes(searchKeyword.toLowerCase())
  )

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>選擇設備查看校正紀錄</CardTitle>
          <CardDescription>選擇設備以查看其校正紀錄與下次校正日期</CardDescription>
        </div>
        {canManage && (
          <Button onClick={onAddClick} disabled={equipmentList.length === 0}>
            <Plus className="h-4 w-4 mr-2" />
            新增校正紀錄
          </Button>
        )}
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="relative">
          <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="搜尋設備名稱..."
            value={searchKeyword}
            onChange={(e) => setSearchKeyword(e.target.value)}
            className="pl-8"
          />
        </div>
        <div className="flex flex-wrap gap-2 max-h-32 overflow-y-auto">
          <Button
            variant={!calibEquipmentFilter ? 'default' : 'outline'}
            size="sm"
            onClick={() => onFilterChange('')}
            className="justify-start"
          >
            <Ruler className="h-4 w-4 mr-2" />
            全部設備
          </Button>
          {filteredEquip.map((e) => (
            <Button
              key={e.id}
              variant={calibEquipmentFilter === e.id ? 'default' : 'outline'}
              size="sm"
              onClick={() => onFilterChange(e.id)}
              className="justify-start"
            >
              <Wrench className="h-4 w-4 mr-2" />
              {e.name}
            </Button>
          ))}
        </div>
        <div className="rounded-md border">
          {isLoading ? (
            <div className="flex justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : records.length === 0 ? (
            <div className="py-12 text-center text-muted-foreground">尚無校正紀錄</div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>設備</TableHead>
                  <TableHead>校正日期</TableHead>
                  <TableHead>下次校正</TableHead>
                  <TableHead>結果</TableHead>
                  <TableHead>備註</TableHead>
                  {canManage && <TableHead className="w-[100px]">操作</TableHead>}
                </TableRow>
              </TableHeader>
              <TableBody>
                {records.map((r) => (
                  <TableRow key={r.id}>
                    <TableCell className="font-medium">{r.equipment_name}</TableCell>
                    <TableCell>
                      {format(new Date(r.calibrated_at), 'yyyy/MM/dd', { locale: zhTW })}
                    </TableCell>
                    <TableCell>
                      {r.next_due_at
                        ? format(new Date(r.next_due_at), 'yyyy/MM/dd', { locale: zhTW })
                        : '—'}
                    </TableCell>
                    <TableCell>{r.result || '—'}</TableCell>
                    <TableCell className="max-w-[200px] truncate">{r.notes || '—'}</TableCell>
                    {canManage && (
                      <TableCell>
                        <div className="flex gap-2">
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => onEdit(r)}
                          >
                            <Pencil className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="text-destructive hover:text-destructive"
                            onClick={() => onDelete(r.id)}
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </div>
                      </TableCell>
                    )}
                  </TableRow>
                ))}
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
  )
}
