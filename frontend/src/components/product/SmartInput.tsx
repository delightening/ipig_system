import * as React from 'react'
import { Search, X, Package, Plus, Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface ProductSuggestion {
  id?: string
  sku?: string
  name: string
  spec: string
  category?: string
  similarity?: number
}

interface SmartInputProps {
  value: string
  onChange: (value: string) => void
  onSelect?: (suggestion: ProductSuggestion) => void
  onCreateNew?: (input: string) => void
  suggestions?: ProductSuggestion[]
  isLoading?: boolean
  placeholder?: string
  disabled?: boolean
  className?: string
}

export function SmartInput({
  value,
  onChange,
  onSelect,
  onCreateNew,
  suggestions = [],
  isLoading = false,
  placeholder = "輸入產品名稱和規格...",
  disabled = false,
  className,
}: SmartInputProps) {
  const [isFocused, setIsFocused] = React.useState(false)
  const [selectedIndex, setSelectedIndex] = React.useState(-1)
  const inputRef = React.useRef<HTMLInputElement>(null)
  const listRef = React.useRef<HTMLUListElement>(null)

  const showSuggestions = isFocused && value.length > 0 && (suggestions.length > 0 || isLoading)

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!showSuggestions) return

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        setSelectedIndex((prev) => 
          prev < suggestions.length ? prev + 1 : prev
        )
        break
      case 'ArrowUp':
        e.preventDefault()
        setSelectedIndex((prev) => (prev > -1 ? prev - 1 : -1))
        break
      case 'Enter':
        e.preventDefault()
        if (selectedIndex === suggestions.length) {
          // Create new
          onCreateNew?.(value)
        } else if (selectedIndex >= 0 && suggestions[selectedIndex]) {
          onSelect?.(suggestions[selectedIndex])
        }
        break
      case 'Escape':
        setIsFocused(false)
        break
    }
  }

  const handleClear = () => {
    onChange('')
    inputRef.current?.focus()
  }

  React.useEffect(() => {
    setSelectedIndex(-1)
  }, [suggestions])

  return (
    <div className={cn("relative", className)}>
      {/* Input Container */}
      <div
        className={cn(
          "relative flex items-center h-14 px-4 rounded-lg border-2 transition-all",
          "bg-white dark:bg-slate-800",
          isFocused
            ? "border-primary ring-4 ring-primary/10"
            : "border-slate-200 dark:border-slate-700 hover:border-slate-300",
          disabled && "opacity-50 cursor-not-allowed"
        )}
      >
        <Search className="w-5 h-5 text-slate-400 mr-3 shrink-0" />
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onFocus={() => setIsFocused(true)}
          onBlur={() => setTimeout(() => setIsFocused(false), 200)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          disabled={disabled}
          className={cn(
            "flex-1 text-base bg-transparent border-0 outline-none",
            "placeholder:text-slate-400",
            "text-slate-900 dark:text-slate-100"
          )}
          role="combobox"
          aria-expanded={showSuggestions}
          aria-controls="suggestions-listbox"
          aria-activedescendant={selectedIndex >= 0 ? `suggestion-${selectedIndex}` : undefined}
        />
        {isLoading && (
          <Loader2 className="w-5 h-5 text-slate-400 animate-spin mr-2" />
        )}
        {value && !isLoading && (
          <button
            type="button"
            onClick={handleClear}
            className="p-1 rounded-full hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors"
          >
            <X className="w-4 h-4 text-slate-400" />
          </button>
        )}
      </div>

      {/* Suggestions Dropdown */}
      {showSuggestions && (
        <ul
          ref={listRef}
          id="suggestions-listbox"
          role="listbox"
          className={cn(
            "absolute z-50 w-full mt-2 py-2 rounded-lg border shadow-lg",
            "bg-white dark:bg-slate-800 border-slate-200 dark:border-slate-700",
            "max-h-80 overflow-auto"
          )}
        >
          {isLoading ? (
            <li className="px-4 py-3 text-sm text-slate-500 flex items-center gap-2">
              <Loader2 className="w-4 h-4 animate-spin" />
              搜尋中...
            </li>
          ) : (
            <>
              {suggestions.map((suggestion, index) => (
                <li
                  key={suggestion.id || index}
                  id={`suggestion-${index}`}
                  role="option"
                  aria-selected={selectedIndex === index}
                  onClick={() => onSelect?.(suggestion)}
                  className={cn(
                    "px-4 py-3 cursor-pointer transition-colors suggestion-item",
                    selectedIndex === index
                      ? "bg-primary/10 text-primary"
                      : "hover:bg-slate-50 dark:hover:bg-slate-700/50"
                  )}
                >
                  <div className="flex items-start gap-3">
                    <Package className="w-5 h-5 text-slate-400 mt-0.5 shrink-0" />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-medium text-slate-900 dark:text-slate-100 truncate">
                          {suggestion.name}
                        </span>
                        {suggestion.spec && (
                          <span className="text-sm text-slate-500 truncate">
                            {suggestion.spec}
                          </span>
                        )}
                      </div>
                      {suggestion.category && (
                        <span className="text-xs text-slate-400">
                          [{suggestion.category}]
                        </span>
                      )}
                    </div>
                    {suggestion.similarity !== undefined && (
                      <span className="text-xs font-medium text-slate-400">
                        {Math.round(suggestion.similarity * 100)}%
                      </span>
                    )}
                  </div>
                </li>
              ))}
              
              {/* Create New Option */}
              {onCreateNew && (
                <>
                  <li className="border-t border-slate-100 dark:border-slate-700 my-1" />
                  <li
                    id={`suggestion-${suggestions.length}`}
                    role="option"
                    aria-selected={selectedIndex === suggestions.length}
                    onClick={() => onCreateNew(value)}
                    className={cn(
                      "px-4 py-3 cursor-pointer transition-colors",
                      selectedIndex === suggestions.length
                        ? "bg-primary/10 text-primary"
                        : "hover:bg-slate-50 dark:hover:bg-slate-700/50"
                    )}
                  >
                    <div className="flex items-center gap-3">
                      <Plus className="w-5 h-5 text-primary" />
                      <span className="text-slate-700 dark:text-slate-300">
                        建立新產品「<span className="font-medium">{value}</span>」
                      </span>
                    </div>
                  </li>
                </>
              )}
            </>
          )}
        </ul>
      )}
    </div>
  )
}
