import { Lock } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import {
    Card,
    CardHeader,
    CardTitle,
    CardDescription,
    CardContent,
} from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'

import type { FixedNotification } from '../types'
import { ChannelBadge } from './ChannelBadge'

interface FixedNotificationsSectionProps {
    items?: FixedNotification[]
}

/**
 * 固定通知（系統流程決定收件人、不經路由、不可調整）的唯讀檢視。
 * 讓管理者在同一頁看見「有什麼狀況會通知誰」，包含寫死的通知路徑。
 */
export function FixedNotificationsSection({ items }: FixedNotificationsSectionProps) {
    const { t } = useTranslation()
    if (!items || items.length === 0) return null

    return (
        <Card>
            <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                    <Lock className="h-4 w-4 text-muted-foreground" />
                    {t('adminOps.notificationRouting.fixed.title')}
                </CardTitle>
                <CardDescription>
                    {t('adminOps.notificationRouting.fixed.description')}
                </CardDescription>
            </CardHeader>
            <CardContent>
                <div className="rounded-lg border bg-card overflow-hidden">
                    <Table>
                        <TableHeader>
                            <TableRow className="bg-muted/50 hover:bg-muted/50">
                                <TableHead className="w-[180px]">{t('adminOps.notificationRouting.fixed.colEvent')}</TableHead>
                                <TableHead className="w-[200px]">{t('adminOps.notificationRouting.fixed.colTrigger')}</TableHead>
                                <TableHead>{t('adminOps.notificationRouting.fixed.colRecipient')}</TableHead>
                                <TableHead className="w-[140px]">{t('adminOps.notificationRouting.channelLabel')}</TableHead>
                                <TableHead>{t('adminOps.notificationRouting.fixed.colNote')}</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {items.map((item, idx) => (
                                <TableRow key={`${item.event_label}-${idx}`}>
                                    <TableCell className="font-medium">{item.event_label}</TableCell>
                                    <TableCell className="text-sm text-muted-foreground">
                                        {item.trigger}
                                    </TableCell>
                                    <TableCell className="text-sm">{item.recipient_label}</TableCell>
                                    <TableCell>
                                        <ChannelBadge channel={item.channel} />
                                    </TableCell>
                                    <TableCell className="text-xs text-muted-foreground">
                                        {item.note}
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </div>
            </CardContent>
        </Card>
    )
}
