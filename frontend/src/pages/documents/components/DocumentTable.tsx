import { Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
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
import { useAuthIsAdmin } from '@/stores/auth'
import { cn } from '@/lib/utils'
import { PendingOwnerBadge, PendingOwnerInline } from '@/components/PendingOwnerBadge'
import type { TFunction } from 'i18next'
import type { DocumentListItem, DocType } from '@/lib/api'
import { DOC_STATUS_NAMES, DOC_TYPE_NAMES } from '../types'

const docTypeNames = DOC_TYPE_NAMES
const statusNames = DOC_STATUS_NAMES

// label 存 i18n 鍵，渲染時才 t()（語言切換才會即時更新）
const receiptStatusConfig: Record<string, { labelKey: string; variant: 'warning' | 'info' | 'success' }> = {
  pending: { labelKey: 'erpDocs.documents.table.receipt.pending', variant: 'warning' },
  partial: { labelKey: 'erpDocs.documents.table.receipt.partial', variant: 'info' },
  complete: { labelKey: 'erpDocs.documents.table.receipt.complete', variant: 'success' },
}

// SO 一段式（#1004）核准即過帳 → 併入「已過帳」badge 判斷
const ACCOUNTING_DOC_TYPES: DocType[] = ['GRN', 'PR', 'SO']

interface DocumentTableProps {
  documents: DocumentListItem[] | undefined
  isLoading: boolean
  onDeleteClick: (doc: DocumentListItem) => void
}

function getStatusBadge(doc: DocumentListItem, t: TFunction) {
  const badges = []

  switch (doc.status) {
    case 'draft':
      badges.push(<Badge key="base" variant="secondary">{statusNames[doc.status]}</Badge>)
      break
    case 'submitted':
      // hover 顯示卡在誰手上（後端 pending_owner，僅 submitted 有值）。
      badges.push(
        <PendingOwnerBadge key="base" owner={doc.pending_owner}>
          <Badge variant="warning">{statusNames[doc.status]}</Badge>
        </PendingOwnerBadge>
      )
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
        {t('erpDocs.documents.table.posted')}
      </Badge>
    )
  }

  return <div className="flex flex-wrap items-center gap-1">{badges}</div>
}

function getReceiptStatusBadge(doc: DocumentListItem, t: TFunction) {
  if (doc.doc_type !== 'PO') return <span className="text-muted-foreground">-</span>
  if (doc.status !== 'approved') return <span className="text-muted-foreground">-</span>
  const cfg = doc.receipt_status ? receiptStatusConfig[doc.receipt_status] : null
  if (!cfg) return <span className="text-muted-foreground">-</span>
  return <StatusBadge variant={cfg.variant} dot>{t(cfg.labelKey)}</StatusBadge>
}

export function DocumentTable({ documents, isLoading, onDeleteClick }: DocumentTableProps) {
  const { t } = useTranslation()
  const { sortedData, sort, toggleSort } = useTableSort(documents)
  const isAdmin = useAuthIsAdmin()

  // 草稿：可編輯 + 刪除（軟刪，一般權限）。
  // 非草稿：僅 admin 顯示刪除鈕（硬刪，用於清理早期系統未完善時建立的殘單）；
  // 後端連動刪除下游草稿、擋下已核准下游（見 DocumentService::delete）。
  const renderActions = (doc: DocumentListItem) => {
    const canDelete = doc.status === 'draft' || isAdmin
    return (
      <>
        <Button variant="ghost" size="icon" asChild title={t('common.view')} aria-label={t('common.view')}>
          <Link to={`/documents/${doc.id}`}>
            <Eye className="h-4 w-4" />
          </Link>
        </Button>
        {doc.status === 'draft' && (
          <Button variant="ghost" size="icon" asChild title={t('common.edit')} aria-label={t('common.edit')}>
            <Link to={`/documents/${doc.id}/edit`}>
              <Edit className="h-4 w-4" />
            </Link>
          </Button>
        )}
        {canDelete && (
          <Button
            variant="ghost"
            size="icon"
            onClick={() => onDeleteClick(doc)}
            title={t('common.delete')}
            aria-label={t('common.delete')}
            className="text-destructive hover:text-destructive"
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        )}
      </>
    )
  }

  return (
    <div className="rounded-lg border bg-card overflow-hidden @container">
      <div className="hidden @[600px]:block overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <SortableTableHead sortKey="doc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.documents.table.docNo')}</SortableTableHead>
              <SortableTableHead sortKey="doc_type" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.type')}</SortableTableHead>
              <SortableTableHead sortKey="status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.status')}</SortableTableHead>
              <SortableTableHead className="hidden @[900px]:table-cell" sortKey="receipt_status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.documents.table.receiptProgress')}</SortableTableHead>
              <SortableTableHead className="hidden @[750px]:table-cell" sortKey="partner_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.partner')}</SortableTableHead>
              <SortableTableHead className="hidden @[900px]:table-cell" sortKey="warehouse_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.warehouse')}</SortableTableHead>
              <SortableTableHead sortKey="doc_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.docDate')}</SortableTableHead>
              <SortableTableHead className="hidden @[750px]:table-cell text-right" sortKey="total_amount" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.amount')}</SortableTableHead>
              <SortableTableHead className="hidden @[900px]:table-cell" sortKey="created_by_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpDocs.shared.createdBy')}</SortableTableHead>
              <TableHead className="text-right">{t('common.actions')}</TableHead>
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
                  <TableCell>{getStatusBadge(doc, t)}</TableCell>
                  <TableCell className="hidden @[900px]:table-cell">{getReceiptStatusBadge(doc, t)}</TableCell>
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
              <TableEmptyRow colSpan={10} icon={FileText} title={t('erpDocs.documents.table.empty')} />
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
                {getStatusBadge(doc, t)}
              </div>
              <div className="text-xs text-muted-foreground space-y-0.5">
                {/* 手機沒有 hover，「卡在誰」直接寫在卡片上 */}
                <PendingOwnerInline owner={doc.pending_owner} />
                {doc.partner_name && <div>{t('erpDocs.documents.table.partnerLine', { name: doc.partner_name })}</div>}
                {doc.warehouse_name && <div>{t('erpDocs.documents.table.warehouseLine', { name: doc.warehouse_name })}</div>}
                {doc.total_amount && <div>{t('erpDocs.documents.table.amountLine', { amount: formatCurrency(doc.total_amount) })}</div>}
                {doc.created_by_name && <div>{t('erpDocs.documents.table.createdByLine', { name: doc.created_by_name })}</div>}
                {doc.doc_type === 'PO' && doc.status === 'approved' && doc.receipt_status && (
                  <div>
                    {t('erpDocs.documents.table.receiptLine', {
                      status: receiptStatusConfig[doc.receipt_status]
                        ? t(receiptStatusConfig[doc.receipt_status].labelKey)
                        : doc.receipt_status,
                    })}
                  </div>
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
            <p className="text-sm">{t('erpDocs.documents.table.empty')}</p>
          </div>
        )}
      </div>
    </div>
  )
}
