import { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import api, { Document, adminApproveDocument, adminRejectDocument, createGrnFromPo, reverseDocument, reverseApproveDocument } from '@/lib/api'
import { useAuthHasRole, useAuthHasPermission, useAuthUser } from '@/stores/auth'
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
import { ArrowLeft, Send, CheckCircle, XCircle, Loader2, ShieldCheck, ShieldX, AlertTriangle, Copy, PackagePlus, Undo2 } from 'lucide-react'
import { formatDate, formatNumber, formatCurrency, formatUom } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/apiError'
import { useTableSort } from '@/hooks/useTableSort'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { PendingOwnerBadge, PendingOwnerInline } from '@/components/PendingOwnerBadge'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { documentChangeQueryKeys } from './queryInvalidation'
import { ReversalNotice } from './components/ReversalNotice'
import { DOC_STATUS_NAMES, DOC_TYPE_NAMES } from './types'

const statusNames = DOC_STATUS_NAMES

/** 單據類型顯示名稱；SR／RTN 已從 DocType 移除，僅舊單據可能出現，故在此個別處理。 */
function docTypeLabel(t: TFunction, docType: string): string | undefined {
  if (docType === 'SR') return t('erpDocs.documents.docType.SR')
  if (docType === 'RTN') return t('erpDocs.documents.docType.RTN')
  return (DOC_TYPE_NAMES as Record<string, string>)[docType]
}

function managerApprovalLabel(t: TFunction, status: string): string | undefined {
  switch (status) {
    case 'pending':
      return t('erpDocs.documents.detail.managerApproval.pending')
    case 'wm_approved':
      return t('erpDocs.documents.detail.managerApproval.wmApproved')
    case 'approved':
      return t('erpDocs.documents.detail.managerApproval.approved')
    case 'rejected':
      return t('erpDocs.documents.detail.managerApproval.rejected')
    default:
      return undefined
  }
}

function AdjApprovalProgress({ document }: { document: Document }) {
  const { t } = useTranslation()
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
          {t('erpDocs.documents.detail.adjProgress.title')}
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
            {managerApprovalLabel(t, status) || status}
          </Badge>
          {document.scrap_total_amount && (
            <span className="text-xs text-muted-foreground">
              {t('erpDocs.documents.detail.adjProgress.amount', { amount: formatCurrency(document.scrap_total_amount) })}
            </span>
          )}
        </div>

        {/* 審批步驟 */}
        <div className="flex items-center gap-2 text-xs">
          <StepIndicator
            label={t('erpDocs.documents.detail.warehouseApprove')}
            done={status !== 'pending'}
            active={status === 'pending'}
          />
          <span className="text-muted-foreground">→</span>
          <StepIndicator
            label={t('erpDocs.documents.detail.adjProgress.stepAdmin')}
            done={status === 'approved'}
            active={status === 'wm_approved'}
            rejected={isRejected}
          />
        </div>

        {isRejected && document.manager_reject_reason && (
          <div className="rounded-md bg-destructive/10 p-3 text-sm">
            <span className="font-medium text-destructive">{t('erpDocs.documents.detail.adjProgress.rejectReason')}</span>
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
      : done ? 'border-status-success-text text-status-success-text bg-status-success-bg'
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
  const { t } = useTranslation()
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const hasRole = useAuthHasRole()
  const hasPermission = useAuthHasPermission()
  const currentUser = useAuthUser()

  const isAdmin = hasRole('admin') || hasRole('SYSTEM_ADMIN')
  const isWarehouseManager = hasRole('WAREHOUSE_MANAGER')
  // R71-8：核准/作廢按鈕 gate 補 permission token（與後端 require_permission! 對齊）；
  // 倉庫核准 vs 終審的分層保留。
  const canApproveDoc = hasPermission('erp.document.approve')
  const canCancelDoc = hasPermission('erp.document.cancel')
  // R97-1c：終審與沖銷核准改看權限碼，不再看「是不是系統管理員」。
  //
  // 負責人（DIRECTOR）刻意不具備倉管的 `erp.document.approve`——他不建單、不送審，
  // 只監督與終審。因此這兩顆按鈕**不可**再 && canApproveDoc，否則負責人看不到。
  // 後端 handler 已同步改為只要求這兩個碼。
  const canFinalApprove = hasPermission('erp.document.final_approve')
  const canReverseApprove = hasPermission('erp.document.reverse_approve')
  // 補齊 R71-8 未涵蓋的三顆：複製（建新單）、採購入庫（建 GRN）、送審。
  // 前兩者後端都是 create_document / create_grn_from_po → erp.document.create；
  // 送審是 submit_document → erp.document.submit（原本完全沒閘）。
  const canCreateDoc = hasPermission('erp.document.create')
  const canSubmitDoc = hasPermission('erp.document.submit')

  const { dialogState, confirm } = useConfirmDialog()
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

  const { sortedData: sortedLines, sort: lineSort, toggleSort: toggleLineSort } = useTableSort(document?.lines)

  // 單據狀態變更後失效相關 query；affectsStock=true 時一併失效庫存查詢，
  // 讓庫存頁面在核准後自動更新、毋須手動重新整理。
  const invalidateAfterDocChange = (affectsStock: boolean) => {
    for (const key of documentChangeQueryKeys(id, { affectsStock })) {
      queryClient.invalidateQueries({ queryKey: key })
    }
  }

  const submitMutation = useMutation({
    mutationFn: () => api.post(`/documents/${id}/submit`),
    onSuccess: () => {
      invalidateAfterDocChange(false)
      toast({ title: t('common.success'), description: t('erpDocs.documents.toast.submitted') })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpDocs.documents.toast.submitFailed')),
        variant: 'destructive',
      })
    },
  })

  const approveMutation = useMutation({
    mutationFn: () => api.post(`/documents/${id}/approve`),
    onSuccess: () => {
      invalidateAfterDocChange(true)
      toast({ title: t('common.success'), description: t('erpDocs.documents.detail.toast.approved') })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpDocs.documents.detail.toast.approveFailed')),
        variant: 'destructive',
      })
    },
  })

  const cancelMutation = useMutation({
    mutationFn: () => api.post(`/documents/${id}/cancel`),
    onSuccess: () => {
      invalidateAfterDocChange(false)
      toast({ title: t('common.success'), description: t('erpDocs.documents.detail.toast.cancelled') })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpDocs.documents.detail.toast.cancelFailed')),
        variant: 'destructive',
      })
    },
  })

  const createGrnMutation = useMutation({
    mutationFn: () => createGrnFromPo(id!),
    onSuccess: (data) => {
      toast({ title: t('common.success'), description: t('erpDocs.documents.detail.toast.grnCreated') })
      navigate(`/documents/${data.id}/edit`)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpDocs.documents.detail.toast.createGrnFailed')),
        variant: 'destructive',
      })
    },
  })

  const adminApproveMutation = useMutation({
    mutationFn: () => adminApproveDocument(id!),
    onSuccess: () => {
      invalidateAfterDocChange(true)
      toast({ title: t('common.success'), description: t('erpDocs.documents.detail.toast.adminApproved') })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpDocs.documents.detail.toast.adminApproveFailed')),
        variant: 'destructive',
      })
    },
  })

  // R84-5 發起沖銷：建立待管理員核准的沖銷單，此階段不動庫存
  const reverseMutation = useMutation({
    mutationFn: () => reverseDocument(id!),
    onSuccess: (data: { id?: string }) => {
      invalidateAfterDocChange(false)
      toast({
        title: t('erpDocs.documents.detail.toast.reverseCreatedTitle'),
        description: t('erpDocs.documents.detail.toast.reverseCreatedDesc'),
      })
      if (data?.id) navigate(`/documents/${data.id}`)
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpDocs.documents.detail.toast.reverseFailed')),
        variant: 'destructive',
      })
    },
  })

  // R84-5 管理員核准沖銷：執行庫存與會計的反向鏡射
  const reverseApproveMutation = useMutation({
    mutationFn: () => reverseApproveDocument(id!),
    onSuccess: () => {
      invalidateAfterDocChange(true)
      toast({
        title: t('erpDocs.documents.detail.toast.reverseApprovedTitle'),
        description: t('erpDocs.documents.detail.toast.reverseApprovedDesc'),
      })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpDocs.documents.detail.toast.reverseApproveFailed')),
        variant: 'destructive',
      })
    },
  })

  const adminRejectMutation = useMutation({
    mutationFn: (reason: string) => adminRejectDocument(id!, reason),
    onSuccess: () => {
      invalidateAfterDocChange(false)
      setRejectDialogOpen(false)
      setRejectReason('')
      toast({ title: t('common.success'), description: t('erpDocs.documents.detail.toast.rejected') })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('erpDocs.documents.detail.toast.rejectFailed')),
        variant: 'destructive',
      })
    },
  })

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'draft':
        return <Badge variant="secondary">{statusNames[status]}</Badge>
      case 'submitted':
        return (
          <PendingOwnerBadge owner={document?.pending_owner}>
            <Badge variant="warning">{statusNames[status]}</Badge>
          </PendingOwnerBadge>
        )
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
        <p className="text-muted-foreground">{t('erpDocs.documents.detail.notFound')}</p>
        <Button variant="outline" className="mt-4" onClick={() => navigate(-1)}>
          {t('erpDocs.shared.back')}
        </Button>
      </div>
    )
  }

  const handleCopyDocument = () => {
    if (!document) return
    const copyData = {
      doc_type: document.doc_type,
      warehouse_id: document.warehouse_id || '',
      warehouse_from_id: document.warehouse_from_id || '',
      warehouse_to_id: document.warehouse_to_id || '',
      partner_id: document.partner_id || '',
      protocol_id: document.protocol_id || '',
      remark: document.remark || '',
      lines: document.lines.map((line) => ({
        product_id: line.product_id,
        product_name: line.product_name,
        product_sku: line.product_sku,
        // 單位下拉的選項來源，與 useDocumentForm 載入既有單據時的 mapping 同源。
        // 少了這兩個欄位，複製出來的單據 buildUomOptions 只剩當前 uom 一個選項，
        // isUomReadOnly 會把該行判成唯讀 —— 複製後就再也改不了單位。
        base_uom: line.product_base_uom,
        alt_uoms: line.product_alt_uoms,
        qty: line.qty,
        uom: line.uom,
        unit_price: line.unit_price || '',
        batch_no: line.batch_no || '',
        expiry_date: line.expiry_date || '',
        remark: line.remark || '',
      })),
    }
    sessionStorage.setItem('document_copy_data', JSON.stringify(copyData))
    navigate(`/documents/new?type=${document.doc_type}&copy=true`)
  }

  // 判斷按鈕顯示邏輯
  const isSubmitted = document.status === 'submitted'
  const isAdjNeedsAdmin = document.requires_manager_approval === true
  const managerStatus = document.manager_approval_status

  // R97-1 SoD：ADJ / GRN 的建立者不得自行核准（後端 services/document/workflow.rs
  // 的 assert_document_sod 為真正的閘）。前端同步隱藏按鈕，避免新增一個
  // 「能點但必定 403」的落點——那正是 R89 已記錄的既有問題類型，不該再長出新的。
  // TR / STK / PO / SO 刻意不在範圍內，理由見後端常數註解。
  const isSodRestrictedType = document.doc_type === 'ADJ' || document.doc_type === 'GRN'
  const isOwnDocument = !!currentUser && currentUser.id === document.created_by
  const blockedBySelfApproval = isSodRestrictedType && isOwnDocument

  // R97-1 SoD 第二道：`admin_approve` 另外擋「第一階段的 WM 核准人再來做最終核准」
  // （approved_by === admin_id）。前端要一起擋，否則同時具 WM + admin 的人會看到
  // 一顆必定 403 的「管理員核准」鈕。
  const blockedByAlreadyApprovedByMe =
    !!currentUser && !!document.approved_by && currentUser.id === document.approved_by

  // 沖銷核准的 SoD **不分單據類型**（後端 approve_reversal 對所有 doc_type 都擋
  // 「沖銷單發起人自核」），故不能沿用只涵蓋 ADJ/GRN 的 blockedBySelfApproval。
  const blockedByReversalSelfApproval = isOwnDocument

  // 倉庫核准按鈕：submitted 且 (非大金額 ADJ 或大金額 ADJ pending)
  const showWmApprove = isSubmitted && canApproveDoc && isWarehouseManager && (
    !isAdjNeedsAdmin || managerStatus === 'pending'
  ) && !blockedBySelfApproval

  // R84-5：沖銷單同樣是 requires_manager_approval + wm_approved，必須從一般「最終核准」
  // 排除——那條路會重跑業務邏輯而非鏡射原單（後端 admin_approve 亦已擋下）。
  const isReversal = !!document.reverses_doc_id

  // 終審核准/駁回按鈕：submitted 且大金額 ADJ 已倉庫核准（不含沖銷單）
  //
  // 終審階段的共同前提（核准與駁回都要滿足）。
  // R97-1c：閘改為 `canFinalApprove`（權限碼），不再是 `canApproveDoc && isAdmin`——
  // 終審者是業務負責人，不具備倉管的 approve 權，用舊條件他會看不到按鈕。
  const adminStageReady = isSubmitted && canFinalApprove && isAdjNeedsAdmin
    && managerStatus === 'wm_approved' && !isReversal

  // 最終核准：受兩道 SoD 守衛（後端 admin_approve 為真正的閘）。
  const showAdminApprove = adminStageReady
    && !blockedBySelfApproval && !blockedByAlreadyApprovedByMe

  // 駁回：**刻意不受 SoD 守衛**，後端 admin_reject 亦然。
  //
  // 理由有二。其一，駁回只是把單退回草稿，不寫庫存也不過帳，屬「不作為」，
  // 而 SoD 防的是未經第二人同意的「作為」（CodeRabbit 於 PR #139 建議連 admin_reject
  // 一併加守衛，此處刻意不採）。
  //
  // 其二更關鍵——**駁回是這條流程唯一的逃生口**：終審者若同時被兩道守衛擋住
  // （例如他既是建單者、又做過第一階段核准），此時若連駁回也一起隱藏，
  // 這張單在介面上將既不能核准也不能駁回，永久卡死。
  // 兩顆按鈕因此必須分開判斷，不可共用同一個旗標。
  const showAdminReject = adminStageReady

  // 沖銷核准；後端 approve_reversal 擋發起人自核。
  // R97-1c：同樣改看權限碼，理由同上。
  const showReversalApprove = isSubmitted && canReverseApprove && isReversal
    && !blockedByReversalSelfApproval

  // 發起沖銷：已核准、尚未被沖銷、本身不是沖銷單，倉管或 admin 可發起
  const showReverse =
    document.status === 'approved' &&
    !isReversal &&
    !document.reversed_by_doc_id &&
    canApproveDoc &&
    (isWarehouseManager || isAdmin)

  // 作廢按鈕：submitted 且 (倉庫管理員或 admin)
  const showCancel = isSubmitted && canCancelDoc && (isWarehouseManager || isAdmin)

  // 採購入庫按鈕：PO 已核准且未完全入庫，倉庫管理員可用
  // 採購入庫建立 GRN 單 → 後端 create_grn_from_po 要求 erp.document.create。
  // 原本只看角色，倉管若沒有該權限會看到一顆必定 403 的按鈕。角色層保留
  // （這是誰該做入庫的業務分工），permission 層是後端真正的閘，兩層都要。
  const showCreateGrn =
    document.doc_type === 'PO' &&
    document.status === 'approved' &&
    (document.receipt_status === 'pending' || document.receipt_status === 'partial') &&
    canCreateDoc &&
    (isWarehouseManager || isAdmin)

  // R71-10：最終核准（admin）為單據生效的關鍵動作，送出前加二次確認。
  const handleAdminApprove = async () => {
    const ok = await confirm({
      title: t('erpDocs.documents.detail.confirm.adminApproveTitle'),
      description: t('erpDocs.documents.detail.confirm.adminApproveDescription'),
      confirmLabel: t('erpDocs.documents.detail.confirm.adminApproveLabel'),
    })
    if (ok) adminApproveMutation.mutate()
  }

  // R84-5：沖銷會反向動庫存與會計帳，且一張單只能沖銷一次，送出前二次確認。
  const handleReverse = async () => {
    const ok = await confirm({
      title: t('erpDocs.documents.detail.confirm.reverseTitle'),
      description: t('erpDocs.documents.detail.confirm.reverseDescription', { docNo: document.doc_no }),
      confirmLabel: t('erpDocs.documents.detail.confirm.reverseLabel'),
    })
    if (ok) reverseMutation.mutate()
  }

  const handleReverseApprove = async () => {
    const ok = await confirm({
      title: t('erpDocs.documents.detail.confirm.reverseApproveTitle'),
      description: t('erpDocs.documents.detail.confirm.reverseApproveDescription'),
      confirmLabel: t('erpDocs.documents.detail.confirm.reverseApproveLabel'),
    })
    if (ok) reverseApproveMutation.mutate()
  }

  // 2026-07-16：GRN（採購入庫）改軟擋——缺儲位仍可核准，但核准後這些量會變成「未分配」，
  // 需事後至倉庫頁分配上架。核准前若有未指定儲位的行，跳確認彈窗提醒。
  const handleApprove = async () => {
    const unshelvedCount =
      document.doc_type === 'GRN'
        ? (document.lines ?? []).filter((l) => !l.storage_location_id).length
        : 0
    if (unshelvedCount > 0) {
      const ok = await confirm({
        title: t('erpDocs.documents.detail.confirm.unshelvedTitle'),
        description: t('erpDocs.documents.detail.confirm.unshelvedDescription', { count: unshelvedCount }),
        confirmLabel: t('erpDocs.documents.detail.confirm.unshelvedLabel'),
      })
      if (!ok) return
    }
    approveMutation.mutate()
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="outline" size="icon" onClick={() => navigate(-1)} aria-label={t('erpDocs.shared.back')}>
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-3xl font-bold tracking-tight">{document.doc_no}</h1>
              {getStatusBadge(document.status)}
            </div>
            <p className="text-muted-foreground">
              {t('erpDocs.documents.detail.createdOn', {
                type: docTypeLabel(t, document.doc_type) ?? document.doc_type,
                date: formatDate(document.created_at),
              })}
            </p>
            {/* 詳情頁不把「卡在誰」藏在 hover 後面：這裡是使用者來查進度的地方 */}
            <PendingOwnerInline
              owner={document.pending_owner}
              className="text-sm text-muted-foreground"
            />
          </div>
        </div>
        <div className="flex gap-2">
          {canCreateDoc && (
            <Button variant="outline" onClick={handleCopyDocument}>
              <Copy className="mr-2 h-4 w-4" />
              {t('erpDocs.documents.detail.copyDocument')}
            </Button>
          )}
          {showCreateGrn && (
            <Button onClick={() => createGrnMutation.mutate()} disabled={createGrnMutation.isPending}>
              {createGrnMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <PackagePlus className="mr-2 h-4 w-4" />
              )}
              {t('erpDocs.documents.docType.GRN')}
            </Button>
          )}
          {document.status === 'draft' && canSubmitDoc && (
            <Button onClick={() => submitMutation.mutate()} disabled={submitMutation.isPending}>
              {submitMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Send className="mr-2 h-4 w-4" />
              )}
              {t('erpDocs.documents.detail.submit')}
            </Button>
          )}
          {showWmApprove && (
            <Button onClick={handleApprove} disabled={approveMutation.isPending}>
              {approveMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <CheckCircle className="mr-2 h-4 w-4" />
              )}
              {isAdjNeedsAdmin ? t('erpDocs.documents.detail.warehouseApprove') : t('erpDocs.documents.detail.approve')}
            </Button>
          )}
          {showAdminApprove && (
            <Button onClick={handleAdminApprove} disabled={adminApproveMutation.isPending}>
              {adminApproveMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <ShieldCheck className="mr-2 h-4 w-4" />
              )}
              {t('erpDocs.documents.detail.finalApprove')}
            </Button>
          )}
          {showAdminReject && (
            <Button
              variant="destructive"
              onClick={() => setRejectDialogOpen(true)}
            >
              <ShieldX className="mr-2 h-4 w-4" />
              {t('erpDocs.documents.detail.reject')}
            </Button>
          )}
          {showReverse && (
            <Button variant="outline" onClick={handleReverse} disabled={reverseMutation.isPending}>
              {reverseMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Undo2 className="mr-2 h-4 w-4" />
              )}
              {t('erpDocs.documents.detail.initiateReversal')}
            </Button>
          )}
          {showReversalApprove && (
            <Button onClick={handleReverseApprove} disabled={reverseApproveMutation.isPending}>
              {reverseApproveMutation.isPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <ShieldCheck className="mr-2 h-4 w-4" />
              )}
              {t('erpDocs.documents.detail.approveReversal')}
            </Button>
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
              {t('erpDocs.documents.detail.voidDocument')}
            </Button>
          )}
        </div>
      </div>

      {/* R84-5 沖銷關聯（雙向）：本單被沖銷 / 本單是沖銷單 */}
      <ReversalNotice document={document} navigate={navigate} />

      {/* 大金額 ADJ 審批進度（沖銷單走專屬流程，不顯示此卡） */}
      {document.requires_manager_approval && !document.reverses_doc_id && (
        <AdjApprovalProgress document={document} />
      )}

      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>{t('erpDocs.shared.docInfo')}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <div className="flex justify-between">
              <span className="text-muted-foreground">{t('erpDocs.shared.docType')}</span>
              <span>{docTypeLabel(t, document.doc_type)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">{t('erpDocs.shared.docDate')}</span>
              <span>{formatDate(document.doc_date)}</span>
            </div>
            {document.warehouse_name && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">{t('erpDocs.shared.warehouse')}</span>
                <span>{document.warehouse_name}</span>
              </div>
            )}
            {document.warehouse_from_name && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">{t('erpDocs.shared.sourceWarehouse')}</span>
                <span>{document.warehouse_from_name}</span>
              </div>
            )}
            {document.warehouse_to_name && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">{t('erpDocs.shared.targetWarehouse')}</span>
                <span>{document.warehouse_to_name}</span>
              </div>
            )}
            {document.partner_name && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">{t('erpDocs.shared.partner')}</span>
                <span>{document.partner_name}</span>
              </div>
            )}
            {document.remark && (
              <div className="flex justify-between">
                <span className="text-muted-foreground">{t('erpDocs.shared.remark')}</span>
                <span>{document.remark}</span>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{t('erpDocs.documents.detail.processingInfo')}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <div className="flex justify-between">
              <span className="text-muted-foreground">{t('erpDocs.shared.createdBy')}</span>
              <span>{document.created_by_name}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">{t('erpDocs.documents.detail.createdAt')}</span>
              <span>{formatDate(document.created_at)}</span>
            </div>
            {document.approved_by_name && (
              <>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">{t('erpDocs.documents.detail.approvedBy')}</span>
                  <span>{document.approved_by_name}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">{t('erpDocs.documents.detail.approvedAt')}</span>
                  <span>{document.approved_at ? formatDate(document.approved_at) : '-'}</span>
                </div>
              </>
            )}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>{t('erpDocs.shared.docLines')}</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="@container">
            <div className="hidden @[600px]:block overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <SortableTableHead className="w-16" sortKey="line_no" currentSort={lineSort.column} currentDirection={lineSort.direction} onSort={toggleLineSort}>{t('erpDocs.shared.lineNo')}</SortableTableHead>
                    <SortableTableHead sortKey="product_name" currentSort={lineSort.column} currentDirection={lineSort.direction} onSort={toggleLineSort}>{t('erpDocs.shared.item')}</SortableTableHead>
                    <SortableTableHead className="text-right" sortKey="qty" currentSort={lineSort.column} currentDirection={lineSort.direction} onSort={toggleLineSort}>{t('erpDocs.shared.quantity')}</SortableTableHead>
                    <SortableTableHead sortKey="uom" currentSort={lineSort.column} currentDirection={lineSort.direction} onSort={toggleLineSort}>{t('erpDocs.shared.unit')}</SortableTableHead>
                    <SortableTableHead className="text-right hidden @[750px]:table-cell" sortKey="unit_price" currentSort={lineSort.column} currentDirection={lineSort.direction} onSort={toggleLineSort}>{t('erpDocs.shared.unitPrice')}</SortableTableHead>
                    <TableHead className="text-right hidden @[750px]:table-cell">{t('erpDocs.shared.amount')}</TableHead>
                    <SortableTableHead className="hidden @[900px]:table-cell" sortKey="batch_no" currentSort={lineSort.column} currentDirection={lineSort.direction} onSort={toggleLineSort}>{t('erpDocs.shared.batchNo')}</SortableTableHead>
                    <SortableTableHead className="hidden @[900px]:table-cell" sortKey="expiry_date" currentSort={lineSort.column} currentDirection={lineSort.direction} onSort={toggleLineSort}>{t('erpDocs.shared.expiryDate')}</SortableTableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {(sortedLines ?? document.lines).map((line) => (
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
                      <TableCell className="text-right hidden @[750px]:table-cell">
                        {line.unit_price ? formatCurrency(line.unit_price) : '-'}
                      </TableCell>
                      <TableCell className="text-right hidden @[750px]:table-cell">
                        {line.unit_price
                          ? formatCurrency(parseFloat(line.qty) * parseFloat(line.unit_price))
                          : '-'}
                      </TableCell>
                      <TableCell className="hidden @[900px]:table-cell">{line.batch_no || '-'}</TableCell>
                      <TableCell className="hidden @[900px]:table-cell">{line.expiry_date ? formatDate(line.expiry_date) : '-'}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>

            <div className="@[600px]:hidden divide-y">
              {(sortedLines ?? document.lines).map((line) => (
                <div key={line.id} className="p-3 space-y-1">
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-baseline gap-2">
                        <span className="text-xs text-muted-foreground">#{line.line_no}</span>
                        <span className="font-medium break-words">{line.product_name}</span>
                      </div>
                      <div className="text-xs text-muted-foreground">{line.product_sku}</div>
                    </div>
                    <div className="text-right shrink-0">
                      <div className="font-medium">{formatNumber(line.qty, 0)} {formatUom(line.uom)}</div>
                      {line.unit_price && (
                        <div className="text-xs text-muted-foreground">
                          {formatCurrency(parseFloat(line.qty) * parseFloat(line.unit_price))}
                        </div>
                      )}
                    </div>
                  </div>
                  {(line.batch_no || line.expiry_date) && (
                    <div className="text-xs text-muted-foreground">
                      {line.batch_no && t('erpDocs.documents.detail.batchColon', { value: line.batch_no })}
                      {line.batch_no && line.expiry_date && ' · '}
                      {line.expiry_date && t('erpDocs.documents.detail.expiryColon', { value: formatDate(line.expiry_date) })}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* ADMIN 駁回原因 Dialog */}
      <Dialog open={rejectDialogOpen} onOpenChange={setRejectDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-destructive" />
              {t('erpDocs.documents.detail.rejectDialog.title')}
            </DialogTitle>
            <DialogDescription>
              {t('erpDocs.documents.detail.rejectDialog.description')}
            </DialogDescription>
          </DialogHeader>
          <Textarea
            placeholder={t('erpDocs.documents.detail.rejectDialog.placeholder')}
            value={rejectReason}
            onChange={(e) => setRejectReason(e.target.value)}
            rows={3}
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setRejectDialogOpen(false)}>
              {t('common.cancel')}
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
              {t('erpDocs.documents.detail.rejectDialog.confirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
