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
                <Label className="text-gray-500 text-sm">檢查日期</Label>
                <p className="font-medium">{detail.blood_test.test_date}</p>
              </div>
              <div>
                <Label className="text-gray-500 text-sm">檢驗機構</Label>
                <p className="font-medium">{detail.blood_test.lab_name || '-'}</p>
              </div>
            </div>
            {detail.blood_test.remark && (
              <div>
                <Label className="text-gray-500 text-sm">備註</Label>
                <p>{detail.blood_test.remark}</p>
              </div>
            )}
            <div>
              <Label className="text-gray-500 text-sm">建立者</Label>
              <p>{detail.created_by_name || '-'}</p>
            </div>

            <div>
              <Label className="text-base font-semibold mb-2 block">
                檢查項目 ({detail.items.length})
              </Label>
              <Card>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>項目名稱</TableHead>
                      <TableHead>結果值</TableHead>
                      <TableHead>單位</TableHead>
                      <TableHead>參考範圍</TableHead>
                      <TableHead className="text-center">狀態</TableHead>
                      <TableHead>備註</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {detail.items.map((item) => (
                      <TableRow key={item.id} className={item.is_abnormal ? 'bg-red-50' : ''}>
                        <TableCell className="font-medium">{item.item_name}</TableCell>
                        <TableCell>{item.result_value || '-'}</TableCell>
                        <TableCell>{item.result_unit || '-'}</TableCell>
                        <TableCell className="text-sm text-gray-500">{item.reference_range || '-'}</TableCell>
                        <TableCell className="text-center">
                          {item.is_abnormal ? (
                            <Badge variant="destructive" className="gap-1">
                              <AlertCircle className="h-3 w-3" />
                              異常
                            </Badge>
                          ) : (
                            <Badge variant="outline" className="text-green-600 border-green-300">正常</Badge>
                          )}
                        </TableCell>
                        <TableCell className="text-sm">{item.remark || '-'}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </Card>
            </div>
          </div>
        ) : (
          <div className="flex justify-center py-8">
            <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
