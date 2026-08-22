/**
 * SearchableSelect — 可搜尋的下拉選擇元件
 *
 * 適用於選項數量多（20+）且需要搜尋篩選的場景。
 * 支援 description 副文字以區分同名選項。
 *
 * 2026-05-11 重寫：用 Radix Popover 取代手刻 portal + native capture，
 * Popover 內建處理 transform containing block、focus trap coexist、
 * 外部點擊偵測、Dialog 巢狀 — 一個元件搞定所有 edge case。
 */

import { useState, useRef, useEffect, useMemo, useCallback } from 'react'
import * as PopoverPrimitive from '@radix-ui/react-popover'
import { Search, ChevronDown, X } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface SearchableSelectOption {
  value: string
  label: string
  description?: string
}

interface SearchableSelectProps {
  options: SearchableSelectOption[]
  value: string
  onValueChange: (value: string) => void
  placeholder?: string
  searchPlaceholder?: string
  emptyMessage?: string
  className?: string
  /** 套用到 trigger button 的 className（如 'h-8' 配合相鄰 input 的 32px 高度）*/
  triggerClassName?: string
  disabled?: boolean
  icon?: React.ComponentType<{ className?: string }>
}

export function SearchableSelect({
  options,
  value,
  onValueChange,
  placeholder = '請選擇',
  searchPlaceholder = '搜尋...',
  emptyMessage = '無符合結果',
  className,
  triggerClassName,
  disabled = false,
  icon: Icon,
}: SearchableSelectProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [searchText, setSearchText] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(-1)
  const inputRef = useRef<HTMLInputElement>(null)
  const listRef = useRef<HTMLUListElement>(null)
  const [triggerWidth, setTriggerWidth] = useState<number>(0)
  const triggerRef = useRef<HTMLButtonElement>(null)

  // 量 trigger 寬度給 dropdown 用（minWidth = 至少 trigger 寬，但不超過 max-w-[24rem]）
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

  const selectedOption = useMemo(
    () => options.find((o) => o.value === value),
    [options, value],
  )

  const handleSelect = useCallback(
    (optValue: string) => {
      onValueChange(optValue)
      setIsOpen(false)
      setSearchText('')
      setSelectedIndex(-1)
    },
    [onValueChange],
  )

  const handleClear = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation()
      onValueChange('')
      setSearchText('')
    },
    [onValueChange],
  )

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        setSelectedIndex((prev) =>
          prev < filtered.length - 1 ? prev + 1 : prev,
        )
        break
      case 'ArrowUp':
        e.preventDefault()
        setSelectedIndex((prev) => (prev > 0 ? prev - 1 : 0))
        break
      case 'Enter':
        e.preventDefault()
        if (selectedIndex >= 0 && filtered[selectedIndex]) {
          handleSelect(filtered[selectedIndex].value)
        }
        break
    }
  }

  // Scroll selected item into view
  useEffect(() => {
    if (selectedIndex >= 0 && listRef.current) {
      const item = listRef.current.children[selectedIndex] as HTMLElement
      item?.scrollIntoView({ block: 'nearest' })
    }
  }, [selectedIndex])

  // Reset search when closed
  useEffect(() => {
    if (!isOpen) {
      setSearchText('')
      setSelectedIndex(-1)
    }
  }, [isOpen])

  return (
    <PopoverPrimitive.Root open={isOpen} onOpenChange={setIsOpen}>
      <div className={cn('relative', className)}>
        {/* Trigger */}
        <PopoverPrimitive.Trigger asChild>
          <button
            ref={triggerRef}
            type="button"
            disabled={disabled}
            className={cn(
              'flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm',
              'ring-offset-background focus:outline-hidden focus:ring-2 focus:ring-ring focus:ring-offset-2',
              'disabled:cursor-not-allowed disabled:opacity-50',
              isOpen && 'ring-2 ring-ring ring-offset-2',
              triggerClassName,
            )}
          >
            <span className="flex items-center gap-2 truncate">
              {Icon && <Icon className="h-4 w-4 text-muted-foreground shrink-0" />}
              {selectedOption ? (
                <span className="truncate">
                  {selectedOption.label}
                  {selectedOption.description && (
                    <span className="text-muted-foreground ml-1">
                      ({selectedOption.description})
                    </span>
                  )}
                </span>
              ) : (
                <span className="text-muted-foreground">{placeholder}</span>
              )}
            </span>
            <span className="flex items-center gap-1 shrink-0">
              {value && !disabled && (
                <span
                  role="button"
                  tabIndex={-1}
                  onClick={handleClear}
                  className="p-0.5 rounded hover:bg-accent transition-colors"
                >
                  <X className="h-3.5 w-3.5 text-muted-foreground" />
                </span>
              )}
              <ChevronDown
                className={cn(
                  'h-4 w-4 text-muted-foreground transition-transform',
                  isOpen && 'rotate-180',
                )}
              />
            </span>
          </button>
        </PopoverPrimitive.Trigger>
      </div>

      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          align="start"
          sideOffset={4}
          style={{ minWidth: Math.max(triggerWidth, 224) }}
          className="z-[9999] max-w-[24rem] rounded-md border border-border bg-popover text-popover-foreground shadow-lg"
          // 阻止 Popover 預設「auto-focus 第一個 focusable」行為，把焦點手動丟到 search input
          onOpenAutoFocus={(e) => {
            e.preventDefault()
            requestAnimationFrame(() => inputRef.current?.focus())
          }}
        >
          {/* Search input */}
          <div className="flex items-center border-b border-border px-3">
            <Search className="h-4 w-4 text-muted-foreground shrink-0" />
            <input
              ref={inputRef}
              type="text"
              value={searchText}
              onChange={(e) => {
                setSearchText(e.target.value)
                setSelectedIndex(-1)
              }}
              onKeyDown={handleKeyDown}
              placeholder={searchPlaceholder}
              className="flex-1 bg-transparent border-0 outline-hidden py-2.5 px-2 text-sm placeholder:text-muted-foreground"
              role="combobox"
              aria-expanded={isOpen}
              // 防止瀏覽器 / 擴充功能自動填入（password manager、email autofill）
              autoComplete="off"
              autoCorrect="off"
              autoCapitalize="off"
              spellCheck={false}
              name="searchable-select-search"
              data-form-type="other"
              data-lpignore="true"
              data-1p-ignore="true"
            />
          </div>

          {/* Options list */}
          <ul
            ref={listRef}
            role="listbox"
            className="max-h-60 overflow-auto py-1"
          >
            {filtered.length === 0 ? (
              <li className="px-3 py-2 text-sm text-muted-foreground">
                {emptyMessage}
              </li>
            ) : (
              filtered.map((option, index) => (
                <li
                  key={option.value}
                  role="option"
                  aria-selected={selectedIndex === index}
                  onClick={() => handleSelect(option.value)}
                  className={cn(
                    'flex items-center justify-between px-3 py-2 cursor-pointer transition-colors text-sm',
                    selectedIndex === index && 'bg-accent text-accent-foreground',
                    option.value === value && selectedIndex !== index && 'bg-accent/50',
                    selectedIndex !== index && option.value !== value && 'hover:bg-accent',
                  )}
                >
                  <span className="truncate font-medium">{option.label}</span>
                  {option.description && (
                    <span className="text-xs text-muted-foreground shrink-0 ml-2">
                      {option.description}
                    </span>
                  )}
                </li>
              ))
            )}
          </ul>
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  )
}
