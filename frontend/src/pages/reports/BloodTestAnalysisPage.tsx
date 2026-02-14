/**
 * 血液檢查結果分析頁面
 * 提供血液檢查數據的統計、趨勢分析、異常標記與視覺化圖表
 */
import { useState, useMemo, useCallback } from 'react'
import { useQuery } from '@tanstack/react-query'
import { bloodTestAnalysisApi } from '@/lib/api'
import type { BloodTestAnalysisRow } from '@/types'
import { formatDate } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import {
    Loader2,
    Download,
    AlertTriangle,
    Activity,
    Users,
    FlaskConical,
    TrendingUp,
    BarChart3,
    FileSpreadsheet,
    ArrowLeft,
} from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    Legend,
    ResponsiveContainer,
} from 'recharts'
import * as XLSX from 'xlsx'

// 圖表顏色調色盤
const CHART_COLORS = [
    '#6366f1', '#ec4899', '#14b8a6', '#f59e0b', '#8b5cf6',
    '#ef4444', '#06b6d4', '#84cc16', '#f97316', '#a855f7',
]

// ============================================
// 盒鬚圖自訂元件（Recharts 沒有內建）
// ============================================
interface BoxPlotData {
    name: string
    min: number
    q1: number
    median: number
    q3: number
    max: number
    unit?: string
    count: number
}

function BoxPlotChart({ data }: { data: BoxPlotData[] }) {
    if (data.length === 0) return null

    const maxVal = Math.max(...data.map(d => d.max))
    const chartHeight = Math.max(200, data.length * 60)

    return (
        <div className="w-full overflow-x-auto">
            <svg width="100%" height={chartHeight} viewBox={`0 0 700 ${chartHeight}`} className="text-sm">
                {data.map((item, idx) => {
                    const y = idx * 56 + 30
                    const barHeight = 24
                    const labelX = 0
                    const plotStart = 180
                    const plotWidth = 480
                    const scale = (v: number) => plotStart + (v / (maxVal * 1.1)) * plotWidth

                    return (
                        <g key={item.name}>
                            {/* 項目名稱 */}
                            <text x={labelX} y={y + barHeight / 2 + 4} className="fill-current text-xs" fontSize="11">
                                {item.name} {item.unit ? `(${item.unit})` : ''} n={item.count}
                            </text>
                            {/* 最小到 Q1 的線 */}
                            <line x1={scale(item.min)} y1={y + barHeight / 2} x2={scale(item.q1)} y2={y + barHeight / 2}
                                stroke="currentColor" strokeWidth="1" className="text-muted-foreground" />
                            {/* 最小值端線 */}
                            <line x1={scale(item.min)} y1={y + 4} x2={scale(item.min)} y2={y + barHeight - 4}
                                stroke="currentColor" strokeWidth="1" className="text-muted-foreground" />
                            {/* Q1 到 Q3 的方框 */}
                            <rect x={scale(item.q1)} y={y} width={scale(item.q3) - scale(item.q1)} height={barHeight}
                                fill="hsl(var(--primary) / 0.15)" stroke="hsl(var(--primary))" strokeWidth="1.5" rx="3" />
                            {/* 中位數線 */}
                            <line x1={scale(item.median)} y1={y} x2={scale(item.median)} y2={y + barHeight}
                                stroke="hsl(var(--primary))" strokeWidth="2.5" />
                            {/* Q3 到最大值的線 */}
                            <line x1={scale(item.q3)} y1={y + barHeight / 2} x2={scale(item.max)} y2={y + barHeight / 2}
                                stroke="currentColor" strokeWidth="1" className="text-muted-foreground" />
                            {/* 最大值端線 */}
                            <line x1={scale(item.max)} y1={y + 4} x2={scale(item.max)} y2={y + barHeight - 4}
                                stroke="currentColor" strokeWidth="1" className="text-muted-foreground" />
                        </g>
                    )
                })}
            </svg>
        </div>
    )
}

// ============================================
// 統計計算工具
// ============================================
function calcBoxPlot(values: number[]): { min: number; q1: number; median: number; q3: number; max: number } | null {
    if (values.length === 0) return null
    const sorted = [...values].sort((a, b) => a - b)
    const n = sorted.length
    const q1Idx = Math.floor(n * 0.25)
    const medIdx = Math.floor(n * 0.5)
    const q3Idx = Math.floor(n * 0.75)
    return {
        min: sorted[0],
        q1: sorted[q1Idx],
        median: n % 2 === 0 ? (sorted[medIdx - 1] + sorted[medIdx]) / 2 : sorted[medIdx],
        q3: sorted[q3Idx],
        max: sorted[n - 1],
    }
}

// ============================================
// 主頁面元件
// ============================================
export function BloodTestAnalysisPage() {
    const navigate = useNavigate()

    // 篩選狀態
    const [iacucNo, setIacucNo] = useState('')
    const [earTag, setEarTag] = useState('')
    const [dateFrom, setDateFrom] = useState('')
    const [dateTo, setDateTo] = useState('')
    const [selectedItems, setSelectedItems] = useState<string[]>([])
    const [activeTab, setActiveTab] = useState<'trend' | 'boxplot' | 'table'>('trend')

    // 建立查詢參數
    const queryParams = useMemo(() => {
        const params = new URLSearchParams()
        if (iacucNo.trim()) params.set('iacuc_no', iacucNo.trim())
        if (dateFrom) params.set('date_from', dateFrom)
        if (dateTo) params.set('date_to', dateTo)
        return params.toString()
    }, [iacucNo, dateFrom, dateTo])

    // 取得分析資料
    const { data: rawData, isLoading } = useQuery<BloodTestAnalysisRow[]>({
        queryKey: ['blood-test-analysis', queryParams],
        queryFn: async () => {
            const response = await bloodTestAnalysisApi.query(queryParams)
            return response.data
        },
    })

    // 篩選耳號（前端篩選，因為 API 使用 pig_id 不方便使用者輸入）
    const filteredData = useMemo(() => {
        if (!rawData) return []
        let data = rawData
        if (earTag.trim()) {
            const search = earTag.trim().toLowerCase()
            data = data.filter(r => r.ear_tag.toLowerCase().includes(search))
        }
        return data
    }, [rawData, earTag])

    // 取得所有不重複的檢查項目名稱
    const availableItems = useMemo(() => {
        const items = new Set<string>()
        filteredData.forEach(r => items.add(r.item_name))
        return Array.from(items).sort()
    }, [filteredData])

    // 根據選中的項目篩選資料（圖表用）
    const chartFilteredData = useMemo(() => {
        if (selectedItems.length === 0) return filteredData
        return filteredData.filter(r => selectedItems.includes(r.item_name))
    }, [filteredData, selectedItems])

    // ============================================
    // 摘要統計
    // ============================================
    const summary = useMemo(() => {
        if (filteredData.length === 0) return { totalItems: 0, abnormalCount: 0, abnormalRate: 0, animalCount: 0, testDates: 0 }

        const abnormal = filteredData.filter(r => r.is_abnormal)
        const animals = new Set(filteredData.map(r => r.pig_id))
        const dates = new Set(filteredData.map(r => `${r.pig_id}_${r.test_date}`))

        return {
            totalItems: filteredData.length,
            abnormalCount: abnormal.length,
            abnormalRate: (abnormal.length / filteredData.length) * 100,
            animalCount: animals.size,
            testDates: dates.size,
        }
    }, [filteredData])

    // ============================================
    // 異常值紀錄
    // ============================================
    const abnormalRecords = useMemo(() => {
        return filteredData.filter(r => r.is_abnormal)
    }, [filteredData])

    // ============================================
    // 折線圖資料：依日期為 X 軸，各動物為不同線條
    // ============================================
    const trendData = useMemo(() => {
        if (chartFilteredData.length === 0) return { chartData: [], animals: [] as string[] }

        // 取得所有動物列表
        const animalSet = new Set<string>()
        chartFilteredData.forEach(r => animalSet.add(r.ear_tag))
        const animals = Array.from(animalSet).sort()

        // 按日期聚合
        const dateMap = new Map<string, Record<string, number | string>>()
        chartFilteredData.forEach(r => {
            const val = r.result_value ? parseFloat(r.result_value) : NaN
            if (isNaN(val)) return

            const key = r.test_date
            if (!dateMap.has(key)) {
                dateMap.set(key, { date: formatDate(key) })
            }
            const entry = dateMap.get(key)!
            entry[r.ear_tag] = val
        })

        const chartData = Array.from(dateMap.values()).sort((a, b) =>
            String(a.date).localeCompare(String(b.date))
        )

        return { chartData, animals }
    }, [chartFilteredData])

    // ============================================
    // 盒鬚圖資料：依檢查項目分組
    // ============================================
    const boxPlotData = useMemo((): BoxPlotData[] => {
        if (filteredData.length === 0) return []

        const itemMap = new Map<string, { values: number[]; unit?: string }>()
        filteredData.forEach(r => {
            const val = r.result_value ? parseFloat(r.result_value) : NaN
            if (isNaN(val)) return

            if (!itemMap.has(r.item_name)) {
                itemMap.set(r.item_name, { values: [], unit: r.result_unit || undefined })
            }
            itemMap.get(r.item_name)!.values.push(val)
        })

        const result: BoxPlotData[] = []
        // 只取選中的項目，若未選中則取全部
        const targetItems = selectedItems.length > 0
            ? availableItems.filter(name => selectedItems.includes(name))
            : availableItems

        targetItems.forEach(name => {
            const info = itemMap.get(name)
            if (!info || info.values.length < 2) return
            const bp = calcBoxPlot(info.values)
            if (!bp) return
            result.push({
                name,
                ...bp,
                unit: info.unit,
                count: info.values.length,
            })
        })

        return result
    }, [filteredData, selectedItems, availableItems])

    // ============================================
    // 匯出功能
    // ============================================
    const exportToCSV = useCallback(() => {
        if (!filteredData.length) return

        const headers = ['專案編號', '耳號', '檢查日期', '實驗室', '項目名稱', '項目代碼', '結果值', '單位', '參考範圍', '是否異常']
        const rows = filteredData.map(r => [
            r.iacuc_no || '',
            r.ear_tag,
            r.test_date,
            r.lab_name || '',
            r.item_name,
            r.template_code || '',
            r.result_value || '',
            r.result_unit || '',
            r.reference_range || '',
            r.is_abnormal ? '是' : '否',
        ])

        const csvContent = [headers, ...rows]
            .map(row => row.map(cell => `"${cell}"`).join(','))
            .join('\n')

        const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
        const link = document.createElement('a')
        link.href = URL.createObjectURL(blob)
        link.download = `blood_test_analysis_${new Date().toISOString().split('T')[0]}.csv`
        link.click()
    }, [filteredData])

    const exportToExcel = useCallback(() => {
        if (!filteredData.length) return

        const wsData = [
            ['專案編號', '耳號', '檢查日期', '實驗室', '項目名稱', '項目代碼', '結果值', '單位', '參考範圍', '是否異常'],
            ...filteredData.map(r => [
                r.iacuc_no || '',
                r.ear_tag,
                r.test_date,
                r.lab_name || '',
                r.item_name,
                r.template_code || '',
                r.result_value || '',
                r.result_unit || '',
                r.reference_range || '',
                r.is_abnormal ? '是' : '否',
            ]),
        ]

        const wb = XLSX.utils.book_new()
        const ws = XLSX.utils.aoa_to_sheet(wsData)

        // 設定欄位寬度
        ws['!cols'] = [
            { wch: 18 }, { wch: 10 }, { wch: 12 }, { wch: 15 },
            { wch: 20 }, { wch: 10 }, { wch: 12 }, { wch: 8 },
            { wch: 15 }, { wch: 8 },
        ]

        XLSX.utils.book_append_sheet(wb, ws, '血液檢查分析')
        XLSX.writeFile(wb, `blood_test_analysis_${new Date().toISOString().split('T')[0]}.xlsx`)
    }, [filteredData])

    // 切換選中項目
    const toggleItem = (item: string) => {
        setSelectedItems(prev =>
            prev.includes(item) ? prev.filter(i => i !== item) : [...prev, item]
        )
    }

    return (
        <div className="space-y-6">
            {/* 標題列 */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                    <Button variant="ghost" size="icon" onClick={() => navigate('/reports')}>
                        <ArrowLeft className="h-5 w-5" />
                    </Button>
                    <div>
                        <h1 className="text-3xl font-bold tracking-tight">血液檢查結果分析</h1>
                        <p className="text-muted-foreground">對血液檢查結果進行統計分析、趨勢追蹤與異常值偵測</p>
                    </div>
                </div>
                <div className="flex gap-2">
                    <Button variant="outline" onClick={exportToCSV} disabled={!filteredData.length}>
                        <Download className="mr-2 h-4 w-4" />
                        CSV
                    </Button>
                    <Button onClick={exportToExcel} disabled={!filteredData.length}>
                        <FileSpreadsheet className="mr-2 h-4 w-4" />
                        Excel
                    </Button>
                </div>
            </div>

            {/* 篩選區 */}
            <Card>
                <CardHeader>
                    <CardTitle className="text-base">篩選條件</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                        <div className="space-y-2">
                            <Label htmlFor="iacuc_no">專案編號 (IACUC No.)</Label>
                            <Input
                                id="iacuc_no"
                                placeholder="例: IACUC-2024-001"
                                value={iacucNo}
                                onChange={(e) => setIacucNo(e.target.value)}
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="ear_tag">動物耳號</Label>
                            <Input
                                id="ear_tag"
                                placeholder="輸入耳號搜尋"
                                value={earTag}
                                onChange={(e) => setEarTag(e.target.value)}
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="date_from">開始日期</Label>
                            <Input
                                id="date_from"
                                type="date"
                                value={dateFrom}
                                onChange={(e) => setDateFrom(e.target.value)}
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="date_to">結束日期</Label>
                            <Input
                                id="date_to"
                                type="date"
                                value={dateTo}
                                onChange={(e) => setDateTo(e.target.value)}
                            />
                        </div>
                    </div>
                </CardContent>
            </Card>

            {/* 載入中 */}
            {isLoading && (
                <div className="flex items-center justify-center py-12">
                    <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                </div>
            )}

            {!isLoading && filteredData.length > 0 && (
                <>
                    {/* 摘要統計卡片 */}
                    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                        <Card>
                            <CardContent className="flex items-center gap-4 pt-6">
                                <div className="rounded-full bg-blue-100 p-3 dark:bg-blue-900/30">
                                    <FlaskConical className="h-6 w-6 text-blue-600 dark:text-blue-400" />
                                </div>
                                <div>
                                    <p className="text-sm text-muted-foreground">檢查項目數</p>
                                    <p className="text-2xl font-bold">{summary.totalItems.toLocaleString()}</p>
                                </div>
                            </CardContent>
                        </Card>
                        <Card>
                            <CardContent className="flex items-center gap-4 pt-6">
                                <div className="rounded-full bg-red-100 p-3 dark:bg-red-900/30">
                                    <AlertTriangle className="h-6 w-6 text-red-600 dark:text-red-400" />
                                </div>
                                <div>
                                    <p className="text-sm text-muted-foreground">異常比率</p>
                                    <p className="text-2xl font-bold">
                                        {summary.abnormalRate.toFixed(1)}%
                                        <span className="text-sm font-normal text-muted-foreground ml-1">
                                            ({summary.abnormalCount})
                                        </span>
                                    </p>
                                </div>
                            </CardContent>
                        </Card>
                        <Card>
                            <CardContent className="flex items-center gap-4 pt-6">
                                <div className="rounded-full bg-green-100 p-3 dark:bg-green-900/30">
                                    <Users className="h-6 w-6 text-green-600 dark:text-green-400" />
                                </div>
                                <div>
                                    <p className="text-sm text-muted-foreground">涵蓋動物數</p>
                                    <p className="text-2xl font-bold">{summary.animalCount}</p>
                                </div>
                            </CardContent>
                        </Card>
                        <Card>
                            <CardContent className="flex items-center gap-4 pt-6">
                                <div className="rounded-full bg-purple-100 p-3 dark:bg-purple-900/30">
                                    <Activity className="h-6 w-6 text-purple-600 dark:text-purple-400" />
                                </div>
                                <div>
                                    <p className="text-sm text-muted-foreground">檢查次數</p>
                                    <p className="text-2xl font-bold">{summary.testDates}</p>
                                </div>
                            </CardContent>
                        </Card>
                    </div>

                    {/* 異常警示區 */}
                    {abnormalRecords.length > 0 && (
                        <Card className="border-red-200 dark:border-red-800">
                            <CardHeader>
                                <CardTitle className="text-base flex items-center gap-2 text-red-600 dark:text-red-400">
                                    <AlertTriangle className="h-5 w-5" />
                                    異常值警示（共 {abnormalRecords.length} 項）
                                </CardTitle>
                            </CardHeader>
                            <CardContent>
                                <div className="rounded-md border border-red-200 dark:border-red-800">
                                    <Table>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead>耳號</TableHead>
                                                <TableHead>專案</TableHead>
                                                <TableHead>日期</TableHead>
                                                <TableHead>項目</TableHead>
                                                <TableHead className="text-right">結果值</TableHead>
                                                <TableHead>參考範圍</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {abnormalRecords.slice(0, 20).map((r, idx) => (
                                                <TableRow key={idx} className="bg-red-50/50 dark:bg-red-950/20">
                                                    <TableCell className="font-medium">{r.ear_tag}</TableCell>
                                                    <TableCell className="font-mono text-sm">{r.iacuc_no || '-'}</TableCell>
                                                    <TableCell>{formatDate(r.test_date)}</TableCell>
                                                    <TableCell>{r.item_name}</TableCell>
                                                    <TableCell className="text-right font-semibold text-red-600 dark:text-red-400">
                                                        {r.result_value} {r.result_unit}
                                                    </TableCell>
                                                    <TableCell className="text-muted-foreground">{r.reference_range || '-'}</TableCell>
                                                </TableRow>
                                            ))}
                                        </TableBody>
                                    </Table>
                                    {abnormalRecords.length > 20 && (
                                        <p className="text-sm text-muted-foreground text-center py-2">
                                            僅顯示前 20 筆，匯出報表可查看完整異常清單
                                        </p>
                                    )}
                                </div>
                            </CardContent>
                        </Card>
                    )}

                    {/* 檢查項目選擇器 */}
                    {availableItems.length > 0 && (
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-base">選擇分析項目</CardTitle>
                            </CardHeader>
                            <CardContent>
                                <div className="flex flex-wrap gap-2">
                                    {availableItems.map(item => (
                                        <Button
                                            key={item}
                                            variant={selectedItems.includes(item) ? 'default' : 'outline'}
                                            size="sm"
                                            onClick={() => toggleItem(item)}
                                            className="text-xs"
                                        >
                                            {item}
                                        </Button>
                                    ))}
                                    {selectedItems.length > 0 && (
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            onClick={() => setSelectedItems([])}
                                            className="text-xs text-muted-foreground"
                                        >
                                            清除選擇
                                        </Button>
                                    )}
                                </div>
                                {selectedItems.length === 0 && (
                                    <p className="text-sm text-muted-foreground mt-2">
                                        點選項目可篩選圖表顯示範圍，未選擇時顯示全部
                                    </p>
                                )}
                            </CardContent>
                        </Card>
                    )}

                    {/* 圖表切換標籤 */}
                    <div className="flex gap-2 border-b pb-2">
                        <Button
                            variant={activeTab === 'trend' ? 'default' : 'ghost'}
                            size="sm"
                            onClick={() => setActiveTab('trend')}
                        >
                            <TrendingUp className="mr-2 h-4 w-4" />
                            趨勢圖
                        </Button>
                        <Button
                            variant={activeTab === 'boxplot' ? 'default' : 'ghost'}
                            size="sm"
                            onClick={() => setActiveTab('boxplot')}
                        >
                            <BarChart3 className="mr-2 h-4 w-4" />
                            盒鬚圖
                        </Button>
                        <Button
                            variant={activeTab === 'table' ? 'default' : 'ghost'}
                            size="sm"
                            onClick={() => setActiveTab('table')}
                        >
                            <FlaskConical className="mr-2 h-4 w-4" />
                            資料明細
                        </Button>
                    </div>

                    {/* 折線圖（趨勢分析） */}
                    {activeTab === 'trend' && (
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-base">
                                    趨勢分析
                                    {selectedItems.length > 0 && (
                                        <span className="text-muted-foreground font-normal ml-2 text-sm">
                                            ({selectedItems.join(', ')})
                                        </span>
                                    )}
                                </CardTitle>
                            </CardHeader>
                            <CardContent>
                                {trendData.chartData.length > 0 ? (
                                    <ResponsiveContainer width="100%" height={400}>
                                        <LineChart data={trendData.chartData}>
                                            <CartesianGrid strokeDasharray="3 3" className="opacity-30" />
                                            <XAxis dataKey="date" fontSize={12} />
                                            <YAxis fontSize={12} />
                                            <Tooltip
                                                contentStyle={{
                                                    backgroundColor: 'hsl(var(--card))',
                                                    border: '1px solid hsl(var(--border))',
                                                    borderRadius: '8px',
                                                    fontSize: '12px',
                                                }}
                                            />
                                            <Legend />
                                            {trendData.animals.map((animal, idx) => (
                                                <Line
                                                    key={animal}
                                                    type="monotone"
                                                    dataKey={animal}
                                                    name={animal}
                                                    stroke={CHART_COLORS[idx % CHART_COLORS.length]}
                                                    strokeWidth={2}
                                                    dot={{ r: 4 }}
                                                    activeDot={{ r: 6 }}
                                                    connectNulls
                                                />
                                            ))}
                                        </LineChart>
                                    </ResponsiveContainer>
                                ) : (
                                    <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                                        <TrendingUp className="h-12 w-12 mb-2" />
                                        <p>請選擇具有數值結果的檢查項目以顯示趨勢圖</p>
                                    </div>
                                )}
                            </CardContent>
                        </Card>
                    )}

                    {/* 盒鬚圖（數值分布） */}
                    {activeTab === 'boxplot' && (
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-base">數值分布（盒鬚圖）</CardTitle>
                            </CardHeader>
                            <CardContent>
                                {boxPlotData.length > 0 ? (
                                    <BoxPlotChart data={boxPlotData} />
                                ) : (
                                    <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                                        <BarChart3 className="h-12 w-12 mb-2" />
                                        <p>需要至少 2 筆數值資料才能繪製盒鬚圖</p>
                                    </div>
                                )}
                            </CardContent>
                        </Card>
                    )}

                    {/* 資料明細表格 */}
                    {activeTab === 'table' && (
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-base">資料明細</CardTitle>
                            </CardHeader>
                            <CardContent>
                                <div className="rounded-md border">
                                    <Table>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead>耳號</TableHead>
                                                <TableHead>專案</TableHead>
                                                <TableHead>日期</TableHead>
                                                <TableHead>實驗室</TableHead>
                                                <TableHead>項目</TableHead>
                                                <TableHead className="text-right">結果值</TableHead>
                                                <TableHead>單位</TableHead>
                                                <TableHead>參考範圍</TableHead>
                                                <TableHead className="text-center">異常</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {chartFilteredData.slice(0, 200).map((r, idx) => (
                                                <TableRow key={idx} className={r.is_abnormal ? 'bg-red-50/50 dark:bg-red-950/20' : ''}>
                                                    <TableCell className="font-medium">{r.ear_tag}</TableCell>
                                                    <TableCell className="font-mono text-sm">{r.iacuc_no || '-'}</TableCell>
                                                    <TableCell>{formatDate(r.test_date)}</TableCell>
                                                    <TableCell>{r.lab_name || '-'}</TableCell>
                                                    <TableCell>{r.item_name}</TableCell>
                                                    <TableCell className={`text-right ${r.is_abnormal ? 'font-semibold text-red-600 dark:text-red-400' : ''}`}>
                                                        {r.result_value || '-'}
                                                    </TableCell>
                                                    <TableCell className="text-muted-foreground">{r.result_unit || '-'}</TableCell>
                                                    <TableCell className="text-muted-foreground">{r.reference_range || '-'}</TableCell>
                                                    <TableCell className="text-center">
                                                        {r.is_abnormal && (
                                                            <AlertTriangle className="h-4 w-4 text-red-500 inline" />
                                                        )}
                                                    </TableCell>
                                                </TableRow>
                                            ))}
                                        </TableBody>
                                    </Table>
                                    {chartFilteredData.length > 200 && (
                                        <p className="text-sm text-muted-foreground text-center py-2">
                                            僅顯示前 200 筆，完整資料請使用匯出功能
                                        </p>
                                    )}
                                </div>
                            </CardContent>
                        </Card>
                    )}
                </>
            )}

            {/* 無資料 */}
            {!isLoading && filteredData.length === 0 && (
                <Card>
                    <CardContent className="flex flex-col items-center justify-center py-12">
                        <FlaskConical className="h-16 w-16 text-muted-foreground mb-4" />
                        <p className="text-lg font-medium text-muted-foreground">尚無血液檢查分析資料</p>
                        <p className="text-sm text-muted-foreground mt-1">
                            請先在動物管理頁面建立血液檢查紀錄，或調整篩選條件
                        </p>
                    </CardContent>
                </Card>
            )}
        </div>
    )
}
