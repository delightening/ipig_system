import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import type { TrainingRecordWithUser } from '../types/training'
import { CheckCircle } from 'lucide-react'

interface TrainingExpiringTabProps {
  records: TrainingRecordWithUser[]
}

const columns: ColumnDef<TrainingRecordWithUser>[] = [
  {
    key: 'user', header: '人員',
    cell: (r) => (
      <div>
        <div className="font-medium">{r.user_name || r.user_email}</div>
        <div className="text-xs text-muted-foreground">{r.user_email}</div>
      </div>
    ),
  },
  { key: 'course', header: '課程名稱', cell: (r) => r.course_name },
  {
    key: 'completed', header: '完成日期',
    cell: (r) => format(new Date(r.completed_at), 'yyyy/MM/dd', { locale: zhTW }),
  },
  {
    key: 'expires', header: '到期日', className: 'text-status-warning-text',
    cell: (r) => (
      <span className="font-bold text-status-warning-text">
        {r.expires_at ? format(new Date(r.expires_at), 'yyyy/MM/dd', { locale: zhTW }) : '\u2014'}
      </span>
    ),
  },
  {
    key: 'notes', header: '備註', className: 'max-w-[200px]',
    cell: (r) => <span className="truncate block">{r.notes || '\u2014'}</span>,
  },
]

export function TrainingExpiringTab({ records }: TrainingExpiringTabProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>證照即將到期</CardTitle>
        <CardDescription>列出 30 天內到期之證照，請盡快安排複訓或更新</CardDescription>
      </CardHeader>
      <CardContent>
        <DataTable
          columns={columns}
          data={records}
          emptyIcon={CheckCircle}
          emptyTitle="目前無即將到期之證照"
          rowKey={(r) => r.id}
        />
      </CardContent>
    </Card>
  )
}
