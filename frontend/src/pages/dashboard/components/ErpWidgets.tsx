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
import { StatusBadge } from '@/components/ui/status-badge'
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
  Cpu,
  CheckCircle2,
} from 'lucide-react'
import type { DocumentListItem, LowStockTotal, ExpiryAlert } from '@/lib/api'
import type { TrendDataPoint, EquipmentStats } from '../hooks/useDashboardData'
import { getMaintenanceBadge, type MaintenanceRecordWithDetails } from '@/pages/admin/types'

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
  /** 點擊時導航的路徑；指定後 Card 會 cursor-pointer + hover bg */
  onClick?: () => void
}

const StatWidget = memo(function StatWidget({ title, description, icon, value, isLoading, onClick }: StatWidgetProps) {
  const clickable = !!onClick
  return (
    <Card
      className={`h-full flex flex-col overflow-hidden${clickable ? ' cursor-pointer hover:bg-muted/40 transition-colors' : ''}`}
      onClick={onClick}
      role={clickable ? 'button' : undefined}
      tabIndex={clickable ? 0 : undefined}
      onKeyDown={clickable ? (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onClick?.() } } : undefined}
    >
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pt-3 pb-2">
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
  alerts: LowStockTotal[] | undefined
  isLoading: boolean
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  return (
    <StatWidget
      title={t('dashboard.widgets.names.low_stock_alert')}
      description={t('dashboard.widgets.descriptions.low_stock_alert')}
      icon={<AlertTriangle className="h-4 w-4 text-status-warning-text" />}
      value={alerts?.length || 0}
      isLoading={isLoading}
      onClick={() => navigate('/inventory?filter=low_stock')}
    />
  )
})

/**
 * R35-17: 7 天內到期（含已過期）品項數 widget — 與 LowStockAlertWidget 對稱。
 * 已過期 (`days_until_expiry < 0`) 與 7 天內 (`0..=7`) 都計入；點 widget 進
 * 庫存頁的「即將到期」過濾檢視（與 NotificationDropdown 一致）。
 *
 * value 用 paginated `total` 而非 `alerts.length`：>200 筆時 .length 被 per_page 上限截斷
 * 會靜默偏低；total 反映真實命中數。
 */
export const ExpiryAlertWidget = memo(function ExpiryAlertWidget({
  alerts,
  total,
  isLoading,
}: {
  alerts: ExpiryAlert[] | undefined
  total: number | undefined
  isLoading: boolean
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const value = total ?? alerts?.length ?? 0
  return (
    <StatWidget
      title={t('dashboard.widgets.names.expiry_alert')}
      description={t('dashboard.widgets.descriptions.expiry_alert')}
      icon={<AlertTriangle className="h-4 w-4 text-status-error-text" />}
      value={value}
      isLoading={isLoading}
      onClick={() => navigate('/inventory?filter=expiry_warning')}
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
      value={todayApprovedDocs.filter((d) => ['SO', 'PR'].includes(d.doc_type)).length}
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
      <CardHeader className="pt-3 pb-2">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <Calendar className="h-4 w-4 text-status-info-text" />
          {t('dashboard.widgets.names.weekly_trend')} ({days}{t('dashboard.widgets.common.daysUnit')})
        </CardTitle>
        <CardDescription className="text-xs">{t('dashboard.widgets.erp.trendDesc', { days })}</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <Table className="[&_th]:h-8" containerClassName="overflow-visible">
            <TableHeader>
              <TableRow>
                <TableHead>{t('dashboard.widgets.erp.docDate')}</TableHead>
                <TableHead className="text-right">{t('dashboard.widgets.erp.types.GRN')}</TableHead>
                {/* 出庫欄統計 SO + PR——兩者都讓貨離開倉庫。
                    SO 型別名沿用 Sales Order，但本系統實際語意是內部耗材領用
                    （見 ERP_SYSTEM.md §5「領用單（原稱銷貨單）」），故與 PR 同列出庫。 */}
                <TableHead className="text-right">{t('dashboard.widgets.erp.outbound')}</TableHead>
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
      <CardHeader className="pt-3 pb-2">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <FileText className="h-4 w-4 text-status-info-text" />
          {t('dashboard.widgets.names.recent_documents')}
        </CardTitle>
        <CardDescription className="text-xs">{t('dashboard.widgets.descriptions.recent_documents')}</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : documents && documents.length > 0 ? (
          <Table className="[&_th]:h-8" containerClassName="overflow-visible">
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
      <CardHeader className="pt-3 pb-2">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <Wrench className="h-4 w-4 text-status-warning-text" />
          {t('dashboard.widgets.names.recent_maintenance')}
        </CardTitle>
        <CardDescription className="text-xs">{t('dashboard.widgets.descriptions.recent_maintenance')}</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : records && records.length > 0 ? (
          <Table className="[&_th]:h-8" containerClassName="overflow-visible">
            <TableHeader>
              <TableRow>
                <TableHead>{t('dashboard.widgets.maintenance.equipment')}</TableHead>
                <TableHead>{t('dashboard.widgets.maintenance.status')}</TableHead>
                <TableHead>{t('dashboard.widgets.maintenance.reportedAt')}</TableHead>
                <TableHead>{t('dashboard.widgets.maintenance.completedAt')}</TableHead>
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
                  <TableCell>
                    {(() => {
                      const badge = getMaintenanceBadge(rec.maintenance_type, rec.status)
                      return <StatusBadge variant={badge.variant}>{t(badge.labelKey)}</StatusBadge>
                    })()}
                  </TableCell>
                  <TableCell>{formatDate(rec.reported_at)}</TableCell>
                  <TableCell>{rec.completed_at ? formatDate(rec.completed_at) : '—'}</TableCell>
                  <TableCell className="max-w-[150px] whitespace-normal break-words">
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

// --- 設備狀態總覽 ---

export const EquipmentStatusWidget = memo(function EquipmentStatusWidget({
  stats,
  isLoading,
}: {
  stats: EquipmentStats | null | undefined
  isLoading: boolean
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()

  const metrics = [
    {
      value: stats?.activeCount ?? 0,
      label: t('dashboard.widgets.equipment.active'),
      icon: CheckCircle2,
      colorClass: 'text-status-success-text',
      bgClass: 'bg-status-success-bg',
      highlight: false,
    },
    {
      value: stats?.repairCount ?? 0,
      label: t('dashboard.widgets.equipment.underRepair'),
      icon: Wrench,
      colorClass: (stats?.repairCount ?? 0) > 0 ? 'text-status-warning-text' : 'text-muted-foreground',
      bgClass: (stats?.repairCount ?? 0) > 0 ? 'bg-status-warning-bg' : 'bg-muted/40',
      highlight: (stats?.repairCount ?? 0) > 0,
    },
    {
      value: stats?.overdueCount ?? 0,
      label: t('dashboard.widgets.equipment.overdueCalibration'),
      icon: AlertTriangle,
      colorClass: (stats?.overdueCount ?? 0) > 0 ? 'text-destructive' : 'text-muted-foreground',
      bgClass: (stats?.overdueCount ?? 0) > 0 ? 'bg-status-error-bg' : 'bg-muted/40',
      highlight: (stats?.overdueCount ?? 0) > 0,
    },
    {
      value: stats?.total ?? 0,
      label: t('dashboard.widgets.equipment.total'),
      icon: Cpu,
      colorClass: 'text-muted-foreground',
      bgClass: 'bg-muted/40',
      highlight: false,
    },
  ]

  return (
    <Card className="h-full flex flex-col overflow-hidden">
      <CardHeader
        className="pt-3 pb-2 cursor-pointer hover:bg-muted/30 transition-colors"
        role="button"
        tabIndex={0}
        onClick={() => navigate('/equipment')}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); navigate('/equipment') } }}
      >
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <Cpu className="h-4 w-4 text-status-info-text" />
          {t('dashboard.widgets.names.equipment_status')}
        </CardTitle>
        <CardDescription className="text-xs">{t('dashboard.widgets.descriptions.equipment_status')}</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-3">
            {metrics.map(({ value, label, icon: Icon, colorClass, bgClass }) => (
              <div
                key={label}
                className={`p-3 rounded-lg ${bgClass} cursor-pointer hover:opacity-80 transition-opacity`}
                onClick={() => navigate('/equipment')}
              >
                <div className="flex items-center justify-between mb-1">
                  <Icon className={`h-4 w-4 ${colorClass}`} />
                </div>
                <div className={`text-2xl font-bold ${colorClass}`}>{value}</div>
                <div className={`text-xs mt-0.5 ${colorClass}`}>{label}</div>
              </div>
            ))}
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
      <CardHeader className="pt-3 pb-2">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <Calendar className="h-4 w-4 text-status-warning-text" />
          {t('dashboard.widgets.names.upcoming_leaves')}
        </CardTitle>
        <CardDescription className="text-xs">{t('dashboard.widgets.descriptions.upcoming_leaves')}</CardDescription>
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
                  <span className="text-2xl font-bold text-status-warning-text">{data?.expiring_soon_days ?? 0}</span>
                  <span className="text-sm text-status-warning-text ml-1">{t('dashboard.widgets.common.days')}</span>
                </div>
              </div>
            )}
            {(data?.expiring_soon_hours ?? 0) > 0 && (
              <div className="flex justify-between items-center p-2 bg-status-warning-bg rounded-lg border border-status-warning-text/20">
                <span className="text-sm text-status-warning-text">{t('dashboard.widgets.hr.expiringSoon')}（{t('dashboard.widgets.hr.compLeave')}）</span>
                <div className="text-right">
                  <span className="text-2xl font-bold text-status-warning-text">{data?.expiring_soon_hours ?? 0}</span>
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
