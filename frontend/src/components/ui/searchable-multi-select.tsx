/**
 * SearchableMultiSelect — 可搜尋的多選下拉元件
 *
 * 適用於選項數量多（20+）且使用者可同時選擇多項的場景，例如同條觀察可掛多隻動物。
 * UX：點選 trigger → 開 popover → 搜尋 → 勾選多項 → trigger 上顯示已選 chip + ✕。
 * 結構參考 SearchableSelect（同檔案夾）以維持 Radix Popover 行為一致性。
 */

import { useState, useRef, useEffect, useMemo, useCallback } from 'react'
import * as PopoverPrimitive from '@radix-ui/react-popover'
import { Search, ChevronDown, X, Check } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface SearchableMultiSelectOption {
  value: string
  label: string
  description?: string
}

interface SearchableMultiSelectProps {
  options: SearchableMultiSelectOption[]
  /** 已選 value 陣列；外部 state 維護 */
  value: string[]
  onValueChange: (value: string[]) => void
  placeholder?: string
  searchPlaceholder?: string
  emptyMessage?: string
  className?: string
  triggerClassName?: string
  disabled?: boolean
}

export function SearchableMultiSelect({
  options,
  value,
  onValueChange,
  placeholder = '請選擇',
  searchPlaceholder = '搜尋...',
  emptyMessage = '無符合結果',
  className,
  triggerClassName,
  disabled = false,
}: SearchableMultiSelectProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [searchText, setSearchText] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)
  const [triggerWidth, setTriggerWidth] = useState<number>(0)
  const triggerRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    if (isOpen && triggerRef.current) {
      setTriggerWidth(triggerRef.current.getBoundingClientRect().width)
    }
  }, [isOpen])

  const filtered = useMemo(() => {
    if (!searchText) return options
    const keyword = searchText.toLowerCase()
    return options.filter(
      (opt) =>
        opt.label.toLowerCase().includes(keyword) ||
        opt.description?.toLowerCase().includes(keyword),
    )
  }, [options, searchText])

  const valueSet = useMemo(() => new Set(value), [value])

  // 已選的選項物件清單（顯示成 chip 用，按選取順序）
  const selectedOptions = useMemo(() => {
    const byValue = new Map(options.map((o) => [o.value, o]))
    return value
      .map((v) => byValue.get(v))
      .filter((o): o is SearchableMultiSelectOption => !!o)
  }, [options, value])

  const toggleValue = useCallback(
    (optValue: string) => {
      if (valueSet.has(optValue)) {
        onValueChange(value.filter((v) => v !== optValue))
      } else {
        onValueChange([...value, optValue])
      }
    },
    [valueSet, value, onValueChange],
  )

  const removeOne = useCallback(
    (e: React.MouseEvent, optValue: string) => {
      e.stopPropagation()
      onValueChange(value.filter((v) => v !== optValue))
    },
    [value, onValueChange],
  )

  // Reset search when closed
  useEffect(() => {
    if (!isOpen) setSearchText('')
  }, [isOpen])

  return (
    <PopoverPrimitive.Root open={isOpen} onOpenChange={setIsOpen}>
      <div className={cn('relative', className)}>
        <PopoverPrimitive.Trigger asChild>
          <button
            ref={triggerRef}
            type="button"
            disabled={disabled}
            className={cn(
              'flex min-h-10 w-full items-center justify-between rounded-md border border-input bg-background px-2 py-1.5 text-sm',
              'ring-offset-background focus:outline-hidden focus:ring-2 focus:ring-ring focus:ring-offset-2',
              'disabled:cursor-not-allowed disabled:opacity-50',
              isOpen && 'ring-2 ring-ring ring-offset-2',
              triggerClassName,
            )}
          >
            <span className="flex flex-1 flex-wrap items-center gap-1 text-left">
              {selectedOptions.length === 0 ? (
                <span className="text-muted-foreground px-1">{placeholder}</span>
              ) : (
                selectedOptions.map((opt) => (
                  <span
                    key={opt.value}
                    className="inline-flex items-center gap-1 rounded bg-accent px-2 py-0.5 text-xs"
                  >
                    {opt.label}
                    {!disabled && (
                      <span
                        role="button"
                        tabIndex={-1}
                        onClick={(e) => removeOne(e, opt.value)}
                        className="hover:text-destructive transition-colors"
                        aria-label={`移除 ${opt.label}`}
                      >
                        <X className="h-3 w-3" />
                      </span>
                    )}
                  </span>
                ))
              )}
            </span>
            <ChevronDown
              className={cn(
                'h-4 w-4 shrink-0 text-muted-foreground transition-transform',
                isOpen && 'rotate-180',
              )}
            />
          </button>
        </PopoverPrimitive.Trigger>
      </div>

      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          align="start"
          sideOffset={4}
          style={{ minWidth: Math.max(triggerWidth, 224) }}
          className="z-[9999] max-w-[24rem] rounded-md border border-border bg-popover text-popover-foreground shadow-lg"
          onOpenAutoFocus={(e) => {
            e.preventDefault()
            requestAnimationFrame(() => inputRef.current?.focus())
          }}
        >
          <div className="flex items-center border-b border-border px-3">
            <Search className="h-4 w-4 text-muted-foreground shrink-0" />
            <input
              ref={inputRef}
              type="text"
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              placeholder={searchPlaceholder}
              className="flex-1 bg-transparent border-0 outline-hidden py-2.5 px-2 text-sm placeholder:text-muted-foreground"
              autoComplete="off"
              autoCorrect="off"
              autoCapitalize="off"
              spellCheck={false}
              name="searchable-multi-select-search"
              data-form-type="other"
              data-lpignore="true"
              data-1p-ignore="true"
            />
          </div>

          <ul role="listbox" aria-multiselectable="true" className="max-h-60 overflow-auto py-1">
            {filtered.length === 0 ? (
              <li className="px-3 py-2 text-sm text-muted-foreground">
                {emptyMessage}
              </li>
            ) : (
              filtered.map((option) => {
                const checked = valueSet.has(option.value)
                return (
                  <li
                    key={option.value}
                    role="option"
                    aria-selected={checked}
                    tabIndex={0}
                    onClick={() => toggleValue(option.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault()
                        toggleValue(option.value)
                      }
                    }}
                    className={cn(
                      'flex items-center justify-between px-3 py-2 cursor-pointer transition-colors text-sm hover:bg-accent focus:bg-accent focus:outline-none',
                      checked && 'bg-accent/40',
                    )}
                  >
                    <span className="flex items-center gap-2 truncate">
                      <span
                        className={cn(
                          'flex h-4 w-4 items-center justify-center rounded border border-input shrink-0',
                          checked && 'bg-primary border-primary text-primary-foreground',
                        )}
                      >
                        {checked && <Check className="h-3 w-3" />}
                      </span>
                      <span className="truncate font-medium">{option.label}</span>
                    </span>
                    {option.description && (
                      <span className="text-xs text-muted-foreground shrink-0 ml-2">
                        {option.description}
                      </span>
                    )}
                  </li>
                )
              })
            )}
          </ul>
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  )
}
