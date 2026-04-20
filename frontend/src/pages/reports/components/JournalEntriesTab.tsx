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
import { BookOpen } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Input } from '@/components/ui/input'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
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

  const { sortedData: sortedEntries, sort, toggleSort } = useTableSort(journalEntries)

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
      {isLoading ? (
        <div className="rounded-lg border bg-card overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                <SortableTableHead sortKey="entry.entry_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>傳票號</SortableTableHead>
                <SortableTableHead sortKey="entry.entry_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>日期</SortableTableHead>
                <SortableTableHead sortKey="entry.description" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>說明</SortableTableHead>
                <TableHead className="text-right">合計借方</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow>
                <TableCell colSpan={4} className="p-0">
                  <TableSkeleton rows={8} cols={4} />
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </div>
      ) : (sortedEntries ?? journalEntries) && (sortedEntries ?? journalEntries)!.length > 0 ? (
        <div className="space-y-4">
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                <SortableTableHead sortKey="entry.entry_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>傳票號</SortableTableHead>
                <SortableTableHead sortKey="entry.entry_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>日期</SortableTableHead>
                <SortableTableHead sortKey="entry.description" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>說明</SortableTableHead>
                <TableHead className="text-right">合計借方</TableHead>
              </TableRow>
            </TableHeader>
          </Table>
          {(sortedEntries ?? journalEntries)!.map(({ entry, lines }) => (
            <div key={entry.id} className="rounded-lg border bg-card overflow-hidden p-4 space-y-3">
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
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
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
          ))}
        </div>
      ) : (
        <div className="rounded-lg border bg-card overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                <SortableTableHead sortKey="entry.entry_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>傳票號</SortableTableHead>
                <SortableTableHead sortKey="entry.entry_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>日期</SortableTableHead>
                <SortableTableHead sortKey="entry.description" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>說明</SortableTableHead>
                <TableHead className="text-right">合計借方</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableEmptyRow colSpan={4} icon={BookOpen} title="尚無傳票資料" />
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  )
}
