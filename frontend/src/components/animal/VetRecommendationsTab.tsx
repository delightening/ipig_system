// 獸醫師建議 Tab — 多筆紀錄列表（日期+觀察+建議處置）

import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { format } from 'date-fns'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/input'
import { DatePicker } from '@/components/ui/date-picker'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { useTableSort } from '@/hooks/useTableSort'
import {
    Loader2,
    Stethoscope,
    Plus,
    Pencil,
    Trash2,
    Save,
    X,
} from 'lucide-react'

interface VetAdviceRecord {
    id: string
    animal_id: string
    advice_date: string
    observation: string
    suggested_treatment: string
    created_at: string
    updated_at: string
}

interface FormData {
    advice_date: string
    observation: string
    suggested_treatment: string
}

const emptyForm: FormData = {
    advice_date: format(new Date(), 'yyyy-MM-dd'),
    observation: '',
    suggested_treatment: '',
}

interface VetRecommendationsTabProps {
    animalId: string
}

export function VetRecommendationsTab({ animalId }: VetRecommendationsTabProps) {
    const queryClient = useQueryClient()
    const queryKey = ['animal-vet-advice-records', animalId]

    const [showForm, setShowForm] = useState(false)
    const [editingId, setEditingId] = useState<string | null>(null)
    const [formData, setFormData] = useState<FormData>({ ...emptyForm })

    const { data: records, isLoading } = useQuery({
        queryKey,
        queryFn: async () => {
            const res = await api.get<VetAdviceRecord[]>(`/animals/${animalId}/vet-advice-records`)
            return res.data
        },
        staleTime: 30_000,
    })

    const { sortedData, sort, toggleSort } = useTableSort(records)

    const createMutation = useMutation({
        mutationFn: async (data: FormData) => {
            await api.post(`/animals/${animalId}/vet-advice-records`, data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey })
            toast({ title: '成功', description: '獸醫師建議已新增' })
            resetForm()
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '新增失敗'), variant: 'destructive' })
        },
    })

    const updateMutation = useMutation({
        mutationFn: async ({ id, data }: { id: string; data: FormData }) => {
            await api.put(`/vet-advice-records/${id}`, data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey })
            toast({ title: '成功', description: '獸醫師建議已更新' })
            resetForm()
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '更新失敗'), variant: 'destructive' })
        },
    })

    const deleteMutation = useMutation({
        mutationFn: async (id: string) => {
            await api.post(`/vet-advice-records/${id}/delete`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey })
            toast({ title: '成功', description: '紀錄已刪除' })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '刪除失敗'), variant: 'destructive' })
        },
    })

    const resetForm = () => {
        setShowForm(false)
        setEditingId(null)
        setFormData({ ...emptyForm })
    }

    const startEdit = (record: VetAdviceRecord) => {
        setEditingId(record.id)
        setFormData({
            advice_date: record.advice_date,
            observation: record.observation,
            suggested_treatment: record.suggested_treatment,
        })
        setShowForm(true)
    }

    const handleSubmit = () => {
        if (!formData.advice_date) {
            toast({ title: '提醒', description: '請選擇日期', variant: 'destructive' })
            return
        }
        if (editingId) {
            updateMutation.mutate({ id: editingId, data: formData })
        } else {
            createMutation.mutate(formData)
        }
    }

    const isSaving = createMutation.isPending || updateMutation.isPending

    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between">
                <div className="flex items-center gap-2">
                    <Stethoscope className="h-5 w-5 text-status-success-solid" />
                    <div>
                        <CardTitle className="text-status-success-solid">獸醫師建議</CardTitle>
                        <CardDescription>獸醫師巡查觀察與建議處置紀錄</CardDescription>
                    </div>
                </div>
                <div className="flex items-center gap-2 shrink-0">
                    <span className="text-sm text-muted-foreground">共 {records?.length ?? 0} 筆</span>
                    {!showForm && (
                        <Button
                            onClick={() => { setShowForm(true); setEditingId(null); setFormData({ ...emptyForm }) }}
                            className="bg-status-success-solid hover:bg-green-700"
                        >
                            <Plus className="h-4 w-4 mr-1" />
                            新增
                        </Button>
                    )}
                </div>
            </CardHeader>
            <CardContent className="space-y-4">
                {/* 新增/編輯表單 */}
                {showForm && (
                    <div className="border rounded-lg p-4 space-y-3 bg-muted/30">
                        <div className="flex items-center justify-between">
                            <h4 className="text-sm font-semibold text-status-success-solid">
                                {editingId ? '編輯建議' : '新增建議'}
                            </h4>
                            <button type="button" onClick={resetForm} className="text-muted-foreground hover:text-foreground">
                                <X className="h-4 w-4" />
                            </button>
                        </div>
                        <div className="grid grid-cols-[180px_1fr] gap-3 items-start">
                            <div>
                                <label className="text-sm font-medium mb-1 block">日期</label>
                                <DatePicker
                                    value={formData.advice_date}
                                    onChange={(v) => setFormData((p) => ({ ...p, advice_date: v }))}
                                    required
                                />
                            </div>
                            <div className="space-y-3">
                                <div>
                                    <label className="text-sm font-medium mb-1 block">觀察</label>
                                    <Textarea
                                        value={formData.observation}
                                        onChange={(e) => setFormData((p) => ({ ...p, observation: e.target.value }))}
                                        placeholder="觀察內容..."
                                        rows={2}
                                        className="resize-none"
                                    />
                                </div>
                                <div>
                                    <label className="text-sm font-medium mb-1 block">建議處置</label>
                                    <Textarea
                                        value={formData.suggested_treatment}
                                        onChange={(e) => setFormData((p) => ({ ...p, suggested_treatment: e.target.value }))}
                                        placeholder="建議處置..."
                                        rows={2}
                                        className="resize-none"
                                    />
                                </div>
                            </div>
                        </div>
                        <div className="flex justify-end gap-2">
                            <Button variant="outline" onClick={resetForm} disabled={isSaving}>
                                取消
                            </Button>
                            <Button
                                onClick={handleSubmit}
                                disabled={isSaving}
                                className="bg-status-success-solid hover:bg-green-700"
                            >
                                {isSaving ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Save className="h-4 w-4 mr-1" />}
                                {editingId ? '更新' : '儲存'}
                            </Button>
                        </div>
                    </div>
                )}

                {/* 紀錄列表 */}
                <Table>
                    <TableHeader>
                        <TableRow>
                            <SortableTableHead className="w-[120px]" sortKey="advice_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                                日期
                            </SortableTableHead>
                            <TableHead>觀察</TableHead>
                            <TableHead>建議處置</TableHead>
                            <TableHead className="w-[80px]">操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={4} className="text-center py-8">
                                    <Loader2 className="h-5 w-5 animate-spin inline-block mr-2" />載入中...
                                </TableCell>
                            </TableRow>
                        ) : !records || records.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={4} className="text-center py-8 text-muted-foreground">
                                    <div>尚無獸醫師建議紀錄</div>
                                    <div className="text-sm mt-1">點擊「新增」開始記錄</div>
                                </TableCell>
                            </TableRow>
                        ) : (
                            (sortedData ?? records).map((r) => (
                                <TableRow key={r.id}>
                                    <TableCell className="whitespace-nowrap font-medium">
                                        {r.advice_date}
                                    </TableCell>
                                    <TableCell className="whitespace-pre-wrap text-sm">
                                        {r.observation || '-'}
                                    </TableCell>
                                    <TableCell className="whitespace-pre-wrap text-sm">
                                        {r.suggested_treatment || '-'}
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex items-center gap-1">
                                            <button
                                                type="button"
                                                onClick={() => startEdit(r)}
                                                className="p-1 rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
                                                title="編輯"
                                            >
                                                <Pencil className="h-3.5 w-3.5" />
                                            </button>
                                            <button
                                                type="button"
                                                onClick={() => {
                                                    if (confirm('確定要刪除此筆建議？')) {
                                                        deleteMutation.mutate(r.id)
                                                    }
                                                }}
                                                className="p-1 rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
                                                title="刪除"
                                            >
                                                <Trash2 className="h-3.5 w-3.5" />
                                            </button>
                                        </div>
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </CardContent>
        </Card>
    )
}
