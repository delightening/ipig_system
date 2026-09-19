import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
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

import type { EventTypeCategory, RoleInfo, CreateRoutingData } from '../types'
import { channelOptions } from '../constants'

interface CreateRoutingDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    form: CreateRoutingData
    onFormChange: (form: CreateRoutingData) => void
    onSubmit: () => void
    isPending: boolean
    eventCategories: EventTypeCategory[] | undefined
    roles: RoleInfo[] | undefined
}

export function CreateRoutingDialog({
    open,
    onOpenChange,
    form,
    onFormChange,
    onSubmit,
    isPending,
    eventCategories,
    roles,
}: CreateRoutingDialogProps) {
    const { t } = useTranslation()
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>{t('adminOps.notificationRouting.createDialog.title')}</DialogTitle>
                    <DialogDescription>
                        {t('adminOps.notificationRouting.createDialog.description')}
                    </DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-4">
                    <div className="space-y-2">
                        <Label>{t('adminOps.notificationRouting.createDialog.eventTypeLabel')}</Label>
                        <Select
                            value={form.event_type}
                            onValueChange={(v) => onFormChange({ ...form, event_type: v })}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder={t('adminOps.notificationRouting.createDialog.eventTypePlaceholder')} />
                            </SelectTrigger>
                            <SelectContent>
                                {eventCategories?.map((cat) => (
                                    <SelectGroup key={`${cat.group}-${cat.category}`}>
                                        <SelectLabel className="font-medium">
                                            {cat.group} — {cat.category}
                                        </SelectLabel>
                                        {cat.event_types.map((et) => (
                                            <SelectItem key={et.code} value={et.code}>
                                                {et.name}
                                            </SelectItem>
                                        ))}
                                    </SelectGroup>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>

                    <div className="space-y-2">
                        <Label>{t('adminOps.notificationRouting.createDialog.roleLabel')}</Label>
                        <Select
                            value={form.role_code}
                            onValueChange={(v) => onFormChange({ ...form, role_code: v })}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder={t('adminOps.notificationRouting.createDialog.rolePlaceholder')} />
                            </SelectTrigger>
                            <SelectContent>
                                {roles?.map((r) => (
                                    <SelectItem key={r.code} value={r.code}>
                                        {r.name}（{r.code}）
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>

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

                    <div className="space-y-2">
                        <Label>{t('adminOps.notificationRouting.descriptionLabel')}</Label>
                        <Input
                            value={form.description}
                            onChange={(e) => onFormChange({ ...form, description: e.target.value })}
                            placeholder={t('adminOps.notificationRouting.createDialog.descriptionPlaceholder')}
                        />
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        {t('common.cancel')}
                    </Button>
                    <Button onClick={onSubmit} disabled={isPending}>
                        {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        {t('adminOps.notificationRouting.createDialog.submit')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
