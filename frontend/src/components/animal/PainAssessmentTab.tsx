// 疼痛評估紀錄 Tab — 照護給藥觀察
// 包含：新增表單、紀錄列表、趨勢折線圖

import { useState, lazy, Suspense } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
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
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Plus, Trash2, Edit2, Loader2, TrendingUp } from 'lucide-react'

const PainAssessmentChart = lazy(() => import('./PainAssessmentChart'))

// 疼痛評估選項
const spiritOptions = ['良好', '尚可', '不佳', '嗜睡', '興奮'] as const
const appetiteOptions = ['良好', '食量減少', '不進食', '嘔吐'] as const
const mobilityOptions = ['正常', '稍慢', '困難', '無法站立/行走'] as const
const attitudeOptions = ['正常', '焦躁', '退縮', '攻擊性', '自殘行為'] as const

// 量化分數映射（用於折線圖）
const spiritScore: Record<string, number> = { '良好': 4, '尚可': 3, '不佳': 2, '嗜睡': 1, '興奮': 3 }
const appetiteScore: Record<string, number> = { '良好': 4, '食量減少': 3, '不進食': 1, '嘔吐': 0 }
const mobilityScore: Record<string, number> = { '正常': 4, '稍慢': 3, '困難': 2, '無法站立/行走': 0 }
const attitudeScore: Record<string, number> = { '正常': 4, '焦躁': 2, '退縮': 2, '攻擊性': 1, '自殘行為': 0 }

interface CareRecord {
    id: string
    record_type: string
    record_id: string
    record_mode: string
    post_op_days: number | null
    time_period: string | null
    spirit: string | null
    appetite: string | null
    mobility_standing: string | null
    mobility_walking: string | null
    attitude_behavior: string | null
    vet_read: boolean
    created_at: string
}

interface PainAssessmentTabProps {
    animalId: string
    observations: Array<{ id: string | number; observation_date?: string }>
    surgeries: Array<{ id: string | number; surgery_date?: string }>
}

const emptyForm = {
    record_type: 'observation' as 'observation' | 'surgery',
    record_id: '',
    post_op_days: '',
    time_period: '',
    spirit: '',
    appetite: '',
    mobility_standing: '',
    mobility_walking: '',
    attitude_behavior: '',
}

export function PainAssessmentTab({ animalId, observations, surgeries }: PainAssessmentTabProps) {
    const queryClient = useQueryClient()
    const [showAddDialog, setShowAddDialog] = useState(false)
    const [editingRecord, setEditingRecord] = useState<CareRecord | null>(null)
    const [showChart, setShowChart] = useState(false)
    const [form, setForm] = useState(emptyForm)

    // 查詢紀錄
    const { data: records, isLoading } = useQuery({
        queryKey: ['animal-care-records', animalId],
        queryFn: async () => {
            const res = await api.get<CareRecord[]>(`/animals/${animalId}/care-records`)
            return res.data
        },
        staleTime: 30_000,
    })

    // 新增紀錄
    const createMutation = useMutation({
        mutationFn: async (data: typeof form) => {
            return api.post(`/animals/${animalId}/care-records`, {
                record_type: data.record_type,
                record_id: data.record_id,
                record_mode: 'pain_assessment',
                post_op_days: data.post_op_days ? parseInt(data.post_op_days) : null,
                time_period: data.time_period || null,
                spirit: data.spirit || null,
                appetite: data.appetite || null,
                mobility_standing: data.mobility_standing || null,
                mobility_walking: data.mobility_walking || null,
                attitude_behavior: data.attitude_behavior || null,
            })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-care-records', animalId] })
            toast({ title: '成功', description: '疼痛評估紀錄已新增' })
            setShowAddDialog(false)
            setForm(emptyForm)
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '新增失敗'),
                variant: 'destructive',
            })
        },
    })

    // 更新紀錄
    const updateMutation = useMutation({
        mutationFn: async ({ id, data }: { id: string; data: typeof form }) => {
            return api.put(`/care-records/${id}`, {
                post_op_days: data.post_op_days ? parseInt(data.post_op_days) : null,
                time_period: data.time_period || null,
                spirit: data.spirit || null,
                appetite: data.appetite || null,
                mobility_standing: data.mobility_standing || null,
                mobility_walking: data.mobility_walking || null,
                attitude_behavior: data.attitude_behavior || null,
            })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-care-records', animalId] })
            toast({ title: '成功', description: '疼痛評估紀錄已更新' })
            setEditingRecord(null)
            setForm(emptyForm)
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '更新失敗'),
                variant: 'destructive',
            })
        },
    })

    // 刪除紀錄
    const deleteMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.delete(`/care-records/${id}`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-care-records', animalId] })
            toast({ title: '成功', description: '紀錄已刪除' })
        },
        onError: (error: unknown) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '刪除失敗'),
                variant: 'destructive',
            })
        },
    })

    // 關聯來源選項
    const sourceOptions = [
        ...observations.map((o) => ({
            value: String(o.id),
            label: `觀察 ${o.observation_date ? new Date(o.observation_date).toLocaleDateString('zh-TW') : String(o.id).slice(0, 8)}`,
            type: 'observation' as const,
        })),
        ...surgeries.map((s) => ({
            value: String(s.id),
            label: `手術 ${s.surgery_date ? new Date(s.surgery_date).toLocaleDateString('zh-TW') : String(s.id).slice(0, 8)}`,
            type: 'surgery' as const,
        })),
    ]

    // 趨勢圖資料
    const chartData = (records || [])
        .slice()
        .sort((a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime())
        .map((r, i) => ({
            name: r.post_op_days != null ? `D${r.post_op_days}` : `#${i + 1}`,
            精神: r.spirit ? spiritScore[r.spirit] ?? 0 : null,
            食慾: r.appetite ? appetiteScore[r.appetite] ?? 0 : null,
            站立: r.mobility_standing ? mobilityScore[r.mobility_standing] ?? 0 : null,
            行走: r.mobility_walking ? mobilityScore[r.mobility_walking] ?? 0 : null,
            行為: r.attitude_behavior ? attitudeScore[r.attitude_behavior] ?? 0 : null,
        }))

    const getBadgeVariant = (val: string | null, scoreMap: Record<string, number>) => {
        if (!val) return 'outline' as const
        const s = scoreMap[val]
        if (s == null) return 'outline' as const
        if (s >= 4) return 'default' as const
        if (s >= 3) return 'secondary' as const
        return 'destructive' as const
    }

    const openEditDialog = (record: CareRecord) => {
        setEditingRecord(record)
        setForm({
            record_type: record.record_type as 'observation' | 'surgery',
            record_id: record.record_id,
            post_op_days: record.post_op_days != null ? String(record.post_op_days) : '',
            time_period: record.time_period || '',
            spirit: record.spirit || '',
            appetite: record.appetite || '',
            mobility_standing: record.mobility_standing || '',
            mobility_walking: record.mobility_walking || '',
            attitude_behavior: record.attitude_behavior || '',
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

    const formatDate = (dateStr: string) => {
        return new Date(dateStr).toLocaleDateString('zh-TW', {
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
        })
    }

    return (
        <div className="space-y-4">
            {/* 工具列 */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <Button onClick={() => { setForm(emptyForm); setEditingRecord(null); setShowAddDialog(true) }}>
                        <Plus className="h-4 w-4 mr-1" />
                        新增評估
                    </Button>
                    <Button variant="outline" onClick={() => setShowChart(!showChart)}>
                        <TrendingUp className="h-4 w-4 mr-1" />
                        {showChart ? '隱藏趨勢' : '顯示趨勢'}
                    </Button>
                </div>
                <span className="text-sm text-muted-foreground">共 {records?.length ?? 0} 筆</span>
            </div>

            {/* 趨勢折線圖 */}
            {showChart && chartData.length > 0 && (
                <Card>
                    <CardHeader className="pb-2">
                        <CardTitle className="text-base">疼痛評估趨勢</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <Suspense fallback={<div className="flex justify-center py-8"><Loader2 className="h-6 w-6 animate-spin" /></div>}>
                            <PainAssessmentChart data={chartData} />
                        </Suspense>
                    </CardContent>
                </Card>
            )}

            {/* 紀錄列表 */}
            <Card>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="w-[100px]">日期</TableHead>
                            <TableHead className="w-[60px]">術後天</TableHead>
                            <TableHead className="w-[60px]">時段</TableHead>
                            <TableHead>精神</TableHead>
                            <TableHead>食慾</TableHead>
                            <TableHead>站立</TableHead>
                            <TableHead>行走</TableHead>
                            <TableHead>行為態度</TableHead>
                            <TableHead className="w-[80px]">操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={9} className="text-center py-8">
                                    <Loader2 className="h-5 w-5 animate-spin inline-block mr-2" />
                                    載入中...
                                </TableCell>
                            </TableRow>
                        ) : !records || records.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={9} className="text-center py-8 text-muted-foreground">
                                    尚無疼痛評估紀錄
                                </TableCell>
                            </TableRow>
                        ) : (
                            records.map((r) => (
                                <TableRow key={r.id}>
                                    <TableCell className="text-xs whitespace-nowrap">{formatDate(r.created_at)}</TableCell>
                                    <TableCell>{r.post_op_days ?? '-'}</TableCell>
                                    <TableCell>{r.time_period || '-'}</TableCell>
                                    <TableCell>
                                        {r.spirit ? <Badge variant={getBadgeVariant(r.spirit, spiritScore)}>{r.spirit}</Badge> : '-'}
                                    </TableCell>
                                    <TableCell>
                                        {r.appetite ? <Badge variant={getBadgeVariant(r.appetite, appetiteScore)}>{r.appetite}</Badge> : '-'}
                                    </TableCell>
                                    <TableCell>
                                        {r.mobility_standing ? <Badge variant={getBadgeVariant(r.mobility_standing, mobilityScore)}>{r.mobility_standing}</Badge> : '-'}
                                    </TableCell>
                                    <TableCell>
                                        {r.mobility_walking ? <Badge variant={getBadgeVariant(r.mobility_walking, mobilityScore)}>{r.mobility_walking}</Badge> : '-'}
                                    </TableCell>
                                    <TableCell>
                                        {r.attitude_behavior ? <Badge variant={getBadgeVariant(r.attitude_behavior, attitudeScore)}>{r.attitude_behavior}</Badge> : '-'}
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex gap-1">
                                            <Button variant="ghost" size="icon" onClick={() => openEditDialog(r)} title="編輯">
                                                <Edit2 className="h-3.5 w-3.5" />
                                            </Button>
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                className="text-destructive"
                                                onClick={() => deleteMutation.mutate(r.id)}
                                                disabled={deleteMutation.isPending}
                                                title="刪除"
                                            >
                                                <Trash2 className="h-3.5 w-3.5" />
                                            </Button>
                                        </div>
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </Card>

            {/* 新增/編輯對話框 */}
            <Dialog open={showAddDialog || !!editingRecord} onOpenChange={(open) => {
                if (!open) { setShowAddDialog(false); setEditingRecord(null); setForm(emptyForm) }
            }}>
                <DialogContent className="max-w-lg max-h-[85vh] overflow-y-auto">
                    <DialogHeader>
                        <DialogTitle>{editingRecord ? '編輯疼痛評估' : '新增疼痛評估'}</DialogTitle>
                    </DialogHeader>
                    <div className="space-y-4">
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
                                        <SelectValue placeholder="選擇時段" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="AM">上午 (AM)</SelectItem>
                                        <SelectItem value="PM">下午 (PM)</SelectItem>
                                        <SelectItem value="NIGHT">夜間</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>

                        {/* 精神 */}
                        <div className="space-y-2">
                            <Label>精神狀況</Label>
                            <Select value={form.spirit} onValueChange={(v) => setForm((f) => ({ ...f, spirit: v }))}>
                                <SelectTrigger>
                                    <SelectValue placeholder="選擇..." />
                                </SelectTrigger>
                                <SelectContent>
                                    {spiritOptions.map((opt) => (
                                        <SelectItem key={opt} value={opt}>{opt}</SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        {/* 食慾 */}
                        <div className="space-y-2">
                            <Label>食慾</Label>
                            <Select value={form.appetite} onValueChange={(v) => setForm((f) => ({ ...f, appetite: v }))}>
                                <SelectTrigger>
                                    <SelectValue placeholder="選擇..." />
                                </SelectTrigger>
                                <SelectContent>
                                    {appetiteOptions.map((opt) => (
                                        <SelectItem key={opt} value={opt}>{opt}</SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>

                        {/* 站立/行走 */}
                        <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-2">
                                <Label>站立能力</Label>
                                <Select value={form.mobility_standing} onValueChange={(v) => setForm((f) => ({ ...f, mobility_standing: v }))}>
                                    <SelectTrigger>
                                        <SelectValue placeholder="選擇..." />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {mobilityOptions.map((opt) => (
                                            <SelectItem key={opt} value={opt}>{opt}</SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                            <div className="space-y-2">
                                <Label>行走能力</Label>
                                <Select value={form.mobility_walking} onValueChange={(v) => setForm((f) => ({ ...f, mobility_walking: v }))}>
                                    <SelectTrigger>
                                        <SelectValue placeholder="選擇..." />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {mobilityOptions.map((opt) => (
                                            <SelectItem key={opt} value={opt}>{opt}</SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>

                        {/* 行為態度 */}
                        <div className="space-y-2">
                            <Label>行為態度</Label>
                            <Select value={form.attitude_behavior} onValueChange={(v) => setForm((f) => ({ ...f, attitude_behavior: v }))}>
                                <SelectTrigger>
                                    <SelectValue placeholder="選擇..." />
                                </SelectTrigger>
                                <SelectContent>
                                    {attitudeOptions.map((opt) => (
                                        <SelectItem key={opt} value={opt}>{opt}</SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
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
        </div>
    )
}
