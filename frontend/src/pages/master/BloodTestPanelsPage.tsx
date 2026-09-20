import { useState, useMemo } from 'react'
import { STALE_TIME } from '@/lib/query'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import {
    bloodTestPanelApi,
    bloodTestTemplateApi,
    BloodTestPanel,
    UpdateBloodTestPanelRequest,
    UpdateBloodTestPanelItemsRequest,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { toast } from '@/components/ui/use-toast'
import {
    Plus,
    Search,
    Edit,
    Power,
    PowerOff,
    Loader2,
    ArrowLeft,
    Settings,
    Droplets,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { getApiErrorMessage } from '@/lib/apiError'

// R57-2: 改 React Hook Form 原生 validation rules（避開 Zod 4 CSP eval probe）
type BloodTestPanelFormData = {
  key: string
  name: string
  icon: string
  sort_order: number
}
import { PanelIcon } from '@/components/ui/panel-icon'
import { useNavigate } from 'react-router-dom'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { TableSkeleton } from '@/components/ui/table-skeleton'

// 顯示篩選
type ShowFilter = 'all' | 'active' | 'inactive'

export function BloodTestPanelsPage() {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const navigate = useNavigate()

    // 狀態
    const [search, setSearch] = useState('')
    const [showFilter, setShowFilter] = useState<ShowFilter>('all')
    const [dialogOpen, setDialogOpen] = useState(false)
    const [editingPanel, setEditingPanel] = useState<BloodTestPanel | null>(null)
    const { register, handleSubmit: rhfHandleSubmit, reset, watch, formState: { errors } } = useForm<BloodTestPanelFormData>({
        defaultValues: { key: '', name: '', icon: '', sort_order: 0 },
    })
    const iconValue = watch('icon')

    // 管理項目對話框
    const [itemsDialogOpen, setItemsDialogOpen] = useState(false)
    const [managingPanel, setManagingPanel] = useState<BloodTestPanel | null>(null)
    const [selectedTemplateIds, setSelectedTemplateIds] = useState<Set<string>>(new Set())
    const [itemSearch, setItemSearch] = useState('')

    // 查詢所有 Panel（含停用）
    const { data: panels, isLoading } = useQuery({
        queryKey: ['blood-test-panels-all'],
        staleTime: STALE_TIME.REFERENCE,
        queryFn: async () => {
            const response = await bloodTestPanelApi.listAll()
            return response.data
        },
    })

    // 查詢所有模板（供管理項目用）
    const { data: allTemplates } = useQuery({
        queryKey: ['blood-test-templates-all'],
        staleTime: STALE_TIME.REFERENCE,
        queryFn: async () => {
            const response = await bloodTestTemplateApi.listAll()
            return response.data
        },
    })

    // 新增 Panel
    const createMutation = useMutation({
        mutationFn: (data: BloodTestPanelFormData) =>
            bloodTestPanelApi.create(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
            toast({ title: t('common.success'), description: t('erpMaster.bloodTest.toast.panelCreated') })
            setDialogOpen(false)
            resetForm()
        },
        onError: (error: unknown) => {
            const msg = getApiErrorMessage(error, t('erpMaster.bloodTest.toast.createFailed'))
            toast({ title: t('common.error'), description: msg, variant: 'destructive' })
        },
    })

    // 更新 Panel
    const updateMutation = useMutation({
        mutationFn: ({ id, data }: { id: string; data: UpdateBloodTestPanelRequest }) =>
            bloodTestPanelApi.update(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-templates-all'] })
            toast({ title: t('common.success'), description: t('erpMaster.bloodTest.toast.panelUpdated') })
            setDialogOpen(false)
            resetForm()
        },
        onError: (error: unknown) => {
            const msg = getApiErrorMessage(error, t('erpMaster.bloodTest.toast.updateFailed'))
            toast({ title: t('common.error'), description: msg, variant: 'destructive' })
        },
    })

    // 停用/啟用 Panel
    const toggleMutation = useMutation({
        mutationFn: ({ id, is_active }: { id: string; is_active: boolean }) =>
            bloodTestPanelApi.update(id, { is_active }),
        onSuccess: (_data, variables) => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
            toast({
                title: t('common.success'),
                description: variables.is_active ? t('erpMaster.bloodTest.toast.panelRestored') : t('erpMaster.bloodTest.toast.panelDeactivated'),
            })
        },
        onError: (error: unknown) => {
            const msg = getApiErrorMessage(error, t('erpMaster.bloodTest.toast.operationFailed'))
            toast({ title: t('common.error'), description: msg, variant: 'destructive' })
        },
    })

    // 更新 Panel 項目
    const updateItemsMutation = useMutation({
        mutationFn: ({ id, data }: { id: string; data: UpdateBloodTestPanelItemsRequest }) =>
            bloodTestPanelApi.updateItems(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-templates-all'] })
            toast({ title: t('common.success'), description: t('erpMaster.bloodTest.toast.panelItemsUpdated') })
            setItemsDialogOpen(false)
            setManagingPanel(null)
        },
        onError: (error: unknown) => {
            const msg = getApiErrorMessage(error, t('erpMaster.bloodTest.toast.updateItemsFailed'))
            toast({ title: t('common.error'), description: msg, variant: 'destructive' })
        },
    })

    // 重置表單
    const resetForm = () => {
        setEditingPanel(null)
        reset({ key: '', name: '', icon: '', sort_order: 0 })
    }

    // 開啟編輯
    const handleEdit = (panel: BloodTestPanel) => {
        setEditingPanel(panel)
        reset({
            key: panel.key,
            name: panel.name,
            icon: panel.icon || '',
            sort_order: panel.sort_order,
        })
        setDialogOpen(true)
    }

    // 開啟管理項目
    const handleManageItems = (panel: BloodTestPanel) => {
        setManagingPanel(panel)
        setSelectedTemplateIds(new Set(panel.items.map((tpl) => tpl.id)))
        setItemSearch('')
        setItemsDialogOpen(true)
    }

    // 提交表單
    const onPanelSubmit = (data: BloodTestPanelFormData) => {
        if (editingPanel) {
            updateMutation.mutate({
                id: editingPanel.id,
                data: {
                    name: data.name,
                    icon: data.icon || undefined,
                    sort_order: data.sort_order,
                },
            })
        } else {
            createMutation.mutate(data)
        }
    }

    // 儲存項目
    const handleSaveItems = () => {
        if (!managingPanel) return
        updateItemsMutation.mutate({
            id: managingPanel.id,
            data: { template_ids: Array.from(selectedTemplateIds) },
        })
    }

    // 切換項目選取
    const toggleTemplate = (id: string) => {
        setSelectedTemplateIds((prev) => {
            const next = new Set(prev)
            if (next.has(id)) {
                next.delete(id)
            } else {
                next.add(id)
            }
            return next
        })
    }

    // 統計
    const totalCount = panels?.length ?? 0
    const activeCount = panels?.filter((p) => p.is_active).length ?? 0

    // 篩選
    const filteredPanels = useMemo(() => {
        if (!panels) return []
        let result = [...panels]

        // 搜尋
        if (search) {
            const q = search.toLowerCase()
            result = result.filter(
                (p) =>
                    p.key.toLowerCase().includes(q) ||
                    p.name.toLowerCase().includes(q)
            )
        }

        // 啟用/停用篩選
        if (showFilter === 'active') {
            result = result.filter((p) => p.is_active)
        } else if (showFilter === 'inactive') {
            result = result.filter((p) => !p.is_active)
        }

        // 排序
        result.sort((a, b) => a.sort_order - b.sort_order)

        return result
    }, [panels, search, showFilter])

    // 篩選可選模板
    const filteredTemplates = useMemo(() => {
        if (!allTemplates) return []
        let result = allTemplates.filter((tpl) => tpl.is_active)
        if (itemSearch) {
            const q = itemSearch.toLowerCase()
            result = result.filter(
                (tpl) =>
                    tpl.code.toLowerCase().includes(q) ||
                    tpl.name.toLowerCase().includes(q)
            )
        }
        result.sort((a, b) => a.sort_order - b.sort_order)
        return result
    }, [allTemplates, itemSearch])

    const { sortedData, sort, toggleSort } = useTableSort(filteredPanels)

    const isSaving = createMutation.isPending || updateMutation.isPending

    return (
        <div className="space-y-6">
            <div className="flex items-center gap-4">
                <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => navigate('/blood-test-templates')}
                    aria-label={t('erpMaster.common.back')}
                >
                    <ArrowLeft className="h-5 w-5" />
                </Button>
                <PageHeader
                    title={t('erpMaster.bloodTest.panels.title')}
                    description={t('erpMaster.bloodTest.panels.description', { total: totalCount, active: activeCount })}
                    className="flex-1"
                    actions={
                        <Button size="sm" onClick={() => { resetForm(); setDialogOpen(true) }}>
                            <Plus className="mr-2 h-4 w-4" />
                            {t('erpMaster.bloodTest.panels.addCategory')}
                        </Button>
                    }
                />
            </div>

            {/* 搜尋與篩選列 */}
            <div className="flex gap-4 items-center">
                <div className="relative flex-1 max-w-sm">
                    <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                    <Input
                        placeholder={t('erpMaster.bloodTest.searchCodeOrName')}
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        className="pl-9"
                    />
                </div>
                <div className="flex gap-1">
                    {(['all', 'active', 'inactive'] as ShowFilter[]).map((f) => (
                        <Button
                            key={f}
                            variant={showFilter === f ? 'default' : 'outline'}
                            size="sm"
                            onClick={() => setShowFilter(f)}
                        >
                            {f === 'all' ? t('erpMaster.common.all') : f === 'active' ? t('erpMaster.common.activeFilter') : t('erpMaster.common.inactiveFilter')}
                        </Button>
                    ))}
                </div>
            </div>

            {/* 表格 */}
            <div className="rounded-lg border bg-card overflow-hidden">
                <Table>
                    <TableHeader>
                        <TableRow className="bg-muted/50 hover:bg-muted/50">
                            <TableHead className="w-[60px]">{t('erpMaster.common.icon')}</TableHead>
                            <SortableTableHead sortKey="key" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="w-[120px]">{t('erpMaster.common.code')}</SortableTableHead>
                            <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('erpMaster.common.name')}</SortableTableHead>
                            <SortableTableHead sortKey="sort_order" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="w-[80px] text-center">{t('erpMaster.common.sortOrder')}</SortableTableHead>
                            <TableHead className="w-[100px] text-center">{t('erpMaster.bloodTest.panels.includedItems')}</TableHead>
                            <SortableTableHead sortKey="is_active" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="w-[80px] text-center">{t('erpMaster.common.status')}</SortableTableHead>
                            <TableHead className="w-[180px] text-right">{t('common.actions')}</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={7} className="p-0">
                                    <TableSkeleton rows={5} cols={7} />
                                </TableCell>
                            </TableRow>
                        ) : filteredPanels.length === 0 ? (
                            <TableEmptyRow colSpan={7} icon={Droplets} title={t('erpMaster.bloodTest.panels.noMatch')} />
                        ) : (
                            (sortedData ?? filteredPanels).map((panel) => (
                                <TableRow
                                    key={panel.id}
                                    className={cn(!panel.is_active && 'opacity-50')}
                                >
                                    <TableCell>
                                        <PanelIcon icon={panel.icon} size={24} />
                                    </TableCell>
                                    <TableCell className="font-mono text-sm">{panel.key}</TableCell>
                                    <TableCell className="font-medium">{panel.name}</TableCell>
                                    <TableCell className="text-center">{panel.sort_order}</TableCell>
                                    <TableCell className="text-center">
                                        <Badge variant="secondary">{panel.items.length}</Badge>
                                    </TableCell>
                                    <TableCell className="text-center">
                                        <Badge variant={panel.is_active ? 'default' : 'outline'}>
                                            {panel.is_active ? t('erpMaster.common.active') : t('erpMaster.common.inactive')}
                                        </Badge>
                                    </TableCell>
                                    <TableCell className="text-right">
                                        <div className="flex justify-end gap-1">
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() => handleManageItems(panel)}
                                                title={t('erpMaster.bloodTest.panels.manageItems')}
                                            >
                                                <Settings className="h-4 w-4" />
                                            </Button>
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() => handleEdit(panel)}
                                                title={t('common.edit')}
                                            >
                                                <Edit className="h-4 w-4" />
                                            </Button>
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() =>
                                                    toggleMutation.mutate({
                                                        id: panel.id,
                                                        is_active: !panel.is_active,
                                                    })
                                                }
                                                title={panel.is_active ? t('erpMaster.products.actions.deactivate') : t('erpMaster.products.actions.activate')}
                                            >
                                                {panel.is_active ? (
                                                    <PowerOff className="h-4 w-4 text-destructive" />
                                                ) : (
                                                    <Power className="h-4 w-4 text-status-success-solid" />
                                                )}
                                            </Button>
                                        </div>
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </div>

            {/* 新增/編輯對話框 */}
            <Dialog open={dialogOpen} onOpenChange={(open) => { if (!open) { resetForm(); setDialogOpen(false) } }}>
                <DialogContent size="sm">
                    <DialogHeader>
                        <DialogTitle>{editingPanel ? t('erpMaster.bloodTest.panels.editCategory') : t('erpMaster.bloodTest.panels.addCategory')}</DialogTitle>
                        <DialogDescription>
                            {editingPanel ? t('erpMaster.bloodTest.panels.editDescription') : t('erpMaster.bloodTest.panels.addDescription')}
                        </DialogDescription>
                    </DialogHeader>
                    <form onSubmit={rhfHandleSubmit(onPanelSubmit)}>
                        <div className="grid gap-4 py-4">
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel-key" className="text-right">{t('erpMaster.common.code')}</Label>
                                <div className="col-span-3 space-y-1">
                                    <Input
                                        id="panel-key"
                                        {...register('key', { required: 'validation.required' })}
                                        placeholder={t('erpMaster.bloodTest.panels.codePlaceholder')}
                                        disabled={!!editingPanel}
                                    />
                                    {errors.key && (
                                        <p className="text-sm text-destructive">{t(errors.key.message ?? 'validation.required')}</p>
                                    )}
                                </div>
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel-name" className="text-right">{t('erpMaster.common.name')}</Label>
                                <div className="col-span-3 space-y-1">
                                    <Input
                                        id="panel-name"
                                        {...register('name', { required: 'validation.required' })}
                                        placeholder={t('erpMaster.bloodTest.panels.namePlaceholder')}
                                    />
                                    {errors.name && (
                                        <p className="text-sm text-destructive">{t(errors.name.message ?? 'validation.required')}</p>
                                    )}
                                </div>
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel-icon" className="text-right">{t('erpMaster.common.icon')}</Label>
                                <div className="col-span-3 flex items-center gap-2">
                                    <Input
                                        id="panel-icon"
                                        {...register('icon')}
                                        placeholder={t('erpMaster.bloodTest.panels.iconPlaceholder')}
                                        className="flex-1"
                                    />
                                    {iconValue && (
                                        <span className="text-2xl"><PanelIcon icon={iconValue} size={28} /></span>
                                    )}
                                </div>
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="panel-sort" className="text-right">{t('erpMaster.common.sortOrder')}</Label>
                                <Input
                                    id="panel-sort"
                                    type="number"
                                    {...register('sort_order', { valueAsNumber: true })}
                                    className="col-span-3"
                                />
                            </div>
                        </div>
                        <DialogFooter>
                            <Button type="button" variant="outline" onClick={() => { resetForm(); setDialogOpen(false) }}>
                                {t('common.cancel')}
                            </Button>
                            <Button type="submit" disabled={isSaving}>
                                {isSaving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                                {editingPanel ? t('common.save') : t('erpMaster.common.createSubmit')}
                            </Button>
                        </DialogFooter>
                    </form>
                </DialogContent>
            </Dialog>

            {/* 管理項目對話框 */}
            <Dialog open={itemsDialogOpen} onOpenChange={(open) => { if (!open) { setItemsDialogOpen(false); setManagingPanel(null) } }}>
                <DialogContent size="lg" className="max-h-[80vh]">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            {managingPanel && <PanelIcon icon={managingPanel.icon} size={22} />}
                            {t('erpMaster.bloodTest.panels.manageItemsTitle', { name: managingPanel?.name })}
                        </DialogTitle>
                        <DialogDescription>
                            {t('erpMaster.bloodTest.panels.manageItemsDescription', { count: selectedTemplateIds.size })}
                        </DialogDescription>
                    </DialogHeader>

                    {/* 項目搜尋 */}
                    <div className="relative">
                        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                        <Input
                            placeholder={t('erpMaster.bloodTest.searchCodeOrName')}
                            value={itemSearch}
                            onChange={(e) => setItemSearch(e.target.value)}
                            className="pl-9"
                        />
                    </div>

                    {/* 項目列表（可捲動） */}
                    <div className="overflow-y-auto max-h-[400px] border rounded-md">
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead className="w-[40px]"></TableHead>
                                    <TableHead className="w-[100px]">{t('erpMaster.common.code')}</TableHead>
                                    <TableHead>{t('erpMaster.common.name')}</TableHead>
                                    <TableHead className="w-[80px]">{t('erpMaster.common.unit')}</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {filteredTemplates.map((tpl) => (
                                    <TableRow
                                        key={tpl.id}
                                        className="cursor-pointer"
                                        onClick={() => toggleTemplate(tpl.id)}
                                    >
                                        <TableCell>
                                            <Checkbox
                                                checked={selectedTemplateIds.has(tpl.id)}
                                                onCheckedChange={() => toggleTemplate(tpl.id)}
                                            />
                                        </TableCell>
                                        <TableCell className="font-mono text-sm">{tpl.code}</TableCell>
                                        <TableCell>{tpl.name}</TableCell>
                                        <TableCell className="text-muted-foreground">{tpl.default_unit || '-'}</TableCell>
                                    </TableRow>
                                ))}
                                {filteredTemplates.length === 0 && (
                                    <TableEmptyRow colSpan={4} icon={Search} title={t('erpMaster.bloodTest.panels.noMatchItems')} />
                                )}
                            </TableBody>
                        </Table>
                    </div>

                    <DialogFooter>
                        <Button
                            type="button"
                            variant="outline"
                            onClick={() => { setItemsDialogOpen(false); setManagingPanel(null) }}
                        >
                            {t('common.cancel')}
                        </Button>
                        <Button
                            onClick={handleSaveItems}
                            disabled={updateItemsMutation.isPending}
                        >
                            {updateItemsMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                            {t('erpMaster.bloodTest.panels.saveItems', { count: selectedTemplateIds.size })}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    )
}
