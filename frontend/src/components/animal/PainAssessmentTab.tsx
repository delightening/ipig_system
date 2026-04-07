// 疼痛評估紀錄 Tab — 依據 TU-03-05-03B 試驗豬隻疼痛評估紀錄表
// 評估項目：傷口狀況、態度/行為、食慾、排便、排尿、疼痛分數 → 總分 → 疼痛分級
// 術後給藥：注射 Ketorolac / 注射 Meloxicam / 口服 Meloxicam

import { useState, lazy, Suspense } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import {
    Dialog,
    DialogContent,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'
import { Plus, Trash2, Edit2, Loader2, TrendingUp } from 'lucide-react'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'

const PainAssessmentChart = lazy(() => import('./PainAssessmentChart'))

import {
    INCISION_OPTIONS,
    ATTITUDE_OPTIONS,
    APPETITE_OPTIONS,
    FECES_OPTIONS,
    URINE_OPTIONS,
    PAIN_SCORE_OPTIONS,
    calcTotal,
    getPainGrade,
    type AssessmentOption,
} from './painAssessmentConstants'

// ── 型別定義 ────────────────────────────────────────────────────────────────
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
    vet_read: boolean
    created_at: string
}

interface PainAssessmentTabProps {
    animalId: string
    observations: Array<{ id: string | number; observation_date?: string }>
    surgeries: Array<{ id: string | number; surgery_date?: string }>
}

const emptyForm = {
    record_type: 'surgery' as 'observation' | 'surgery',
    record_id: '',
    post_op_days: '',
    time_period: 'AM',
    incision: '',
    attitude_behavior: '',
    appetite: '',
    feces: '',
    urine: '',
    pain_score: '',
    injection_ketorolac: false,
    injection_meloxicam: false,
    oral_meloxicam: false,
}

// ── 主元件 ──────────────────────────────────────────────────────────────────
export function PainAssessmentTab({ animalId, observations, surgeries }: PainAssessmentTabProps) {
    const queryClient = useQueryClient()
    const [showAddDialog, setShowAddDialog] = useState(false)
    const [editingRecord, setEditingRecord] = useState<CareRecord | null>(null)
    const [showChart, setShowChart] = useState(false)
    const [deleteTarget, setDeleteTarget] = useState<string | null>(null)
    const [form, setForm] = useState(emptyForm)

    const { data: records, isLoading } = useQuery({
        queryKey: ['animal-care-records', animalId],
        queryFn: async () => {
            const res = await api.get<CareRecord[]>(`/animals/${animalId}/care-records`)
            return res.data
        },
        staleTime: 30_000,
    })

    const { sortedData: sortedRecords, sort, toggleSort } = useTableSort(records)

    const buildPayload = (data: typeof form) => ({
        record_type: data.record_type,
        record_id: data.record_id,
        record_mode: 'pain_assessment',
        post_op_days: data.post_op_days ? parseInt(data.post_op_days) : null,
        time_period: data.time_period || null,
        incision: data.incision !== '' ? parseInt(data.incision) : null,
        attitude_behavior: data.attitude_behavior !== '' ? parseInt(data.attitude_behavior) : null,
        appetite: data.appetite !== '' ? parseInt(data.appetite) : null,
        feces: data.feces !== '' ? parseInt(data.feces) : null,
        urine: data.urine !== '' ? parseInt(data.urine) : null,
        pain_score: data.pain_score !== '' ? parseInt(data.pain_score) : null,
        injection_ketorolac: data.injection_ketorolac,
        injection_meloxicam: data.injection_meloxicam,
        oral_meloxicam: data.oral_meloxicam,
    })

    const createMutation = useMutation({
        mutationFn: async (data: typeof form) => api.post(`/animals/${animalId}/care-records`, buildPayload(data)),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-care-records', animalId] })
            toast({ title: '成功', description: '疼痛評估紀錄已新增' })
            setShowAddDialog(false)
            setForm(emptyForm)
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '新增失敗'), variant: 'destructive' })
        },
    })

    const updateMutation = useMutation({
        mutationFn: async ({ id, data }: { id: string; data: typeof form }) => {
            const { record_type: _rt, record_id: _ri, ...rest } = buildPayload(data)
            return api.put(`/care-records/${id}`, rest)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-care-records', animalId] })
            toast({ title: '成功', description: '疼痛評估紀錄已更新' })
            setEditingRecord(null)
            setForm(emptyForm)
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '更新失敗'), variant: 'destructive' })
        },
    })

    const deleteMutation = useMutation({
        mutationFn: async ({ id, reason }: { id: string; reason: string }) =>
            deleteResource(`/care-records/${id}`, { data: { reason } }),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-care-records', animalId] })
            toast({ title: '成功', description: '紀錄已刪除' })
            setDeleteTarget(null)
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '刪除失敗'), variant: 'destructive' })
        },
    })

    const sourceOptions = [
        ...observations.map((o) => ({
            value: String(o.id),
            label: `觀察 ${o.observation_date ? new Date(o.observation_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' }) : String(o.id).slice(0, 8)}`,
            type: 'observation' as const,
        })),
        ...surgeries.map((s) => ({
            value: String(s.id),
            label: `手術 ${s.surgery_date ? new Date(s.surgery_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' }) : String(s.id).slice(0, 8)}`,
            type: 'surgery' as const,
        })),
    ]

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

    const openEditDialog = (record: CareRecord) => {
        setEditingRecord(record)
        setForm({
            record_type: record.record_type as 'observation' | 'surgery',
            record_id: record.record_id,
            post_op_days: record.post_op_days != null ? String(record.post_op_days) : '',
            time_period: record.time_period || 'AM',
            incision: record.incision != null ? String(record.incision) : '',
            attitude_behavior: record.attitude_behavior != null ? String(record.attitude_behavior) : '',
            appetite: record.appetite != null ? String(record.appetite) : '',
            feces: record.feces != null ? String(record.feces) : '',
            urine: record.urine != null ? String(record.urine) : '',
            pain_score: record.pain_score != null ? String(record.pain_score) : '',
            injection_ketorolac: record.injection_ketorolac,
            injection_meloxicam: record.injection_meloxicam,
            oral_meloxicam: record.oral_meloxicam,
        })
    }

    const handleSubmit = () => {
        if (!form.record_id) {
            toast({ title: '請選擇關聯紀錄', variant: 'destructive' })
            return
        }
        if (editingRecord) {
            updateMutation.mutate({ id: editingRecord.id, data: form })
        } else {
            createMutation.mutate(form)
        }
    }

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

    const formTotal = calcTotal(
        form.incision, form.attitude_behavior, form.appetite,
        form.feces, form.urine, form.pain_score,
    )
    const formGrade = getPainGrade(formTotal)

    return (
        <>
            <Card>
                <CardHeader className="flex flex-row items-center justify-between">
                    <div>
                        <CardTitle>疼痛評估紀錄</CardTitle>
                        <CardDescription>依據 TU-03-05-03B 記錄術後疼痛評估與給藥</CardDescription>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                        <span className="text-sm text-muted-foreground mr-2">共 {records?.length ?? 0} 筆</span>
                        <Button variant="outline" onClick={() => setShowChart(!showChart)}>
                            <TrendingUp className="h-4 w-4 mr-1" />
                            {showChart ? '隱藏趨勢' : '顯示趨勢'}
                        </Button>
                        <Button
                            className="bg-purple-600 hover:bg-purple-700"
                            onClick={() => { setForm(emptyForm); setEditingRecord(null); setShowAddDialog(true) }}
                        >
                            <Plus className="h-4 w-4 mr-1" />
                            新增評估
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
                                <TableHead className="w-[80px] text-right">操作</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {isLoading ? (
                                <TableRow>
                                    <TableCell colSpan={13} className="text-center py-8">
                                        <Loader2 className="h-5 w-5 animate-spin inline-block mr-2" />載入中...
                                    </TableCell>
                                </TableRow>
                            ) : !records || records.length === 0 ? (
                                <TableRow>
                                    <TableCell colSpan={13} className="text-center py-8 text-muted-foreground">
                                        <div>尚無疼痛評估紀錄</div>
                                        <div className="text-sm mt-1">點擊上方按鈕新增</div>
                                    </TableCell>
                                </TableRow>
                            ) : (
                                (sortedRecords ?? records)?.map((r) => {
                                    const total = getRecordTotal(r)
                                    const grade = getPainGrade(total)
                                    const meds = [
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
                                            <TableCell>
                                                <div className="flex items-center justify-end gap-1">
                                                    <Button variant="ghost" size="icon" onClick={() => openEditDialog(r)} title="編輯">
                                                        <Edit2 className="h-3.5 w-3.5" />
                                                    </Button>
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        className="text-destructive"
                                                        onClick={() => setDeleteTarget(r.id)}
                                                        disabled={deleteMutation.isPending}
                                                        title="刪除"
                                                    >
                                                        <Trash2 className="h-3.5 w-3.5" />
                                                    </Button>
                                                </div>
                                            </TableCell>
                                        </TableRow>
                                    )
                                })
                            )}
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>

            {/* 新增/編輯對話框 */}
            <Dialog open={showAddDialog || !!editingRecord} onOpenChange={(open) => {
                if (!open) { setShowAddDialog(false); setEditingRecord(null); setForm(emptyForm) }
            }}>
                <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
                    <DialogHeader>
                        <DialogTitle>{editingRecord ? '編輯疼痛評估' : '新增疼痛評估'}</DialogTitle>
                    </DialogHeader>
                    <div className="space-y-5">
                        {/* 關聯紀錄 */}
                        {!editingRecord && (
                            <div className="space-y-2">
                                <Label>關聯紀錄 *</Label>
                                <Select
                                    value={form.record_id ? `${form.record_type}:${form.record_id}` : ''}
                                    onValueChange={(v) => {
                                        const [type, id] = v.split(':')
                                        setForm((f) => ({ ...f, record_type: type as 'observation' | 'surgery', record_id: id }))
                                    }}
                                >
                                    <SelectTrigger>
                                        <SelectValue placeholder="選擇觀察或手術紀錄..." />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {sourceOptions.length === 0 ? (
                                            <SelectItem value="__none__" disabled>尚無紀錄</SelectItem>
                                        ) : (
                                            sourceOptions.map((opt) => (
                                                <SelectItem key={`${opt.type}:${opt.value}`} value={`${opt.type}:${opt.value}`}>
                                                    {opt.label}
                                                </SelectItem>
                                            ))
                                        )}
                                    </SelectContent>
                                </Select>
                            </div>
                        )}

                        {/* 術後天數 & 時段 */}
                        <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-2">
                                <Label>術後天數</Label>
                                <Input
                                    type="number"
                                    min={0}
                                    value={form.post_op_days}
                                    onChange={(e) => setForm((f) => ({ ...f, post_op_days: e.target.value }))}
                                    placeholder="例如 0, 1, 2..."
                                />
                            </div>
                            <div className="space-y-2">
                                <Label>時段</Label>
                                <Select value={form.time_period} onValueChange={(v) => setForm((f) => ({ ...f, time_period: v }))}>
                                    <SelectTrigger>
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="AM">上午 (AM)</SelectItem>
                                        <SelectItem value="PM">下午 (PM)</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>

                        {/* 評估項目 */}
                        <div className="border rounded-md overflow-hidden">
                            <div className="bg-muted px-4 py-2 text-sm font-semibold">評估項目</div>
                            <div className="divide-y">
                                <AssessmentRow
                                    label="傷口狀況"
                                    subLabel="Incision"
                                    options={INCISION_OPTIONS}
                                    value={form.incision}
                                    onChange={(v) => setForm((f) => ({ ...f, incision: v }))}
                                />
                                <AssessmentRow
                                    label="態度/行為"
                                    subLabel="Attitude/Behavior"
                                    options={ATTITUDE_OPTIONS}
                                    value={form.attitude_behavior}
                                    onChange={(v) => setForm((f) => ({ ...f, attitude_behavior: v }))}
                                />
                                <AssessmentRow
                                    label="食慾"
                                    subLabel="Appetite"
                                    options={APPETITE_OPTIONS}
                                    value={form.appetite}
                                    onChange={(v) => setForm((f) => ({ ...f, appetite: v }))}
                                />
                                <AssessmentRow
                                    label="排便"
                                    subLabel="Feces"
                                    options={FECES_OPTIONS}
                                    value={form.feces}
                                    onChange={(v) => setForm((f) => ({ ...f, feces: v }))}
                                />
                                <AssessmentRow
                                    label="排尿"
                                    subLabel="Urine"
                                    options={URINE_OPTIONS}
                                    value={form.urine}
                                    onChange={(v) => setForm((f) => ({ ...f, urine: v }))}
                                />
                                <AssessmentRow
                                    label="疼痛分數"
                                    subLabel="Pain score"
                                    options={PAIN_SCORE_OPTIONS}
                                    value={form.pain_score}
                                    onChange={(v) => setForm((f) => ({ ...f, pain_score: v }))}
                                />
                            </div>
                        </div>

                        {/* 總分 & 疼痛分級（即時計算） */}
                        {formTotal !== null && formGrade && (
                            <div className="rounded-md border p-4 space-y-2 bg-muted/40">
                                <div className="flex items-center justify-between">
                                    <span className="text-sm font-medium">總分</span>
                                    <span className="text-2xl font-bold">{formTotal}</span>
                                </div>
                                <div className="flex items-center justify-between">
                                    <span className="text-sm font-medium">疼痛分級</span>
                                    <Badge variant={formGrade.variant} className="text-sm px-3 py-1">
                                        第{formGrade.grade}級：{formGrade.label.split('（')[0]}
                                    </Badge>
                                </div>
                                <p className="text-xs text-muted-foreground">
                                    {formGrade.grade === 1 && '不給藥，仍需持續觀察'}
                                    {formGrade.grade === 2 && '給予止痛藥'}
                                    {formGrade.grade === 3 && '每 8–12 小時給一次止痛藥'}
                                    {formGrade.grade === 4 && '每 8–12 小時給一次止痛藥並考慮合併用藥'}
                                </p>
                            </div>
                        )}

                        {/* 術後給藥 */}
                        <div className="space-y-3">
                            <Label className="text-sm font-semibold">術後給藥</Label>
                            <div className="grid grid-cols-3 gap-3 border rounded-md p-4">
                                <Checkbox
                                    label="注射 Ketorolac"
                                    checked={form.injection_ketorolac}
                                    onCheckedChange={(c) => setForm((f) => ({ ...f, injection_ketorolac: c }))}
                                />
                                <Checkbox
                                    label="注射 Meloxicam"
                                    checked={form.injection_meloxicam}
                                    onCheckedChange={(c) => setForm((f) => ({ ...f, injection_meloxicam: c }))}
                                />
                                <Checkbox
                                    label="口服 Meloxicam"
                                    checked={form.oral_meloxicam}
                                    onCheckedChange={(c) => setForm((f) => ({ ...f, oral_meloxicam: c }))}
                                />
                            </div>
                            <p className="text-xs text-muted-foreground">
                                Ketorolac (IM)：60mg/50kg↑，30mg/50kg↓ (SID/BID)　·　Meloxicam：0.1–0.4 mg/kg (SID)
                            </p>
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => { setShowAddDialog(false); setEditingRecord(null); setForm(emptyForm) }}>
                            取消
                        </Button>
                        <Button
                            onClick={handleSubmit}
                            disabled={createMutation.isPending || updateMutation.isPending}
                        >
                            {(createMutation.isPending || updateMutation.isPending) && (
                                <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                            )}
                            {editingRecord ? '更新' : '新增'}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            <DeleteReasonDialog
                open={deleteTarget !== null}
                onOpenChange={(open) => !open && setDeleteTarget(null)}
                title="刪除疼痛評估紀錄"
                description="此操作將標記紀錄為已刪除，資料將保留於系統中以符合 GLP 規範。"
                onConfirm={(reason) => deleteTarget && deleteMutation.mutate({ id: deleteTarget, reason })}
                isPending={deleteMutation.isPending}
            />
        </>
    )
}

// ── 評估項目行元件 ───────────────────────────────────────────────────────────
interface AssessmentRowProps {
    label: string
    subLabel: string
    options: Array<{ score: number; label: string }>
    value: string
    onChange: (v: string) => void
}

function AssessmentRow({ label, subLabel, options, value, onChange }: AssessmentRowProps) {
    return (
        <div className="grid grid-cols-[140px_1fr] items-center gap-3 px-4 py-3">
            <div>
                <div className="text-sm font-medium">{label}</div>
                <div className="text-xs text-muted-foreground">{subLabel}</div>
            </div>
            <Select value={value} onValueChange={onChange}>
                <SelectTrigger className="h-9">
                    <SelectValue placeholder="選擇..." />
                </SelectTrigger>
                <SelectContent>
                    {options.map((opt) => (
                        <SelectItem key={opt.score} value={String(opt.score)}>
                            <span className="inline-flex items-center gap-2">
                                <span className="inline-flex items-center justify-center w-5 h-5 rounded-full bg-muted text-xs font-bold shrink-0">
                                    {opt.score}
                                </span>
                                <span className="text-sm">{opt.label}</span>
                            </span>
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </div>
    )
}
