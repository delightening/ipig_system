/**
 * 設備管理分頁內容：搜尋、表格、分頁
 */
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
import { Pencil, Trash2, Loader2, Search } from 'lucide-react'

import type { Equipment, EquipmentForm } from '../types'

interface EquipmentTabContentProps {
  canManage: boolean
  keyword: string
  onKeywordChange: (v: string) => void
  isLoading: boolean
  records: Equipment[]
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  onEdit: (equip: Equipment) => void
  onDelete: (id: string, name: string) => void
}

export function EquipmentTabContent({
  canManage,
  keyword,
  onKeywordChange,
  isLoading,
  records,
  page,
  totalPages,
  onPageChange,
  onEdit,
  onDelete,
}: EquipmentTabContentProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>設備清單</CardTitle>
        <CardDescription>管理實驗室設備，搜尋並維護設備基本資料</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="搜尋設備名稱或型號..."
            value={keyword}
            onChange={(e) => onKeywordChange(e.target.value)}
            className="pl-8"
          />
        </div>
        <div className="rounded-md border">
          {isLoading ? (
            <div className="flex justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
          ) : records.length === 0 ? (
            <div className="py-12 text-center text-muted-foreground">尚無設備</div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>名稱</TableHead>
                  <TableHead>型號</TableHead>
                  <TableHead>序號</TableHead>
                  <TableHead>位置</TableHead>
                  <TableHead>狀態</TableHead>
                  {canManage && <TableHead className="w-[100px]">操作</TableHead>}
                </TableRow>
              </TableHeader>
              <TableBody>
                {records.map((r) => (
                  <TableRow key={r.id}>
                    <TableCell className="font-medium">{r.name}</TableCell>
                    <TableCell>{r.model || '—'}</TableCell>
                    <TableCell>{r.serial_number || '—'}</TableCell>
                    <TableCell>{r.location || '—'}</TableCell>
                    <TableCell>{r.is_active ? '啟用' : '停用'}</TableCell>
                    {canManage && (
                      <TableCell>
                        <div className="flex gap-2">
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => onEdit(r)}
                          >
                            <Pencil className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="text-destructive hover:text-destructive"
                            onClick={() => onDelete(r.id, r.name)}
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </div>
                      </TableCell>
                    )}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </div>
        {totalPages > 1 && (
          <div className="flex justify-center gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={page <= 1}
              onClick={() => onPageChange(page - 1)}
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
              onClick={() => onPageChange(page + 1)}
            >
              下一頁
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
