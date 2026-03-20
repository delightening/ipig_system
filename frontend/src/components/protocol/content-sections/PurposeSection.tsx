import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

interface PurposeSectionProps {
  purpose: ProtocolWorkingContent['purpose']
}

export function PurposeSection({ purpose }: PurposeSectionProps) {
  const { t } = useTranslation()

  if (!purpose.significance && !purpose.replacement && !purpose.reduction) {
    return null
  }

  return (
    <section className="mb-8 border-t pt-6 section-2" data-section={t('protocols.content.sections.purpose')}>
      <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.purpose')}</h2>

      {purpose.significance && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.significance')}</h3>
          <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.significance}</p>
        </div>
      )}

      {purpose.replacement && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.replacement')}</h3>

          {purpose.replacement.rationale && (
            <div className="mb-3">
              <h4 className="text-base font-medium mb-1">{t('protocols.content.sections.replacementRationale')}</h4>
              <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.replacement.rationale}</p>
            </div>
          )}

          {purpose.replacement.alt_search && (
            <div className="mb-3">
              <h4 className="text-base font-medium mb-1">{t('protocols.content.sections.alternativeSearch')}</h4>
              {purpose.replacement.alt_search.platforms && purpose.replacement.alt_search.platforms.length > 0 && (
                <ul className="list-disc list-inside text-sm mb-2">
                  {purpose.replacement.alt_search.platforms.map((p: string, idx: number) => (
                    <li key={idx}>{p}</li>
                  ))}
                </ul>
              )}
              {purpose.replacement.alt_search.keywords && (
                <p className="text-sm mb-2"><strong>{t('protocols.content.sections.searchKeywords')}: </strong>{purpose.replacement.alt_search.keywords}</p>
              )}
              {purpose.replacement.alt_search.conclusion && (
                <div>
                  <p className="text-sm font-medium mb-1">{t('protocols.content.sections.searchConclusion')}:</p>
                  <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.replacement.alt_search.conclusion}</p>
                </div>
              )}
            </div>
          )}

          {purpose.duplicate && (
            <div className="mb-3">
              <h4 className="text-base font-medium mb-1">{t('protocols.content.sections.duplicate')}</h4>
              <p className="text-sm mb-2">{purpose.duplicate.experiment ? t('protocols.content.sections.duplicateYes') : t('protocols.content.sections.duplicateNo')}</p>
              {purpose.duplicate.experiment && purpose.duplicate.justification && (
                <div>
                  <p className="text-sm font-medium mb-1">{t('protocols.content.sections.duplicateJustification')}</p>
                  <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.duplicate.justification}</p>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {purpose.reduction && purpose.reduction.design && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.reduction')}</h3>
          <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.reduction.design}</p>
        </div>
      )}
    </section>
  )
}
