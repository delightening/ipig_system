import { useState } from 'react'
import { useTranslation } from 'react-i18next'

import { uiLocale } from '@/lib/utils'
import { GuestHide } from '@/components/ui/guest-hide'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { AnimalSacrifice } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Plus, Edit2, Heart } from 'lucide-react'
import { SacrificeFormDialog } from './SacrificeFormDialog'
import { ByproductSamplesPanel } from './ByproductSamplesPanel'

interface SacrificeTabProps {
  animalId: string
  earTag: string
  sacrifice: AnimalSacrifice | undefined
}

export function SacrificeTab({ animalId, earTag, sacrifice }: SacrificeTabProps) {
  const { t } = useTranslation()
  const [showDialog, setShowDialog] = useState(false)

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>{t('animalDetail.tabs.sacrifice')}</CardTitle>
            <CardDescription>{t('animalActions.sacrifice.tab.description')}</CardDescription>
          </div>
          <GuestHide>
            <Can permission={PERMISSIONS.ANIMAL_RECORD_CREATE}>
              <Button
                className="bg-status-purple-solid hover:bg-status-purple-solid/90 text-white shrink-0"
                onClick={() => setShowDialog(true)}
              >
                {sacrifice ? (
                  <>
                    <Edit2 className="h-4 w-4 mr-2" />
                    {t('common.edit')}
                  </>
                ) : (
                  <>
                    <Plus className="h-4 w-4 mr-2" />
                    {t('animalActions.sacrifice.tab.createRecord')}
                  </>
                )}
              </Button>
            </Can>
          </GuestHide>
        </CardHeader>
        <CardContent>
          {!sacrifice ? (
            <div className="text-center py-12 text-muted-foreground">
              <Heart className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
              <p>{t('animalActions.sacrifice.tab.empty')}</p>
              <p className="text-sm mt-1">{t('animalActions.sacrifice.tab.emptyHint')}</p>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label className="text-muted-foreground">{t('animalActions.sacrifice.tab.sacrificeDate')}</Label>
                  <p className="font-medium">
                    {sacrifice.sacrifice_date
                      ? new Date(sacrifice.sacrifice_date).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })
                      : '-'
                    }
                  </p>
                </div>
                <div>
                  <Label className="text-muted-foreground">{t('animalActions.sacrifice.confirmedSacrifice')}</Label>
                  <p className="font-medium">
                    {sacrifice.confirmed_sacrifice ? (
                      <Badge className="bg-status-error-bg text-status-error-text">{t('animalActions.sacrifice.tab.confirmed')}</Badge>
                    ) : t('common.no')}
                  </p>
                </div>
                <div>
                  <Label className="text-muted-foreground">Zoletil-50 (ml)</Label>
                  <p className="font-medium">{sacrifice.zoletil_dose || '-'}</p>
                </div>
                <div>
                  <Label className="text-muted-foreground">{t('animalActions.sacrifice.tab.electrocution200')}</Label>
                  <p className="font-medium">{sacrifice.method_electrocution ? t('common.yes') : t('common.no')}</p>
                </div>
                <div>
                  <Label className="text-muted-foreground">{t('animalActions.sacrifice.bloodletting')}</Label>
                  <p className="font-medium">{sacrifice.method_bloodletting ? t('common.yes') : t('common.no')}</p>
                </div>
                <div>
                  <Label className="text-muted-foreground">{t('animalActions.sacrifice.tab.otherMethod')}</Label>
                  <p className="font-medium">{sacrifice.method_other || '-'}</p>
                </div>
                <div>
                  <Label className="text-muted-foreground">{t('animalActions.sacrifice.tab.sampling')}</Label>
                  <p className="font-medium">{sacrifice.sampling || '-'}</p>
                </div>
                <div>
                  <Label className="text-muted-foreground">{t('animalActions.sacrifice.tab.bloodSamplingMl')}</Label>
                  <p className="font-medium">{sacrifice.blood_volume_ml || '-'}</p>
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* R53-5 廢棄物再利用紀錄（byproduct samples）— 僅 view 權限可見 */}
      <ByproductSamplesPanel
        animalId={animalId}
        earTag={earTag}
        animalSacrificed={Boolean(sacrifice?.confirmed_sacrifice)}
      />

      <SacrificeFormDialog
        open={showDialog}
        onOpenChange={setShowDialog}
        animalId={animalId}
        earTag={earTag}
        sacrifice={sacrifice || undefined}
      />
    </>
  )
}
