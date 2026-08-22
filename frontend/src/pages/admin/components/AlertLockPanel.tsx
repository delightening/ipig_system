import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { LockOpen, ShieldOff } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { alertLockApi } from '@/lib/api/alertLock'
import { ipBlocklistApi } from '@/lib/api/ipBlocklist'
import { formatDateTime } from '@/lib/utils'

interface AlertLockPanelProps {
    alertId: string
}

/**
 * 警報對象目前的鎖定狀態與解除入口。
 *
 * 刻意與「標記為已解決」分開兩顆按鈕：關警報是「我看過了」的稽核動作，
 * 解鎖是放行安全控制。真的遇到爆破攻擊時，關警報不該順手把門打開。
 */
export function AlertLockPanel({ alertId }: AlertLockPanelProps) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const [reason, setReason] = useState('')

    const { data: lock, isError } = useQuery({
        queryKey: ['alert-lock-status', alertId],
        queryFn: async () => (await alertLockApi.status(alertId)).data,
    })

    const invalidate = () => {
        queryClient.invalidateQueries({ queryKey: ['alert-lock-status', alertId] })
        queryClient.invalidateQueries({ queryKey: ['ip-blocklist'] })
        setReason('')
    }

    const unlockAccount = useMutation({
        mutationFn: (email: string) => alertLockApi.clearAccountLockout(email, reason),
        onSuccess: () => {
            toast({ title: t('admin.alertLockPanel.toastAccountUnlocked') })
            invalidate()
        },
    })

    const unblockIp = useMutation({
        mutationFn: (id: string) => ipBlocklistApi.unblock(id, reason),
        onSuccess: () => {
            toast({ title: t('admin.alertLockPanel.toastIpUnblocked') })
            invalidate()
        },
    })

    // 查詢失敗與「查得到但沒鎖」都會讓面板消失，但兩者意義相反：後者是「確定
    // 沒鎖」，前者是「不知道有沒有鎖」。靜默 return null 會讓管理員以為門是開的，
    // 於是不去解鎖——而帳號其實還鎖著。
    // 條件帶 `!lock`：refetch 失敗時 `isError` 為 true 但快取狀態仍在，那種情況
    // 顯示既有狀態比換成一行錯誤字有用。
    if (isError && !lock) {
        return (
            <div className="rounded-md border border-border bg-muted/30 p-4">
                <p className="text-sm text-status-error-text">{t('common.loadFailed')}</p>
            </div>
        )
    }

    if (!lock) return null

    const account = lock.account
    const ip = lock.ip
    const accountLocked = account?.locked === true
    const hasAction = accountLocked || ip !== null
    // 兩者都沒鎖就不佔版面——但帳號查得到卻沒鎖時仍顯示一行，
    // 讓管理員知道「不用解，本來就進得去」而不是功能壞了
    if (!hasAction && !account) return null

    const reasonMissing = reason.trim().length === 0

    return (
        <div className="rounded-md border border-border bg-muted/30 p-4 space-y-3">
            <Label className="font-medium">{t('admin.alertLockPanel.title')}</Label>

            {account && (
                <div className="flex flex-wrap items-center gap-2 text-sm">
                    <Badge variant={accountLocked ? 'destructive' : 'secondary'}>
                        {accountLocked
                            ? t('admin.alertLockPanel.accountLocked')
                            : t('admin.alertLockPanel.accountUnlocked')}
                    </Badge>
                    <span className="font-mono break-all">{account.email}</span>
                    {!account.user_exists && (
                        <span className="text-muted-foreground">
                            {t('admin.alertLockPanel.accountNotFound')}
                        </span>
                    )}
                    {accountLocked && (
                        <span className="text-muted-foreground">
                            {t('admin.alertLockPanel.accountFailCount', {
                                count: account.fail_count,
                                max: account.max_attempts,
                            })}
                            {account.unlock_at &&
                                t('admin.alertLockPanel.accountAutoUnlockAt', {
                                    time: formatDateTime(account.unlock_at),
                                })}
                        </span>
                    )}
                </div>
            )}

            {ip && (
                <div className="flex flex-wrap items-center gap-2 text-sm">
                    <Badge variant="destructive">{t('admin.alertLockPanel.ipBlocked')}</Badge>
                    <span className="font-mono break-all">{ip.ip_address}</span>
                    <span className="text-muted-foreground break-all">{ip.reason}</span>
                </div>
            )}

            {hasAction && (
                <div className="space-y-2">
                    <Label htmlFor="alert-unlock-reason" className="text-muted-foreground">
                        {t('admin.alertLockPanel.reasonLabel')}
                    </Label>
                    <Input
                        id="alert-unlock-reason"
                        value={reason}
                        onChange={(e) => setReason(e.target.value)}
                        placeholder={t('admin.alertLockPanel.reasonPlaceholder')}
                    />
                    <div className="flex flex-wrap gap-2">
                        {accountLocked && account && (
                            <Button
                                variant="outline"
                                size="sm"
                                disabled={reasonMissing || unlockAccount.isPending}
                                onClick={() => unlockAccount.mutate(account.email)}
                            >
                                <LockOpen className="h-4 w-4 mr-1" />
                                {t('admin.alertLockPanel.unlockAccount')}
                            </Button>
                        )}
                        {ip && (
                            <Button
                                variant="outline"
                                size="sm"
                                disabled={reasonMissing || unblockIp.isPending}
                                onClick={() => unblockIp.mutate(ip.id)}
                            >
                                <ShieldOff className="h-4 w-4 mr-1" />
                                {t('admin.alertLockPanel.unblockIp')}
                            </Button>
                        )}
                    </div>
                </div>
            )}
        </div>
    )
}
