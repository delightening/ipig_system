import React, { useState, useRef, useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api, {
  Animal,
  AnimalWeight,
  ProtocolListItem,
  facilityApi,
} from '@/lib/api'
import { animalSpeciesLabel } from '@/lib/animalSpecies'
import { Card, CardContent } from '@/components/ui/card'
import { SearchableSelect } from '@/components/ui/searchable-select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { uiLocale } from '@/lib/utils'

import { StatusHelpPopover } from '@/components/animal/StatusHelpPopover'
import { GuestHide } from '@/components/ui/guest-hide'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'

import { getPenLocationDisplay } from '../constants'

interface AnimalHeaderCardProps {
  animalId: string
  animal: Animal
  weights: AnimalWeight[] | undefined
  approvedProtocols: ProtocolListItem[] | undefined
  assignTrialMutation: ReturnType<typeof useMutation<unknown, unknown, string>>
}

export function AnimalHeaderCard({
  animalId,
  animal,
  weights,
  approvedProtocols,
  assignTrialMutation,
}: AnimalHeaderCardProps) {
  const { t } = useTranslation()
  const penLocation = getPenLocationDisplay(animal, t)

  return (
    <Card className="bg-gradient-to-r from-muted to-muted/50 border-border">
      <CardContent className="pt-6">
        <div className="grid grid-cols-3 gap-6">
          <LeftColumn animal={animal} animalId={animalId} penLocation={penLocation} />
          <MiddleColumn animal={animal} weights={weights} />
          <RightColumn
            animal={animal}
            approvedProtocols={approvedProtocols}
            assignTrialMutation={assignTrialMutation}
          />
        </div>
      </CardContent>
    </Card>
  )
}

function PenLocationField({ animal, animalId, penLocation }: { animal: Animal; animalId: string; penLocation: string }) {
  const { t } = useTranslation()
  const [editing, setEditing] = useState(false)
  const queryClient = useQueryClient()
  const containerRef = useRef<HTMLDivElement>(null)

  const { data: pens } = useQuery({
    queryKey: ['pens'],
    queryFn: async () => (await facilityApi.listPens()).data,
    staleTime: 600_000,
  })

  const mutation = useMutation({
    mutationFn: (penCode: string) =>
      // R30-B: 帶當前 version 防 lost update（從 prop 取得的 animal query 結果）
      api.put(`/animals/${animalId}`, { pen_location: penCode || null, version: animal.version }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      setEditing(false)
      toast({ title: t('common.success'), description: t('animalPages.headerCard.penUpdated') })
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(error, t('animalPages.shared.updateFailed')), variant: 'destructive' })
    },
  })

  useEffect(() => {
    if (!editing) return
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setEditing(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [editing])

  const penOptions = (pens ?? []).map((p) => ({
    value: p.code,
    label: p.code,
    description: p.name && p.name !== p.code ? p.name : undefined,
  }))

  if (editing) {
    return (
      <div ref={containerRef}>
        <SearchableSelect
          options={penOptions}
          value={animal.pen_location ?? ''}
          onValueChange={(v) => mutation.mutate(v)}
          placeholder={t('animalPages.shared.selectPen')}
          searchPlaceholder={t('animalPages.headerCard.penSearchPlaceholder')}
          emptyMessage={t('animalPages.headerCard.penNotFound')}
          className="w-36"
        />
      </div>
    )
  }

  return (
    <GuestHide>
      <Can permission={PERMISSIONS.ANIMAL_ANIMAL_EDIT} fallback={<span>{penLocation}</span>}>
        <button
          type="button"
          onClick={() => setEditing(true)}
          className="inline-flex items-center px-2.5 py-0.5 rounded-md bg-gray-500 text-white text-sm font-medium hover:bg-gray-600 transition-colors"
          title={t('animalPages.headerCard.clickToEditPen')}
        >
          {penLocation}
        </button>
      </Can>
    </GuestHide>
  )
}

function LeftColumn({ animal, animalId, penLocation }: { animal: Animal; animalId: string; penLocation: string }) {
  const { t } = useTranslation()
  return (
    <div className="space-y-3">
      <div>
        <span className="text-sm text-muted-foreground">{t('animals.earTag')}</span>
        <p className="text-2xl font-bold text-status-warning-text">{animal.ear_tag}</p>
      </div>
      <div>
        <span className="text-sm text-muted-foreground">{t('animalPages.headerCard.penNumber')}</span>
        <div className="mt-0.5">
          <PenLocationField animal={animal} animalId={animalId} penLocation={penLocation} />
        </div>
      </div>
      <div>
        <span className="text-sm text-muted-foreground">{t('animals.breed')}</span>
        <p className="font-medium">{animalSpeciesLabel(animal, (b) => t(`animals.breedLabels.${b}`))}</p>
      </div>
    </div>
  )
}

function MiddleColumn({
  animal,
  weights,
}: {
  animal: Animal
  weights: AnimalWeight[] | undefined
}) {
  const { t } = useTranslation()
  return (
    <div className="space-y-3">
      <div>
        <span className="text-sm text-muted-foreground">{t('animals.birthDate')}</span>
        <p className="font-medium">
          {animal.birth_date
            ? new Date(animal.birth_date).toLocaleDateString(uiLocale(), {
                timeZone: 'Asia/Taipei',
              })
            : '-'}
        </p>
      </div>
      <div>
        <span className="text-sm text-muted-foreground">IACUC No.</span>
        <p className="font-medium">{animal.iacuc_no || t('animals.notAssigned')}</p>
      </div>
      {animal.status !== 'unassigned' &&
        (animal.experiment_assigned_by_name || animal.experiment_date) && (
          <div>
            <span className="text-sm text-muted-foreground">{t('animalPages.headerCard.experimentAssignment')}</span>
            <p className="font-medium text-sm">
              {animal.experiment_assigned_by_name && (
                <span>{animal.experiment_assigned_by_name}</span>
              )}
              {animal.experiment_date && (
                <span className="text-muted-foreground ml-1">
                  (
                  {new Date(animal.experiment_date).toLocaleDateString(uiLocale(), {
                    timeZone: 'Asia/Taipei',
                  })}
                  )
                </span>
              )}
            </p>
          </div>
        )}
      <div>
        <span className="text-sm text-muted-foreground">{t('animalPages.headerCard.latestWeight')}</span>
        <p className="font-medium">
          {weights && weights.length > 0
            ? `${weights[0].weight} kg`
            : animal.entry_weight
              ? t('animalPages.headerCard.entryWeightValue', { weight: animal.entry_weight })
              : '-'}
        </p>
      </div>
    </div>
  )
}

function RightColumn({
  animal,
  approvedProtocols,
  assignTrialMutation,
}: {
  animal: Animal
  approvedProtocols: ProtocolListItem[] | undefined
  assignTrialMutation: ReturnType<typeof useMutation<unknown, unknown, string>>
}) {
  const { t } = useTranslation()
  return (
    <div className="space-y-3">
      <div>
        <span className="text-sm text-muted-foreground">{t('animals.systemNo')}</span>
        <p className="font-medium" title={animal.id}>
          {animal.id.slice(0, 8)}
        </p>
      </div>
      <div>
        <span className="text-sm text-muted-foreground">{t('animals.tabGroups.status')}</span>
        <div className="mt-0.5 flex items-center gap-1.5">
          <StatusBadge
            animal={animal}
            approvedProtocols={approvedProtocols}
            assignTrialMutation={assignTrialMutation}
          />
          <StatusHelpPopover status={animal.status} />
        </div>
      </div>
      <div>
        <span className="text-sm text-muted-foreground">{t('animals.gender')}</span>
        <p className="font-medium">{t(`animals.genderLabels.${animal.gender}`)}</p>
      </div>
    </div>
  )
}

function StatusBadge({
  animal,
  approvedProtocols,
  assignTrialMutation,
}: {
  animal: Animal
  approvedProtocols: ProtocolListItem[] | undefined
  assignTrialMutation: ReturnType<typeof useMutation<unknown, unknown, string>>
}) {
  const { t } = useTranslation()
  const [editing, setEditing] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!editing) return
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setEditing(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [editing])

  const statusName = t(`animals.statusLabels.${animal.status}`)

  if (animal.status === 'unassigned' && editing) {
    const protocolOptions = (approvedProtocols ?? []).map((p) => ({
      value: p.iacuc_no!,
      label: `${p.iacuc_no} - ${p.title}`,
    }))
    return (
      <div ref={containerRef}>
        <SearchableSelect
          options={protocolOptions}
          value=""
          onValueChange={(v) => {
            if (v) {
              assignTrialMutation.mutate(v)
              setEditing(false)
            }
          }}
          placeholder={t('animalPages.headerCard.selectStudy')}
          searchPlaceholder={t('animalPages.headerCard.searchStudy')}
          emptyMessage={t('animalPages.headerCard.noActiveStudies')}
          disabled={assignTrialMutation.isPending}
          className="w-48"
        />
      </div>
    )
  }

  if (animal.status === 'unassigned') {
    const staticBadge = (
      <button
        type="button"
        className="inline-flex items-center px-2.5 py-0.5 rounded-md bg-gray-500 text-white text-sm font-medium cursor-default"
      >
        {statusName}
      </button>
    )
    return (
      <GuestHide>
        <Can permission={PERMISSIONS.ANIMAL_ANIMAL_EDIT} fallback={staticBadge}>
          <button
            type="button"
            onClick={() => setEditing(true)}
            className="inline-flex items-center px-2.5 py-0.5 rounded-md bg-gray-500 text-white text-sm font-medium hover:bg-gray-600 transition-colors"
            title={t('animalPages.headerCard.clickToAssignStudy')}
          >
            {statusName}
          </button>
        </Can>
      </GuestHide>
    )
  }

  return (
    <button
      type="button"
      className="inline-flex items-center px-2.5 py-0.5 rounded-md bg-gray-500 text-white text-sm font-medium cursor-default"
      title={t('animalPages.headerCard.statusChangeViaTransfer')}
    >
      {statusName}
    </button>
  )
}
