import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Clock, ImagePlus } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import api from '@/lib/api'
import { useAuthHasRole } from '@/stores/auth'
import type { StaffInfo } from '@/types/hr'
import { LEAVE_TYPE_CODES, leaveTypeLabel } from '../constants'
import type { useLeaveRequestForm } from '../hooks/useLeaveRequestForm'

interface CreateLeaveDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    leaveForm: ReturnType<typeof useLeaveRequestForm>
    staffList: StaffInfo[] | undefined
    hasHistory: boolean
    onPrefillLastLeave: () => void
    onSubmit: () => void
    isPending: boolean
}

export function CreateLeaveDialog({
    open,
    onOpenChange,
    leaveForm,
    staffList,
    hasHistory,
    onPrefillLastLeave,
    onSubmit,
    isPending,
}: CreateLeaveDialogProps) {
    const { t } = useTranslation()
    const isAnnualLeave = leaveForm.isAnnualLeave
    const errors = leaveForm.rhf?.formState?.errors
    const hasRole = useAuthHasRole()
    // 負責人之上無人可代理其職務，代理人改為選填；未指定時後端走報備制（送出即核准）。
    const isDirector = hasRole('DIRECTOR')

    const uploadMutation = useMutation({
        mutationFn: async (files: FileList) => {
            const formData = new FormData()
            for (let i = 0; i < files.length; i++) {
                formData.append('files', files[i])
            }
            const res = await api.post<{ id: string; file_path: string }[]>('/hr/leaves/attachments', formData)
            return res.data
        },
        onSuccess: (data) => {
            const newUrls = data.map(r => `/api/uploads/${r.file_path}`)
            leaveForm.addSupportingImages(newUrls)
            toast({ title: t('common.success'), description: t('hrPages.leaves.create.imageUploaded') })
        },
        onError: () => {
            toast({ title: t('common.error'), description: t('hrPages.leaves.create.imageUploadFailed'), variant: 'destructive' })
        },
    })

    const handleImageUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
        const files = e.target.files
        if (!files || files.length === 0) return
        uploadMutation.mutate(files, {
            onSettled: () => {
                e.target.value = ''
            },
        })
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent size="md">
                <DialogHeader>
                    <DialogTitle>{t('hrPages.leaves.create.title')}</DialogTitle>
                    <DialogDescription>{t('hrPages.leaves.create.description')}</DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                    {hasHistory && (
                        <div className="flex justify-end">
                            <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                onClick={onPrefillLastLeave}
                                className="text-xs"
                            >
                                {t('hrPages.leaves.create.prefillLast')}
                            </Button>
                        </div>
                    )}
                    <div className="grid gap-2">
                        <Label>{t('hrPages.shared.col.leaveType')} *</Label>
                        <Select value={leaveForm.form.leaveType} onValueChange={(v) => leaveForm.updateField('leaveType', v)}>
                            <SelectTrigger>
                                <SelectValue placeholder={t('hrPages.leaves.create.leaveTypePlaceholder')} />
                            </SelectTrigger>
                            <SelectContent>
                                {LEAVE_TYPE_CODES.map((code) => (
                                    <SelectItem key={code} value={code}>
                                        {leaveTypeLabel(t, code)}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        {errors?.leaveType && (
                            <p className="text-sm text-destructive">{errors.leaveType.message}</p>
                        )}
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                        <div className="grid gap-2">
                            <Label>{t('hrPages.shared.filter.startDate')} *</Label>
                            <Input
                                type="date"
                                value={leaveForm.form.startDate}
                                onChange={(e) => leaveForm.handleStartDateChange(e.target.value)}
                            />
                        </div>
                        <div className="grid gap-2">
                            <Label>{t('hrPages.shared.filter.endDate')} *</Label>
                            <Input
                                type="date"
                                value={leaveForm.form.endDate}
                                onChange={(e) => leaveForm.handleEndDateChange(e.target.value)}
                            />
                        </div>
                    </div>
                    <div className="grid gap-2">
                        <Label>{t('hrPages.shared.col.hours')} <span className="text-muted-foreground text-xs">{t('hrPages.leaves.create.hoursHint')}</span></Label>
                        <Input
                            type="number"
                            step="0.5"
                            min="0.5"
                            value={leaveForm.form.totalHours}
                            onChange={(e) => leaveForm.handleTotalHoursChange(e.target.value)}
                        />
                        {errors?.totalHours && (
                            <p className="text-sm text-destructive">{errors.totalHours.message}</p>
                        )}
                    </div>
                    <div className="grid gap-2">
                        <Label>
                            {t('hrPages.leaves.create.delegate')} {!isDirector && '*'}
                            {isDirector && <span className="text-muted-foreground text-xs ml-1">{t('hrPages.shared.optionalParen')}</span>}
                        </Label>
                        <Select value={leaveForm.form.proxyUserId} onValueChange={(v) => leaveForm.updateField('proxyUserId', v)}>
                            <SelectTrigger>
                                <SelectValue placeholder={isDirector ? t('hrPages.leaves.create.delegatePlaceholderDirector') : t('hrPages.leaves.create.delegatePlaceholder')} />
                            </SelectTrigger>
                            <SelectContent>
                                {isDirector && (
                                    <SelectItem value="__none__">{t('hrPages.leaves.create.noDelegate')}</SelectItem>
                                )}
                                {staffList?.map((staff) => (
                                    <SelectItem key={staff.id} value={staff.id}>
                                        {staff.display_name}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        <p className="text-xs text-muted-foreground">
                            {isDirector
                                ? t('hrPages.leaves.create.delegateHintDirector')
                                : t('hrPages.leaves.create.delegateHint')}
                        </p>
                    </div>
                    <div className="grid gap-2">
                        <Label>
                            {t('hrPages.leaves.create.reason')} {!isAnnualLeave && '*'}
                            {isAnnualLeave && <span className="text-muted-foreground text-xs ml-1">{t('hrPages.shared.optionalParen')}</span>}
                        </Label>
                        <Textarea
                            placeholder={isAnnualLeave ? t('hrPages.leaves.create.reasonPlaceholderOptional') : t('hrPages.leaves.create.reasonPlaceholder')}
                            value={leaveForm.form.reason}
                            onChange={(e) => leaveForm.updateField('reason', e.target.value)}
                            rows={3}
                        />
                    </div>

                    {/* 圖片附件上傳 */}
                    <div className="grid gap-2">
                        <Label>{t('hrPages.leaves.create.attachments')} <span className="text-muted-foreground text-xs">{t('hrPages.shared.optionalParen')}</span></Label>
                        <div className="flex flex-wrap gap-2">
                            {leaveForm.form.supportingImages.map((url, index) => (
                                <div key={index} className="relative group">
                                    <img
                                        src={url}
                                        alt={t('hrPages.leaves.create.attachmentAlt', { index: index + 1 })}
                                        className="h-16 w-16 object-cover rounded border"
                                    />
                                    <button
                                        type="button"
                                        onClick={() => leaveForm.removeSupportingImage(index)}
                                        className="absolute -top-2 -right-2 bg-destructive text-destructive-foreground rounded-full w-5 h-5 flex items-center justify-center text-xs opacity-0 group-hover:opacity-100 transition-opacity"
                                    >
                                        &times;
                                    </button>
                                </div>
                            ))}
                            <label className="h-16 w-16 border-2 border-dashed rounded flex items-center justify-center cursor-pointer hover:border-primary hover:bg-muted/50 transition-colors">
                                <input
                                    type="file"
                                    accept="image/*"
                                    multiple
                                    className="hidden"
                                    onChange={handleImageUpload}
                                    disabled={uploadMutation.isPending}
                                />
                                {uploadMutation.isPending ? (
                                    <Clock className="h-5 w-5 animate-spin text-muted-foreground" />
                                ) : (
                                    <ImagePlus className="h-5 w-5 text-muted-foreground" />
                                )}
                            </label>
                        </div>
                        <p className="text-xs text-muted-foreground">
                            {t('hrPages.leaves.create.attachmentsHint')}
                        </p>
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        {t('common.cancel')}
                    </Button>
                    <Button onClick={onSubmit} disabled={isPending}>
                        {t('hrPages.shared.action.create')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
