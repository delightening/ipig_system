import { Partner } from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { useTableSort } from '@/hooks/useTableSort'
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
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { Edit, Trash2, Users } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'

interface PartnerTableProps {
  partners: Partner[] | undefined
  isLoading: boolean
  onEdit: (partner: Partner) => void
  onDelete: (partner: Partner, hard: boolean) => void
  confirm: (opts: {
    title: string
    description: string
    variant?: 'default' | 'destructive'
    confirmLabel?: string
  }) => Promise<boolean>
}

export function PartnerTable({
  partners,
  isLoading,
  onEdit,
  onDelete,
  confirm,
}: PartnerTableProps) {
  const { sortedData, sort, toggleSort } = useTableSort(partners)

  const handleDeleteClick = async (partner: Partner) => {
    const isAdmin = useAuthStore.getState().user?.roles.includes('admin')
    const ok = await confirm({
      title: isAdmin ? '管理員權限：永久刪除夥伴' : '刪除夥伴',
      description: isAdmin
        ? '警告：具有管理員權限，此操作將永久從資料庫中移除資料，且無法復原。確定要執行硬刪除嗎？'
        : '確定要刪除此夥伴嗎？',
      variant: 'destructive',
      confirmLabel: isAdmin ? '執行硬刪除' : '確認刪除',
    })
    if (ok) {
      onDelete(partner, !!isAdmin)
    }
  }

  return (
    <div className="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <SortableTableHead sortKey="partner_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>類型</SortableTableHead>
            <SortableTableHead sortKey="code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>代碼</SortableTableHead>
            <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>名稱</SortableTableHead>
            <SortableTableHead sortKey="tax_id" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>統編</SortableTableHead>
            <SortableTableHead sortKey="phone" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>電話</SortableTableHead>
            <SortableTableHead sortKey="is_active" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>狀態</SortableTableHead>
            <TableHead className="text-right">操作</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow>
              <TableCell colSpan={7} className="p-0">
                <TableSkeleton rows={8} cols={5} />
              </TableCell>
            </TableRow>
          ) : sortedData && sortedData.length > 0 ? (
            sortedData.map((partner) => (
              <PartnerRow
                key={partner.id}
                partner={partner}
                onEdit={onEdit}
                onDelete={handleDeleteClick}
              />
            ))
          ) : (
            <TableEmptyRow colSpan={7} icon={Users} title="尚無夥伴資料" />
          )}
        </TableBody>
      </Table>
    </div>
  )
}

function PartnerRow({
  partner,
  onEdit,
  onDelete,
}: {
  partner: Partner
  onEdit: (p: Partner) => void
  onDelete: (p: Partner) => void
}) {
  return (
    <TableRow>
      <TableCell>
        <Badge variant={partner.partner_type === 'supplier' ? 'default' : 'secondary'}>
          {partner.partner_type === 'supplier' ? '供應商' : '客戶'}
        </Badge>
      </TableCell>
      <TableCell className="font-mono">{partner.code}</TableCell>
      <TableCell className="font-medium">{partner.name}</TableCell>
      <TableCell>{partner.tax_id || '-'}</TableCell>
      <TableCell>
        {partner.phone || '-'}
        {partner.phone_ext ? ` #${partner.phone_ext}` : ''}
      </TableCell>
      <TableCell>
        {partner.is_active ? (
          <Badge variant="success">啟用</Badge>
        ) : (
          <Badge variant="destructive">停用</Badge>
        )}
      </TableCell>
      <TableCell className="text-right">
        <Button variant="ghost" size="icon" onClick={() => onEdit(partner)} aria-label="編輯">
          <Edit className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon" onClick={() => onDelete(partner)} aria-label="刪除">
          <Trash2 className="h-4 w-4" />
        </Button>
      </TableCell>
    </TableRow>
  )
}
