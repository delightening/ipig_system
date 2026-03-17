import { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Document, adminApproveDocument, adminRejectDocument } from '@/lib/api'
import { useAuthHasRole } from '@/stores/auth'
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Textarea } from '@/components/ui/textarea'
import { toast } from '@/components/ui/use-toast'
import { ArrowLeft, Send, CheckCircle, XCircle, Loader2, ShieldCheck, ShieldX, AlertTriangle } from 'lucide-react'
import { formatDate, formatNumber, formatCurrency, formatUom } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'

const docTypeNames: Record<string, string> = {
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

const managerApprovalLabels: Record<string, string> = {
  pending: '待倉庫核准',
  wm_approved: '倉庫已核准，待管理員核准',
  approved: '管理員已核准',
  rejected: '管理員已駁回',
}

function AdjApprovalProgress({ document }: { document: Document }) {
  if (!document.requires_manager_approval) return null

  const status = document.manager_approval_status || 'pending'
  const isRejected = status === 'rejected'

  return (
    <Card className={isRejected ? 'border-destructive' : ''}>
      <CardHeader className="pb-3">
        <CardTitle className="text-sm flex items-center gap-2">
          {isRejected ? (
            <AlertTriangle className="h-4 w-4 text-destructive" />
          ) : (
            <ShieldCheck className="h-4 w-4 text-primary" />
          )}
          大金額調整單審批進度
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex items-center gap-2">
          <Badge variant={
            status === 'pending' ? 'warning'
            : status === 'wm_approved' ? 'secondary'
            : status === 'approved' ? 'success'
            : 'destructive'
          }>
            {managerApprovalLabels[status] || status}
          </Badge>
          {document.scrap_total_amount && (
            <span className="text-xs text-muted-foreground">
              調整金額：{formatCurrency(document.scrap_total_amount)}
            </span>
          )}
        </div>

        {/* 審批步驟 */}
        <div className="flex items-center gap-2 text-xs">
          <StepIndicator
            label="倉庫核准"
            done={status !== 'pending'}
            active={status === 'pending'}
          />
          <span className="text-muted-foreground">→</span>
          <StepIndicator
            label="管理員核准"
            done={status === 'approved'}
            active={status === 'wm_approved'}
            rejected={isRejected}
          />
        </div>

        {isRejected && document.manager_reject_reason && (
          <div className="rounded-md bg-destructive/10 p-3 text-sm">
            <span className="font-medium text-destructive">駁回原因：</span>
            <span>{document.manager_reject_reason}</span>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

function StepIndicator({ label, done, active, rejected }: {
  label: string
  done: boolean
  active: boolean
  rejected?: boolean
}) {
  return (
    <div className={`flex items-center gap-1 px-2 py-1 rounded-full border text-xs ${
      rejected ? 'border-destructive text-destructive bg-destructive/10'
      : done ? 'border-green-500 text-green-700 bg-green-50'
      : active ? 'border-primary text-primary bg-primary/10'
      : 'border-muted text-muted-foreground'
    }`}>
      {rejected ? <XCircle className="h-3 w-3" /> :
       done ? <CheckCircle className="h-3 w-3" /> : null}
      {label}
    </div>
  )
}

export function DocumentDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const hasRole = useAuthHasRole()

  const isAdmin = hasRole('admin') || hasRole('SYSTEM_ADMIN')
  const isWarehouseManager = hasRole('WAREHOUSE_MANAGER')

  const [rejectDialogOpen, setRejectDialogOpen] = useState(false)
  const [rejectReason, setRejectReason] = useState('')

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
      setTimeout(() => {
        window.location.reload()
      }, 500)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '送審失敗'),
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
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '核准失敗'),
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
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '作廢失敗'),
        variant: 'destructive',
      })
    },
  })

  const adminApproveMutation = useMutation({
    mutationFn: () => adminApproveDocument(id!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['document', id] })
      toast({ title: '成功', description: '管理員已完成最終核准' })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '管理員核准失敗'),
        variant: 'destructive',
      })
    },
  })

  const adminRejectMutation = useMutation({
    mutationFn: (reason: string) => adminRejectDocument(id!, reason),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['document', id] })
      setRejectDialogOpen(false)
      setRejectReason('')
      toast({ title: '成功', description: '單據已駁回，退回草稿' })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '駁回失敗'),
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

  // 判斷按鈕顯示邏輯
  const isSubmitted = document.status === 'submitted'
  const isAdjNeedsAdmin = document.requires_manager_approval === true
  const managerStatus = document.manager_approval_status

  // 倉庫核准按鈕：submitted 且 (非大金額 ADJ 或大金額 ADJ pending)
  const showWmApprove = isSubmitted && isWarehouseManager && (
    !isAdjNeedsAdmin || managerStatus === 'pending'
  )

  // ADMIN 核准/駁回按鈕：submitted 且大金額 ADJ 已倉庫核准
  const showAdminActions = isSubmitted && isAdmin && isAdjNeedsAdmin && managerStatus === 'wm_approved'

  // 作廢按鈕：submitted 且 (倉庫管理員或 admin)
  const showCancel = isSubmitted && (isWarehouseManager || isAdmin)

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
          {showWmApprove && (
            <Button onClick={() => approveMutation.mutate()} disabled={approveMutation.isPending}>
              {approveMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <CheckCircle className="mr-2 h-4 w-4" />
              )}
              {isAdjNeedsAdmin ? '倉庫核准' : '核准'}
            </Button>
          )}
          {showAdminActions && (
            <>
              <Button onClick={() => adminApproveMutation.mutate()} disabled={adminApproveMutation.isPending}>
                {adminApproveMutation.isPending ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <ShieldCheck className="mr-2 h-4 w-4" />
                )}
                最終核准
              </Button>
              <Button
                variant="destructive"
                onClick={() => setRejectDialogOpen(true)}
              >
                <ShieldX className="mr-2 h-4 w-4" />
                駁回
              </Button>
            </>
          )}
          {showCancel && (
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
          )}
        </div>
      </div>

      {/* 大金額 ADJ 審批進度 */}
      {document.requires_manager_approval && (
        <AdjApprovalProgress document={document} />
      )}

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
                  <TableCell>{formatUom(line.uom)}</TableCell>
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

      {/* ADMIN 駁回原因 Dialog */}
      <Dialog open={rejectDialogOpen} onOpenChange={setRejectDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-destructive" />
              駁回調整單
            </DialogTitle>
            <DialogDescription>
              駁回後單據將退回草稿狀態，建立者可修改後重新提交。
            </DialogDescription>
          </DialogHeader>
          <Textarea
            placeholder="請輸入駁回原因..."
            value={rejectReason}
            onChange={(e) => setRejectReason(e.target.value)}
            rows={3}
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setRejectDialogOpen(false)}>
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={() => adminRejectMutation.mutate(rejectReason)}
              disabled={!rejectReason.trim() || adminRejectMutation.isPending}
            >
              {adminRejectMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <ShieldX className="mr-2 h-4 w-4" />
              )}
              確認駁回
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
