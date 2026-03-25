import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Bell, Pencil, Trash2, FileCheck, PawPrint, Package, Users } from 'lucide-react'

import type { NotificationRouting } from '../types'
import type { GroupKey } from '../constants'
import { GROUP_KEYS } from '../constants'
import { ChannelBadge } from './ChannelBadge'

const groupIcons: Record<GroupKey, typeof FileCheck> = {
    AUP: FileCheck,
    Animal: PawPrint,
    ERP: Package,
    HR: Users,
}

interface RoutingTableProps {
    rulesByGroup: Record<GroupKey, NotificationRouting[]>
    eventNameMap: Record<string, string>
    roleNameMap: Record<string, string>
    onEdit: (rule: NotificationRouting) => void
    onDelete: (rule: NotificationRouting) => void
    onToggleActive: (id: string, isActive: boolean) => void
}

export function RoutingTable({
    rulesByGroup,
    eventNameMap,
    roleNameMap,
    onEdit,
    onDelete,
    onToggleActive,
}: RoutingTableProps) {
    return (
        <Tabs defaultValue="AUP" className="w-full">
            <TabsList className="grid w-full grid-cols-4 lg:w-auto lg:inline-flex">
                {GROUP_KEYS.map((key) => {
                    const Icon = groupIcons[key]
                    return (
                        <TabsTrigger key={key} value={key} className="flex items-center gap-2">
                            <Icon className="h-4 w-4" />
                            {key}
                            <Badge variant="secondary" className="ml-1">
                                {rulesByGroup[key].length}
                            </Badge>
                        </TabsTrigger>
                    )
                })}
            </TabsList>
            {GROUP_KEYS.map((groupKey) => (
                <TabsContent key={groupKey} value={groupKey} className="mt-4">
                    <div className="rounded-md border">
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead className="w-[200px]">事件類型</TableHead>
                                    <TableHead className="w-[140px]">通知角色</TableHead>
                                    <TableHead className="w-[160px]">通知管道</TableHead>
                                    <TableHead className="w-[80px] text-center">啟用</TableHead>
                                    <TableHead>描述</TableHead>
                                    <TableHead className="w-[100px] text-right">操作</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {rulesByGroup[groupKey].length > 0 ? (
                                    rulesByGroup[groupKey].map((rule) => (
                                        <RoutingRow
                                            key={rule.id}
                                            rule={rule}
                                            eventNameMap={eventNameMap}
                                            roleNameMap={roleNameMap}
                                            onEdit={onEdit}
                                            onDelete={onDelete}
                                            onToggleActive={onToggleActive}
                                        />
                                    ))
                                ) : (
                                    <TableRow>
                                        <TableCell colSpan={6} className="h-24 text-center text-muted-foreground">
                                            此分類尚無通知路由規則，可點擊「新增規則」建立
                                        </TableCell>
                                    </TableRow>
                                )}
                            </TableBody>
                        </Table>
                    </div>
                </TabsContent>
            ))}
        </Tabs>
    )
}

interface RoutingRowProps {
    rule: NotificationRouting
    eventNameMap: Record<string, string>
    roleNameMap: Record<string, string>
    onEdit: (rule: NotificationRouting) => void
    onDelete: (rule: NotificationRouting) => void
    onToggleActive: (id: string, isActive: boolean) => void
}

function RoutingRow({
    rule,
    eventNameMap,
    roleNameMap,
    onEdit,
    onDelete,
    onToggleActive,
}: RoutingRowProps) {
    return (
        <TableRow className={!rule.is_active ? 'opacity-50' : ''}>
            <TableCell>
                <div className="flex items-center gap-2">
                    <Bell className="h-4 w-4 text-primary shrink-0" />
                    <div>
                        <div className="font-medium">
                            {eventNameMap[rule.event_type] || rule.event_type}
                        </div>
                        <div className="text-xs text-muted-foreground font-mono">
                            {rule.event_type}
                        </div>
                    </div>
                </div>
            </TableCell>
            <TableCell>
                <Badge variant="outline">
                    {roleNameMap[rule.role_code] || rule.role_code}
                </Badge>
            </TableCell>
            <TableCell>
                <ChannelBadge channel={rule.channel} />
            </TableCell>
            <TableCell className="text-center">
                <Switch
                    checked={rule.is_active}
                    onCheckedChange={(checked) => onToggleActive(rule.id, checked)}
                />
            </TableCell>
            <TableCell className="text-sm text-muted-foreground">
                {rule.description || '—'}
            </TableCell>
            <TableCell className="text-right">
                <div className="flex justify-end gap-1">
                    <Button variant="ghost" size="icon" onClick={() => onEdit(rule)} aria-label="編輯">
                        <Pencil className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="icon" onClick={() => onDelete(rule)} aria-label="刪除">
                        <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                </div>
            </TableCell>
        </TableRow>
    )
}
