import { Label } from '@/components/ui/label'
import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

type AnimalEntry = ProtocolWorkingContent['animals']['animals'][number]

interface AnimalsSectionProps {
  animals: ProtocolWorkingContent['animals']
}

export function AnimalsSection({ animals }: AnimalsSectionProps) {
  const { t } = useTranslation()

  if (!animals.animals || animals.animals.length === 0) {
    return null
  }

  return (
    <section className="mb-8 border-t pt-6 section-7" data-section={t('protocols.content.sections.animals')}>
      <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.animals')}</h2>

      <div className="space-y-4">
        {animals.animals.map((animal: AnimalEntry, index: number) => (
          <div key={index} className="p-4 border rounded bg-muted">
            <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.animalGroup')} #{index + 1}</h3>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <Label className="font-semibold">{t('protocols.content.sections.species')}</Label>
                <p className="mt-1">{animal.species || '-'}{animal.species_other ? ` (${animal.species_other})` : ''}</p>
              </div>
              {animal.strain && (
                <div>
                  <Label className="font-semibold">{t('protocols.content.sections.strain')}</Label>
                  <p className="mt-1">{animal.strain}{animal.strain_other ? ` (${animal.strain_other})` : ''}</p>
                </div>
              )}
              <div>
                <Label className="font-semibold">{t('protocols.content.sections.sex')}</Label>
                <p className="mt-1">{animal.sex || '-'}</p>
              </div>
              <div>
                <Label className="font-semibold">{t('protocols.content.sections.number')}</Label>
                <p className="mt-1">{animal.number || '-'}</p>
              </div>
              <div>
                <Label className="font-semibold">{t('protocols.content.sections.ageRange')}</Label>
                <p className="mt-1">
                  {animal.age_unlimited ? t('protocols.content.sections.unlimited') : `${animal.age_min || t('protocols.content.sections.unlimited')} ~ ${animal.age_max || t('protocols.content.sections.unlimited')}`}
                </p>
              </div>
              <div>
                <Label className="font-semibold">{t('protocols.content.sections.weightRange')}</Label>
                <p className="mt-1">
                  {animal.weight_unlimited ? t('protocols.content.sections.unlimited') : `${animal.weight_min || t('protocols.content.sections.unlimited')}kg ~ ${animal.weight_max || t('protocols.content.sections.unlimited')}kg`}
                </p>
              </div>
              {animal.housing_location && (
                <div className="col-span-2">
                  <Label className="font-semibold">{t('protocols.content.sections.housingLocation')}</Label>
                  <p className="mt-1">{animal.housing_location}</p>
                </div>
              )}
            </div>
          </div>
        ))}

        {animals.total_animals && (
          <div className="mt-4 p-4 bg-status-info-bg rounded">
            <Label className="text-lg font-semibold">{t('protocols.content.sections.totalAnimals')}: {animals.total_animals}</Label>
          </div>
        )}
      </div>
    </section>
  )
}
