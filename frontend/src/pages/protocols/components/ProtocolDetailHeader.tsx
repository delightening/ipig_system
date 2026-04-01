import { Link, useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { GuestHide } from '@/components/ui/guest-hide'
import { ArrowLeft, Edit, Send, Loader2 } from 'lucide-react'
import type { Protocol, ProtocolStatus } from '@/lib/api'
import { statusColors } from '../constants'

interface ProtocolDetailHeaderProps {
  protocol: Protocol
  protocolId: string
  isRevisionStatus: boolean
  canEditProtocol: boolean
  canAssignReviewer: boolean
  isVet: boolean | undefined
  availableTransitions: ProtocolStatus[]
  submitIsPending: boolean
  onSubmit: () => void
  onOpenStatusDialog: () => void
}

export function ProtocolDetailHeader({
  protocol,
  protocolId,
  isRevisionStatus,
  canEditProtocol,
  canAssignReviewer,
  isVet,
  availableTransitions,
  submitIsPending,
  onSubmit,
  onOpenStatusDialog,
}: ProtocolDetailHeaderProps) {
  const navigate = useNavigate()
  const { t } = useTranslation()

  const showEditButton = protocol.status === 'DRAFT' || (isRevisionStatus && canEditProtocol)
  const showSubmitButton = protocol.status === 'DRAFT' || (isRevisionStatus && canEditProtocol)
  const showStatusButton = availableTransitions.length > 0
    && protocol.status !== 'DRAFT'
    && (canAssignReviewer || (isVet && protocol.status === 'VET_REVIEW'))

  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
      <div className="flex items-center gap-3 md:gap-4">
        <Button variant="ghost" size="icon" onClick={() => navigate(-1)} aria-label="返回">
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
        </div>
      </GuestHide>
    </div>
  )
}
