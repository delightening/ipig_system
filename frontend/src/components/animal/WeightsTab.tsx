import React, { useState } from 'react'
import { GuestHide } from '@/components/ui/guest-hide'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource, AnimalWeight } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Plus, Edit2, Trash2, Scale, Loader2 } from 'lucide-react'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'

interface WeightsTabProps {
  animalId: string
  earTag: string
  afterParam: string
  weights: AnimalWeight[] | undefined
  hasAdminRole: boolean
  developerMode: boolean
  toggleDeveloperMode: () => void
}

export const WeightsTab = React.memo(function WeightsTab({
  animalId, earTag, afterParam: _afterParam, weights,
  hasAdminRole, developerMode, toggleDeveloperMode,
}: WeightsTabProps) {
  const queryClient = useQueryClient()
  const { sortedData, sort, toggleSort } = useTableSort(weights)

  const [showAddDialog, setShowAddDialog] = useState(false)
  const [newWeight, setNewWeight] = useState({ measure_date: new Date().toISOString().split('T')[0], weight: '' })
  const [deleteTarget, setDeleteTarget] = useState<number | null>(null)

  const addMutation = useMutation({
    mutationFn: async (data: typeof newWeight) => {
      return api.post(`/animals/${animalId}/weights`, {
        measure_date: data.measure_date,
        weight: parseFloat(data.weight),
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-weights', animalId] })
      toast({ title: '成功', description: '體重紀錄已新增' })
      setShowAddDialog(false)
      setNewWeight({ measure_date: new Date().toISOString().split('T')[0], weight: '' })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '新增失敗'),
        variant: 'destructive',
      })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: number; reason: string }) => {
      return deleteResource(`/weights/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-weights', animalId] })
      toast({ title: '成功', description: '體重紀錄已刪除' })
      setDeleteTarget(null)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '刪除失敗'),
        variant: 'destructive',
      })
    },
  })

  return (
    <>
      <Card className="overflow-hidden">
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>體重紀錄</CardTitle>
            <CardDescription>記錄動物體重變化歷程</CardDescription>
          </div>
          <div className="flex items-center gap-2">
            {hasAdminRole && (
              <label className="flex items-center gap-2 text-sm text-muted-foreground cursor-pointer">
                <input
                  type="checkbox"
                  checked={developerMode}
                  onChange={() => toggleDeveloperMode()}
                  className="rounded"
                />
                顯示系統號
              </label>
            )}
            <GuestHide>
              <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowAddDialog(true)}>
                <Plus className="h-4 w-4 mr-2" />
                新增紀錄
              </Button>
            </GuestHide>
          </div>
        </CardHeader>
        <CardContent>
          <div className="@container">

            {/* Table view: container ≥ 600px */}
            <div className="hidden @[600px]:block overflow-x-auto">
              <Table className="w-full" style={{ minWidth: 380 }}>
                <TableHeader>
                  <TableRow className="bg-muted/50 hover:bg-muted/50">
                    {developerMode && <TableHead style={{ width: 80 }} className="hidden @[620px]:table-cell">系統號</TableHead>}
                    <SortableTableHead style={{ width: 100 }} sortKey="measure_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>測量日期</SortableTableHead>
                    <SortableTableHead style={{ width: 90 }} sortKey="weight" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>體重 (kg)</SortableTableHead>
                    <TableHead style={{ width: 100 }}>記錄者</TableHead>
                    <SortableTableHead style={{ width: 160 }} sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="hidden @[620px]:table-cell">建立時間</SortableTableHead>
                    <TableHead style={{ width: 90 }} className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {!weights || weights.length === 0 ? (
                    <TableEmptyRow colSpan={developerMode ? 6 : 5} icon={Scale} title="尚無體重紀錄" />
                  ) : (
                    sortedData?.map((weight) => (
                      <TableRow key={weight.id} data-record-id={weight.id}>
                        {developerMode && <TableCell style={{ width: 80 }} className="font-mono text-xs text-muted-foreground hidden @[620px]:table-cell">{weight.id}</TableCell>}
                        <TableCell style={{ width: 100 }} className="whitespace-nowrap">{new Date(weight.measure_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })}</TableCell>
                        <TableCell style={{ width: 90 }} className="font-medium">{weight.weight}</TableCell>
                        <TableCell style={{ width: 100 }} className="whitespace-normal break-words">{weight.created_by_name || '-'}</TableCell>
                        <TableCell style={{ width: 160 }} className="text-xs text-muted-foreground hidden @[620px]:table-cell">{new Date(weight.created_at).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei' })}</TableCell>
                        <TableCell style={{ width: 90 }} className="text-right">
                          <div className="flex items-center justify-end gap-1">
                            <GuestHide>
                              <Button variant="ghost" size="icon" title={`系統號: ${weight.id} - 點擊編輯`}>
                                <Edit2 className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => setDeleteTarget(weight.id)}
                                title={`系統號: ${weight.id} - 點擊刪除`}
                              >
                                <Trash2 className="h-4 w-4 text-status-error-solid" />
                              </Button>
                            </GuestHide>
                          </div>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>

            {/* Card view: container < 600px */}
            <div className="@[600px]:hidden space-y-3 py-1">
              {!weights || weights.length === 0 ? (
                <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
                  <Scale className="h-8 w-8" />
                  <p className="text-sm">尚無體重紀錄</p>
                </div>
              ) : (
                sortedData?.map((weight) => (
                  <div key={weight.id} data-record-id={weight.id} className="rounded-lg border bg-card p-3">
                    <div className="text-xs text-muted-foreground mb-0.5">
                      {new Date(weight.measure_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })}
                    </div>
                    <div className="text-lg font-bold text-foreground mb-2">
                      ⚖️ {weight.weight} kg
                    </div>
                    <div className="flex items-center justify-between gap-2 pt-1 border-t">
                      <span className="text-xs text-muted-foreground">{weight.created_by_name || '-'}</span>
                      <div className="flex gap-0.5">
                        <GuestHide>
                          <Button variant="ghost" size="icon" title="編輯">
                            <Edit2 className="h-4 w-4" />
                          </Button>
                          <Button variant="ghost" size="icon" onClick={() => setDeleteTarget(weight.id)} title="刪除">
                            <Trash2 className="h-4 w-4 text-status-error-solid" />
                          </Button>
                        </GuestHide>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>

          </div>
        </CardContent>
      </Card>

      <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新增體重紀錄</DialogTitle>
            <DialogDescription>耳號：{earTag}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="weight_date">測量日期 *</Label>
              <Input
                id="weight_date"
                type="date"
                value={newWeight.measure_date}
                onChange={(e) => setNewWeight({ ...newWeight, measure_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="weight_value">體重 (kg) *</Label>
              <Input
                id="weight_value"
                type="number"
                step="0.1"
                value={newWeight.weight}
                onChange={(e) => setNewWeight({ ...newWeight, weight: e.target.value })}
                placeholder="輸入體重"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAddDialog(false)}>
              取消
            </Button>
            <Button
              onClick={() => addMutation.mutate(newWeight)}
              disabled={addMutation.isPending || !newWeight.weight || !newWeight.measure_date}
              className="bg-status-success-solid hover:bg-green-700"
            >
              {addMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <DeleteReasonDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        title="刪除體重紀錄"
        description="此操作將標記紀錄為已刪除，資料將保留於系統中以符合 GLP 規範。"
        onConfirm={(reason) => deleteMutation.mutate({ id: deleteTarget!, reason })}
        isPending={deleteMutation.isPending}
      />
    </>
  )
})
