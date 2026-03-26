import { useState, useMemo } from 'react'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { useTableSort } from '@/hooks/useTableSort'
import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { formatDate } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { Loader2, Download, Droplets, FlaskConical, DollarSign, Hash } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'

// 血液檢查費用報表型別
interface BloodTestCostReport {
    iacuc_no: string | null
    ear_tag: string
    animal_id: string
    test_date: string
    lab_name: string | null
    item_count: number
    total_cost: string | null
    created_by_name: string | null
    created_at: string
}

export function BloodTestCostReportPage() {
    const [iacucNo, setIacucNo] = useState('')
    const { from: dateFrom, to: dateTo, setFrom: setDateFrom, setTo: setDateTo } = useDateRangeFilter()
    const [labName, setLabName] = useState('')

    // 建立查詢參數
    const queryParams = useMemo(() => {
        const params = new URLSearchParams()
        if (iacucNo.trim()) params.set('iacuc_no', iacucNo.trim())
        if (dateFrom) params.set('date_from', dateFrom)
        if (dateTo) params.set('date_to', dateTo)
        if (labName.trim()) params.set('lab_name', labName.trim())
        return params.toString()
    }, [iacucNo, dateFrom, dateTo, labName])

    const { data: report, isLoading } = useQuery<BloodTestCostReport[]>({
        queryKey: ['report-blood-test-cost', queryParams],
        queryFn: async () => {
            const url = queryParams ? `/reports/blood-test-cost?${queryParams}` : '/reports/blood-test-cost'
            const response = await api.get<BloodTestCostReport[]>(url)
            return response.data
        },
    })

    const { sortedData, sort, toggleSort } = useTableSort(report)

    // 摘要統計
    const summary = useMemo(() => {
        if (!report || report.length === 0) return { totalCost: 0, totalItems: 0, totalTests: 0 }
        return {
            totalCost: report.reduce((sum, r) => sum + parseFloat(r.total_cost || '0'), 0),
            totalItems: report.reduce((sum, r) => sum + r.item_count, 0),
            totalTests: report.length,
        }
    }, [report])

    // 格式化金額
    const formatCurrency = (value: number) => {
        return new Intl.NumberFormat('zh-TW', {
            style: 'currency',
            currency: 'TWD',
            minimumFractionDigits: 0,
            maximumFractionDigits: 2,
        }).format(value)
    }

    // CSV 匯出
    const exportToCSV = () => {
        if (!report) return

        const headers = ['專案編號', '耳號', '檢查日期', '實驗室', '檢查項目數', '費用', '建立者', '建立時間']
        const rows = report.map(r => [
            r.iacuc_no || '',
            r.ear_tag,
            r.test_date,
            r.lab_name || '',
            r.item_count.toString(),
            r.total_cost || '0',
            r.created_by_name || '',
            r.created_at,
        ])

        const csvContent = [headers, ...rows]
            .map(row => row.map(cell => `"${cell}"`).join(','))
            .join('\n')

        const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
        const link = document.createElement('a')
        link.href = URL.createObjectURL(blob)
        link.download = `blood_test_cost_${new Date().toISOString().split('T')[0]}.csv`
        link.click()
    }

    return (
        <div className="space-y-6">
            <PageHeader
                title="血液檢查費用報表"
                description="依專案、日期與實驗室查詢血液檢查費用"
                actions={
                    <Button size="sm" onClick={exportToCSV} disabled={!report?.length}>
                        <Download className="mr-2 h-4 w-4" />
                        匯出 CSV
                    </Button>
                }
            />

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
                                placeholder="例: PIG-115001"
                                value={iacucNo}
                                onChange={(e) => setIacucNo(e.target.value)}
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
                        <div className="space-y-2">
                            <Label htmlFor="lab_name">實驗室</Label>
                            <Input
                                id="lab_name"
                                placeholder="輸入實驗室名稱"
                                value={labName}
                                onChange={(e) => setLabName(e.target.value)}
                            />
                        </div>
                    </div>
                </CardContent>
            </Card>

            {/* 摘要卡片 */}
            {report && report.length > 0 && (
                <div className="grid gap-4 sm:grid-cols-3">
                    <Card>
                        <CardContent className="flex items-center gap-4 pt-6">
                            <div className="rounded-full bg-primary/10 p-3">
                                <DollarSign className="h-6 w-6 text-primary" />
                            </div>
                            <div>
                                <p className="text-sm text-muted-foreground">總費用</p>
                                <p className="text-2xl font-bold">{formatCurrency(summary.totalCost)}</p>
                            </div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="flex items-center gap-4 pt-6">
                            <div className="rounded-full bg-status-success-bg p-3">
                                <Hash className="h-6 w-6 text-status-success-text" />
                            </div>
                            <div>
                                <p className="text-sm text-muted-foreground">檢查項目數</p>
                                <p className="text-2xl font-bold">{summary.totalItems}</p>
                            </div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="flex items-center gap-4 pt-6">
                            <div className="rounded-full bg-status-purple-bg p-3">
                                <FlaskConical className="h-6 w-6 text-status-purple-text" />
                            </div>
                            <div>
                                <p className="text-sm text-muted-foreground">檢查次數</p>
                                <p className="text-2xl font-bold">{summary.totalTests}</p>
                            </div>
                        </CardContent>
                    </Card>
                </div>
            )}

            {/* 載入中 */}
            {isLoading && (
                <div className="flex items-center justify-center py-8">
                    <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                </div>
            )}

            {/* 報表表格 */}
            {!isLoading && (
                <div className="rounded-md border">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <SortableTableHead sortKey="iacuc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>專案編號</SortableTableHead>
                                <SortableTableHead sortKey="ear_tag" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>耳號</SortableTableHead>
                                <SortableTableHead sortKey="test_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>檢查日期</SortableTableHead>
                                <SortableTableHead sortKey="lab_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>實驗室</SortableTableHead>
                                <SortableTableHead sortKey="item_count" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">項目數</SortableTableHead>
                                <SortableTableHead sortKey="total_cost" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-right">費用</SortableTableHead>
                                <SortableTableHead sortKey="created_by_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>建立者</SortableTableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {sortedData && sortedData.length > 0 ? (
                                sortedData.map((row, idx) => (
                                    <TableRow key={`${row.iacuc_no}-${row.ear_tag}-${idx}`}>
                                        <TableCell className="font-mono text-sm">
                                            {row.iacuc_no || <span className="text-muted-foreground">未分配</span>}
                                        </TableCell>
                                        <TableCell className="font-medium">{row.ear_tag}</TableCell>
                                        <TableCell>{formatDate(row.test_date)}</TableCell>
                                        <TableCell>{row.lab_name || '-'}</TableCell>
                                        <TableCell className="text-right">{row.item_count}</TableCell>
                                        <TableCell className="text-right font-medium">
                                            {row.total_cost && parseFloat(row.total_cost) > 0
                                                ? formatCurrency(parseFloat(row.total_cost))
                                                : '-'}
                                        </TableCell>
                                        <TableCell>{row.created_by_name || '-'}</TableCell>
                                    </TableRow>
                                ))
                            ) : (
                                <TableEmptyRow colSpan={7} icon={Droplets} title="尚無血液檢查費用資料" />
                            )}
                        </TableBody>
                    </Table>
                </div>
            )}
        </div>
    )
}
