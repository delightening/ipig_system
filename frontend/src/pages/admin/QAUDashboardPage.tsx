import { useMemo } from 'react'
import { FileText, ClipboardList, Shield, Stethoscope, AlertTriangle, BookOpen, Calendar } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'

import api from '@/lib/api'
import { useGuestQuery } from '@/hooks/useGuestQuery'
import { DEMO_QAU_DASHBOARD } from '@/lib/guest-demo'
import { PageHeader } from '@/components/ui/page-header'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { SkeletonPulse } from '@/components/ui/skeleton'
import { STALE_TIME } from '@/lib/query'
import { useTableSort } from '@/hooks/useTableSort'
import { getEntityTypeLabel } from './constants/auditLogs'

interface ProtocolStatusCount {
  status: string
  display_name: string
  count: number
}

interface ReviewProgressSummary {
  status_changes_last_7_days: number
  protocols_in_review: number
  protocols_pending_pi_response: number
}

interface AuditEntityCount {
  entity_type: string
  count: number
}

interface AnimalStatusCount {
  status: string
  display_name: string
  count: number
}

interface AnimalSummary {
  total: number
  by_status: AnimalStatusCount[]
  in_experiment: number
  euthanized: number
  completed: number
}

interface StatusCount {
  status: string
  count: number
}

interface QaPlanSummary {
  open_nc_count: number
  overdue_nc_count: number
  active_sop_count: number
  inspection_by_status: StatusCount[]
  schedule_items_by_status: StatusCount[]
}

interface QauDashboard {
  protocol_status_summary: ProtocolStatusCount[]
  review_progress: ReviewProgressSummary
  audit_summary: AuditEntityCount[]
  animal_summary: AnimalSummary
  qa_plan_summary: QaPlanSummary
}

// QA 稽查 / 排程 / NC status 中文。涵蓋三個 enum：
//   QaInspectionStatus: draft / submitted / closed
//   QaScheduleItemStatus: planned / in_progress / completed / cancelled / overdue
//   NcStatus: open / in_progress / pending_verification / closed
// （Gemini PR #412 review 補完，原本缺 draft / submitted / closed / planned）
// 值為 i18n 鍵（渲染時才 t()），避免 module 級常數凍結語言。
const INSPECTION_STATUS_LABEL_KEYS: Record<string, string> = {
  // 稽查報告
  draft: 'adminGlp.shared.statusLabel.draft',
  submitted: 'adminGlp.qaDashboard.status.submitted',
  closed: 'adminGlp.shared.statusLabel.closed',
  // 稽查排程（item）
  planned: 'adminGlp.qaDashboard.status.planned',
  scheduled: 'adminGlp.qaDashboard.status.scheduled', // legacy alias
  in_progress: 'adminGlp.shared.statusLabel.inProgress',
  completed: 'adminGlp.shared.statusLabel.completed',
  cancelled: 'adminGlp.shared.statusLabel.cancelled',
  overdue: 'adminGlp.shared.statusLabel.overdue',
  // NC
  open: 'adminGlp.qaDashboard.status.open',
  pending_verification: 'adminGlp.shared.statusLabel.pendingVerification',
  pending: 'adminGlp.qaDashboard.status.pending',
}

function formatStatusList(items: StatusCount[], t: TFunction): string {
  if (items.length === 0) return t('adminGlp.qaDashboard.noData')
  return items
    .map((s) => {
      const labelKey = INSPECTION_STATUS_LABEL_KEYS[s.status]
      return t('adminGlp.qaDashboard.statusCount', {
        status: labelKey ? t(labelKey) : s.status,
        count: s.count,
      })
    })
    .join(t('adminGlp.shared.listSeparator'))
}

function entityLabel(entityType: string, t: TFunction): string {
  return getEntityTypeLabel(t, entityType) ?? entityType
}

// 後端 display_name 固定中文（backend/src/services/qau.rs），改依 status code 走語言包；
// 語言包沒有的 code（後端日後新增）退回後端的 display_name，不顯示鍵名。
// 回傳新陣列，讓 useTableSort 依翻譯後的文字排序。
function localizeStatusRows<T extends { status: string; display_name: string }>(
  rows: T[] | undefined,
  keyPrefix: string,
  t: TFunction,
): T[] | undefined {
  return rows?.map((row) => ({
    ...row,
    display_name: t(`${keyPrefix}.${row.status}`, { defaultValue: row.display_name }),
  }))
}

export function QAUDashboardPage() {
  const { t } = useTranslation()
  const { data, isLoading, error } = useGuestQuery(
    DEMO_QAU_DASHBOARD as unknown as QauDashboard,
    {
      queryKey: ['qau-dashboard'],
      queryFn: async () => {
        const res = await api.get<QauDashboard>('/qau/dashboard')
        return res.data
      },
      staleTime: STALE_TIME.LIST,
    },
  )

  // react-i18next 切換語言時會換一個新的 t，deps 放 t 即可跟著重算
  const protocolStatusRows = useMemo(
    () => localizeStatusRows(data?.protocol_status_summary, 'protocols.status', t),
    [data?.protocol_status_summary, t],
  )
  const animalStatusRows = useMemo(
    () => localizeStatusRows(data?.animal_summary?.by_status, 'animals.statusLabels', t),
    [data?.animal_summary?.by_status, t],
  )

  const {
    sortedData: sortedProtocolStatus, sort: protocolSort, toggleSort: toggleProtocolSort,
  } = useTableSort(protocolStatusRows)
  const {
    sortedData: sortedAudit, sort: auditSort, toggleSort: toggleAuditSort,
  } = useTableSort(data?.audit_summary)
  const {
    sortedData: sortedAnimalStatus, sort: animalSort, toggleSort: toggleAnimalSort,
  } = useTableSort(animalStatusRows)

  if (error) {
    return (
      <div className="p-6">
        <div className="text-destructive">{t('adminGlp.qaDashboard.loadFailed', { message: (error as Error).message })}</div>
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <PageHeader
        title={t('adminGlp.qaDashboard.title')}
        description={t('adminGlp.qaDashboard.description')}
      />

      {isLoading ? (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {[1, 2, 3, 4].map((i) => (
            <SkeletonPulse key={i} className="h-32" />
          ))}
        </div>
      ) : data ? (
        <>
          {/* 審查進度概覽 */}
          <div className="grid gap-4 md:grid-cols-3">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{t('adminGlp.qaDashboard.card.statusChanges')}</CardTitle>
                <ClipboardList className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {data.review_progress.status_changes_last_7_days}
                </div>
                <p className="text-xs text-muted-foreground">{t('adminGlp.qaDashboard.card.statusChangesHint')}</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{t('adminGlp.qaDashboard.card.protocolsInReview')}</CardTitle>
                <FileText className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{data.review_progress.protocols_in_review}</div>
                <p className="text-xs text-muted-foreground">
                  {t('adminGlp.qaDashboard.card.protocolsInReviewHint')}
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{t('adminGlp.qaDashboard.card.pendingPi')}</CardTitle>
                <FileText className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {data.review_progress.protocols_pending_pi_response}
                </div>
                <p className="text-xs text-muted-foreground">
                  {t('adminGlp.qaDashboard.card.pendingPiHint')}
                </p>
              </CardContent>
            </Card>
          </div>

          {/* QA 計畫管理摘要 */}
          <div className="grid gap-4 md:grid-cols-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{t('adminGlp.qaDashboard.card.openNc')}</CardTitle>
                <AlertTriangle className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{data.qa_plan_summary.open_nc_count}</div>
                <p className="text-xs text-muted-foreground">{t('adminGlp.qaDashboard.card.openNcHint')}</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{t('adminGlp.qaDashboard.card.overdueNc')}</CardTitle>
                <AlertTriangle className="h-4 w-4 text-destructive" />
              </CardHeader>
              <CardContent>
                <div className={`text-2xl font-bold ${data.qa_plan_summary.overdue_nc_count > 0 ? 'text-destructive' : ''}`}>
                  {data.qa_plan_summary.overdue_nc_count}
                </div>
                <p className="text-xs text-muted-foreground">{t('adminGlp.qaDashboard.card.overdueNcHint')}</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{t('adminGlp.qaDashboard.card.activeSop')}</CardTitle>
                <BookOpen className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{data.qa_plan_summary.active_sop_count}</div>
                <p className="text-xs text-muted-foreground">{t('adminGlp.qaDashboard.card.activeSopHint')}</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{t('adminGlp.qaDashboard.card.inspectionReports')}</CardTitle>
                <Calendar className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {data.qa_plan_summary.inspection_by_status.reduce((sum, s) => sum + s.count, 0)}
                </div>
                <p className="text-xs text-muted-foreground">
                  {formatStatusList(data.qa_plan_summary.inspection_by_status, t)}
                </p>
              </CardContent>
            </Card>
          </div>

          {/* 動物實驗概覽 */}
          <div className="grid gap-4 md:grid-cols-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{t('adminGlp.qaDashboard.card.totalAnimals')}</CardTitle>
                <Stethoscope className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{data.animal_summary.total}</div>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{t('adminGlp.qaDashboard.card.inExperiment')}</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{data.animal_summary.in_experiment}</div>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{t('adminGlp.qaDashboard.card.euthanized')}</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{data.animal_summary.euthanized}</div>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{t('adminGlp.qaDashboard.card.experimentCompleted')}</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{data.animal_summary.completed}</div>
              </CardContent>
            </Card>
          </div>

          <div className="grid gap-6 lg:grid-cols-2">
            {/* 計畫狀態分布 */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <FileText className="h-5 w-5" />
                  {t('adminGlp.qaDashboard.protocolStatus.title')}
                </CardTitle>
                <p className="text-sm text-muted-foreground">{t('adminGlp.qaDashboard.protocolStatus.description')}</p>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <SortableTableHead sortKey="display_name" currentSort={protocolSort.column} currentDirection={protocolSort.direction} onSort={toggleProtocolSort}>{t('adminGlp.shared.status')}</SortableTableHead>
                      <SortableTableHead sortKey="count" currentSort={protocolSort.column} currentDirection={protocolSort.direction} onSort={toggleProtocolSort} className="text-right">{t('adminGlp.qaDashboard.col.count')}</SortableTableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {(sortedProtocolStatus ?? protocolStatusRows ?? []).map((row) => (
                      <TableRow key={row.status}>
                        <TableCell>{row.display_name}</TableCell>
                        <TableCell className="text-right">{row.count}</TableCell>
                      </TableRow>
                    ))}
                    {data.protocol_status_summary.length === 0 && (
                      <TableEmptyRow colSpan={2} icon={FileText} title={t('adminGlp.qaDashboard.noData')} />
                    )}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>

            {/* 稽核摘要（7 日內） */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Shield className="h-5 w-5" />
                  {t('adminGlp.qaDashboard.auditSummary.title')}
                </CardTitle>
                <p className="text-sm text-muted-foreground">
                  {t('adminGlp.qaDashboard.auditSummary.description')}
                </p>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <SortableTableHead sortKey="entity_type" currentSort={auditSort.column} currentDirection={auditSort.direction} onSort={toggleAuditSort}>{t('adminGlp.qaDashboard.col.entityType')}</SortableTableHead>
                      <SortableTableHead sortKey="count" currentSort={auditSort.column} currentDirection={auditSort.direction} onSort={toggleAuditSort} className="text-right">{t('adminGlp.qaDashboard.col.recordCount')}</SortableTableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {(sortedAudit ?? data.audit_summary).map((row) => (
                      <TableRow key={row.entity_type}>
                        <TableCell>{entityLabel(row.entity_type, t)}</TableCell>
                        <TableCell className="text-right">{row.count}</TableCell>
                      </TableRow>
                    ))}
                    {data.audit_summary.length === 0 && (
                      <TableEmptyRow colSpan={2} icon={Shield} title={t('adminGlp.qaDashboard.noData')} />
                    )}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          </div>

          {/* 動物狀態分布 */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Stethoscope className="h-5 w-5" />
                {t('adminGlp.qaDashboard.animalStatus.title')}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <SortableTableHead sortKey="display_name" currentSort={animalSort.column} currentDirection={animalSort.direction} onSort={toggleAnimalSort}>{t('adminGlp.shared.status')}</SortableTableHead>
                    <SortableTableHead sortKey="count" currentSort={animalSort.column} currentDirection={animalSort.direction} onSort={toggleAnimalSort} className="text-right">{t('adminGlp.qaDashboard.col.count')}</SortableTableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {(sortedAnimalStatus ?? animalStatusRows ?? []).map((row) => (
                    <TableRow key={row.status}>
                      <TableCell>{row.display_name}</TableCell>
                      <TableCell className="text-right">{row.count}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </>
      ) : null}
    </div>
  )
}
