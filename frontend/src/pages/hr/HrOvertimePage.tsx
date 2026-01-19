import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import {
    CheckCircle,
    Clock,
    Plus,
    Send,
    Trash2,
    XCircle,
} from 'lucide-react'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { Badge } from '@/components/ui/badge'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from '@/components/ui/dialog'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import type { OvertimeWithUser } from '@/types/hr'

// Helper to safely parse Decimal strings from backend
const parseDecimal = (value: number | string | null | undefined): number => {
    if (value === null || value === undefined) return 0
    return typeof value === 'string' ? parseFloat(value) : value
}

interface PaginatedResponse<T> {
    data: T[]
    total: number
    page: number
    per_page: number
    total_pages: number
}

const OVERTIME_TYPE_NAMES: Record<string, string> = {
    A: '平日加班',
    B: '假日加班',
    C: '國定假日加班',
    D: '天災加班',
}

const OVERTIME_STATUS_NAMES: Record<string, string> = {
    draft: '草稿',
    pending: '待審核',
    approved: '已核准',
    rejected: '已駁回',
    cancelled: '已取消',
}

export function HrOvertimePage() {
    const [activeTab, setActiveTab] = useState('my-overtime')
    const [showCreateDialog, setShowCreateDialog] = useState(false)
    const queryClient = useQueryClient()

    // 新加班表單狀態
    const [overtimeDate, setOvertimeDate] = useState('')
    const [startTime, setStartTime] = useState('18:00')
    const [endTime, setEndTime] = useState('21:00')
    const [overtimeType, setOvertimeType] = useState('A')
    const [reason, setReason] = useState('')

    // 我的加班記錄
    const { data: myOvertime, isLoading: loadingOvertime } = useQuery({
        queryKey: ['hr-my-overtime'],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<OvertimeWithUser>>('/hr/overtime')
            return res.data
        },
    })

    // 待審核的加班（主管）- 總是載入以顯示待審核數量
    const { data: pendingOvertime, isLoading: loadingPending } = useQuery({
        queryKey: ['hr-pending-overtime'],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<OvertimeWithUser>>(
                '/hr/overtime?pending_approval=true'
            )
            return res.data
        },
    })

    // 建立加班
    const createOvertimeMutation = useMutation({
        mutationFn: async (data: {
            overtime_date: string
            start_time: string
            end_time: string
            overtime_type: string
            reason: string
        }) => {
            return api.post('/hr/overtime', data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-my-overtime'] })
            setShowCreateDialog(false)
            resetForm()
            toast({ title: '成功', description: '已建立加班申請' })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '建立失敗',
                variant: 'destructive',
            })
        },
    })

    // 提交加班
    const submitOvertimeMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/overtime/${id}/submit`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-my-overtime'] })
            toast({ title: '成功', description: '已送出審核' })
        },
    })

    // 核准加班
    const approveOvertimeMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/overtime/${id}/approve`, {})
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-pending-overtime'] })
            queryClient.invalidateQueries({ queryKey: ['hr-my-overtime'] })
            toast({ title: '成功', description: '已核准' })
        },
    })

    // 駁回加班
    const rejectOvertimeMutation = useMutation({
        mutationFn: async ({ id, reason }: { id: string; reason: string }) => {
            return api.post(`/hr/overtime/${id}/reject`, { reason })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-pending-overtime'] })
            toast({ title: '已駁回', description: '加班已被駁回' })
        },
    })

    // 刪除加班
    const deleteOvertimeMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.delete(`/hr/overtime/${id}`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-my-overtime'] })
            toast({ title: '成功', description: '已刪除加班申請' })
        },
    })

    const resetForm = () => {
        setOvertimeDate('')
        setStartTime('18:00')
        setEndTime('21:00')
        setOvertimeType('A')
        setReason('')
    }

    const handleCreateOvertime = () => {
        if (!overtimeDate || !startTime || !endTime || !reason) {
            toast({ title: '錯誤', description: '請填寫所有必填欄位', variant: 'destructive' })
            return
        }
        createOvertimeMutation.mutate({
            overtime_date: overtimeDate,
            start_time: startTime,
            end_time: endTime,
            overtime_type: overtimeType,
            reason,
        })
    }

    const getStatusBadge = (status: string) => {
        const statusName = OVERTIME_STATUS_NAMES[status] || status
        switch (status) {
            case 'approved':
                return <Badge className="bg-green-500">{statusName}</Badge>
            case 'rejected':
                return <Badge variant="destructive">{statusName}</Badge>
            case 'cancelled':
                return <Badge variant="secondary">{statusName}</Badge>
            case 'draft':
                return <Badge variant="outline">{statusName}</Badge>
            default:
                return <Badge>{statusName}</Badge>
        }
    }

    const formatDate = (dateStr: string) => {
        return format(new Date(dateStr), 'yyyy/MM/dd (EEEE)', { locale: zhTW })
    }

    // 計算預估補休時數 (C 和 D 固定 8 小時補休)
    const calculateCompTime = () => {
        // C (國定假日) 和 D (天災) 固定補給 8 小時 (1天) 補休
        if (overtimeType === 'C' || overtimeType === 'D') return 8.0
        return 0  // A 和 B 沒有補休時數
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold">加班管理</h1>
                    <p className="text-muted-foreground">申請加班與累積補休時數</p>
                </div>
                <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
                    <DialogTrigger asChild>
                        <Button>
                            <Plus className="h-4 w-4 mr-2" />
                            新增加班
                        </Button>
                    </DialogTrigger>
                    <DialogContent className="sm:max-w-[500px]">
                        <DialogHeader>
                            <DialogTitle>新增加班申請</DialogTitle>
                            <DialogDescription>填寫加班資訊後送出審核</DialogDescription>
                        </DialogHeader>
                        <div className="grid gap-4 py-4">
                            <div className="grid gap-2">
                                <Label>加班日期 *</Label>
                                <Input
                                    type="date"
                                    value={overtimeDate}
                                    onChange={(e) => setOvertimeDate(e.target.value)}
                                />
                            </div>
                            <div className="grid grid-cols-2 gap-4">
                                <div className="grid gap-2">
                                    <Label>開始時間 *</Label>
                                    <Input
                                        type="time"
                                        value={startTime}
                                        onChange={(e) => setStartTime(e.target.value)}
                                    />
                                </div>
                                <div className="grid gap-2">
                                    <Label>結束時間 *</Label>
                                    <Input
                                        type="time"
                                        value={endTime}
                                        onChange={(e) => setEndTime(e.target.value)}
                                    />
                                </div>
                            </div>
                            <div className="grid gap-2">
                                <Label>加班類型</Label>
                                <Select value={overtimeType} onValueChange={setOvertimeType}>
                                    <SelectTrigger>
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {Object.entries(OVERTIME_TYPE_NAMES).map(([code, name]) => (
                                            <SelectItem key={code} value={code}>
                                                {name}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                            <div className="grid gap-2">
                                <Label>加班事由 *</Label>
                                <Textarea
                                    placeholder="請說明加班原因..."
                                    value={reason}
                                    onChange={(e) => setReason(e.target.value)}
                                    rows={3}
                                />
                            </div>
                            <div className="p-3 bg-muted rounded-lg">
                                <div className="flex justify-between items-center">
                                    <span className="text-sm text-muted-foreground">預估補休時數</span>
                                    <span className="text-lg font-semibold">{calculateCompTime().toFixed(1)} 小時</span>
                                </div>
                            </div>
                        </div>
                        <DialogFooter>
                            <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
                                取消
                            </Button>
                            <Button
                                onClick={handleCreateOvertime}
                                disabled={createOvertimeMutation.isPending}
                            >
                                建立
                            </Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </div>

            <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList>
                    <TabsTrigger value="my-overtime" className="flex items-center gap-2">
                        <Clock className="h-4 w-4" />
                        我的加班
                    </TabsTrigger>
                    <TabsTrigger value="approvals" className="flex items-center gap-2">
                        <CheckCircle className="h-4 w-4" />
                        待我審核
                        {pendingOvertime && pendingOvertime.total > 0 && (
                            <Badge variant="destructive" className="ml-1">
                                {pendingOvertime.total}
                            </Badge>
                        )}
                    </TabsTrigger>
                </TabsList>

                {/* 我的加班 */}
                <TabsContent value="my-overtime" className="space-y-4">
                    <Card>
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>日期</TableHead>
                                    <TableHead>時間</TableHead>
                                    <TableHead>類型</TableHead>
                                    <TableHead>時數</TableHead>
                                    <TableHead>補休</TableHead>
                                    <TableHead>事由</TableHead>
                                    <TableHead>狀態</TableHead>
                                    <TableHead>操作</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {loadingOvertime ? (
                                    <TableRow>
                                        <TableCell colSpan={8} className="text-center py-8">
                                            載入中...
                                        </TableCell>
                                    </TableRow>
                                ) : myOvertime?.data?.length === 0 ? (
                                    <TableRow>
                                        <TableCell colSpan={8} className="text-center py-8 text-muted-foreground">
                                            沒有加班記錄
                                        </TableCell>
                                    </TableRow>
                                ) : (
                                    myOvertime?.data?.map((ot) => (
                                        <TableRow key={ot.id}>
                                            <TableCell className="whitespace-nowrap">
                                                {formatDate(ot.overtime_date)}
                                            </TableCell>
                                            <TableCell>
                                                {ot.start_time} ~ {ot.end_time}
                                            </TableCell>
                                            <TableCell>
                                                {OVERTIME_TYPE_NAMES[ot.overtime_type] || ot.overtime_type}
                                            </TableCell>
                                            <TableCell>{parseDecimal(ot.hours).toFixed(1)} 小時</TableCell>
                                            <TableCell className="text-green-600 font-medium">
                                                {parseDecimal(ot.comp_time_hours).toFixed(1)} 小時
                                            </TableCell>
                                            <TableCell className="max-w-[150px] truncate">{ot.reason}</TableCell>
                                            <TableCell>{getStatusBadge(ot.status)}</TableCell>
                                            <TableCell>
                                                <div className="flex gap-2">
                                                    {ot.status === 'draft' && (
                                                        <>
                                                            <Button
                                                                variant="default"
                                                                size="sm"
                                                                onClick={() => submitOvertimeMutation.mutate(ot.id)}
                                                                disabled={submitOvertimeMutation.isPending}
                                                            >
                                                                <Send className="h-4 w-4 mr-1" />
                                                                送審
                                                            </Button>
                                                            <Button
                                                                variant="destructive"
                                                                size="sm"
                                                                onClick={() => deleteOvertimeMutation.mutate(ot.id)}
                                                                disabled={deleteOvertimeMutation.isPending}
                                                            >
                                                                <Trash2 className="h-4 w-4" />
                                                            </Button>
                                                        </>
                                                    )}
                                                </div>
                                            </TableCell>
                                        </TableRow>
                                    ))
                                )}
                            </TableBody>
                        </Table>
                    </Card>
                </TabsContent>

                {/* 待我審核 */}
                <TabsContent value="approvals" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>待審核加班</CardTitle>
                            <CardDescription>您需要審核的加班申請</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>申請人</TableHead>
                                        <TableHead>日期</TableHead>
                                        <TableHead>時間</TableHead>
                                        <TableHead>時數</TableHead>
                                        <TableHead>事由</TableHead>
                                        <TableHead>操作</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {loadingPending ? (
                                        <TableRow>
                                            <TableCell colSpan={6} className="text-center py-8">
                                                載入中...
                                            </TableCell>
                                        </TableRow>
                                    ) : pendingOvertime?.data?.length === 0 ? (
                                        <TableRow>
                                            <TableCell colSpan={6} className="text-center py-8 text-muted-foreground">
                                                沒有待審核的加班
                                            </TableCell>
                                        </TableRow>
                                    ) : (
                                        pendingOvertime?.data?.map((ot) => (
                                            <TableRow key={ot.id}>
                                                <TableCell>
                                                    <div>
                                                        <div className="font-medium">{ot.user_name}</div>
                                                        <div className="text-sm text-muted-foreground">{ot.user_email}</div>
                                                    </div>
                                                </TableCell>
                                                <TableCell className="whitespace-nowrap">
                                                    {formatDate(ot.overtime_date)}
                                                </TableCell>
                                                <TableCell>
                                                    {ot.start_time} ~ {ot.end_time}
                                                </TableCell>
                                                <TableCell>{parseDecimal(ot.hours).toFixed(1)} 小時</TableCell>
                                                <TableCell className="max-w-[200px] truncate">{ot.reason}</TableCell>
                                                <TableCell>
                                                    <div className="flex gap-2">
                                                        <Button
                                                            variant="default"
                                                            size="sm"
                                                            onClick={() => approveOvertimeMutation.mutate(ot.id)}
                                                            disabled={approveOvertimeMutation.isPending}
                                                        >
                                                            <CheckCircle className="h-4 w-4 mr-1" />
                                                            核准
                                                        </Button>
                                                        <Button
                                                            variant="destructive"
                                                            size="sm"
                                                            onClick={() =>
                                                                rejectOvertimeMutation.mutate({ id: ot.id, reason: '不符合規定' })
                                                            }
                                                            disabled={rejectOvertimeMutation.isPending}
                                                        >
                                                            <XCircle className="h-4 w-4 mr-1" />
                                                            駁回
                                                        </Button>
                                                    </div>
                                                </TableCell>
                                            </TableRow>
                                        ))
                                    )}
                                </TableBody>
                            </Table>
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    )
}

export default HrOvertimePage
