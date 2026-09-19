// 效期通知範圍設定 Panel

import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Trans, useTranslation } from 'react-i18next'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import { useToast } from '@/components/ui/use-toast'
import { Loader2, Info } from 'lucide-react'
import { expiryConfigApi } from '@/lib/api/notification'
import { getApiErrorMessage } from '@/lib/apiError'
import type { UpdateExpiryNotificationConfigRequest } from '@/types/notification'

export function ExpiryConfigPanel() {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const { toast } = useToast()

    const { data: config, isLoading } = useQuery({
        queryKey: ['expiry-notification-config'],
        queryFn: async () => {
            const res = await expiryConfigApi.get()
            return res.data
        },
    })

    const [warnDays, setWarnDays] = useState(60)
    const [cutoffDays, setCutoffDays] = useState(90)
    const [monthlyEnabled, setMonthlyEnabled] = useState(false)
    const [monthlyDays, setMonthlyDays] = useState(30)

    useEffect(() => {
        if (!config) return
        setWarnDays(config.warn_days)
        setCutoffDays(config.cutoff_days)
        setMonthlyEnabled(config.monthly_threshold_days !== null)
        setMonthlyDays(config.monthly_threshold_days ?? 30)
    }, [config])

    const updateMutation = useMutation({
        mutationFn: (data: UpdateExpiryNotificationConfigRequest) => expiryConfigApi.update(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['expiry-notification-config'] })
            toast({ title: t('common.success'), description: t('adminOps.notificationRouting.expiry.updated') })
        },
        onError: (error: unknown) => {
            toast({ title: t('common.error'), description: getApiErrorMessage(error, t('adminOps.shared.updateFailed')), variant: 'destructive' })
        },
    })

    const handleSave = () => {
        if (warnDays < 1 || warnDays > 365) {
            toast({ title: t('common.error'), description: t('adminOps.notificationRouting.expiry.warnDaysRange'), variant: 'destructive' })
            return
        }
        if (cutoffDays < 1 || cutoffDays > 730) {
            toast({ title: t('common.error'), description: t('adminOps.notificationRouting.expiry.cutoffDaysRange'), variant: 'destructive' })
            return
        }
        updateMutation.mutate({
            warn_days: warnDays,
            cutoff_days: cutoffDays,
            monthly_threshold_days: monthlyEnabled ? monthlyDays : null,
        })
    }

    return (
        <Card>
            <CardHeader className="pb-3">
                <CardTitle className="text-base">{t('adminOps.notificationRouting.expiry.title')}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-5">
                {isLoading ? (
                    <div className="flex items-center gap-2 text-muted-foreground py-4">
                        <Loader2 className="h-4 w-4 animate-spin" />
                        <span className="text-sm">{t('common.loading')}</span>
                    </div>
                ) : (
                    <>
                        {/* 提前預警天數 */}
                        <div className="grid grid-cols-[1fr_auto] items-center gap-4">
                            <div className="space-y-1">
                                <Label className="text-sm">{t('adminOps.notificationRouting.expiry.warnDaysLabel')}</Label>
                                <p className="text-xs text-muted-foreground">{t('adminOps.notificationRouting.expiry.warnDaysHint')}</p>
                            </div>
                            <div className="flex items-center gap-2">
                                <Input
                                    type="number"
                                    min={1}
                                    max={365}
                                    value={warnDays}
                                    onChange={(e) => setWarnDays(Number(e.target.value))}
                                    className="w-20 h-8 text-sm"
                                />
                                <span className="text-sm text-muted-foreground shrink-0">{t('adminOps.notificationRouting.expiry.daysUnit')}</span>
                            </div>
                        </div>

                        {/* 過期截止天數 */}
                        <div className="grid grid-cols-[1fr_auto] items-center gap-4">
                            <div className="space-y-1">
                                <Label className="text-sm">{t('adminOps.notificationRouting.expiry.cutoffDaysLabel')}</Label>
                                <p className="text-xs text-muted-foreground">{t('adminOps.notificationRouting.expiry.cutoffDaysHint')}</p>
                            </div>
                            <div className="flex items-center gap-2">
                                <Input
                                    type="number"
                                    min={1}
                                    max={730}
                                    value={cutoffDays}
                                    onChange={(e) => setCutoffDays(Number(e.target.value))}
                                    className="w-20 h-8 text-sm"
                                />
                                <span className="text-sm text-muted-foreground shrink-0">{t('adminOps.notificationRouting.expiry.daysUnit')}</span>
                            </div>
                        </div>

                        {/* 月度彙整模式 */}
                        <div className="space-y-3 rounded-md border p-4 bg-muted/20">
                            <div className="flex items-center justify-between">
                                <div className="space-y-0.5">
                                    <Label className="text-sm">{t('adminOps.notificationRouting.expiry.monthlyLabel')}</Label>
                                    <p className="text-xs text-muted-foreground">
                                        {t('adminOps.notificationRouting.expiry.monthlyHint')}
                                    </p>
                                </div>
                                <Switch
                                    checked={monthlyEnabled}
                                    onCheckedChange={setMonthlyEnabled}
                                />
                            </div>
                            {monthlyEnabled && (
                                <div className="flex items-center gap-3 pt-1">
                                    <Label className="text-sm shrink-0">{t('adminOps.notificationRouting.expiry.monthlyPrefix')}</Label>
                                    <Input
                                        type="number"
                                        min={1}
                                        max={cutoffDays}
                                        value={monthlyDays}
                                        onChange={(e) => setMonthlyDays(Number(e.target.value))}
                                        className="w-20 h-8 text-sm"
                                    />
                                    <span className="text-sm text-muted-foreground">{t('adminOps.notificationRouting.expiry.monthlySuffix')}</span>
                                </div>
                            )}
                        </div>

                        {/* 現行設定說明 */}
                        {config && (
                            <div className="flex items-start gap-2 text-xs text-muted-foreground bg-muted/30 rounded-md p-3">
                                <Info className="h-3.5 w-3.5 mt-0.5 shrink-0" />
                                <span>
                                    <Trans
                                        i18nKey="adminOps.notificationRouting.expiry.currentSummary"
                                        values={{ warnDays: config.warn_days, cutoffDays: config.cutoff_days }}
                                        components={{ strong: <strong /> }}
                                    />
                                    {config.monthly_threshold_days !== null && (
                                        <Trans
                                            i18nKey="adminOps.notificationRouting.expiry.currentSummaryMonthly"
                                            values={{ days: config.monthly_threshold_days }}
                                            components={{ strong: <strong /> }}
                                        />
                                    )}
                                </span>
                            </div>
                        )}

                        <Button size="sm" onClick={handleSave} disabled={updateMutation.isPending}>
                            {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                            {t('adminOps.notificationRouting.expiry.saveButton')}
                        </Button>
                    </>
                )}
            </CardContent>
        </Card>
    )
}
