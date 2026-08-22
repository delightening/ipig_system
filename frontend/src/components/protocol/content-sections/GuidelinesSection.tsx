import { useTranslation } from 'react-i18next'

import { safeHref } from '@/lib/sanitize'
import type { ProtocolWorkingContent } from '@/types/protocol'

import { CheckIndicator } from './ChoiceList'
import { GUIDELINE_DATABASE_CODES } from '@/lib/constants/protocolChoiceOptions'

type GuidelineDatabase = ProtocolWorkingContent['guidelines']['databases'][number]
type GuidelineReference = ProtocolWorkingContent['guidelines']['references'][number]

interface GuidelinesSectionProps {
  guidelines: ProtocolWorkingContent['guidelines']
}

export function GuidelinesSection({ guidelines }: GuidelinesSectionProps) {
  const { t } = useTranslation()

  const hasContent = guidelines.content ||
    (guidelines.databases && guidelines.databases.some((db: GuidelineDatabase) => db.checked)) ||
    (guidelines.references && guidelines.references.length > 0)

  return (
    <section className="mb-8 border-t pt-6 section-5" data-section={t('protocols.content.sections.guidelines')}>
      <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.guidelines')}</h2>

      {hasContent ? (
        <>
          {guidelines.content && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.guidelinesContent')}</h3>
              <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{guidelines.content}</p>
            </div>
          )}

          {/* 文獻查詢資料庫 A–L：顯示完整清單 + 已勾選/未勾選（讓審查員看見哪些查了、哪些沒查） */}
          <div className="mb-4">
            <h3 className="text-lg font-semibold mb-3">{t('aup.guidelines.databasesTitle')}</h3>
            <ul className="space-y-1 text-sm">
              {GUIDELINE_DATABASE_CODES.map((code) => {
                const db = guidelines.databases?.find((d: GuidelineDatabase) => d.code === code)
                const on = !!db?.checked
                return (
                  <li key={code} className={`flex items-start gap-2 ${on ? 'text-foreground' : 'text-muted-foreground/60'}`}>
                    <CheckIndicator on={on} />
                    <span className={on ? 'font-medium' : ''}>
                      {code}. {t(`aup.guidelines.databases.${code}`)}
                      <span className="sr-only">{on ? '（已選擇）' : '（未選擇）'}</span>
                      {on && db?.keywords && (
                        <span className="ml-2 font-normal text-muted-foreground">— {t('aup.guidelines.keywordsLabel')}: {db.keywords}</span>
                      )}
                      {on && db?.note && (
                        <span className="block font-normal text-muted-foreground ml-4">{db.note}</span>
                      )}
                    </span>
                  </li>
                )
              })}
            </ul>
          </div>

          {guidelines.references && guidelines.references.length > 0 && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-3">{t('aup.guidelines.referencesTitle')}</h3>
              <ol className="list-decimal list-inside space-y-2">
                {guidelines.references.map((ref: GuidelineReference, index: number) => (
                  <li key={index} className="text-sm">
                    {ref.citation || '-'}
                    {ref.url && (
                      <span className="text-status-info-text ml-2">
                        {safeHref(ref.url) ? (
                          <a href={safeHref(ref.url)} target="_blank" rel="noopener noreferrer">{ref.url}</a>
                        ) : (
                          <span>{ref.url}</span>
                        )}
                      </span>
                    )}
                  </li>
                ))}
              </ol>
            </div>
          )}
        </>
      ) : (
        <p className="text-sm text-muted-foreground">{t('protocols.content.sections.omitted')}</p>
      )}
    </section>
  )
}
