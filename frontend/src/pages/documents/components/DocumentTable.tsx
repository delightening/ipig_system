import { Link } from 'react-router-dom'
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
import { Eye, Edit, Trash2, Loader2, FileText } from 'lucide-react'
import { formatDate, formatCurrency } from '@/lib/utils'
import type { DocumentListItem, DocType } from '@/lib/api'

const docTypeNames: Record<DocType, string> = {
  PO: '採購單',
  GRN: '採購入庫',
  PR: '採購退貨',
  SO: '銷貨單',
  DO: '銷貨出庫',
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

const receiptStatusNames: Record<string, string> = {
  pending: '未入庫',
  partial: '部分入庫',
  complete: '已入庫',
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

  if (doc.doc_type === 'PO' && doc.status === 'approved' && doc.receipt_status) {
    switch (doc.receipt_status) {
      case 'pending':
        badges.push(<Badge key="receipt" variant="destructive" className="ml-1">{receiptStatusNames[doc.receipt_status]}</Badge>)
        break
      case 'partial':
        badges.push(<Badge key="receipt" variant="warning" className="ml-1">{receiptStatusNames[doc.receipt_status]}</Badge>)
        break
      case 'complete':
        badges.push(<Badge key="receipt" variant="success" className="ml-1">{receiptStatusNames[doc.receipt_status]}</Badge>)
        break
    }
  }

  if (ACCOUNTING_DOC_TYPES.includes(doc.doc_type) && doc.has_journal_entry) {
    badges.push(
      <Badge key="journal" variant="outline" className="ml-1 text-xs border-emerald-300 text-emerald-700">
        已過帳
      </Badge>
    )
  }

  return <div className="flex flex-wrap items-center gap-1">{badges}</div>
}

export function DocumentTable({ documents, isLoading, onDeleteClick }: DocumentTableProps) {
  return (
    <div className="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>單號</TableHead>
            <TableHead>類型</TableHead>
            <TableHead>狀態</TableHead>
            <TableHead>對象</TableHead>
            <TableHead>倉庫</TableHead>
            <TableHead>單據日期</TableHead>
            <TableHead className="text-right">金額</TableHead>
            <TableHead>建立人</TableHead>
            <TableHead className="text-right">操作</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {isLoading ? (
            <TableRow>
              <TableCell colSpan={9} className="text-center py-8">
                <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
              </TableCell>
            </TableRow>
          ) : documents && documents.length > 0 ? (
            documents.map((doc) => (
              <TableRow key={doc.id}>
                <TableCell className="font-mono font-medium">{doc.doc_no}</TableCell>
                <TableCell>{docTypeNames[doc.doc_type]}</TableCell>
                <TableCell>{getStatusBadge(doc)}</TableCell>
                <TableCell>{doc.partner_name || '-'}</TableCell>
                <TableCell>{doc.warehouse_name || '-'}</TableCell>
                <TableCell>{formatDate(doc.doc_date)}</TableCell>
                <TableCell className="text-right">
                  {doc.total_amount ? formatCurrency(doc.total_amount) : '-'}
                </TableCell>
                <TableCell>{doc.created_by_name}</TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button variant="ghost" size="icon" asChild title="檢視">
                      <Link to={`/documents/${doc.id}`}>
                        <Eye className="h-4 w-4" />
                      </Link>
                    </Button>
                    {doc.status === 'draft' && (
                      <>
                        <Button variant="ghost" size="icon" asChild title="編輯">
                          <Link to={`/documents/${doc.id}/edit`}>
                            <Edit className="h-4 w-4" />
                          </Link>
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => onDeleteClick(doc)}
                          title="刪除"
                          className="text-destructive hover:text-destructive"
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </>
                    )}
                  </div>
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={9} className="text-center py-8">
                <FileText className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                <p className="text-muted-foreground">尚無單據資料</p>
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  )
}
