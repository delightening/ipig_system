import { useParams, useNavigate } from 'react-router-dom'
import { useMutation, useQuery } from '@tanstack/react-query'

import api, { WarehouseReportData, StorageLocationWithInventory } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { ToastAction } from '@/components/ui/toast'
import { Loader2, Printer, Download, ArrowLeft } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { uiLocale } from '@/lib/utils'

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

    /** 取倉庫顯示名稱（name 優先，空則 fallback code），用於檔名 + 預覽分頁標題 */
    const fileLabel = (report?.warehouse.name?.trim() || report?.warehouse.code || 'warehouse')
    /** 檔名固定後綴 — 與 backend `_倉庫現況報表.pdf` 對齊 */
    const REPORT_FILENAME_SUFFIX = '_倉庫現況報表.pdf'

    /**
     * fetch backend PDF blob → URL.createObjectURL。`responseType: 'blob'` 後
     * `res.data` 已是 Blob，無需再包一層 `new Blob([res.data])`（Gemini PR #344 review）
     */
    const fetchPdfBlob = async (inline = false): Promise<string> => {
        const url = `/warehouses/${warehouseId}/report/pdf${inline ? '?inline=1' : ''}`
        const res = await api.get<Blob>(url, { responseType: 'blob' })
        return window.URL.createObjectURL(res.data)
    }

    /**
     * 列印：以 PDF 為準（取代既有 window.print() 走 React print stylesheet 的路徑）。
     * fetch PDF blob → 開新 window → 瀏覽器 PDF viewer 用 Ctrl+P 列印。
     *
     * 記憶體管理（Gemini + CodeRabbit PR #344 review）：
     * - 彈窗被阻擋（`win === null`）→ 立即 `revokeObjectURL` 後拋錯（避免確定性洩漏）
     * - 成功路徑 → 用 setTimeout 延遲 revoke，給新分頁載入 PDF 一些緩衝時間；
     *   即便使用者直接關分頁，60s 後仍會釋放 blob ref（瀏覽器層 GC 不保證即時）
     */
    const printPdfMutation = useMutation({
        mutationFn: async () => {
            const blobUrl = await fetchPdfBlob(true)
            const win = window.open(blobUrl, '_blank')
            if (!win) {
                window.URL.revokeObjectURL(blobUrl)
                throw new Error('彈出視窗被阻擋；請允許後重試')
            }
            // R35-4: PDF 分頁標題從 blob: → 倉庫名稱
            const pdfTitle = `${fileLabel}${REPORT_FILENAME_SUFFIX}`
            win.addEventListener('load', () => {
                try { win.document.title = pdfTitle } catch { /* cross-origin or PDF viewer override */ }
            }, { once: true })
            // 60s 後釋放 blob URL — 足夠新分頁載入 PDF（一般 < 5s）
            setTimeout(() => window.URL.revokeObjectURL(blobUrl), 60_000)
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '開啟列印 PDF 失敗'),
                variant: 'destructive',
                action: (
                    <ToastAction altText="重試列印" onClick={() => printPdfMutation.mutate()}>
                        重試
                    </ToastAction>
                ),
            })
        },
    })

    const downloadPdfMutation = useMutation({
        mutationFn: async () => {
            const blobUrl = await fetchPdfBlob()
            try {
                const link = document.createElement('a')
                link.href = blobUrl
                link.setAttribute('download', `${fileLabel}${REPORT_FILENAME_SUFFIX}`)
                document.body.appendChild(link)
                link.click()
                link.remove()
            } finally {
                // 確保 link.click() 失敗時也釋放 blob URL
                window.URL.revokeObjectURL(blobUrl)
            }
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, 'PDF 下載失敗'),
                variant: 'destructive',
                action: (
                    <ToastAction altText="重試下載" onClick={() => downloadPdfMutation.mutate()}>
                        重試
                    </ToastAction>
                ),
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
                <Button
                    variant="outline"
                    size="sm"
                    onClick={() => printPdfMutation.mutate()}
                    disabled={printPdfMutation.isPending}
                    title="開啟 PDF 預覽（在 PDF viewer 內 Ctrl+P 列印）"
                >
                    {printPdfMutation.isPending
                        ? <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        : <Printer className="mr-2 h-4 w-4" />}
                    {printPdfMutation.isPending ? '載入中…' : '列印'}
                </Button>
                <Button
                    variant="outline"
                    size="sm"
                    onClick={() => downloadPdfMutation.mutate()}
                    disabled={downloadPdfMutation.isPending}
                >
                    {downloadPdfMutation.isPending
                        ? <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        : <Download className="mr-2 h-4 w-4" />}
                    {downloadPdfMutation.isPending ? '下載中…' : '下載 PDF'}
                </Button>
            </div>

            {/* 標題 */}
            <div className="text-center mb-6">
                <h1 className="text-2xl font-bold print:text-xl">倉庫現況報表</h1>
                <p className="text-sm text-muted-foreground mt-1">
                    產出時間：{new Date(report.generated_at).toLocaleString(uiLocale(), { timeZone: 'Asia/Taipei' })}
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

            {/* 摘要統計 — R35-3 (redo on R35-16): 5 卡，新增「庫存價值」(SUM(qty × selling_price)，缺價產品不計入) */}
            <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3 mb-6">
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
                <SummaryCard label="庫存價值" value={formatInventoryValue(summary.total_inventory_value)} />
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

/** R35-3 (redo on R35-16)：庫存價值字串化（後端 Decimal → 字串避免 JS 浮點誤差），千分位 + NTD$ 前綴
 *
 * 不可用 Number(raw) — 大金額（>2^53）或高精度 Decimal 轉回 JS number 會失真，抵消後端 Decimal 設計。
 * 純字串解析整數段 + 千分位插入；小數段直接四捨五入（檢查首個小數 >=5 → 整數段 +1）。 */
function formatInventoryValue(raw: string | number | null | undefined): string {
    if (raw === null || raw === undefined || raw === '') return 'NT$ 0'

    // 統一成 string；數字型別 toString 不會有精度誤差（直接 IEEE-754 → 字串）
    const s = String(raw).trim()
    if (s === '') return 'NT$ 0'

    // 接受 [-]?digits[.digits]?；其他歸為「無效」
    const match = /^(-?)(\d+)(?:\.(\d+))?$/.exec(s)
    if (!match) return 'NT$ -'

    const [, sign, intPart, fracPart] = match

    // 進位：首個小數位 >= 5 → 整數段 +1（純字串加法，避免 BigInt 缺失環境）
    let carriedInt = intPart
    if (fracPart && fracPart[0] >= '5') {
        carriedInt = stringIntAddOne(intPart)
    }

    // 千分位
    const grouped = carriedInt.replace(/\B(?=(\d{3})+(?!\d))/g, ',')
    return `NT$ ${sign}${grouped}`
}

/** 純字串十進位整數 +1（避免大數轉 number 失真） */
function stringIntAddOne(s: string): string {
    const arr = s.split('')
    let i = arr.length - 1
    while (i >= 0) {
        if (arr[i] === '9') {
            arr[i] = '0'
            i -= 1
        } else {
            arr[i] = String(Number(arr[i]) + 1)
            return arr.join('')
        }
    }
    return '1' + arr.join('')
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
                            title={buildLocationTooltip(loc)}
                        >
                            {loc.name || loc.code}
                        </div>
                    )
                })}
            </div>
        </div>
    )
}

/** R35-2: 平面圖 hover tooltip — 顯示前 5 項庫存 + 總品項數，列印時瀏覽器自動隱藏 title */
function buildLocationTooltip(loc: StorageLocationWithInventory): string {
    const head = `${loc.code}${loc.name ? ` - ${loc.name}` : ''}` +
        ` (${loc.current_count}${loc.capacity && loc.capacity > 0 ? `/${loc.capacity}` : ''})`
    if (loc.inventory.length === 0) return head
    const top = loc.inventory.slice(0, 5).map(it => {
        const qty = Math.floor(Number(it.on_hand_qty))
        return `${it.product_name} ×${qty}${it.base_uom}`
    })
    const more = loc.inventory.length > 5 ? `\n…等共 ${loc.inventory.length} 項` : ''
    return `${head}\n${top.join('\n')}${more}`
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
