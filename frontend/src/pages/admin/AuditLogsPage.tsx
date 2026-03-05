import { useState } from 'react'
import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
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
import { Loader2, History, Search, Eye, FileJson, ChevronLeft, ChevronRight, Download, FileText } from 'lucide-react'
import type { User } from '@/types/auth'
import type { UserActivityLog } from '@/types/hr'
import { logger } from '@/lib/logger'

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
  ANIMAL_CREATE: { label: '建立動物', color: 'bg-green-500' },
  ANIMAL_UPDATE: { label: '更新動物', color: 'bg-blue-500' },
  ANIMAL_DELETE: { label: '刪除動物', color: 'bg-red-500' },
  ANIMAL_BATCH_ASSIGN: { label: '批次分配', color: 'bg-purple-500' },
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
  SACRIFICE_UPSERT: { label: '犧牲/安樂死', color: 'bg-orange-600' },
  PATHOLOGY_UPSERT: { label: '病理報告', color: 'bg-violet-500' },
  VET_RECOMMENDATION_ADD: { label: '獸醫建議', color: 'bg-teal-500' },
  MEDICAL_EXPORT: { label: '匯出醫療資料', color: 'bg-cyan-500' },
  BLOOD_TEST_CREATE: { label: '建立血液檢查', color: 'bg-green-500' },
  BLOOD_TEST_UPDATE: { label: '更新血液檢查', color: 'bg-blue-500' },
  BLOOD_TEST_DELETE: { label: '刪除血液檢查', color: 'bg-red-500' },
  TEMPLATE_CREATE: { label: '建立模板', color: 'bg-green-500' },
  TEMPLATE_UPDATE: { label: '更新模板', color: 'bg-blue-500' },
  TEMPLATE_DELETE: { label: '刪除模板', color: 'bg-red-500' },
  PANEL_CREATE: { label: '建立組合', color: 'bg-green-500' },
  PANEL_UPDATE: { label: '更新組合', color: 'bg-blue-500' },
  PANEL_DELETE: { label: '刪除組合', color: 'bg-red-500' },
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
  animal: '動物',
  animal_observation: '觀察記錄',
  animal_surgery: '手術記錄',
  animal_weight: '體重記錄',
  animal_vaccination: '疫苗記錄',
  animal_sacrifice: '犧牲記錄',
  animal_pathology: '病理報告',
  animal_blood_test: '血液檢查',
  blood_test_template: '血檢模板',
  blood_test_panel: '血檢組合',
  vet_recommendation: '獸醫建議',
  protocol: '計畫書',
}

// 事件類別 → 可選實體類型 配對
const categoryEntityMap: Record<string, { value: string; label: string }[]> = {
  all: [
    { value: 'document', label: '單據' },
    { value: 'product', label: '產品' },
    { value: 'warehouse', label: '倉庫' },
    { value: 'partner', label: '夥伴' },
    { value: 'blood_test_template', label: '血檢模板' },
    { value: 'blood_test_panel', label: '血檢組合' },
    { value: 'animal', label: '動物' },
    { value: 'animal_observation', label: '觀察記錄' },
    { value: 'animal_surgery', label: '手術記錄' },
    { value: 'animal_weight', label: '體重記錄' },
    { value: 'animal_vaccination', label: '疫苗記錄' },
    { value: 'animal_sacrifice', label: '犧牲記錄' },
    { value: 'animal_pathology', label: '病理報告' },
    { value: 'animal_blood_test', label: '血液檢查' },
    { value: 'vet_recommendation', label: '獸醫建議' },
    { value: 'protocol', label: '計畫書' },
    { value: 'role', label: '角色' },
  ],
  ERP: [
    { value: 'document', label: '單據' },
    { value: 'product', label: '產品' },
    { value: 'warehouse', label: '倉庫' },
    { value: 'partner', label: '夥伴' },
    { value: 'blood_test_template', label: '血檢模板' },
    { value: 'blood_test_panel', label: '血檢組合' },
  ],
  AUP: [
    { value: 'protocol', label: '計畫書' },
  ],
  ANIMAL: [
    { value: 'animal', label: '動物' },
    { value: 'animal_observation', label: '觀察記錄' },
    { value: 'animal_surgery', label: '手術記錄' },
    { value: 'animal_weight', label: '體重記錄' },
    { value: 'animal_vaccination', label: '疫苗記錄' },
    { value: 'animal_sacrifice', label: '犧牲記錄' },
    { value: 'animal_pathology', label: '病理報告' },
    { value: 'animal_blood_test', label: '血液檢查' },
    { value: 'vet_recommendation', label: '獸醫建議' },
  ],
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

  const { from: dateFrom, to: dateTo, setFrom: setDateFrom, setTo: setDateTo } = useDateRangeFilter({
    initialFrom: getDefaultDateFrom,
    initialTo: getDefaultDateTo,
  })
  const [categoryFilter, setCategoryFilter] = useState<string>('all')
  const [entityTypeFilter, setEntityTypeFilter] = useState<string>('all')
  const [userFilter, setUserFilter] = useState<string>('all')
  const [selectedLog, setSelectedLog] = useState<UserActivityLog | null>(null)
  const [currentPage, setCurrentPage] = useState(1)
  const [isExporting, setIsExporting] = useState(false)
  const perPage = 50

  // 根據事件類別取得可選的實體類型
  const availableEntityTypes = categoryEntityMap[categoryFilter] || categoryEntityMap.all

  const { data: users = [] } = useQuery({
    queryKey: ['users-list-audit'],
    queryFn: async () => {
      const response = await api.get<User[]>('/users?per_page=500')
      return response.data
    },
  })

  const { data: activityLogs, isLoading } = useQuery({
    queryKey: ['audit-logs-activities', dateFrom, dateTo, categoryFilter, entityTypeFilter, userFilter, currentPage],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (dateFrom) params.set('from', dateFrom)
      if (dateTo) params.set('to', dateTo)
      if (categoryFilter !== 'all') params.set('event_category', categoryFilter)
      if (entityTypeFilter !== 'all') params.set('entity_type', entityTypeFilter)
      if (userFilter !== 'all') params.set('user_id', userFilter)
      params.set('page', String(currentPage))
      params.set('per_page', String(perPage))

      const response = await api.get<PaginatedResponse<UserActivityLog>>(
        `/admin/audit/activities?${params.toString()}`
      )
      return response.data
    },
  })

  // 切換事件類別時，若目前實體類型不在新類別的可選清單中，則重設
  const handleCategoryChange = (val: string) => {
    setCategoryFilter(val)
    setCurrentPage(1)
    const newAvailable = categoryEntityMap[val] || categoryEntityMap.all
    if (entityTypeFilter !== 'all' && !newAvailable.some(e => e.value === entityTypeFilter)) {
      setEntityTypeFilter('all')
    }
  }
  const handleEntityTypeChange = (val: string) => {
    setEntityTypeFilter(val)
    setCurrentPage(1)
  }
  const handleUserChange = (val: string) => {
    setUserFilter(val)
    setCurrentPage(1)
  }
  const handleDateFromChange = (val: string) => {
    setDateFrom(val)
    setCurrentPage(1)
  }
  const handleDateToChange = (val: string) => {
    setDateTo(val)
    setCurrentPage(1)
  }

  // === 匯出相關函數 ===

  const buildExportParams = () => {
    const params = new URLSearchParams()
    if (dateFrom) params.set('from', dateFrom)
    if (dateTo) params.set('to', dateTo)
    if (categoryFilter !== 'all') params.set('event_category', categoryFilter)
    if (entityTypeFilter !== 'all') params.set('entity_type', entityTypeFilter)
    if (userFilter !== 'all') params.set('user_id', userFilter)
    return params.toString()
  }

  const handleExportCSV = async () => {
    setIsExporting(true)
    try {
      const response = await api.get<UserActivityLog[]>(
        `/admin/audit/activities/export?${buildExportParams()}`
      )
      const logs = response.data

      // 建立 CSV 內容（含 BOM 以確保 Excel 正確顯示中文）
      const headers = ['時間', '操作者', '操作者信箱', '類別', '事件類型', '實體類型', '實體名稱', 'IP 位址', '可疑']
      const csvRows = [headers.join(',')]
      for (const log of logs) {
        const evtLabel = eventTypeLabels[log.event_type]?.label || log.event_type
        const catLabel = categoryLabels[log.event_category] || log.event_category
        const row = [
          formatDateTime(log.created_at),
          `"${(log.actor_display_name || '').replace(/"/g, '""')}"`,
          `"${(log.actor_email || '').replace(/"/g, '""')}"`,
          catLabel,
          evtLabel,
          log.entity_type || '',
          `"${(log.entity_display_name || '').replace(/"/g, '""')}"`,
          log.ip_address || '',
          log.is_suspicious ? '是' : '否',
        ]
        csvRows.push(row.join(','))
      }

      const bom = '\uFEFF'
      const blob = new Blob([bom + csvRows.join('\n')], { type: 'text/csv;charset=utf-8;' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `操作日誌_${dateFrom}_${dateTo}.csv`
      a.click()
      URL.revokeObjectURL(url)
    } catch (err) {
      logger.error('匯出 CSV 失敗', err)
    } finally {
      setIsExporting(false)
    }
  }

  const handleExportPDF = async () => {
    setIsExporting(true)
    try {
      const response = await api.get<UserActivityLog[]>(
        `/admin/audit/activities/export?${buildExportParams()}`
      )
      const logs = response.data

      // 建立可列印的 HTML 頁面
      const tableRows = logs.map(log => {
        const evtLabel = eventTypeLabels[log.event_type]?.label || log.event_type
        const catLabel = categoryLabels[log.event_category] || log.event_category
        return `<tr>
          <td>${formatDateTime(log.created_at)}</td>
          <td>${log.actor_display_name || ''}</td>
          <td>${catLabel}</td>
          <td>${evtLabel}</td>
          <td>${log.entity_type || ''}</td>
          <td>${log.entity_display_name || ''}</td>
          <td>${log.ip_address || ''}</td>
        </tr>`
      }).join('')

      const html = `<!DOCTYPE html>
<html lang="zh-Hant">
<head>
  <meta charset="utf-8" />
  <title>操作日誌 ${dateFrom} ~ ${dateTo}</title>
  <style>
    body { font-family: 'Noto Sans TC', 'Microsoft JhengHei', sans-serif; font-size: 11px; margin: 20px; }
    h1 { font-size: 18px; margin-bottom: 4px; }
    p { color: #666; margin-bottom: 12px; }
    table { width: 100%; border-collapse: collapse; }
    th, td { border: 1px solid #ddd; padding: 4px 6px; text-align: left; }
    th { background: #f5f5f5; font-weight: 600; }
    tr:nth-child(even) { background: #fafafa; }
    @media print { body { margin: 0; } }
  </style>
</head>
<body>
  <h1>操作日誌報表</h1>
  <p>期間：${dateFrom} - ${dateTo} | 共 ${logs.length} 筆</p>
  <table>
    <thead>
      <tr><th>時間</th><th>操作者</th><th>類別</th><th>事件</th><th>實體類型</th><th>實體名稱</th><th>IP</th></tr>
    </thead>
    <tbody>${tableRows}</tbody>
  </table>
</body>
</html>`

      const printWindow = window.open('', '_blank')
      if (printWindow) {
        printWindow.document.write(html)
        printWindow.document.close()
        // 延遲觸發列印，讓樣式載入完成
        setTimeout(() => printWindow.print(), 300)
      }
    } catch (err) {
      logger.error('匯出 PDF 失敗', err)
    } finally {
      setIsExporting(false)
    }
  }

  const totalPages = activityLogs ? Math.ceil(activityLogs.total / perPage) : 0

  const formatDateTime = (dateStr: string) => {
    return new Date(dateStr).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei', year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit' })
  }

  const formatDateTimeDisplay = (dateStr: string) => {
    const d = new Date(dateStr)
    const datePart = d.toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei', year: 'numeric', month: '2-digit', day: '2-digit' })
    const timePart = d.toLocaleTimeString('zh-TW', { timeZone: 'Asia/Taipei', hour: '2-digit', minute: '2-digit', second: '2-digit' })
    return (
      <div className="block leading-tight">
        <div>{datePart}</div>
        <div>{timePart}</div>
      </div>
    )
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
        <h1 className="text-2xl md:text-3xl font-bold tracking-tight">操作日誌</h1>
        <p className="text-sm md:text-base text-muted-foreground">追蹤所有使用者的操作記錄與變更歷史</p>
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
          <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
            <div className="space-y-2">
              <Label>操作者</Label>
              <Select
                value={userFilter}
                onValueChange={handleUserChange}
              >
                <SelectTrigger>
                  <SelectValue placeholder="全部操作者" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部操作者</SelectItem>
                  {users.map((u) => (
                    <SelectItem key={u.id} value={u.id}>
                      {u.display_name || u.email}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>事件類別</Label>
              <Select
                value={categoryFilter}
                onValueChange={handleCategoryChange}
              >
                <SelectTrigger>
                  <SelectValue placeholder="全部類別" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部類別</SelectItem>
                  <SelectItem value="ERP">ERP</SelectItem>
                  <SelectItem value="AUP">計畫書</SelectItem>
                  <SelectItem value="ANIMAL">實驗動物</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>實體類型</Label>
              <Select
                value={entityTypeFilter}
                onValueChange={handleEntityTypeChange}
              >
                <SelectTrigger>
                  <SelectValue placeholder="全部類型" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部類型</SelectItem>
                  {availableEntityTypes.map((et) => (
                    <SelectItem key={et.value} value={et.value}>{et.label}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>開始日期</Label>
              <Input
                type="date"
                value={dateFrom}
                onChange={(e) => handleDateFromChange(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>結束日期</Label>
              <Input
                type="date"
                value={dateTo}
                onChange={(e) => handleDateToChange(e.target.value)}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* 匯出按鈕列 */}
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          disabled={isExporting}
          onClick={handleExportCSV}
        >
          {isExporting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Download className="h-4 w-4 mr-1" />}
          匯出 CSV
        </Button>
        <Button
          variant="outline"
          size="sm"
          disabled={isExporting}
          onClick={handleExportPDF}
        >
          {isExporting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <FileText className="h-4 w-4 mr-1" />}
          匯出 PDF
        </Button>
        {activityLogs && (
          <span className="text-sm text-muted-foreground ml-auto">
            共 {activityLogs.total} 筆紀錄
          </span>
        )}
      </div>

      {/* 日誌列表 */}
      <div className="rounded-md border bg-white overflow-x-auto">
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
                  <TableCell className="text-sm">
                    {formatDateTimeDisplay(log.created_at)}
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
                  <p className="text-muted-foreground">尚無操作日誌</p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>

        {/* 分頁控制列 */}
        {activityLogs && totalPages > 0 && (
          <div className="flex flex-col md:flex-row items-center justify-between px-4 py-3 border-t gap-2">
            <p className="text-sm text-muted-foreground">
              共 {activityLogs.total} 筆，第 {currentPage} / {totalPages} 頁
            </p>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                disabled={currentPage <= 1}
                onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
              >
                <ChevronLeft className="h-4 w-4 mr-1" />
                上一頁
              </Button>
              <Button
                variant="outline"
                size="sm"
                disabled={currentPage >= totalPages}
                onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
              >
                下一頁
                <ChevronRight className="h-4 w-4 ml-1" />
              </Button>
            </div>
          </div>
        )}
      </div>

      {/* 詳情對話框 */}
      <Dialog open={!!selectedLog} onOpenChange={() => setSelectedLog(null)}>
        <DialogContent className="max-w-3xl">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <FileJson className="h-5 w-5" />
              操作日誌詳情
            </DialogTitle>
          </DialogHeader>
          {selectedLog && (
            <div className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <Label className="text-muted-foreground">操作時間</Label>
                  <div className="font-medium">{formatDateTimeDisplay(selectedLog.created_at)}</div>
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
