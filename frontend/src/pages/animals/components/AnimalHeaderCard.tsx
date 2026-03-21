import React from 'react'
import { useMutation } from '@tanstack/react-query'

import {
  Animal,
  AnimalWeight,
  ProtocolListItem,
  allAnimalStatusNames,
  animalBreedNames,
  animalGenderNames,
} from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

import { detailStatusColors, getPenLocationDisplay } from '../constants'

interface AnimalHeaderCardProps {
  animal: Animal
  weights: AnimalWeight[] | undefined
  showTrialSelect: boolean
  setShowTrialSelect: (v: boolean) => void
  approvedProtocols: ProtocolListItem[] | undefined
  assignTrialMutation: ReturnType<typeof useMutation<unknown, unknown, string>>
}

export function AnimalHeaderCard({
  animal,
  weights,
  showTrialSelect,
  setShowTrialSelect,
  approvedProtocols,
  assignTrialMutation,
}: AnimalHeaderCardProps) {
  const penLocation = getPenLocationDisplay(animal, () => '\u72A7\u7272')

  return (
    <Card className="bg-gradient-to-r from-slate-50 to-blue-50 border-slate-200">
      <CardContent className="pt-6">
        <div className="grid grid-cols-3 gap-6">
          <LeftColumn animal={animal} penLocation={penLocation} />
          <MiddleColumn animal={animal} weights={weights} />
          <RightColumn
            animal={animal}
            showTrialSelect={showTrialSelect}
            setShowTrialSelect={setShowTrialSelect}
            approvedProtocols={approvedProtocols}
            assignTrialMutation={assignTrialMutation}
          />
        </div>
      </CardContent>
    </Card>
  )
}

function LeftColumn({ animal, penLocation }: { animal: Animal; penLocation: string }) {
  return (
    <div className="space-y-3">
      <div>
        <span className="text-sm text-slate-500">{'\u8033\u865F'}</span>
        <p className="text-2xl font-bold text-orange-600">{animal.ear_tag}</p>
      </div>
      <div>
        <span className="text-sm text-slate-500">{'\u6B04\u865F'}</span>
        <p className="font-medium">{penLocation}</p>
      </div>
      <div>
        <span className="text-sm text-slate-500">{'\u54C1\u7A2E'}</span>
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
        <span className="text-sm text-slate-500">{'\u51FA\u751F\u65E5\u671F'}</span>
        <p className="font-medium">
          {animal.birth_date
            ? new Date(animal.birth_date).toLocaleDateString('zh-TW', {
                timeZone: 'Asia/Taipei',
              })
            : '-'}
        </p>
      </div>
      <div>
        <span className="text-sm text-slate-500">IACUC No.</span>
        <p className="font-medium">{animal.iacuc_no || '\u672A\u5206\u914D'}</p>
      </div>
      {animal.status !== 'unassigned' &&
        (animal.experiment_assigned_by_name || animal.experiment_date) && (
          <div>
            <span className="text-sm text-slate-500">{'\u5BE6\u9A57\u5206\u914D'}</span>
            <p className="font-medium text-sm">
              {animal.experiment_assigned_by_name && (
                <span>{animal.experiment_assigned_by_name}</span>
              )}
              {animal.experiment_date && (
                <span className="text-slate-400 ml-1">
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
        <span className="text-sm text-slate-500">{'\u6700\u8FD1\u9AD4\u91CD'}</span>
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
  showTrialSelect,
  setShowTrialSelect,
  approvedProtocols,
  assignTrialMutation,
}: {
  animal: Animal
  showTrialSelect: boolean
  setShowTrialSelect: (v: boolean) => void
  approvedProtocols: ProtocolListItem[] | undefined
  assignTrialMutation: ReturnType<typeof useMutation<unknown, unknown, string>>
}) {
  return (
    <div className="space-y-3">
      <div>
        <span className="text-sm text-slate-500">{'\u7CFB\u7D71\u865F'}</span>
        <p className="font-medium" title={animal.id}>
          {animal.id.slice(0, 8)}
        </p>
      </div>
      <div>
        <span className="text-sm text-slate-500">{'\u52D5\u7269\u72C0\u614B'}</span>
        <p className="mt-1">
          <StatusBadge
            animal={animal}
            showTrialSelect={showTrialSelect}
            setShowTrialSelect={setShowTrialSelect}
            approvedProtocols={approvedProtocols}
            assignTrialMutation={assignTrialMutation}
          />
        </p>
      </div>
      <div>
        <span className="text-sm text-slate-500">{'\u6027\u5225'}</span>
        <p className="font-medium">{animalGenderNames[animal.gender]}</p>
      </div>
    </div>
  )
}

function StatusBadge({
  animal,
  showTrialSelect,
  setShowTrialSelect,
  approvedProtocols,
  assignTrialMutation,
}: {
  animal: Animal
  showTrialSelect: boolean
  setShowTrialSelect: (v: boolean) => void
  approvedProtocols: ProtocolListItem[] | undefined
  assignTrialMutation: ReturnType<typeof useMutation<unknown, unknown, string>>
}) {
  if (animal.status === 'unassigned' && !showTrialSelect) {
    return (
      <Badge
        className={`${detailStatusColors[animal.status]} text-white cursor-pointer hover:bg-gray-600 transition-colors`}
        onClick={() => setShowTrialSelect(true)}
        title={'\u9EDE\u64CA\u5206\u914D\u8A66\u9A57'}
      >
        {allAnimalStatusNames[animal.status]} {'\u25BE'}
      </Badge>
    )
  }

  if (animal.status === 'unassigned' && showTrialSelect) {
    return (
      <div className="flex flex-col gap-1">
        <Select
          onValueChange={(value) => {
            if (value === '__cancel__') {
              setShowTrialSelect(false)
            } else {
              assignTrialMutation.mutate(value)
            }
          }}
          disabled={assignTrialMutation.isPending}
        >
          <SelectTrigger className="w-[220px] h-8 text-xs">
            <SelectValue placeholder={'\u9078\u64C7\u8A66\u9A57...'} />
          </SelectTrigger>
          <SelectContent>
            {approvedProtocols && approvedProtocols.length > 0 ? (
              approvedProtocols.map((protocol) => (
                <SelectItem key={protocol.id} value={protocol.iacuc_no!}>
                  {protocol.iacuc_no} - {protocol.title}
                </SelectItem>
              ))
            ) : (
              <SelectItem value="__none__" disabled>
                {'\u76EE\u524D\u7121\u9032\u884C\u4E2D\u7684\u8A66\u9A57'}
              </SelectItem>
            )}
          </SelectContent>
        </Select>
        <button
          className="text-xs text-slate-400 hover:text-slate-600 text-left"
          onClick={() => setShowTrialSelect(false)}
        >
          {'\u53D6\u6D88'}
        </button>
      </div>
    )
  }

  return (
    <Badge className={`${detailStatusColors[animal.status]} text-white`}>
      {allAnimalStatusNames[animal.status]}
    </Badge>
  )
}
