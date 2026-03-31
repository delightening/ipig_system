import { useQuery } from '@tanstack/react-query'
import { FileText, ClipboardList, Shield, Stethoscope } from 'lucide-react'
import api from '@/lib/api'
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
import { SkeletonPulse } from '@/components/ui/skeleton'
import { STALE_TIME } from '@/lib/query'
import { useTableSort } from '@/hooks/useTableSort'

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

interface QauDashboard {
  protocol_status_summary: ProtocolStatusCount[]
  review_progress: ReviewProgressSummary
  audit_summary: AuditEntityCount[]
  animal_summary: AnimalSummary
}

export function QAUDashboardPage() {
  const { data, isLoading, error } = useQuery({
    queryKey: ['qau-dashboard'],
    queryFn: async () => {
      const res = await api.get<QauDashboard>('/qau/dashboard')
      return res.data
    },
    staleTime: STALE_TIME.LIST,
  })

  const {
    sortedData: sortedProtocolStatus, sort: protocolSort, toggleSort: toggleProtocolSort,
  } = useTableSort(data?.protocol_status_summary)
  const {
    sortedData: sortedAudit, sort: auditSort, toggleSort: toggleAuditSort,
  } = useTableSort(data?.audit_summary)
  const {
    sortedData: sortedAnimalStatus, sort: animalSort, toggleSort: toggleAnimalSort,
  } = useTableSort(data?.animal_summary?.by_status)

  if (error) {
    return (
      <div className="p-6">
        <div className="text-destructive">載入失敗：{(error as Error).message}</div>
      </div>
    )
  }

  return (
    <div className="p-6 space-y-6">
      <PageHeader
        title="QAU 品質保證儀表板"
        description="GLP 合規：研究狀態、審查進度、稽核摘要、動物實驗概覽（唯讀）"
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
                <CardTitle className="text-sm font-medium">7 日內狀態變更</CardTitle>
                <ClipboardList className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {data.review_progress.status_changes_last_7_days}
                </div>
                <p className="text-xs text-muted-foreground">protocol_status_history</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">審查中計畫</CardTitle>
                <FileText className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{data.review_progress.protocols_in_review}</div>
                <p className="text-xs text-muted-foreground">
                  UNDER_REVIEW / VET_REVIEW / PRE_REVIEW / SUBMITTED / RESUBMITTED
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">待 PI 回覆</CardTitle>
                <FileText className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {data.review_progress.protocols_pending_pi_response}
                </div>
                <p className="text-xs text-muted-foreground">
                  REVISION_REQUIRED / PRE_REVIEW_REVISION_REQUIRED / VET_REVISION_REQUIRED
                </p>
              </CardContent>
            </Card>
          </div>

          {/* 動物實驗概覽 */}
          <div className="grid gap-4 md:grid-cols-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">動物總數</CardTitle>
                <Stethoscope className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{data.animal_summary.total}</div>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">實驗中</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{data.animal_summary.in_experiment}</div>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">安樂死</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{data.animal_summary.euthanized}</div>
              </CardContent>
            </Card>
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">實驗完成</CardTitle>
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
                  計畫狀態分布
                </CardTitle>
                <p className="text-sm text-muted-foreground">protocols 依 status 聚合</p>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <SortableTableHead sortKey="display_name" currentSort={protocolSort.column} currentDirection={protocolSort.direction} onSort={toggleProtocolSort}>狀態</SortableTableHead>
                      <SortableTableHead sortKey="count" currentSort={protocolSort.column} currentDirection={protocolSort.direction} onSort={toggleProtocolSort} className="text-right">數量</SortableTableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {(sortedProtocolStatus ?? data.protocol_status_summary).map((row) => (
                      <TableRow key={row.status}>
                        <TableCell>{row.display_name}</TableCell>
                        <TableCell className="text-right">{row.count}</TableCell>
                      </TableRow>
                    ))}
                    {data.protocol_status_summary.length === 0 && (
                      <TableRow>
                        <TableCell colSpan={2} className="text-center text-muted-foreground">
                          無資料
                        </TableCell>
                      </TableRow>
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
                  稽核活動摘要（7 日內）
                </CardTitle>
                <p className="text-sm text-muted-foreground">
                  user_activity_logs 依 entity_type 聚合
                </p>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <SortableTableHead sortKey="entity_type" currentSort={auditSort.column} currentDirection={auditSort.direction} onSort={toggleAuditSort}>實體類型</SortableTableHead>
                      <SortableTableHead sortKey="count" currentSort={auditSort.column} currentDirection={auditSort.direction} onSort={toggleAuditSort} className="text-right">筆數</SortableTableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {(sortedAudit ?? data.audit_summary).map((row) => (
                      <TableRow key={row.entity_type}>
                        <TableCell>{row.entity_type}</TableCell>
                        <TableCell className="text-right">{row.count}</TableCell>
                      </TableRow>
                    ))}
                    {data.audit_summary.length === 0 && (
                      <TableRow>
                        <TableCell colSpan={2} className="text-center text-muted-foreground">
                          無資料
                        </TableCell>
                      </TableRow>
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
                動物狀態分布
              </CardTitle>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <SortableTableHead sortKey="display_name" currentSort={animalSort.column} currentDirection={animalSort.direction} onSort={toggleAnimalSort}>狀態</SortableTableHead>
                    <SortableTableHead sortKey="count" currentSort={animalSort.column} currentDirection={animalSort.direction} onSort={toggleAnimalSort} className="text-right">數量</SortableTableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {(sortedAnimalStatus ?? data.animal_summary.by_status).map((row) => (
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
