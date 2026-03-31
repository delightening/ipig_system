// 效期通知範圍設定 Panel

import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import { useToast } from '@/components/ui/use-toast'
import { Loader2, Info } from 'lucide-react'
import { expiryConfigApi } from '@/lib/api/notification'
import { getApiErrorMessage } from '@/lib/validation'
import type { UpdateExpiryNotificationConfigRequest } from '@/types/notification'

export function ExpiryConfigPanel() {
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
            toast({ title: '成功', description: '效期通知設定已更新' })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '更新失敗'), variant: 'destructive' })
        },
    })

    const handleSave = () => {
        if (warnDays < 1 || warnDays > 365) {
            toast({ title: '錯誤', description: '提前預警天數需介於 1-365', variant: 'destructive' })
            return
        }
        if (cutoffDays < 1 || cutoffDays > 730) {
            toast({ title: '錯誤', description: '過期截止天數需介於 1-730', variant: 'destructive' })
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
                <CardTitle className="text-base">效期通知範圍設定</CardTitle>
            </CardHeader>
            <CardContent className="space-y-5">
                {isLoading ? (
                    <div className="flex items-center gap-2 text-muted-foreground py-4">
                        <Loader2 className="h-4 w-4 animate-spin" />
                        <span className="text-sm">載入中...</span>
                    </div>
                ) : (
                    <>
                        {/* 提前預警天數 */}
                        <div className="grid grid-cols-[1fr_auto] items-center gap-4">
                            <div className="space-y-1">
                                <Label className="text-sm">提前預警天數</Label>
                                <p className="text-xs text-muted-foreground">效期前幾天開始顯示預警（1-365）</p>
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
                                <span className="text-sm text-muted-foreground shrink-0">天</span>
                            </div>
                        </div>

                        {/* 過期截止天數 */}
                        <div className="grid grid-cols-[1fr_auto] items-center gap-4">
                            <div className="space-y-1">
                                <Label className="text-sm">過期截止天數</Label>
                                <p className="text-xs text-muted-foreground">過期超過幾天後停止通知（1-730）</p>
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
                                <span className="text-sm text-muted-foreground shrink-0">天</span>
                            </div>
                        </div>

                        {/* 月度彙整模式 */}
                        <div className="space-y-3 rounded-md border p-4 bg-muted/20">
                            <div className="flex items-center justify-between">
                                <div className="space-y-0.5">
                                    <Label className="text-sm">月度彙整通知</Label>
                                    <p className="text-xs text-muted-foreground">
                                        過期超過指定天數後，改為每月一次彙整通知並與上月比較
                                    </p>
                                </div>
                                <Switch
                                    checked={monthlyEnabled}
                                    onCheckedChange={setMonthlyEnabled}
                                />
                            </div>
                            {monthlyEnabled && (
                                <div className="flex items-center gap-3 pt-1">
                                    <Label className="text-sm shrink-0">過期超過</Label>
                                    <Input
                                        type="number"
                                        min={1}
                                        max={cutoffDays}
                                        value={monthlyDays}
                                        onChange={(e) => setMonthlyDays(Number(e.target.value))}
                                        className="w-20 h-8 text-sm"
                                    />
                                    <span className="text-sm text-muted-foreground">天後轉為月度彙整</span>
                                </div>
                            )}
                        </div>

                        {/* 現行設定說明 */}
                        {config && (
                            <div className="flex items-start gap-2 text-xs text-muted-foreground bg-muted/30 rounded-md p-3">
                                <Info className="h-3.5 w-3.5 mt-0.5 shrink-0" />
                                <span>
                                    現行設定：效期前 <strong>{config.warn_days}</strong> 天開始預警，
                                    過期超過 <strong>{config.cutoff_days}</strong> 天停止通知
                                    {config.monthly_threshold_days !== null && (
                                        <>，過期超過 <strong>{config.monthly_threshold_days}</strong> 天轉月度彙整</>
                                    )}
                                </span>
                            </div>
                        )}

                        <Button size="sm" onClick={handleSave} disabled={updateMutation.isPending}>
                            {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                            儲存設定
                        </Button>
                    </>
                )}
            </CardContent>
        </Card>
    )
}
