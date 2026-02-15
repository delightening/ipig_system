import { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
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
import { Loader2, Download, Droplets, FlaskConical, DollarSign, Hash } from 'lucide-react'

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
    const [dateFrom, setDateFrom] = useState('')
    const [dateTo, setDateTo] = useState('')
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
            {/* 標題列 */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">血液檢查費用報表</h1>
                    <p className="text-muted-foreground">依專案、日期與實驗室查詢血液檢查費用</p>
                </div>
                <Button onClick={exportToCSV} disabled={!report?.length}>
                    <Download className="mr-2 h-4 w-4" />
                    匯出 CSV
                </Button>
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
                            <div className="rounded-full bg-blue-100 p-3 dark:bg-blue-900/30">
                                <DollarSign className="h-6 w-6 text-blue-600 dark:text-blue-400" />
                            </div>
                            <div>
                                <p className="text-sm text-muted-foreground">總費用</p>
                                <p className="text-2xl font-bold">{formatCurrency(summary.totalCost)}</p>
                            </div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="flex items-center gap-4 pt-6">
                            <div className="rounded-full bg-green-100 p-3 dark:bg-green-900/30">
                                <Hash className="h-6 w-6 text-green-600 dark:text-green-400" />
                            </div>
                            <div>
                                <p className="text-sm text-muted-foreground">檢查項目數</p>
                                <p className="text-2xl font-bold">{summary.totalItems}</p>
                            </div>
                        </CardContent>
                    </Card>
                    <Card>
                        <CardContent className="flex items-center gap-4 pt-6">
                            <div className="rounded-full bg-purple-100 p-3 dark:bg-purple-900/30">
                                <FlaskConical className="h-6 w-6 text-purple-600 dark:text-purple-400" />
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
                                <TableHead>專案編號</TableHead>
                                <TableHead>耳號</TableHead>
                                <TableHead>檢查日期</TableHead>
                                <TableHead>實驗室</TableHead>
                                <TableHead className="text-right">項目數</TableHead>
                                <TableHead className="text-right">費用</TableHead>
                                <TableHead>建立者</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {report && report.length > 0 ? (
                                report.map((row, idx) => (
                                    <TableRow key={idx}>
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
                                <TableRow>
                                    <TableCell colSpan={7} className="text-center py-8">
                                        <Droplets className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                                        <p className="text-muted-foreground">尚無血液檢查費用資料</p>
                                    </TableCell>
                                </TableRow>
                            )}
                        </TableBody>
                    </Table>
                </div>
            )}
        </div>
    )
}
