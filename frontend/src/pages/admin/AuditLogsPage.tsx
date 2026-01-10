import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Loader2, History, Search, Eye, FileJson } from 'lucide-react'

interface AuditLog {
  id: string
  actor_user_id: string
  actor_email: string
  actor_name: string
  action: string
  entity_type: string
  entity_id: string
  before_data?: Record<string, unknown>
  after_data?: Record<string, unknown>
  created_at: string
}

interface AuditLogQuery {
  entity_type?: string
  action?: string
  actor_user_id?: string
  start_date?: string
  end_date?: string
}

const actionLabels: Record<string, { label: string; color: string }> = {
  create: { label: '創建', color: 'bg-green-500' },
  update: { label: '更新', color: 'bg-blue-500' },
  delete: { label: '刪除', color: 'bg-red-500' },
  approve: { label: '核准', color: 'bg-purple-500' },
  submit: { label: '送審', color: 'bg-yellow-500' },
  cancel: { label: '作廢', color: 'bg-gray-500' },
  login: { label: '登入', color: 'bg-cyan-500' },
  logout: { label: '登出', color: 'bg-slate-500' },
}

const entityLabels: Record<string, string> = {
  user: '使用者',
  role: '角色',
  warehouse: '倉庫',
  product: '產品',
  partner: '夥伴',
  document: '單據',
  stock: '庫存',
}

export function AuditLogsPage() {
  const [query, setQuery] = useState<AuditLogQuery>({})
  const [selectedLog, setSelectedLog] = useState<AuditLog | null>(null)

  const { data: logs, isLoading } = useQuery({
    queryKey: ['audit-logs', query],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (query.entity_type) params.set('entity_type', query.entity_type)
      if (query.action) params.set('action', query.action)
      if (query.start_date) params.set('start_date', query.start_date)
      if (query.end_date) params.set('end_date', query.end_date)
      
      const response = await api.get<AuditLog[]>(`/audit-logs?${params.toString()}`)
      return response.data
    },
  })

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleString('zh-TW', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    })
  }

  const getActionBadge = (action: string) => {
    const config = actionLabels[action] || { label: action, color: 'bg-gray-500' }
    return (
      <Badge className={`${config.color} text-white`}>
        {config.label}
      </Badge>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">審計日誌</h1>
        <p className="text-muted-foreground">追蹤系統操作記錄與變更歷史</p>
      </div>

      {/* 篩選條件 */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Search className="h-5 w-5" />
            搜尋條件
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="space-y-2">
              <Label>實體類型</Label>
              <Select
                value={query.entity_type || 'all'}
                onValueChange={(value) => setQuery({ ...query, entity_type: value === 'all' ? undefined : value })}
              >
                <SelectTrigger>
                  <SelectValue placeholder="全部類型" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部類型</SelectItem>
                  <SelectItem value="user">使用者</SelectItem>
                  <SelectItem value="role">角色</SelectItem>
                  <SelectItem value="warehouse">倉庫</SelectItem>
                  <SelectItem value="product">產品</SelectItem>
                  <SelectItem value="partner">夥伴</SelectItem>
                  <SelectItem value="document">單據</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>操作類型</Label>
              <Select
                value={query.action || 'all'}
                onValueChange={(value) => setQuery({ ...query, action: value === 'all' ? undefined : value })}
              >
                <SelectTrigger>
                  <SelectValue placeholder="全部操作" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部操作</SelectItem>
                  <SelectItem value="create">創建</SelectItem>
                  <SelectItem value="update">更新</SelectItem>
                  <SelectItem value="delete">刪除</SelectItem>
                  <SelectItem value="approve">核准</SelectItem>
                  <SelectItem value="submit">送審</SelectItem>
                  <SelectItem value="cancel">作廢</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>開始日期</Label>
              <Input
                type="date"
                value={query.start_date || ''}
                onChange={(e) => setQuery({ ...query, start_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label>結束日期</Label>
              <Input
                type="date"
                value={query.end_date || ''}
                onChange={(e) => setQuery({ ...query, end_date: e.target.value })}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 日誌列表 */}
      <div className="rounded-md border bg-white">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>時間</TableHead>
              <TableHead>操作者</TableHead>
              <TableHead>操作</TableHead>
              <TableHead>實體類型</TableHead>
              <TableHead>實體 ID</TableHead>
              <TableHead className="text-right">詳情</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={6} className="text-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                </TableCell>
              </TableRow>
            ) : logs && logs.length > 0 ? (
              logs.map((log) => (
                <TableRow key={log.id}>
                  <TableCell className="text-sm">
                    {formatDate(log.created_at)}
                  </TableCell>
                  <TableCell>
                    <div>
                      <p className="font-medium">{log.actor_name}</p>
                      <p className="text-xs text-muted-foreground">{log.actor_email}</p>
                    </div>
                  </TableCell>
                  <TableCell>{getActionBadge(log.action)}</TableCell>
                  <TableCell>
                    <Badge variant="outline">
                      {entityLabels[log.entity_type] || log.entity_type}
                    </Badge>
                  </TableCell>
                  <TableCell className="font-mono text-xs">
                    {log.entity_id.slice(0, 8)}...
                  </TableCell>
                  <TableCell className="text-right">
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => setSelectedLog(log)}
                    >
                      <Eye className="h-4 w-4" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={6} className="text-center py-8">
                  <History className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                  <p className="text-muted-foreground">尚無審計日誌</p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>

      {/* 詳情對話框 */}
      <Dialog open={!!selectedLog} onOpenChange={() => setSelectedLog(null)}>
        <DialogContent className="max-w-3xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <FileJson className="h-5 w-5" />
              審計日誌詳情
            </DialogTitle>
          </DialogHeader>
          {selectedLog && (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label className="text-muted-foreground">操作時間</Label>
                  <p className="font-medium">{formatDate(selectedLog.created_at)}</p>
                </div>
                <div>
                  <Label className="text-muted-foreground">操作者</Label>
                  <p className="font-medium">{selectedLog.actor_name}</p>
                  <p className="text-sm text-muted-foreground">{selectedLog.actor_email}</p>
                </div>
                <div>
                  <Label className="text-muted-foreground">操作類型</Label>
                  <div className="mt-1">{getActionBadge(selectedLog.action)}</div>
                </div>
                <div>
                  <Label className="text-muted-foreground">實體類型</Label>
                  <p className="font-medium">{entityLabels[selectedLog.entity_type] || selectedLog.entity_type}</p>
                </div>
                <div className="col-span-2">
                  <Label className="text-muted-foreground">實體 ID</Label>
                  <p className="font-mono text-sm">{selectedLog.entity_id}</p>
                </div>
              </div>

              {selectedLog.before_data && (
                <div>
                  <Label className="text-muted-foreground">變更前資料</Label>
                  <pre className="mt-1 p-3 bg-red-50 border border-red-200 rounded-md text-sm overflow-x-auto">
                    {JSON.stringify(selectedLog.before_data, null, 2)}
                  </pre>
                </div>
              )}

              {selectedLog.after_data && (
                <div>
                  <Label className="text-muted-foreground">變更後資料</Label>
                  <pre className="mt-1 p-3 bg-green-50 border border-green-200 rounded-md text-sm overflow-x-auto">
                    {JSON.stringify(selectedLog.after_data, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}
