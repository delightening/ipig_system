// 疼痛評估紀錄 Tab — 唯讀展示
// 評估項目：傷口狀況、態度/行為、食慾、排便、排尿、疼痛分數 → 總分 → 疼痛分級

import { lazy, Suspense, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import { Loader2, TrendingUp } from 'lucide-react'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { calcTotal, getPainGrade } from './painAssessmentConstants'

const PainAssessmentChart = lazy(() => import('./PainAssessmentChart'))

interface CareRecord {
    id: string
    record_type: string
    record_id: string
    record_mode: string
    post_op_days: number | null
    time_period: string | null
    incision: number | null
    attitude_behavior: number | null
    appetite: number | null
    feces: number | null
    urine: number | null
    pain_score: number | null
    injection_ketorolac: boolean
    injection_meloxicam: boolean
    oral_meloxicam: boolean
    post_medications: Array<{ name: string; dose?: string; dosage_unit?: string }> | null
    vet_read: boolean
    created_at: string
}

interface PainAssessmentTabProps {
    animalId: string
    observations: Array<{ id: string | number; observation_date?: string }>
    surgeries: Array<{ id: string | number; surgery_date?: string }>
}

export function PainAssessmentTab({ animalId }: PainAssessmentTabProps) {
    const [showChart, setShowChart] = useState(false)

    const { data: records, isLoading } = useQuery({
        queryKey: ['animal-care-records', animalId],
        queryFn: async () => {
            const res = await api.get<CareRecord[]>(`/animals/${animalId}/care-records`)
            return res.data
        },
        staleTime: 30_000,
    })

    const { sortedData: sortedRecords, sort, toggleSort } = useTableSort(records)

    const chartData = (records || [])
        .slice()
        .sort((a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime())
        .map((r, i) => {
            const total = calcTotal(
                r.incision != null ? String(r.incision) : '',
                r.attitude_behavior != null ? String(r.attitude_behavior) : '',
                r.appetite != null ? String(r.appetite) : '',
                r.feces != null ? String(r.feces) : '',
                r.urine != null ? String(r.urine) : '',
                r.pain_score != null ? String(r.pain_score) : '',
            )
            return {
                name: r.post_op_days != null
                    ? `D${r.post_op_days}${r.time_period ? `-${r.time_period}` : ''}`
                    : `#${i + 1}`,
                總分: total,
            }
        })

    const formatDate = (dateStr: string) =>
        new Date(dateStr).toLocaleDateString('zh-TW', {
            timeZone: 'Asia/Taipei',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
        })

    const getRecordTotal = (r: CareRecord) =>
        calcTotal(
            r.incision != null ? String(r.incision) : '',
            r.attitude_behavior != null ? String(r.attitude_behavior) : '',
            r.appetite != null ? String(r.appetite) : '',
            r.feces != null ? String(r.feces) : '',
            r.urine != null ? String(r.urine) : '',
            r.pain_score != null ? String(r.pain_score) : '',
        )

    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between">
                <div>
                    <CardTitle>疼痛評估紀錄</CardTitle>
                    <CardDescription>依據 TU-03-05-03B 記錄術後疼痛評估與給藥（請於手術紀錄中新增）</CardDescription>
                </div>
                <div className="flex items-center gap-2 shrink-0">
                    <span className="text-sm text-muted-foreground mr-2">共 {records?.length ?? 0} 筆</span>
                    <Button variant="outline" onClick={() => setShowChart(!showChart)}>
                        <TrendingUp className="h-4 w-4 mr-1" />
                        {showChart ? '隱藏趨勢' : '顯示趨勢'}
                    </Button>
                </div>
            </CardHeader>
            <CardContent className="space-y-4">
                {showChart && chartData.length > 0 && (
                    <div>
                        <h4 className="text-base font-semibold mb-2">疼痛總分趨勢</h4>
                        <Suspense fallback={<div className="flex justify-center py-8"><Loader2 className="h-6 w-6 animate-spin" /></div>}>
                            <PainAssessmentChart data={chartData} />
                        </Suspense>
                    </div>
                )}

                <Table>
                    <TableHeader>
                        <TableRow>
                            <SortableTableHead className="w-[100px]" sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>日期</SortableTableHead>
                            <SortableTableHead className="w-[60px]" sortKey="post_op_days" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>術後天</SortableTableHead>
                            <TableHead className="w-[60px]">時段</TableHead>
                            <TableHead className="w-[50px]">傷口</TableHead>
                            <TableHead className="w-[50px]">行為</TableHead>
                            <TableHead className="w-[50px]">食慾</TableHead>
                            <TableHead className="w-[50px]">排便</TableHead>
                            <TableHead className="w-[50px]">排尿</TableHead>
                            <TableHead className="w-[50px]">疼痛</TableHead>
                            <TableHead className="w-[50px]">總分</TableHead>
                            <TableHead>疼痛分級</TableHead>
                            <TableHead>給藥</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={12} className="text-center py-8">
                                    <Loader2 className="h-5 w-5 animate-spin inline-block mr-2" />載入中...
                                </TableCell>
                            </TableRow>
                        ) : !records || records.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={12} className="text-center py-8 text-muted-foreground">
                                    <div>尚無疼痛評估紀錄</div>
                                    <div className="text-sm mt-1">請於手術紀錄中新增疼痛評估</div>
                                </TableCell>
                            </TableRow>
                        ) : (
                            (sortedRecords ?? records)?.map((r) => {
                                const total = getRecordTotal(r)
                                const grade = getPainGrade(total)
                                const meds = r.post_medications && r.post_medications.length > 0
                                    ? r.post_medications.map((m) => m.name + (m.dose ? ` ${m.dose}${m.dosage_unit || ''}` : '')).join('、')
                                    : [
                                        r.injection_ketorolac && 'Ketorolac(IM)',
                                        r.injection_meloxicam && 'Meloxicam(IM)',
                                        r.oral_meloxicam && 'Meloxicam(PO)',
                                    ].filter(Boolean).join(' ')
                                return (
                                    <TableRow key={r.id}>
                                        <TableCell className="text-xs whitespace-nowrap">{formatDate(r.created_at)}</TableCell>
                                        <TableCell>{r.post_op_days ?? '-'}</TableCell>
                                        <TableCell>{r.time_period || '-'}</TableCell>
                                        <TableCell className="text-center">{r.incision ?? '-'}</TableCell>
                                        <TableCell className="text-center">{r.attitude_behavior ?? '-'}</TableCell>
                                        <TableCell className="text-center">{r.appetite ?? '-'}</TableCell>
                                        <TableCell className="text-center">{r.feces ?? '-'}</TableCell>
                                        <TableCell className="text-center">{r.urine ?? '-'}</TableCell>
                                        <TableCell className="text-center">{r.pain_score ?? '-'}</TableCell>
                                        <TableCell className="text-center font-bold">{total ?? '-'}</TableCell>
                                        <TableCell>
                                            {grade ? (
                                                <Badge variant={grade.variant}>{grade.label}</Badge>
                                            ) : '-'}
                                        </TableCell>
                                        <TableCell className="text-xs text-muted-foreground">
                                            {meds || '-'}
                                        </TableCell>
                                    </TableRow>
                                )
                            })
                        )}
                    </TableBody>
                </Table>
            </CardContent>
        </Card>
    )
}
