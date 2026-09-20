import { useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { AnimalStatus, animalStatusNames, animalFieldCorrectionApi } from '@/lib/api'
import type { AnimalEditFormData } from './hooks/useAnimalEdit'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import { ArrowLeft, Loader2, Save, AlertCircle, FileEdit } from 'lucide-react'
import { RequestCorrectionDialog } from '@/components/animal/RequestCorrectionDialog'
import { SearchableSelect } from '@/components/ui/searchable-select'
import { AnimalEditReadOnlyFields } from './components/AnimalEditReadOnlyFields'
import { useAnimalEdit } from './hooks/useAnimalEdit'

export function AnimalEditPage() {
  const { t } = useTranslation()
  const { id } = useParams<{ id: string }>()
  const animalId = id!
  const [correctionDialogOpen, setCorrectionDialogOpen] = useState(false)

  const { form, animal, animalLoading, sources, pens, approvedProtocols, updateMutation, navigate, queryClient } = useAnimalEdit(animalId)
  const { register, handleSubmit, watch, setValue, formState: { errors, isDirty } } = form

  const watchedStatus = watch('status')
  const watchedIacucNo = watch('iacuc_no')
  const watchedPenLocation = watch('pen_location')

  const penOptions = (pens ?? []).map((p) => ({
    value: p.code,
    label: p.code,
    description: p.name ?? undefined,
  }))

  const onValid = (data: AnimalEditFormData) => {
    if (data.pen_location && pens && !pens.some((p) => p.code === data.pen_location)) {
      form.setError('pen_location', { message: t('animalPages.editPage.penNotExist') })
      return
    }
    updateMutation.mutate(data)
  }

  if (animalLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!animal) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px]">
        <AlertCircle className="h-12 w-12 text-muted-foreground mb-4" />
        <p className="text-muted-foreground">{t('animalDetail.notFound')}</p>
        <Button variant="outline" className="mt-4" onClick={() => navigate('/animals')}>{t('animalDetail.backToList')}</Button>
      </div>
    )
  }

  return (
    <div className="space-y-6 max-w-4xl mx-auto">
      <div className="flex items-center justify-between">
        <Link to={`/animals/${animalId}`} className="inline-flex items-center text-muted-foreground hover:text-foreground">
          <ArrowLeft className="h-4 w-4 mr-2" />
          {t('animalPages.editPage.backToDetail')}
        </Link>
      </div>

      <PageHeader
        title={t('animalPages.editPage.title')}
        description={t('animalPages.editPage.description', { earTag: animal.ear_tag })}
        actions={
          <Button size="sm" type="button" variant="outline" onClick={() => setCorrectionDialogOpen(true)}
            className="text-status-warning-text border-status-warning-border hover:bg-status-warning-bg">
            <FileEdit className="h-4 w-4 mr-2" />
            {t('animalPages.editPage.requestCorrection')}
          </Button>
        }
      />

      <RequestCorrectionDialog
        open={correctionDialogOpen}
        onOpenChange={setCorrectionDialogOpen}
        animal={animal}
        onSubmit={async (data) => {
          await animalFieldCorrectionApi.create(animalId, data)
          queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
          toast({ title: t('common.success'), description: t('animalPages.editPage.correctionSubmitted') })
        }}
      />

      <form onSubmit={handleSubmit(onValid)}>
        <Card>
          <CardHeader>
            <CardTitle>{t('animalPages.editPage.basicInfo')}</CardTitle>
            <CardDescription>{t('animalPages.editPage.basicInfoDescription')}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-6">
              <AnimalEditReadOnlyFields animal={animal} sources={sources} />

              {/* 狀態 */}
              <div className="space-y-2">
                <Label>{t('animals.status')} *</Label>
                <input type="hidden" {...register('status', { required: t('animalPages.editPage.statusRequired') })} />
                <Select value={watchedStatus ?? ''} onValueChange={(v) => setValue('status', v as AnimalStatus, { shouldValidate: true, shouldDirty: true })}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {Object.entries(animalStatusNames)
                      .filter(([value]) => value !== 'transferred' && value !== 'sudden_death')
                      .map(([value]) => <SelectItem key={value} value={value}>{t(`animals.statusLabels.${value}`)}</SelectItem>)}
                  </SelectContent>
                </Select>
                {errors.status && <p className="text-sm text-destructive">{errors.status.message}</p>}
              </div>

              {/* 欄位 */}
              <div className="space-y-2">
                <Label>{t('animals.pen')}</Label>
                <SearchableSelect
                  options={penOptions}
                  value={watchedPenLocation ?? ''}
                  onValueChange={(v) => setValue('pen_location', v, { shouldValidate: true, shouldDirty: true })}
                  placeholder={t('animalPages.shared.selectPen')}
                  searchPlaceholder={t('animalPages.editPage.penSearchPlaceholder')}
                  emptyMessage={t('animalPages.editPage.penNotFoundCheck')}
                />
                {errors.pen_location && (
                  <p className="text-sm text-destructive">{errors.pen_location.message}</p>
                )}
              </div>

              {/* IACUC No. */}
              <div className="space-y-2">
                <Label>
                  IACUC No.
                  {watchedStatus === 'in_experiment' && <span className="text-destructive ml-1">*</span>}
                </Label>
                <input
                  type="hidden"
                  {...register('iacuc_no', {
                    validate: (value, formValues) =>
                      !(formValues.status === 'in_experiment' && !value) ||
                      t('animalPages.editPage.iacucRequiredInExperiment'),
                  })}
                />
                {animal.status === 'in_experiment' ? (
                  <>
                    <Input value={animal.iacuc_no || ''} disabled className="bg-muted" />
                    <p className="text-xs text-status-warning-text">{t('animalPages.editPage.iacucLocked')}</p>
                  </>
                ) : (
                  <Select value={watchedIacucNo || ''} onValueChange={(v) => setValue('iacuc_no', v === '' ? '' : v, { shouldValidate: true, shouldDirty: true })}>
                    <SelectTrigger><SelectValue placeholder={t('animalPages.editPage.selectIacuc')} /></SelectTrigger>
                    <SelectContent>
                      {approvedProtocols?.map((protocol) => (
                        <SelectItem key={protocol.id} value={protocol.iacuc_no!}>{protocol.iacuc_no}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
                {errors.iacuc_no && <p className="text-sm text-destructive">{errors.iacuc_no.message}</p>}
              </div>

              <div className="space-y-2">
                <Label>{t('animalPages.editPage.experimentDate')}</Label>
                <Input type="date" {...register('experiment_date')} />
              </div>

              {/* 備註 */}
              <div className="space-y-2 col-span-2">
                <Label htmlFor="remark">{t('animalPages.shared.remark')}</Label>
                <Textarea id="remark" {...register('remark')} placeholder={t('animalPages.shared.otherRemarks')} className="min-h-[100px]" />
              </div>
            </div>
          </CardContent>
        </Card>

        <div className="flex items-center justify-end gap-3 mt-6">
          <Button type="button" variant="outline" onClick={() => navigate(`/animals/${animalId}`)}>{t('common.cancel')}</Button>
          <Button type="submit" disabled={updateMutation.isPending || !isDirty} className="bg-primary hover:bg-primary/90">
            {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            <Save className="h-4 w-4 mr-2" />
            {t('animalPages.editPage.saveChanges')}
          </Button>
        </div>
      </form>
    </div>
  )
}
