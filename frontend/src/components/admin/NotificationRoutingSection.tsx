/**
 * 通知路由管理元件（改良版）
 * - 分類可收合/展開，減少視覺壓力
 * - Switch 取代 Toggle icon
 * - 角色顯示中文名稱
 * - ConfirmDialog 取代 window.confirm
 * - 整齊的 grid layout
 */
import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import {
    Route,
    Plus,
    Trash2,
    Loader2,
    AlertCircle,
    Shield,
    Stethoscope,
    Package,
    Users,
    FileText,
    ChevronDown,
    ChevronRight,
} from 'lucide-react'
import { notificationRoutingApi } from '@/lib/api'
import type {
    NotificationRouting,
    CreateNotificationRoutingRequest,
    UpdateNotificationRoutingRequest,
    EventTypeCategory,
    NotificationFrequency,
} from '@/types/notification'
import { BATCH_EVENT_TYPES } from '@/types/notification'

/** 事件不屬於任何後端分類時的內部分組鍵；顯示時才翻譯（不可與後端分類名稱衝突）。 */
const OTHER_CATEGORY = '__other__'

/** 頻率值 → i18n 鍵（顯示文字在渲染時才 `t()`）。 */
const FREQUENCY_LABEL_KEYS: Record<NotificationFrequency, string> = {
    immediate: 'adminOps.notificationRouting.frequency.immediate',
    daily: 'adminOps.notificationRouting.frequency.daily',
    weekly: 'adminOps.notificationRouting.frequency.weekly',
    monthly: 'adminOps.notificationRouting.frequency.monthly',
}

/** 星期日(0)～星期六(6)，順序對應 day_of_week 數值；顯示文字走 i18n。 */
const WEEKDAY_KEYS = ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'] as const

// 鍵為後端 event-types 端點回傳的分類名稱（資料值，用於比對 icon，非顯示字串）
const categoryIcons: Record<string, React.ReactNode> = {
    'AUP 計畫審查': <Shield className="h-4 w-4" />,
    '修正案': <FileText className="h-4 w-4" />,
    '動物健康': <Stethoscope className="h-4 w-4" />,
    'ERP 進銷存': <Package className="h-4 w-4" />,
    'HR 人事': <Users className="h-4 w-4" />,
}

interface CategorizedItem {
    category: string
    eventType: string
    eventName: string
    rules: NotificationRouting[]
}

function categorizeRules(
    rules: NotificationRouting[],
    categories: EventTypeCategory[]
): CategorizedItem[] {
    const result: CategorizedItem[] = []
    const eventCategoryMap = new Map<string, { category: string; name: string }>()
    for (const cat of categories) {
        for (const et of cat.event_types) {
            eventCategoryMap.set(et.code, { category: cat.category, name: et.name })
        }
    }

    const grouped = new Map<string, NotificationRouting[]>()
    for (const rule of rules) {
        if (!grouped.has(rule.event_type)) grouped.set(rule.event_type, [])
        grouped.get(rule.event_type)!.push(rule)
    }

    const categoryOrder = categories.map(c => c.category)
    const sortedEntries = [...grouped.entries()].sort((a, b) => {
        const catA = eventCategoryMap.get(a[0])?.category || OTHER_CATEGORY
        const catB = eventCategoryMap.get(b[0])?.category || OTHER_CATEGORY
        const idxA = categoryOrder.indexOf(catA)
        const idxB = categoryOrder.indexOf(catB)
        return (idxA === -1 ? 99 : idxA) - (idxB === -1 ? 99 : idxB)
    })

    for (const [eventType, eventRules] of sortedEntries) {
        const info = eventCategoryMap.get(eventType)
        result.push({
            category: info?.category || OTHER_CATEGORY,
            eventType,
            eventName: info?.name || eventType,
            rules: eventRules,
        })
    }

    return result
}

export function NotificationRoutingSection() {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const { dialogState, confirm } = useConfirmDialog()
    const [showAddForm, setShowAddForm] = useState(false)
    // 預設收合：只記錄「已展開」的分類，空 Set = 全部收合
    const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set())
    const [newRule, setNewRule] = useState<CreateNotificationRoutingRequest>({
        event_type: '',
        role_code: '',
        channel: 'both',
    })

    const { data: rules, isLoading, error } = useQuery({
        queryKey: ['notification-routing'],
        queryFn: async () => {
            const res = await notificationRoutingApi.list()
            return res.data
        },
    })

    const { data: eventCategories } = useQuery({
        queryKey: ['notification-routing-event-types'],
        queryFn: async () => {
            const res = await notificationRoutingApi.getEventTypes()
            return res.data
        },
    })

    const { data: availableRoles } = useQuery({
        queryKey: ['notification-routing-roles'],
        queryFn: async () => {
            const res = await notificationRoutingApi.getRoles()
            return res.data
        },
    })

    const roleNameMap = useMemo(() => {
        const map = new Map<string, string>()
        for (const role of availableRoles || []) {
            map.set(role.code, role.name)
        }
        return map
    }, [availableRoles])

    const createMutation = useMutation({
        mutationFn: (data: CreateNotificationRoutingRequest) =>
            notificationRoutingApi.create(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
            setShowAddForm(false)
            setNewRule({ event_type: '', role_code: '', channel: 'both' })
            toast({ title: t('common.success'), description: t('adminOps.notificationRouting.section.toast.created') })
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('adminOps.shared.createFailed')),
                variant: 'destructive',
            })
        },
    })

    const updateMutation = useMutation({
        mutationFn: ({ id, data }: { id: string; data: UpdateNotificationRoutingRequest }) =>
            notificationRoutingApi.update(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('adminOps.shared.updateFailed')),
                variant: 'destructive',
            })
        },
    })

    const deleteMutation = useMutation({
        mutationFn: (id: string) => notificationRoutingApi.delete(id),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notification-routing'] })
            toast({ title: t('common.success'), description: t('adminOps.notificationRouting.section.toast.deleted') })
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('adminOps.shared.deleteFailed')),
                variant: 'destructive',
            })
        },
    })

    const categorizedRules = useMemo(
        () => categorizeRules(rules || [], eventCategories || []),
        [rules, eventCategories]
    )

    const groupedByCategory = useMemo(() => {
        const map = new Map<string, CategorizedItem[]>()
        for (const item of categorizedRules) {
            if (!map.has(item.category)) map.set(item.category, [])
            map.get(item.category)!.push(item)
        }
        return map
    }, [categorizedRules])

    const toggleCategory = (cat: string) => {
        setExpandedCategories(prev => {
            const next = new Set(prev)
            if (next.has(cat)) next.delete(cat)
            else next.add(cat)
            return next
        })
    }

    const handleToggleActive = (rule: NotificationRouting) => {
        updateMutation.mutate({
            id: rule.id,
            data: { is_active: !rule.is_active },
        })
    }

    const handleChannelChange = (rule: NotificationRouting, channel: string) => {
        updateMutation.mutate({
            id: rule.id,
            data: { channel },
        })
    }

    const handleFrequencyChange = (rule: NotificationRouting, frequency: NotificationFrequency) => {
        updateMutation.mutate({
            id: rule.id,
            data: {
                frequency,
                // weekly 時保留 day_of_week，否則清除
                day_of_week: frequency === 'weekly' ? (rule.day_of_week ?? 1) : null,
            },
        })
    }

    const handleHourChange = (rule: NotificationRouting, hour: number) => {
        updateMutation.mutate({ id: rule.id, data: { hour_of_day: hour } })
    }

    const handleDowChange = (rule: NotificationRouting, dow: number) => {
        updateMutation.mutate({ id: rule.id, data: { day_of_week: dow } })
    }

    const handleDelete = async (rule: NotificationRouting) => {
        const roleName = roleNameMap.get(rule.role_code) || rule.role_code
        const ok = await confirm({
            title: t('adminOps.notificationRouting.deleteDialog.title'),
            description: t('adminOps.notificationRouting.section.deleteConfirmDescription', { role: roleName }),
            variant: 'destructive',
            confirmLabel: t('common.delete'),
        })
        if (ok) deleteMutation.mutate(rule.id)
    }

    const handleCreate = () => {
        if (!newRule.event_type || !newRule.role_code) {
            toast({
                title: t('adminOps.notificationRouting.section.toast.noticeTitle'),
                description: t('adminOps.notificationRouting.section.toast.selectEventAndRole'),
                variant: 'destructive',
            })
            return
        }
        createMutation.mutate(newRule)
    }

    const totalRules = rules?.length || 0
    const activeRules = rules?.filter(r => r.is_active).length || 0

    return (
        <div className="border-t pt-6">
            <div className="flex items-center justify-between mb-4">
                <div>
                    <h2 className="text-2xl font-bold tracking-tight">{t('adminOps.notificationRouting.section.title')}</h2>
                    <p className="text-muted-foreground">
                        {t('adminOps.notificationRouting.section.subtitle')}
                        {totalRules > 0 && (
                            <span className="ml-2 text-xs">
                                {t('adminOps.notificationRouting.section.totalSummary', { total: totalRules, active: activeRules })}
                            </span>
                        )}
                    </p>
                </div>
                <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowAddForm(!showAddForm)}
                >
                    <Plus className="mr-1 h-4 w-4" />
                    {t('adminOps.notificationRouting.page.addRule')}
                </Button>
            </div>

            {/* 新增表單 */}
            {showAddForm && (
                <Card className="mb-6 border-status-info-border bg-status-info-bg/30">
                    <CardHeader className="pb-3">
                        <CardTitle className="text-base">{t('adminOps.notificationRouting.section.addFormTitle')}</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                            <div className="space-y-1.5">
                                <Label>{t('adminOps.notificationRouting.section.eventType')}</Label>
                                <Select
                                    value={newRule.event_type}
                                    onValueChange={(v) => setNewRule({ ...newRule, event_type: v })}
                                >
                                    <SelectTrigger>
                                        <SelectValue placeholder={t('adminOps.notificationRouting.createDialog.eventTypePlaceholder')} />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {(eventCategories || []).map((cat) => (
                                            <SelectGroup key={cat.category}>
                                                <SelectLabel className="flex items-center gap-1.5">
                                                    {categoryIcons[cat.category] || <Route className="h-3 w-3" />}
                                                    {cat.category}
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

                            <div className="space-y-1.5">
                                <Label>{t('adminOps.notificationRouting.section.role')}</Label>
                                <Select
                                    value={newRule.role_code}
                                    onValueChange={(v) => setNewRule({ ...newRule, role_code: v })}
                                >
                                    <SelectTrigger>
                                        <SelectValue placeholder={t('adminOps.notificationRouting.createDialog.rolePlaceholder')} />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {(availableRoles || []).map((role) => (
                                            <SelectItem key={role.code} value={role.code}>
                                                {role.name}（{role.code}）
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>

                            <div className="space-y-1.5">
                                <Label>{t('adminOps.notificationRouting.section.channel')}</Label>
                                <Select
                                    value={newRule.channel || 'both'}
                                    onValueChange={(v) => setNewRule({ ...newRule, channel: v })}
                                >
                                    <SelectTrigger>
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="in_app">{t('adminOps.notificationRouting.channels.inApp')}</SelectItem>
                                        <SelectItem value="email">{t('adminOps.notificationRouting.channels.email')}</SelectItem>
                                        <SelectItem value="both">{t('adminOps.notificationRouting.section.channelBoth')}</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>

                            <div className="flex items-end gap-2">
                                <Button
                                    size="sm"
                                    onClick={handleCreate}
                                    disabled={createMutation.isPending}
                                >
                                    {createMutation.isPending ? (
                                        <Loader2 className="mr-1 h-4 w-4 animate-spin" />
                                    ) : (
                                        <Plus className="mr-1 h-4 w-4" />
                                    )}
                                    {t('common.create')}
                                </Button>
                                <Button size="sm" variant="ghost" onClick={() => setShowAddForm(false)}>
                                    {t('common.cancel')}
                                </Button>
                            </div>
                        </div>

                        <div className="mt-3">
                            <Label>{t('adminOps.notificationRouting.section.descriptionOptional')}</Label>
                            <Input
                                value={newRule.description || ''}
                                onChange={(e) => setNewRule({ ...newRule, description: e.target.value })}
                                placeholder={t('adminOps.notificationRouting.section.descriptionPlaceholder')}
                                className="mt-1"
                            />
                        </div>
                    </CardContent>
                </Card>
            )}

            {isLoading && (
                <div className="flex items-center justify-center py-12">
                    <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                </div>
            )}

            {error && (
                <Card className="border-status-error-border bg-status-error-bg">
                    <CardContent className="flex items-center gap-3 py-6">
                        <AlertCircle className="h-5 w-5 text-status-error-solid" />
                        <span className="text-status-error-text">{t('adminOps.notificationRouting.section.loadError')}</span>
                    </CardContent>
                </Card>
            )}

            {!isLoading && !error && (
                <div className="space-y-4">
                    {groupedByCategory.size === 0 ? (
                        <Card>
                            <CardContent className="py-8 text-center text-muted-foreground">
                                {t('adminOps.notificationRouting.section.empty')}
                            </CardContent>
                        </Card>
                    ) : (
                        [...groupedByCategory.entries()].map(([category, items]) => {
                            const isExpanded = expandedCategories.has(category)
                            const catRuleCount = items.reduce((sum, i) => sum + i.rules.length, 0)
                            const catActiveCount = items.reduce(
                                (sum, i) => sum + i.rules.filter(r => r.is_active).length, 0
                            )

                            return (
                                <Card key={category}>
                                    {/* 分類標題列（可點擊收合） */}
                                    <button
                                        type="button"
                                        className="flex items-center justify-between w-full px-6 py-4 text-left hover:bg-muted/30 transition-colors rounded-t-lg"
                                        onClick={() => toggleCategory(category)}
                                    >
                                        <div className="flex items-center gap-2.5">
                                            {isExpanded ? (
                                                <ChevronDown className="h-4 w-4 text-muted-foreground" />
                                            ) : (
                                                <ChevronRight className="h-4 w-4 text-muted-foreground" />
                                            )}
                                            <span className="text-muted-foreground">
                                                {categoryIcons[category] || <Route className="h-4 w-4" />}
                                            </span>
                                            <span className="font-semibold">
                                                {category === OTHER_CATEGORY ? t('adminOps.notificationRouting.section.categoryOther') : category}
                                            </span>
                                        </div>
                                        <span className="text-xs text-muted-foreground tabular-nums">
                                            {t('adminOps.notificationRouting.section.categoryActiveCount', { active: catActiveCount, total: catRuleCount })}
                                        </span>
                                    </button>

                                    {isExpanded && (
                                        <CardContent className="pt-0 pb-4 space-y-4">
                                            {items.map((item) => (
                                                <div key={item.eventType}>
                                                    {/* 事件標題 */}
                                                    <div className="flex items-center gap-2 mb-2">
                                                        <span className="text-sm font-medium">{item.eventName}</span>
                                                        <span className="text-xs text-muted-foreground font-mono bg-muted px-1.5 py-0.5 rounded">
                                                            {item.eventType}
                                                        </span>
                                                    </div>

                                                    {/* 規則列表 */}
                                                    <div className="space-y-1.5 ml-0.5">
                                                        {item.rules.map((rule) => {
                                                            const isBatch = BATCH_EVENT_TYPES.has(rule.event_type)
                                                            const frequencyKey = FREQUENCY_LABEL_KEYS[rule.frequency as NotificationFrequency]
                                                            return (
                                                                <div key={rule.id} className="space-y-1">
                                                                    <div
                                                                        className={`grid grid-cols-[minmax(100px,1fr)_120px_44px_32px] items-center gap-3 rounded-lg border px-4 py-2.5 transition-colors ${
                                                                            rule.is_active ? 'bg-white' : 'bg-muted/40 opacity-60'
                                                                        }`}
                                                                    >
                                                                        {/* 角色 + 說明 + 頻率標籤 */}
                                                                        <div className="flex items-center gap-2 min-w-0">
                                                                            <span className="inline-flex items-center rounded-md bg-status-info-bg px-2 py-1 text-xs font-medium text-status-info-text ring-1 ring-inset ring-blue-700/10 shrink-0">
                                                                                {roleNameMap.get(rule.role_code) || rule.role_code}
                                                                            </span>
                                                                            <span className="text-xs text-muted-foreground bg-muted px-1.5 py-0.5 rounded shrink-0">
                                                                                {frequencyKey ? t(frequencyKey) : rule.frequency}
                                                                            </span>
                                                                            {rule.description && (
                                                                                <span className="text-xs text-muted-foreground truncate" title={rule.description}>
                                                                                    {rule.description}
                                                                                </span>
                                                                            )}
                                                                        </div>

                                                                        {/* 通道選擇 */}
                                                                        <Select
                                                                            value={rule.channel}
                                                                            onValueChange={(v) => handleChannelChange(rule, v)}
                                                                        >
                                                                            <SelectTrigger className="h-8 text-xs">
                                                                                <SelectValue />
                                                                            </SelectTrigger>
                                                                            <SelectContent>
                                                                                <SelectItem value="in_app">{t('adminOps.notificationRouting.channels.inApp')}</SelectItem>
                                                                                <SelectItem value="email">{t('adminOps.notificationRouting.channels.email')}</SelectItem>
                                                                                <SelectItem value="both">{t('adminOps.notificationRouting.section.channelBoth')}</SelectItem>
                                                                            </SelectContent>
                                                                        </Select>

                                                                        {/* 啟用 Switch */}
                                                                        <Switch
                                                                            checked={rule.is_active}
                                                                            onCheckedChange={() => handleToggleActive(rule)}
                                                                        />

                                                                        {/* 刪除 */}
                                                                        <Button
                                                                            variant="ghost"
                                                                            size="icon"
                                                                            className="h-8 w-8 text-status-error-solid hover:text-status-error-text hover:bg-status-error-bg"
                                                                            onClick={() => handleDelete(rule)}
                                                                        >
                                                                            <Trash2 className="h-4 w-4" />
                                                                        </Button>
                                                                    </div>

                                                                    {/* 批次事件的頻率設定列 */}
                                                                    {isBatch && (
                                                                        <div className="flex items-center gap-2 pl-4 pb-1">
                                                                            <span className="text-xs text-muted-foreground w-12 shrink-0">{t('adminOps.notificationRouting.section.frequencyLabel')}</span>
                                                                            <Select
                                                                                value={rule.frequency}
                                                                                onValueChange={(v) => handleFrequencyChange(rule, v as NotificationFrequency)}
                                                                            >
                                                                                <SelectTrigger className="h-7 w-24 text-xs">
                                                                                    <SelectValue />
                                                                                </SelectTrigger>
                                                                                <SelectContent>
                                                                                    <SelectItem value="daily">{t('adminOps.notificationRouting.frequency.daily')}</SelectItem>
                                                                                    <SelectItem value="weekly">{t('adminOps.notificationRouting.frequency.weekly')}</SelectItem>
                                                                                    <SelectItem value="monthly">{t('adminOps.notificationRouting.frequency.monthly')}</SelectItem>
                                                                                </SelectContent>
                                                                            </Select>
                                                                            <span className="text-xs text-muted-foreground">{t('adminOps.notificationRouting.section.timeLabel')}</span>
                                                                            <Select
                                                                                value={String(rule.hour_of_day)}
                                                                                onValueChange={(v) => handleHourChange(rule, Number(v))}
                                                                            >
                                                                                <SelectTrigger className="h-7 w-20 text-xs">
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
                                                                            {rule.frequency === 'weekly' && (
                                                                                <>
                                                                                    <span className="text-xs text-muted-foreground">{t('adminOps.notificationRouting.section.weekdayLabel')}</span>
                                                                                    <Select
                                                                                        value={String(rule.day_of_week ?? 1)}
                                                                                        onValueChange={(v) => handleDowChange(rule, Number(v))}
                                                                                    >
                                                                                        <SelectTrigger className="h-7 w-20 text-xs">
                                                                                            <SelectValue />
                                                                                        </SelectTrigger>
                                                                                        <SelectContent>
                                                                                            {WEEKDAY_KEYS.map((d, i) => (
                                                                                                <SelectItem key={i} value={String(i)}>{t(`adminOps.notificationRouting.weekdays.${d}`)}</SelectItem>
                                                                                            ))}
                                                                                        </SelectContent>
                                                                                    </Select>
                                                                                </>
                                                                            )}
                                                                        </div>
                                                                    )}
                                                                </div>
                                                            )
                                                        })}
                                                    </div>
                                                </div>
                                            ))}
                                        </CardContent>
                                    )}
                                </Card>
                            )
                        })
                    )}
                </div>
            )}

            <ConfirmDialog state={dialogState} />
        </div>
    )
}
