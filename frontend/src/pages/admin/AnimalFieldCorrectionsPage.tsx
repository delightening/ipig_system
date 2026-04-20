import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { animalFieldCorrectionApi } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Loader2, Check, X, FileEdit, CheckCircle2 } from 'lucide-react'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { toast } from '@/components/ui/use-toast'
import { useState } from 'react'
import { getApiErrorMessage } from '@/lib/validation'
import { useTableSort } from '@/hooks/useTableSort'

const FIELD_LABELS: Record<string, string> = {
  ear_tag: '耳號',
  birth_date: '出生日期',
  gender: '性別',
  breed: '品種',
}

const formatValue = (field: string, value: string | null): string => {
  if (!value) return '-'
  if (field === 'birth_date') {
    try {
      const d = new Date(value)
      if (!isNaN(d.getTime())) return d.toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })
    } catch {
      // ignore
    }
  }
  if (field === 'gender') {
    if (value === 'male') return '公'
    if (value === 'female') return '母'
  }
  if (field === 'breed') {
    const m: Record<string, string> = {
      miniature: '迷你豬',
      minipig: '迷你豬',
      white: '白豬',
      LYD: 'LYD',
      lyd: 'LYD',
      other: '其他',
    }
    return m[value] || value
  }
  return value
}

export function AnimalFieldCorrectionsPage() {
  const queryClient = useQueryClient()
  const [rejectDialog, setRejectDialog] = useState<{ id: string; earTag: string } | null>(null)
  const [rejectReason, setRejectReason] = useState('')

  const { data: pending, isLoading } = useQuery({
    queryKey: ['animals-animal-field-corrections-pending'],
    queryFn: async () => {
      const res = await animalFieldCorrectionApi.listPending()
      return res.data
    },
  })

  const approveMutation = useMutation({
    mutationFn: (id: string) =>
      animalFieldCorrectionApi.review(id, { approved: true }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animals-animal-field-corrections-pending'] })
      toast({ title: '成功', description: '已批准修正申請' })
    },
    onError: (err) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(err, '批准失敗'),
        variant: 'destructive',
      })
    },
  })

  const rejectMutation = useMutation({
    mutationFn: ({ id, reason }: { id: string; reason: string }) =>
      animalFieldCorrectionApi.review(id, { approved: false, reject_reason: reason }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animals-animal-field-corrections-pending'] })
      setRejectDialog(null)
      setRejectReason('')
      toast({ title: '成功', description: '已拒絕修正申請' })
    },
    onError: (err) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(err, '拒絕失敗'),
        variant: 'destructive',
      })
    },
  })

  const { sortedData: sortedPending, sort, toggleSort } = useTableSort(pending)

  const handleReject = () => {
    if (!rejectDialog) return
    rejectMutation.mutate({ id: rejectDialog.id, reason: rejectReason || '未提供原因' })
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="修正審核"
        description="耳號、出生日期、性別、品種等欄位建立後不可直接修改，需經管理員批准後才能套用修正。"
      />

      <Card className="overflow-hidden">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileEdit className="h-5 w-5" />
            待審核申請
          </CardTitle>
          <CardDescription>
            共 {pending?.length ?? 0} 筆待審核
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                <SortableTableHead sortKey="animal_ear_tag" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>耳號</SortableTableHead>
                <SortableTableHead sortKey="field_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>欄位</SortableTableHead>
                <TableHead>原值</TableHead>
                <TableHead>新值</TableHead>
                <TableHead>原因</TableHead>
                <SortableTableHead sortKey="requested_by_name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>申請人</SortableTableHead>
                <SortableTableHead sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>申請時間</SortableTableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow><TableCell colSpan={8} className="p-0"><TableSkeleton rows={5} cols={8} /></TableCell></TableRow>
              ) : !pending?.length ? (
                <TableEmptyRow colSpan={8} icon={CheckCircle2} title="目前沒有待審核的修正申請" description="所有修正申請皆已處理完畢" />
              ) : (
                (sortedPending ?? pending)?.map((r) => (
                  <TableRow key={r.id}>
                    <TableCell>
                      <Link
                        to={`/animals/${r.animal_id}`}
                        className="text-primary hover:underline font-medium"
                      >
                        {r.animal_ear_tag || '-'}
                      </Link>
                    </TableCell>
                    <TableCell>{FIELD_LABELS[r.field_name] || r.field_name}</TableCell>
                    <TableCell>{formatValue(r.field_name, r.old_value)}</TableCell>
                    <TableCell className="font-medium">{formatValue(r.field_name, r.new_value)}</TableCell>
                    <TableCell className="max-w-[200px] whitespace-normal break-words" title={r.reason}>
                      {r.reason}
                    </TableCell>
                    <TableCell>{r.requested_by_name || '-'}</TableCell>
                    <TableCell>{new Date(r.created_at).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei' })}</TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-2">
                        <Button
                          size="sm"
                          className="bg-status-success-text hover:bg-status-success-text/90"
                          onClick={() => approveMutation.mutate(r.id)}
                          disabled={approveMutation.isPending}
                        >
                          {approveMutation.isPending ? (
                            <Loader2 className="h-4 w-4 animate-spin" />
                          ) : (
                            <>
                              <Check className="h-4 w-4 mr-1" />
                              批准
                            </>
                          )}
                        </Button>
                        <Button
                          size="sm"
                          variant="destructive"
                          onClick={() => setRejectDialog({ id: r.id, earTag: r.animal_ear_tag || '' })}
                          disabled={rejectMutation.isPending}
                        >
                          <X className="h-4 w-4 mr-1" />
                          拒絕
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <Dialog open={!!rejectDialog} onOpenChange={(open) => !open && setRejectDialog(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>拒絕修正申請</DialogTitle>
            <DialogDescription>
              請填寫拒絕原因（選填）。耳號：{rejectDialog?.earTag}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Label>拒絕原因</Label>
            <Input
              value={rejectReason}
              onChange={(e) => setRejectReason(e.target.value)}
              placeholder="如：經查證原資料正確，無需修正"
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRejectDialog(null)}>
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={handleReject}
              disabled={rejectMutation.isPending}
            >
              {rejectMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              確認拒絕
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
