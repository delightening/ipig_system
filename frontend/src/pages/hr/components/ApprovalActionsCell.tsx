import { useTranslation } from 'react-i18next'
import { CheckCircle, XCircle } from 'lucide-react'

import { Button } from '@/components/ui/button'

interface ApprovalActionsCellProps {
    id: string
    canApprove: boolean
    onApprove: (id: string) => void
    onReject: (id: string, reason: string) => void
    approvePending: boolean
    rejectPending: boolean
    rejectReason?: string
}

export function ApprovalActionsCell({
    id,
    canApprove,
    onApprove,
    onReject,
    approvePending,
    rejectPending,
    // 這個預設值會隨駁回請求送進後端、存進資料庫（API 契約），不是畫面文字，故不走 i18n
    rejectReason = '不符合規定',
}: ApprovalActionsCellProps) {
    const { t } = useTranslation()
    if (!canApprove) {
        return (
            <span className="text-muted-foreground">
                <span aria-hidden="true">—</span>
                <span className="sr-only">{t('hrPages.shared.noApprovalPermission')}</span>
            </span>
        )
    }

    return (
        <div className="flex items-center justify-end gap-1">
            <Button
                variant="default"
                size="sm"
                onClick={() => onApprove(id)}
                disabled={approvePending}
            >
                <CheckCircle className="h-4 w-4 mr-1" />
                {t('hrPages.shared.action.approve')}
            </Button>
            <Button
                variant="destructive"
                size="sm"
                onClick={() => onReject(id, rejectReason)}
                disabled={rejectPending}
            >
                <XCircle className="h-4 w-4 mr-1" />
                {t('hrPages.shared.action.reject')}
            </Button>
        </div>
    )
}
