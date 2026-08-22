// 獸醫師建議 Tab — 唯讀。內容來自巡場報告「完成」時自動歸位（單一來源、單向同步）。
// 獸醫的撰寫入口統一在巡場報告，這裡只呈現該動物的建議歷史，供交班/查閱。

import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { useTableSort } from '@/hooks/useTableSort'
import { Stethoscope } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'

interface VetAdviceRecord {
    id: string
    animal_id: string
    advice_date: string
    observation: string
    suggested_treatment: string
    follow_up: string
    created_at: string
    updated_at: string
}

interface VetRecommendationsTabProps {
    animalId: string
}

export function VetRecommendationsTab({ animalId }: VetRecommendationsTabProps) {
    const queryKey = ['animal-vet-advice-records', animalId]

    const { data: records, isLoading } = useQuery({
        queryKey,
        queryFn: async () => {
            const res = await api.get<VetAdviceRecord[]>(`/animals/${animalId}/vet-advice-records`)
            return res.data
        },
        staleTime: 30_000,
    })

    const { sortedData, sort, toggleSort } = useTableSort(records)
    // 手機版內容文字樣式（與桌面表格一致：非 muted，內容需可讀）
    const cellText = 'text-sm whitespace-pre-wrap leading-snug'

    return (
        <Card className="overflow-hidden">
            <CardHeader className="flex flex-row items-center justify-between">
                <div className="flex items-center gap-2">
                    <Stethoscope className="h-5 w-5 text-status-success-solid" />
                    <div>
                        <CardTitle className="text-status-success-solid">獸醫師建議</CardTitle>
                        <CardDescription>來自巡場報告完成後自動歸位（唯讀）</CardDescription>
                    </div>
                </div>
                <span className="text-sm text-muted-foreground shrink-0">共 {records?.length ?? 0} 筆</span>
            </CardHeader>
            <CardContent>
                <div className="@container">

                    {/* Table view: container ≥ 600px */}
                    <div className="hidden @[600px]:block overflow-x-auto">
                        <Table className="w-full" style={{ minWidth: 560 }}>
                            <TableHeader>
                                <TableRow className="bg-muted/50 hover:bg-muted/50">
                                    <SortableTableHead style={{ width: 120 }} sortKey="advice_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                                        日期
                                    </SortableTableHead>
                                    <TableHead style={{ minWidth: 120 }}>觀察</TableHead>
                                    <TableHead style={{ minWidth: 120 }}>建議</TableHead>
                                    <TableHead style={{ minWidth: 120 }}>追蹤改善</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {isLoading ? (
                                    <TableRow><TableCell colSpan={4} className="p-0"><TableSkeleton rows={5} cols={4} /></TableCell></TableRow>
                                ) : !records || records.length === 0 ? (
                                    <TableEmptyRow colSpan={4} icon={Stethoscope} title="尚無獸醫師建議" description="巡場報告完成後會自動出現在這裡" />
                                ) : (
                                    (sortedData ?? records).map((r) => (
                                        <TableRow key={r.id}>
                                            <TableCell style={{ width: 120 }} className="whitespace-nowrap font-medium">
                                                {r.advice_date}
                                            </TableCell>
                                            <TableCell style={{ minWidth: 120 }} className="whitespace-pre-wrap text-sm">
                                                {r.observation || '-'}
                                            </TableCell>
                                            <TableCell style={{ minWidth: 120 }} className="whitespace-pre-wrap text-sm">
                                                {r.suggested_treatment || '-'}
                                            </TableCell>
                                            <TableCell style={{ minWidth: 120 }} className="whitespace-pre-wrap text-sm">
                                                {r.follow_up || '-'}
                                            </TableCell>
                                        </TableRow>
                                    ))
                                )}
                            </TableBody>
                        </Table>
                    </div>

                    {/* Card view: container < 600px */}
                    <div className="@[600px]:hidden space-y-3 py-1">
                        {isLoading ? (
                            <TableSkeleton rows={3} cols={1} />
                        ) : !records || records.length === 0 ? (
                            <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                                <Stethoscope className="h-8 w-8" />
                                <p className="text-sm">尚無獸醫師建議</p>
                                <p className="text-xs">巡場報告完成後會自動出現在這裡</p>
                            </div>
                        ) : (
                            (sortedData ?? records).map((r) => (
                                <div key={r.id} className="rounded-lg border bg-card p-3 space-y-2">
                                    <span className="text-sm font-medium text-foreground">{r.advice_date}</span>
                                    <div>
                                        <div className="text-xs text-muted-foreground uppercase tracking-wide mb-0.5">觀察</div>
                                        <p className={cellText}>{r.observation || '-'}</p>
                                    </div>
                                    <div>
                                        <div className="text-xs text-muted-foreground uppercase tracking-wide mb-0.5">建議</div>
                                        <p className={cellText}>{r.suggested_treatment || '-'}</p>
                                    </div>
                                    <div>
                                        <div className="text-xs text-muted-foreground uppercase tracking-wide mb-0.5">追蹤改善</div>
                                        <p className={cellText}>{r.follow_up || '-'}</p>
                                    </div>
                                </div>
                            ))
                        )}
                    </div>

                </div>
            </CardContent>
        </Card>
    )
}
