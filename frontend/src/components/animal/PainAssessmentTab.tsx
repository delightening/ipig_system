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
import { Loader2, TrendingUp, Activity } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
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
        <Card className="overflow-hidden">
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

                <div className="@container">

                    {/* ── Table view: container ≥ 600px ── */}
                    <div className="hidden @[600px]:block overflow-x-auto">
                        <Table className="w-full" style={{ minWidth: 510 }}>
                            <TableHeader>
                                <TableRow className="bg-muted/50 hover:bg-muted/50">
                                    <SortableTableHead style={{ width: 100 }} sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>日期</SortableTableHead>
                                    <SortableTableHead style={{ width: 60 }} sortKey="post_op_days" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>術後天</SortableTableHead>
                                    <TableHead style={{ width: 60 }}>時段</TableHead>
                                    <TableHead style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">傷口</TableHead>
                                    <TableHead style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">行為</TableHead>
                                    <TableHead style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">食慾</TableHead>
                                    <TableHead style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">排便</TableHead>
                                    <TableHead style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">排尿</TableHead>
                                    <TableHead style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">疼痛</TableHead>
                                    <TableHead style={{ width: 50 }} className="text-center">總分</TableHead>
                                    <TableHead style={{ width: 90 }}>疼痛分級</TableHead>
                                    <TableHead style={{ width: 150 }}>給藥</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {isLoading ? (
                                    <TableRow><TableCell colSpan={12} className="p-0"><TableSkeleton rows={5} cols={12} /></TableCell></TableRow>
                                ) : !records || records.length === 0 ? (
                                    <TableEmptyRow colSpan={12} icon={Activity} title="尚無疼痛評估紀錄" description="請於手術紀錄中新增疼痛評估" />
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
                                                <TableCell style={{ width: 100 }} className="text-xs whitespace-nowrap">{formatDate(r.created_at)}</TableCell>
                                                <TableCell style={{ width: 60 }}>{r.post_op_days ?? '-'}</TableCell>
                                                <TableCell style={{ width: 60 }}>{r.time_period || '-'}</TableCell>
                                                <TableCell style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">{r.incision ?? '-'}</TableCell>
                                                <TableCell style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">{r.attitude_behavior ?? '-'}</TableCell>
                                                <TableCell style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">{r.appetite ?? '-'}</TableCell>
                                                <TableCell style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">{r.feces ?? '-'}</TableCell>
                                                <TableCell style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">{r.urine ?? '-'}</TableCell>
                                                <TableCell style={{ width: 50 }} className="text-center hidden @[810px]:table-cell">{r.pain_score ?? '-'}</TableCell>
                                                <TableCell style={{ width: 50 }} className="text-center font-bold">{total ?? '-'}</TableCell>
                                                <TableCell style={{ width: 90 }}>
                                                    {grade ? (
                                                        <Badge variant={grade.variant}>{grade.label}</Badge>
                                                    ) : '-'}
                                                </TableCell>
                                                <TableCell style={{ width: 150 }} className="text-xs text-muted-foreground whitespace-normal break-words">
                                                    {meds || '-'}
                                                </TableCell>
                                            </TableRow>
                                        )
                                    })
                                )}
                            </TableBody>
                        </Table>
                    </div>

                    {/* ── Card view: container < 600px ── */}
                    <div className="@[600px]:hidden space-y-3 py-1">
                        {isLoading ? (
                            <TableSkeleton rows={3} cols={1} />
                        ) : !records || records.length === 0 ? (
                            <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                                <Activity className="h-8 w-8" />
                                <p className="text-sm">尚無疼痛評估紀錄</p>
                                <p className="text-xs">請於手術紀錄中新增疼痛評估</p>
                            </div>
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
                                const dayLabel = r.post_op_days != null
                                    ? `D${r.post_op_days}${r.time_period ? `-${r.time_period}` : ''}`
                                    : (r.time_period || '-')
                                return (
                                    <div key={r.id} className="rounded-lg border bg-card p-3 space-y-2">
                                        <div className="flex items-center justify-between gap-2">
                                            <span className="text-sm font-medium text-foreground">{formatDate(r.created_at)}</span>
                                            <span className="text-xs text-muted-foreground">{dayLabel}</span>
                                        </div>
                                        <div className="flex items-center gap-2">
                                            {grade ? <Badge variant={grade.variant}>{grade.label}</Badge> : null}
                                            <span className="text-sm font-bold">總分 {total ?? '-'}</span>
                                        </div>
                                        <div className="grid grid-cols-3 gap-x-3 gap-y-1 bg-muted/50 p-2 rounded text-xs text-muted-foreground">
                                            <span>傷口：{r.incision ?? '-'}</span>
                                            <span>行為：{r.attitude_behavior ?? '-'}</span>
                                            <span>食慾：{r.appetite ?? '-'}</span>
                                            <span>排便：{r.feces ?? '-'}</span>
                                            <span>排尿：{r.urine ?? '-'}</span>
                                            <span>疼痛：{r.pain_score ?? '-'}</span>
                                        </div>
                                        {meds && (
                                            <div className="text-xs text-muted-foreground pt-1 border-t">
                                                💊 {meds}
                                            </div>
                                        )}
                                    </div>
                                )
                            })
                        )}
                    </div>

                </div>
            </CardContent>
        </Card>
    )
}
