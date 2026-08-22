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
  /** R30-14: 自由文字搜尋（已 debounce） */
  searchQuery?: string
}

function buildExportQueryString(params: ExportParams): string {
  const qs = new URLSearchParams()
  if (params.dateFrom) qs.set('from', params.dateFrom)
  if (params.dateTo) qs.set('to', params.dateTo)
  if (params.categoryFilter !== 'all') qs.set('event_category', params.categoryFilter)
  if (params.entityTypeFilter !== 'all') qs.set('entity_type', params.entityTypeFilter)
  if (params.userFilter !== 'all') qs.set('user_id', params.userFilter)
  if (params.searchQuery && params.searchQuery.trim()) qs.set('query', params.searchQuery.trim())
  return qs.toString()
}

function buildCsvContent(logs: UserActivityLog[]): string {
  const headers = ['時間', '操作者', '操作者信箱', '類別', '事件類型', '資料類型', '資料名稱', 'IP 位址', '可疑']
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

async function fetchExportLogs(params: ExportParams): Promise<UserActivityLog[]> {
  const response = await api.get<UserActivityLog[]>(
    `/admin/audit/activities/export?${buildExportQueryString(params)}`
  )
  return response.data
}

/**
 * 從 Content-Disposition header 解析下載檔名（RFC 6266）。
 * 優先 `filename*=UTF-8''<percent-encoded>`（中文用），回退 `filename="..."`。
 * 解析失敗回 null，由 caller 提供 fallback。
 */
function parseFilenameFromContentDisposition(header: string | undefined): string | null {
  if (!header) return null
  // RFC 5987：filename*=UTF-8''%E6%93%8D...
  const star = header.match(/filename\*\s*=\s*(?:UTF-8'[^']*')?([^;]+)/i)
  if (star?.[1]) {
    try {
      return decodeURIComponent(star[1].trim().replace(/^"|"$/g, ''))
    } catch {
      // 落到 plain filename 路徑
    }
  }
  const plain = header.match(/filename\s*=\s*"?([^";]+)"?/i)
  return plain?.[1]?.trim() || null
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

  // R32-A8i：PDF 走 backend docxtpl + Word COM/Gotenberg LibreOffice
  // （取代原本 client-side HTML + window.print()，與其他 R32 報表一致）
  const exportPDFMutation = useMutation({
    mutationFn: async () => {
      const res = await api.get(
        `/admin/audit/activities/export-pdf?${buildExportQueryString(params)}`,
        { responseType: 'blob' }
      )
      const url = URL.createObjectURL(new Blob([res.data], { type: 'application/pdf' }))
      const a = document.createElement('a')
      a.href = url
      // 從 backend Content-Disposition 解出檔名（單一真實來源 per Gemini PR #346）；
      // 解析失敗才退回前端組合 fallback
      a.download = parseFilenameFromContentDisposition(
        res.headers?.['content-disposition'] as string | undefined
      ) ?? `操作日誌_${params.dateFrom || 'all'}_${params.dateTo || 'all'}.pdf`
      document.body.appendChild(a)
      a.click()
      a.remove()
      URL.revokeObjectURL(url)
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
