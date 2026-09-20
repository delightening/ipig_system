import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { Check, ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface QuickSelectItem {
  id: string
  icon?: React.ReactNode
  /** 按鈕顯示文字（隨 UI 語系） */
  label: string
  /** 點按後寫入名稱欄的資料值（固定中文，使用者裁定 2026-09-19）；未提供時退回 label */
  value?: string
  displayLabel?: React.ReactNode
  sublabel?: string
  specs?: QuickSelectSpec[]
}

export interface QuickSelectSpec {
  id: string
  /** 按鈕顯示文字（隨 UI 語系） */
  primary: string
  secondary?: string
  /** 點按後寫入規格欄的資料值（固定中文，使用者裁定 2026-09-19）；未提供時退回 primary / secondary */
  value?: { primary: string; secondary?: string }
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
        "min-w-[100px] h-[88px] p-4",
        selected
          ? "bg-primary/10 border-primary text-primary"
          : "bg-white dark:bg-slate-800 border-border dark:border-slate-700 hover:border-primary/50 hover:bg-muted dark:hover:bg-slate-700/50",
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
          selected ? "text-primary" : "text-muted-foreground dark:text-muted-foreground"
        )}>
          {item.icon}
        </div>
      )}
      <span className={cn(
        "text-sm font-medium",
        selected ? "text-primary" : "text-foreground dark:text-muted-foreground"
      )}>
        {item.displayLabel || item.label}
      </span>
      {item.sublabel && (
        <span className="text-xs text-muted-foreground mt-0.5">
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
  const { t } = useTranslation()

  return (
    <div className={cn("grid grid-cols-3 sm:grid-cols-4 lg:grid-cols-6 xl:grid-cols-8 gap-4", className)}>
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
            "min-w-[100px] h-[88px] p-4",
            "border-border dark:border-slate-600 hover:border-primary/50 hover:bg-muted dark:hover:bg-slate-700/50",
            disabled && "opacity-50 cursor-not-allowed"
          )}
        >
          <ChevronRight className="w-5 h-5 text-muted-foreground mb-1" />
          <span className="text-xs text-muted-foreground">{t('erpMaster.quickSelect.more')}</span>
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
  const { t } = useTranslation()

  return (
    <div className={cn("space-y-4", className)}>
      <h4 className="text-sm font-medium text-foreground dark:text-muted-foreground">
        {t('erpMaster.quickSelect.selectSpec', { title })}
      </h4>

      <div className="grid grid-cols-3 sm:grid-cols-4 lg:grid-cols-6 gap-3">
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
                : "bg-white dark:bg-slate-800 border-border dark:border-slate-700 hover:border-primary/50",
              disabled && "opacity-50 cursor-not-allowed"
            )}
          >
            <span className="text-sm font-medium">{spec.primary}</span>
            {spec.secondary && (
              <span className="text-xs text-muted-foreground mt-0.5">{spec.secondary}</span>
            )}
          </button>
        ))}
      </div>

      {extraOptions && extraOptions.length > 0 && (
        <div className="space-y-3 pt-2">
          {extraOptions.map((option, index) => (
            <div key={index} className="flex items-center gap-4">
              <span className="text-sm text-muted-foreground dark:text-muted-foreground min-w-[60px]">
                {t('erpMaster.quickSelect.optionLabel', { label: option.label })}
              </span>
              <div className="flex gap-2">
                {option.options.map((opt) => (
                  <label
                    key={opt.value}
                    className={cn(
                      "flex items-center gap-1.5 px-3 py-1.5 rounded-full border cursor-pointer transition-all text-sm",
                      option.value === opt.value
                        ? "bg-primary/10 border-primary text-primary"
                        : "border-border dark:border-slate-700 hover:border-primary/50"
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
