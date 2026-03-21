import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import type { TrainingRecordWithUser } from '../types/training'

interface TrainingExpiringTabProps {
  records: TrainingRecordWithUser[]
}

export function TrainingExpiringTab({ records }: TrainingExpiringTabProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>證照即將到期</CardTitle>
        <CardDescription>列出 30 天內到期之證照，請盡快安排複訓或更新</CardDescription>
      </CardHeader>
      <CardContent>
        {records.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">目前無即將到期之證照</div>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>人員</TableHead>
                <TableHead>課程名稱</TableHead>
                <TableHead>完成日期</TableHead>
                <TableHead className="text-orange-500">到期日</TableHead>
                <TableHead>備註</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {records.map((r) => (
                <TableRow key={r.id}>
                  <TableCell>
                    <div className="font-medium">{r.user_name || r.user_email}</div>
                    <div className="text-xs text-muted-foreground">{r.user_email}</div>
                  </TableCell>
                  <TableCell>{r.course_name}</TableCell>
                  <TableCell>
                    {format(new Date(r.completed_at), 'yyyy/MM/dd', { locale: zhTW })}
                  </TableCell>
                  <TableCell className="font-bold text-orange-500">
                    {r.expires_at
                      ? format(new Date(r.expires_at), 'yyyy/MM/dd', { locale: zhTW })
                      : '\u2014'}
                  </TableCell>
                  <TableCell className="max-w-[200px] truncate">{r.notes || '\u2014'}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </CardContent>
    </Card>
  )
}
