import { useMutation } from '@tanstack/react-query'
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
import { LEAVE_TYPE_NAMES } from '@/types/hr'
import type { StaffInfo } from '@/types/hr'
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
    const isAnnualLeave = leaveForm.isAnnualLeave
    const errors = leaveForm.rhf?.formState?.errors

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
            toast({ title: '成功', description: '圖片已上傳' })
        },
        onError: () => {
            toast({ title: '錯誤', description: '圖片上傳失敗', variant: 'destructive' })
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
            <DialogContent className="sm:max-w-[500px]">
                <DialogHeader>
                    <DialogTitle>新增請假申請</DialogTitle>
                    <DialogDescription>填寫請假資訊後送出審核</DialogDescription>
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
                                基於上次申請預填
                            </Button>
                        </div>
                    )}
                    <div className="grid gap-2">
                        <Label>假別 *</Label>
                        <Select value={leaveForm.form.leaveType} onValueChange={(v) => leaveForm.updateField('leaveType', v)}>
                            <SelectTrigger>
                                <SelectValue placeholder="選擇假別" />
                            </SelectTrigger>
                            <SelectContent>
                                {Object.entries(LEAVE_TYPE_NAMES).map(([code, name]) => (
                                    <SelectItem key={code} value={code}>
                                        {name}
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
                            <Label>開始日期 *</Label>
                            <Input
                                type="date"
                                value={leaveForm.form.startDate}
                                onChange={(e) => leaveForm.handleStartDateChange(e.target.value)}
                            />
                        </div>
                        <div className="grid gap-2">
                            <Label>結束日期 *</Label>
                            <Input
                                type="date"
                                value={leaveForm.form.endDate}
                                onChange={(e) => leaveForm.handleEndDateChange(e.target.value)}
                            />
                        </div>
                    </div>
                    <div className="grid gap-2">
                        <Label>時數 <span className="text-muted-foreground text-xs">(以 0.5 小時為單位)</span></Label>
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
                        <Label>代理人 <span className="text-muted-foreground text-xs">(選填)</span></Label>
                        <Select value={leaveForm.form.proxyUserId} onValueChange={(v) => leaveForm.updateField('proxyUserId', v)}>
                            <SelectTrigger>
                                <SelectValue placeholder="選擇代理人..." />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="__none__">不選擇</SelectItem>
                                {staffList?.map((staff) => (
                                    <SelectItem key={staff.id} value={staff.id}>
                                        {staff.display_name}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="grid gap-2">
                        <Label>
                            請假事由 {!isAnnualLeave && '*'}
                            {isAnnualLeave && <span className="text-muted-foreground text-xs ml-1">(選填)</span>}
                        </Label>
                        <Textarea
                            placeholder={isAnnualLeave ? "選填，可不填寫..." : "請說明請假原因..."}
                            value={leaveForm.form.reason}
                            onChange={(e) => leaveForm.updateField('reason', e.target.value)}
                            rows={3}
                        />
                    </div>

                    {/* 圖片附件上傳 */}
                    <div className="grid gap-2">
                        <Label>附件圖片 <span className="text-muted-foreground text-xs">(選填)</span></Label>
                        <div className="flex flex-wrap gap-2">
                            {leaveForm.form.supportingImages.map((url, index) => (
                                <div key={index} className="relative group">
                                    <img
                                        src={url}
                                        alt={`附件 ${index + 1}`}
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
                            可上傳診斷證明、相關文件等
                        </p>
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        取消
                    </Button>
                    <Button onClick={onSubmit} disabled={isPending}>
                        建立
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
