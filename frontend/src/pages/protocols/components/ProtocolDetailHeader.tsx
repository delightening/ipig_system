import { Link, useNavigate, useLocation } from 'react-router-dom'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { GuestHide } from '@/components/ui/guest-hide'
import { ArrowLeft, Edit, Send, Loader2, Download, Trash2 } from 'lucide-react'
import type { Protocol, ProtocolStatus } from '@/lib/api'
import { statusColors } from '../constants'

interface ProtocolDetailHeaderProps {
  protocol: Protocol
  protocolId: string
  isRevisionStatus: boolean
  canEditProtocol: boolean
  canChangeStatus: boolean
  isVet: boolean | undefined
  availableTransitions: ProtocolStatus[]
  submitIsPending: boolean
  canSoftDelete: boolean
  softDeleteIsPending: boolean
  onSubmit: () => void
  onOpenStatusDialog: () => void
  onSoftDelete: () => void
}

export function ProtocolDetailHeader({
  protocol,
  protocolId,
  isRevisionStatus,
  canEditProtocol,
  canChangeStatus,
  isVet,
  availableTransitions,
  submitIsPending,
  canSoftDelete,
  softDeleteIsPending,
  onSubmit,
  onOpenStatusDialog,
  onSoftDelete,
}: ProtocolDetailHeaderProps) {
  const navigate = useNavigate()
  const location = useLocation()
  const { t } = useTranslation()

  // 返回對應的列表頁（依進入此詳情的路由），而非瀏覽器上一頁：
  // /my-projects/:id → /my-projects；其餘（/protocols/:id）→ /protocols。
  const backTo = location.pathname.startsWith('/my-projects') ? '/my-projects' : '/protocols'

  // 草稿 / 退回補件階段才顯示編輯·送出，且須為可編輯者（admin / PI / SD）——
  // 執秘 / CLIENT 等檢視草稿時不再出現編輯·送出鈕（對齊後端授權）。
  const showEditButton = (protocol.status === 'DRAFT' || isRevisionStatus) && canEditProtocol
  const showSubmitButton = (protocol.status === 'DRAFT' || isRevisionStatus) && canEditProtocol
  const showStatusButton = availableTransitions.length > 0
    && protocol.status !== 'DRAFT'
    && (canChangeStatus || (isVet && protocol.status === 'VET_REVIEW'))

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div className="flex items-center gap-3 md:gap-4">
        <Button variant="ghost" size="icon" onClick={() => navigate(backTo)} aria-label="返回">
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div>
          <div className="flex flex-wrap items-center gap-2 md:gap-3">
            <h1 className="text-xl md:text-2xl font-bold">{protocol.title}</h1>
            <Badge variant={statusColors[protocol.status]} className="text-sm">
              {t(`protocols.status.${protocol.status}`)}
            </Badge>
          </div>
        </div>
      </div>
      <GuestHide>
        <div className="flex flex-wrap gap-2 pl-11 md:pl-0">
          {showEditButton && (
            <Button variant="outline" asChild>
              <Link to={`/protocols/${protocolId}/edit`}>
                <Edit className="mr-2 h-4 w-4" />
                {isRevisionStatus ? t('protocols.detail.revise') : t('protocols.detail.edit')}
              </Link>
            </Button>
          )}
          {showSubmitButton && (
            <Button onClick={onSubmit} disabled={submitIsPending}>
              {submitIsPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Send className="mr-2 h-4 w-4" />
              )}
              {t('protocols.detail.submit')}
            </Button>
          )}
          {showStatusButton && (
            <Button variant="outline" onClick={onOpenStatusDialog}>
              {t('protocols.detail.changeStatus')}
            </Button>
          )}
          {canSoftDelete && (
            <Button variant="destructive" onClick={onSoftDelete} disabled={softDeleteIsPending}>
              {softDeleteIsPending ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Trash2 className="mr-2 h-4 w-4" />
              )}
              {t('protocols.detail.softDelete')}
            </Button>
          )}
          {/* AUP 計畫書 PDF 匯出（Word 匯出已移除：print-pdf 僅產 PDF，docx 為定版唯一輸出之外的損毀檔，見 R74-1） */}
          <Button
            variant="outline"
            onClick={() => {
              window.location.href = `/api/v1/protocols/${protocolId}/export-aup-v3?format=pdf`
            }}
            title={t('protocols.detail.downloadPdfHint')}
          >
            <Download className="mr-2 h-4 w-4" />
            {t('protocols.detail.downloadPdf')}
          </Button>
        </div>
      </GuestHide>
    </div>
  )
}
