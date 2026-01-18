import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { ProtocolListItem, ProtocolStatus, protocolStatusNames } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Plus, Search, Eye, Edit, Loader2, FileText, X, Trash2 } from 'lucide-react'
import { formatDate } from '@/lib/utils'
import { toast } from '@/components/ui/use-toast'

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
  DELETED: 'outline',
}

export function ProtocolsPage() {
  const queryClient = useQueryClient()
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>('all')

  const { data: protocols, isLoading } = useQuery({
    queryKey: ['protocols', statusFilter, search],
    queryFn: async () => {
      let params = ''
      if (statusFilter && statusFilter !== 'all') params += `status=${statusFilter}&`
      if (search) params += `keyword=${encodeURIComponent(search)}&`
      const response = await api.get<ProtocolListItem[]>(`/protocols?${params}`)
      // 雙重保險：即使後端返回了 DELETED 狀態，前端也要過濾掉
      return response.data.filter(p => p.status !== 'DELETED')
    },
  })

  const clearFilters = () => {
    setSearch('')
    setStatusFilter('all')
  }

  const hasFilters = search || (statusFilter && statusFilter !== 'all')

  const getStatusBadge = (status: ProtocolStatus) => {
    return (
      <Badge variant={statusColors[status]}>
        {protocolStatusNames[status]}
      </Badge>
    )
  }

  const canEditProtocol = (status: ProtocolStatus | string) => {
    const normalized = String(status).toUpperCase()
    return normalized === 'DRAFT' || normalized === 'REVISION_REQUIRED'
  }

  const canDeleteProtocol = (status: ProtocolStatus | string) => {
    const normalized = String(status).toUpperCase()
    return normalized === 'DRAFT'
  }

  const deleteMutation = useMutation({
    mutationFn: async (protocolId: string) => {
      return api.post(`/protocols/${protocolId}/status`, { to_status: 'DELETED' })
    },
    onSuccess: () => {
      toast({ title: '成功', description: '計畫書已刪除' })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '刪除失敗',
        variant: 'destructive'
      })
    },
  })

  const handleDelete = (protocolId: string, title: string) => {
    if (confirm(`確定要刪除計畫書「${title}」嗎？`)) {
      deleteMutation.mutate(protocolId)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">計畫書管理</h1>
          <p className="text-muted-foreground">
            管理 IACUC 動物試驗計畫書（AUP）
          </p>
        </div>
        <Button asChild>
          <Link to="/protocols/new">
            <Plus className="mr-2 h-4 w-4" />
            新增計畫書
          </Link>
        </Button>
      </div>

      <div className="space-y-4">
        <div className="flex flex-wrap gap-4 items-end">
          <div className="relative flex-1 min-w-[200px] max-w-sm">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="搜尋計畫書編號或標題..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-9"
            />
          </div>
          <Select value={statusFilter} onValueChange={setStatusFilter}>
            <SelectTrigger className="w-40">
              <SelectValue placeholder="全部狀態" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部狀態</SelectItem>
              {Object.entries(protocolStatusNames)
                .filter(([key]) => key !== 'DELETED')
                .map(([key, name]) => (
                  <SelectItem key={key} value={key}>
                    {name}
                  </SelectItem>
                ))}
            </SelectContent>
          </Select>

          {hasFilters && (
            <Button variant="ghost" size="sm" onClick={clearFilters}>
              <X className="h-4 w-4 mr-1" />
              清除篩選
            </Button>
          )}
        </div>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>IACUC No.</TableHead>
              <TableHead>計畫書標題</TableHead>
              <TableHead>計畫書主持人</TableHead>
              <TableHead>所屬單位</TableHead>
              <TableHead>狀態</TableHead>
              <TableHead>執行期間</TableHead>
              <TableHead>建立日期</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                </TableCell>
              </TableRow>
            ) : protocols && protocols.length > 0 ? (
              protocols.map((protocol) => (
                <TableRow key={protocol.id}>
                  <TableCell className="font-mono">
                    {protocol.iacuc_no ? (
                      <Link 
                        to={`/protocols/${protocol.id}`}
                        className="text-orange-600 hover:text-orange-700 hover:underline cursor-pointer"
                      >
                        {protocol.iacuc_no}
                      </Link>
                    ) : '-'}
                  </TableCell>
                  <TableCell className="max-w-[200px] truncate">
                    <Link 
                      to={`/protocols/${protocol.id}`}
                      className="text-blue-600 hover:text-blue-700 hover:underline cursor-pointer"
                    >
                      {protocol.title}
                    </Link>
                  </TableCell>
                  <TableCell>{protocol.pi_name}</TableCell>
                  <TableCell>{protocol.pi_organization || '-'}</TableCell>
                  <TableCell>{getStatusBadge(protocol.status)}</TableCell>
                  <TableCell>
                    {protocol.start_date && protocol.end_date
                      ? `${formatDate(protocol.start_date)} ~ ${formatDate(protocol.end_date)}`
                      : '-'}
                  </TableCell>
                  <TableCell>{formatDate(protocol.created_at)}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-1">
                      <Button variant="ghost" size="icon" asChild title="查看">
                        <Link to={`/protocols/${protocol.id}`}>
                          <Eye className="h-4 w-4" />
                        </Link>
                      </Button>
                      {canEditProtocol(protocol.status) && (
                        <Button variant="ghost" size="icon" asChild title="編輯">
                          <Link to={`/protocols/${protocol.id}/edit`}>
                            <Edit className="h-4 w-4" />
                          </Link>
                        </Button>
                      )}
                      {canDeleteProtocol(protocol.status) && (
                        <Button
                          variant="ghost"
                          size="icon"
                          title="刪除"
                          onClick={() => handleDelete(protocol.id, protocol.title)}
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={8} className="text-center py-8">
                  <FileText className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                  <p className="text-muted-foreground">尚無計畫書資料</p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}








