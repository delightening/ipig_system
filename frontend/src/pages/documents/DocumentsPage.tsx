import { useState } from 'react'
import { useSearchParams, Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { DocumentListItem, DocType } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Label } from '@/components/ui/label'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useToast } from '@/components/ui/use-toast'
import { Plus, Search, Eye, Loader2, FileText, Calendar, X, Edit, Trash2 } from 'lucide-react'
import { formatDate, formatCurrency } from '@/lib/utils'

const docTypeNames: Record<DocType, string> = {
  PO: '採購單',
  GRN: '採購入庫',
  PR: '採購退貨',
  SO: '銷售單',
  DO: '銷售出庫',
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

export function DocumentsPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const typeFilter = searchParams.get('type') || ''
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')
  const [dateFrom, setDateFrom] = useState('')
  const [dateTo, setDateTo] = useState('')
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [documentToDelete, setDocumentToDelete] = useState<DocumentListItem | null>(null)
  const queryClient = useQueryClient()
  const { toast } = useToast()

  const { data: documents, isLoading } = useQuery({
    queryKey: ['documents', typeFilter, statusFilter, search, dateFrom, dateTo],
    queryFn: async () => {
      let params = ''
      if (typeFilter) params += `doc_type=${typeFilter}&`
      if (statusFilter && statusFilter !== 'all') params += `status=${statusFilter}&`
      if (search) params += `keyword=${encodeURIComponent(search)}&`
      if (dateFrom) params += `date_from=${dateFrom}&`
      if (dateTo) params += `date_to=${dateTo}&`
      const response = await api.get<DocumentListItem[]>(`/documents?${params}`)
      return response.data
    },
  })

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/documents/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['documents'] })
      toast({
        title: '成功',
        description: '單據已刪除',
      })
      setDeleteDialogOpen(false)
      setDocumentToDelete(null)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '刪除失敗',
        variant: 'destructive',
      })
    },
  })

  const handleDeleteClick = (doc: DocumentListItem) => {
    setDocumentToDelete(doc)
    setDeleteDialogOpen(true)
  }

  const handleConfirmDelete = () => {
    if (documentToDelete) {
      deleteMutation.mutate(documentToDelete.id)
    }
  }

  const clearFilters = () => {
    setSearch('')
    setStatusFilter('all')
    setDateFrom('')
    setDateTo('')
  }

  const hasFilters = search || (statusFilter && statusFilter !== 'all') || dateFrom || dateTo

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'draft':
        return <Badge variant="secondary">{statusNames[status]}</Badge>
      case 'submitted':
        return <Badge variant="warning">{statusNames[status]}</Badge>
      case 'approved':
        return <Badge variant="success">{statusNames[status]}</Badge>
      case 'cancelled':
        return <Badge variant="destructive">{statusNames[status]}</Badge>
      default:
        return <Badge variant="outline">{status}</Badge>
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">
            {typeFilter ? docTypeNames[typeFilter as DocType] : '單據管理'}
          </h1>
          <p className="text-muted-foreground">
            {typeFilter ? `管理${docTypeNames[typeFilter as DocType]}` : '管理系統中的所有單據'}
          </p>
        </div>
        <Button asChild>
          <Link to={`/documents/new${typeFilter ? `?type=${typeFilter}` : ''}`}>
            <Plus className="mr-2 h-4 w-4" />
            新增單據
          </Link>
        </Button>
      </div>

      <div className="space-y-4">
        <div className="flex flex-wrap gap-4 items-end">
          <div className="relative flex-1 min-w-[200px] max-w-sm">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="搜尋單號..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-9"
            />
          </div>
          {!typeFilter && (
            <Select
              value={typeFilter || 'all'}
              onValueChange={(value) => {
                if (value && value !== 'all') {
                  setSearchParams({ type: value })
                } else {
                  setSearchParams({})
                }
              }}
            >
              <SelectTrigger className="w-40">
                <SelectValue placeholder="全部類型" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部類型</SelectItem>
                {Object.entries(docTypeNames).map(([key, name]) => (
                  <SelectItem key={key} value={key}>
                    {name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
          <Select value={statusFilter} onValueChange={setStatusFilter}>
            <SelectTrigger className="w-40">
              <SelectValue placeholder="全部狀態" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部狀態</SelectItem>
              {Object.entries(statusNames).map(([key, name]) => (
                <SelectItem key={key} value={key}>
                  {name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          
          {/* 日期區間篩選 */}
          <div className="flex items-center gap-2">
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground">起始日期</Label>
              <div className="relative">
                <Calendar className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  type="date"
                  value={dateFrom}
                  onChange={(e) => setDateFrom(e.target.value)}
                  className="pl-9 w-[150px]"
                />
              </div>
            </div>
            <span className="text-muted-foreground mt-5">~</span>
            <div className="space-y-1">
              <Label className="text-xs text-muted-foreground">結束日期</Label>
              <div className="relative">
                <Calendar className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  type="date"
                  value={dateTo}
                  onChange={(e) => setDateTo(e.target.value)}
                  className="pl-9 w-[150px]"
                />
              </div>
            </div>
          </div>

          {hasFilters && (
            <Button variant="ghost" size="sm" onClick={clearFilters} className="mt-5">
              <X className="h-4 w-4 mr-1" />
              清除篩選
            </Button>
          )}
        </div>
      </div>

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
                  <TableCell>{getStatusBadge(doc.status)}</TableCell>
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
                            onClick={() => handleDeleteClick(doc)}
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

      {/* 刪除確認對話框 */}
      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>確認刪除</DialogTitle>
            <DialogDescription>
              確定要刪除單據「{documentToDelete?.doc_no}」嗎？此操作無法復原。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteDialogOpen(false)}>
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={handleConfirmDelete}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              確認刪除
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
