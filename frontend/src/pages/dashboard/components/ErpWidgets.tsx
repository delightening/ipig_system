/**
 * ERP 相關的 Dashboard Widget 元件
 */
import { memo } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { queryKeys } from '@/lib/queryKeys'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { formatDate } from '@/lib/utils'
import {
  TrendingUp,
  TrendingDown,
  AlertTriangle,
  FileText,
  Loader2,
  Calendar,
  Wrench,
} from 'lucide-react'
import type { DocumentListItem, LowStockAlert } from '@/lib/api'
import type { TrendDataPoint } from '../hooks/useDashboardData'
import type { MaintenanceRecordWithDetails } from '@/pages/admin/types'

// --- Helper ---

function getStatusBadge(status: string, t: (key: string) => string) {
  switch (status) {
    case 'draft':
      return <Badge variant="secondary">{t('dashboard.widgets.erp.status.draft')}</Badge>
    case 'submitted':
      return <Badge variant="warning">{t('dashboard.widgets.erp.status.submitted')}</Badge>
    case 'approved':
      return <Badge variant="success">{t('dashboard.widgets.erp.status.approved')}</Badge>
    case 'cancelled':
      return <Badge variant="destructive">{t('dashboard.widgets.erp.status.cancelled')}</Badge>
    default:
      return <Badge variant="outline">{status}</Badge>
  }
}

// --- Widgets ---

interface StatWidgetProps {
  title: string
  description: string
  icon: React.ReactNode
  value: number | string
  isLoading: boolean
}

const StatWidget = memo(function StatWidget({ title, description, icon, value, isLoading }: StatWidgetProps) {
  return (
    <Card className="h-full flex flex-col overflow-hidden">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        {icon}
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">
        <div className="text-2xl font-bold">{isLoading ? '-' : value}</div>
        <p className="text-xs text-muted-foreground">{description}</p>
      </CardContent>
    </Card>
  )
})

export const LowStockAlertWidget = memo(function LowStockAlertWidget({
  alerts,
  isLoading,
}: {
  alerts: LowStockAlert[] | undefined
  isLoading: boolean
}) {
  const { t } = useTranslation()
  return (
    <StatWidget
      title={t('dashboard.widgets.names.low_stock_alert')}
      description={t('dashboard.widgets.descriptions.low_stock_alert')}
      icon={<AlertTriangle className="h-4 w-4 text-status-warning-text" />}
      value={alerts?.length || 0}
      isLoading={isLoading}
    />
  )
})

export const PendingDocumentsWidget = memo(function PendingDocumentsWidget({
  documents,
  isLoading,
}: {
  documents: DocumentListItem[] | undefined
  isLoading: boolean
}) {
  const { t } = useTranslation()
  return (
    <StatWidget
      title={t('dashboard.widgets.names.pending_documents')}
      description={t('dashboard.widgets.descriptions.pending_documents')}
      icon={<FileText className="h-4 w-4 text-status-info-text" />}
      value={documents?.filter((d) => d.status === 'submitted').length || 0}
      isLoading={isLoading}
    />
  )
})

export const TodayInboundWidget = memo(function TodayInboundWidget({
  todayApprovedDocs,
  isLoading,
}: {
  todayApprovedDocs: DocumentListItem[]
  isLoading: boolean
}) {
  const { t } = useTranslation()
  return (
    <StatWidget
      title={t('dashboard.widgets.names.today_inbound')}
      description={t('dashboard.widgets.descriptions.today_inbound')}
      icon={<TrendingUp className="h-4 w-4 text-status-success-text" />}
      value={todayApprovedDocs.filter((d) => ['GRN'].includes(d.doc_type)).length}
      isLoading={isLoading}
    />
  )
})

export const TodayOutboundWidget = memo(function TodayOutboundWidget({
  todayApprovedDocs,
  isLoading,
}: {
  todayApprovedDocs: DocumentListItem[]
  isLoading: boolean
}) {
  const { t } = useTranslation()
  return (
    <StatWidget
      title={t('dashboard.widgets.names.today_outbound')}
      description={t('dashboard.widgets.descriptions.today_outbound')}
      icon={<TrendingDown className="h-4 w-4 text-status-error-text" />}
      value={todayApprovedDocs.filter((d) => ['DO', 'PR'].includes(d.doc_type)).length}
      isLoading={isLoading}
    />
  )
})

export const WeeklyTrendWidget = memo(function WeeklyTrendWidget({
  trendData,
  days,
  isLoading,
}: {
  trendData: TrendDataPoint[]
  days: number
  isLoading: boolean
}) {
  const { t } = useTranslation()
  return (
    <Card className="h-full flex flex-col overflow-hidden">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Calendar className="h-5 w-5 text-status-info-text" />
          {t('dashboard.widgets.names.weekly_trend')} ({days}{t('dashboard.widgets.common.daysUnit')})
        </CardTitle>
        <CardDescription>{t('dashboard.widgets.erp.trendDesc', { days })}</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('dashboard.widgets.erp.docDate')}</TableHead>
                <TableHead className="text-right">{t('dashboard.widgets.erp.types.GRN')}</TableHead>
                <TableHead className="text-right">{t('dashboard.widgets.erp.types.DO')}</TableHead>
                <TableHead className="text-right">{t('dashboard.widgets.erp.netChange')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {trendData.map((day) => {
                const net = day.inbound - day.outbound
                return (
                  <TableRow key={day.date}>
                    <TableCell className="font-medium">{day.dateStr}</TableCell>
                    <TableCell className="text-right">
                      <span className="inline-flex items-center gap-1 text-status-success-text">
                        <TrendingUp className="h-3 w-3" />
                        {day.inbound}
                      </span>
                    </TableCell>
                    <TableCell className="text-right">
                      <span className="inline-flex items-center gap-1 text-status-error-text">
                        <TrendingDown className="h-3 w-3" />
                        {day.outbound}
                      </span>
                    </TableCell>
                    <TableCell className="text-right">
                      <span className={net > 0 ? 'text-status-success-text' : net < 0 ? 'text-status-error-text' : 'text-muted-foreground'}>
                        {net > 0 ? '+' : ''}{net}
                      </span>
                    </TableCell>
                  </TableRow>
                )
              })}
            </TableBody>
          </Table>
        )}
      </CardContent>
    </Card>
  )
})

export const RecentDocumentsWidget = memo(function RecentDocumentsWidget({
  documents,
  isLoading,
}: {
  documents: DocumentListItem[] | undefined
  isLoading: boolean
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()

  return (
    <Card className="h-full flex flex-col overflow-hidden">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <FileText className="h-5 w-5 text-status-info-text" />
          {t('dashboard.widgets.names.recent_documents')}
        </CardTitle>
        <CardDescription>{t('dashboard.widgets.descriptions.recent_documents')}</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : documents && documents.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('dashboard.widgets.erp.docNo')}</TableHead>
                <TableHead>{t('dashboard.widgets.erp.docType')}</TableHead>
                <TableHead>{t('protocols.columns.status')}</TableHead>
                <TableHead>{t('dashboard.widgets.erp.docDate')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {documents.slice(0, 5).map((doc) => (
                <TableRow
                  key={doc.id}
                  className="cursor-pointer hover:bg-muted/50 transition-colors"
                  onClick={() => navigate(`/documents/${doc.id}`)}
                >
                  <TableCell className="font-medium">{doc.doc_no}</TableCell>
                  <TableCell>{t(`dashboard.widgets.erp.types.${doc.doc_type}`, { defaultValue: doc.doc_type })}</TableCell>
                  <TableCell>{getStatusBadge(doc.status, t)}</TableCell>
                  <TableCell>{formatDate(doc.doc_date)}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : (
          <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
            <FileText className="h-12 w-12 mb-2" />
            <p>{t('dashboard.widgets.erp.noDocs')}</p>
          </div>
        )}
      </CardContent>
    </Card>
  )
})

// --- 維修/保養紀錄 ---

function getMaintenanceStatusBadge(status: string) {
  switch (status) {
    case 'pending':
      return <Badge variant="warning">待處理</Badge>
    case 'in_progress':
      return <Badge variant="secondary">進行中</Badge>
    case 'pending_review':
      return <Badge variant="outline">待驗收</Badge>
    case 'completed':
      return <Badge variant="success">已完成</Badge>
    case 'unrepairable':
      return <Badge variant="destructive">無法維修</Badge>
    default:
      return <Badge variant="outline">{status}</Badge>
  }
}

function getMaintenanceTypeBadge(type: string) {
  switch (type) {
    case 'repair':
      return <Badge variant="destructive">維修</Badge>
    case 'maintenance':
      return <Badge variant="secondary">保養</Badge>
    default:
      return <Badge variant="outline">{type}</Badge>
  }
}

export const RecentMaintenanceWidget = memo(function RecentMaintenanceWidget({
  records,
  isLoading,
}: {
  records: MaintenanceRecordWithDetails[] | undefined
  isLoading: boolean
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()

  return (
    <Card className="h-full flex flex-col overflow-hidden">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Wrench className="h-5 w-5 text-status-warning-text" />
          {t('dashboard.widgets.names.recent_maintenance')}
        </CardTitle>
        <CardDescription>{t('dashboard.widgets.descriptions.recent_maintenance')}</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : records && records.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('dashboard.widgets.maintenance.equipment')}</TableHead>
                <TableHead>{t('dashboard.widgets.maintenance.type')}</TableHead>
                <TableHead>{t('protocols.columns.status')}</TableHead>
                <TableHead>{t('dashboard.widgets.maintenance.reportedAt')}</TableHead>
                <TableHead>{t('dashboard.widgets.maintenance.description')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {records.slice(0, 5).map((rec) => (
                <TableRow
                  key={rec.id}
                  className="cursor-pointer hover:bg-muted/50 transition-colors"
                  onClick={() => navigate('/equipment?tab=maintenance')}
                >
                  <TableCell className="font-medium">{rec.equipment_name}</TableCell>
                  <TableCell>{getMaintenanceTypeBadge(rec.maintenance_type)}</TableCell>
                  <TableCell>{getMaintenanceStatusBadge(rec.status)}</TableCell>
                  <TableCell>{formatDate(rec.reported_at)}</TableCell>
                  <TableCell className="max-w-[150px] truncate">
                    {rec.problem_description || rec.maintenance_items || '—'}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : (
          <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
            <Wrench className="h-12 w-12 mb-2" />
            <p>{t('dashboard.widgets.maintenance.noRecords')}</p>
          </div>
        )}
      </CardContent>
    </Card>
  )
})

// --- 即將到期假期 ---

interface BalanceSummaryData {
  expiring_soon_days: number
  expiring_soon_hours: number
}

export function UpcomingLeavesWidget() {
  const { t } = useTranslation()
  const { data, isLoading, error } = useQuery({
    queryKey: queryKeys.hr.balanceSummaryExpiring,
    queryFn: async () => {
      const res = await api.get<BalanceSummaryData>('/hr/balances/summary')
      return res.data
    },
    staleTime: 300_000,
  })

  const hasExpiring = (data?.expiring_soon_days ?? 0) > 0 || (data?.expiring_soon_hours ?? 0) > 0

  return (
    <Card className="h-full flex flex-col overflow-hidden">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <Calendar className="h-4 w-4 text-status-warning-text" />
          {t('dashboard.widgets.names.upcoming_leaves')}
        </CardTitle>
        <CardDescription>{t('dashboard.widgets.descriptions.upcoming_leaves')}</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-4">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        ) : error ? (
          <p className="text-sm text-muted-foreground">{t('dashboard.widgets.common.loadFailed')}</p>
        ) : !hasExpiring ? (
          <div className="flex flex-col items-center justify-center py-4 text-muted-foreground">
            <Calendar className="h-8 w-8 mb-2 text-status-success-text" />
            <p className="text-sm">{t('dashboard.widgets.hr.noExpiring')}</p>
          </div>
        ) : (
          <div className="space-y-3">
            {(data?.expiring_soon_days ?? 0) > 0 && (
              <div className="flex justify-between items-center p-2 bg-status-warning-bg rounded-lg border border-status-warning-text/20">
                <span className="text-sm text-status-warning-text">{t('dashboard.widgets.hr.expiringSoon')}（{t('dashboard.widgets.hr.annualLeave')}）</span>
                <div className="text-right">
                  <span className="text-lg font-semibold text-status-warning-text">{data?.expiring_soon_days ?? 0}</span>
                  <span className="text-sm text-status-warning-text ml-1">{t('dashboard.widgets.common.days')}</span>
                </div>
              </div>
            )}
            {(data?.expiring_soon_hours ?? 0) > 0 && (
              <div className="flex justify-between items-center p-2 bg-status-warning-bg rounded-lg border border-status-warning-text/20">
                <span className="text-sm text-status-warning-text">{t('dashboard.widgets.hr.expiringSoon')}（{t('dashboard.widgets.hr.compLeave')}）</span>
                <div className="text-right">
                  <span className="text-lg font-semibold text-status-warning-text">{data?.expiring_soon_hours ?? 0}</span>
                  <span className="text-sm text-status-warning-text ml-1">{t('dashboard.widgets.common.hours')}</span>
                </div>
              </div>
            )}
            <p className="text-xs text-muted-foreground text-center">{t('dashboard.widgets.hr.expiringIn30Days')}</p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
