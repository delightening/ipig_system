import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Document } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { toast } from '@/components/ui/use-toast'
import { ArrowLeft, Send, CheckCircle, XCircle, Loader2 } from 'lucide-react'
import { formatDate, formatNumber, formatCurrency } from '@/lib/utils'

const docTypeNames: Record<string, string> = {
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

export function DocumentDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const { data: document, isLoading } = useQuery({
    queryKey: ['document', id],
    queryFn: async () => {
      const response = await api.get<Document>(`/documents/${id}`)
      return response.data
    },
    enabled: !!id,
  })

  const submitMutation = useMutation({
    mutationFn: () => api.post(`/documents/${id}/submit`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['document', id] })
      toast({ title: '成功', description: '單據已送審' })
      // 自動重新整理頁面以獲得更新後的資訊
      setTimeout(() => {
        window.location.reload()
      }, 500)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '送審失敗',
        variant: 'destructive',
      })
    },
  })

  const approveMutation = useMutation({
    mutationFn: () => api.post(`/documents/${id}/approve`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['document', id] })
      toast({ title: '成功', description: '單據已核准' })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '核准失敗',
        variant: 'destructive',
      })
    },
  })

  const cancelMutation = useMutation({
    mutationFn: () => api.post(`/documents/${id}/cancel`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['document', id] })
      toast({ title: '成功', description: '單據已作廢' })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '作廢失敗',
        variant: 'destructive',
      })
    },
  })

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

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!document) {
    return (
      <div className="text-center py-8">
        <p className="text-muted-foreground">找不到此單據</p>
        <Button variant="outline" className="mt-4" onClick={() => navigate(-1)}>
          返回
        </Button>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="outline" size="icon" onClick={() => navigate(-1)}>
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-3xl font-bold tracking-tight">{document.doc_no}</h1>
              {getStatusBadge(document.status)}
            </div>
            <p className="text-muted-foreground">
              {docTypeNames[document.doc_type]} · 建立於 {formatDate(document.created_at)}
            </p>
          </div>
        </div>
        <div className="flex gap-2">
          {document.status === 'draft' && (
            <Button onClick={() => submitMutation.mutate()} disabled={submitMutation.isPending}>
              {submitMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Send className="mr-2 h-4 w-4" />
              )}
              送審
            </Button>
          )}
          {document.status === 'submitted' && (
            <>
              <Button onClick={() => approveMutation.mutate()} disabled={approveMutation.isPending}>
                {approveMutation.isPending ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <CheckCircle className="mr-2 h-4 w-4" />
                )}
                核准
              </Button>
              <Button
                variant="destructive"
                onClick={() => cancelMutation.mutate()}
                disabled={cancelMutation.isPending}
              >
                {cancelMutation.isPending ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <XCircle className="mr-2 h-4 w-4" />
                )}
                作廢
              </Button>
            </>
          )}
        </div>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>單據資訊</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <div className="flex justify-between">
              <span className="text-muted-foreground">單據類型</span>
              <span>{docTypeNames[document.doc_type]}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">單據日期</span>
              <span>{formatDate(document.doc_date)}</span>
            </div>
            {document.warehouse_name && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">倉庫</span>
                <span>{document.warehouse_name}</span>
              </div>
            )}
            {document.warehouse_from_name && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">來源倉庫</span>
                <span>{document.warehouse_from_name}</span>
              </div>
            )}
            {document.warehouse_to_name && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">目標倉庫</span>
                <span>{document.warehouse_to_name}</span>
              </div>
            )}
            {document.partner_name && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">對象</span>
                <span>{document.partner_name}</span>
              </div>
            )}
            {document.remark && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">備註</span>
                <span>{document.remark}</span>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>處理資訊</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <div className="flex justify-between">
              <span className="text-muted-foreground">建立人</span>
              <span>{document.created_by_name}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">建立時間</span>
              <span>{formatDate(document.created_at)}</span>
            </div>
            {document.approved_by_name && (
              <>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">核准人</span>
                  <span>{document.approved_by_name}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">核准時間</span>
                  <span>{document.approved_at ? formatDate(document.approved_at) : '-'}</span>
                </div>
              </>
            )}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>單據明細</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-16">項次</TableHead>
                <TableHead>品項</TableHead>
                <TableHead className="text-right">數量</TableHead>
                <TableHead>單位</TableHead>
                <TableHead className="text-right">單價</TableHead>
                <TableHead className="text-right">金額</TableHead>
                <TableHead>批號</TableHead>
                <TableHead>效期</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {document.lines.map((line) => (
                <TableRow key={line.id}>
                  <TableCell>{line.line_no}</TableCell>
                  <TableCell>
                    <div>
                      <div className="font-medium">{line.product_name}</div>
                      <div className="text-xs text-muted-foreground">{line.product_sku}</div>
                    </div>
                  </TableCell>
                  <TableCell className="text-right">{formatNumber(line.qty, 0)}</TableCell>
                  <TableCell>{line.uom}</TableCell>
                  <TableCell className="text-right">
                    {line.unit_price ? formatCurrency(line.unit_price) : '-'}
                  </TableCell>
                  <TableCell className="text-right">
                    {line.unit_price
                      ? formatCurrency(parseFloat(line.qty) * parseFloat(line.unit_price))
                      : '-'}
                  </TableCell>
                  <TableCell>{line.batch_no || '-'}</TableCell>
                  <TableCell>{line.expiry_date ? formatDate(line.expiry_date) : '-'}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
