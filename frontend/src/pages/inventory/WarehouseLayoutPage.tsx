import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
    Warehouse,
    StorageLocationWithWarehouse,
    StorageLocationType,
    StorageLayoutItem,
    StorageLocationInventoryItem,
    UpdateStorageLocationInventoryItemRequest,
    UnassignedInventoryItem,
} from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
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

interface FormData {
    name: string
    location_type: StorageLocationType
    capacity: string
    color: string
}

const initialFormData: FormData = {
    name: '',
    location_type: 'shelf',
    capacity: '',
    color: DEFAULT_COLORS.shelf,
}

export function WarehouseLayoutPage() {
    const queryClient = useQueryClient()
    const { hasPermission } = useAuthStore()
    const { dialogState, confirm } = useConfirmDialog()

    // 狀態管理
    const [selectedWarehouseId, setSelectedWarehouseId] = useState<string>('')
    const [isEditMode, setIsEditMode] = useState(false)
    const [showDialog, setShowDialog] = useState(false)
    const [editingLocation, setEditingLocation] = useState<StorageLocationWithWarehouse | null>(null)
    const [formData, setFormData] = useState<FormData>(initialFormData)
    const [pendingLayoutChanges, setPendingLayoutChanges] = useState<StorageLayoutItem[]>([])
    const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)
    const [editingInventory, setEditingInventory] = useState<Record<string, string>>({})
    const [selectedLocation, setSelectedLocation] = useState<StorageLocationWithWarehouse | null>(null)
    const [activeTab, setActiveTab] = useState<string>('location-inventory')
    const [showImportDialog, setShowImportDialog] = useState(false)

    const canEditInventory = hasPermission('erp.storage.inventory.edit')

    // 取得啟用中的倉庫（供選擇器使用）
    const { data: warehouses } = useQuery({
        queryKey: ['warehouses'],
        queryFn: async () => {
            const res = await api.get<Warehouse[]>('/warehouses')
            return res.data.filter(w => w.is_active)
        },
    })

    // 初始化選擇第一個倉庫
    useEffect(() => {
        if (warehouses && warehouses.length > 0 && !selectedWarehouseId) {
            setSelectedWarehouseId(warehouses[0].id)
        }
    }, [warehouses, selectedWarehouseId])

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
        mutationFn: async (data: typeof formData) => {
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
            toast({ title: '成功', description: '儲位/結構已建立' })
            setShowDialog(false)
        },
    })

    const updateMutation = useMutation({
        mutationFn: async ({ id, data }: { id: string; data: typeof formData }) => {
            return api.put(`/storage-locations/${id}`, {
                name: data.name,
                location_type: data.location_type,
                capacity: data.capacity ? parseInt(data.capacity) : undefined,
                color: data.color,
            })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['storage-locations', selectedWarehouseId] })
            toast({ title: '成功', description: '儲位/結構已更新' })
            setShowDialog(false)
        },
    })

    const saveLayoutMutation = useMutation({
        mutationFn: async (items: StorageLayoutItem[]) => {
            return api.put(`/warehouses/${selectedWarehouseId}/layout`, { items })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['storage-locations', selectedWarehouseId] })
            toast({ title: '成功', description: '佈局已儲存' })
            setHasUnsavedChanges(false)
            setPendingLayoutChanges([])
        },
    })

    const updateInventoryMutation = useMutation({
        mutationFn: async ({ itemId, qty }: { itemId: string; qty: string }) => {
            const data: UpdateStorageLocationInventoryItemRequest = { on_hand_qty: qty }
            return api.put(`/storage-locations/inventory/${itemId}`, data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['storage-location-inventory', selectedLocation?.id] })
            queryClient.invalidateQueries({ queryKey: ['storage-locations', selectedWarehouseId] })
            toast({ title: '成功', description: '庫存已更新' })
            setEditingInventory({})
        },
    })

    // Event Handlers
    const handleAddLocation = () => {
        setEditingLocation(null)
        setFormData(initialFormData)
        setShowDialog(true)
    }

    const handleEditLocation = (loc: StorageLocationWithWarehouse) => {
        setEditingLocation(loc)
        setFormData({
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
        const headers = ['代碼', '名稱', '地址', '狀態']
        const rows = warehouses.map((w) => [
            w.code,
            w.name,
            w.address || '',
            w.is_active ? '啟用' : '停用',
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
                {selectedWarehouseId ? (
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
                    <div className="h-64 flex items-center justify-center border rounded-lg bg-slate-50 text-muted-foreground">
                        請先選擇一個倉庫以檢視佈局
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
                        canEditInventory={canEditInventory}
                        editingInventory={editingInventory}
                        setEditingInventory={setEditingInventory}
                        onUpdateInventory={(itemId, qty) => updateInventoryMutation.mutate({ itemId, qty })}
                        isUpdatingInventory={updateInventoryMutation.isPending}
                        onEditLocationClick={handleEditLocation}
                    />
                )}
            </div>

            {/* 儲位編輯/建立 Dialog */}
            <Dialog open={showDialog} onOpenChange={setShowDialog}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>{editingLocation ? '編輯項目' : '新增項目'}</DialogTitle>
                        <DialogDescription>建立儲位或是牆壁、門、窗等建築結構</DialogDescription>
                    </DialogHeader>
                    <form onSubmit={(e) => {
                        e.preventDefault()
                        if (editingLocation) {
                            updateMutation.mutate({ id: editingLocation.id, data: formData })
                        } else {
                            createMutation.mutate(formData)
                        }
                    }}>
                        <div className="grid gap-4 py-4">
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="loc-name" className="text-right">名稱 *</Label>
                                <Input
                                    id="loc-name"
                                    value={formData.name}
                                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                                    className="col-span-3"
                                    required
                                />
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="loc-type" className="text-right">類型</Label>
                                <Select
                                    value={formData.location_type}
                                    onValueChange={(v: StorageLocationType) => setFormData({
                                        ...formData,
                                        location_type: v,
                                        color: DEFAULT_COLORS[v]
                                    })}
                                >
                                    <SelectTrigger className="col-span-3">
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="shelf">貨架</SelectItem>
                                        <SelectItem value="rack">儲物架</SelectItem>
                                        <SelectItem value="zone">區域</SelectItem>
                                        <SelectItem value="bin">儲物格</SelectItem>
                                        <SelectItem value="wall">牆壁</SelectItem>
                                        <SelectItem value="door">門</SelectItem>
                                        <SelectItem value="window">窗戶</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                            {['shelf', 'rack', 'zone', 'bin'].includes(formData.location_type) && (
                                <div className="grid grid-cols-4 items-center gap-4">
                                    <Label htmlFor="loc-capacity" className="text-right">容量</Label>
                                    <Input
                                        id="loc-capacity"
                                        type="number"
                                        value={formData.capacity}
                                        onChange={(e) => setFormData({ ...formData, capacity: e.target.value })}
                                        className="col-span-3"
                                    />
                                </div>
                            )}
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="loc-color" className="text-right">顏色</Label>
                                <div className="col-span-3 flex gap-2">
                                    <input
                                        type="color"
                                        value={formData.color}
                                        onChange={(e) => setFormData({ ...formData, color: e.target.value })}
                                        className="h-10 w-14 rounded border cursor-pointer"
                                    />
                                    <Input
                                        value={formData.color}
                                        onChange={(e) => setFormData({ ...formData, color: e.target.value })}
                                        className="flex-1"
                                    />
                                </div>
                            </div>
                        </div>
                        <DialogFooter>
                            <Button type="button" variant="outline" onClick={() => setShowDialog(false)}>取消</Button>
                            <Button type="submit" disabled={createMutation.isPending || updateMutation.isPending}>
                                {(createMutation.isPending || updateMutation.isPending) && <Loader2 className="h-4 w-4 mr-1 animate-spin" />}
                                確認
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
