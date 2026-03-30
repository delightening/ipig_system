import { useParams, useNavigate } from 'react-router-dom'
import { useMutation, useQuery } from '@tanstack/react-query'

import api, { WarehouseReportData, StorageLocationWithInventory } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Loader2, Printer, Download, ArrowLeft } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

const STRUCTURE_TYPES = ['wall', 'door', 'window']

export function WarehouseReportPage() {
    const { warehouseId } = useParams<{ warehouseId: string }>()
    const navigate = useNavigate()

    const { data: report, isLoading } = useQuery({
        queryKey: ['warehouse-report', warehouseId],
        queryFn: async () => {
            const res = await api.get<WarehouseReportData>(`/warehouses/${warehouseId}/report`)
            return res.data
        },
        enabled: !!warehouseId,
    })

    const handlePrint = () => {
        window.print()
    }

    const downloadPdfMutation = useMutation({
        mutationFn: async () => {
            const res = await api.get(`/warehouses/${warehouseId}/report/pdf`, {
                responseType: 'blob',
            })
            const url = window.URL.createObjectURL(new Blob([res.data]))
            const link = document.createElement('a')
            link.href = url
            link.setAttribute('download', `${report?.warehouse.code ?? 'warehouse'}_倉庫現況報表.pdf`)
            document.body.appendChild(link)
            link.click()
            link.remove()
            window.URL.revokeObjectURL(url)
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, 'PDF 下載失敗'),
                variant: 'destructive',
            })
        },
    })

    if (isLoading) {
        return (
            <div className="flex items-center justify-center min-h-screen">
                <Loader2 className="h-8 w-8 animate-spin" />
            </div>
        )
    }

    if (!report) {
        return (
            <div className="flex items-center justify-center min-h-screen">
                <p className="text-muted-foreground">查無報表資料</p>
            </div>
        )
    }

    const { warehouse, summary, locations } = report

    return (
        <div className="max-w-[900px] mx-auto p-6 print:p-2 print:max-w-none">
            {/* 操作列 - 列印時隱藏 */}
            <div className="flex gap-2 mb-6 print:hidden">
                <Button variant="ghost" size="sm" onClick={() => navigate(-1)}>
                    <ArrowLeft className="mr-2 h-4 w-4" />
                    返回
                </Button>
                <Button variant="outline" size="sm" onClick={handlePrint}>
                    <Printer className="mr-2 h-4 w-4" />
                    列印
                </Button>
                <Button variant="outline" size="sm" onClick={() => downloadPdfMutation.mutate()} disabled={downloadPdfMutation.isPending}>
                    <Download className="mr-2 h-4 w-4" />
                    下載 PDF
                </Button>
            </div>

            {/* 標題 */}
            <div className="text-center mb-6">
                <h1 className="text-2xl font-bold print:text-xl">倉庫現況報表</h1>
                <p className="text-sm text-muted-foreground mt-1">
                    產出時間：{new Date(report.generated_at).toLocaleString('zh-TW')}
                </p>
            </div>

            {/* 倉庫基本資訊 */}
            <Card className="mb-4 print:border print:shadow-none">
                <CardHeader className="pb-2">
                    <CardTitle className="text-base">倉庫資訊</CardTitle>
                </CardHeader>
                <CardContent className="grid grid-cols-2 gap-2 text-sm">
                    <div><span className="text-muted-foreground">代碼：</span>{warehouse.code}</div>
                    <div><span className="text-muted-foreground">名稱：</span>{warehouse.name}</div>
                    {warehouse.address && (
                        <div className="col-span-2">
                            <span className="text-muted-foreground">地址：</span>{warehouse.address}
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* 摘要統計 */}
            <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
                <SummaryCard label="儲位總數" value={summary.total_locations} />
                <SummaryCard label="使用中" value={summary.active_locations} />
                <SummaryCard
                    label="容量使用"
                    value={
                        summary.total_capacity > 0
                            ? `${summary.total_current_count}/${summary.total_capacity}`
                            : `${summary.total_current_count}`
                    }
                />
                <SummaryCard label="庫存品項" value={summary.total_inventory_items} />
            </div>

            {/* 佈局圖 */}
            {locations.length > 0 && <LayoutDiagram locations={locations} />}

            {/* 庫存明細 - 列印時強制換頁 */}
            <div className="mt-6" style={{ pageBreakBefore: 'always' }}>
                <h2 className="text-lg font-semibold mb-3">各儲位庫存明細</h2>
                {locations
                    .filter(l => !STRUCTURE_TYPES.includes(l.location_type))
                    .map(loc => (
                        <LocationInventoryTable key={loc.id} location={loc} />
                    ))}
            </div>
        </div>
    )
}

function SummaryCard({ label, value }: { label: string; value: string | number }) {
    return (
        <Card className="print:border print:shadow-none">
            <CardContent className="p-3 text-center">
                <div className="text-xs text-muted-foreground">{label}</div>
                <div className="text-xl font-bold mt-1">{value}</div>
            </CardContent>
        </Card>
    )
}

function LayoutDiagram({ locations }: { locations: StorageLocationWithInventory[] }) {
    const maxCol = Math.max(...locations.map(l => l.col_index + l.width), 1)
    const maxRow = Math.max(...locations.map(l => l.row_index + l.height), 1)

    return (
        <div className="print:break-inside-avoid">
            <h2 className="text-lg font-semibold mb-3">儲位佈局圖</h2>
            <div
                className="relative border rounded bg-muted"
                style={{
                    width: '100%',
                    aspectRatio: `${maxCol} / ${maxRow}`,
                    maxHeight: '400px',
                }}
            >
                {locations.map(loc => {
                    const isStructure = STRUCTURE_TYPES.includes(loc.location_type)
                    return (
                        <div
                            key={loc.id}
                            className="absolute flex items-center justify-center text-white text-xs font-medium rounded-sm overflow-hidden print:!bg-white print:!text-black print:!border-black print:!border"
                            style={{
                                left: `${(loc.col_index / maxCol) * 100}%`,
                                top: `${(loc.row_index / maxRow) * 100}%`,
                                width: `${(loc.width / maxCol) * 100}%`,
                                height: `${(loc.height / maxRow) * 100}%`,
                                backgroundColor: isStructure
                                    ? getStructureColor(loc.location_type)
                                    : (loc.color || '#3b82f6'),
                                border: '1px solid rgba(255,255,255,0.3)',
                            }}
                            title={`${loc.code}${loc.name ? ` - ${loc.name}` : ''} (${loc.current_count}${loc.capacity && loc.capacity > 0 ? `/${loc.capacity}` : ''})`}
                        >
                            {loc.name || loc.code}
                        </div>
                    )
                })}
            </div>
        </div>
    )
}

function getStructureColor(type: string): string {
    switch (type) {
        case 'wall': return '#999999'
        case 'door': return '#8B5A2B'
        case 'window': return '#B3D9EC'
        default: return '#666666'
    }
}

function LocationInventoryTable({ location }: { location: StorageLocationWithInventory }) {
    const title = location.name
        ? `【${location.code}】${location.name}`
        : `【${location.code}】`

    const capacityInfo = location.capacity && location.capacity > 0
        ? `${location.current_count}/${location.capacity}`
        : `${location.current_count}`

    return (
        <div className="mb-4 print:break-inside-avoid">
            <div className="flex items-baseline gap-2 mb-1">
                <h3 className="text-sm font-semibold">{title}</h3>
                <span className="text-xs text-muted-foreground">（{capacityInfo}）</span>
            </div>
            {location.inventory.length === 0 ? (
                <p className="text-xs text-muted-foreground pl-2 mb-2">（無庫存）</p>
            ) : (
                <table className="w-full text-xs border-collapse mb-2">
                    <thead>
                        <tr className="bg-muted print:bg-muted">
                            <th className="text-left p-1 border">產品名稱</th>
                            <th className="text-left p-1 border">SKU</th>
                            <th className="text-right p-1 border">數量</th>
                            <th className="text-left p-1 border">單位</th>
                            <th className="text-left p-1 border">批號</th>
                            <th className="text-left p-1 border">效期</th>
                        </tr>
                    </thead>
                    <tbody>
                        {location.inventory.map(item => (
                            <tr key={item.id}>
                                <td className="p-1 border">{item.product_name}</td>
                                <td className="p-1 border">{item.product_sku}</td>
                                <td className="p-1 border text-right">{Math.floor(Number(item.on_hand_qty))}</td>
                                <td className="p-1 border">{item.base_uom}</td>
                                <td className="p-1 border">{item.batch_no || '-'}</td>
                                <td className="p-1 border">{item.expiry_date || '-'}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}
        </div>
    )
}
