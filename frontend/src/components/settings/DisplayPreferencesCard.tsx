// 顯示偏好卡片（恆顯，兩種模式皆渲染）：歡迎指引開關 + 字體大小。
// 自 ProfileSettingsPage 抽出，行為不變（偏好查詢/變更與字體邏輯內聚於此）。
import { useTranslation } from 'react-i18next'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Sparkles } from 'lucide-react'
import api from '@/lib/api'
import { useWelcomeGuidePref, WELCOME_GUIDE_PREF_KEY } from '@/hooks/useWelcomeGuidePref'
import { useUIPreferences, type FontSizePreference } from '@/stores/uiPreferences'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { useToast } from '@/components/ui/use-toast'
import { cn } from '@/lib/utils'

export function DisplayPreferencesCard() {
    const { t } = useTranslation()
    const { fontSize, setFontSize } = useUIPreferences()
    const { toast } = useToast()
    const queryClient = useQueryClient()

    // 與儀表板橫幅共用同一支偏好查詢（同 key，React Query 自動去重）
    const { enabled: welcomeGuideEnabled } = useWelcomeGuidePref()

    const toggleWelcomeGuideMutation = useMutation({
        mutationFn: async (enabled: boolean) => {
            return api.put('/me/preferences/show_welcome_guide', { value: enabled })
        },
        onSuccess: (_data, enabled) => {
            queryClient.setQueryData(WELCOME_GUIDE_PREF_KEY, enabled)
            toast({
                title: t('common.success'),
                description: enabled ? t('profile.welcomeGuideEnabled') : t('profile.welcomeGuideDisabled'),
            })
        },
    })

    const fontSizeOptions: { value: FontSizePreference; label: string }[] = [
        { value: 'default', label: t('profile.fontSizeDefault') },
        { value: 'large', label: t('profile.fontSizeLarge') },
        { value: 'xl', label: t('profile.fontSizeXl') },
    ]

    return (
        <Card>
            <CardHeader className="border-b bg-muted/50">
                <CardTitle className="flex items-center gap-2">
                    <Sparkles className="h-5 w-5 text-primary" />
                    {t('profile.displayPreferences')}
                </CardTitle>
            </CardHeader>
            <CardContent className="pt-4 space-y-5">
                <div
                    className={cn(
                        'flex items-center justify-between p-3 rounded-lg border transition-all cursor-pointer',
                        welcomeGuideEnabled
                            ? 'bg-primary/5 border-primary/20 ring-1 ring-primary/20'
                            : 'hover:bg-muted border-border',
                    )}
                    onClick={() => toggleWelcomeGuideMutation.mutate(!welcomeGuideEnabled)}
                >
                    <div className="flex items-center gap-3">
                        <Checkbox
                            id="show_welcome_guide"
                            checked={welcomeGuideEnabled}
                            onCheckedChange={checked => toggleWelcomeGuideMutation.mutate(!!checked)}
                        />
                        <label htmlFor="show_welcome_guide" className="text-sm font-medium leading-none cursor-pointer">
                            {t('profile.showWelcomeGuide')}
                        </label>
                    </div>
                </div>
                <p className="text-xs text-muted-foreground mt-2 ml-1">
                    {t('profile.showWelcomeGuideDescription')}
                </p>

                <div className="space-y-2">
                    <label className="text-sm font-medium">{t('profile.fontSize')}</label>
                    <div className="flex gap-2">
                        {fontSizeOptions.map(({ value, label }) => (
                            <button
                                key={value}
                                type="button"
                                onClick={() => setFontSize(value)}
                                className={cn(
                                    'flex-1 py-2 rounded-lg border text-sm font-medium transition-all',
                                    fontSize === value
                                        ? 'bg-primary text-primary-foreground border-primary'
                                        : 'border-border hover:bg-muted',
                                )}
                            >
                                {label}
                            </button>
                        ))}
                    </div>
                    <p className="text-xs text-muted-foreground">{t('profile.fontSizeDescription')}</p>
                </div>
            </CardContent>
        </Card>
    )
}
