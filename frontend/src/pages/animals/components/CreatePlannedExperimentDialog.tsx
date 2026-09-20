import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import { reservationPlanningApi } from '@/lib/api/reservationPlanning'
import { getApiErrorMessage } from '@/lib/apiError'
import { toast } from '@/components/ui/use-toast'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'

export function CreatePlannedExperimentDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (o: boolean) => void
}) {
  const { t } = useTranslation()
  const [unit, setUnit] = useState('')
  const [description, setDescription] = useState('')
  const [demand, setDemand] = useState('')
  const qc = useQueryClient()

  const create = useMutation({
    mutationFn: () =>
      reservationPlanningApi.createPlannedExperiment({
        unit: unit.trim(),
        description: description.trim() || null,
        demand_count: Number(demand) || 0,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['reservation-planning'] })
      toast({
        title: t('animalPages.reservation.createDialog.createdTitle'),
        description: t('animalPages.reservation.createDialog.createdDescription'),
      })
      setUnit('')
      setDescription('')
      setDemand('')
      onOpenChange(false)
    },
    onError: (e: unknown) =>
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(e, t('animalPages.shared.addFailed')),
        variant: 'destructive',
      }),
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="md">
        <DialogHeader>
          <DialogTitle>{t('animalPages.reservation.addPlanned')}</DialogTitle>
          <DialogDescription>{t('animalPages.reservation.createDialog.description')}</DialogDescription>
        </DialogHeader>
        <div className="space-y-3">
          <label className="block text-sm">
            <span className="mb-1 block text-muted-foreground">
              {t('animalPages.reservation.createDialog.client')} <span className="text-status-error-text">*</span>
            </span>
            <Input
              value={unit}
              onChange={(e) => setUnit(e.target.value)}
              placeholder={t('animalPages.reservation.createDialog.clientPlaceholder')}
            />
          </label>
          <label className="block text-sm">
            <span className="mb-1 block text-muted-foreground">{t('animalPages.reservation.createDialog.summary')}</span>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder={t('animalPages.reservation.createDialog.summaryPlaceholder')}
            />
          </label>
          <label className="block text-sm">
            <span className="mb-1 block text-muted-foreground">{t('animalPages.reservation.createDialog.demand')}</span>
            <Input
              type="number"
              min="0"
              value={demand}
              onChange={(e) => setDemand(e.target.value)}
              className="w-32"
            />
          </label>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button disabled={!unit.trim() || create.isPending} onClick={() => create.mutate()}>
            {t('animalPages.reservation.createDialog.submit')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
