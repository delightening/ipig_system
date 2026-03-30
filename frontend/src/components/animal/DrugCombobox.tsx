/**
 * DrugCombobox — 藥物下拉搜尋選擇元件
 *
 * 功能：
 * - 輸入文字時即時搜尋篩選藥物列表
 * - 選擇藥物後自動帶入預設劑量單位
 * - 支援手動輸入藥名（不在選單中的藥物）
 * - 搭配劑量值 + 單位選擇
 */

import { useState, useRef, useEffect, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Search, X, Loader2, ChevronDown, Pill } from 'lucide-react'
import { cn } from '@/lib/utils'
import { treatmentDrugApi } from '@/lib/api'
import type { TreatmentDrugOption } from '@/types/treatment-drug'

export interface DrugSelection {
    /** 藥物選項 ID（從藥物庫選擇時） */
    drug_option_id?: string
    /** 藥物名稱 */
    drug_name: string
    /** 劑量值（如 5、0.2） */
    dosage_value: string
    /** 劑量單位（如 mg、ml、mg/kg） */
    dosage_unit: string
}

interface DrugComboboxProps {
    /** 目前的選擇值 */
    value: DrugSelection
    /** 值改變時觸發 */
    onChange: (value: DrugSelection) => void
    /** 是否顯示劑量輸入（預設 true） */
    showDosage?: boolean
    /** 是否停用 */
    disabled?: boolean
    /** 佔位文字 */
    placeholder?: string
    /** 額外 CSS 類名 */
    className?: string
}

export function DrugCombobox({
    value,
    onChange,
    showDosage = true,
    disabled = false,
    placeholder = '搜尋或輸入藥物名稱...',
    className,
}: DrugComboboxProps) {
    const [isFocused, setIsFocused] = useState(false)
    const [searchText, setSearchText] = useState('')
    const [selectedIndex, setSelectedIndex] = useState(-1)
    const inputRef = useRef<HTMLInputElement>(null)
    const dropdownRef = useRef<HTMLDivElement>(null)

    // 取得藥物選項列表
    const { data: drugOptions = [], isLoading } = useQuery({
        queryKey: ['treatment-drugs'],
        queryFn: async () => {
            const res = await treatmentDrugApi.list()
            return res.data
        },
        staleTime: 5 * 60 * 1000, // 5 分鐘快取
    })

    // 搜尋過濾
    const filteredOptions = useMemo(() => {
        if (!searchText) return drugOptions
        const keyword = searchText.toLowerCase()
        return drugOptions.filter(
            (opt) =>
                opt.name.toLowerCase().includes(keyword) ||
                opt.display_name?.toLowerCase().includes(keyword) ||
                opt.category?.toLowerCase().includes(keyword)
        )
    }, [drugOptions, searchText])

    // 是否顯示下拉
    const showDropdown = isFocused && (filteredOptions.length > 0 || isLoading)

    // 選擇藥物
    const handleSelect = (option: TreatmentDrugOption) => {
        onChange({
            drug_option_id: option.id,
            drug_name: option.display_name || option.name,
            dosage_value: option.default_dosage_value || value.dosage_value,
            dosage_unit: option.default_dosage_unit || value.dosage_unit || 'mg',
        })
        setSearchText('')
        setIsFocused(false)
        setSelectedIndex(-1)
    }

    // 手動輸入確認
    const handleManualConfirm = () => {
        if (searchText.trim()) {
            onChange({
                ...value,
                drug_option_id: undefined,
                drug_name: searchText.trim(),
            })
            setSearchText('')
            setIsFocused(false)
        }
    }

    // 鍵盤操作
    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (!showDropdown) {
            if (e.key === 'Enter') {
                e.preventDefault()
                handleManualConfirm()
            }
            return
        }

        switch (e.key) {
            case 'ArrowDown':
                e.preventDefault()
                setSelectedIndex((prev) =>
                    prev < filteredOptions.length - 1 ? prev + 1 : prev
                )
                break
            case 'ArrowUp':
                e.preventDefault()
                setSelectedIndex((prev) => (prev > 0 ? prev - 1 : 0))
                break
            case 'Enter':
                e.preventDefault()
                if (selectedIndex >= 0 && filteredOptions[selectedIndex]) {
                    handleSelect(filteredOptions[selectedIndex])
                } else {
                    handleManualConfirm()
                }
                break
            case 'Escape':
                setIsFocused(false)
                break
        }
    }

    // 清除選擇
    const handleClear = () => {
        onChange({
            drug_option_id: undefined,
            drug_name: '',
            dosage_value: '',
            dosage_unit: '',
        })
        setSearchText('')
        inputRef.current?.focus()
    }

    // 點擊外部關閉下拉
    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (
                dropdownRef.current &&
                !dropdownRef.current.contains(event.target as Node)
            ) {
                setIsFocused(false)
            }
        }
        document.addEventListener('mousedown', handleClickOutside)
        return () => document.removeEventListener('mousedown', handleClickOutside)
    }, [])

    // 取得選中藥物的可用單位
    const availableUnits = useMemo(() => {
        if (value.drug_option_id) {
            const selected = drugOptions.find((opt) => opt.id === value.drug_option_id)
            if (selected?.available_units) return selected.available_units
        }
        // 預設單位列表
        return ['mg', 'ml', 'mg/kg', 'cap', 'tab', 'cc', 'L/min', '%', 'cm', 'g', 'pcs']
    }, [value.drug_option_id, drugOptions])

    const displayValue = value.drug_name || ''

    return (
        <div className={cn('flex gap-2', className)} ref={dropdownRef}>
            {/* 藥物名稱搜尋 */}
            <div className="relative flex-1">
                <div
                    className={cn(
                        'relative flex items-center h-9 px-3 rounded-md border transition-all text-sm',
                        'bg-white',
                        isFocused
                            ? 'border-status-info-solid ring-1 ring-blue-500/30'
                            : 'border-border hover:border-slate-400',
                        disabled && 'opacity-50 cursor-not-allowed bg-muted'
                    )}
                >
                    {value.drug_name ? (
                        // 已選擇狀態
                        <>
                            <Pill className="w-4 h-4 text-status-info-solid mr-2 shrink-0" />
                            <span className="flex-1 truncate text-foreground">
                                {displayValue}
                            </span>
                            {!disabled && (
                                <button
                                    type="button"
                                    onClick={handleClear}
                                    className="p-0.5 rounded hover:bg-muted transition-colors"
                                >
                                    <X className="w-3.5 h-3.5 text-muted-foreground" />
                                </button>
                            )}
                        </>
                    ) : (
                        // 搜尋輸入狀態
                        <>
                            <Search className="w-4 h-4 text-muted-foreground mr-2 shrink-0" />
                            <input
                                ref={inputRef}
                                type="text"
                                value={searchText}
                                onChange={(e) => {
                                    setSearchText(e.target.value)
                                    setSelectedIndex(-1)
                                }}
                                onFocus={() => setIsFocused(true)}
                                onKeyDown={handleKeyDown}
                                placeholder={placeholder}
                                disabled={disabled}
                                className="flex-1 bg-transparent border-0 outline-none placeholder:text-muted-foreground text-foreground"
                                role="combobox"
                                aria-expanded={showDropdown}
                            />
                            {isLoading && (
                                <Loader2 className="w-4 h-4 text-muted-foreground animate-spin" />
                            )}
                            {!isLoading && (
                                <ChevronDown className="w-4 h-4 text-muted-foreground" />
                            )}
                        </>
                    )}
                </div>

                {/* 下拉選項 */}
                {showDropdown && (
                    <ul
                        role="listbox"
                        className="absolute z-50 w-full mt-1 py-1 rounded-md border shadow-lg bg-white border-border max-h-60 overflow-auto"
                    >
                        {isLoading ? (
                            <li className="px-3 py-2 text-sm text-muted-foreground flex items-center gap-2">
                                <Loader2 className="w-4 h-4 animate-spin" /> 載入中...
                            </li>
                        ) : filteredOptions.length === 0 ? (
                            <li className="px-3 py-2 text-sm text-muted-foreground">
                                無符合結果，按 Enter 使用自訂名稱
                            </li>
                        ) : (
                            filteredOptions.map((option, index) => (
                                <li
                                    key={option.id}
                                    role="option"
                                    aria-selected={selectedIndex === index}
                                    onClick={() => handleSelect(option)}
                                    className={cn(
                                        'px-3 py-2 cursor-pointer transition-colors text-sm',
                                        selectedIndex === index
                                            ? 'bg-status-info-bg text-status-info-text'
                                            : 'hover:bg-muted'
                                    )}
                                >
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-2 min-w-0">
                                            <span className="font-medium truncate">
                                                {option.display_name || option.name}
                                            </span>
                                            {option.category && (
                                                <span className="shrink-0 text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                                                    {option.category}
                                                </span>
                                            )}
                                        </div>
                                        {option.default_dosage_unit && (
                                            <span className="text-xs text-muted-foreground shrink-0 ml-2">
                                                {option.default_dosage_unit}
                                            </span>
                                        )}
                                    </div>
                                </li>
                            ))
                        )}
                    </ul>
                )}
            </div>

            {/* 劑量值 + 單位 */}
            {showDosage && (
                <>
                    <input
                        type="text"
                        value={value.dosage_value}
                        onChange={(e) =>
                            onChange({ ...value, dosage_value: e.target.value })
                        }
                        placeholder="劑量"
                        disabled={disabled}
                        className="w-20 h-9 px-2 rounded-md border border-border text-sm text-center
                       focus:border-status-info-solid focus:ring-1 focus:ring-primary/30 outline-none
                       disabled:opacity-50 disabled:bg-muted"
                    />
                    <select
                        value={value.dosage_unit}
                        onChange={(e) =>
                            onChange({ ...value, dosage_unit: e.target.value })
                        }
                        disabled={disabled}
                        className="w-20 h-9 px-1 rounded-md border border-border text-sm
                       focus:border-status-info-solid focus:ring-1 focus:ring-primary/30 outline-none
                       disabled:opacity-50 disabled:bg-muted"
                    >
                        <option value="">單位</option>
                        {availableUnits.map((unit) => (
                            <option key={unit} value={unit}>
                                {unit}
                            </option>
                        ))}
                    </select>
                </>
            )}
        </div>
    )
}
