import { Label } from '@/components/ui/label'
import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

type PersonnelMember = ProtocolWorkingContent['personnel'][number]

interface PersonnelSectionProps {
  personnel: ProtocolWorkingContent['personnel']
}

export function PersonnelSection({ personnel }: PersonnelSectionProps) {
  const { t } = useTranslation()

  if (!personnel || personnel.length === 0) {
    return null
  }

  return (
    <section className="mb-8 border-t pt-6 section-8" data-section={t('protocols.content.sections.personnel')}>
      <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.personnel')}</h2>

      <div className="space-y-4">
        {personnel.map((person: PersonnelMember, index: number) => (
          <div key={index} className="p-4 border rounded bg-slate-50">
            <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.person')} #{index + 1}</h3>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <Label className="font-semibold">{t('protocols.content.sections.piName')}</Label>
                <p className="mt-1">{person.name || '-'}</p>
              </div>
              <div>
                <Label className="font-semibold">{t('protocols.content.sections.position')}</Label>
                <p className="mt-1">{person.position || '-'}</p>
              </div>
              <div>
                <Label className="font-semibold">{t('protocols.content.sections.yearsExperience')}</Label>
                <p className="mt-1">{person.years_experience || '-'} {t('protocols.content.sections.years')}</p>
              </div>
              {person.roles && person.roles.length > 0 && (
                <div className="col-span-2">
                  <Label className="font-semibold">{t('protocols.content.sections.workContent')}</Label>
                  <div className="mt-1 flex flex-wrap gap-2">
                    {person.roles.map((role: string, roleIndex: number) => (
                      <span key={roleIndex} className="px-2 py-1 bg-blue-100 rounded text-xs">{role}</span>
                    ))}
                  </div>
                </div>
              )}
              {person.trainings && person.trainings.length > 0 && (
                <div className="col-span-2">
                  <Label className="font-semibold">{t('protocols.content.sections.training')}</Label>
                  <div className="mt-1 flex flex-wrap gap-2">
                    {person.trainings.map((training: string, trainingIndex: number) => (
                      <span key={trainingIndex} className="px-2 py-1 bg-green-100 rounded text-xs">{training}</span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
    </section>
  )
}
