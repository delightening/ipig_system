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
import { Pencil, Trash2, Search, User, RefreshCw } from 'lucide-react'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import type { TrainingRecordWithUser, TrainingUser } from '../types/training'
import { EmptyState } from '@/components/ui/empty-state'
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
  const { sortedData, sort, toggleSort } = useTableSort(records)

  return (
    <Card>
      <CardHeader>
        <CardTitle>{canManageAll ? '選擇員工查看訓練紀錄' : '我的訓練紀錄'}</CardTitle>
        <CardDescription>
          {canManageAll
            ? '選擇員工以查看其訓練課程紀錄與證照有效期限'
            : '查看與管理您的訓練課程紀錄與證照有效期限'}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {canManageAll && (
          <>
            <div className="relative">
              <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="搜尋員工姓名或 Email..."
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
                全部人員
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
          placeholder="搜尋課程名稱..."
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
          className="max-w-sm"
        />

        <div className="mt-4">
          {isLoading ? (
            <div className="flex justify-center py-8">
              <RefreshCw className="h-6 w-6 animate-spin" />
            </div>
          ) : records.length === 0 ? (
            <EmptyState icon={FileText} title="尚無訓練紀錄" />
          ) : (
            <>
              <Table>
                <TableHeader>
                  <TableRow>
                    <SortableTableHead sortKey="user_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>人員</SortableTableHead>
                    <SortableTableHead sortKey="course_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>課程名稱</SortableTableHead>
                    <SortableTableHead sortKey="completed_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>完成日期</SortableTableHead>
                    <SortableTableHead sortKey="expires_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>有效期限</SortableTableHead>
                    <TableHead>備註</TableHead>
                    <TableHead className="w-[100px] text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {(sortedData ?? records).map((r) => (
                    <TableRow key={r.id}>
                      <TableCell>
                        <div className="font-medium">{r.user_name || r.user_email}</div>
                        <div className="text-xs text-muted-foreground">{r.user_email}</div>
                      </TableCell>
                      <TableCell>{r.course_name}</TableCell>
                      <TableCell>
                        {format(new Date(r.completed_at), 'yyyy/MM/dd', { locale: zhTW })}
                      </TableCell>
                      <TableCell>
                        {r.expires_at
                          ? format(new Date(r.expires_at), 'yyyy/MM/dd', { locale: zhTW })
                          : '\u2014'}
                      </TableCell>
                      <TableCell className="max-w-[200px] truncate">
                        {r.notes || '\u2014'}
                      </TableCell>
                      <TableCell>
                        {(canManage || isGuestUser) && (
                          <div className="flex items-center justify-end gap-1">
                            <Button variant="ghost" size="icon" onClick={isGuestUser ? undefined : () => onEdit(r)} disabled={isGuestUser} aria-label="編輯" title={isGuestUser ? '訪客模式' : undefined}>
                              <Pencil className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={isGuestUser ? undefined : () => onDelete(r)}
                              disabled={isGuestUser}
                              className="text-destructive hover:text-destructive"
                              aria-label="刪除"
                              title={isGuestUser ? '訪客模式' : undefined}
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </div>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
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
                    上一頁
                  </Button>
                  <span className="flex items-center px-4 text-sm text-muted-foreground">
                    第 {page} / {totalPages} 頁
                  </span>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={page >= totalPages}
                    onClick={() => setPage((p) => p + 1)}
                  >
                    下一頁
                  </Button>
                </div>
              )}
            </>
          )}
        </div>
      </CardContent>
    </Card>
  )
}
