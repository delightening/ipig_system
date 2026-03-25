import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { Plus } from 'lucide-react'

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
    overtimeRequestSchema,
    type OvertimeRequestFormData,
} from '@/lib/validation'
import {
    OVERTIME_TYPE_NAMES,
    calculateOvertimeHours,
    calculateCompTime,
} from '../constants'
import type { CreateOvertimeData } from '../constants'

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
    const {
        register,
        handleSubmit,
        watch,
        setValue,
        reset,
        formState: { errors },
    } = useForm<OvertimeRequestFormData>({
        resolver: zodResolver(overtimeRequestSchema),
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
                    新增加班
                </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[500px]">
                <DialogHeader>
                    <DialogTitle>新增加班申請</DialogTitle>
                    <DialogDescription>填寫加班資訊後送出審核</DialogDescription>
                </DialogHeader>
                <form onSubmit={handleSubmit(onValid)}>
                    <div className="grid gap-4 py-4">
                        <div className="grid gap-2">
                            <Label>加班日期 *</Label>
                            <Input type="date" {...register('overtimeDate')} aria-label="加班日期" />
                            {errors.overtimeDate && (
                                <p className="text-sm text-destructive">{errors.overtimeDate.message}</p>
                            )}
                        </div>
                        <div className="grid grid-cols-2 gap-4">
                            <div className="grid gap-2">
                                <Label>開始時間 *</Label>
                                <Input type="time" {...register('startTime')} aria-label="開始時間" />
                            </div>
                            <div className="grid gap-2">
                                <Label>結束時間 *</Label>
                                <Input type="time" {...register('endTime')} aria-label="結束時間" />
                                {errors.endTime && (
                                    <p className="text-sm text-destructive">{errors.endTime.message}</p>
                                )}
                            </div>
                        </div>
                        <div className="grid gap-2">
                            <Label>加班類型</Label>
                            <Select value={overtimeType} onValueChange={(v) => setValue('overtimeType', v)}>
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {Object.entries(OVERTIME_TYPE_NAMES).map(([code, name]) => (
                                        <SelectItem key={code} value={code}>
                                            {name}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>
                        <div className="grid gap-2">
                            <Label>加班事由 *</Label>
                            <Textarea
                                placeholder="請說明加班原因..."
                                {...register('reason')}
                                rows={3}
                            />
                            {errors.reason && (
                                <p className="text-sm text-destructive">{errors.reason.message}</p>
                            )}
                        </div>
                        <div className="grid gap-2 p-3 bg-muted rounded-lg space-y-1">
                            <div className="flex justify-between items-center">
                                <span className="text-sm text-muted-foreground">預估加班時數</span>
                                <span className="text-lg font-semibold">
                                    {calculateOvertimeHours(startTime, endTime).toFixed(1)} 小時
                                </span>
                            </div>
                            <div className="flex justify-between items-center">
                                <span className="text-sm text-muted-foreground">預估補休時數</span>
                                <span className="text-lg font-semibold">
                                    {calculateCompTime(overtimeType).toFixed(1)} 小時
                                </span>
                            </div>
                        </div>
                    </div>
                    <DialogFooter>
                        <Button type="button" variant="outline" onClick={() => handleOpenChange(false)}>
                            取消
                        </Button>
                        <Button type="submit" disabled={isPending}>
                            建立
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    )
}
