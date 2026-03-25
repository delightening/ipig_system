import React from 'react'
import { Link } from 'react-router-dom'
import type { AnimalListItem } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Eye, Edit2 } from 'lucide-react'
import type { getZoneColors } from '../hooks/useFacilityLayout'

interface PenCellProps {
  penCode: string | null
  colors: ReturnType<typeof getZoneColors>
  penAnimals: AnimalListItem[]
  isEditing: boolean
  editingEarTag: string
  isQuickMovePending: boolean
  onMouseEnter: () => void
  onEarTagChange: (value: string) => void
  onSubmit: () => void
  onCancel: () => void
}

export const PenCell = React.memo(function PenCell({
  penCode,
  colors,
  penAnimals,
  isEditing,
  editingEarTag,
  isQuickMovePending,
  onMouseEnter,
  onEarTagChange,
  onSubmit,
  onCancel,
}: PenCellProps) {
  if (!penCode) {
    return <div className="px-3 py-2 text-muted-foreground"></div>
  }

  if (penAnimals.length === 0) {
    return (
      <div
        className="grid grid-cols-5 gap-1 px-3 py-2 items-center text-sm group"
        onMouseEnter={onMouseEnter}
      >
        <div className={`font-semibold ${colors.text}`}>{penCode}</div>
        {isEditing ? (
          <Input
            className="h-7 text-sm"
            value={editingEarTag}
            onChange={(e) => onEarTagChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && editingEarTag.trim()) {
                e.preventDefault()
                onSubmit()
              } else if (e.key === 'Escape') {
                e.preventDefault()
                onCancel()
              }
            }}
            onBlur={() => {
              setTimeout(() => {
                if (editingEarTag.trim()) {
                  onSubmit()
                } else {
                  onCancel()
                }
              }, 150)
            }}
            placeholder="輸入耳號"
            autoFocus
            disabled={isQuickMovePending}
          />
        ) : (
          <div className="text-muted-foreground italic group-hover:text-foreground transition-colors cursor-text">空</div>
        )}
        <div className="text-muted-foreground">-</div>
        <div className="text-muted-foreground">-</div>
        <div></div>
      </div>
    )
  }

  return penAnimals.map((animal, animalIdx) => (
    <div key={animal.id} className={`grid grid-cols-5 gap-1 px-3 py-2 items-center text-sm ${animalIdx > 0 ? 'border-t border-dashed border-border' : ''}`}>
      <div className={`font-semibold ${colors.text}`}>{animalIdx === 0 ? penCode : ''}</div>
      <Link
        to={`/animals/${animal.id}`}
        className="font-medium truncate text-primary hover:text-primary/80 hover:underline cursor-pointer"
        title={`點擊進入動物詳情 · ${animal.ear_tag}`}
      >
        {animal.ear_tag}
      </Link>
      <div className="text-xs text-muted-foreground truncate" title={animal.vet_last_viewed_at ? new Date(animal.vet_last_viewed_at).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei' }) : '-'}>
        {animal.vet_last_viewed_at ? new Date(animal.vet_last_viewed_at).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' }) : '-'}
      </div>
      <div className={`text-xs truncate ${animal.has_abnormal_record ? 'text-status-error-text font-medium' : 'text-muted-foreground'}`}>
        {animal.has_abnormal_record ? '有異常' : '-'}
      </div>
      <div className="flex items-center justify-center gap-1">
        <Button variant="ghost" size="icon" className="h-6 w-6" asChild title="檢視" aria-label="檢視">
          <Link to={`/animals/${animal.id}`}>
            <Eye className="h-3 w-3" />
          </Link>
        </Button>
        <Button variant="ghost" size="icon" className="h-6 w-6" asChild title="編輯" aria-label="編輯">
          <Link to={`/animals/${animal.id}/edit`}>
            <Edit2 className="h-3 w-3" />
          </Link>
        </Button>
      </div>
    </div>
  ))
})
