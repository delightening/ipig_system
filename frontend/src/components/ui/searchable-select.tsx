/**
 * SearchableSelect — 可搜尋的下拉選擇元件
 *
 * 適用於選項數量多（20+）且需要搜尋篩選的場景。
 * 支援 description 副文字以區分同名選項。
 */

import { useState, useRef, useEffect, useMemo, useCallback } from 'react'
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
  disabled = false,
  icon: Icon,
}: SearchableSelectProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [searchText, setSearchText] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(-1)
  const containerRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)
  const listRef = useRef<HTMLUListElement>(null)

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
    if (!isOpen) {
      if (e.key === 'Enter' || e.key === 'ArrowDown') {
        e.preventDefault()
        setIsOpen(true)
      }
      return
    }

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
      case 'Escape':
        setIsOpen(false)
        setSearchText('')
        setSelectedIndex(-1)
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

  // Click outside to close
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false)
        setSearchText('')
        setSelectedIndex(-1)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  // Auto-focus search input when opened
  useEffect(() => {
    if (isOpen) {
      requestAnimationFrame(() => inputRef.current?.focus())
    }
  }, [isOpen])

  return (
    <div className={cn('relative', className)} ref={containerRef}>
      {/* Trigger */}
      <button
        type="button"
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled}
        className={cn(
          'flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm',
          'ring-offset-background focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
          'disabled:cursor-not-allowed disabled:opacity-50',
          isOpen && 'ring-2 ring-ring ring-offset-2',
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

      {/* Dropdown */}
      {isOpen && (
        <div className="absolute z-50 w-full mt-1 rounded-md border border-border bg-popover text-popover-foreground shadow-md">
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
              className="flex-1 bg-transparent border-0 outline-none py-2.5 px-2 text-sm placeholder:text-muted-foreground"
              role="combobox"
              aria-expanded={isOpen}
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
        </div>
      )}
    </div>
  )
}
