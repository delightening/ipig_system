import React, { useState, useRef, useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import api, {
  Animal,
  AnimalWeight,
  ProtocolListItem,
  allAnimalStatusNames,
  animalBreedNames,
  animalGenderNames,
  facilityApi,
} from '@/lib/api'
import { Card, CardContent } from '@/components/ui/card'
import { SearchableSelect } from '@/components/ui/searchable-select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

import { detailStatusColors, getPenLocationDisplay } from '../constants'
import { GuestHide } from '@/components/ui/guest-hide'

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
  const penLocation = getPenLocationDisplay(animal, () => '\u72A7\u7272')

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
      api.put(`/animals/${animalId}`, { pen_location: penCode || null }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      setEditing(false)
      toast({ title: '成功', description: '欄號已更新' })
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(error, '更新失敗'), variant: 'destructive' })
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
          placeholder="選擇欄號"
          searchPlaceholder="搜尋欄號..."
          emptyMessage="找不到此欄號"
          className="w-36"
        />
      </div>
    )
  }

  return (
    <GuestHide>
      <button
        type="button"
        onClick={() => setEditing(true)}
        className="inline-flex items-center px-2.5 py-0.5 rounded-md bg-gray-500 text-white text-sm font-medium hover:bg-gray-600 transition-colors"
        title="點擊編輯欄號"
      >
        {penLocation}
      </button>
    </GuestHide>
  )
}

function LeftColumn({ animal, animalId, penLocation }: { animal: Animal; animalId: string; penLocation: string }) {
  return (
    <div className="space-y-3">
      <div>
        <span className="text-sm text-muted-foreground">{'\u8033\u865F'}</span>
        <p className="text-2xl font-bold text-status-warning-text">{animal.ear_tag}</p>
      </div>
      <div>
        <span className="text-sm text-muted-foreground">{'\u6B04\u865F'}</span>
        <div className="mt-0.5">
          <PenLocationField animal={animal} animalId={animalId} penLocation={penLocation} />
        </div>
      </div>
      <div>
        <span className="text-sm text-muted-foreground">{'\u54C1\u7A2E'}</span>
        <p className="font-medium">
          {animal.breed === 'other'
            ? (animal.breed_other || '\u5176\u4ED6')
            : animalBreedNames[animal.breed]}
        </p>
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
  return (
    <div className="space-y-3">
      <div>
        <span className="text-sm text-muted-foreground">{'\u51FA\u751F\u65E5\u671F'}</span>
        <p className="font-medium">
          {animal.birth_date
            ? new Date(animal.birth_date).toLocaleDateString('zh-TW', {
                timeZone: 'Asia/Taipei',
              })
            : '-'}
        </p>
      </div>
      <div>
        <span className="text-sm text-muted-foreground">IACUC No.</span>
        <p className="font-medium">{animal.iacuc_no || '\u672A\u5206\u914D'}</p>
      </div>
      {animal.status !== 'unassigned' &&
        (animal.experiment_assigned_by_name || animal.experiment_date) && (
          <div>
            <span className="text-sm text-muted-foreground">{'\u5BE6\u9A57\u5206\u914D'}</span>
            <p className="font-medium text-sm">
              {animal.experiment_assigned_by_name && (
                <span>{animal.experiment_assigned_by_name}</span>
              )}
              {animal.experiment_date && (
                <span className="text-muted-foreground ml-1">
                  (
                  {new Date(animal.experiment_date).toLocaleDateString('zh-TW', {
                    timeZone: 'Asia/Taipei',
                  })}
                  )
                </span>
              )}
            </p>
          </div>
        )}
      <div>
        <span className="text-sm text-muted-foreground">{'\u6700\u8FD1\u9AD4\u91CD'}</span>
        <p className="font-medium">
          {weights && weights.length > 0
            ? `${weights[0].weight} kg`
            : animal.entry_weight
              ? `${animal.entry_weight} kg (\u9032\u5834)`
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
  return (
    <div className="space-y-3">
      <div>
        <span className="text-sm text-muted-foreground">{'\u7CFB\u7D71\u865F'}</span>
        <p className="font-medium" title={animal.id}>
          {animal.id.slice(0, 8)}
        </p>
      </div>
      <div>
        <span className="text-sm text-muted-foreground">{'\u52D5\u7269\u72C0\u614B'}</span>
        <div className="mt-0.5">
          <StatusBadge
            animal={animal}
            approvedProtocols={approvedProtocols}
            assignTrialMutation={assignTrialMutation}
          />
        </div>
      </div>
      <div>
        <span className="text-sm text-muted-foreground">{'\u6027\u5225'}</span>
        <p className="font-medium">{animalGenderNames[animal.gender]}</p>
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

  const statusName = allAnimalStatusNames[animal.status]

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
          placeholder={'\u9078\u64C7\u8A66\u9A57\u2026'}
          searchPlaceholder={'\u641C\u5C0B\u8A66\u9A57\u2026'}
          emptyMessage={'\u76EE\u524D\u7121\u9032\u884C\u4E2D\u7684\u8A66\u9A57'}
          disabled={assignTrialMutation.isPending}
          className="w-48"
        />
      </div>
    )
  }

  if (animal.status === 'unassigned') {
    return (
      <GuestHide>
        <button
          type="button"
          onClick={() => setEditing(true)}
          className="inline-flex items-center px-2.5 py-0.5 rounded-md bg-gray-500 text-white text-sm font-medium hover:bg-gray-600 transition-colors"
          title={'\u9EDE\u64CA\u5206\u914D\u8A66\u9A57'}
        >
          {statusName}
        </button>
      </GuestHide>
    )
  }

  return (
    <button
      type="button"
      className="inline-flex items-center px-2.5 py-0.5 rounded-md bg-gray-500 text-white text-sm font-medium cursor-default"
      title={'\u72C0\u614B\u8B8A\u66F4\u9700\u900F\u904E\u8F49\u8B93\u7A0B\u5E8F'}
    >
      {statusName}
    </button>
  )
}
