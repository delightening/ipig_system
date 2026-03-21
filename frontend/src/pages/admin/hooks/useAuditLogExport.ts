import { useMutation } from '@tanstack/react-query'

import api from '@/lib/api'
import { formatDateTime } from '@/lib/utils'
import { logger } from '@/lib/logger'
import type { UserActivityLog } from '@/types/hr'

import { categoryLabels, eventTypeLabels } from '../constants/auditLogs'

interface ExportParams {
  dateFrom: string
  dateTo: string
  categoryFilter: string
  entityTypeFilter: string
  userFilter: string
}

function buildExportQueryString(params: ExportParams): string {
  const qs = new URLSearchParams()
  if (params.dateFrom) qs.set('from', params.dateFrom)
  if (params.dateTo) qs.set('to', params.dateTo)
  if (params.categoryFilter !== 'all') qs.set('event_category', params.categoryFilter)
  if (params.entityTypeFilter !== 'all') qs.set('entity_type', params.entityTypeFilter)
  if (params.userFilter !== 'all') qs.set('user_id', params.userFilter)
  return qs.toString()
}

function buildCsvContent(logs: UserActivityLog[]): string {
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
  return csvRows.join('\n')
}

function buildPrintHtml(logs: UserActivityLog[], dateFrom: string, dateTo: string): string {
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

  return `<!DOCTYPE html>
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
}

async function fetchExportLogs(params: ExportParams): Promise<UserActivityLog[]> {
  const response = await api.get<UserActivityLog[]>(
    `/admin/audit/activities/export?${buildExportQueryString(params)}`
  )
  return response.data
}

export function useAuditLogExport(params: ExportParams) {
  const exportCSVMutation = useMutation({
    mutationFn: () => fetchExportLogs(params),
    onSuccess: (logs) => {
      const bom = '\uFEFF'
      const blob = new Blob([bom + buildCsvContent(logs)], { type: 'text/csv;charset=utf-8;' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `操作日誌_${params.dateFrom}_${params.dateTo}.csv`
      a.click()
      URL.revokeObjectURL(url)
    },
    onError: (err) => {
      logger.error('匯出 CSV 失敗', err)
    },
  })

  const exportPDFMutation = useMutation({
    mutationFn: () => fetchExportLogs(params),
    onSuccess: (logs) => {
      const html = buildPrintHtml(logs, params.dateFrom, params.dateTo)
      const printWindow = window.open('', '_blank')
      if (printWindow) {
        printWindow.document.write(html)
        printWindow.document.close()
        setTimeout(() => printWindow.print(), 300)
      }
    },
    onError: (err) => {
      logger.error('匯出 PDF 失敗', err)
    },
  })

  const isExporting = exportCSVMutation.isPending || exportPDFMutation.isPending

  return {
    isExporting,
    handleExportCSV: () => exportCSVMutation.mutate(),
    handleExportPDF: () => exportPDFMutation.mutate(),
  }
}
