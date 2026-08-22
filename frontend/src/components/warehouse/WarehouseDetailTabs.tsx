import api, {
    Warehouse,
    StorageLocationWithWarehouse,
    StorageLocationInventoryItem,
    storageLocationTypeNames,
    UnassignedInventoryItem,
} from '@/lib/api'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import type { UnassignedSourceDoc } from '@/types/erp'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
    Loader2,
    Package,
    Edit3,
    PackagePlus,
    ChevronRight,
} from 'lucide-react'
import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { cn, formatUom } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { AssignToShelfDialog } from '@/pages/inventory/components/AssignToShelfDialog'

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
    onEditLocationClick,
}: WarehouseDetailTabsProps) {
    const filteredLocations = locations.filter(loc => !['wall', 'door', 'window'].includes(loc.location_type))

    const { sortedData: sortedInventory, sort: invSort, toggleSort: toggleInvSort } = useTableSort(inventoryItems)
    const { sortedData: sortedLocations, sort: locSort, toggleSort: toggleLocSort } = useTableSort(filteredLocations)
    const { sortedData: sortedUnassigned, sort: unaSort, toggleSort: toggleUnaSort } = useTableSort(unassignedItems)

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
                        <span className="ml-1 inline-flex items-center justify-center rounded-full bg-status-warning-solid text-white text-[10px] px-1.5 min-w-[18px] h-[18px]">
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
                            <Can permission={PERMISSIONS.ERP_STORAGE_EDIT}>
                              <Button
                                  size="sm"
                                  variant="outline"
                                  onClick={() => onEditLocationClick(selectedLocation)}
                              >
                                  <Edit3 className="h-4 w-4 mr-1" />
                                  編輯儲位
                              </Button>
                            </Can>
                        )}
                    </CardHeader>
                    <CardContent>
                        {!selectedLocation ? (
                            <div className="flex flex-col items-center justify-center py-12 text-muted-foreground bg-muted border border-dashed rounded-lg">
                                <Package className="h-10 w-10 mb-2 opacity-20" />
                                <p className="text-sm">請從上方佈局圖或「儲位列表」中選擇一個儲位以檢視庫存。</p>
                            </div>
                        ) : loadingInventory ? (
                            <div className="flex justify-center py-12">
                                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                            </div>
                        ) : sortedInventory && sortedInventory.length > 0 ? (
                            <div className="max-h-[400px] overflow-y-auto border rounded-md">
                                <Table>
                                    <TableHeader className="sticky top-0 bg-muted z-10">
                                        <TableRow>
                                            <SortableTableHead sortKey="product_name" currentSort={invSort.column} currentDirection={invSort.direction} onSort={toggleInvSort}>產品</SortableTableHead>
                                            <SortableTableHead sortKey="on_hand_qty" currentSort={invSort.column} currentDirection={invSort.direction} onSort={toggleInvSort} className="text-right">數量</SortableTableHead>
                                            <SortableTableHead sortKey="batch_no" currentSort={invSort.column} currentDirection={invSort.direction} onSort={toggleInvSort}>批號</SortableTableHead>
                                            <SortableTableHead sortKey="expiry_date" currentSort={invSort.column} currentDirection={invSort.direction} onSort={toggleInvSort}>效期</SortableTableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {sortedInventory.map((item) => (
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
                                                    <span className="text-sm font-medium">
                                                        {Math.trunc(parseFloat(item.on_hand_qty || '0') || 0).toLocaleString()}{' '}
                                                        {formatUom(item.base_uom)}
                                                    </span>
                                                </TableCell>
                                                <TableCell className="text-xs text-muted-foreground">
                                                    {item.batch_no || '-'}
                                                </TableCell>
                                                <TableCell className="text-xs text-muted-foreground">
                                                    {item.expiry_date || '-'}
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            </div>
                        ) : (
                            <div className="text-center py-12 text-muted-foreground bg-muted/50 rounded-lg">
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
                        ) : sortedLocations && sortedLocations.length > 0 ? (
                            <div className="border rounded-md max-h-[400px] overflow-y-auto">
                                <Table>
                                    <TableHeader className="sticky top-0 bg-muted z-10">
                                        <TableRow>
                                            <SortableTableHead sortKey="name" currentSort={locSort.column} currentDirection={locSort.direction} onSort={toggleLocSort}>名稱</SortableTableHead>
                                            <SortableTableHead sortKey="code" currentSort={locSort.column} currentDirection={locSort.direction} onSort={toggleLocSort}>代碼</SortableTableHead>
                                            <SortableTableHead sortKey="location_type" currentSort={locSort.column} currentDirection={locSort.direction} onSort={toggleLocSort}>類型</SortableTableHead>
                                            <SortableTableHead sortKey="current_count" currentSort={locSort.column} currentDirection={locSort.direction} onSort={toggleLocSort} className="text-right">產品數量</SortableTableHead>
                                            <SortableTableHead sortKey="capacity" currentSort={locSort.column} currentDirection={locSort.direction} onSort={toggleLocSort} className="text-right">容量</SortableTableHead>
                                            <TableHead className="w-16" />
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {sortedLocations.map((loc) => {
                                            const isSelected = selectedLocation?.id === loc.id
                                            return (
                                                <TableRow
                                                    key={loc.id}
                                                    className={isSelected ? 'bg-status-info-bg/50' : 'cursor-pointer hover:bg-muted'}
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
                                                        <Can permission={PERMISSIONS.ERP_STORAGE_EDIT}>
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
                                                        </Can>
                                                    </TableCell>
                                                </TableRow>
                                            )
                                        })}
                                    </TableBody>
                                </Table>
                            </div>
                        ) : (
                            <div className="text-center py-12 text-muted-foreground bg-muted rounded-lg">
                                <p className="text-sm">此倉庫尚未建立儲位。</p>
                            </div>
                        )}
                    </CardContent>
                </Card>
            </TabsContent>

            <TabsContent value="unassigned">
                <Card>
                    <CardHeader>
                        <CardTitle className="text-base flex items-center gap-2 text-status-warning-text">
                             <Package className="h-4 w-4" />
                             未分配庫存
                        </CardTitle>
                    </CardHeader>
                    <CardContent>
                        {loadingUnassigned ? (
                            <div className="flex justify-center py-12">
                                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                            </div>
                        ) : !sortedUnassigned || sortedUnassigned.length === 0 ? (
                            <div className="text-center py-12 text-muted-foreground bg-muted rounded-lg border-dashed border">
                                <p className="text-sm text-status-success-text font-medium">✨ 所有庫存均已分配至具體儲位。</p>
                            </div>
                        ) : (
                            <div className="border rounded-md max-h-[400px] overflow-y-auto">
                                <Table>
                                    <TableHeader className="sticky top-0 bg-muted z-10">
                                        <TableRow>
                                            <SortableTableHead sortKey="product_name" currentSort={unaSort.column} currentDirection={unaSort.direction} onSort={toggleUnaSort}>產品</SortableTableHead>
                                            <SortableTableHead sortKey="qty_on_warehouse" currentSort={unaSort.column} currentDirection={unaSort.direction} onSort={toggleUnaSort} className="text-right">倉庫總庫存</SortableTableHead>
                                            <SortableTableHead sortKey="qty_on_shelves" currentSort={unaSort.column} currentDirection={unaSort.direction} onSort={toggleUnaSort} className="text-right">已在儲位</SortableTableHead>
                                            <SortableTableHead sortKey="qty_unassigned" currentSort={unaSort.column} currentDirection={unaSort.direction} onSort={toggleUnaSort} className="text-right">未分配數量</SortableTableHead>
                                            <TableHead className="w-[120px] text-right">操作</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {sortedUnassigned.map((item) => (
                                            <UnassignedRow key={`${item.warehouse_id}-${item.product_id}`} item={item} />
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

function UnassignedRow({ item }: { item: UnassignedInventoryItem }) {
    const [open, setOpen] = useState(false)
    const [sourcesOpen, setSourcesOpen] = useState(false)
    const unassignedQty = parseFloat(item.qty_unassigned)
    return (
        <>
            <TableRow>
                <TableCell>
                    <div className="flex items-start gap-1.5">
                        <button
                            type="button"
                            onClick={() => setSourcesOpen((v) => !v)}
                            className="mt-0.5 text-muted-foreground/60 hover:text-foreground"
                            title="查看來源單據"
                            aria-label="查看來源單據"
                        >
                            <ChevronRight className={cn('h-4 w-4 transition-transform', sourcesOpen && 'rotate-90')} />
                        </button>
                        <div>
                            <div className="font-medium text-sm">{item.product_name}</div>
                            <div className="text-[11px] text-muted-foreground font-mono">{item.product_sku}</div>
                        </div>
                    </div>
                </TableCell>
                <TableCell className="text-right text-sm">
                    {parseFloat(item.qty_on_warehouse).toLocaleString()} {formatUom(item.base_uom)}
                </TableCell>
                <TableCell className="text-right text-sm text-muted-foreground">
                    {parseFloat(item.qty_on_shelves).toLocaleString()} {formatUom(item.base_uom)}
                </TableCell>
                <TableCell className="text-right font-semibold text-status-warning-text text-sm">
                    {unassignedQty.toLocaleString()} {formatUom(item.base_uom)}
                </TableCell>
                <TableCell className="text-right">
                    {/* R89-6：後端 assign_unassigned_inventory 檢查的是 erp.stock.adjust，
                        不是 erp.storage.inventory.edit——原本這裡打錯權限碼，會讓沒有
                        調整庫存權限但有儲位編輯權限的角色看到能點但一定 403 的按鈕。 */}
                    <Can permission={PERMISSIONS.ERP_STOCK_ADJUST}>
                      <Button size="sm" variant="outline" className="h-7 gap-1.5" onClick={() => setOpen(true)}>
                          <PackagePlus className="h-3.5 w-3.5" />
                          分配
                      </Button>
                    </Can>
                </TableCell>
            </TableRow>
            {sourcesOpen && (
                <UnassignedSourceRows
                    warehouseId={item.warehouse_id}
                    productId={item.product_id}
                    baseUom={item.base_uom}
                />
            )}
            <AssignToShelfDialog
                open={open}
                onOpenChange={setOpen}
                warehouseId={item.warehouse_id}
                warehouseName={item.warehouse_name}
                productId={item.product_id}
                productName={item.product_name}
                productSku={item.product_sku}
                unassignedQty={unassignedQty}
                baseUom={item.base_uom}
            />
        </>
    )
}

/** 展開列：追溯造成此未分配的來源採購入庫單（GRN）明細 */
function UnassignedSourceRows({
    warehouseId,
    productId,
    baseUom,
}: {
    warehouseId: string
    productId: string
    baseUom: string
}) {
    const { data, isLoading, isError } = useQuery({
        queryKey: ['inventory', 'unassigned', 'sources', warehouseId, productId],
        queryFn: async () => {
            const res = await api.get<UnassignedSourceDoc[]>('/inventory/unassigned/sources', {
                params: { warehouse_id: warehouseId, product_id: productId },
            })
            return res.data
        },
    })

    if (isLoading) {
        return (
            <TableRow className="bg-muted/20">
                <TableCell colSpan={5} className="py-2 pl-10 text-xs text-muted-foreground">
                    <Loader2 className="inline h-3 w-3 animate-spin mr-1.5" />
                    載入來源單據...
                </TableCell>
            </TableRow>
        )
    }
    if (isError) {
        return (
            <TableRow className="bg-muted/20">
                <TableCell colSpan={5} className="py-2 pl-10 text-xs text-destructive">
                    來源單據載入失敗，請稍後重試
                </TableCell>
            </TableRow>
        )
    }
    if (!data || data.length === 0) {
        return (
            <TableRow className="bg-muted/20">
                <TableCell colSpan={5} className="py-2 pl-10 text-xs text-muted-foreground">
                    查無可追溯的來源採購入庫單（可能為早期匯入 / 調整，來源不明）
                </TableCell>
            </TableRow>
        )
    }
    return (
        <>
            {data.map((s) => (
                <TableRow key={`${s.document_id}-${s.line_no}`} className="bg-muted/20 text-xs">
                    <TableCell colSpan={2} className="pl-10">
                        <span className="font-mono font-medium text-primary">{s.doc_no}</span>
                        <span className="text-muted-foreground ml-2">
                            第 {s.line_no} 行 · {s.doc_date}
                        </span>
                        {s.partner_name && <span className="text-muted-foreground ml-2">· {s.partner_name}</span>}
                    </TableCell>
                    <TableCell colSpan={2} className="text-right text-muted-foreground">
                        {s.batch_no ? `批號 ${s.batch_no}` : ''}
                    </TableCell>
                    <TableCell className="text-right font-medium text-status-warning-text">
                        {parseFloat(s.remaining_unshelved).toLocaleString()} {formatUom(baseUom)}
                    </TableCell>
                </TableRow>
            ))}
        </>
    )
}
