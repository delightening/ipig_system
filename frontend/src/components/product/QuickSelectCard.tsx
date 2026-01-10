import * as React from 'react'
import { Check, ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface QuickSelectItem {
  id: string
  icon?: React.ReactNode
  label: string
  sublabel?: string
  specs?: QuickSelectSpec[]
}

export interface QuickSelectSpec {
  id: string
  primary: string
  secondary?: string
}

interface QuickSelectCardProps {
  item: QuickSelectItem
  selected?: boolean
  onClick?: () => void
  disabled?: boolean
  className?: string
}

export function QuickSelectCard({
  item,
  selected = false,
  onClick,
  disabled = false,
  className,
}: QuickSelectCardProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={cn(
        "quick-select-card relative flex flex-col items-center justify-center p-3 rounded-lg border-2 transition-all",
        "min-w-[80px] h-[72px]",
        selected
          ? "bg-primary/10 border-primary text-primary"
          : "bg-white dark:bg-slate-800 border-slate-200 dark:border-slate-700 hover:border-primary/50 hover:bg-slate-50 dark:hover:bg-slate-700/50",
        disabled && "opacity-50 cursor-not-allowed",
        className
      )}
    >
      {selected && (
        <div className="absolute top-1 right-1">
          <Check className="w-3.5 h-3.5 text-primary" />
        </div>
      )}
      {item.icon && (
        <div className={cn(
          "text-2xl mb-1",
          selected ? "text-primary" : "text-slate-600 dark:text-slate-400"
        )}>
          {item.icon}
        </div>
      )}
      <span className={cn(
        "text-sm font-medium",
        selected ? "text-primary" : "text-slate-700 dark:text-slate-300"
      )}>
        {item.label}
      </span>
      {item.sublabel && (
        <span className="text-xs text-slate-400 mt-0.5">
          {item.sublabel}
        </span>
      )}
    </button>
  )
}

interface QuickSelectGridProps {
  items: QuickSelectItem[]
  selectedId?: string
  onSelect?: (item: QuickSelectItem) => void
  disabled?: boolean
  showMore?: boolean
  onShowMore?: () => void
  className?: string
}

export function QuickSelectGrid({
  items,
  selectedId,
  onSelect,
  disabled = false,
  showMore = false,
  onShowMore,
  className,
}: QuickSelectGridProps) {
  return (
    <div className={cn("grid grid-cols-4 sm:grid-cols-6 lg:grid-cols-8 gap-2", className)}>
      {items.map((item) => (
        <QuickSelectCard
          key={item.id}
          item={item}
          selected={selectedId === item.id}
          onClick={() => onSelect?.(item)}
          disabled={disabled}
        />
      ))}
      {showMore && onShowMore && (
        <button
          type="button"
          onClick={onShowMore}
          disabled={disabled}
          className={cn(
            "flex flex-col items-center justify-center p-3 rounded-lg border-2 border-dashed transition-all",
            "min-w-[80px] h-[72px]",
            "border-slate-300 dark:border-slate-600 hover:border-primary/50 hover:bg-slate-50 dark:hover:bg-slate-700/50",
            disabled && "opacity-50 cursor-not-allowed"
          )}
        >
          <ChevronRight className="w-5 h-5 text-slate-400 mb-1" />
          <span className="text-xs text-slate-500">更多</span>
        </button>
      )}
    </div>
  )
}

// Spec Selection Panel (展開後的規格選擇)
interface SpecSelectionPanelProps {
  title: string
  specs: QuickSelectSpec[]
  selectedId?: string
  onSelect?: (spec: QuickSelectSpec) => void
  extraOptions?: {
    label: string
    options: { value: string; label: string }[]
    value?: string
    onChange?: (value: string) => void
  }[]
  disabled?: boolean
  className?: string
}

export function SpecSelectionPanel({
  title,
  specs,
  selectedId,
  onSelect,
  extraOptions,
  disabled = false,
  className,
}: SpecSelectionPanelProps) {
  return (
    <div className={cn("space-y-4", className)}>
      <h4 className="text-sm font-medium text-slate-700 dark:text-slate-300">
        選擇規格：{title}
      </h4>
      
      <div className="grid grid-cols-4 sm:grid-cols-6 lg:grid-cols-8 gap-2">
        {specs.map((spec) => (
          <button
            key={spec.id}
            type="button"
            onClick={() => onSelect?.(spec)}
            disabled={disabled}
            className={cn(
              "flex flex-col items-center justify-center p-3 rounded-lg border-2 transition-all min-h-[56px]",
              selectedId === spec.id
                ? "bg-primary/10 border-primary text-primary"
                : "bg-white dark:bg-slate-800 border-slate-200 dark:border-slate-700 hover:border-primary/50",
              disabled && "opacity-50 cursor-not-allowed"
            )}
          >
            <span className="text-sm font-medium">{spec.primary}</span>
            {spec.secondary && (
              <span className="text-xs text-slate-400 mt-0.5">{spec.secondary}</span>
            )}
          </button>
        ))}
      </div>

      {extraOptions && extraOptions.length > 0 && (
        <div className="space-y-3 pt-2">
          {extraOptions.map((option, index) => (
            <div key={index} className="flex items-center gap-4">
              <span className="text-sm text-slate-600 dark:text-slate-400 min-w-[60px]">
                {option.label}：
              </span>
              <div className="flex gap-2">
                {option.options.map((opt) => (
                  <label
                    key={opt.value}
                    className={cn(
                      "flex items-center gap-1.5 px-3 py-1.5 rounded-full border cursor-pointer transition-all text-sm",
                      option.value === opt.value
                        ? "bg-primary/10 border-primary text-primary"
                        : "border-slate-200 dark:border-slate-700 hover:border-primary/50"
                    )}
                  >
                    <input
                      type="radio"
                      name={option.label}
                      value={opt.value}
                      checked={option.value === opt.value}
                      onChange={() => option.onChange?.(opt.value)}
                      className="sr-only"
                    />
                    <span>{opt.label}</span>
                  </label>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
