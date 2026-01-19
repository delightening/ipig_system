import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import {
    Calendar,
    CheckCircle,
    Clock,
    FileText,
    ImagePlus,
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
import {
    BalanceSummary,
    LeaveRequestWithUser,
    LEAVE_STATUS_NAMES,
    LEAVE_TYPE_NAMES,
} from '@/types/hr'

interface PaginatedResponse<T> {
    data: T[]
    total: number
    page: number
    per_page: number
    total_pages: number
}

interface User {
    id: string
    email: string
    display_name: string
    is_active: boolean
    roles?: string[]
}

// 工作人員簡易資訊（從 /hr/staff API 返回）
interface StaffInfo {
    id: string
    display_name: string
    email: string
}

// Helper to safely parse Decimal strings from backend
const parseDecimal = (value: number | string | null | undefined): number => {
    if (value === null || value === undefined) return 0
    return typeof value === 'string' ? parseFloat(value) : value
}

export function HrLeavePage() {
    const [activeTab, setActiveTab] = useState('my-leaves')
    const [showCreateDialog, setShowCreateDialog] = useState(false)
    const queryClient = useQueryClient()

    // 新假單表單狀態
    const [leaveType, setLeaveType] = useState('')
    const [startDate, setStartDate] = useState('')
    const [endDate, setEndDate] = useState('')
    const [totalDays, setTotalDays] = useState('1')
    const [reason, setReason] = useState('')
    const [proxyUserId, setProxyUserId] = useState('')  // 代理人
    const [supportingImages, setSupportingImages] = useState<string[]>([])  // 附件圖片 URLs
    const [uploadingImage, setUploadingImage] = useState(false)

    // 追蹤最後修改的欄位（用於決定計算方向）
    const [lastChangedField, setLastChangedField] = useState<'startDate' | 'endDate' | 'totalDays' | null>(null)

    // 我的餘額
    const { data: balanceSummary } = useQuery({
        queryKey: ['hr-balance-summary'],
        queryFn: async () => {
            const res = await api.get<BalanceSummary>('/hr/balances/summary')
            return res.data
        },
    })

    // 我的請假記錄
    const { data: myLeaves, isLoading: loadingLeaves } = useQuery({
        queryKey: ['hr-my-leaves'],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<LeaveRequestWithUser>>('/hr/leaves')
            return res.data
        },
    })

    // 待審核的請假（主管）- 始終載入以顯示待審核數量
    const { data: pendingLeaves, isLoading: loadingPending } = useQuery({
        queryKey: ['hr-pending-leaves'],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<LeaveRequestWithUser>>(
                '/hr/leaves?pending_approval=true'
            )
            return res.data
        },
        // 移除 enabled 條件，讓頁面載入時就抓取待審核數量
    })

    // 取得試驗工作人員列表（供代理人選擇）
    // 使用專用 API，不需要 dev.user.view 權限
    const { data: staffList } = useQuery({
        queryKey: ['hr-staff-for-proxy'],
        queryFn: async () => {
            const res = await api.get<StaffInfo[]>('/hr/staff')
            return res.data
        },
    })

    // 日期/天數處理函式
    const handleStartDateChange = (value: string) => {
        setStartDate(value)
        setLastChangedField('startDate')
    }

    const handleEndDateChange = (value: string) => {
        setEndDate(value)
        setLastChangedField('endDate')
    }

    const handleTotalDaysChange = (value: string) => {
        setTotalDays(value)
        setLastChangedField('totalDays')
    }

    // 自動計算日期/天數（雙向計算）
    useEffect(() => {
        if (!lastChangedField) return

        // 如果修改的是開始日期或結束日期 → 計算天數
        if ((lastChangedField === 'startDate' || lastChangedField === 'endDate') && startDate && endDate) {
            const start = new Date(startDate)
            const end = new Date(endDate)
            if (end >= start) {
                const diffTime = end.getTime() - start.getTime()
                const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24)) + 1
                setTotalDays(String(diffDays))
            }
        }

        // 如果修改的是天數 → 計算結束日期
        if (lastChangedField === 'totalDays' && startDate && totalDays) {
            const days = parseFloat(totalDays)
            if (days >= 0.5) {
                const start = new Date(startDate)
                // 天數 - 1 是因為開始日期算第一天
                const end = new Date(start.getTime() + (Math.ceil(days) - 1) * 24 * 60 * 60 * 1000)
                const newEndDate = end.toISOString().split('T')[0]
                if (newEndDate !== endDate) {
                    setEndDate(newEndDate)
                }
            }
        }
    }, [startDate, endDate, totalDays, lastChangedField])

    // 建立請假
    const createLeaveMutation = useMutation({
        mutationFn: async (data: {
            leave_type: string
            start_date: string
            end_date: string
            total_days: number
            reason?: string
            supporting_documents?: string[]
            proxy_user_id?: string
        }) => {
            return api.post('/hr/leaves', data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-my-leaves'] })
            setShowCreateDialog(false)
            resetForm()
            toast({ title: '成功', description: '已建立請假申請' })
        },
        onError: (error: any) => {
            toast({
                title: '錯誤',
                description: error?.response?.data?.error?.message || '建立失敗',
                variant: 'destructive',
            })
        },
    })

    // 提交請假
    const submitLeaveMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/leaves/${id}/submit`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-my-leaves'] })
            toast({ title: '成功', description: '已送出審核' })
        },
    })

    // 核准請假
    const approveLeaveMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/leaves/${id}/approve`, {})
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-pending-leaves'] })
            queryClient.invalidateQueries({ queryKey: ['hr-my-leaves'] })
            toast({ title: '成功', description: '已核准' })
        },
    })

    // 駁回請假
    const rejectLeaveMutation = useMutation({
        mutationFn: async ({ id, reason }: { id: string; reason: string }) => {
            return api.post(`/hr/leaves/${id}/reject`, { reason })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-pending-leaves'] })
            toast({ title: '已駁回', description: '請假已被駁回' })
        },
    })

    // 取消請假
    const cancelLeaveMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/leaves/${id}/cancel`, {})
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-my-leaves'] })
            toast({ title: '成功', description: '已取消請假' })
        },
    })

    const resetForm = () => {
        setLeaveType('')
        setStartDate('')
        setEndDate('')
        setTotalDays('1')
        setReason('')
        setProxyUserId('')
        setSupportingImages([])
    }

    // 處理圖片上傳 (上傳至後端)
    const handleImageUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const files = e.target.files
        if (!files || files.length === 0) return

        setUploadingImage(true)
        try {
            const formData = new FormData()
            for (let i = 0; i < files.length; i++) {
                formData.append('files', files[i])
            }
            const res = await api.post<{ id: string; file_path: string }[]>('/hr/leaves/attachments', formData, {
                headers: { 'Content-Type': 'multipart/form-data' }
            })
            // 使用 file_path 作為圖片 URL
            const newUrls = res.data.map(r => `/api/uploads/${r.file_path}`)
            setSupportingImages(prev => [...prev, ...newUrls])
            toast({ title: '成功', description: '圖片已上傳' })
        } catch {
            toast({ title: '錯誤', description: '圖片上傳失敗', variant: 'destructive' })
        } finally {
            setUploadingImage(false)
            e.target.value = ''
        }
    }

    // 移除已上傳的圖片
    const removeImage = (index: number) => {
        setSupportingImages(prev => prev.filter((_, i) => i !== index))
    }

    // 檢查是否為特休假（不需要填寫理由）
    const isAnnualLeave = leaveType === 'ANNUAL'

    const handleCreateLeave = () => {
        // 特休假不需要填寫理由，其他假別需要
        if (!leaveType || !startDate || !endDate) {
            toast({ title: '錯誤', description: '請填寫必填欄位', variant: 'destructive' })
            return
        }
        if (!isAnnualLeave && !reason.trim()) {
            toast({ title: '錯誤', description: '請填寫請假事由', variant: 'destructive' })
            return
        }
        createLeaveMutation.mutate({
            leave_type: leaveType,
            start_date: startDate,
            end_date: endDate,
            total_days: parseFloat(totalDays),
            reason: reason.trim() || undefined,
            supporting_documents: supportingImages.length > 0 ? supportingImages : undefined,
            proxy_user_id: proxyUserId && proxyUserId !== '__none__' ? proxyUserId : undefined,
        })
    }

    const getStatusBadge = (status: string) => {
        const statusName = LEAVE_STATUS_NAMES[status] || status
        switch (status) {
            case 'APPROVED':
                return <Badge className="bg-green-500">{statusName}</Badge>
            case 'REJECTED':
                return <Badge variant="destructive">{statusName}</Badge>
            case 'CANCELLED':
                return <Badge variant="secondary">{statusName}</Badge>
            case 'DRAFT':
                return <Badge variant="outline">{statusName}</Badge>
            default:
                return <Badge>{statusName}</Badge>
        }
    }

    const formatDate = (dateStr: string) => {
        return format(new Date(dateStr), 'yyyy/MM/dd', { locale: zhTW })
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold">請假管理</h1>
                    <p className="text-muted-foreground">申請請假與查看假期餘額</p>
                </div>
                <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
                    <DialogTrigger asChild>
                        <Button>
                            <Plus className="h-4 w-4 mr-2" />
                            新增請假
                        </Button>
                    </DialogTrigger>
                    <DialogContent className="sm:max-w-[500px]">
                        <DialogHeader>
                            <DialogTitle>新增請假申請</DialogTitle>
                            <DialogDescription>填寫請假資訊後送出審核</DialogDescription>
                        </DialogHeader>
                        <div className="grid gap-4 py-4">
                            <div className="grid gap-2">
                                <Label>假別 *</Label>
                                <Select value={leaveType} onValueChange={setLeaveType}>
                                    <SelectTrigger>
                                        <SelectValue placeholder="選擇假別" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {Object.entries(LEAVE_TYPE_NAMES).map(([code, name]) => (
                                            <SelectItem key={code} value={code}>
                                                {name}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                            <div className="grid grid-cols-2 gap-4">
                                <div className="grid gap-2">
                                    <Label>開始日期 *</Label>
                                    <Input
                                        type="date"
                                        value={startDate}
                                        onChange={(e) => handleStartDateChange(e.target.value)}
                                    />
                                </div>
                                <div className="grid gap-2">
                                    <Label>結束日期 *</Label>
                                    <Input
                                        type="date"
                                        value={endDate}
                                        onChange={(e) => handleEndDateChange(e.target.value)}
                                    />
                                </div>
                            </div>
                            <div className="grid gap-2">
                                <Label>天數</Label>
                                <Input
                                    type="number"
                                    step="0.5"
                                    min="0.5"
                                    value={totalDays}
                                    onChange={(e) => handleTotalDaysChange(e.target.value)}
                                />
                            </div>
                            <div className="grid gap-2">
                                <Label>代理人 <span className="text-muted-foreground text-xs">(選填)</span></Label>
                                <Select value={proxyUserId} onValueChange={setProxyUserId}>
                                    <SelectTrigger>
                                        <SelectValue placeholder="選擇代理人..." />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="__none__">不選擇</SelectItem>
                                        {staffList?.map((staff) => (
                                            <SelectItem key={staff.id} value={staff.id}>
                                                {staff.display_name}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                            <div className="grid gap-2">
                                <Label>
                                    請假事由 {!isAnnualLeave && '*'}
                                    {isAnnualLeave && <span className="text-muted-foreground text-xs ml-1">(選填)</span>}
                                </Label>
                                <Textarea
                                    placeholder={isAnnualLeave ? "選填，可不填寫..." : "請說明請假原因..."}
                                    value={reason}
                                    onChange={(e) => setReason(e.target.value)}
                                    rows={3}
                                />
                            </div>

                            {/* 圖片附件上傳 */}
                            <div className="grid gap-2">
                                <Label>附件圖片 <span className="text-muted-foreground text-xs">(選填)</span></Label>
                                <div className="flex flex-wrap gap-2">
                                    {supportingImages.map((url, index) => (
                                        <div key={index} className="relative group">
                                            <img
                                                src={url}
                                                alt={`附件 ${index + 1}`}
                                                className="h-16 w-16 object-cover rounded border"
                                            />
                                            <button
                                                type="button"
                                                onClick={() => removeImage(index)}
                                                className="absolute -top-2 -right-2 bg-destructive text-destructive-foreground rounded-full w-5 h-5 flex items-center justify-center text-xs opacity-0 group-hover:opacity-100 transition-opacity"
                                            >
                                                ×
                                            </button>
                                        </div>
                                    ))}
                                    <label className="h-16 w-16 border-2 border-dashed rounded flex items-center justify-center cursor-pointer hover:border-primary hover:bg-muted/50 transition-colors">
                                        <input
                                            type="file"
                                            accept="image/*"
                                            multiple
                                            className="hidden"
                                            onChange={handleImageUpload}
                                            disabled={uploadingImage}
                                        />
                                        {uploadingImage ? (
                                            <Clock className="h-5 w-5 animate-spin text-muted-foreground" />
                                        ) : (
                                            <ImagePlus className="h-5 w-5 text-muted-foreground" />
                                        )}
                                    </label>
                                </div>
                                <p className="text-xs text-muted-foreground">
                                    可上傳診斷證明、相關文件等
                                </p>
                            </div>
                        </div>
                        <DialogFooter>
                            <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
                                取消
                            </Button>
                            <Button
                                onClick={handleCreateLeave}
                                disabled={createLeaveMutation.isPending}
                            >
                                建立
                            </Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </div>

            {/* 餘額摘要 */}
            <div className="grid gap-4 md:grid-cols-4">
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">特休剩餘</CardTitle>
                        <Calendar className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">
                            {balanceSummary?.annual_leave_remaining ?? 0} 天
                        </div>
                        <p className="text-xs text-muted-foreground">
                            已使用 {balanceSummary?.annual_leave_used ?? 0} /{' '}
                            {balanceSummary?.annual_leave_total ?? 0} 天
                        </p>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">補休剩餘</CardTitle>
                        <Clock className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">
                            {parseDecimal(balanceSummary?.comp_time_remaining).toFixed(1)} 小時
                        </div>
                        <p className="text-xs text-muted-foreground">
                            已使用 {parseDecimal(balanceSummary?.comp_time_used).toFixed(1)} 小時
                        </p>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">即將到期（特休）</CardTitle>
                        <Clock className="h-4 w-4 text-orange-500" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold text-orange-500">
                            {balanceSummary?.expiring_soon_days ?? 0} 天
                        </div>
                        <p className="text-xs text-muted-foreground">30 天內到期</p>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">即將到期（補休）</CardTitle>
                        <Clock className="h-4 w-4 text-orange-500" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold text-orange-500">
                            {parseDecimal(balanceSummary?.expiring_soon_hours).toFixed(1)} 小時
                        </div>
                        <p className="text-xs text-muted-foreground">30 天內到期</p>
                    </CardContent>
                </Card>
            </div>

            <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList>
                    <TabsTrigger value="my-leaves" className="flex items-center gap-2">
                        <FileText className="h-4 w-4" />
                        我的請假
                    </TabsTrigger>
                    <TabsTrigger value="approvals" className="flex items-center gap-2">
                        <CheckCircle className="h-4 w-4" />
                        待我審核
                        {pendingLeaves && pendingLeaves.total > 0 && (
                            <Badge variant="destructive" className="ml-1">
                                {pendingLeaves.total}
                            </Badge>
                        )}
                    </TabsTrigger>
                </TabsList>

                {/* 我的請假 */}
                <TabsContent value="my-leaves" className="space-y-4">
                    <Card>
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>假別</TableHead>
                                    <TableHead>日期</TableHead>
                                    <TableHead>天數</TableHead>
                                    <TableHead>事由</TableHead>
                                    <TableHead>狀態</TableHead>
                                    <TableHead>操作</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {loadingLeaves ? (
                                    <TableRow>
                                        <TableCell colSpan={6} className="text-center py-8">
                                            載入中...
                                        </TableCell>
                                    </TableRow>
                                ) : myLeaves?.data?.length === 0 ? (
                                    <TableRow>
                                        <TableCell colSpan={6} className="text-center py-8 text-muted-foreground">
                                            沒有請假記錄
                                        </TableCell>
                                    </TableRow>
                                ) : (
                                    myLeaves?.data?.map((leave) => (
                                        <TableRow key={leave.id}>
                                            <TableCell>{LEAVE_TYPE_NAMES[leave.leave_type] || leave.leave_type}</TableCell>
                                            <TableCell className="whitespace-nowrap">
                                                {formatDate(leave.start_date)}
                                                {leave.start_date !== leave.end_date && ` ~ ${formatDate(leave.end_date)}`}
                                            </TableCell>
                                            <TableCell>{leave.total_days} 天</TableCell>
                                            <TableCell className="max-w-[200px] truncate">{leave.reason}</TableCell>
                                            <TableCell>{getStatusBadge(leave.status)}</TableCell>
                                            <TableCell>
                                                <div className="flex gap-2">
                                                    {leave.status === 'DRAFT' && (
                                                        <>
                                                            <Button
                                                                variant="default"
                                                                size="sm"
                                                                onClick={() => submitLeaveMutation.mutate(leave.id)}
                                                                disabled={submitLeaveMutation.isPending}
                                                            >
                                                                <Send className="h-4 w-4 mr-1" />
                                                                送審
                                                            </Button>
                                                            <Button
                                                                variant="destructive"
                                                                size="sm"
                                                                onClick={() => cancelLeaveMutation.mutate(leave.id)}
                                                                disabled={cancelLeaveMutation.isPending}
                                                            >
                                                                <Trash2 className="h-4 w-4" />
                                                            </Button>
                                                        </>
                                                    )}
                                                    {(leave.status.startsWith('PENDING') || leave.status === 'APPROVED') && (
                                                        <Button
                                                            variant="outline"
                                                            size="sm"
                                                            onClick={() => cancelLeaveMutation.mutate(leave.id)}
                                                            disabled={cancelLeaveMutation.isPending}
                                                        >
                                                            取消
                                                        </Button>
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
                            <CardTitle>待審核請假</CardTitle>
                            <CardDescription>您需要審核的請假申請</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>申請人</TableHead>
                                        <TableHead>假別</TableHead>
                                        <TableHead>日期</TableHead>
                                        <TableHead>天數</TableHead>
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
                                    ) : pendingLeaves?.data?.length === 0 ? (
                                        <TableRow>
                                            <TableCell colSpan={6} className="text-center py-8 text-muted-foreground">
                                                沒有待審核的請假
                                            </TableCell>
                                        </TableRow>
                                    ) : (
                                        pendingLeaves?.data?.map((leave) => (
                                            <TableRow key={leave.id}>
                                                <TableCell>
                                                    <div>
                                                        <div className="font-medium">{leave.user_name}</div>
                                                        <div className="text-sm text-muted-foreground">{leave.user_email}</div>
                                                    </div>
                                                </TableCell>
                                                <TableCell>{LEAVE_TYPE_NAMES[leave.leave_type] || leave.leave_type}</TableCell>
                                                <TableCell className="whitespace-nowrap">
                                                    {formatDate(leave.start_date)}
                                                    {leave.start_date !== leave.end_date && ` ~ ${formatDate(leave.end_date)}`}
                                                </TableCell>
                                                <TableCell>{leave.total_days} 天</TableCell>
                                                <TableCell className="max-w-[200px] truncate">{leave.reason}</TableCell>
                                                <TableCell>
                                                    <div className="flex gap-2">
                                                        <Button
                                                            variant="default"
                                                            size="sm"
                                                            onClick={() => approveLeaveMutation.mutate(leave.id)}
                                                            disabled={approveLeaveMutation.isPending}
                                                        >
                                                            <CheckCircle className="h-4 w-4 mr-1" />
                                                            核准
                                                        </Button>
                                                        <Button
                                                            variant="destructive"
                                                            size="sm"
                                                            onClick={() =>
                                                                rejectLeaveMutation.mutate({ id: leave.id, reason: '不符合規定' })
                                                            }
                                                            disabled={rejectLeaveMutation.isPending}
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

export default HrLeavePage
