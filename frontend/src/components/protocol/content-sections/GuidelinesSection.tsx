import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

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
              <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{guidelines.content}</p>
            </div>
          )}

          {guidelines.databases && guidelines.databases.some((db: GuidelineDatabase) => db.checked) && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-3">{t('aup.guidelines.databasesTitle')}</h3>
              <ul className="space-y-2">
                {guidelines.databases.filter((db: GuidelineDatabase) => db.checked).map((db: GuidelineDatabase) => (
                  <li key={db.code} className="text-sm p-2 bg-slate-50 rounded">
                    <span className="font-medium">{db.code}. {t(`aup.guidelines.databases.${db.code}`)}</span>
                    {db.keywords && (
                      <span className="ml-2 text-muted-foreground">
                        — {t('aup.guidelines.keywordsLabel')}: {db.keywords}
                      </span>
                    )}
                    {db.note && (
                      <p className="mt-1 text-muted-foreground ml-4">{db.note}</p>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {guidelines.references && guidelines.references.length > 0 && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-3">{t('aup.guidelines.referencesTitle')}</h3>
              <ol className="list-decimal list-inside space-y-2">
                {guidelines.references.map((ref: GuidelineReference, index: number) => (
                  <li key={index} className="text-sm">
                    {ref.citation || '-'}
                    {ref.url && (
                      <span className="text-blue-600 ml-2">
                        <a href={ref.url} target="_blank" rel="noopener noreferrer">{ref.url}</a>
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
