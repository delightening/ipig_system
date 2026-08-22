import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { DataTable, type ColumnDef } from '@/components/ui/data-table'
import { format } from 'date-fns'
import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { getDateFnsLocale } from '@/lib/utils'
import type { TrainingRecordWithUser } from '../types/training'
import { CheckCircle } from 'lucide-react'

interface TrainingExpiringTabProps {
  records: TrainingRecordWithUser[]
}

export function TrainingExpiringTab({ records }: TrainingExpiringTabProps) {
  const { t } = useTranslation()

  const columns: ColumnDef<TrainingRecordWithUser>[] = useMemo(
    () => [
      {
        key: 'user', header: t('admin.trainingExpiringTab.columnUser'),
        cell: (r) => (
          <div>
            <div className="font-medium">{r.user_name || r.user_email}</div>
            <div className="text-xs text-muted-foreground">{r.user_email}</div>
          </div>
        ),
      },
      { key: 'course', header: t('admin.trainingExpiringTab.columnCourse'), cell: (r) => r.course_name },
      {
        key: 'completed', header: t('admin.trainingExpiringTab.columnCompleted'),
        cell: (r) => format(new Date(r.completed_at), 'yyyy/MM/dd', { locale: getDateFnsLocale() }),
      },
      {
        key: 'expires', header: t('admin.trainingExpiringTab.columnExpires'), className: 'text-status-warning-text',
        cell: (r) => (
          <span className="font-bold text-status-warning-text">
            {r.expires_at ? format(new Date(r.expires_at), 'yyyy/MM/dd', { locale: getDateFnsLocale() }) : '—'}
          </span>
        ),
      },
      {
        key: 'notes', header: t('admin.trainingExpiringTab.columnNotes'), className: 'max-w-[200px] whitespace-normal break-words',
        cell: (r) => <span className="block">{r.notes || '—'}</span>,
      },
    ],
    [t],
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('admin.trainingExpiringTab.title')}</CardTitle>
        <CardDescription>{t('admin.trainingExpiringTab.description')}</CardDescription>
      </CardHeader>
      <CardContent>
        <DataTable
          columns={columns}
          data={records}
          emptyIcon={CheckCircle}
          emptyTitle={t('admin.trainingExpiringTab.emptyTitle')}
          rowKey={(r) => r.id}
        />
      </CardContent>
    </Card>
  )
}
