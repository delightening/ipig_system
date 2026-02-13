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
    Users,
    XCircle,
} from 'lucide-react'
import api from '@/lib/api'
import { useAuthStore } from '@/stores/auth'
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

// ============================================
// Types & Constants
// ============================================

interface PaginatedResponse<T> {
    data: T[]
    total: number
    page: number
    per_page: number
    total_pages: number
}

interface CreateOvertimeData {
    overtime_date: string
    start_time: string
    end_time: string
    overtime_type: string
    reason: string
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
    pending_admin_staff: '待行政審核',
    pending_admin: '待負責人審核',
    approved: '已核准',
    rejected: '已駁回',
    cancelled: '已取消',
}

// ============================================
// Utility Functions
// ============================================

/** Safely parse Decimal strings from backend */
const parseDecimal = (value: number | string | null | undefined): number => {
    if (value === null || value === undefined) return 0
    return typeof value === 'string' ? parseFloat(value) : value
}

/** Format date string to localized format */
const formatDate = (dateStr: string): string => {
    return format(new Date(dateStr), 'yyyy/MM/dd (EEEE)', { locale: zhTW })
}

/** Get badge component for overtime status */
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
        case 'pending_admin_staff':
            return <Badge className="bg-yellow-500">{statusName}</Badge>
        case 'pending_admin':
            return <Badge className="bg-orange-500">{statusName}</Badge>
        default:
            return <Badge>{statusName}</Badge>
    }
}

/**
 * Calculate estimated comp time hours
 * C (國定假日) and D (天災) fixed 8 hours comp time
 * A and B have no comp time
 */
const calculateCompTime = (overtimeType: string): number => {
    if (overtimeType === 'C' || overtimeType === 'D') return 8.0
    return 0
}

// ============================================
// Custom Hooks
// ============================================

/** Hook for fetching my overtime records */
const useMyOvertime = () => {
    return useQuery({
        queryKey: ['hr-my-overtime'],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<OvertimeWithUser>>('/hr/overtime')
            return res.data
        },
    })
}

/** Hook for fetching pending overtime approvals */
const usePendingOvertime = () => {
    return useQuery({
        queryKey: ['hr-pending-overtime'],
        queryFn: async () => {
            const res = await api.get<PaginatedResponse<OvertimeWithUser>>(
                '/hr/overtime?pending_approval=true'
            )
            return res.data
        },
    })
}

/** Hook for overtime mutations (CRUD operations) */
const useOvertimeMutations = () => {
    const queryClient = useQueryClient()

    const createOvertime = useMutation({
        mutationFn: async (data: CreateOvertimeData) => {
            return api.post('/hr/overtime', data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-my-overtime'] })
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

    const submitOvertime = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/overtime/${id}/submit`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-my-overtime'] })
            toast({ title: '成功', description: '已送出審核' })
        },
    })

    const approveOvertime = useMutation({
        mutationFn: async (id: string) => {
            return api.post(`/hr/overtime/${id}/approve`, {})
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-pending-overtime'] })
            queryClient.invalidateQueries({ queryKey: ['hr-my-overtime'] })
            toast({ title: '成功', description: '已核准' })
        },
    })

    const rejectOvertime = useMutation({
        mutationFn: async ({ id, reason }: { id: string; reason: string }) => {
            return api.post(`/hr/overtime/${id}/reject`, { reason })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-pending-overtime'] })
            toast({ title: '已駁回', description: '加班已被駁回' })
        },
    })

    const deleteOvertime = useMutation({
        mutationFn: async (id: string) => {
            return api.delete(`/hr/overtime/${id}`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['hr-my-overtime'] })
            toast({ title: '成功', description: '已刪除加班申請' })
        },
    })

    return {
        createOvertime,
        submitOvertime,
        approveOvertime,
        rejectOvertime,
        deleteOvertime,
    }
}

// ============================================
// Sub-Components
// ============================================

interface CreateOvertimeDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    onSubmit: (data: CreateOvertimeData) => void
    isPending: boolean
}

function CreateOvertimeDialog({
    open,
    onOpenChange,
    onSubmit,
    isPending,
}: CreateOvertimeDialogProps) {
    const [overtimeDate, setOvertimeDate] = useState('')
    const [startTime, setStartTime] = useState('18:00')
    const [endTime, setEndTime] = useState('21:00')
    const [overtimeType, setOvertimeType] = useState('A')
    const [reason, setReason] = useState('')

    const resetForm = () => {
        setOvertimeDate('')
        setStartTime('18:00')
        setEndTime('21:00')
        setOvertimeType('A')
        setReason('')
    }

    const handleSubmit = () => {
        if (!overtimeDate || !startTime || !endTime || !reason) {
            toast({ title: '錯誤', description: '請填寫所有必填欄位', variant: 'destructive' })
            return
        }
        onSubmit({
            overtime_date: overtimeDate,
            start_time: startTime,
            end_time: endTime,
            overtime_type: overtimeType,
            reason,
        })
        resetForm()
    }

    const handleOpenChange = (newOpen: boolean) => {
        if (!newOpen) resetForm()
        onOpenChange(newOpen)
    }

    return (
        <Dialog open={open} onOpenChange={handleOpenChange}>
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
                            <span className="text-lg font-semibold">
                                {calculateCompTime(overtimeType).toFixed(1)} 小時
                            </span>
                        </div>
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => handleOpenChange(false)}>
                        取消
                    </Button>
                    <Button onClick={handleSubmit} disabled={isPending}>
                        建立
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}

interface OvertimeTableRowProps {
    overtime: OvertimeWithUser
    onSubmit: (id: string) => void
    onDelete: (id: string) => void
    isSubmitting: boolean
    isDeleting: boolean
}

function MyOvertimeTableRow({
    overtime,
    onSubmit,
    onDelete,
    isSubmitting,
    isDeleting,
}: OvertimeTableRowProps) {
    return (
        <TableRow>
            <TableCell className="whitespace-nowrap">
                {formatDate(overtime.overtime_date)}
            </TableCell>
            <TableCell>
                {overtime.start_time} ~ {overtime.end_time}
            </TableCell>
            <TableCell>
                {OVERTIME_TYPE_NAMES[overtime.overtime_type] || overtime.overtime_type}
            </TableCell>
            <TableCell>{parseDecimal(overtime.hours).toFixed(1)} 小時</TableCell>
            <TableCell className="text-green-600 font-medium">
                {parseDecimal(overtime.comp_time_hours).toFixed(1)} 小時
            </TableCell>
            <TableCell className="max-w-[150px] truncate">{overtime.reason}</TableCell>
            <TableCell>{getStatusBadge(overtime.status)}</TableCell>
            <TableCell>
                <div className="flex gap-2">
                    {overtime.status === 'draft' && (
                        <>
                            <Button
                                variant="default"
                                size="sm"
                                onClick={() => onSubmit(overtime.id)}
                                disabled={isSubmitting}
                            >
                                <Send className="h-4 w-4 mr-1" />
                                送審
                            </Button>
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => onDelete(overtime.id)}
                                disabled={isDeleting}
                            >
                                <Trash2 className="h-4 w-4" />
                            </Button>
                        </>
                    )}
                </div>
            </TableCell>
        </TableRow>
    )
}

interface PendingApprovalRowProps {
    overtime: OvertimeWithUser
    onApprove: (id: string) => void
    onReject: (id: string, reason: string) => void
    isApproving: boolean
    isRejecting: boolean
}

function PendingApprovalRow({
    overtime,
    onApprove,
    onReject,
    isApproving,
    isRejecting,
}: PendingApprovalRowProps) {
    return (
        <TableRow>
            <TableCell>
                <div>
                    <div className="font-medium">{overtime.user_name}</div>
                    <div className="text-sm text-muted-foreground">{overtime.user_email}</div>
                </div>
            </TableCell>
            <TableCell className="whitespace-nowrap">
                {formatDate(overtime.overtime_date)}
            </TableCell>
            <TableCell>
                {overtime.start_time} ~ {overtime.end_time}
            </TableCell>
            <TableCell>{parseDecimal(overtime.hours).toFixed(1)} 小時</TableCell>
            <TableCell className="max-w-[200px] truncate">{overtime.reason}</TableCell>
            <TableCell>
                <div className="flex gap-2">
                    <Button
                        variant="default"
                        size="sm"
                        onClick={() => onApprove(overtime.id)}
                        disabled={isApproving}
                    >
                        <CheckCircle className="h-4 w-4 mr-1" />
                        核准
                    </Button>
                    <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => onReject(overtime.id, '不符合規定')}
                        disabled={isRejecting}
                    >
                        <XCircle className="h-4 w-4 mr-1" />
                        駁回
                    </Button>
                </div>
            </TableCell>
        </TableRow>
    )
}

interface MyOvertimeTabContentProps {
    overtimeData: PaginatedResponse<OvertimeWithUser> | undefined
    isLoading: boolean
    onSubmit: (id: string) => void
    onDelete: (id: string) => void
    isSubmitting: boolean
    isDeleting: boolean
}

function MyOvertimeTabContent({
    overtimeData,
    isLoading,
    onSubmit,
    onDelete,
    isSubmitting,
    isDeleting,
}: MyOvertimeTabContentProps) {
    return (
        <TabsContent value="my-overtime" className="space-y-4">
            <Card>
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>日期</TableHead>
                            <TableHead>時間</TableHead>
                            <TableHead>類型</TableHead>
                            <TableHead>加班時數</TableHead>
                            <TableHead>補休</TableHead>
                            <TableHead>事由</TableHead>
                            <TableHead>狀態</TableHead>
                            <TableHead>操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableRow>
                                <TableCell colSpan={8} className="text-center py-8">
                                    載入中...
                                </TableCell>
                            </TableRow>
                        ) : overtimeData?.data?.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={8} className="text-center py-8 text-muted-foreground">
                                    沒有加班記錄
                                </TableCell>
                            </TableRow>
                        ) : (
                            overtimeData?.data?.map((ot) => (
                                <MyOvertimeTableRow
                                    key={ot.id}
                                    overtime={ot}
                                    onSubmit={onSubmit}
                                    onDelete={onDelete}
                                    isSubmitting={isSubmitting}
                                    isDeleting={isDeleting}
                                />
                            ))
                        )}
                    </TableBody>
                </Table>
            </Card>
        </TabsContent>
    )
}

interface PendingApprovalsTabContentProps {
    pendingData: PaginatedResponse<OvertimeWithUser> | undefined
    isLoading: boolean
    onApprove: (id: string) => void
    onReject: (id: string, reason: string) => void
    isApproving: boolean
    isRejecting: boolean
}

function PendingApprovalsTabContent({
    pendingData,
    isLoading,
    onApprove,
    onReject,
    isApproving,
    isRejecting,
}: PendingApprovalsTabContentProps) {
    return (
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
                                <TableHead>加班時數</TableHead>
                                <TableHead>事由</TableHead>
                                <TableHead>操作</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {isLoading ? (
                                <TableRow>
                                    <TableCell colSpan={6} className="text-center py-8">
                                        載入中...
                                    </TableCell>
                                </TableRow>
                            ) : pendingData?.data?.length === 0 ? (
                                <TableRow>
                                    <TableCell colSpan={6} className="text-center py-8 text-muted-foreground">
                                        沒有待審核的加班
                                    </TableCell>
                                </TableRow>
                            ) : (
                                pendingData?.data?.map((ot) => (
                                    <PendingApprovalRow
                                        key={ot.id}
                                        overtime={ot}
                                        onApprove={onApprove}
                                        onReject={onReject}
                                        isApproving={isApproving}
                                        isRejecting={isRejecting}
                                    />
                                ))
                            )}
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>
        </TabsContent>
    )
}

// ============================================
// Main Component
// ============================================

export function HrOvertimePage() {
    const [activeTab, setActiveTab] = useState('my-overtime')
    const [showCreateDialog, setShowCreateDialog] = useState(false)
    const { hasRole } = useAuthStore()

    // 角色判斷：admin 或 ADMIN_STAFF 可看所有紀錄
    const canViewAll = hasRole('admin') || hasRole('ADMIN_STAFF')

    // 全部紀錄篩選狀態
    const [filterStatus, setFilterStatus] = useState<string>('all')
    const [filterOvertimeType, setFilterOvertimeType] = useState<string>('all')
    const [filterFrom, setFilterFrom] = useState('')
    const [filterTo, setFilterTo] = useState('')

    // Data fetching
    const { data: myOvertime, isLoading: loadingOvertime } = useMyOvertime()
    const { data: pendingOvertime, isLoading: loadingPending } = usePendingOvertime()

    // 所有員工的加班紀錄（admin/ADMIN_STAFF 專用）
    const { data: allOvertime, isLoading: loadingAllOvertime } = useQuery({
        queryKey: ['hr-all-overtime', filterStatus, filterOvertimeType, filterFrom, filterTo],
        queryFn: async () => {
            const params = new URLSearchParams({ view_all: 'true' })
            if (filterStatus !== 'all') params.append('status', filterStatus)
            if (filterFrom) params.append('from', filterFrom)
            if (filterTo) params.append('to', filterTo)
            const res = await api.get<PaginatedResponse<OvertimeWithUser>>(`/hr/overtime?${params.toString()}`)
            return res.data
        },
        enabled: canViewAll && activeTab === 'all-records',
    })

    // Mutations
    const {
        createOvertime,
        submitOvertime,
        approveOvertime,
        rejectOvertime,
        deleteOvertime,
    } = useOvertimeMutations()

    // Handlers
    const handleCreateOvertime = (data: CreateOvertimeData) => {
        createOvertime.mutate(data, {
            onSuccess: () => setShowCreateDialog(false),
        })
    }

    const handleSubmitOvertime = (id: string) => {
        submitOvertime.mutate(id)
    }

    const handleDeleteOvertime = (id: string) => {
        deleteOvertime.mutate(id)
    }

    const handleApproveOvertime = (id: string) => {
        approveOvertime.mutate(id)
    }

    const handleRejectOvertime = (id: string, reason: string) => {
        rejectOvertime.mutate({ id, reason })
    }

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold">加班管理</h1>
                    <p className="text-muted-foreground">申請加班與累積補休時數</p>
                </div>
                <CreateOvertimeDialog
                    open={showCreateDialog}
                    onOpenChange={setShowCreateDialog}
                    onSubmit={handleCreateOvertime}
                    isPending={createOvertime.isPending}
                />
            </div>

            {/* Tabs */}
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
                    {canViewAll && (
                        <TabsTrigger value="all-records" className="flex items-center gap-2">
                            <Users className="h-4 w-4" />
                            加班紀錄
                        </TabsTrigger>
                    )}
                </TabsList>

                {/* My Overtime Tab */}
                <MyOvertimeTabContent
                    overtimeData={myOvertime}
                    isLoading={loadingOvertime}
                    onSubmit={handleSubmitOvertime}
                    onDelete={handleDeleteOvertime}
                    isSubmitting={submitOvertime.isPending}
                    isDeleting={deleteOvertime.isPending}
                />

                {/* Pending Approvals Tab */}
                <PendingApprovalsTabContent
                    pendingData={pendingOvertime}
                    isLoading={loadingPending}
                    onApprove={handleApproveOvertime}
                    onReject={handleRejectOvertime}
                    isApproving={approveOvertime.isPending}
                    isRejecting={rejectOvertime.isPending}
                />

                {/* 所有員工加班紀錄（admin/ADMIN_STAFF） */}
                {canViewAll && (
                    <TabsContent value="all-records" className="space-y-4">
                        <Card>
                            <CardHeader>
                                <CardTitle>全部加班紀錄</CardTitle>
                                <CardDescription>查看所有員工的加班資料</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                {/* 篩選列 */}
                                <div className="flex flex-wrap gap-3 items-end">
                                    <div className="grid gap-1">
                                        <Label className="text-xs">狀態</Label>
                                        <Select value={filterStatus} onValueChange={setFilterStatus}>
                                            <SelectTrigger className="w-[140px]">
                                                <SelectValue />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="all">全部狀態</SelectItem>
                                                {Object.entries(OVERTIME_STATUS_NAMES).map(([code, name]) => (
                                                    <SelectItem key={code} value={code}>{name}</SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    </div>
                                    <div className="grid gap-1">
                                        <Label className="text-xs">加班類型</Label>
                                        <Select value={filterOvertimeType} onValueChange={setFilterOvertimeType}>
                                            <SelectTrigger className="w-[140px]">
                                                <SelectValue />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="all">全部類型</SelectItem>
                                                {Object.entries(OVERTIME_TYPE_NAMES).map(([code, name]) => (
                                                    <SelectItem key={code} value={code}>{name}</SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    </div>
                                    <div className="grid gap-1">
                                        <Label className="text-xs">起始日期</Label>
                                        <Input
                                            type="date"
                                            value={filterFrom}
                                            onChange={(e) => setFilterFrom(e.target.value)}
                                            className="w-[160px]"
                                        />
                                    </div>
                                    <div className="grid gap-1">
                                        <Label className="text-xs">結束日期</Label>
                                        <Input
                                            type="date"
                                            value={filterTo}
                                            onChange={(e) => setFilterTo(e.target.value)}
                                            className="w-[160px]"
                                        />
                                    </div>
                                    {(filterStatus !== 'all' || filterOvertimeType !== 'all' || filterFrom || filterTo) && (
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            onClick={() => {
                                                setFilterStatus('all')
                                                setFilterOvertimeType('all')
                                                setFilterFrom('')
                                                setFilterTo('')
                                            }}
                                        >
                                            清除篩選
                                        </Button>
                                    )}
                                </div>

                                {/* 表格 */}
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHead>申請人</TableHead>
                                            <TableHead>日期</TableHead>
                                            <TableHead>時間</TableHead>
                                            <TableHead>類型</TableHead>
                                            <TableHead>時數</TableHead>
                                            <TableHead>補休</TableHead>
                                            <TableHead>事由</TableHead>
                                            <TableHead>狀態</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {loadingAllOvertime ? (
                                            <TableRow>
                                                <TableCell colSpan={8} className="text-center py-8">
                                                    載入中...
                                                </TableCell>
                                            </TableRow>
                                        ) : allOvertime?.data?.length === 0 ? (
                                            <TableRow>
                                                <TableCell colSpan={8} className="text-center py-8 text-muted-foreground">
                                                    沒有符合條件的加班紀錄
                                                </TableCell>
                                            </TableRow>
                                        ) : (
                                            allOvertime?.data
                                                ?.filter((ot) => filterOvertimeType === 'all' || ot.overtime_type === filterOvertimeType)
                                                .map((overtime) => (
                                                    <TableRow key={overtime.id}>
                                                        <TableCell>
                                                            <div>
                                                                <div className="font-medium">{overtime.user_name}</div>
                                                                <div className="text-sm text-muted-foreground">{overtime.user_email}</div>
                                                            </div>
                                                        </TableCell>
                                                        <TableCell className="whitespace-nowrap">
                                                            {formatDate(overtime.overtime_date)}
                                                        </TableCell>
                                                        <TableCell className="whitespace-nowrap">
                                                            {format(new Date(overtime.start_time), 'HH:mm')} ~ {format(new Date(overtime.end_time), 'HH:mm')}
                                                        </TableCell>
                                                        <TableCell>{OVERTIME_TYPE_NAMES[overtime.overtime_type] || overtime.overtime_type}</TableCell>
                                                        <TableCell>{parseDecimal(overtime.hours).toFixed(1)}h</TableCell>
                                                        <TableCell>{parseDecimal(overtime.comp_time_hours) > 0 ? `${parseDecimal(overtime.comp_time_hours).toFixed(1)}h` : '-'}</TableCell>
                                                        <TableCell className="max-w-[200px] truncate">{overtime.reason}</TableCell>
                                                        <TableCell>{getStatusBadge(overtime.status)}</TableCell>
                                                    </TableRow>
                                                ))
                                        )}
                                    </TableBody>
                                </Table>
                                {allOvertime && allOvertime.total > 0 && (
                                    <div className="text-sm text-muted-foreground">
                                        共 {allOvertime.total} 筆紀錄
                                    </div>
                                )}
                            </CardContent>
                        </Card>
                    </TabsContent>
                )}
            </Tabs>
        </div>
    )
}

export default HrOvertimePage
