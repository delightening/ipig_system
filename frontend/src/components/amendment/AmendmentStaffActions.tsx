import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Loader2, Play, Tag } from 'lucide-react'

interface StaffAction {
  enabled: boolean
  pending?: boolean
  onClick: () => void
}

interface AmendmentStaffActionsProps {
  classify: StaffAction
  startReview: StaffAction
  changeStatus: StaffAction
}

/** PageHeader 右側執秘 / 管理者操作按鈕（分類 / 開始審查 / 變更狀態）。 */
export function AmendmentStaffActions({ classify, startReview, changeStatus }: AmendmentStaffActionsProps) {
  const { t } = useTranslation()

  return (
    <div className="flex flex-wrap items-center gap-2">
      {classify.enabled && (
        <Button variant="outline" size="sm" onClick={classify.onClick}>
          <Tag className="mr-1 h-4 w-4" />
          {t('amendments.decision.classify.action')}
        </Button>
      )}
      {startReview.enabled && (
        <Button variant="outline" size="sm" onClick={startReview.onClick} disabled={startReview.pending}>
          {startReview.pending
            ? <Loader2 className="mr-1 h-4 w-4 animate-spin" />
            : <Play className="mr-1 h-4 w-4" />}
          {t('amendments.decision.startReview')}
        </Button>
      )}
      {changeStatus.enabled && (
        <Button variant="outline" size="sm" onClick={changeStatus.onClick}>
          {t('amendments.decision.statusDialog.action')}
        </Button>
      )}
    </div>
  )
}
