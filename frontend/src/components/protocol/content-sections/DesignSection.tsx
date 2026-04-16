import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

interface DesignSectionProps {
  design: ProtocolWorkingContent['design']
}

export function DesignSection({ design }: DesignSectionProps) {
  const { t } = useTranslation()

  if (!design.procedures && !design.anesthesia && !design.pain && !design.endpoints) {
    return null
  }

  return (
    <section className="mb-8 border-t pt-6 section-4" data-section={t('protocols.content.sections.design')}>
      <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.design')}</h2>

      {design.procedures && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.procedures')}</h3>
          <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{design.procedures}</p>
        </div>
      )}

      {design.anesthesia && design.anesthesia.is_under_anesthesia !== null && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.anesthesia')}</h3>
          <p className="text-sm">{design.anesthesia.is_under_anesthesia ? t('protocols.content.sections.sterileYes') : t('protocols.content.sections.sterileNo')}</p>
          {design.anesthesia.anesthesia_type && (
            <p className="text-sm mt-2">{t('protocols.content.sections.anesthesiaType')}: {design.anesthesia.anesthesia_type}</p>
          )}
        </div>
      )}

      {design.pain && design.pain.category && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.painCategory')}</h3>
          <p className="text-sm">{design.pain.category}</p>
          {design.pain.relief_measures && (
            <div className="mt-2">
              <p className="text-sm font-medium mb-1">{t('protocols.content.sections.painManagement')}</p>
              <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{design.pain.relief_measures}</p>
              {design.pain.relief_drug_name && (
                <p className="text-sm mt-1">{design.pain.relief_drug_name}</p>
              )}
              {design.pain.no_relief_justification && (
                <p className="text-sm mt-1">{design.pain.no_relief_justification}</p>
              )}
            </div>
          )}
        </div>
      )}

      {design.endpoints && (
        <div className="mb-4">
          {design.endpoints.experimental_endpoint && (
            <div className="mb-3">
              <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.experimentalEndpoint')}</h3>
              <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{design.endpoints.experimental_endpoint}</p>
            </div>
          )}
          {design.endpoints.humane_endpoint && (
            <div>
              <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.humaneEndpoint')}</h3>
              <p className="text-sm whitespace-pre-wrap bg-muted p-3 rounded">{design.endpoints.humane_endpoint}</p>
            </div>
          )}
        </div>
      )}
    </section>
  )
}
