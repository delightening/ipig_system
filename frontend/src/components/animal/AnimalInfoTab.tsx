import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'

import { Animal, AnimalStatus, animalFieldCorrectionApi } from '@/lib/api'
import { animalSpeciesLabel } from '@/lib/animalSpecies'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import { Edit2, FileEdit } from 'lucide-react'
import { GuestHide } from '@/components/ui/guest-hide'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { uiLocale } from '@/lib/utils'
import { RequestCorrectionDialog } from './RequestCorrectionDialog'

const getPenLocationDisplay = (animal: { status: AnimalStatus; pen_location?: string | null }, t: TFunction) => {
  if (animal.status === 'completed' && !animal.pen_location) {
    return t('animals.sacrificed')
  }
  return animal.pen_location || '-'
}

interface AnimalInfoTabProps {
  animal: Animal
}

export function AnimalInfoTab({ animal }: AnimalInfoTabProps) {
  const { t } = useTranslation()
  const [correctionDialogOpen, setCorrectionDialogOpen] = useState(false)
  const queryClient = useQueryClient()

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>{t('animalDetail.tabs.info')}</CardTitle>
          <CardDescription>{t('animalActions.common.animalBasicInfo')}</CardDescription>
        </div>
        <GuestHide>
          <div className="flex gap-2">
            <Button
              variant="outline"
              onClick={() => setCorrectionDialogOpen(true)}
              className="text-status-warning-text border-status-warning-border hover:bg-status-warning-bg"
            >
              <FileEdit className="h-4 w-4 mr-2" />
              {t('animalActions.info.requestCorrection')}
            </Button>
            <Can permission={PERMISSIONS.ANIMAL_ANIMAL_EDIT}>
              <Button className="bg-status-purple-solid hover:bg-status-purple-solid/90 text-white" asChild>
                <Link to={`/animals/${animal.id}/edit`}>
                  <Edit2 className="h-4 w-4 mr-2" />
                  {t('common.edit')}
                </Link>
              </Button>
            </Can>
          </div>
        </GuestHide>
      </CardHeader>

      <RequestCorrectionDialog
        open={correctionDialogOpen}
        onOpenChange={setCorrectionDialogOpen}
        animal={animal}
        onSubmit={async (data) => {
          await animalFieldCorrectionApi.create(animal.id, data)
          queryClient.invalidateQueries({ queryKey: ['animal', animal.id] })
          toast({ title: t('common.success'), description: t('animalActions.info.correctionSubmitted') })
        }}
      />
      <CardContent>
        <div className="grid grid-cols-2 gap-x-8 gap-y-4">
          <div>
            <Label className="text-muted-foreground">{t('animals.earTag')}</Label>
            <p className="font-medium">{animal.ear_tag}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animals.tabGroups.status')}</Label>
            <p className="font-medium">{t(`animals.statusLabels.${animal.status}`)}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animals.entryDate')}</Label>
            <p className="font-medium">{new Date(animal.entry_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animals.breed')}</Label>
            <p className="font-medium">{animalSpeciesLabel(animal, (breed) => t(`animals.breedLabels.${breed}`))}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animalActions.info.source')}</Label>
            <p className="font-medium">{animal.source_name || '-'}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animalActions.common.entryWeightKg')}</Label>
            <p className="font-medium">{animal.entry_weight || '-'}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animals.gender')}</Label>
            <p className="font-medium">{t(`animals.genderLabels.${animal.gender}`)}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animals.birthDate')}</Label>
            <p className="font-medium">
              {animal.birth_date ? new Date(animal.birth_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' }) : '-'}
            </p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animalActions.info.preExperimentCode')}</Label>
            <p className="font-medium">{animal.pre_experiment_code || '-'}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">IACUC No.</Label>
            <p className="font-medium">{animal.iacuc_no || '-'}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animalActions.info.experimentDate')}</Label>
            <p className="font-medium">
              {animal.experiment_date ? new Date(animal.experiment_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' }) : '-'}
            </p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animals.pen')}</Label>
            <p className="font-medium">{getPenLocationDisplay(animal, t)}</p>
          </div>
          <div className="col-span-2">
            <Label className="text-muted-foreground">{t('animalActions.common.notes')}</Label>
            <p className="font-medium">{animal.remark || '-'}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animals.systemNo')}</Label>
            <p className="font-medium" title={animal.id}>{animal.id.slice(0, 8)}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">{t('animalActions.info.createdAt')}</Label>
            <p className="font-medium">{new Date(animal.created_at).toLocaleString(uiLocale(), { timeZone: 'Asia/Taipei' })}</p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
