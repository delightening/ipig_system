/**
 * 「本場受僱人員 / 外部人員」二擇一欄位。
 *
 * 建立使用者與建立邀請兩個對話框共用——這兩條路徑先前對這件事做出**相反的
 * 假設而且都不問**（建立走後端預設 true、邀請硬編 false），造成 9 位在職同仁
 * 的分類與其角色矛盾。把選項與說明集中在這裡，是為了避免兩邊的文案與預選
 * 規則再度分岔。
 */
import { useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { Label } from '@/components/ui/label'
import { cn } from '@/lib/utils'
import { deriveAffiliation } from '@/lib/staffAffiliation'

interface StaffAffiliationFieldProps {
    /** 目前值；`null` 代表尚未選擇 */
    value: boolean | null
    /** `null` 代表「回到未選擇」——角色改成無法判定時必須能清掉舊的推導值 */
    onChange: (next: boolean | null) => void
    /** 已勾選角色的 code，用於自動預選 */
    selectedRoleCodes: string[]
    /** 送出時未選會顯示的錯誤 */
    error?: string
}

// module 級常數存 i18n 鍵、渲染時才 t()（頂層存翻譯後字串會讓語言切換後不更新）
const OPTIONS = [
    {
        value: true,
        titleKey: 'adminUsers.shared.affiliation.internal',
        detailKey: 'adminUsers.shared.affiliation.internalDetail',
    },
    {
        value: false,
        titleKey: 'adminUsers.shared.affiliation.external',
        detailKey: 'adminUsers.shared.affiliation.externalDetail',
    },
] as const

export function StaffAffiliationField({
    value,
    onChange,
    selectedRoleCodes,
    error,
}: StaffAffiliationFieldProps) {
    const { t } = useTranslation()
    // 使用者是否已「自己點過」。
    //
    // ⚠️ 不能用 `value !== null` 代替：那分不出「自動推導出來的值」與「使用者
    // 明確選的值」。先前就是這樣寫的，結果是**第一次推導之後所有角色變更都被
    // 忽略**——先選 EXPERIMENT_STAFF（推導成受僱人員）再改成 PI，畫面仍停在
    // 受僱人員並就這樣送出，而且不會有任何提示。
    const touchedRef = useRef(false)

    // 角色變動時重新推導，除非使用者已經自己選過。
    // 推導不出來（曖昧角色、內外部混選）就清回未選擇，強迫重新判斷——
    // 留著上一次的推導值比空白更危險。
    useEffect(() => {
        if (touchedRef.current) return
        const derived = deriveAffiliation(selectedRoleCodes)
        if (derived !== value) onChange(derived)
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [selectedRoleCodes.join(',')])

    const handlePick = (next: boolean) => {
        touchedRef.current = true
        onChange(next)
    }

    return (
        <div className="space-y-2">
            <Label>
                {t('adminUsers.shared.affiliation.label')} <span className="text-destructive">*</span>
            </Label>
            <div className="grid gap-2 sm:grid-cols-2">
                {OPTIONS.map(opt => {
                    const selected = value === opt.value
                    return (
                        <button
                            key={String(opt.value)}
                            type="button"
                            role="radio"
                            aria-checked={selected}
                            onClick={() => handlePick(opt.value)}
                            className={cn(
                                'rounded-md border p-3 text-left transition-colors',
                                selected
                                    ? 'border-primary bg-primary/5 ring-1 ring-primary'
                                    : 'border-border hover:border-foreground/40',
                            )}
                        >
                            <div className="text-sm font-medium">{t(opt.titleKey)}</div>
                            <div className="mt-1 text-xs text-muted-foreground">{t(opt.detailKey)}</div>
                        </button>
                    )
                })}
            </div>
            {/* 說明為什麼要分——只寫「內部 vs 外部」等於同義反覆，講後果才有用 */}
            <p className="text-xs text-muted-foreground">
                {t('adminUsers.shared.affiliation.note')}
            </p>
            {error && <p className="text-xs text-destructive">{error}</p>}
        </div>
    )
}
