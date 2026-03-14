import { useState, useMemo, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import type { ProtocolListItem, ProtocolStatus } from '@/types/aup'
import { useDebounce } from '@/hooks/useDebounce'
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
import { Plus, Search, Eye, Edit, Loader2, FileText, X, Trash2, ArrowUpDown, ArrowUp, ArrowDown } from 'lucide-react'
import { formatDate } from '@/lib/utils'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { statusColors } from './constants'

type SortField = 'iacuc_no' | 'title' | 'pi_name' | 'pi_organization' | 'status' | 'start_date' | 'created_at'
type SortOrder = 'asc' | 'desc'

interface SortConfig {
  field: SortField
  order: SortOrder
}

export function ProtocolsPage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()
  const [search, setSearch] = useState('')
  const debouncedSearch = useDebounce(search, 400)
  const [statusFilter, setStatusFilter] = useState<string>('all')
  const [sortConfig, setSortConfig] = useState<SortConfig>({ field: 'created_at', order: 'desc' })

  // Get status name from i18n
  const getStatusName = useCallback(
    (status: ProtocolStatus) => t(`protocols.status.${status}`) || status,
    [t]
  )

  const { data: rawProtocols, isLoading } = useQuery({
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

  // Sort logic
  const sortedProtocols = useMemo(() => {
    if (!rawProtocols) return []
    const sorted = [...rawProtocols]

    sorted.sort((a, b) => {
      let valA: string | number | undefined = (a as unknown as Record<string, unknown>)[sortConfig.field] as string | number | undefined
      let valB: string | number | undefined = (b as unknown as Record<string, unknown>)[sortConfig.field] as string | number | undefined
      valA = valA ?? ''
      valB = valB ?? ''

      // Handle status sorting by translated name
      if (sortConfig.field === 'status') {
        valA = getStatusName(a.status)
        valB = getStatusName(b.status)
      }

      if (valA < valB) return sortConfig.order === 'asc' ? -1 : 1
      if (valA > valB) return sortConfig.order === 'asc' ? 1 : -1
      return 0
    })

    return sorted
  }, [rawProtocols, sortConfig, getStatusName])

  const handleSort = (field: SortField) => {
    setSortConfig(prev => ({
      field,
      order: prev.field === field && prev.order === 'asc' ? 'desc' : 'asc'
    }))
  }

  const getSortIcon = (field: SortField) => {
    if (sortConfig.field !== field) return <ArrowUpDown className="ml-1 h-3 w-3" />
    return sortConfig.order === 'asc' ?
      <ArrowUp className="ml-1 h-3 w-3 text-primary" /> :
      <ArrowDown className="ml-1 h-3 w-3 text-primary" />
  }

  const clearFilters = () => {
    setSearch('')
    setStatusFilter('all')
  }

  const hasFilters = search || (statusFilter && statusFilter !== 'all')

  const getStatusBadge = (status: ProtocolStatus) => {
    return (
      <Badge variant={statusColors[status]}>
        {getStatusName(status)}
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
      toast({ title: t('common.success'), description: t('protocols.deleted') })
      queryClient.invalidateQueries({ queryKey: ['protocols'] })
    },
    onError: (error: unknown) => {
      toast({
        title: t('common.error'),
        description: getApiErrorMessage(error, t('protocols.deleteFailed')),
        variant: 'destructive'
      })
    },
  })

  const handleDelete = async (protocolId: string, title: string) => {
    const ok = await confirm({ title: '刪除計畫書', description: t('protocols.deleteConfirm', { title }), variant: 'destructive', confirmLabel: '確認刪除' })
    if (ok) {
      deleteMutation.mutate(protocolId)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">{t('protocols.title')}</h1>
          <p className="text-muted-foreground">
            {t('protocols.subtitle')}
          </p>
        </div>
        <Button asChild>
          <Link to="/protocols/new">
            <Plus className="mr-2 h-4 w-4" />
            {t('protocols.createNew')}
          </Link>
        </Button>
      </div>

      <div className="space-y-4">
        <div className="flex flex-wrap gap-4 items-end">
          <div className="relative flex-1 min-w-[200px] max-w-sm">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder={t('protocols.searchPlaceholder')}
              aria-label={t('protocols.searchPlaceholder')}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-9"
            />
          </div>
          <Select value={statusFilter} onValueChange={setStatusFilter}>
            <SelectTrigger className="w-40">
              <SelectValue placeholder={t('common.allStatus')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('common.allStatus')}</SelectItem>
              {Object.keys(statusColors).filter(k => k !== 'DELETED').map((key) => (
                <SelectItem key={key} value={key}>
                  {getStatusName(key as ProtocolStatus)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {hasFilters && (
            <Button variant="ghost" size="sm" onClick={clearFilters}>
              <X className="h-4 w-4 mr-1" />
              {t('common.clearFilters')}
            </Button>
          )}
        </div>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead
                className="cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => handleSort('iacuc_no')}
              >
                <div className="flex items-center">
                  {t('protocols.columns.iacucNo')}
                  {getSortIcon('iacuc_no')}
                </div>
              </TableHead>
              <TableHead
                className="cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => handleSort('title')}
              >
                <div className="flex items-center">
                  {t('protocols.columns.protocolTitle')}
                  {getSortIcon('title')}
                </div>
              </TableHead>
              <TableHead
                className="cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => handleSort('pi_name')}
              >
                <div className="flex items-center">
                  {t('protocols.columns.pi')}
                  {getSortIcon('pi_name')}
                </div>
              </TableHead>
              <TableHead
                className="cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => handleSort('pi_organization')}
              >
                <div className="flex items-center">
                  {t('protocols.columns.organization')}
                  {getSortIcon('pi_organization')}
                </div>
              </TableHead>
              <TableHead
                className="cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => handleSort('status')}
              >
                <div className="flex items-center">
                  {t('protocols.columns.status')}
                  {getSortIcon('status')}
                </div>
              </TableHead>
              <TableHead
                className="cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => handleSort('start_date')}
              >
                <div className="flex items-center">
                  {t('protocols.columns.period')}
                  {getSortIcon('start_date')}
                </div>
              </TableHead>
              <TableHead
                className="cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => handleSort('created_at')}
              >
                <div className="flex items-center">
                  {t('protocols.columns.createdAt')}
                  {getSortIcon('created_at')}
                </div>
              </TableHead>
              <TableHead className="text-right">{t('protocols.columns.actions')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                </TableCell>
              </TableRow>
            ) : sortedProtocols && sortedProtocols.length > 0 ? (
              sortedProtocols.map((protocol) => (
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
                      <Button variant="ghost" size="icon" asChild title={t('common.view')}>
                        <Link to={`/protocols/${protocol.id}`}>
                          <Eye className="h-4 w-4" />
                        </Link>
                      </Button>
                      {canEditProtocol(protocol.status) && (
                        <Button variant="ghost" size="icon" asChild title={t('common.edit')}>
                          <Link to={`/protocols/${protocol.id}/edit`}>
                            <Edit className="h-4 w-4" />
                          </Link>
                        </Button>
                      )}
                      {canDeleteProtocol(protocol.status) && (
                        <Button
                          variant="ghost"
                          size="icon"
                          title={t('common.delete')}
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
                  <p className="text-muted-foreground">{t('protocols.noData')}</p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    <ConfirmDialog state={dialogState} />
    </div>
  )
}
