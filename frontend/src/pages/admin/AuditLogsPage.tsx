import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'

import { useDateRangeFilter } from '@/hooks/useDateRangeFilter'
import api from '@/lib/api'
import type { User } from '@/types/auth'
import type { UserActivityLog } from '@/types/hr'
import type { PaginatedResponse } from '@/types/common'

import { GuestHide } from '@/components/ui/guest-hide'
import { PageHeader } from '@/components/ui/page-header'
import { categoryEntityMap } from './constants/auditLogs'
import { useAuditLogExport } from './hooks/useAuditLogExport'
import { AuditLogFilters } from './components/AuditLogFilters'
import { AuditLogExportBar } from './components/AuditLogExportBar'
import { AuditLogTable } from './components/AuditLogTable'
import { ActivityLogDetailDialog } from './components/ActivityLogDetailDialog'

const PER_PAGE = 50

function getDefaultDateFrom() {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-01`
}

function getDefaultDateTo() {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`
}

export function AuditLogsPage() {
  const { from: dateFrom, to: dateTo, setFrom: setDateFrom, setTo: setDateTo } = useDateRangeFilter({
    initialFrom: getDefaultDateFrom,
    initialTo: getDefaultDateTo,
  })
  const [categoryFilter, setCategoryFilter] = useState<string>('all')
  const [entityTypeFilter, setEntityTypeFilter] = useState<string>('all')
  const [userFilter, setUserFilter] = useState<string>('all')
  const [selectedLog, setSelectedLog] = useState<UserActivityLog | null>(null)
  const [currentPage, setCurrentPage] = useState(1)

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
      params.set('per_page', String(PER_PAGE))

      const response = await api.get<PaginatedResponse<UserActivityLog>>(
        `/admin/audit/activities?${params.toString()}`
      )
      return response.data
    },
  })

  const { isExporting, handleExportCSV, handleExportPDF } = useAuditLogExport({
    dateFrom,
    dateTo,
    categoryFilter,
    entityTypeFilter,
    userFilter,
  })

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

  const totalPages = activityLogs ? Math.ceil(activityLogs.total / PER_PAGE) : 0

  return (
    <div className="space-y-6">
      <PageHeader
        title="操作日誌"
        description="追蹤所有使用者的操作記錄與變更歷史"
      />

      <AuditLogFilters
        users={users}
        userFilter={userFilter}
        categoryFilter={categoryFilter}
        entityTypeFilter={entityTypeFilter}
        dateFrom={dateFrom}
        dateTo={dateTo}
        availableEntityTypes={availableEntityTypes}
        onUserChange={handleUserChange}
        onCategoryChange={handleCategoryChange}
        onEntityTypeChange={handleEntityTypeChange}
        onDateFromChange={handleDateFromChange}
        onDateToChange={handleDateToChange}
      />

      <GuestHide>
        <AuditLogExportBar
          isExporting={isExporting}
          totalCount={activityLogs?.total}
          onExportCSV={handleExportCSV}
          onExportPDF={handleExportPDF}
        />
      </GuestHide>

      <AuditLogTable
        activityLogs={activityLogs}
        isLoading={isLoading}
        currentPage={currentPage}
        totalPages={totalPages}
        onPageChange={setCurrentPage}
        onSelectLog={setSelectedLog}
      />

      <ActivityLogDetailDialog
        selectedLog={selectedLog}
        onClose={() => setSelectedLog(null)}
      />
    </div>
  )
}
