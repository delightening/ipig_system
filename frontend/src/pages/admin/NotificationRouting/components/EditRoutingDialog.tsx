import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Loader2 } from 'lucide-react'

import type { NotificationRouting, UpdateRoutingData } from '../types'
import { channelOptions, recipientLabel } from '../constants'
import { BATCH_EVENT_TYPES } from '@/types/notification'

const FREQUENCY_OPTIONS = [
    { value: 'daily', labelKey: 'adminOps.notificationRouting.frequency.daily' },
    { value: 'weekly', labelKey: 'adminOps.notificationRouting.frequency.weekly' },
    { value: 'monthly', labelKey: 'adminOps.notificationRouting.frequency.monthly' },
] as const

/** 星期日(0)～星期六(6)，順序對應 day_of_week 數值；顯示文字走 i18n。 */
const DOW_KEYS = ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'] as const

interface EditRoutingDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    selectedRule: NotificationRouting | null
    form: UpdateRoutingData
    onFormChange: (form: UpdateRoutingData) => void
    onSubmit: () => void
    isPending: boolean
    eventNameMap: Record<string, string>
    roleNameMap: Record<string, string>
}

export function EditRoutingDialog({
    open,
    onOpenChange,
    selectedRule,
    form,
    onFormChange,
    onSubmit,
    isPending,
    eventNameMap,
    roleNameMap,
}: EditRoutingDialogProps) {
    const { t } = useTranslation()
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>{t('adminOps.notificationRouting.editDialog.title')}</DialogTitle>
                    <DialogDescription>
                        {selectedRule && (
                            <>
                                {eventNameMap[selectedRule.event_type] || selectedRule.event_type}
                                {' → '}
                                {recipientLabel(selectedRule, roleNameMap, t)}
                            </>
                        )}
                    </DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-4">
                    <div className="space-y-2">
                        <Label>{t('adminOps.notificationRouting.channelLabel')}</Label>
                        <Select
                            value={form.channel}
                            onValueChange={(v) => onFormChange({ ...form, channel: v })}
                        >
                            <SelectTrigger>
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                {channelOptions.map((opt) => (
                                    <SelectItem key={opt.value} value={opt.value}>
                                        {t(opt.labelKey)}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>

                    <div className="flex items-center justify-between">
                        <Label>{t('adminOps.notificationRouting.editDialog.activeLabel')}</Label>
                        <Switch
                            checked={form.is_active ?? true}
                            onCheckedChange={(checked) => onFormChange({ ...form, is_active: checked })}
                        />
                    </div>

                    <div className="space-y-2">
                        <Label>{t('adminOps.notificationRouting.descriptionLabel')}</Label>
                        <Input
                            value={form.description ?? ''}
                            onChange={(e) => onFormChange({ ...form, description: e.target.value })}
                            placeholder={t('adminOps.notificationRouting.editDialog.descriptionPlaceholder')}
                        />
                    </div>

                    {/* 批次事件的頻率設定 */}
                    {selectedRule && BATCH_EVENT_TYPES.has(selectedRule.event_type) && (
                        <div className="space-y-3 rounded-md border p-4 bg-muted/20">
                            <Label className="text-sm font-semibold">{t('adminOps.notificationRouting.editDialog.frequencySettings')}</Label>
                            <div className="space-y-2">
                                <Label className="text-xs text-muted-foreground">{t('adminOps.notificationRouting.editDialog.frequencyLabel')}</Label>
                                <Select
                                    value={form.frequency ?? 'daily'}
                                    onValueChange={(v) => onFormChange({
                                        ...form,
                                        frequency: v,
                                        day_of_week: v === 'weekly' ? (form.day_of_week ?? 1) : null,
                                    })}
                                >
                                    <SelectTrigger>
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {FREQUENCY_OPTIONS.map((opt) => (
                                            <SelectItem key={opt.value} value={opt.value}>{t(opt.labelKey)}</SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                            <div className="grid grid-cols-2 gap-3">
                                <div className="space-y-2">
                                    <Label className="text-xs text-muted-foreground">{t('adminOps.notificationRouting.editDialog.executionTime')}</Label>
                                    <Select
                                        value={String(form.hour_of_day ?? 8)}
                                        onValueChange={(v) => onFormChange({ ...form, hour_of_day: Number(v) })}
                                    >
                                        <SelectTrigger>
                                            <SelectValue />
                                        </SelectTrigger>
                                        <SelectContent>
                                            {Array.from({ length: 24 }, (_, i) => (
                                                <SelectItem key={i} value={String(i)}>
                                                    {String(i).padStart(2, '0')}:00
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                </div>
                                {form.frequency === 'weekly' && (
                                    <div className="space-y-2">
                                        <Label className="text-xs text-muted-foreground">{t('adminOps.notificationRouting.editDialog.dayOfWeek')}</Label>
                                        <Select
                                            value={String(form.day_of_week ?? 1)}
                                            onValueChange={(v) => onFormChange({ ...form, day_of_week: Number(v) })}
                                        >
                                            <SelectTrigger>
                                                <SelectValue />
                                            </SelectTrigger>
                                            <SelectContent>
                                                {DOW_KEYS.map((d, i) => (
                                                    <SelectItem key={i} value={String(i)}>{t(`adminOps.notificationRouting.weekdays.${d}`)}</SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    </div>
                                )}
                            </div>
                        </div>
                    )}
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        {t('common.cancel')}
                    </Button>
                    <Button onClick={onSubmit} disabled={isPending}>
                        {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        {t('common.save')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
