// 置於「基本資料」下方的 sticky 進度/儲存列：顯示必填完整度進度條（已完成/總數 +
// 百分比）+ 主要儲存按鈕；向下捲動時吸附在 app top bar 下方（top-14 / md:top-16，
// 對齊 SparePoolPanel 慣例）。必填未完成時停用按鈕。僅於編輯模式渲染。
import { useTranslation } from 'react-i18next'
import { Save, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface Props {
    doneCount: number
    totalCount: number
    percent: number
    isComplete: boolean
    isSaving: boolean
    onSave: () => void
}

export function ProfileSaveBar({ doneCount, totalCount, percent, isComplete, isSaving, onSave }: Props) {
    const { t } = useTranslation()

    return (
        <div className="sticky top-14 z-20 rounded-xl border border-border bg-background/95 px-4 py-3 shadow-sm backdrop-blur supports-[backdrop-filter]:bg-background/80 md:top-16">
            <div className="flex flex-col items-center gap-4 sm:flex-row">
                <div className="flex-1 w-full space-y-1.5">
                    <div className="flex items-center justify-between text-sm">
                        <span className="font-medium text-foreground">{t('profile.completeness')}</span>
                        <span className={cn('font-medium', isComplete ? 'text-status-success-text' : 'text-muted-foreground')}>
                            {doneCount}/{totalCount} · {percent}%
                        </span>
                    </div>
                    <div className="h-2 w-full rounded-full bg-muted overflow-hidden">
                        <div
                            className={cn('h-full rounded-full transition-all', isComplete ? 'bg-status-success-text' : 'bg-primary')}
                            style={{ width: `${percent}%` }}
                        />
                    </div>
                    <p className={cn('text-xs', isComplete ? 'text-status-success-text' : 'text-status-warning-text')}>
                        {isComplete ? t('profile.readyToSave') : t('profile.requiredIncomplete')}
                    </p>
                </div>
                <Button
                    size="lg"
                    className="w-full sm:w-auto shrink-0"
                    onClick={onSave}
                    disabled={!isComplete || isSaving}
                >
                    {isSaving ? (
                        <Loader2 className="mr-2 h-5 w-5 animate-spin" />
                    ) : (
                        <Save className="mr-2 h-5 w-5" />
                    )}
                    {isSaving ? t('profile.saving') : t('common.saveChanges')}
                </Button>
            </div>
        </div>
    )
}
