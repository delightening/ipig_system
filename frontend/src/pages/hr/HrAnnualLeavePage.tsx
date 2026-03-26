import { useState } from 'react'
import { useDialogSet } from '@/hooks/useDialogSet'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import {
    AlertTriangle,
    Calendar,
    CheckCircle,
    Download,
    Plus,
    RefreshCw,
    Search,
    User,
    Users,
} from 'lucide-react'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { PageHeader } from '@/components/ui/page-header'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import { StatusBadge } from '@/components/ui/status-badge'
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
    AnnualLeaveBalanceView,
    ExpiredLeaveReport,
    CreateAnnualLeaveRequest,
} from '@/types/hr'
import { EmptyState } from '@/components/ui/empty-state'
import { annualLeaveEntitlementSchema, type AnnualLeaveEntitlementFormData } from '@/lib/validation'

interface User {
    id: string
    email: string
    display_name: string
    is_active: boolean
    is_internal: boolean
}

export function HrAnnualLeavePage() {
    const dialogs = useDialogSet(['create'] as const)
    const [selectedUserId, setSelectedUserId] = useState<string>('')
    const [searchQuery, setSearchQuery] = useState('')
    const queryClient = useQueryClient()

    // 表單狀態
    const { register, handleSubmit, setValue, watch, reset: resetForm, formState: { errors } } = useForm<AnnualLeaveEntitlementFormData>({
        resolver: zodResolver(annualLeaveEntitlementSchema),
        defaultValues: {
            userId: '',
            year: new Date().getFullYear(),
            days: '' as unknown as number,
            hireDate: '',
            notes: '',
        },
    })

    // 取得所有內部員工（使用專用的 HR 端點）
    const { data: usersData } = useQuery({
        queryKey: ['internal-users-for-balance'],
        queryFn: async () => {
            // 使用專用的 HR 端點，有 hr.balance.manage 權限即可訪問
            const res = await api.get<User[]>('/hr/internal-users')
            return res.data
        },
    })

    // 取得選定員工的特休額度
    const { data: userBalances, isLoading: loadingBalances } = useQuery({
        queryKey: ['user-annual-balances', selectedUserId],
        queryFn: async () => {
            if (!selectedUserId) return []
            const res = await api.get<AnnualLeaveBalanceView[]>(
                `/hr/balances/annual?user_id=${selectedUserId}`
            )
            return res.data
        },
        enabled: !!selectedUserId,
    })

    // 取得過期特休報表（待補償）
    const { data: expiredLeaves, isLoading: loadingExpired } = useQuery({
        queryKey: ['expired-leave-compensation'],
        queryFn: async () => {
            const res = await api.get<ExpiredLeaveReport[]>('/hr/balances/expired-compensation')
            return res.data
        },
    })

    // 建立特休額度
    const createEntitlementMutation = useMutation({
        mutationFn: async (data: CreateAnnualLeaveRequest) => {
            return api.post('/hr/balances/annual-entitlements', data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['user-annual-balances'] })
            queryClient.invalidateQueries({ queryKey: ['expired-leave-compensation'] })
            dialogs.close('create')
            resetForm()
            toast({
                title: '建立成功',
                description: '已成功建立特休額度',
            })
        },
        onError: (error: Error) => {
            toast({
                title: '建立失敗',
                description: error.message,
                variant: 'destructive',
            })
        },
    })

    const onValidSubmit = (data: AnnualLeaveEntitlementFormData) => {
        createEntitlementMutation.mutate({
            user_id: data.userId,
            entitlement_year: data.year,
            entitled_days: data.days,
            hire_date: data.hireDate || undefined,
            notes: data.notes || undefined,
        })
    }

    // 計算過期待補償總天數
    const totalExpiredDays = expiredLeaves?.reduce((sum, item) => sum + item.remaining_days, 0) || 0

    // 過濾員工列表
    const filteredUsers = usersData?.filter(
        u =>
            u.display_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            u.email.toLowerCase().includes(searchQuery.toLowerCase())
    )

    // 匯出過期報表為 CSV
    const exportExpiredReport = () => {
        if (!expiredLeaves || expiredLeaves.length === 0) {
            toast({ title: '無資料可匯出', variant: 'destructive' })
            return
        }

        const headers = ['員工姓名', 'Email', '年度', '已使用天數', '待補償天數', '到期日']
        const rows = expiredLeaves.map(item => [
            item.user_name,
            item.user_email,
            item.entitlement_year.toString(),
            item.used_days.toString(),
            item.remaining_days.toString(),
            item.expires_at,
        ])

        const csvContent = [headers, ...rows].map(row => row.join(',')).join('\n')
        const blob = new Blob(['\uFEFF' + csvContent], { type: 'text/csv;charset=utf-8;' })
        const link = document.createElement('a')
        link.href = URL.createObjectURL(blob)
        link.download = `過期特休補償報表_${format(new Date(), 'yyyy-MM-dd')}.csv`
        link.click()

        toast({ title: '匯出成功', description: '報表已下載' })
    }

    return (
        <div className="space-y-6">
            <PageHeader
                title="特休額度管理"
                description="管理員工特休假額度、查看過期待補償報表"
                actions={
                    <Button size="sm" onClick={() => dialogs.open('create')}>
                        <Plus className="h-4 w-4 mr-2" />
                        新增特休額度
                    </Button>
                }
            />

            {/* 統計卡片 */}
            <div className="grid gap-4 md:grid-cols-3">
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">內部員工數</CardTitle>
                        <Users className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">{usersData?.length || 0}</div>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">待補償記錄</CardTitle>
                        <AlertTriangle className="h-4 w-4 text-status-warning-text" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold text-status-warning-text">
                            {expiredLeaves?.length || 0}
                        </div>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">待補償總天數</CardTitle>
                        <Calendar className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold text-status-warning-text">
                            {totalExpiredDays.toFixed(1)} 天
                        </div>
                    </CardContent>
                </Card>
            </div>

            <PageTabs
                tabs={[
                    { value: 'entitlements', label: '員工特休額度', icon: User },
                    { value: 'expired', label: '過期待補償', icon: AlertTriangle, badge: expiredLeaves?.length },
                ]}
                defaultTab="entitlements"
            >
                {/* 員工特休額度 Tab */}
                <PageTabContent value="entitlements" className="space-y-4">
                    <Card>
                        <CardHeader>
                            <CardTitle>選擇員工查看特休額度</CardTitle>
                            <CardDescription>
                                選擇員工以查看其各年度特休假額度與使用狀況
                            </CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            {/* 搜尋框 */}
                            <div className="relative">
                                <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                                <Input
                                    placeholder="搜尋員工姓名或 Email..."
                                    value={searchQuery}
                                    onChange={e => setSearchQuery(e.target.value)}
                                    className="pl-8"
                                />
                            </div>

                            {/* 員工列表 */}
                            <div className="flex flex-wrap gap-2 max-h-48 overflow-y-auto">
                                {filteredUsers?.map(user => (
                                    <Button
                                        key={user.id}
                                        variant={selectedUserId === user.id ? 'default' : 'outline'}
                                        size="sm"
                                        onClick={() => setSelectedUserId(user.id)}
                                        className="justify-start"
                                    >
                                        <User className="h-4 w-4 mr-2" />
                                        {user.display_name}
                                    </Button>
                                ))}
                            </div>

                            {/* 選定員工的特休額度 */}
                            {selectedUserId && (
                                <div className="mt-4">
                                    <h3 className="text-lg font-semibold mb-2">
                                        {usersData?.find(u => u.id === selectedUserId)?.display_name} 的特休額度
                                    </h3>
                                    {loadingBalances ? (
                                        <div className="flex justify-center py-4">
                                            <RefreshCw className="h-6 w-6 animate-spin" />
                                        </div>
                                    ) : userBalances && userBalances.length > 0 ? (
                                        <Table>
                                            <TableHeader>
                                                <TableRow>
                                                    <TableHead>年度</TableHead>
                                                    <TableHead>總天數</TableHead>
                                                    <TableHead>已使用</TableHead>
                                                    <TableHead>剩餘</TableHead>
                                                    <TableHead>到期日</TableHead>
                                                    <TableHead>狀態</TableHead>
                                                </TableRow>
                                            </TableHeader>
                                            <TableBody>
                                                {userBalances.map((balance, idx) => (
                                                    <TableRow key={idx}>
                                                        <TableCell className="font-medium">
                                                            {balance.entitlement_year}
                                                        </TableCell>
                                                        <TableCell>{balance.entitled_days}</TableCell>
                                                        <TableCell>{balance.used_days}</TableCell>
                                                        <TableCell className="font-semibold">
                                                            {balance.remaining_days}
                                                        </TableCell>
                                                        <TableCell>
                                                            {format(new Date(balance.expires_at), 'yyyy/MM/dd', { locale: zhTW })}
                                                        </TableCell>
                                                        <TableCell>
                                                            {balance.is_expired ? (
                                                                <Badge variant="destructive">已過期</Badge>
                                                            ) : balance.days_until_expiry <= 30 ? (
                                                                <StatusBadge variant="warning">
                                                                    即將到期 ({balance.days_until_expiry}天)
                                                                </StatusBadge>
                                                            ) : (
                                                                <Badge variant="secondary">
                                                                    <CheckCircle className="h-3 w-3 mr-1" />
                                                                    有效
                                                                </Badge>
                                                            )}
                                                        </TableCell>
                                                    </TableRow>
                                                ))}
                                            </TableBody>
                                        </Table>
                                    ) : (
                                        <p className="text-muted-foreground text-center py-4">
                                            該員工尚無特休額度記錄
                                        </p>
                                    )}
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </PageTabContent>

                {/* 過期待補償 Tab */}
                <PageTabContent value="expired" className="space-y-4">
                    <Card>
                        <CardHeader className="flex flex-row items-center justify-between">
                            <div>
                                <CardTitle>過期特休待補償報表</CardTitle>
                                <CardDescription>
                                    列出所有已過期但仍有剩餘天數的特休假，供會計部門處理補償
                                </CardDescription>
                            </div>
                            <Button variant="outline" onClick={exportExpiredReport}>
                                <Download className="h-4 w-4 mr-2" />
                                匯出 CSV
                            </Button>
                        </CardHeader>
                        <CardContent>
                            {loadingExpired ? (
                                <div className="flex justify-center py-8">
                                    <RefreshCw className="h-6 w-6 animate-spin" />
                                </div>
                            ) : expiredLeaves && expiredLeaves.length > 0 ? (
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHead>員工姓名</TableHead>
                                            <TableHead>Email</TableHead>
                                            <TableHead>年度</TableHead>
                                            <TableHead>總天數</TableHead>
                                            <TableHead>已使用</TableHead>
                                            <TableHead className="text-status-warning-text">待補償天數</TableHead>
                                            <TableHead>到期日</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {expiredLeaves.map((item, idx) => (
                                            <TableRow key={idx}>
                                                <TableCell className="font-medium">
                                                    {item.user_name}
                                                </TableCell>
                                                <TableCell className="text-muted-foreground">
                                                    {item.user_email}
                                                </TableCell>
                                                <TableCell>{item.entitlement_year}</TableCell>
                                                <TableCell>{item.entitled_days}</TableCell>
                                                <TableCell>{item.used_days}</TableCell>
                                                <TableCell className="font-bold text-status-warning-text">
                                                    {item.remaining_days}
                                                </TableCell>
                                                <TableCell>
                                                    {format(new Date(item.expires_at), 'yyyy/MM/dd', { locale: zhTW })}
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            ) : (
                                <EmptyState icon={CheckCircle} title="目前沒有過期待補償的特休假" />
                            )}
                        </CardContent>
                    </Card>
                </PageTabContent>
            </PageTabs>

            {/* 新增特休額度對話框 */}
            <Dialog open={dialogs.isOpen('create')} onOpenChange={dialogs.setOpen('create')}>
                <DialogContent className="max-w-md">
                    <DialogHeader>
                        <DialogTitle>新增特休額度</DialogTitle>
                        <DialogDescription>
                            為員工建立新年度的特休假額度。若有提供到職日，到期日將自動計算為到職週年日 + 2年。
                        </DialogDescription>
                    </DialogHeader>
                    <form onSubmit={handleSubmit(onValidSubmit)} className="space-y-4">
                        {/* 選擇員工 */}
                        <div className="space-y-2">
                            <Label>員工 *</Label>
                            <Select value={watch('userId')} onValueChange={v => setValue('userId', v, { shouldValidate: true })}>
                                <SelectTrigger>
                                    <SelectValue placeholder="選擇員工..." />
                                </SelectTrigger>
                                <SelectContent>
                                    {usersData?.map(user => (
                                        <SelectItem key={user.id} value={user.id}>
                                            {user.display_name} ({user.email})
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                            {errors.userId && (
                                <p className="text-sm text-destructive">{errors.userId.message}</p>
                            )}
                        </div>

                        {/* 年度 */}
                        <div className="space-y-2">
                            <Label>授予年度 *</Label>
                            <Select value={String(watch('year'))} onValueChange={v => setValue('year', Number(v), { shouldValidate: true })}>
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {[...Array(3)].map((_, i) => {
                                        const year = new Date().getFullYear() - 1 + i
                                        return (
                                            <SelectItem key={year} value={year.toString()}>
                                                {year} 年
                                            </SelectItem>
                                        )
                                    })}
                                </SelectContent>
                            </Select>
                            {errors.year && (
                                <p className="text-sm text-destructive">{errors.year.message}</p>
                            )}
                        </div>

                        {/* 特休天數 */}
                        <div className="space-y-2">
                            <Label>特休天數 *</Label>
                            <Input
                                type="number"
                                min="0"
                                step="0.5"
                                {...register('days', { valueAsNumber: true })}
                                placeholder="例：7"
                            />
                            {errors.days && (
                                <p className="text-sm text-destructive">{errors.days.message}</p>
                            )}
                        </div>

                        {/* 到職日 */}
                        <div className="space-y-2">
                            <Label>到職日（用於計算到期日）</Label>
                            <Input
                                type="date"
                                {...register('hireDate')}
                            />
                            <p className="text-xs text-muted-foreground">
                                到期日 = 授予年度 + 2年的到職週年日。若不填寫，到期日為授予年度 + 2年的12月31日。
                            </p>
                        </div>

                        {/* 備註 */}
                        <div className="space-y-2">
                            <Label>備註</Label>
                            <Input
                                {...register('notes')}
                                placeholder="選填"
                            />
                        </div>
                    </form>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => dialogs.close('create')}>
                            取消
                        </Button>
                        <Button
                            onClick={handleSubmit(onValidSubmit)}
                            disabled={createEntitlementMutation.isPending}
                        >
                            {createEntitlementMutation.isPending ? '建立中...' : '建立'}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    )
}
