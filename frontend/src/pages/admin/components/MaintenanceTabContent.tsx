/**
 * 維修/保養紀錄分頁內容：表格、分頁、新增/編輯/刪除
 */
import { useMemo } from 'react'
import { truncateText } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { StatusBadge } from '@/components/ui/status-badge'
import type { StatusVariant } from '@/components/ui/status-badge'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { History, Pencil, Plus, Trash2, Wrench } from 'lucide-react'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'

import type { MaintenanceRecordWithDetails } from '../types'
import { MAINTENANCE_TYPE_LABELS, MAINTENANCE_STATUS_LABELS } from '../types'

interface MaintenanceTabContentProps {
  canManage: boolean
  records: MaintenanceRecordWithDetails[]
  isLoading: boolean
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  onDelete: (id: string) => void
  onAdd?: () => void
  onEdit?: (record: MaintenanceRecordWithDetails) => void
  onViewHistory?: (id: string) => void
}

const TYPE_VARIANT: Record<string, StatusVariant> = {
  repair: 'error',
  maintenance: 'info',
}

const STATUS_VARIANT: Record<string, StatusVariant> = {
  pending: 'neutral',
  in_progress: 'warning',
  completed: 'success',
  unrepairable: 'error',
}


export function MaintenanceTabContent({
  canManage,
  records,
  isLoading,
  page,
  totalPages,
  onPageChange,
  onDelete,
  onAdd,
  onEdit,
  onViewHistory,
}: MaintenanceTabContentProps) {
  const columns = useMemo<ColumnDef<MaintenanceRecordWithDetails>[]>(() => {
    const cols: ColumnDef<MaintenanceRecordWithDetails>[] = [
      { key: 'equipment', header: '設備', cell: (r) => <span className="font-medium">{r.equipment_name}</span> },
      {
        key: 'type', header: '類型',
        cell: (r) => (
          <StatusBadge variant={TYPE_VARIANT[r.maintenance_type] || 'neutral'}>
            {MAINTENANCE_TYPE_LABELS[r.maintenance_type]}
          </StatusBadge>
        ),
      },
      {
        key: 'status', header: '狀態',
        cell: (r) => (
          <StatusBadge variant={STATUS_VARIANT[r.status] || 'neutral'}>
            {MAINTENANCE_STATUS_LABELS[r.status]}
          </StatusBadge>
        ),
      },
      {
        key: 'reported', header: '報修日期',
        cell: (r) => format(new Date(r.reported_at), 'yyyy/MM/dd', { locale: zhTW }),
      },
      {
        key: 'completed', header: '完修日期',
        cell: (r) => r.completed_at ? format(new Date(r.completed_at), 'yyyy/MM/dd', { locale: zhTW }) : '—',
      },
      {
        key: 'description', header: '問題描述/保養項目', className: 'max-w-[200px]',
        cell: (r) => r.maintenance_type === 'repair' ? truncateText(r.problem_description, 30) : truncateText(r.maintenance_items, 30),
      },
    ]
    if (canManage || onViewHistory) {
      cols.push({
        key: 'actions', header: '操作', className: 'w-[120px] text-right',
        cell: (r) => (
          <div className="flex items-center justify-end gap-1">
            {onViewHistory && (
              <Button variant="ghost" size="icon" onClick={() => onViewHistory(r.id)} aria-label="歷史">
                <History className="h-4 w-4" />
              </Button>
            )}
            {canManage && onEdit && (
              <Button variant="ghost" size="icon" onClick={() => onEdit(r)} aria-label="編輯">
                <Pencil className="h-4 w-4" />
              </Button>
            )}
            {canManage && (
              <Button variant="ghost" size="icon" className="text-destructive hover:text-destructive" onClick={() => onDelete(r.id)} aria-label="刪除">
                <Trash2 className="h-4 w-4" />
              </Button>
            )}
          </div>
        ),
      })
    }
    return cols
  }, [canManage, onDelete, onEdit, onViewHistory])

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>維修/保養紀錄</CardTitle>
            <CardDescription>查看設備維修與定期保養紀錄</CardDescription>
          </div>
          {canManage && onAdd && (
            <Button size="sm" onClick={onAdd}>
              <Plus className="h-4 w-4 mr-2" />
              新增維修/保養
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <DataTable
          columns={columns}
          data={records}
          isLoading={isLoading}
          emptyIcon={Wrench}
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
