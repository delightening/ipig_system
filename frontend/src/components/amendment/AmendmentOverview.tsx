import { useTranslation } from 'react-i18next'

import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { formatDateTime } from '@/lib/utils'
import { amendmentStatusColors } from '@/types/amendment'
import type { Amendment, AmendmentChangeContent } from '@/types/amendment'
import { AmendmentChangeDetail } from './AmendmentChangeDetail'

interface AmendmentOverviewProps {
  amendment: Amendment
}

/**
 * 變更申請概覽（Phase 1）：目標（標題 + 說明）、結構化變更明細、變更項目 Badge、狀態 / 類型 Badge。
 * 變更項目標籤對應 MyAmendmentsPage 的 i18n 慣例（後端可能回小寫，轉大寫查 key）。
 */
export function AmendmentOverview({ amendment }: AmendmentOverviewProps) {
  const { t } = useTranslation()

  const changeItems = amendment.change_items ?? []
  // R71-12：結構化變更明細（目的 + 項次/前後對照），存於 changes_content；舊資料無則略過。
  const changeContent = amendment.changes_content as AmendmentChangeContent | undefined

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>{amendment.title}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant={amendmentStatusColors[amendment.status]}>
              {t(`amendments.status.${amendment.status}`)}
            </Badge>
            <Badge variant="outline">{t(`amendments.types.${amendment.amendment_type}`)}</Badge>
            {amendment.is_historical && (
              <Badge variant="outline">{t('amendments.decision.historicalTag')}</Badge>
            )}
            {amendment.effective_from && (
              <span className="text-xs text-muted-foreground">
                {t('amendments.decision.effectiveSince', {
                  date: formatDateTime(amendment.effective_from),
                })}
              </span>
            )}
          </div>

          {amendment.description ? (
            <p className="text-sm text-muted-foreground leading-relaxed whitespace-pre-wrap break-words">
              {amendment.description}
            </p>
          ) : (
            <p className="text-sm text-muted-foreground">{t('amendments.decision.noDescription')}</p>
          )}
        </CardContent>
      </Card>

      <AmendmentChangeDetail content={changeContent} />

      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t('amendments.columns.changeItems')}</CardTitle>
        </CardHeader>
        <CardContent>
          {changeItems.length > 0 ? (
            <div className="flex flex-wrap gap-2">
              {changeItems.map((item) => (
                <Badge key={item} variant="secondary">
                  {t(`amendments.changeItemLabels.${item.toUpperCase()}`, { defaultValue: item })}
                </Badge>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">{t('amendments.decision.noChangeItems')}</p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
