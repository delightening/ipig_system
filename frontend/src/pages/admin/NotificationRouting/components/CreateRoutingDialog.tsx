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
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-lg">
                <DialogHeader>
                    <DialogTitle>新增通知路由規則</DialogTitle>
                    <DialogDescription>
                        設定當特定事件發生時，通知哪個角色以及使用何種通知方式
                    </DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-4">
                    <div className="space-y-2">
                        <Label>事件類型 *</Label>
                        <Select
                            value={form.event_type}
                            onValueChange={(v) => onFormChange({ ...form, event_type: v })}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder="選擇事件類型" />
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
                        <Label>通知角色 *</Label>
                        <Select
                            value={form.role_code}
                            onValueChange={(v) => onFormChange({ ...form, role_code: v })}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder="選擇角色" />
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

                    <div className="space-y-2">
                        <Label>描述</Label>
                        <Input
                            value={form.description}
                            onChange={(e) => onFormChange({ ...form, description: e.target.value })}
                            placeholder="例如：計畫提交後通知 IACUC 執行秘書"
                        />
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        取消
                    </Button>
                    <Button onClick={onSubmit} disabled={isPending}>
                        {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        建立
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
