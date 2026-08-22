import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { Pencil, Trash2, Search, User } from 'lucide-react'
import { format } from 'date-fns'
import { getDateFnsLocale } from '@/lib/utils'
import type { TrainingRecordWithUser, TrainingUser } from '../types/training'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { FileText } from 'lucide-react'
import { useTableSort } from '@/hooks/useTableSort'

interface TrainingRecordsTabProps {
  canManage: boolean
  canManageAll: boolean
  isGuestUser?: boolean
  records: TrainingRecordWithUser[]
  isLoading: boolean
  totalPages: number
  page: number
  setPage: (updater: (prev: number) => number) => void
  selectedUserId: string
  setSelectedUserId: (id: string) => void
  searchQuery: string
  setSearchQuery: (query: string) => void
  keyword: string
  setKeyword: (keyword: string) => void
  filteredUsers: TrainingUser[]
  onEdit: (record: TrainingRecordWithUser) => void
  onDelete: (record: TrainingRecordWithUser) => void
}

export function TrainingRecordsTab({
  canManage,
  canManageAll,
  records,
  isLoading,
  totalPages,
  page,
  setPage,
  selectedUserId,
  setSelectedUserId,
  searchQuery,
  setSearchQuery,
  keyword,
  setKeyword,
  filteredUsers,
  onEdit,
  onDelete,
  isGuestUser = false,
}: TrainingRecordsTabProps) {
  const { t } = useTranslation()
  const { sortedData, sort, toggleSort } = useTableSort(records)

  return (
    <Card>
      <CardHeader>
        <CardTitle>
          {canManageAll
            ? t('admin.trainingRecordsTab.selectStaffTitle')
            : t('admin.trainingRecordsTab.myRecordsTitle')}
        </CardTitle>
        <CardDescription>
          {canManageAll
            ? t('admin.trainingRecordsTab.selectStaffDesc')
            : t('admin.trainingRecordsTab.myRecordsDesc')}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {canManageAll && (
          <>
            <div className="relative">
              <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder={t('admin.trainingRecordsTab.searchStaffPlaceholder')}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-8"
              />
            </div>
            <div className="flex flex-wrap gap-2 max-h-48 overflow-y-auto">
              <Button
                variant={!selectedUserId ? 'default' : 'outline'}
                size="sm"
                onClick={() => setSelectedUserId('')}
                className="justify-start"
              >
                {t('admin.trainingRecordsTab.allStaff')}
              </Button>
              {filteredUsers.map((u) => (
                <Button
                  key={u.id}
                  variant={selectedUserId === u.id ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setSelectedUserId(u.id)}
                  className="justify-start"
                >
                  <User className="h-4 w-4 mr-2" />
                  {u.display_name || u.email}
                </Button>
              ))}
            </div>
          </>
        )}

        <Input
          placeholder={t('admin.trainingRecordsTab.searchCoursePlaceholder')}
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
          className="max-w-sm"
        />

        <div className="mt-4">
          <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <SortableTableHead sortKey="user_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.trainingRecordsTab.colStaff')}</SortableTableHead>
                    <SortableTableHead sortKey="course_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.trainingRecordsTab.colCourse')}</SortableTableHead>
                    <SortableTableHead sortKey="completed_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.trainingRecordsTab.colCompletedAt')}</SortableTableHead>
                    <SortableTableHead sortKey="expires_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('admin.trainingRecordsTab.colExpiresAt')}</SortableTableHead>
                    <TableHead>{t('admin.trainingRecordsTab.colNotes')}</TableHead>
                    <TableHead className="w-[100px] text-right">{t('common.actions')}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoading ? (
                    <TableRow><TableCell colSpan={6} className="p-0"><TableSkeleton rows={8} cols={6} /></TableCell></TableRow>
                  ) : records.length === 0 ? (
                    <TableEmptyRow colSpan={6} icon={FileText} title={t('admin.trainingRecordsTab.empty')} />
                  ) : (
                    (sortedData ?? records).map((r) => (
                    <TableRow key={r.id}>
                      <TableCell>
                        <div className="font-medium">{r.user_name || r.user_email}</div>
                        <div className="text-xs text-muted-foreground">{r.user_email}</div>
                      </TableCell>
                      <TableCell>{r.course_name}</TableCell>
                      <TableCell>
                        {format(new Date(r.completed_at), 'yyyy/MM/dd', { locale: getDateFnsLocale() })}
                      </TableCell>
                      <TableCell>
                        {r.expires_at
                          ? format(new Date(r.expires_at), 'yyyy/MM/dd', { locale: getDateFnsLocale() })
                          : '\u2014'}
                      </TableCell>
                      <TableCell className="max-w-[200px] whitespace-normal break-words">
                        {r.notes || '\u2014'}
                      </TableCell>
                      <TableCell>
                        {(canManage || isGuestUser) && (
                          <div className="flex items-center justify-end gap-1">
                            <Button variant="ghost" size="icon" onClick={isGuestUser ? undefined : () => onEdit(r)} disabled={isGuestUser} aria-label={t('common.edit')} title={isGuestUser ? t('admin.trainingRecordsTab.guestMode') : undefined}>
                              <Pencil className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={isGuestUser ? undefined : () => onDelete(r)}
                              disabled={isGuestUser}
                              className="text-destructive hover:text-destructive"
                              aria-label={t('common.delete')}
                              title={isGuestUser ? t('admin.trainingRecordsTab.guestMode') : undefined}
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </div>
                        )}
                      </TableCell>
                    </TableRow>
                  )))}
                </TableBody>
              </Table>
              {totalPages > 1 && (
                <div className="flex justify-center gap-2 mt-4">
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={page <= 1}
                    onClick={() => setPage((p) => p - 1)}
                  >
                    {t('admin.trainingRecordsTab.prevPage')}
                  </Button>
                  <span className="flex items-center px-4 text-sm text-muted-foreground">
                    {t('admin.trainingRecordsTab.pageIndicator', { page, totalPages })}
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={page >= totalPages}
                    onClick={() => setPage((p) => p + 1)}
                  >
                    {t('admin.trainingRecordsTab.nextPage')}
                  </Button>
                </div>
              )}
            </>
        </div>
      </CardContent>
    </Card>
  )
}
