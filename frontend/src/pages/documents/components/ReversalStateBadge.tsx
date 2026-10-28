import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import type { ReversalDisplayState } from '../documentActionState'

/** 「已沖銷」「沖銷中」徽章；`state` 為空時不渲染。 */
export function ReversalStateBadge({
  state,
  className,
}: {
  state: ReversalDisplayState
  className?: string
}) {
  const { t } = useTranslation()
  if (!state) return null
  return (
    <Badge variant={state === 'reversed' ? 'destructive' : 'warning'} className={className}>
      {t(
        state === 'reversed'
          ? 'erpDocs.documents.reversal.badgeReversed'
          : 'erpDocs.documents.reversal.badgeReversing',
      )}
    </Badge>
  )
}
