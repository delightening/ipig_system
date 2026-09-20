import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Plus } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { FormField } from '@/components/ui/form-field'
import { Textarea } from '@/components/ui/textarea'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import {
    OVERTIME_TYPE_CODES,
    overtimeTypeLabel,
    calculateOvertimeHours,
    calculateCompTime,
} from '../constants'
import type { CreateOvertimeData } from '../constants'

interface OvertimeRequestFormData {
    overtimeDate: string
    startTime: string
    endTime: string
    overtimeType: string
    reason: string
}

const DATE_PATTERN = /^\d{4}-\d{2}-\d{2}$/
const TIME_PATTERN = /^\d{2}:\d{2}$/

interface CreateOvertimeDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    onSubmit: (data: CreateOvertimeData) => void
    isPending: boolean
}

export function CreateOvertimeDialog({
    open,
    onOpenChange,
    onSubmit,
    isPending,
}: CreateOvertimeDialogProps) {
    const { t } = useTranslation()
    const {
        register,
        handleSubmit,
        watch,
        setValue,
        reset,
        formState: { errors },
    } = useForm<OvertimeRequestFormData>({
        defaultValues: {
            overtimeDate: '',
            startTime: '18:00',
            endTime: '21:00',
            overtimeType: 'A',
            reason: '',
        },
    })

    const startTime = watch('startTime')
    const endTime = watch('endTime')
    const overtimeType = watch('overtimeType')

    const onValid = (data: OvertimeRequestFormData) => {
        onSubmit({
            overtime_date: data.overtimeDate,
            start_time: data.startTime,
            end_time: data.endTime,
            overtime_type: data.overtimeType,
            reason: data.reason,
        })
        reset()
    }

    const handleOpenChange = (newOpen: boolean) => {
        if (!newOpen) reset()
        onOpenChange(newOpen)
    }

    return (
        <Dialog open={open} onOpenChange={handleOpenChange}>
            <DialogTrigger asChild>
                <Button>
                    <Plus className="h-4 w-4 mr-2" />
                    {t('hrPages.overtime.create.button')}
                </Button>
            </DialogTrigger>
            <DialogContent size="md">
                <DialogHeader>
                    <DialogTitle>{t('hrPages.overtime.create.title')}</DialogTitle>
                    <DialogDescription>{t('hrPages.overtime.create.description')}</DialogDescription>
                </DialogHeader>
                <form onSubmit={handleSubmit(onValid)}>
                    <div className="grid gap-4 py-4">
                        <FormField label={t('hrPages.overtime.create.date')} required error={errors.overtimeDate?.message}>
                            <Input type="date" {...register('overtimeDate', {
                                required: t('hrPages.overtime.create.dateRequired'),
                                pattern: { value: DATE_PATTERN, message: t('hrPages.overtime.create.dateRequired') },
                            })} aria-label={t('hrPages.overtime.create.date')} />
                        </FormField>
                        <div className="grid grid-cols-2 gap-4">
                            <FormField label={t('hrPages.overtime.create.startTime')} required>
                                <Input type="time" {...register('startTime', {
                                    required: t('hrPages.overtime.create.startTimeRequired'),
                                    pattern: { value: TIME_PATTERN, message: t('hrPages.overtime.create.startTimeRequired') },
                                })} aria-label={t('hrPages.overtime.create.startTime')} />
                            </FormField>
                            <FormField label={t('hrPages.overtime.create.endTime')} required error={errors.endTime?.message}>
                                <Input type="time" {...register('endTime', {
                                    required: t('hrPages.overtime.create.endTimeRequired'),
                                    pattern: { value: TIME_PATTERN, message: t('hrPages.overtime.create.endTimeRequired') },
                                    validate: (value, formValues) => value > formValues.startTime || t('hrPages.overtime.create.endTimeAfterStart'),
                                })} aria-label={t('hrPages.overtime.create.endTime')} />
                            </FormField>
                        </div>
                        <FormField label={t('hrPages.overtime.type')}>
                            <Select value={overtimeType} onValueChange={(v) => setValue('overtimeType', v)}>
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {OVERTIME_TYPE_CODES.map((code) => (
                                        <SelectItem key={code} value={code}>
                                            {overtimeTypeLabel(t, code)}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </FormField>
                        <FormField label={t('hrPages.overtime.create.reason')} required error={errors.reason?.message}>
                            <Textarea
                                placeholder={t('hrPages.overtime.create.reasonPlaceholder')}
                                {...register('reason', {
                                    required: t('hrPages.overtime.create.reasonRequired'),
                                    maxLength: { value: 500, message: t('hrPages.overtime.create.reasonTooLong') },
                                })}
                                rows={3}
                            />
                        </FormField>
                        <div className="grid gap-2 p-3 bg-muted rounded-lg space-y-1">
                            <div className="flex justify-between items-center">
                                <span className="text-sm text-muted-foreground">{t('hrPages.overtime.create.estimatedHours')}</span>
                                <span className="text-lg font-semibold">
                                    {t('hrPages.shared.hoursValue', { hours: calculateOvertimeHours(startTime, endTime).toFixed(1) })}
                                </span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-sm text-muted-foreground">{t('hrPages.overtime.create.estimatedCompHours')}</span>
                                <span className="text-lg font-semibold">
                                    {t('hrPages.shared.hoursValue', { hours: calculateCompTime(overtimeType).toFixed(1) })}
                                </span>
                            </div>
                        </div>
                    </div>
                    <DialogFooter>
                        <Button type="button" variant="outline" onClick={() => handleOpenChange(false)}>
                            {t('common.cancel')}
                        </Button>
                        <Button type="submit" disabled={isPending}>
                            {t('hrPages.shared.action.create')}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    )
}
