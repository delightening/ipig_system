import { Plus, X } from 'lucide-react'

import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface UnitOption {
  code: string
  name: string
}

interface UnitButtonGroupProps {
  units: UnitOption[]
  selectedUnit: string
  onSelect: (unit: UnitOption) => void
  isCustom: boolean
  onCustomToggle: () => void
  customValue: string
  onCustomChange: (value: string) => void
  onCustomClear?: () => void
  disabled?: boolean
  /** Units highlighted with a blue ring (e.g. already selected in other layers) */
  highlightedCodes?: string[]
}

export function UnitButtonGroup({
  units,
  selectedUnit,
  onSelect,
  isCustom,
  onCustomToggle,
  customValue,
  onCustomChange,
  onCustomClear,
  disabled = false,
  highlightedCodes = [],
}: UnitButtonGroupProps) {
  return (
    <div className="flex items-center gap-3">
      <div className="flex flex-wrap gap-2 flex-1">
        {units.map((unit) => (
          <button
            key={unit.code}
            type="button"
            onClick={() => onSelect(unit)}
            disabled={disabled}
            className={cn(
              "relative flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 transition-all",
              selectedUnit === unit.name && !isCustom
                ? "border-primary bg-primary/10"
                : "border-slate-200 dark:border-slate-700 hover:border-primary/50",
              highlightedCodes.includes(unit.code) && "ring-2 ring-blue-300 dark:ring-blue-700"
            )}
          >
            <span className="font-mono font-semibold text-sm">{unit.code}</span>
            <span className="text-xs text-slate-500">{unit.name}</span>
            {highlightedCodes.includes(unit.code) && (
              <span className="absolute -top-1 -right-1 w-3 h-3 bg-blue-500 rounded-full border-2 border-white dark:border-slate-800" />
            )}
          </button>
        ))}
        <button
          type="button"
          onClick={onCustomToggle}
          disabled={disabled}
          className={cn(
            "flex flex-col items-center justify-center w-16 h-14 rounded-lg border-2 border-dashed transition-all",
            isCustom
              ? "border-primary bg-primary/10"
              : "border-slate-300 dark:border-slate-600 hover:border-primary/50"
          )}
        >
          <Plus className="w-5 h-5 text-slate-400" />
          <span className="text-[10px] text-slate-500 mt-1">自填量詞</span>
        </button>
      </div>
      {isCustom && (
        <div className="flex items-center gap-2">
          <Input
            placeholder="輸入量詞"
            value={customValue}
            onChange={(e) => onCustomChange(e.target.value)}
            className="w-24"
            disabled={disabled}
          />
          {onCustomClear && (
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={onCustomClear}
            >
              <X className="h-4 w-4" />
            </Button>
          )}
        </div>
      )}
    </div>
  )
}
