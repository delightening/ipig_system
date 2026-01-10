import { Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import api, { ProtocolListItem, ProtocolStatus, protocolStatusNames } from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Button } from '@/components/ui/button'
import { Eye, Loader2, FileText, Calendar, Building } from 'lucide-react'
import { formatDate } from '@/lib/utils'

const statusColors: Record<ProtocolStatus, 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline'> = {
  DRAFT: 'secondary',
  SUBMITTED: 'default',
  PRE_REVIEW: 'default',
  UNDER_REVIEW: 'warning',
  REVISION_REQUIRED: 'destructive',
  RESUBMITTED: 'default',
  APPROVED: 'success',
  APPROVED_WITH_CONDITIONS: 'success',
  DEFERRED: 'secondary',
  REJECTED: 'destructive',
  SUSPENDED: 'destructive',
  CLOSED: 'outline',
}

export function MyProjectsPage() {
  const { data: projects, isLoading } = useQuery({
    queryKey: ['my-projects'],
    queryFn: async () => {
      const response = await api.get<ProtocolListItem[]>('/my-projects')
      return response.data
    },
  })

  const getStatusBadge = (status: ProtocolStatus) => {
    return (
      <Badge variant={statusColors[status]}>
        {protocolStatusNames[status]}
      </Badge>
    )
  }

  // 統計數據
  const stats = {
    total: projects?.length || 0,
    approved: projects?.filter(p => p.status === 'APPROVED' || p.status === 'APPROVED_WITH_CONDITIONS').length || 0,
    underReview: projects?.filter(p => ['SUBMITTED', 'PRE_REVIEW', 'UNDER_REVIEW', 'RESUBMITTED'].includes(p.status)).length || 0,
    draft: projects?.filter(p => p.status === 'DRAFT').length || 0,
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">我的計劃</h1>
        <p className="text-muted-foreground">
          查看您參與或主持的 IACUC 計劃
        </p>
      </div>

      {/* 統計卡片 */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">全部計劃</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-green-600">已核准</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">{stats.approved}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-yellow-600">審查中</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-yellow-600">{stats.underReview}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-slate-500">草稿</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-slate-500">{stats.draft}</div>
          </CardContent>
        </Card>
      </div>

      {/* 計劃列表 */}
      <Card>
        <CardHeader>
          <CardTitle>計劃清單</CardTitle>
          <CardDescription>您參與的所有 IACUC 計劃</CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : projects && projects.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>申請案號</TableHead>
                  <TableHead>IACUC No.</TableHead>
                  <TableHead>委託人</TableHead>
                  <TableHead>委託單位</TableHead>
                  <TableHead>審查狀態</TableHead>
                  <TableHead>計畫名稱</TableHead>
                  <TableHead>起迄執行日期</TableHead>
                  <TableHead className="text-right">詳細內容</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {projects.map((project) => (
                  <TableRow key={project.id}>
                    <TableCell className="font-mono font-medium">
                      {project.protocol_no}
                    </TableCell>
                    <TableCell className="font-mono text-orange-600 font-semibold">
                      {project.iacuc_no || '-'}
                    </TableCell>
                    <TableCell>{project.pi_name}</TableCell>
                    <TableCell>{project.pi_organization || '-'}</TableCell>
                    <TableCell>{getStatusBadge(project.status)}</TableCell>
                    <TableCell className="max-w-[200px]">
                      <div className="truncate" title={project.title}>
                        {project.title}
                      </div>
                    </TableCell>
                    <TableCell>
                      {project.start_date && project.end_date ? (
                        <div className="flex items-center gap-1 text-sm">
                          <Calendar className="h-3 w-3" />
                          {formatDate(project.start_date)} ~ {formatDate(project.end_date)}
                        </div>
                      ) : (
                        '-'
                      )}
                    </TableCell>
                    <TableCell className="text-right">
                      <Button variant="outline" size="sm" asChild>
                        <Link to={`/my-projects/${project.id}`}>
                          <Eye className="mr-2 h-4 w-4" />
                          檢視
                        </Link>
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <div className="text-center py-12">
              <FileText className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
              <h3 className="text-lg font-semibold mb-2">尚無計劃資料</h3>
              <p className="text-muted-foreground">
                您目前沒有參與任何 IACUC 計劃
              </p>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
