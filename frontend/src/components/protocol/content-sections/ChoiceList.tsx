import { useTranslation } from 'react-i18next'
import { Check } from 'lucide-react'

/**
 * 計劃書 HTML 唯讀檢視「顯示未選選項」共用呈現元件。
 *
 * 列出某選擇欄位的「完整選項」，已選實色 ☑、未選淡灰 ☐，讓審查員看見申請者
 * 「選了哪個、沒選哪些」（見 docs/design/protocol-unselected-options-spec.md）。
 * Boolean / 單選 / 複選共用：boolean 與單選即 selectedValues 長度 0~1。
 * 樣式一律走 DESIGN.md token（primary / muted-foreground），不硬編色。
 */
export interface ChoiceListItem {
    value: string
    /** i18n key（與 label 二擇一） */
    labelKey?: string
    /** 直接給定的文字（與 labelKey 二擇一，如「是 / 否」） */
    label?: string
}

interface ChoiceListProps {
    /** 完整選項；ProtocolChoiceOption[] 亦可賦值（labelKey 必填即滿足此型別） */
    options: ChoiceListItem[]
    /** 已選值；單選/boolean 傳 0~1 個，複選傳多個 */
    selectedValues: string[]
}

/**
 * 已選 ☑（實色）/ 未選 ☐（淡灰）勾選方塊。供 ChoiceList 與需自訂列內容的
 * 區塊（如 Guidelines 12 資料庫含關鍵字/備註）共用，確保視覺一致（DRY）。
 */
export function CheckIndicator({ on }: { on: boolean }) {
    return (
        <span
            className={`mt-0.5 inline-flex h-4 w-4 shrink-0 items-center justify-center rounded-sm border ${
                on ? 'border-primary bg-primary text-primary-foreground' : 'border-muted-foreground/40'
            }`}
            aria-hidden
        >
            {on && <Check className="h-3 w-3" />}
        </span>
    )
}

export function ChoiceList({ options, selectedValues }: ChoiceListProps) {
    const { t } = useTranslation()
    const selected = new Set(selectedValues)

    return (
        <ul className="space-y-1 text-sm">
            {options.map((opt) => {
                const on = selected.has(opt.value)
                const label = opt.label || (opt.labelKey ? t(opt.labelKey) : '')
                return (
                    <li
                        key={opt.value}
                        className={`flex items-start gap-2 ${on ? 'text-foreground' : 'text-muted-foreground/60'}`}
                    >
                        <CheckIndicator on={on} />
                        <span className={on ? 'font-medium' : ''}>
                            {label}
                            <span className="sr-only">{on ? '（已選擇）' : '（未選擇）'}</span>
                        </span>
                    </li>
                )
            })}
        </ul>
    )
}
