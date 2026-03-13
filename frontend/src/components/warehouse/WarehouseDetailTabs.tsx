import {
    Warehouse,
    StorageLocationWithWarehouse,
    StorageLocationInventoryItem,
    storageLocationTypeNames,
    UnassignedInventoryItem,
    UpdateStorageLocationInventoryItemRequest,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
    Loader2,
    Package,
    Edit3,
    Check,
} from 'lucide-react'
import { formatUom } from '@/lib/utils'

interface WarehouseDetailTabsProps {
    warehouse: Warehouse | undefined
    selectedLocation: StorageLocationWithWarehouse | null
    onLocationSelect: (loc: StorageLocationWithWarehouse) => void
    locations: StorageLocationWithWarehouse[]
    loadingLocations: boolean
    inventoryItems: StorageLocationInventoryItem[] | undefined
    loadingInventory: boolean
    unassignedItems: UnassignedInventoryItem[] | undefined
    loadingUnassigned: boolean
    activeTab: string
    onTabChange: (v: string) => void
    canEditInventory: boolean
    editingInventory: Record<string, string>
    setEditingInventory: (v: Record<string, string>) => void
    onUpdateInventory: (itemId: string, qty: string) => void
    isUpdatingInventory: boolean
    onEditLocationClick: (loc: StorageLocationWithWarehouse) => void
}

export function WarehouseDetailTabs({
    selectedLocation,
    onLocationSelect,
    locations,
    loadingLocations,
    inventoryItems,
    loadingInventory,
    unassignedItems,
    loadingUnassigned,
    activeTab,
    onTabChange,
    canEditInventory,
    editingInventory,
    setEditingInventory,
    onUpdateInventory,
    isUpdatingInventory,
    onEditLocationClick,
}: WarehouseDetailTabsProps) {
    // 過濾出非建築結構的儲位供列表顯示
    const filteredLocations = locations.filter(loc => !['wall', 'door', 'window'].includes(loc.location_type))

    return (
        <Tabs
            value={activeTab}
            onValueChange={onTabChange}
            className="space-y-4"
        >
            <TabsList>
                <TabsTrigger value="location-inventory">儲位庫存</TabsTrigger>
                <TabsTrigger value="location-list">儲位列表</TabsTrigger>
                <TabsTrigger value="unassigned">
                    未分配庫存
                    {unassignedItems && unassignedItems.length > 0 && (
                        <span className="ml-1 inline-flex items-center justify-center rounded-full bg-amber-500 text-white text-[10px] px-1.5 min-w-[18px] h-[18px]">
                            {unassignedItems.length}
                        </span>
                    )}
                </TabsTrigger>
            </TabsList>

            <TabsContent value="location-inventory">
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between py-4">
                        <CardTitle className="text-base flex items-center gap-2">
                            <Package className="h-4 w-4" />
                            {selectedLocation
                                ? `儲位庫存：${selectedLocation.name || selectedLocation.code}`
                                : '儲位庫存'}
                        </CardTitle>
                        {selectedLocation && (
                            <Button
                                size="sm"
                                variant="outline"
                                onClick={() => onEditLocationClick(selectedLocation)}
                            >
                                <Edit3 className="h-4 w-4 mr-1" />
                                編輯儲位
                            </Button>
                        )}
                    </CardHeader>
                    <CardContent>
                        {!selectedLocation ? (
                            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground bg-slate-50 border border-dashed rounded-lg">
                                <Package className="h-10 w-10 mb-2 opacity-20" />
                                <p className="text-sm">請從上方佈局圖或「儲位列表」中選擇一個儲位以檢視庫存。</p>
                            </div>
                        ) : loadingInventory ? (
                            <div className="flex justify-center py-12">
                                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                            </div>
                        ) : inventoryItems && inventoryItems.length > 0 ? (
                            <div className="max-h-[400px] overflow-y-auto border rounded-md">
                                <Table>
                                    <TableHeader className="sticky top-0 bg-muted z-10">
                                        <TableRow>
                                            <TableHead>產品</TableHead>
                                            <TableHead className="text-right">數量</TableHead>
                                            <TableHead>批號</TableHead>
                                            <TableHead>效期</TableHead>
                                            {canEditInventory && <TableHead className="w-16" />}
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {inventoryItems.map((item) => (
                                            <TableRow key={item.id}>
                                                <TableCell>
                                                    <div className="font-medium text-sm">
                                                        {item.product_name}
                                                    </div>
                                                    <div className="text-[11px] text-muted-foreground font-mono">
                                                        {item.product_sku}
                                                    </div>
                                                </TableCell>
                                                <TableCell className="text-right">
                                                    {canEditInventory ? (
                                                        <div className="flex items-center justify-end gap-1">
                                                            <Input
                                                                type="number"
                                                                step="0.01"
                                                                min="0"
                                                                className="w-24 h-8 text-right text-xs"
                                                                value={
                                                                    editingInventory[item.id] ??
                                                                    item.on_hand_qty
                                                                }
                                                                onChange={(e) =>
                                                                    setEditingInventory({
                                                                        ...editingInventory,
                                                                        [item.id]: e.target.value,
                                                                    })
                                                                }
                                                            />
                                                            <span className="text-[11px] text-muted-foreground whitespace-nowrap">
                                                                {formatUom(item.base_uom)}
                                                            </span>
                                                        </div>
                                                    ) : (
                                                        <span className="text-sm font-medium">
                                                            {parseFloat(item.on_hand_qty).toLocaleString()}{' '}
                                                            {formatUom(item.base_uom)}
                                                        </span>
                                                    )}
                                                </TableCell>
                                                <TableCell className="text-xs text-muted-foreground">
                                                    {item.batch_no || '-'}
                                                </TableCell>
                                                <TableCell className="text-xs text-muted-foreground">
                                                    {item.expiry_date || '-'}
                                                </TableCell>
                                                {canEditInventory && (
                                                    <TableCell className="text-right">
                                                        <Button
                                                            type="button"
                                                            variant="ghost"
                                                            size="icon"
                                                            className="h-8 w-8"
                                                            disabled={
                                                                isUpdatingInventory ||
                                                                editingInventory[item.id] === undefined ||
                                                                editingInventory[item.id] === item.on_hand_qty
                                                            }
                                                            onClick={() => onUpdateInventory(item.id, editingInventory[item.id])}
                                                        >
                                                            {isUpdatingInventory ? (
                                                                <Loader2 className="h-4 w-4 animate-spin" />
                                                            ) : (
                                                                <Check className="h-4 w-4 text-green-600" />
                                                            )}
                                                        </Button>
                                                    </TableCell>
                                                )}
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            </div>
                        ) : (
                            <div className="text-center py-12 text-muted-foreground bg-slate-50/50 rounded-lg">
                                <p className="text-sm">此儲位目前尚無庫存。</p>
                            </div>
                        )}
                    </CardContent>
                </Card>
            </TabsContent>

            <TabsContent value="location-list">
                <Card>
                    <CardHeader>
                        <CardTitle className="text-base flex items-center gap-2">
                             <Edit3 className="h-4 w-4" />
                             儲位列表
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        {loadingLocations ? (
                            <div className="flex justify-center py-12">
                                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                            </div>
                        ) : filteredLocations.length > 0 ? (
                            <div className="border rounded-md max-h-[400px] overflow-y-auto">
                                <Table>
                                    <TableHeader className="sticky top-0 bg-muted z-10">
                                        <TableRow>
                                            <TableHead>名稱</TableHead>
                                            <TableHead>代碼</TableHead>
                                            <TableHead>類型</TableHead>
                                            <TableHead className="text-right">產品數量</TableHead>
                                            <TableHead className="text-right">容量</TableHead>
                                            <TableHead className="w-16" />
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {filteredLocations.map((loc) => {
                                            const isSelected = selectedLocation?.id === loc.id
                                            return (
                                                <TableRow
                                                    key={loc.id}
                                                    className={isSelected ? 'bg-blue-50/50' : 'cursor-pointer hover:bg-slate-50'}
                                                    onClick={() => {
                                                        onLocationSelect(loc)
                                                        onTabChange('location-inventory')
                                                    }}
                                                >
                                                    <TableCell className="font-medium text-sm">
                                                        {loc.name || loc.code}
                                                    </TableCell>
                                                    <TableCell className="font-mono text-xs text-muted-foreground">
                                                        {loc.code}
                                                    </TableCell>
                                                    <TableCell className="text-xs">
                                                        {storageLocationTypeNames[loc.location_type]}
                                                    </TableCell>
                                                    <TableCell className="text-right text-sm">
                                                        {loc.current_count}
                                                    </TableCell>
                                                    <TableCell className="text-right text-sm text-muted-foreground">
                                                        {loc.capacity ?? '-'}
                                                    </TableCell>
                                                    <TableCell className="text-right">
                                                        <Button
                                                            variant="ghost"
                                                            size="icon"
                                                            className="h-8 w-8"
                                                            onClick={(e) => {
                                                                e.stopPropagation()
                                                                onEditLocationClick(loc)
                                                            }}
                                                        >
                                                            <Edit3 className="h-4 w-4" />
                                                        </Button>
                                                    </TableCell>
                                                </TableRow>
                                            )
                                        })}
                                    </TableBody>
                                </Table>
                            </div>
                        ) : (
                            <div className="text-center py-12 text-muted-foreground bg-slate-50 rounded-lg">
                                <p className="text-sm">此倉庫尚未建立儲位。</p>
                            </div>
                        )}
                    </CardContent>
                </Card>
            </TabsContent>

            <TabsContent value="unassigned">
                <Card>
                    <CardHeader>
                        <CardTitle className="text-base flex items-center gap-2 text-amber-600">
                             <Package className="h-4 w-4" />
                             未分配庫存
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        {loadingUnassigned ? (
                            <div className="flex justify-center py-12">
                                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                            </div>
                        ) : !unassignedItems || unassignedItems.length === 0 ? (
                            <div className="text-center py-12 text-muted-foreground bg-slate-50 rounded-lg border-dashed border">
                                <p className="text-sm text-green-600 font-medium">✨ 所有庫存均已分配至具體儲位。</p>
                            </div>
                        ) : (
                            <div className="border rounded-md max-h-[400px] overflow-y-auto">
                                <Table>
                                    <TableHeader className="sticky top-0 bg-muted z-10">
                                        <TableRow>
                                            <TableHead>產品</TableHead>
                                            <TableHead className="text-right">倉庫總庫存</TableHead>
                                            <TableHead className="text-right">已在儲位</TableHead>
                                            <TableHead className="text-right">未分配數量</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {unassignedItems.map((item) => (
                                            <TableRow key={`${item.warehouse_id}-${item.product_id}`}>
                                                <TableCell>
                                                    <div className="font-medium text-sm">{item.product_name}</div>
                                                    <div className="text-[11px] text-muted-foreground font-mono">{item.product_sku}</div>
                                                </TableCell>
                                                <TableCell className="text-right text-sm">
                                                    {parseFloat(item.qty_on_warehouse).toLocaleString()} {formatUom(item.base_uom)}
                                                </TableCell>
                                                <TableCell className="text-right text-sm text-muted-foreground">
                                                    {parseFloat(item.qty_on_shelves).toLocaleString()} {formatUom(item.base_uom)}
                                                </TableCell>
                                                <TableCell className="text-right font-semibold text-amber-700 text-sm">
                                                    {parseFloat(item.qty_unassigned).toLocaleString()} {formatUom(item.base_uom)}
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            </div>
                        )}
                    </CardContent>
                </Card>
            </TabsContent>
        </Tabs>
    )
}
