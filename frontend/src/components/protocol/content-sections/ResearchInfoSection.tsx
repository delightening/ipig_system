import { Label } from '@/components/ui/label'
import { formatDate } from '@/lib/utils'
import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

interface ResearchInfoSectionProps {
  basic: ProtocolWorkingContent['basic']
  protocolTitle: string
  startDate?: string
  endDate?: string
}

export function ResearchInfoSection({ basic, protocolTitle, startDate, endDate }: ResearchInfoSectionProps) {
  const { t } = useTranslation()

  return (
    <section className="mb-8 section-1" data-section={t('protocols.content.sections.researchInfo')}>
      <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.researchInfo')}</h2>

      <div className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <Label className="text-sm font-semibold">{t('protocols.content.sections.glpAttribute')}</Label>
            <p className="mt-1">{basic.is_glp ? t('protocols.content.sections.glpCompliant') : t('protocols.content.sections.glpNonCompliant')}</p>
            {basic.is_glp && basic.registration_authorities?.length > 0 && (
              <div className="mt-2">
                <Label className="text-sm font-semibold">{t('aup.basic.registrationAuthorities')}</Label>
                <p className="mt-1">
                  {basic.registration_authorities.map((auth: string) =>
                    t(`aup.basic.registrationAuthorityOptions.${auth}`)
                  ).join('、')}
                  {basic.registration_authority_other && ` (${basic.registration_authority_other})`}
                </p>
              </div>
            )}
          </div>
          <div>
            <Label className="text-sm font-semibold">{t('protocols.content.sections.projectName')}</Label>
            <p className="mt-1">{protocolTitle || '-'}</p>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <Label className="text-sm font-semibold">{t('protocols.content.sections.projectType')}</Label>
            <p className="mt-1">
              {basic.project_type ? t(`aup.projectTypes.${basic.project_type}`) : '-'}
              {basic.project_type_other && ` (${basic.project_type_other})`}
            </p>
          </div>
          <div>
            <Label className="text-sm font-semibold">{t('protocols.content.sections.projectCategory')}</Label>
            <p className="mt-1">
              {basic.project_category ? t(`aup.projectCategories.${basic.project_category}`) : '-'}
              {basic.project_category_other && ` (${basic.project_category_other})`}
            </p>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <Label className="text-sm font-semibold">{t('protocols.content.sections.expectedSchedule')}</Label>
            <p className="mt-1">
              {(startDate || basic.start_date) && (endDate || basic.end_date)
                ? `${formatDate(startDate || basic.start_date)} ~ ${formatDate(endDate || basic.end_date)}`
                : '-'}
            </p>
          </div>
        </div>

        {/* PI Info */}
        {basic.pi && (
          <div className="border-t pt-4 mt-4">
            <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.piInfo')}</h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.piName')}</Label>
                <p className="mt-1">{basic.pi.name || '-'}</p>
              </div>
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.piPhone')}</Label>
                <p className="mt-1">
                  {basic.pi.phone || '-'}
                  {basic.pi.phone_ext && ` #${basic.pi.phone_ext}`}
                </p>
              </div>
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.piEmail')}</Label>
                <p className="mt-1">{basic.pi.email || '-'}</p>
              </div>
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.piAddress')}</Label>
                <p className="mt-1">{basic.pi.address || '-'}</p>
              </div>
            </div>
          </div>
        )}

        {/* Sponsor Info */}
        {basic.sponsor && (
          <div className="border-t pt-4 mt-4">
            <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.sponsorInfo')}</h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.sponsorName')}</Label>
                <p className="mt-1">{basic.sponsor.name || '-'}</p>
              </div>
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.contactPerson')}</Label>
                <p className="mt-1">{basic.sponsor.contact_person || '-'}</p>
              </div>
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.contactPhone')}</Label>
                <p className="mt-1">
                  {basic.sponsor.contact_phone || '-'}
                  {basic.sponsor.contact_phone_ext && ` #${basic.sponsor.contact_phone_ext}`}
                </p>
              </div>
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.contactEmail')}</Label>
                <p className="mt-1">{basic.sponsor.contact_email || '-'}</p>
              </div>
            </div>
          </div>
        )}

        {/* Institution and Facilities */}
        <div className="border-t pt-4 mt-4">
          <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.facility')}</h3>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label className="text-sm font-semibold">{t('protocols.content.sections.facilityName')}</Label>
              <p className="mt-1">{basic.facility?.title || '-'}</p>
            </div>
            <div>
              <Label className="text-sm font-semibold">{t('protocols.content.sections.location')}</Label>
              <p className="mt-1">{basic.housing_location || '-'}</p>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
