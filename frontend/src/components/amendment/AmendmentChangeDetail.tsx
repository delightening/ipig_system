import { useTranslation } from 'react-i18next'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { AmendmentChangeContent } from '@/types/amendment'

interface AmendmentChangeDetailProps {
  content?: AmendmentChangeContent
}

/**
 * R71-12 結構化變更明細卡：目的 + 逐項前後對照。
 * changes_content 為彈性 JSON，items 非陣列時以空陣列保護（Array.isArray）。
 * 無目的且無項次時不渲染（回傳 null），由舊資料的標題 / 變更項目 Badge 承接。
 */
export function AmendmentChangeDetail({ content }: AmendmentChangeDetailProps) {
  const { t } = useTranslation()

  const detailItems = Array.isArray(content?.items) ? content.items : []
  if (!content?.purpose && detailItems.length === 0) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{t('amendments.decision.changeDetailTitle')}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {content?.purpose && (
          <div>
            <p className="text-xs font-medium text-muted-foreground mb-1">
              {t('amendments.decision.changePurpose')}
            </p>
            <p className="text-sm whitespace-pre-wrap break-words">{content.purpose}</p>
          </div>
        )}
        {detailItems.map((item, idx) => (
          <div key={idx} className="rounded-md border p-3 space-y-2">
            <p className="text-xs font-mono text-muted-foreground">
              {t('amendments.decision.section')} {item.section || '—'}
            </p>
            <div className="grid gap-3 sm:grid-cols-2">
              <div>
                <p className="text-xs text-muted-foreground mb-0.5">{t('amendments.decision.before')}</p>
                <p className="text-sm whitespace-pre-wrap break-words">{item.before || '—'}</p>
              </div>
              <div>
                <p className="text-xs text-muted-foreground mb-0.5">{t('amendments.decision.after')}</p>
                <p className="text-sm whitespace-pre-wrap break-words">{item.after || '—'}</p>
              </div>
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
