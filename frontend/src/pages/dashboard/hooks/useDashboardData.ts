/**
 * Dashboard ERP 相關資料查詢 Hook
 */
import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { zhTW, enUS } from 'date-fns/locale'
import { useTranslation } from 'react-i18next'
import api, { LowStockAlert, DocumentListItem } from '@/lib/api'
import type { PaginatedResponse } from '@/types/common'
import type { MaintenanceRecordWithDetails } from '@/pages/admin/types'

export interface TrendDataPoint {
  date: string
  dateStr: string
  inbound: number
  outbound: number
}

export function useDashboardData(hasErpPermission: boolean) {
  const { i18n } = useTranslation()
  const currentLocale = i18n.language === 'zh-TW' ? zhTW : enUS

  const { data: lowStockAlerts, isLoading: loadingAlerts } = useQuery({
    queryKey: ['low-stock-alerts'],
    queryFn: async () => {
      const response = await api.get<LowStockAlert[]>('/inventory/low-stock')
      return response.data
    },
    staleTime: 30_000,
    enabled: hasErpPermission,
  })

  const { data: recentDocuments, isLoading: loadingDocuments } = useQuery({
    queryKey: ['recent-documents'],
    queryFn: async () => {
      const response = await api.get<DocumentListItem[]>('/documents')
      return response.data.slice(0, 10)
    },
    staleTime: 60_000,
    enabled: hasErpPermission,
  })

  const { data: recentMaintenance, isLoading: loadingMaintenance } = useQuery({
    queryKey: ['recent-maintenance'],
    queryFn: async () => {
      const response = await api.get<PaginatedResponse<MaintenanceRecordWithDetails>>(
        '/equipment-maintenance',
        { params: { per_page: 10 } }
      )
      return response.data.data
    },
    staleTime: 60_000,
    enabled: hasErpPermission,
  })

  const getTrendData = (days: number = 7): TrendDataPoint[] => {
    if (!recentDocuments) return []
    const today = new Date()
    const result: TrendDataPoint[] = []
    for (let i = days - 1; i >= 0; i--) {
      const date = new Date(today)
      date.setDate(date.getDate() - i)
      const dateStr = date.toISOString().split('T')[0]
      const displayDate = format(date, 'MMM d', { locale: currentLocale })
      const dayDocs = recentDocuments.filter(
        (d) => d.status === 'approved' && d.approved_at?.startsWith(dateStr)
      )
      const inbound = dayDocs.filter((d) => ['GRN'].includes(d.doc_type)).length
      const outbound = dayDocs.filter((d) => ['DO', 'PR'].includes(d.doc_type)).length
      result.push({ date: dateStr, dateStr: displayDate, inbound, outbound })
    }
    return result
  }

  const todayApprovedDocs = useMemo(() => {
    if (!recentDocuments) return []
    const todayStr = new Date().toDateString()
    return recentDocuments.filter(
      (d) => d.status === 'approved' && new Date(d.approved_at || '').toDateString() === todayStr
    )
  }, [recentDocuments])

  return {
    lowStockAlerts,
    loadingAlerts,
    recentDocuments,
    loadingDocuments,
    recentMaintenance,
    loadingMaintenance,
    todayApprovedDocs,
    getTrendData,
  }
}
