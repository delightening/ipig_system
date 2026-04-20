/**
 * 血液檢查詳情 Dialog
 */
import { Badge } from '@/components/ui/badge'
import { Card } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { useTableSort } from '@/hooks/useTableSort'
import { Loader2, AlertCircle } from 'lucide-react'
import type { AnimalBloodTestWithItems } from '@/types'

interface BloodTestDetailDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  detail: AnimalBloodTestWithItems | null | undefined
}

export function BloodTestDetailDialog({
  open,
  onOpenChange,
  detail,
}: BloodTestDetailDialogProps) {
  const { sortedData, sort, toggleSort } = useTableSort(detail?.items)

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>血液檢查詳情</DialogTitle>
        </DialogHeader>

        {detail ? (
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="text-muted-foreground text-sm">檢查日期</Label>
                <p className="font-medium">{detail.blood_test.test_date}</p>
              </div>
              <div>
                <Label className="text-muted-foreground text-sm">檢驗機構</Label>
                <p className="font-medium">{detail.blood_test.lab_name || '-'}</p>
              </div>
            </div>
            {detail.blood_test.remark && (
              <div>
                <Label className="text-muted-foreground text-sm">備註</Label>
                <p>{detail.blood_test.remark}</p>
              </div>
            )}
            <div>
              <Label className="text-muted-foreground text-sm">建立者</Label>
              <p>{detail.created_by_name || '-'}</p>
            </div>

            <div>
              <Label className="text-base font-semibold mb-2 block">
                檢查項目 ({detail.items.length})
              </Label>
              <Card className="@container overflow-hidden">
                <div className="hidden @[500px]:block">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <SortableTableHead sortKey="item_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>項目名稱</SortableTableHead>
                        <SortableTableHead sortKey="result_value" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>結果值</SortableTableHead>
                        <TableHead className="hidden @[650px]:table-cell">單位</TableHead>
                        <TableHead className="hidden @[650px]:table-cell">參考範圍</TableHead>
                        <SortableTableHead sortKey="is_abnormal" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="text-center">狀態</SortableTableHead>
                        <TableHead className="hidden @[750px]:table-cell">備註</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {(sortedData ?? detail.items).map((item) => (
                        <TableRow key={item.id} className={item.is_abnormal ? 'bg-status-error-bg' : ''}>
                          <TableCell className="font-medium">{item.item_name}</TableCell>
                          <TableCell>
                            {item.result_value || '-'}
                            <span className="@[650px]:hidden text-xs text-muted-foreground ml-1">{item.result_unit || ''}</span>
                          </TableCell>
                          <TableCell className="hidden @[650px]:table-cell">{item.result_unit || '-'}</TableCell>
                          <TableCell className="hidden @[650px]:table-cell text-sm text-muted-foreground">{item.reference_range || '-'}</TableCell>
                          <TableCell className="text-center">
                            {item.is_abnormal ? (
                              <Badge variant="destructive" className="gap-1">
                                <AlertCircle className="h-3 w-3" />
                                異常
                              </Badge>
                            ) : (
                              <Badge variant="outline" className="text-status-success-text border-status-success-border">正常</Badge>
                            )}
                          </TableCell>
                          <TableCell className="hidden @[750px]:table-cell text-sm">{item.remark || '-'}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>

                <div className="@[500px]:hidden divide-y">
                  {(sortedData ?? detail.items).map((item) => (
                    <div key={item.id} className={`p-3 space-y-1 ${item.is_abnormal ? 'bg-status-error-bg' : ''}`}>
                      <div className="flex items-start justify-between gap-2">
                        <div className="font-medium">{item.item_name}</div>
                        {item.is_abnormal ? (
                          <Badge variant="destructive" className="gap-1 shrink-0">
                            <AlertCircle className="h-3 w-3" />異常
                          </Badge>
                        ) : (
                          <Badge variant="outline" className="text-status-success-text border-status-success-border shrink-0">正常</Badge>
                        )}
                      </div>
                      <div className="text-sm">
                        <span className="font-medium">{item.result_value || '-'}</span>
                        {item.result_unit && <span className="text-muted-foreground ml-1">{item.result_unit}</span>}
                        {item.reference_range && <span className="text-xs text-muted-foreground ml-2">參考: {item.reference_range}</span>}
                      </div>
                      {item.remark && <div className="text-xs text-muted-foreground">{item.remark}</div>}
                    </div>
                  ))}
                </div>
              </Card>
            </div>
          </div>
        ) : (
          <div className="flex justify-center py-8">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
