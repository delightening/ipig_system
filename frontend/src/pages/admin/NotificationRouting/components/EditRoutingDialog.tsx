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
