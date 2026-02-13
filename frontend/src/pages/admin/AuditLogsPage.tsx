import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
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
import type { UserActivityLog } from '@/types/hr'

interface PaginatedResponse<T> {
  data: T[]
  total: number
  page: number
  per_page: number
  total_pages: number
}

// 事件類別對應中文
const categoryLabels: Record<string, string> = {
  ERP: 'ERP',
  AUP: '計畫書',
  ANIMAL: '實驗動物',
  SYSTEM: '系統',
}

// 事件類型對應中文顯示
const eventTypeLabels: Record<string, { label: string; color: string }> = {
  // 通用操作
  DOC_CREATE: { label: '建立單據', color: 'bg-green-500' },
  DOC_UPDATE: { label: '更新單據', color: 'bg-blue-500' },
  DOC_DELETE: { label: '刪除單據', color: 'bg-red-500' },
  DOC_SUBMIT: { label: '送審單據', color: 'bg-yellow-500' },
  DOC_APPROVE: { label: '核准單據', color: 'bg-purple-500' },
  DOC_CANCEL: { label: '作廢單據', color: 'bg-gray-500' },
  PARTNER_CREATE: { label: '建立夥伴', color: 'bg-green-500' },
  PARTNER_UPDATE: { label: '更新夥伴', color: 'bg-blue-500' },
  PARTNER_DELETE: { label: '刪除夥伴', color: 'bg-red-500' },
  PRODUCT_CREATE: { label: '建立產品', color: 'bg-green-500' },
  PRODUCT_UPDATE: { label: '更新產品', color: 'bg-blue-500' },
  PRODUCT_DELETE: { label: '刪除產品', color: 'bg-red-500' },
  CATEGORY_CREATE: { label: '建立分類', color: 'bg-green-500' },
  WAREHOUSE_CREATE: { label: '建立倉庫', color: 'bg-green-500' },
  WAREHOUSE_UPDATE: { label: '更新倉庫', color: 'bg-blue-500' },
  WAREHOUSE_DELETE: { label: '刪除倉庫', color: 'bg-red-500' },
  ROLE_CREATE: { label: '建立角色', color: 'bg-green-500' },
  ROLE_UPDATE: { label: '更新角色', color: 'bg-blue-500' },
  ROLE_DELETE: { label: '刪除角色', color: 'bg-red-500' },
  // 動物管理
  PIG_CREATE: { label: '建立豬隻', color: 'bg-green-500' },
  PIG_UPDATE: { label: '更新豬隻', color: 'bg-blue-500' },
  PIG_DELETE: { label: '刪除豬隻', color: 'bg-red-500' },
  PIG_BATCH_ASSIGN: { label: '批次分配', color: 'bg-purple-500' },
  OBSERVATION_CREATE: { label: '建立觀察', color: 'bg-green-500' },
  OBSERVATION_UPDATE: { label: '更新觀察', color: 'bg-blue-500' },
  OBSERVATION_DELETE: { label: '刪除觀察', color: 'bg-red-500' },
  SURGERY_CREATE: { label: '建立手術', color: 'bg-green-500' },
  SURGERY_UPDATE: { label: '更新手術', color: 'bg-blue-500' },
  SURGERY_DELETE: { label: '刪除手術', color: 'bg-red-500' },
  WEIGHT_CREATE: { label: '記錄體重', color: 'bg-green-500' },
  WEIGHT_UPDATE: { label: '更新體重', color: 'bg-blue-500' },
  WEIGHT_DELETE: { label: '刪除體重', color: 'bg-red-500' },
  VACCINATION_CREATE: { label: '建立疫苗', color: 'bg-green-500' },
  VACCINATION_UPDATE: { label: '更新疫苗', color: 'bg-blue-500' },
  VACCINATION_DELETE: { label: '刪除疫苗', color: 'bg-red-500' },
  // 計畫書
  PROTOCOL_CREATE: { label: '建立計畫書', color: 'bg-green-500' },
  PROTOCOL_UPDATE: { label: '更新計畫書', color: 'bg-blue-500' },
  PROTOCOL_SUBMIT: { label: '送審計畫書', color: 'bg-yellow-500' },
  PROTOCOL_APPROVE: { label: '核准計畫書', color: 'bg-purple-500' },
  PROTOCOL_REJECT: { label: '駁回計畫書', color: 'bg-red-500' },
  PROTOCOL_STATUS_CHANGE: { label: '狀態變更', color: 'bg-orange-500' },
  PROTOCOL_REVIEWER_ASSIGN: { label: '指派審查委員', color: 'bg-indigo-500' },
}

// 實體類型對應中文
const entityTypeLabels: Record<string, string> = {
  document: '單據',
  partner: '夥伴',
  product: '產品',
  product_category: '產品分類',
  warehouse: '倉庫',
  role: '角色',
  pig: '豬隻',
  pig_observation: '觀察記錄',
  pig_surgery: '手術記錄',
  pig_weight: '體重記錄',
  pig_vaccination: '疫苗記錄',
  pig_sacrifice: '犧牲記錄',
  protocol: '計畫書',
}

export function AuditLogsPage() {
  const getDefaultDateFrom = () => {
    const now = new Date()
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-01`
  }
  const getDefaultDateTo = () => {
    const now = new Date()
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`
  }

  const [dateFrom, setDateFrom] = useState(getDefaultDateFrom)
  const [dateTo, setDateTo] = useState(getDefaultDateTo)
  const [categoryFilter, setCategoryFilter] = useState<string>('all')
  const [entityTypeFilter, setEntityTypeFilter] = useState<string>('all')
  const [selectedLog, setSelectedLog] = useState<UserActivityLog | null>(null)

  const { data: activityLogs, isLoading } = useQuery({
    queryKey: ['audit-logs-activities', dateFrom, dateTo, categoryFilter, entityTypeFilter],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (dateFrom) params.set('from', dateFrom)
      if (dateTo) params.set('to', dateTo)
      if (categoryFilter !== 'all') params.set('category', categoryFilter)
      if (entityTypeFilter !== 'all') params.set('entity_type', entityTypeFilter)

      const response = await api.get<PaginatedResponse<UserActivityLog>>(
        `/admin/audit/activities?${params.toString()}`
      )
      return response.data
    },
  })

  const formatDateTime = (dateStr: string) => {
    return format(new Date(dateStr), 'yyyy/MM/dd HH:mm:ss', { locale: zhTW })
  }

  const getEventBadge = (eventType: string) => {
    const config = eventTypeLabels[eventType] || { label: eventType, color: 'bg-gray-500' }
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
              <Label>事件類別</Label>
              <Select
                value={categoryFilter}
                onValueChange={setCategoryFilter}
              >
                <SelectTrigger>
                  <SelectValue placeholder="全部類別" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部類別</SelectItem>
                  <SelectItem value="ERP">ERP</SelectItem>
                  <SelectItem value="AUP">計畫書</SelectItem>
                  <SelectItem value="ANIMAL">實驗動物</SelectItem>
                  <SelectItem value="SYSTEM">系統</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>實體類型</Label>
              <Select
                value={entityTypeFilter}
                onValueChange={setEntityTypeFilter}
              >
                <SelectTrigger>
                  <SelectValue placeholder="全部類型" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部類型</SelectItem>
                  <SelectItem value="document">單據</SelectItem>
                  <SelectItem value="product">產品</SelectItem>
                  <SelectItem value="warehouse">倉庫</SelectItem>
                  <SelectItem value="partner">夥伴</SelectItem>
                  <SelectItem value="role">角色</SelectItem>
                  <SelectItem value="pig">豬隻</SelectItem>
                  <SelectItem value="protocol">計畫書</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>開始日期</Label>
              <Input
                type="date"
                value={dateFrom}
                onChange={(e) => setDateFrom(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>結束日期</Label>
              <Input
                type="date"
                value={dateTo}
                onChange={(e) => setDateTo(e.target.value)}
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
              <TableHead>類別</TableHead>
              <TableHead>事件</TableHead>
              <TableHead>實體類型</TableHead>
              <TableHead>實體名稱</TableHead>
              <TableHead className="text-right">詳情</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                </TableCell>
              </TableRow>
            ) : activityLogs?.data && activityLogs.data.length > 0 ? (
              activityLogs.data.map((log) => (
                <TableRow key={log.id}>
                  <TableCell className="text-sm whitespace-nowrap">
                    {formatDateTime(log.created_at)}
                  </TableCell>
                  <TableCell>
                    <div>
                      <p className="font-medium">{log.actor_display_name || '-'}</p>
                      <p className="text-xs text-muted-foreground">{log.actor_email || ''}</p>
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline">
                      {categoryLabels[log.event_category] || log.event_category}
                    </Badge>
                  </TableCell>
                  <TableCell>{getEventBadge(log.event_type)}</TableCell>
                  <TableCell>
                    <Badge variant="secondary">
                      {entityTypeLabels[log.entity_type || ''] || log.entity_type || '-'}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-sm">
                    {log.entity_display_name || (log.entity_id ? `${log.entity_id.slice(0, 8)}...` : '-')}
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
                <TableCell colSpan={7} className="text-center py-8">
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
                  <p className="font-medium">{formatDateTime(selectedLog.created_at)}</p>
                </div>
                <div>
                  <Label className="text-muted-foreground">操作者</Label>
                  <p className="font-medium">{selectedLog.actor_display_name || '-'}</p>
                  <p className="text-sm text-muted-foreground">{selectedLog.actor_email || ''}</p>
                </div>
                <div>
                  <Label className="text-muted-foreground">事件類別</Label>
                  <p className="font-medium">{categoryLabels[selectedLog.event_category] || selectedLog.event_category}</p>
                </div>
                <div>
                  <Label className="text-muted-foreground">事件類型</Label>
                  <div className="mt-1">{getEventBadge(selectedLog.event_type)}</div>
                </div>
                <div>
                  <Label className="text-muted-foreground">實體類型</Label>
                  <p className="font-medium">{entityTypeLabels[selectedLog.entity_type || ''] || selectedLog.entity_type || '-'}</p>
                </div>
                <div>
                  <Label className="text-muted-foreground">實體名稱</Label>
                  <p className="font-medium">{selectedLog.entity_display_name || '-'}</p>
                </div>
                {selectedLog.entity_id && (
                  <div className="col-span-2">
                    <Label className="text-muted-foreground">實體 ID</Label>
                    <p className="font-mono text-sm">{selectedLog.entity_id}</p>
                  </div>
                )}
                {selectedLog.ip_address && (
                  <div>
                    <Label className="text-muted-foreground">IP 位址</Label>
                    <p className="text-sm">{selectedLog.ip_address}</p>
                  </div>
                )}
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
