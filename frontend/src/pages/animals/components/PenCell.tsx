import React from 'react'
import { Link } from 'react-router-dom'
import type { AnimalListItem } from '@/lib/api'
import { uiLocale } from '@/lib/utils'
import { Input } from '@/components/ui/input'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import type { getZoneColors } from '../hooks/useFacilityLayout'

interface PenCellProps {
  penCode: string | null
  colors: ReturnType<typeof getZoneColors>
  penAnimals: AnimalListItem[]
  isEditing: boolean
  editingEarTag: string
  isQuickMovePending: boolean
  onStartEdit: () => void
  onEarTagChange: (value: string) => void
  onSubmit: () => void
  onCancel: () => void
}

function EarTagQuickInput({
  value,
  onChange,
  onSubmit,
  onCancel,
  disabled,
}: {
  value: string
  onChange: (value: string) => void
  onSubmit: () => void
  onCancel: () => void
  disabled: boolean
}) {
  return (
    <Input
      className="h-7 text-sm"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      onKeyDown={(e) => {
        if (e.key === 'Enter' && value.trim()) {
          e.preventDefault()
          onSubmit()
        } else if (e.key === 'Escape') {
          e.preventDefault()
          onCancel()
        }
      }}
      onBlur={() => {
        setTimeout(() => {
          if (value.trim()) {
            onSubmit()
          } else {
            onCancel()
          }
        }, 150)
      }}
      placeholder="輸入耳號"
      autoFocus
      disabled={disabled}
    />
  )
}

export const PenCell = React.memo(function PenCell({
  penCode,
  colors,
  penAnimals,
  isEditing,
  editingEarTag,
  isQuickMovePending,
  onStartEdit,
  onEarTagChange,
  onSubmit,
  onCancel,
}: PenCellProps) {
  if (!penCode) {
    return <div className="px-3 py-2 text-muted-foreground"></div>
  }

  if (penAnimals.length === 0) {
    const readOnlyEmptyCell = (
      <div className="grid grid-cols-2 @[600px]:grid-cols-4 gap-1 px-3 py-2 items-center text-sm">
        <div className={`font-semibold ${colors.text}`}>{penCode}</div>
        <div className="text-muted-foreground italic">空</div>
        <div className="hidden @[600px]:block text-muted-foreground">-</div>
        <div className="hidden @[600px]:block text-muted-foreground">-</div>
      </div>
    )
    return (
      <Can permission={PERMISSIONS.ANIMAL_ANIMAL_EDIT} fallback={readOnlyEmptyCell}>
        <div
          className="grid grid-cols-2 @[600px]:grid-cols-4 gap-1 px-3 py-2 items-center text-sm group"
          onMouseEnter={onStartEdit}
        >
          <div className={`font-semibold ${colors.text}`}>{penCode}</div>
          {isEditing ? (
            <EarTagQuickInput
              value={editingEarTag}
              onChange={onEarTagChange}
              onSubmit={onSubmit}
              onCancel={onCancel}
              disabled={isQuickMovePending}
            />
          ) : (
            <div className="text-muted-foreground italic group-hover:text-foreground transition-colors cursor-text">空</div>
          )}
          <div className="hidden @[600px]:block text-muted-foreground">-</div>
          <div className="hidden @[600px]:block text-muted-foreground">-</div>
        </div>
      </Can>
    )
  }

  return (
    <>
      {isEditing && (
        <div className="grid grid-cols-2 @[600px]:grid-cols-4 gap-1 px-3 py-2 items-center text-sm">
          <div className={`font-semibold ${colors.text}`}>{penCode}</div>
          <EarTagQuickInput
            value={editingEarTag}
            onChange={onEarTagChange}
            onSubmit={onSubmit}
            onCancel={onCancel}
            disabled={isQuickMovePending}
          />
          <div className="hidden @[600px]:block text-muted-foreground">-</div>
          <div className="hidden @[600px]:block text-muted-foreground">-</div>
        </div>
      )}
      {penAnimals.map((animal, animalIdx) => (
        <div
          key={animal.id}
          className={`grid grid-cols-2 @[600px]:grid-cols-4 gap-1 px-3 py-2 items-center text-sm ${isEditing || animalIdx > 0 ? 'border-t border-dashed border-border' : ''}`}
        >
          <div className={`font-semibold ${colors.text}`}>
            {!isEditing && animalIdx === 0 && (
              <Can permission={PERMISSIONS.ANIMAL_ANIMAL_EDIT} fallback={<span>{penCode}</span>}>
                <button
                  type="button"
                  onClick={onStartEdit}
                  disabled={isQuickMovePending}
                  className="hover:opacity-60 transition-opacity"
                  title="點擊加入豬隻"
                >
                  {penCode}
                </button>
              </Can>
            )}
          </div>
          <Link
            to={`/animals/${animal.id}`}
            className={`font-medium break-words hover:underline cursor-pointer ${animal.has_abnormal_record ? 'text-status-error-text hover:text-status-error-text/80' : 'text-primary hover:text-primary/80'}`}
            title={animal.has_abnormal_record ? `有異常 · ${animal.ear_tag}` : `點擊進入動物詳情 · ${animal.ear_tag}`}
          >
            {animal.ear_tag}
          </Link>
          <div className="hidden @[600px]:block text-xs text-muted-foreground break-words" title={animal.vet_last_viewed_at ? new Date(animal.vet_last_viewed_at).toLocaleString(uiLocale(), { timeZone: 'Asia/Taipei' }) : '-'}>
            {animal.vet_last_viewed_at ? new Date(animal.vet_last_viewed_at).toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' }) : '-'}
          </div>
          <div className={`hidden @[600px]:block text-xs break-words ${animal.has_abnormal_record ? 'text-status-error-text font-medium' : 'text-muted-foreground'}`}>
            {animal.has_abnormal_record ? '有異常' : '-'}
          </div>
        </div>
      ))}
    </>
  )
})
