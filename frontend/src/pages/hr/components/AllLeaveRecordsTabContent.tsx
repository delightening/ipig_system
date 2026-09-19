import { useState } from 'react'
import { Trans, useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import { FileText } from 'lucide-react'

import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import { useQuery } from '@tanstack/react-query'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { StatusBadge } from '@/components/ui/status-badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { DataTable } from '@/components/ui/data-table'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { formatDate, formatDateTime } from '@/lib/utils'
import type { LeaveRequestWithUser } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'
import {
    LEAVE_STATUS_CODES,
    LEAVE_TYPE_CODES,
    formatLeaveHours,
    getLeaveStatusVariant,
    getLeaveWaitingDays,
    getWaitingDaysClass,
    leaveStatusLabel,
    leaveTypeLabel,
} from '../constants'

/**
 * 依審核關卡換標籤文字。PENDING_L1/PENDING_DIRECTOR 的名字是後端動態算出的一批合法審核人
 * （部門主管／負責人，缺人時 fallback 管理員），非單一 DB 欄位——見 HrService::list_leaves。
 */
const approverLabelFor = (t: TFunction, status: string) => {
    switch (status) {
        case 'PENDING_PROXY':
            return t('hrPages.leaves.approver.delegate')
        case 'PENDING_L1':
            return t('hrPages.leaves.approver.unitSupervisor')
        case 'PENDING_DIRECTOR':
            return t('hrPages.leaves.approver.director')
        default:
            return t('hrPages.leaves.approver.reviewer')
    }
}

/**
 * 狀態徽章。待審核狀態附 tooltip 顯示目前卡在哪個人手上（代理人／單位主管等審核人）、已等待幾天。
 * Radix Trigger 本身是 button，鍵盤可聚焦，不是純 hover-only。
 */
function LeaveStatusCell({ leave }: { leave: LeaveRequestWithUser }) {
    const { t } = useTranslation()
    const status = getLeaveStatusVariant(t, leave.status)
    const badge = <StatusBadge variant={status.variant}>{status.label}</StatusBadge>

    if (!leave.status.startsWith('PENDING') || !leave.current_approver_name) return badge

    const days = getLeaveWaitingDays(leave.submitted_at)
    return (
        <Tooltip>
            <TooltipTrigger className="cursor-help align-middle">{badge}</TooltipTrigger>
            <TooltipContent>
                <div>{t('hrPages.leaves.approverLabel', { label: approverLabelFor(t, leave.status) })}{leave.current_approver_name}</div>
                {days !== null && (
                    <div className="mt-0.5">
                        <Trans
                            i18nKey="hrPages.leaves.waitedDays"
                            values={{ days }}
                            components={{ w: <span className={getWaitingDaysClass(days)} /> }}
                        />
                    </div>
                )}
            </TooltipContent>
        </Tooltip>
    )
}

/** 窄容器（< 700px）卡片版：代理人與等待天數直接寫在版面上，不靠 hover（手機沒有 hover）。 */
function LeaveRecordCard({ leave }: { leave: LeaveRequestWithUser }) {
    const { t } = useTranslation()
    const status = getLeaveStatusVariant(t, leave.status)
    const days = getLeaveWaitingDays(leave.submitted_at)
    return (
        <div className="space-y-2">
            <div className="flex items-start justify-between gap-3">
                <div>
                    <div className="font-medium">{leave.user_name}</div>
                    <div className="text-sm text-muted-foreground">{leave.user_email}</div>
                </div>
                <StatusBadge variant={status.variant}>{status.label}</StatusBadge>
            </div>
            <div className="text-sm text-muted-foreground">
                {leaveTypeLabel(t, leave.leave_type)}
                {' · '}
                {formatDate(leave.start_date)}
                {leave.start_date !== leave.end_date && ` ~ ${formatDate(leave.end_date)}`}
                {' · '}
                {formatLeaveHours(t, leave)}
            </div>
            {leave.reason && <div className="text-sm break-words">{leave.reason}</div>}
            {leave.status.startsWith('PENDING') && leave.current_approver_name && (
                <div className="text-sm">
                    <span className="text-muted-foreground">{t('hrPages.leaves.approverLabel', { label: approverLabelFor(t, leave.status) })}</span>
                    {leave.current_approver_name}
                    {days !== null && (
                        <>
                            {' · '}
                            <Trans
                                i18nKey="hrPages.leaves.waitedDays"
                                values={{ days }}
                                components={{ w: <span className={getWaitingDaysClass(days)} /> }}
                            />
                        </>
                    )}
                </div>
            )}
            {leave.submitted_at && (
                <div className="text-sm text-muted-foreground">
                    {t('hrPages.leaves.submittedAtLabel', { time: formatDateTime(leave.submitted_at) })}
                </div>
            )}
        </div>
    )
}

export function AllLeaveRecordsTabContent() {
    const { t } = useTranslation()
    const [filterStatus, setFilterStatus] = useState<string>('all')
    const [filterLeaveType, setFilterLeaveType] = useState<string>('all')
    const { from: filterFrom, to: filterTo, setFrom: setFilterFrom, setTo: setFilterTo, reset: resetDateRange } =
        useDateRangeFilter()

    const { data: allLeaves, isLoading } = useQuery({
        queryKey: queryKeys.hr.allLeaves({ filterStatus, filterLeaveType, filterFrom, filterTo }),
        queryFn: async () => {
            const params = new URLSearchParams({ view_all: 'true' })
            if (filterStatus !== 'all') params.append('status', filterStatus)
            if (filterLeaveType !== 'all') params.append('leave_type', filterLeaveType)
            if (filterFrom) params.append('from', filterFrom)
            if (filterTo) params.append('to', filterTo)
            const res = await api.get<PaginatedResponse<LeaveRequestWithUser>>(`/hr/leaves?${params.toString()}`)
            return res.data
        },
    })

    const hasFilters = filterStatus !== 'all' || filterLeaveType !== 'all' || filterFrom || filterTo

    return (
        <Card>
            <CardHeader>
                <CardTitle>{t('hrPages.leaves.records.title')}</CardTitle>
                <CardDescription>{t('hrPages.leaves.records.description')}</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
                {/* 篩選列 */}
                <div className="flex flex-wrap gap-3 items-end">
                    <div className="grid gap-1">
                        <Label className="text-xs">{t('hrPages.shared.col.status')}</Label>
                        <Select value={filterStatus} onValueChange={setFilterStatus}>
                            <SelectTrigger className="w-[140px]">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="all">{t('common.allStatus')}</SelectItem>
                                {LEAVE_STATUS_CODES.map((code) => (
                                    <SelectItem key={code} value={code}>{leaveStatusLabel(t, code)}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="grid gap-1">
                        <Label className="text-xs">{t('hrPages.shared.col.leaveType')}</Label>
                        <Select value={filterLeaveType} onValueChange={setFilterLeaveType}>
                            <SelectTrigger className="w-[140px]">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="all">{t('hrPages.leaves.records.allLeaveTypes')}</SelectItem>
                                {LEAVE_TYPE_CODES.map((code) => (
                                    <SelectItem key={code} value={code}>{leaveTypeLabel(t, code)}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="grid gap-1">
                        <Label className="text-xs">{t('hrPages.shared.filter.fromDate')}</Label>
                        <Input
                            type="date"
                            value={filterFrom}
                            onChange={(e) => setFilterFrom(e.target.value)}
                            className="w-[160px]"
                        />
                    </div>
                    <div className="grid gap-1">
                        <Label className="text-xs">{t('hrPages.shared.filter.endDate')}</Label>
                        <Input
                            type="date"
                            value={filterTo}
                            onChange={(e) => setFilterTo(e.target.value)}
                            className="w-[160px]"
                        />
                    </div>
                    {hasFilters && (
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                                setFilterStatus('all')
                                setFilterLeaveType('all')
                                resetDateRange()
                            }}
                        >
                            {t('common.clearFilters')}
                        </Button>
                    )}
                </div>

                {/* 表格 */}
                <DataTable<LeaveRequestWithUser>
                    columns={[
                        {
                            key: 'applicant',
                            header: t('hrPages.shared.col.applicant'),
                            cell: (leave) => (
                                <div>
                                    <div className="font-medium">{leave.user_name}</div>
                                    <div className="text-sm text-muted-foreground">{leave.user_email}</div>
                                </div>
                            ),
                        },
                        {
                            key: 'leave_type',
                            header: t('hrPages.shared.col.leaveType'),
                            hideClassName: 'hidden @[700px]:table-cell',
                            cell: (leave) => leaveTypeLabel(t, leave.leave_type),
                        },
                        {
                            key: 'date',
                            header: t('hrPages.shared.col.date'),
                            cell: (leave) => (
                                <span className="whitespace-nowrap">
                                    {formatDate(leave.start_date)}
                                    {leave.start_date !== leave.end_date && ` ~ ${formatDate(leave.end_date)}`}
                                </span>
                            ),
                        },
                        {
                            key: 'hours',
                            header: t('hrPages.shared.col.hours'),
                            hideClassName: 'hidden @[700px]:table-cell',
                            cell: (leave) => formatLeaveHours(t, leave),
                        },
                        {
                            key: 'reason',
                            header: t('hrPages.shared.col.reason'),
                            className: 'max-w-[200px] whitespace-normal break-words',
                            hideClassName: 'hidden @[880px]:table-cell',
                            cell: (leave) => leave.reason,
                        },
                        {
                            key: 'submitted_at',
                            header: t('hrPages.shared.col.submittedAt'),
                            hideClassName: 'hidden @[1020px]:table-cell',
                            cell: (leave) => (
                                <span className="whitespace-nowrap">
                                    {leave.submitted_at ? formatDateTime(leave.submitted_at) : '-'}
                                </span>
                            ),
                        },
                        {
                            key: 'status',
                            header: t('hrPages.shared.col.status'),
                            cell: (leave) => <LeaveStatusCell leave={leave} />,
                        },
                    ]}
                    data={allLeaves?.data}
                    isLoading={isLoading}
                    emptyIcon={FileText}
                    emptyTitle={t('hrPages.leaves.records.empty')}
                    rowKey={(row) => row.id}
                    mobileCard={(leave) => <LeaveRecordCard leave={leave} />}
                    cardBreakpoint={700}
                />
                {allLeaves && allLeaves.total > 0 && (
                    <div className="text-sm text-muted-foreground">
                        {t('hrPages.shared.totalRecords', { count: allLeaves.total })}
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
