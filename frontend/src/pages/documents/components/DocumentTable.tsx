import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { StatusBadge } from '@/components/ui/status-badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { Eye, Edit, Trash2, FileText } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { formatDate, formatCurrency } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'
import { cn } from '@/lib/utils'
import type { DocumentListItem, DocType } from '@/lib/api'

const docTypeNames: Record<DocType, string> = {
  PO: '採購單',
  GRN: '採購入庫',
  PR: '採購退貨',
  SO: '銷貨單',
  DO: '銷貨出庫',
  SR: '銷貨退貨',
  RTN: '退貨單',
  TR: '調撥單',
  STK: '盤點單',
  ADJ: '調整單',
  RM: '退料單',
}

const statusNames: Record<string, string> = {
  draft: '草稿',
  submitted: '待核准',
  approved: '已核准',
  cancelled: '已作廢',
}

const receiptStatusConfig: Record<string, { label: string; variant: 'warning' | 'info' | 'success' }> = {
  pending: { label: '未入庫', variant: 'warning' },
  partial: { label: '部分入庫', variant: 'info' },
  complete: { label: '已入庫', variant: 'success' },
}

const ACCOUNTING_DOC_TYPES: DocType[] = ['GRN', 'DO', 'PR']

interface DocumentTableProps {
  documents: DocumentListItem[] | undefined
  isLoading: boolean
  onDeleteClick: (doc: DocumentListItem) => void
}

function getStatusBadge(doc: DocumentListItem) {
  const badges = []

  switch (doc.status) {
    case 'draft':
      badges.push(<Badge key="base" variant="secondary">{statusNames[doc.status]}</Badge>)
      break
    case 'submitted':
      badges.push(<Badge key="base" variant="warning">{statusNames[doc.status]}</Badge>)
      break
    case 'approved':
      badges.push(<Badge key="base" variant="success">{statusNames[doc.status]}</Badge>)
      break
    case 'cancelled':
      badges.push(<Badge key="base" variant="destructive">{statusNames[doc.status]}</Badge>)
      break
    default:
      badges.push(<Badge key="base" variant="outline">{doc.status}</Badge>)
  }

  if (ACCOUNTING_DOC_TYPES.includes(doc.doc_type) && doc.has_journal_entry) {
    badges.push(
      <Badge key="journal" variant="outline" className="ml-1 text-xs border-status-success-text/30 text-status-success-text">
        已過帳
      </Badge>
    )
  }

  return <div className="flex flex-wrap items-center gap-1">{badges}</div>
}

function getReceiptStatusBadge(doc: DocumentListItem) {
  if (doc.doc_type !== 'PO') return <span className="text-muted-foreground">-</span>
  if (doc.status !== 'approved') return <span className="text-muted-foreground">-</span>
  const cfg = doc.receipt_status ? receiptStatusConfig[doc.receipt_status] : null
  if (!cfg) return <span className="text-muted-foreground">-</span>
  return <StatusBadge variant={cfg.variant} dot>{cfg.label}</StatusBadge>
}

export function DocumentTable({ documents, isLoading, onDeleteClick }: DocumentTableProps) {
  const { sortedData, sort, toggleSort } = useTableSort(documents)

  const renderActions = (doc: DocumentListItem) => (
    <>
      <Button variant="ghost" size="icon" asChild title="檢視" aria-label="檢視">
        <Link to={`/documents/${doc.id}`}>
          <Eye className="h-4 w-4" />
        </Link>
      </Button>
      {doc.status === 'draft' && (
        <>
          <Button variant="ghost" size="icon" asChild title="編輯" aria-label="編輯">
            <Link to={`/documents/${doc.id}/edit`}>
              <Edit className="h-4 w-4" />
            </Link>
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => onDeleteClick(doc)}
            title="刪除"
            aria-label="刪除"
            className="text-destructive hover:text-destructive"
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </>
      )}
    </>
  )

  return (
    <div className="rounded-lg border bg-card overflow-hidden @container">
      <div className="hidden @[600px]:block overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <SortableTableHead sortKey="doc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>單號</SortableTableHead>
              <SortableTableHead sortKey="doc_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>類型</SortableTableHead>
              <SortableTableHead sortKey="status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>狀態</SortableTableHead>
              <SortableTableHead className="hidden @[900px]:table-cell" sortKey="receipt_status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>入庫進度</SortableTableHead>
              <SortableTableHead className="hidden @[750px]:table-cell" sortKey="partner_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>對象</SortableTableHead>
              <SortableTableHead className="hidden @[900px]:table-cell" sortKey="warehouse_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>倉庫</SortableTableHead>
              <SortableTableHead sortKey="doc_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>單據日期</SortableTableHead>
              <SortableTableHead className="hidden @[750px]:table-cell text-right" sortKey="total_amount" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>金額</SortableTableHead>
              <SortableTableHead className="hidden @[900px]:table-cell" sortKey="created_by_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>建立人</SortableTableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={10} className="p-0">
                  <TableSkeleton rows={8} cols={10} />
                </TableCell>
              </TableRow>
            ) : sortedData && sortedData.length > 0 ? (
              sortedData.map((doc) => (
                <TableRow key={doc.id} className={cn(doc.status === 'cancelled' && 'bg-destructive/5')}>
                  <TableCell className="font-mono font-medium">{doc.doc_no}</TableCell>
                  <TableCell>{docTypeNames[doc.doc_type]}</TableCell>
                  <TableCell>{getStatusBadge(doc)}</TableCell>
                  <TableCell className="hidden @[900px]:table-cell">{getReceiptStatusBadge(doc)}</TableCell>
                  <TableCell className="hidden @[750px]:table-cell">{doc.partner_name || '-'}</TableCell>
                  <TableCell className="hidden @[900px]:table-cell">{doc.warehouse_name || '-'}</TableCell>
                  <TableCell>{formatDate(doc.doc_date)}</TableCell>
                  <TableCell className="hidden @[750px]:table-cell text-right">
                    {doc.total_amount ? formatCurrency(doc.total_amount) : '-'}
                  </TableCell>
                  <TableCell className="hidden @[900px]:table-cell">{doc.created_by_name}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-1">
                      {renderActions(doc)}
                    </div>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={10} icon={FileText} title="尚無單據資料" />
            )}
          </TableBody>
        </Table>
      </div>

      <div className="@[600px]:hidden divide-y">
        {isLoading ? (
          <div className="p-3"><TableSkeleton rows={3} cols={1} /></div>
        ) : sortedData && sortedData.length > 0 ? (
          sortedData.map((doc) => (
            <div key={doc.id} className={cn('p-3 space-y-2', doc.status === 'cancelled' && 'bg-destructive/5')}>
              <div className="flex items-start justify-between gap-2">
                <div className="flex-1 min-w-0">
                  <div className="font-mono font-medium text-sm">{doc.doc_no}</div>
                  <div className="text-xs text-muted-foreground">
                    {docTypeNames[doc.doc_type]} · {formatDate(doc.doc_date)}
                  </div>
                </div>
                {getStatusBadge(doc)}
              </div>
              <div className="text-xs text-muted-foreground space-y-0.5">
                {doc.partner_name && <div>對象：{doc.partner_name}</div>}
                {doc.warehouse_name && <div>倉庫：{doc.warehouse_name}</div>}
                {doc.total_amount && <div>金額：{formatCurrency(doc.total_amount)}</div>}
                {doc.created_by_name && <div>建立：{doc.created_by_name}</div>}
                {doc.doc_type === 'PO' && doc.status === 'approved' && doc.receipt_status && (
                  <div>入庫進度：{receiptStatusConfig[doc.receipt_status]?.label || doc.receipt_status}</div>
                )}
              </div>
              <div className="flex justify-end gap-1 pt-1 border-t">
                {renderActions(doc)}
              </div>
            </div>
          ))
        ) : (
          <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
            <FileText className="h-8 w-8" />
            <p className="text-sm">尚無單據資料</p>
          </div>
        )}
      </div>
    </div>
  )
}
