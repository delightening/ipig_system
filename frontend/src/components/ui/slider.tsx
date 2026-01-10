import * as React from 'react'
import { cn } from '@/lib/utils'

interface SliderProps {
  value: number
  onChange: (value: number) => void
  min?: number
  max?: number
  step?: number
  quickValues?: number[]
  label?: string
  unit?: string
  unitOptions?: { value: string; label: string }[]
  selectedUnit?: string
  onUnitChange?: (unit: string) => void
  disabled?: boolean
  className?: string
}

export function Slider({
  value,
  onChange,
  min = 0,
  max = 500,
  step = 1,
  quickValues = [50, 100, 200, 500],
  label,
  unit,
  unitOptions,
  selectedUnit,
  onUnitChange,
  disabled = false,
  className,
}: SliderProps) {
  const percentage = ((value - min) / (max - min)) * 100
  const inputRef = React.useRef<HTMLInputElement>(null)

  const handleSliderChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(Number(e.target.value))
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = Number(e.target.value)
    if (!isNaN(newValue)) {
      onChange(Math.min(max, Math.max(min, newValue)))
    }
  }

  const handleQuickValue = (qv: number) => {
    onChange(qv)
  }

  return (
    <div className={cn("space-y-3", className)}>
      {label && (
        <label className="block text-sm font-medium text-slate-700 dark:text-slate-300">
          {label}
        </label>
      )}

      {/* Quick Value Buttons */}
      <div className="flex flex-wrap gap-2">
        {quickValues.map((qv) => (
          <button
            key={qv}
            type="button"
            onClick={() => handleQuickValue(qv)}
            disabled={disabled}
            className={cn(
              "px-4 py-1.5 text-sm font-medium rounded-full border transition-all",
              value === qv
                ? "bg-primary text-primary-foreground border-primary"
                : "bg-white dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700 hover:border-primary/50 hover:bg-primary/5",
              disabled && "opacity-50 cursor-not-allowed"
            )}
          >
            {qv}
          </button>
        ))}
        <button
          type="button"
          onClick={() => inputRef.current?.focus()}
          disabled={disabled}
          className={cn(
            "px-4 py-1.5 text-sm font-medium rounded-full border transition-all",
            !quickValues.includes(value) && value > 0
              ? "bg-primary text-primary-foreground border-primary"
              : "bg-white dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700 hover:border-primary/50 hover:bg-primary/5",
            disabled && "opacity-50 cursor-not-allowed"
          )}
        >
          自訂
        </button>
      </div>

      {/* Slider Track */}
      <div className="relative pt-1">
        <div className="relative h-2 rounded-full bg-slate-200 dark:bg-slate-700">
          <div
            className="absolute h-full rounded-full bg-gradient-to-r from-primary/80 to-primary transition-all"
            style={{ width: `${percentage}%` }}
          />
        </div>
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={handleSliderChange}
          disabled={disabled}
          className={cn(
            "absolute top-0 left-0 w-full h-2 opacity-0 cursor-pointer",
            disabled && "cursor-not-allowed"
          )}
          style={{ marginTop: '4px' }}
        />
        {/* Thumb indicator */}
        <div
          className={cn(
            "absolute top-1/2 -translate-y-1/2 w-5 h-5 rounded-full bg-white border-2 border-primary shadow-md slider-thumb transition-transform",
            disabled && "opacity-50"
          )}
          style={{ 
            left: `calc(${percentage}% - 10px)`,
            marginTop: '4px',
            pointerEvents: 'none'
          }}
        />
        {/* Scale labels */}
        <div className="flex justify-between mt-2 text-xs text-slate-400">
          <span>{min}</span>
          <span>{max}</span>
        </div>
      </div>

      {/* Value Input with Unit */}
      <div className="flex items-center gap-3">
        <span className="text-sm text-slate-500 dark:text-slate-400">當前值：</span>
        <input
          ref={inputRef}
          type="number"
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={handleInputChange}
          disabled={disabled}
          className={cn(
            "w-24 px-3 py-1.5 text-center font-mono text-sm border rounded-md",
            "bg-white dark:bg-slate-800 border-slate-200 dark:border-slate-700",
            "focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary",
            disabled && "opacity-50 cursor-not-allowed"
          )}
        />
        
        {/* Unit Display or Selector */}
        {unitOptions && onUnitChange ? (
          <select
            value={selectedUnit}
            onChange={(e) => onUnitChange(e.target.value)}
            disabled={disabled}
            className={cn(
              "px-3 py-1.5 text-sm border rounded-md",
              "bg-white dark:bg-slate-800 border-slate-200 dark:border-slate-700",
              "focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary",
              disabled && "opacity-50 cursor-not-allowed"
            )}
          >
            {unitOptions.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        ) : unit && (
          <span className="text-sm text-slate-600 dark:text-slate-400">{unit}</span>
        )}
      </div>
    </div>
  )
}
