import { useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router-dom'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import type { ProtocolListItem, ProtocolStatus } from '@/types/aup'
import { useGuestQuery } from '@/hooks/useGuestQuery'
import { DEMO_PROTOCOLS } from '@/lib/guest-demo'
import { useDebounce } from '@/hooks/useDebounce'
import { useTableSort } from '@/hooks/useTableSort'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { FilterBar } from '@/components/ui/filter-bar'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
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
import { GuestHide } from '@/components/ui/guest-hide'
import { useAuthHasPermission, useAuthHasRole } from '@/stores/auth'
import { Plus, Eye, Edit, FileText, Trash2, Copy, FileInput } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { useNavigate } from 'react-router-dom'
import { formatDate } from '@/lib/utils'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { statusColors } from '../constants'

/** 計畫書清單分頁（原 ProtocolsPage 內容，抽出供 計畫書管理 分頁使用）。 */
export function ProtocolListTab() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const navigate = useNavigate()
  const { dialogState, confirm } = useConfirmDialog()
  const hasPermission = useAuthHasPermission()
  const canImportApproved = hasPermission('aup.protocol.import_approved')
  // 後端 create_protocol / copy_protocol 皆為 `aup.protocol.create || ROLE_PI`；
  // PI 角色本身就持有 aup.protocol.create（seed 已授予），故用 permission 即可涵蓋，
  // 不需要跟著抄 role 判斷。
  const canCreateProtocol = hasPermission('aup.protocol.create')
  const canDeletePermission = hasPermission('aup.protocol.delete')
  const isAdmin = useAuthHasRole()('admin')
  const [search, setSearch] = useState('')
  const debouncedSearch = useDebounce(search, 400)
  const [statusFilter, setStatusFilter] = useState<string>('all')

  const getStatusName = useCallback(
    (status: ProtocolStatus) => t(`protocols.status.${status}`) || status,
    [t]
  )

  const { data: rawProtocols, isLoading } = useGuestQuery(DEMO_PROTOCOLS, {
    queryKey: ['protocols', statusFilter, debouncedSearch],
    queryFn: async () => {
      let params = ''
      if (statusFilter && statusFilter !== 'all') params += `status=${statusFilter}&`
      if (debouncedSearch) params += `keyword=${encodeURIComponent(debouncedSearch)}&`
      const response = await api.get<ProtocolListItem[]>(`/protocols?${params}`)
      return response.data.filter(p => p.status !== 'DELETED')
    },
    staleTime: 60_000,
  })

  const { sortedData: sortedProtocols, sort, toggleSort } = useTableSort<ProtocolListItem, string>(
    rawProtocols,
    { column: 'created_at', direction: 'desc' },
  )

  const clearFilters = () => {
    setSearch('')
    setStatusFilter('all')
  }

  const hasFilters = search || (statusFilter && statusFilter !== 'all')

  const getStatusBadge = (status: ProtocolStatus) => (
    <Badge variant={statusColors[status]}>{getStatusName(status)}</Badge>
  )

  // 可編輯狀態（草稿 / 各退回補件）；實際是否顯示編輯鈕再 AND 後端的 can_edit（PI / SD / admin）。
  const isEditableStatus = (status: ProtocolStatus | string) => {
    const normalized = String(status).toUpperCase()
    return normalized === 'DRAFT' || normalized === 'REVISION_REQUIRED' || normalized === 'PRE_REVIEW_REVISION_REQUIRED' || normalized === 'VET_REVISION_REQUIRED'
  }

  const canEditProtocol = (p: ProtocolListItem) => isEditableStatus(p.status) && p.can_edit === true

  // 刪除走 POST /protocols/:id/status（to_status=DELETED），後端 change_protocol_status
  // 對該轉換要求 require_permission!("aup.protocol.delete")。原本只看 p.can_edit＝擁有權，
  // 導致有編輯權但無刪除權的人看得到一顆必定 403 的垃圾桶。兩層都要：擁有權 + 權限。
  const canDeleteProtocol = (p: ProtocolListItem) =>
    String(p.status).toUpperCase() === 'DRAFT' && p.can_edit === true && canDeletePermission

  // admin 硬刪除（DELETE /protocols/:id/imported）：匯入計劃 / 已駁回 / 草稿，且無下游資料（後端守衛）。
  const canAdminHardDelete = (p: { imported_at?: string | null; status: ProtocolStatus | string }) =>
    isAdmin && (!!p.imported_at || ['REJECTED', 'DRAFT'].includes(String(p.status).toUpperCase()))

  const deleteMutation = useMutation({
    mutationFn: async (protocolId: string) => api.post(`/protocols/${protocolId}/status`, { to_status: 'DELETED' }),
    onSuccess: () => {
      toast({ title: t('common.success'), description: t('protocols.deleted') })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(error, t('protocols.deleteFailed')), variant: 'destructive' })
    },
  })

  const handleDelete = async (protocolId: string, title: string) => {
    const ok = await confirm({ title: '刪除計畫書', description: t('protocols.deleteConfirm', { title }), variant: 'destructive', confirmLabel: '確認刪除' })
    if (ok) deleteMutation.mutate(protocolId)
  }

  const copyMutation = useMutation({
    mutationFn: (protocolId: string) => api.post(`/protocols/${protocolId}/copy`),
    onSuccess: (res) => {
      const newId = res.data?.id
      toast({ title: '已複製計畫書', description: '新草稿已建立，即將開啟編輯頁。' })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
      if (newId) navigate(`/protocols/${newId}/edit`)
    },
    onError: (error: unknown) => {
      toast({ title: '複製失敗', description: getApiErrorMessage(error), variant: 'destructive' })
    },
  })

  const handleCopy = async (protocolId: string, title: string) => {
    const ok = await confirm({ title: '複製計畫書', description: `確定要複製「${title}」建立新草稿嗎？`, confirmLabel: '確認複製' })
    if (ok) copyMutation.mutate(protocolId)
  }

  // 刪除匯入計劃（admin only）：呼叫既有後端 DELETE /protocols/:id/imported（硬刪 + 已 audit + 下游守衛）
  const deleteImportedMutation = useMutation({
    mutationFn: (protocolId: string) => api.delete(`/protocols/${protocolId}/imported`),
    onSuccess: () => {
      toast({ title: t('common.success'), description: '計畫已刪除' })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
    },
    onError: (error: unknown) => {
      toast({ title: t('common.error'), description: getApiErrorMessage(error, '刪除失敗'), variant: 'destructive' })
    },
  })

  const handleDeleteImported = async (protocolId: string, title: string) => {
    const ok = await confirm({ title: '硬刪除計畫', description: `確定要硬刪除「${title}」嗎？此操作不可復原（限匯入計劃 / 已駁回 / 草稿，且無變更申請或廢棄物樣品等下游資料）。`, variant: 'destructive', confirmLabel: t('common.confirmDelete') })
    if (ok) deleteImportedMutation.mutate(protocolId)
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-end gap-2">
        {canImportApproved && (
          <Button size="sm" variant="outline" asChild>
            <Link to="/protocols/import-approved"><FileInput className="mr-2 h-4 w-4" />匯入已核准計劃</Link>
          </Button>
        )}
        {canCreateProtocol && (
          <Button size="sm" asChild>
            <Link to="/protocols/new"><Plus className="mr-2 h-4 w-4" />{t('protocols.createNew')}</Link>
          </Button>
        )}
      </div>

      <FilterBar
        search={search}
        onSearchChange={setSearch}
        searchPlaceholder={t('protocols.searchPlaceholder')}
        hasActiveFilters={!!hasFilters}
        onClearFilters={clearFilters}
      >
        <Select value={statusFilter} onValueChange={setStatusFilter}>
          <SelectTrigger className="w-40">
            <SelectValue placeholder={t('common.allStatus')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t('common.allStatus')}</SelectItem>
            {Object.keys(statusColors).filter(k => k !== 'DELETED').map((key) => (
              <SelectItem key={key} value={key}>{getStatusName(key as ProtocolStatus)}</SelectItem>
            ))}
          </SelectContent>
        </Select>
      </FilterBar>

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <SortableTableHead sortKey="iacuc_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.columns.iacucNo')}</SortableTableHead>
              <SortableTableHead sortKey="title" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.columns.protocolTitle')}</SortableTableHead>
              <SortableTableHead sortKey="pi_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} breakKeep>{t('protocols.columns.pi')}</SortableTableHead>
              <SortableTableHead sortKey="pi_organization" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.columns.organization')}</SortableTableHead>
              <SortableTableHead sortKey="status" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.columns.status')}</SortableTableHead>
              <SortableTableHead sortKey="start_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.columns.period')}</SortableTableHead>
              <SortableTableHead sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.columns.createdAt')}</SortableTableHead>
              <TableHead className="text-right">{t('protocols.columns.actions')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow><TableCell colSpan={8} className="p-0"><TableSkeleton rows={8} cols={8} /></TableCell></TableRow>
            ) : sortedProtocols && sortedProtocols.length > 0 ? (
              sortedProtocols.map((protocol: ProtocolListItem) => (
                <TableRow key={protocol.id}>
                  <TableCell className="font-mono">
                    {protocol.iacuc_no ? (
                      <Link to={`/protocols/${protocol.id}`} className="text-status-warning-text hover:text-status-warning-text/80 hover:underline cursor-pointer">{protocol.iacuc_no}</Link>
                    ) : '-'}
                  </TableCell>
                  <TableCell className="max-w-[200px] whitespace-normal break-words">
                    <Link to={`/protocols/${protocol.id}`} className="text-primary hover:text-primary/80 hover:underline cursor-pointer">{protocol.title}</Link>
                  </TableCell>
                  <TableCell>{protocol.pi_name}</TableCell>
                  <TableCell>{protocol.pi_organization || '-'}</TableCell>
                  <TableCell>{getStatusBadge(protocol.status)}</TableCell>
                  <TableCell>{protocol.start_date && protocol.end_date ? `${formatDate(protocol.start_date)} ~ ${formatDate(protocol.end_date)}` : '-'}</TableCell>
                  <TableCell>{formatDate(protocol.created_at)}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-1">
                      <Button variant="ghost" size="icon" asChild title={t('common.view')} aria-label={t('common.view')}>
                        <Link to={`/protocols/${protocol.id}`}><Eye className="h-4 w-4" /></Link>
                      </Button>
                      <GuestHide>
                        {canEditProtocol(protocol) && (
                          <Button variant="ghost" size="icon" asChild title={t('common.edit')} aria-label={t('common.edit')}>
                            <Link to={`/protocols/${protocol.id}/edit`}><Edit className="h-4 w-4" /></Link>
                          </Button>
                        )}
                        {canCreateProtocol && (
                          <Button variant="ghost" size="icon" title="複製計畫書" aria-label="複製計畫書" onClick={() => handleCopy(protocol.id, protocol.title)} disabled={copyMutation.isPending}>
                            <Copy className="h-4 w-4" />
                          </Button>
                        )}
                        {canDeleteProtocol(protocol) && !isAdmin && (
                          <Button variant="ghost" size="icon" title={t('common.delete')} aria-label={t('common.delete')} onClick={() => handleDelete(protocol.id, protocol.title)}>
                            <Trash2 className="h-4 w-4 text-destructive" />
                          </Button>
                        )}
                        {canAdminHardDelete(protocol) && (
                          <Button variant="ghost" size="icon" title="硬刪除計畫" aria-label="硬刪除計畫" onClick={() => handleDeleteImported(protocol.id, protocol.title)} disabled={deleteImportedMutation.isPending}>
                            <Trash2 className="h-4 w-4 text-destructive" />
                          </Button>
                        )}
                      </GuestHide>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={8} icon={FileText} title={t('protocols.noData')} />
            )}
          </TableBody>
        </Table>
      </div>
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
