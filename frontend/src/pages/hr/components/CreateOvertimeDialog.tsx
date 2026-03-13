import { useState } from 'react'
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
import { toast } from '@/components/ui/use-toast'
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
    const [overtimeDate, setOvertimeDate] = useState('')
    const [startTime, setStartTime] = useState('18:00')
    const [endTime, setEndTime] = useState('21:00')
    const [overtimeType, setOvertimeType] = useState('A')
    const [reason, setReason] = useState('')

    const resetForm = () => {
        setOvertimeDate('')
        setStartTime('18:00')
        setEndTime('21:00')
        setOvertimeType('A')
        setReason('')
    }

    const handleSubmit = () => {
        if (!overtimeDate || !startTime || !endTime || !reason) {
            toast({ title: '錯誤', description: '請填寫所有必填欄位', variant: 'destructive' })
            return
        }
        onSubmit({
            overtime_date: overtimeDate,
            start_time: startTime,
            end_time: endTime,
            overtime_type: overtimeType,
            reason,
        })
        resetForm()
    }

    const handleOpenChange = (newOpen: boolean) => {
        if (!newOpen) resetForm()
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
                <div className="grid gap-4 py-4">
                    <div className="grid gap-2">
                        <Label>加班日期 *</Label>
                        <Input
                            type="date"
                            value={overtimeDate}
                            onChange={(e) => setOvertimeDate(e.target.value)}
                        />
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                        <div className="grid gap-2">
                            <Label>開始時間 *</Label>
                            <Input
                                type="time"
                                value={startTime}
                                onChange={(e) => setStartTime(e.target.value)}
                            />
                        </div>
                        <div className="grid gap-2">
                            <Label>結束時間 *</Label>
                            <Input
                                type="time"
                                value={endTime}
                                onChange={(e) => setEndTime(e.target.value)}
                            />
                        </div>
                    </div>
                    <div className="grid gap-2">
                        <Label>加班類型</Label>
                        <Select value={overtimeType} onValueChange={setOvertimeType}>
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
                            value={reason}
                            onChange={(e) => setReason(e.target.value)}
                            rows={3}
                        />
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
                    <Button variant="outline" onClick={() => handleOpenChange(false)}>
                        取消
                    </Button>
                    <Button onClick={handleSubmit} disabled={isPending}>
                        建立
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
