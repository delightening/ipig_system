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
import { channelOptions } from '../constants'
import { BATCH_EVENT_TYPES } from '@/types/notification'

const FREQUENCY_OPTIONS = [
    { value: 'daily', label: '每日' },
    { value: 'weekly', label: '每週' },
    { value: 'monthly', label: '每月' },
] as const

const DOW_OPTIONS = ['日', '一', '二', '三', '四', '五', '六'] as const

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
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-lg">
                <DialogHeader>
                    <DialogTitle>編輯通知路由規則</DialogTitle>
                    <DialogDescription>
                        {selectedRule && (
                            <>
                                {eventNameMap[selectedRule.event_type] || selectedRule.event_type}
                                {' → '}
                                {roleNameMap[selectedRule.role_code] || selectedRule.role_code}
                            </>
                        )}
                    </DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-4">
                    <div className="space-y-2">
                        <Label>通知管道</Label>
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
                                        {opt.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>

                    <div className="flex items-center justify-between">
                        <Label>啟用狀態</Label>
                        <Switch
                            checked={form.is_active ?? true}
                            onCheckedChange={(checked) => onFormChange({ ...form, is_active: checked })}
                        />
                    </div>

                    <div className="space-y-2">
                        <Label>描述</Label>
                        <Input
                            value={form.description ?? ''}
                            onChange={(e) => onFormChange({ ...form, description: e.target.value })}
                            placeholder="規則描述"
                        />
                    </div>

                    {/* 批次事件的頻率設定 */}
                    {selectedRule && BATCH_EVENT_TYPES.has(selectedRule.event_type) && (
                        <div className="space-y-3 rounded-md border p-4 bg-muted/20">
                            <Label className="text-sm font-semibold">通知頻率設定</Label>
                            <div className="space-y-2">
                                <Label className="text-xs text-muted-foreground">頻率</Label>
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
                                            <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                            <div className="grid grid-cols-2 gap-3">
                                <div className="space-y-2">
                                    <Label className="text-xs text-muted-foreground">執行時間</Label>
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
                                        <Label className="text-xs text-muted-foreground">星期幾</Label>
                                        <Select
                                            value={String(form.day_of_week ?? 1)}
                                            onValueChange={(v) => onFormChange({ ...form, day_of_week: Number(v) })}
                                        >
                                            <SelectTrigger>
                                                <SelectValue />
                                            </SelectTrigger>
                                            <SelectContent>
                                                {DOW_OPTIONS.map((d, i) => (
                                                    <SelectItem key={i} value={String(i)}>星期{d}</SelectItem>
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
                        取消
                    </Button>
                    <Button onClick={onSubmit} disabled={isPending}>
                        {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        儲存
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
