import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { formatNumber } from '@/lib/utils'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Loader2 } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import type { JournalEntryResponse } from '@/types/accounting'

function formatDate(d: string) {
  return new Date(d).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })
}

interface JournalEntriesTabProps {
  dateFrom: string
  dateTo: string
  onDateFromChange: (date: string) => void
  onDateToChange: (date: string) => void
}

export function JournalEntriesTab({
  dateFrom,
  dateTo,
  onDateFromChange,
  onDateToChange,
}: JournalEntriesTabProps) {
  const { data: journalEntries, isLoading } = useQuery<JournalEntryResponse[]>({
    queryKey: ['accounting-journal-entries', dateFrom, dateTo],
    queryFn: async () => {
      const r = await api.get<JournalEntryResponse[]>('/accounting/journal-entries', {
        params: { date_from: dateFrom, date_to: dateTo, limit: 100 },
      })
      return r.data
    },
  })

  if (isLoading) {
    return (
      <div className="flex justify-center py-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex items-end gap-4 flex-wrap">
        <div className="space-y-2">
          <Label>日期起</Label>
          <Input
            type="date"
            value={dateFrom}
            onChange={(e) => onDateFromChange(e.target.value)}
            className="w-40"
          />
        </div>
        <div className="space-y-2">
          <Label>日期訖</Label>
          <Input
            type="date"
            value={dateTo}
            onChange={(e) => onDateToChange(e.target.value)}
            className="w-40"
          />
        </div>
      </div>
      <div className="space-y-4">
        {journalEntries && journalEntries.length > 0 ? (
          journalEntries.map(({ entry, lines }) => (
            <div key={entry.id} className="rounded-md border p-4 space-y-3">
              <div className="flex items-center justify-between">
                <div>
                  <span className="font-mono font-medium">{entry.entry_no}</span>
                  <span className="mx-2 text-muted-foreground">|</span>
                  <span>{formatDate(entry.entry_date)}</span>
                  {entry.description && (
                    <span className="ml-2 text-muted-foreground">{entry.description}</span>
                  )}
                </div>
              </div>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-16">行號</TableHead>
                    <TableHead>科目</TableHead>
                    <TableHead>說明</TableHead>
                    <TableHead className="text-right">借方</TableHead>
                    <TableHead className="text-right">貸方</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {lines.map((l) => (
                    <TableRow key={l.line_id}>
                      <TableCell>{l.line_no}</TableCell>
                      <TableCell className="font-mono">
                        {l.account_code} {l.account_name}
                      </TableCell>
                      <TableCell>{l.description || '-'}</TableCell>
                      <TableCell className="text-right">
                        {Number(l.debit_amount) > 0 ? formatNumber(Number(l.debit_amount), 2) : '-'}
                      </TableCell>
                      <TableCell className="text-right">
                        {Number(l.credit_amount) > 0 ? formatNumber(Number(l.credit_amount), 2) : '-'}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          ))
        ) : (
          <div className="rounded-md border py-12 text-center text-muted-foreground">
            尚無傳票資料
          </div>
        )}
      </div>
    </div>
  )
}
