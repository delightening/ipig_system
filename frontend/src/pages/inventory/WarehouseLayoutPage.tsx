import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, {
    Warehouse,
    StorageLocationWithWarehouse,
    StorageLocationType,
    StorageLayoutItem,
    StorageLocationInventoryItem,
    UnassignedInventoryItem,
} from '@/lib/api'
// R57-2: 改 React Hook Form 原生 validation rules（避開 Zod 4 CSP eval probe）
type StorageLocationFormData = {
    name: string
    location_type: StorageLocationType
    capacity: string
    color: string
}
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import { Loader2 } from 'lucide-react'
import { WarehouseActionHeader } from '@/components/warehouse/WarehouseActionHeader'
import { StorageLocationEditor } from '@/components/warehouse/StorageLocationEditor'
import { WarehouseDetailTabs } from '@/components/warehouse/WarehouseDetailTabs'
import { WarehouseImportDialog } from '@/components/warehouse/WarehouseImportDialog'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import {
    resolveLocationFromUrl,
    useWarehouseUrlSelection,
} from '@/hooks/useWarehouseUrlSelection'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

// 預設顏色
const DEFAULT_COLORS: Record<StorageLocationType, string> = {
    shelf: '#3b82f6',
    rack: '#10b981',
    zone: '#f59e0b',
    bin: '#6366f1',
    wall: '#475569',
    door: '#94a3b8',
    window: '#bae6fd',
}

const initialFormValues: StorageLocationFormData = {
    name: '',
    location_type: 'shelf',
    capacity: '',
    color: DEFAULT_COLORS.shelf,
}

export function WarehouseLayoutPage() {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const { dialogState } = useConfirmDialog()

    // 取得啟用中的倉庫（供選擇器使用）
    const { data: warehouses } = useQuery({
        queryKey: ['warehouses'],
        queryFn: async () => {
            const res = await api.get<Warehouse[]>('/warehouses')
            return res.data.filter(w => w.is_active)
        },
    })

    // 選中的倉庫/儲位存在網址 query（?warehouse=<倉庫代碼>&location=<儲位代碼>）。
    // 解析/寫回邏輯抽至 useWarehouseUrlSelection（純函式 + 單元測試釘行為）。
    const {
        selectedWarehouseId,
        urlLocationCode,
        setSelectedWarehouseId,
        setSelectedLocation,
    } = useWarehouseUrlSelection(warehouses)

    const [isEditMode, setIsEditMode] = useState(false)
    const [showDialog, setShowDialog] = useState(false)
    const [editingLocation, setEditingLocation] = useState<StorageLocationWithWarehouse | null>(null)

    const {
        register: registerLoc,
        handleSubmit: handleLocSubmit,
        watch: watchLoc,
        setValue: setLocValue,
        reset: resetLoc,
        formState: { errors: locErrors },
    } = useForm<StorageLocationFormData>({
        defaultValues: initialFormValues,
    })

    const locationType = watchLoc('location_type')
    const locColor = watchLoc('color')
    const [pendingLayoutChanges, setPendingLayoutChanges] = useState<StorageLayoutItem[]>([])
    const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)
    const [activeTab, setActiveTab] = useState<string>('location-inventory')
    const [showImportDialog, setShowImportDialog] = useState(false)

    // 取得儲位與結構列表
    const { data: locations, isLoading: loadingLocations } = useQuery({
        queryKey: ['storage-locations', selectedWarehouseId],
        queryFn: async () => {
            const res = await api.get<StorageLocationWithWarehouse[]>(
                `/storage-locations?warehouse_id=${selectedWarehouseId}`
            )
            return res.data
        },
        enabled: !!selectedWarehouseId,
    })

    // 選中儲位由網址 location 代碼派生；換倉庫時 location 已於 setter 清除。
    // 解析規則（含畸形連結取捨）見 resolveLocationFromUrl doc。
    const selectedLocation = resolveLocationFromUrl(locations, urlLocationCode)

    // 取得選取儲位的庫存
    const { data: inventoryItems, isLoading: loadingInventory } = useQuery({
        queryKey: ['storage-location-inventory', selectedLocation?.id],
        queryFn: async () => {
            const res = await api.get<StorageLocationInventoryItem[]>(
                `/storage-locations/${selectedLocation!.id}/inventory`,
            )
            return res.data
        },
        enabled: !!selectedLocation,
    })

    // 取得未分配庫存
    const { data: unassignedItems, isLoading: loadingUnassigned } = useQuery({
        queryKey: ['unassigned-inventory', selectedWarehouseId],
        queryFn: async () => {
            const res = await api.get<UnassignedInventoryItem[]>(
                `/inventory/unassigned?warehouse_id=${selectedWarehouseId}`,
            )
            return res.data
        },
        enabled: !!selectedWarehouseId,
    })

    // Mutations
    const createMutation = useMutation({
        mutationFn: async (data: StorageLocationFormData) => {
            return api.post('/storage-locations', {
                warehouse_id: selectedWarehouseId,
                name: data.name,
                location_type: data.location_type,
                capacity: data.capacity ? parseInt(data.capacity) : undefined,
                color: data.color,
                row_index: 0,
                col_index: 0,
                width: 2,
                height: 2,
            })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['storage-locations', selectedWarehouseId] })
            toast({ title: t('common.success'), description: t('erpDocs.warehouse.layout.created') })
            setShowDialog(false)
        },
    })

    const updateMutation = useMutation({
        mutationFn: async ({ id, data }: { id: string; data: StorageLocationFormData }) => {
            return api.put(`/storage-locations/${id}`, {
                name: data.name,
                location_type: data.location_type,
                capacity: data.capacity ? parseInt(data.capacity) : undefined,
                color: data.color,
            })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['storage-locations', selectedWarehouseId] })
            toast({ title: t('common.success'), description: t('erpDocs.warehouse.layout.updated') })
            setShowDialog(false)
        },
    })

    const saveLayoutMutation = useMutation({
        mutationFn: async (items: StorageLayoutItem[]) => {
            return api.put(`/warehouses/${selectedWarehouseId}/layout`, { items })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['storage-locations', selectedWarehouseId] })
            toast({ title: t('common.success'), description: t('erpDocs.warehouse.layout.layoutSaved') })
            setHasUnsavedChanges(false)
            setPendingLayoutChanges([])
        },
    })

    // Event Handlers
    const handleAddLocation = () => {
        setEditingLocation(null)
        resetLoc(initialFormValues)
        setShowDialog(true)
    }

    const handleEditLocation = (loc: StorageLocationWithWarehouse) => {
        setEditingLocation(loc)
        resetLoc({
            name: loc.name || loc.code,
            location_type: loc.location_type,
            capacity: loc.capacity?.toString() || '',
            color: loc.color || DEFAULT_COLORS[loc.location_type],
        })
        setShowDialog(true)
    }

    const handleLocationClick = (loc: StorageLocationWithWarehouse) => {
        setSelectedLocation(loc)
        setActiveTab('location-inventory')
    }

    const handleExportWarehouses = () => {
        if (!warehouses || warehouses.length === 0) return
        const headers = [
            t('erpDocs.shared.code'),
            t('erpDocs.shared.name'),
            t('erpDocs.shared.address'),
            t('erpDocs.shared.status'),
        ]
        const rows = warehouses.map((w) => [
            w.code,
            w.name,
            w.address || '',
            w.is_active ? t('erpDocs.warehouse.layout.active') : t('erpDocs.warehouse.layout.inactive'),
        ])
        const csvContent = ['\ufeff' + headers.join(','), ...rows.map(r => r.join(','))].join('\n')
        const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
        const link = document.createElement('a')
        link.href = URL.createObjectURL(blob)
        link.download = `warehouses_${new Date().toISOString().split('T')[0]}.csv`
        link.click()
    }

    return (
        <div className="container mx-auto py-8 max-w-7xl animate-in fade-in duration-500">
            {/* 上部：倉庫管理 */}
            <WarehouseActionHeader
                selectedWarehouseId={selectedWarehouseId}
                onWarehouseChange={setSelectedWarehouseId}
                onImportClick={() => setShowImportDialog(true)}
                onExportClick={handleExportWarehouses}
            />

            <div className="grid grid-cols-1 gap-8 mt-8">
                {/* 中部：儲位 2D 佈局 */}
                {warehouses === undefined ? (
                    // 冷載入/分享連結：warehouses 未到前無法由代碼解析倉庫，顯示 spinner 而非
                    // 誤導的「請先選擇倉庫」空狀態（避免閃爍；清單就緒後自動切到內容）。
                    <div className="h-64 flex items-center justify-center rounded-xl border bg-card shadow-xs">
                        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                    </div>
                ) : selectedWarehouseId ? (
                    <StorageLocationEditor
                        locations={locations || []}
                        isLoading={loadingLocations}
                        isEditMode={isEditMode}
                        setIsEditMode={setIsEditMode}
                        onLayoutChange={(newLayout) => {
                            setPendingLayoutChanges(newLayout)
                            setHasUnsavedChanges(true)
                        }}
                        onSaveLayout={() => saveLayoutMutation.mutate(pendingLayoutChanges)}
                        isSavingLayout={saveLayoutMutation.isPending}
                        hasUnsavedChanges={hasUnsavedChanges}
                        onAddLocationClick={handleAddLocation}
                        selectedLocationId={selectedLocation?.id || null}
                        onLocationClick={handleLocationClick}
                    />
                ) : (
                    <div className="h-64 flex items-center justify-center rounded-xl border bg-card shadow-xs text-muted-foreground">
                        {t('erpDocs.warehouse.layout.selectFirst')}
                    </div>
                )}

                {/* 下部：詳情與清單 */}
                {selectedWarehouseId && (
                    <WarehouseDetailTabs
                        warehouse={warehouses?.find(w => w.id === selectedWarehouseId)}
                        selectedLocation={selectedLocation}
                        onLocationSelect={setSelectedLocation}
                        locations={locations || []}
                        loadingLocations={loadingLocations}
                        inventoryItems={inventoryItems}
                        loadingInventory={loadingInventory}
                        unassignedItems={unassignedItems}
                        loadingUnassigned={loadingUnassigned}
                        activeTab={activeTab}
                        onTabChange={setActiveTab}
                        onEditLocationClick={handleEditLocation}
                    />
                )}
            </div>

            {/* 儲位編輯/建立 Dialog */}
            <Dialog open={showDialog} onOpenChange={setShowDialog}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>{editingLocation ? t('erpDocs.warehouse.layout.editItem') : t('erpDocs.warehouse.layout.newItem')}</DialogTitle>
                        <DialogDescription>{t('erpDocs.warehouse.layout.dialogDescription')}</DialogDescription>
                    </DialogHeader>
                    <form onSubmit={handleLocSubmit((data) => {
                        if (editingLocation) {
                            updateMutation.mutate({ id: editingLocation.id, data })
                        } else {
                            createMutation.mutate(data)
                        }
                    })}>
                        <div className="grid gap-4 py-4">
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="loc-name" className="text-right">{t('erpDocs.shared.nameRequired')}</Label>
                                <div className="col-span-3">
                                    <Input
                                        id="loc-name"
                                        {...registerLoc('name', { required: t('erpDocs.warehouse.layout.nameRequired') })}
                                    />
                                    {locErrors.name && (
                                        <p className="text-sm text-destructive mt-1">{locErrors.name.message}</p>
                                    )}
                                </div>
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="loc-type" className="text-right">{t('erpDocs.shared.type')}</Label>
                                <Select
                                    value={locationType}
                                    onValueChange={(v: StorageLocationType) => {
                                        setLocValue('location_type', v)
                                        setLocValue('color', DEFAULT_COLORS[v])
                                    }}
                                >
                                    <SelectTrigger className="col-span-3">
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="shelf">{t('erpDocs.warehouse.locationType.shelf')}</SelectItem>
                                        <SelectItem value="rack">{t('erpDocs.warehouse.locationType.rack')}</SelectItem>
                                        <SelectItem value="zone">{t('erpDocs.warehouse.locationType.zone')}</SelectItem>
                                        <SelectItem value="bin">{t('erpDocs.warehouse.locationType.bin')}</SelectItem>
                                        <SelectItem value="wall">{t('erpDocs.warehouse.locationType.wall')}</SelectItem>
                                        <SelectItem value="door">{t('erpDocs.warehouse.locationType.door')}</SelectItem>
                                        <SelectItem value="window">{t('erpDocs.warehouse.locationType.window')}</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                            {['shelf', 'rack', 'zone', 'bin'].includes(locationType) && (
                                <div className="grid grid-cols-4 items-center gap-4">
                                    <Label htmlFor="loc-capacity" className="text-right">{t('erpDocs.shared.capacity')}</Label>
                                    <Input
                                        id="loc-capacity"
                                        type="number"
                                        {...registerLoc('capacity')}
                                        className="col-span-3"
                                    />
                                </div>
                            )}
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="loc-color" className="text-right">{t('erpDocs.warehouse.layout.color')}</Label>
                                <div className="col-span-3 flex gap-2">
                                    <input
                                        type="color"
                                        value={locColor}
                                        onChange={(e) => setLocValue('color', e.target.value)}
                                        className="h-10 w-14 rounded border cursor-pointer"
                                    />
                                    <Input
                                        {...registerLoc('color', { required: t('erpDocs.warehouse.layout.colorRequired') })}
                                        className="flex-1"
                                    />
                                </div>
                            </div>
                        </div>
                        <DialogFooter>
                            <Button type="button" variant="outline" onClick={() => setShowDialog(false)}>{t('common.cancel')}</Button>
                            <Button type="submit" disabled={createMutation.isPending || updateMutation.isPending}>
                                {(createMutation.isPending || updateMutation.isPending) && <Loader2 className="h-4 w-4 mr-1 animate-spin" />}
                                {t('common.confirm')}
                            </Button>
                        </DialogFooter>
                    </form>
                </DialogContent>
            </Dialog>

            <ConfirmDialog state={dialogState} />
            <WarehouseImportDialog open={showImportDialog} onOpenChange={setShowImportDialog} />
        </div>
    )
}
