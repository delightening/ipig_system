import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, {
    Warehouse,
    StorageLocationWithWarehouse,
    StorageLocationType,
    StorageLayoutItem,
    storageLocationTypeNames,
    StorageLocationInventoryItem,
    UpdateStorageLocationInventoryItemRequest,
} from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
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
import { toast } from '@/components/ui/use-toast'
import {
    Loader2,
    Lock,
    Unlock,
    Plus,
    Save,
    Check,
    Package,
    Warehouse as WarehouseIcon,
    Trash2,
} from 'lucide-react'
import { Responsive, WidthProvider } from 'react-grid-layout/legacy'
import 'react-grid-layout/css/styles.css'
import 'react-resizable/css/styles.css'

// 定義 react-grid-layout 的 LayoutItem 型別
interface GridLayoutItem {
    i: string
    x: number
    y: number
    w: number
    h: number
    minW?: number
    minH?: number
}

type Layouts = { [P: string]: GridLayoutItem[] }

const ResponsiveGridLayout = WidthProvider(Responsive)

const BREAKPOINTS = { lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }
const COLS = { lg: 12, md: 9, sm: 6, xs: 4, xxs: 2 }
const ROW_HEIGHT = 60

// 預設顏色
const DEFAULT_COLORS: Record<StorageLocationType, string> = {
    shelf: '#3b82f6',  // blue
    rack: '#10b981',   // green
    zone: '#f59e0b',   // amber
    bin: '#6366f1',    // indigo
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
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const { hasPermission } = useAuthStore()

    const [selectedWarehouseId, setSelectedWarehouseId] = useState<string>('')
    const [isEditMode, setIsEditMode] = useState(false)
    const [showDialog, setShowDialog] = useState(false)
    const [editingLocation, setEditingLocation] = useState<StorageLocationWithWarehouse | null>(null)
    const [formData, setFormData] = useState<FormData>(initialFormData)
    const [pendingLayoutChanges, setPendingLayoutChanges] = useState<StorageLayoutItem[]>([])
    const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)
    // 儲位庫存編輯狀態
    const [editingInventory, setEditingInventory] = useState<Record<string, string>>({})

    // 檢查是否有庫存編輯權限
    const canEditInventory = hasPermission('erp.storage.inventory.edit')

    // 取得倉庫列表
    const { data: warehouses, isLoading: loadingWarehouses } = useQuery({
        queryKey: ['warehouses'],
        queryFn: async () => {
            const res = await api.get<Warehouse[]>('/warehouses')
            return res.data.filter(w => w.is_active)
        },
    })

    // 取得儲位列表
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

    // 取得儲位庫存
    const { data: inventoryItems, isLoading: loadingInventory } = useQuery({
        queryKey: ['storage-location-inventory', editingLocation?.id],
        queryFn: async () => {
            const res = await api.get<StorageLocationInventoryItem[]>(
                `/storage-locations/${editingLocation!.id}/inventory`
            )
            return res.data
        },
        enabled: !!editingLocation,
    })

    // 建立儲位
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
            toast({ title: '成功', description: '儲位已建立' })
            setShowDialog(false)
            resetForm()
        },
        onError: (error: unknown) => {
            const err = error as { response?: { data?: { error?: { message?: string } } } }
            toast({
                title: '錯誤',
                description: err?.response?.data?.error?.message || '建立失敗',
                variant: 'destructive',
            })
        },
    })

    // 更新儲位
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
            toast({ title: '成功', description: '儲位已更新' })
            setShowDialog(false)
            resetForm()
        },
        onError: (error: unknown) => {
            const err = error as { response?: { data?: { error?: { message?: string } } } }
            toast({
                title: '錯誤',
                description: err?.response?.data?.error?.message || '更新失敗',
                variant: 'destructive',
            })
        },
    })

    // 刪除儲位
    const deleteMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.delete(`/storage-locations/${id}`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['storage-locations', selectedWarehouseId] })
            toast({ title: '成功', description: '儲位已刪除' })
        },
        onError: (error: unknown) => {
            const err = error as { response?: { data?: { error?: { message?: string } } } }
            toast({
                title: '錯誤',
                description: err?.response?.data?.error?.message || '刪除失敗',
                variant: 'destructive',
            })
        },
    })

    // 儲存佈局
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
        onError: (error: unknown) => {
            const err = error as { response?: { data?: { error?: { message?: string } } } }
            toast({
                title: '錯誤',
                description: err?.response?.data?.error?.message || '儲存佈局失敗',
                variant: 'destructive',
            })
        },
    })

    // 更新儲位庫存項目
    const updateInventoryMutation = useMutation({
        mutationFn: async ({ itemId, data }: { itemId: string; data: UpdateStorageLocationInventoryItemRequest }) => {
            return api.put<StorageLocationInventoryItem>(`/storage-locations/inventory/${itemId}`, data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['storage-location-inventory', editingLocation?.id] })
            queryClient.invalidateQueries({ queryKey: ['storage-locations', selectedWarehouseId] })
            toast({ title: '成功', description: '庫存數量已更新' })
        },
        onError: (error: unknown) => {
            const err = error as { response?: { data?: { error?: { message?: string } } } }
            toast({
                title: '錯誤',
                description: err?.response?.data?.error?.message || '更新失敗',
                variant: 'destructive',
            })
        },
    })

    const resetForm = () => {
        setFormData(initialFormData)
        setEditingLocation(null)
        setEditingInventory({})
    }

    const handleEdit = (location: StorageLocationWithWarehouse) => {
        setEditingLocation(location)
        setFormData({
            name: location.name || location.code,
            location_type: location.location_type,
            capacity: location.capacity?.toString() || '',
            color: location.color || DEFAULT_COLORS[location.location_type],
        })
        setShowDialog(true)
    }

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (editingLocation) {
            updateMutation.mutate({ id: editingLocation.id, data: formData })
        } else {
            createMutation.mutate(formData)
        }
    }

    const handleLayoutChange = (newLayout: GridLayoutItem[]) => {
        if (!isEditMode || !locations) return

        const items: StorageLayoutItem[] = newLayout.map((item) => ({
            id: item.i,
            row_index: item.y,
            col_index: item.x,
            width: item.w,
            height: item.h,
        }))

        setPendingLayoutChanges(items)
        setHasUnsavedChanges(true)
    }

    const handleSaveLayout = () => {
        if (pendingLayoutChanges.length > 0) {
            saveLayoutMutation.mutate(pendingLayoutChanges)
        }
    }

    // 轉換為 react-grid-layout 格式
    const gridLayout: GridLayoutItem[] = useMemo(() => {
        if (!locations) return []
        return locations.map((loc) => ({
            i: loc.id,
            x: loc.col_index,
            y: loc.row_index,
            w: loc.width,
            h: loc.height,
        }))
    }, [locations])

    const responsiveLayouts: Layouts = useMemo(() => {
        return {
            lg: gridLayout,
            md: gridLayout,
            sm: gridLayout,
            xs: gridLayout,
            xxs: gridLayout,
        }
    }, [gridLayout])

    const selectedWarehouse = warehouses?.find((w) => w.id === selectedWarehouseId)

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">倉庫平面圖編輯器</h1>
                    <p className="text-muted-foreground">拖拽調整貨架位置，視覺化管理倉庫佈局</p>
                </div>
                <div className="flex gap-2">
                    {isEditMode ? (
                        <>
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={() => {
                                    setIsEditMode(false)
                                    setHasUnsavedChanges(false)
                                    setPendingLayoutChanges([])
                                }}
                            >
                                <Lock className="h-4 w-4 mr-1" />
                                鎖定
                            </Button>
                            {hasUnsavedChanges && (
                                <Button
                                    size="sm"
                                    onClick={handleSaveLayout}
                                    disabled={saveLayoutMutation.isPending}
                                >
                                    {saveLayoutMutation.isPending && (
                                        <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                                    )}
                                    <Save className="h-4 w-4 mr-1" />
                                    儲存佈局
                                </Button>
                            )}
                        </>
                    ) : (
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setIsEditMode(true)}
                            disabled={!selectedWarehouseId}
                        >
                            <Unlock className="h-4 w-4 mr-1" />
                            解鎖編輯
                        </Button>
                    )}
                    <Button
                        size="sm"
                        onClick={() => {
                            resetForm()
                            setShowDialog(true)
                        }}
                        disabled={!selectedWarehouseId}
                    >
                        <Plus className="h-4 w-4 mr-1" />
                        新增儲位
                    </Button>
                </div>
            </div>

            {/* Warehouse Selector */}
            <Card>
                <CardHeader className="pb-3">
                    <CardTitle className="text-base flex items-center gap-2">
                        <WarehouseIcon className="h-4 w-4" />
                        選擇倉庫
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <Select
                        value={selectedWarehouseId}
                        onValueChange={setSelectedWarehouseId}
                    >
                        <SelectTrigger className="w-[300px]">
                            <SelectValue placeholder="請選擇倉庫..." />
                        </SelectTrigger>
                        <SelectContent>
                            {loadingWarehouses ? (
                                <div className="p-2 text-center">
                                    <Loader2 className="h-4 w-4 animate-spin mx-auto" />
                                </div>
                            ) : (
                                warehouses?.map((w) => (
                                    <SelectItem key={w.id} value={w.id}>
                                        {w.code} - {w.name}
                                    </SelectItem>
                                ))
                            )}
                        </SelectContent>
                    </Select>
                </CardContent>
            </Card>

            {/* Edit Mode Hint */}
            {isEditMode && (
                <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg text-sm text-blue-700">
                    📐 編輯模式啟用中：拖拽貨架調整位置，拖拽邊角調整大小。修改後記得點擊「儲存佈局」。
                </div>
            )}

            {/* Layout Grid */}
            {selectedWarehouseId ? (
                loadingLocations ? (
                    <div className="flex items-center justify-center py-16">
                        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                    </div>
                ) : locations && locations.length > 0 ? (
                    <div className="border rounded-lg bg-slate-50 p-4 min-h-[600px]">
                        <ResponsiveGridLayout
                            className="layout"
                            layouts={responsiveLayouts}
                            breakpoints={BREAKPOINTS}
                            cols={COLS}
                            rowHeight={ROW_HEIGHT}
                            onLayoutChange={(layout) => handleLayoutChange([...layout])}
                            isDraggable={isEditMode}
                            isResizable={isEditMode}
                            margin={[12, 12]}
                            containerPadding={[0, 0]}
                            useCSSTransforms={true}
                            autoSize={true}
                            compactType={null}
                        >
                            {locations.map((loc) => (
                                <div
                                    key={loc.id}
                                    className={`rounded-lg shadow-sm border-2 overflow-hidden transition-all ${isEditMode ? 'cursor-move' : 'cursor-pointer hover:opacity-90'}`}
                                    style={{
                                        backgroundColor: loc.color || DEFAULT_COLORS[loc.location_type],
                                        borderColor: isEditMode ? 'rgb(147, 197, 253)' : 'transparent',
                                    }}
                                    onClick={() => !isEditMode && handleEdit(loc)}
                                >
                                    <div className="h-full p-3 flex flex-col justify-between text-white">
                                        <div>
                                            <div className="font-bold text-lg" style={{ marginBottom: '10px' }}>{loc.name || loc.code}</div>
                                            <div className="text-sm opacity-90 truncate">{loc.code}</div>
                                        </div>
                                        <div className="flex items-center justify-between mt-2">
                                            <Badge
                                                variant="secondary"
                                                className="bg-white/20 text-white text-xs"
                                            >
                                                {storageLocationTypeNames[loc.location_type]}
                                            </Badge>
                                            <div className="flex items-center gap-1 text-sm">
                                                <Package className="h-3 w-3" />
                                                <span>
                                                    {loc.current_count}
                                                    {loc.capacity && `/${loc.capacity}`}
                                                </span>
                                            </div>
                                        </div>

                                    </div>
                                </div>
                            ))}
                        </ResponsiveGridLayout>
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
                        <Package className="h-16 w-16 mb-4" />
                        <p className="text-lg mb-2">此倉庫尚無儲位資料</p>
                        <Button onClick={() => setShowDialog(true)}>
                            <Plus className="h-4 w-4 mr-1" />
                            新增第一個儲位
                        </Button>
                    </div>
                )
            ) : (
                <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
                    <WarehouseIcon className="h-16 w-16 mb-4" />
                    <p className="text-lg">請先選擇一個倉庫</p>
                </div>
            )}

            {/* Create/Edit Dialog */}
            <Dialog open={showDialog} onOpenChange={setShowDialog}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>{editingLocation ? '編輯儲位' : '新增儲位'}</DialogTitle>
                        <DialogDescription>
                            {editingLocation
                                ? '修改儲位資料'
                                : `在 ${selectedWarehouse?.name || '倉庫'} 中建立新的儲位`}
                        </DialogDescription>
                    </DialogHeader>
                    <form onSubmit={handleSubmit}>
                        <div className="grid gap-4 py-4">
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="name" className="text-right">
                                    名稱 *
                                </Label>
                                <Input
                                    id="name"
                                    value={formData.name}
                                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                                    className="col-span-3"
                                    placeholder="如 冷藏區-1號架"
                                    required
                                />
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="type" className="text-right">
                                    類型
                                </Label>
                                <Select
                                    value={formData.location_type}
                                    onValueChange={(v: StorageLocationType) => {
                                        setFormData({
                                            ...formData,
                                            location_type: v,
                                            color: DEFAULT_COLORS[v],
                                        })
                                    }}
                                >
                                    <SelectTrigger className="col-span-3">
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {Object.entries(storageLocationTypeNames).map(([key, label]) => (
                                            <SelectItem key={key} value={key}>
                                                {label}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="capacity" className="text-right">
                                    容量
                                </Label>
                                <Input
                                    id="capacity"
                                    type="number"
                                    value={formData.capacity}
                                    onChange={(e) => setFormData({ ...formData, capacity: e.target.value })}
                                    className="col-span-3"
                                    placeholder="選填，如 100"
                                />
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="color" className="text-right">
                                    顏色
                                </Label>
                                <div className="col-span-3 flex gap-2 items-center">
                                    <input
                                        type="color"
                                        id="color"
                                        value={formData.color}
                                        onChange={(e) => setFormData({ ...formData, color: e.target.value })}
                                        className="h-10 w-14 rounded border cursor-pointer"
                                    />
                                    <span className="text-sm text-muted-foreground">{formData.color}</span>
                                </div>
                            </div>
                        </div>

                        {/* 儲位庫存明細 */}
                        {editingLocation && (
                            <div className="mt-4 pt-4 border-t">
                                <h4 className="font-medium mb-3 flex items-center gap-2">
                                    <Package className="h-4 w-4" />
                                    儲位庫存 ({inventoryItems?.length || 0} 項)
                                </h4>
                                {loadingInventory ? (
                                    <div className="flex justify-center py-4">
                                        <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                                    </div>
                                ) : inventoryItems && inventoryItems.length > 0 ? (
                                    <div className="max-h-48 overflow-y-auto border rounded-md">
                                        <table className="w-full text-sm">
                                            <thead className="bg-muted sticky top-0">
                                                <tr>
                                                    <th className="text-left p-2">產品</th>
                                                    <th className="text-right p-2">數量</th>
                                                    <th className="text-left p-2">批號</th>
                                                    <th className="text-left p-2">效期</th>
                                                    {canEditInventory && <th className="p-2 w-16"></th>}
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {inventoryItems.map((item) => (
                                                    <tr key={item.id} className="border-t">
                                                        <td className="p-2">
                                                            <div className="font-medium">{item.product_name}</div>
                                                            <div className="text-xs text-muted-foreground">{item.product_sku}</div>
                                                        </td>
                                                        <td className="text-right p-2">
                                                            {canEditInventory ? (
                                                                <div className="flex items-center justify-end gap-1">
                                                                    <Input
                                                                        type="number"
                                                                        step="0.01"
                                                                        min="0"
                                                                        className="w-24 h-7 text-right text-sm"
                                                                        value={editingInventory[item.id] ?? item.on_hand_qty}
                                                                        onChange={(e) => setEditingInventory({
                                                                            ...editingInventory,
                                                                            [item.id]: e.target.value
                                                                        })}
                                                                    />
                                                                    <span className="text-xs text-muted-foreground">{item.base_uom}</span>
                                                                </div>
                                                            ) : (
                                                                <>{parseFloat(item.on_hand_qty).toLocaleString()} {item.base_uom}</>
                                                            )}
                                                        </td>
                                                        <td className="p-2 text-muted-foreground">{item.batch_no || '-'}</td>
                                                        <td className="p-2 text-muted-foreground">{item.expiry_date || '-'}</td>
                                                        {canEditInventory && (
                                                            <td className="p-2">
                                                                <Button
                                                                    type="button"
                                                                    variant="ghost"
                                                                    size="icon"
                                                                    className="h-7 w-7"
                                                                    disabled={
                                                                        updateInventoryMutation.isPending ||
                                                                        (editingInventory[item.id] === undefined) ||
                                                                        (editingInventory[item.id] === item.on_hand_qty)
                                                                    }
                                                                    onClick={() => {
                                                                        const newQty = editingInventory[item.id]
                                                                        if (newQty !== undefined && newQty !== item.on_hand_qty) {
                                                                            updateInventoryMutation.mutate({
                                                                                itemId: item.id,
                                                                                data: { on_hand_qty: newQty }
                                                                            })
                                                                        }
                                                                    }}
                                                                >
                                                                    {updateInventoryMutation.isPending ? (
                                                                        <Loader2 className="h-4 w-4 animate-spin" />
                                                                    ) : (
                                                                        <Check className="h-4 w-4 text-green-600" />
                                                                    )}
                                                                </Button>
                                                            </td>
                                                        )}
                                                    </tr>
                                                ))}
                                            </tbody>
                                        </table>
                                    </div>
                                ) : (
                                    <p className="text-sm text-muted-foreground text-center py-4">此儲位尚無庫存</p>
                                )}
                            </div>
                        )}
                        <DialogFooter className="flex justify-between">
                            <div>
                                {editingLocation && (
                                    <Button
                                        type="button"
                                        variant="destructive"
                                        onClick={() => {
                                            if (confirm('確定要刪除此儲位嗎？此操作無法復原。')) {
                                                deleteMutation.mutate(editingLocation.id)
                                                setShowDialog(false)
                                                resetForm()
                                            }
                                        }}
                                        disabled={deleteMutation.isPending}
                                    >
                                        {deleteMutation.isPending ? (
                                            <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                                        ) : (
                                            <Trash2 className="h-4 w-4 mr-1" />
                                        )}
                                        刪除儲位
                                    </Button>
                                )}
                            </div>
                            <div className="flex gap-2">
                                <Button type="button" variant="outline" onClick={() => setShowDialog(false)}>
                                    取消
                                </Button>
                                <Button
                                    type="submit"
                                    disabled={createMutation.isPending || updateMutation.isPending}
                                >
                                    {(createMutation.isPending || updateMutation.isPending) && (
                                        <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                                    )}
                                    {editingLocation ? '更新' : '建立'}
                                </Button>
                            </div>
                        </DialogFooter>
                    </form>
                </DialogContent>
            </Dialog>
        </div>
    )
}
