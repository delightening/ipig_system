// 圖片說明輸入框（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import { useState, useEffect, useRef } from 'react'
import { Input } from '@/components/ui/input'

/**
 * 圖片說明（caption）輸入框。
 *
 * 用本地 state 而非每次 keystroke 寫回 react-query 快取——後者的間接 re-render 會打斷
 * 中文輸入法（IME）組字，導致注音符號直接外漏（打出注音文）。本地 state 與同 dialog 內
 * 觀察/建議欄位（純 useState）一致，IME 正常；僅在 onBlur 失焦時才呼叫 onSave 存檔。
 */
export function CaptionInput({
    value,
    placeholder,
    className,
    onSave,
    disabled,
}: {
    value: string
    placeholder: string
    className?: string
    onSave: (caption: string) => void
    /** 無照片管理權限（非獸醫 / 已鎖定階段）時唯讀——後端 update caption 為 vet-only */
    disabled?: boolean
}) {
    const [draft, setDraft] = useState(value)
    const focusedRef = useRef(false)
    // 是否有「使用者實際編輯過、尚未存檔」的本地變更。
    const dirtyRef = useRef(false)

    // 伺服器值在「非編輯中且無未存變更」時同步本地草稿（存檔成功 refetch）；
    // 編輯中或有未存變更時不同步，避免 refetch 回來的值蓋掉使用者正在打的字。
    useEffect(() => {
        if (!focusedRef.current && !dirtyRef.current) setDraft(value)
    }, [value])

    return (
        <Input
            value={draft}
            placeholder={placeholder}
            className={className}
            disabled={disabled}
            onChange={(e) => { dirtyRef.current = true; setDraft(e.target.value) }}
            onFocus={() => { focusedRef.current = true }}
            onBlur={() => {
                focusedRef.current = false
                if (dirtyRef.current && draft !== value) {
                    // 使用者實際改過字 → 存檔
                    onSave(draft)
                } else {
                    // 未編輯就失焦：同步可能在聚焦期間被背景 refetch 更新的伺服器值，
                    // 確保顯示最新內容，且不會用舊草稿蓋掉伺服器。
                    setDraft(value)
                }
                dirtyRef.current = false
            }}
        />
    )
}
