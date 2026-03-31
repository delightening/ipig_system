/**
 * 校正/確效/查核紀錄分頁內容：設備篩選、表格、分頁
 */
import { useMemo } from 'react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { StatusBadge } from '@/components/ui/status-badge'
import type { StatusVariant } from '@/components/ui/status-badge'
import { SearchableSelect } from '@/components/ui/searchable-select'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { Plus, Pencil, Trash2, Wrench, Ruler } from 'lucide-react'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'

import type { Equipment, CalibrationWithEquipment } from '../types'
import { CALIBRATION_TYPE_LABELS } from '../types'

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

const TYPE_VARIANT: Record<string, StatusVariant> = {
  calibration: 'info',
  validation: 'purple',
  inspection: 'warning',
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
  const equipmentOptions = useMemo(
    () =>
      equipmentList.map((e) => ({
        value: e.id,
        label: e.name,
        description: e.serial_number || undefined,
      })),
    [equipmentList],
  )

  const columns = useMemo<ColumnDef<CalibrationWithEquipment>[]>(() => {
    const cols: ColumnDef<CalibrationWithEquipment>[] = [
      { key: 'equipment', header: '設備', cell: (r) => <span className="font-medium">{r.equipment_name}</span> },
      { key: 'serial', header: '序號', cell: (r) => r.equipment_serial_number || '—' },
      {
        key: 'type', header: '類型',
        cell: (r) => (
          <StatusBadge variant={TYPE_VARIANT[r.calibration_type] || 'neutral'}>
            {CALIBRATION_TYPE_LABELS[r.calibration_type]}
          </StatusBadge>
        ),
      },
      {
        key: 'date', header: '執行日期',
        cell: (r) => format(new Date(r.calibrated_at), 'yyyy/MM/dd', { locale: zhTW }),
      },
      {
        key: 'nextDue', header: '下次到期',
        cell: (r) => {
          if (!r.next_due_at) return '—'
          const overdue = new Date(r.next_due_at) < new Date()
          return (
            <span className={overdue ? 'text-destructive font-semibold' : ''}>
              {format(new Date(r.next_due_at), 'yyyy/MM/dd', { locale: zhTW })}
              {overdue && ' (逾期)'}
            </span>
          )
        },
      },
      { key: 'result', header: '結果', cell: (r) => r.result || '—' },
      {
        key: 'report', header: '報告/人員',
        cell: (r) => r.calibration_type === 'inspection' ? (r.inspector || '—') : (r.report_number || '—'),
      },
    ]
    if (canManage) {
      cols.push({
        key: 'actions', header: '操作', className: 'w-[100px] text-right',
        cell: (r) => (
          <div className="flex items-center justify-end gap-1">
            <Button variant="ghost" size="icon" onClick={() => onEdit(r)} aria-label="編輯">
              <Pencil className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" className="text-destructive hover:text-destructive" onClick={() => onDelete(r.id)} aria-label="刪除">
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        ),
      })
    }
    return cols
  }, [canManage, onEdit, onDelete])

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>校正/確效/查核紀錄</CardTitle>
          <CardDescription>選擇設備以查看其校正、確效或查核紀錄</CardDescription>
        </div>
        {canManage && (
          <Button onClick={onAddClick} disabled={equipmentList.length === 0}>
            <Plus className="h-4 w-4 mr-2" />
            新增紀錄
          </Button>
        )}
      </CardHeader>
      <CardContent className="space-y-4">
        <SearchableSelect
          options={equipmentOptions}
          value={calibEquipmentFilter}
          onValueChange={onFilterChange}
          placeholder="全部設備"
          searchPlaceholder="搜尋設備名稱或序號..."
          emptyMessage="找不到符合的設備"
          icon={Wrench}
          className="w-72"
        />
        <DataTable
          columns={columns}
          data={records}
          isLoading={isLoading}
          emptyIcon={Ruler}
          emptyTitle="尚無紀錄"
          rowKey={(r) => r.id}
          page={page}
          totalPages={totalPages}
          onPageChange={onPageChange}
        />
      </CardContent>
    </Card>
  )
}
