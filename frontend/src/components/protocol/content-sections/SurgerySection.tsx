import { Label } from '@/components/ui/label'
import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

type SurgeryDrug = ProtocolWorkingContent['surgery']['drugs'][number]

interface SurgerySectionProps {
  design: ProtocolWorkingContent['design']
  surgery: ProtocolWorkingContent['surgery']
}

export function SurgerySection({ design, surgery }: SurgerySectionProps) {
  const { t } = useTranslation()

  const needsSurgeryPlan = design.anesthesia?.is_under_anesthesia === true &&
    (design.anesthesia?.anesthesia_type === 'survival_surgery' || design.anesthesia?.anesthesia_type === 'non_survival_surgery')

  return (
    <section className="mb-8 border-t pt-6 section-6" data-section={t('protocols.content.sections.surgery')}>
      <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.surgery')}</h2>

      {needsSurgeryPlan ? (
        <>
          {surgery.surgery_type && (
            <div className="mb-4">
              <Label className="text-sm font-semibold">{t('protocols.content.sections.surgeryType')}</Label>
              <p className="mt-1">{surgery.surgery_type}</p>
            </div>
          )}

          {surgery.preop_preparation && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.preop_Preparation')}</h3>
              <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.preop_preparation}</p>
            </div>
          )}

          {surgery.surgery_description && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.surgeryDescription')}</h3>
              <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.surgery_description}</p>
            </div>
          )}

          {surgery.monitoring && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.monitoring')}</h3>
              <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.monitoring}</p>
            </div>
          )}

          {surgery.postop_care && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.postopCare')}</h3>
              <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.postop_care}</p>
            </div>
          )}

          {surgery.drugs && surgery.drugs.length > 0 && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.drugPlan')}</h3>
              <div className="space-y-2">
                {surgery.drugs.map((drug: SurgeryDrug, index: number) => (
                  <div key={index} className="p-3 border rounded bg-slate-50">
                    <p className="text-sm font-medium">{drug.drug_name || '-'}</p>
                    <p className="text-xs text-muted-foreground">
                      {t('protocols.content.sections.dose')}: {drug.dose || '-'} | {t('protocols.content.sections.route')}: {drug.route || '-'} | {t('protocols.content.sections.frequency')}: {drug.frequency || '-'} | {t('protocols.content.sections.itemPurpose')}: {drug.purpose || '-'}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </>
      ) : (
        <p className="text-sm text-muted-foreground">{t('protocols.content.sections.omitted')}</p>
      )}
    </section>
  )
}
