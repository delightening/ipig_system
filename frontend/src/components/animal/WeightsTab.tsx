import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { AnimalWeight } from '@/lib/api'
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
import { Plus, Edit2, Trash2, Scale, Loader2 } from 'lucide-react'
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

export function WeightsTab({
  animalId, earTag, afterParam, weights,
  hasAdminRole, developerMode, toggleDeveloperMode,
}: WeightsTabProps) {
  const queryClient = useQueryClient()

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
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '新增失敗',
        variant: 'destructive',
      })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: number; reason: string }) => {
      return api.delete(`/weights/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-weights', animalId] })
      toast({ title: '成功', description: '體重紀錄已刪除' })
      setDeleteTarget(null)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '刪除失敗',
        variant: 'destructive',
      })
    },
  })

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>體重紀錄</CardTitle>
            <CardDescription>記錄動物體重變化歷程</CardDescription>
          </div>
          <div className="flex items-center gap-2">
            {hasAdminRole && (
              <label className="flex items-center gap-2 text-sm text-slate-500 cursor-pointer">
                <input
                  type="checkbox"
                  checked={developerMode}
                  onChange={() => toggleDeveloperMode()}
                  className="rounded"
                />
                顯示系統號
              </label>
            )}
            <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowAddDialog(true)}>
              <Plus className="h-4 w-4 mr-2" />
              新增紀錄
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {!weights || weights.length === 0 ? (
            <div className="text-center py-12 text-slate-500">
              <Scale className="h-12 w-12 mx-auto mb-4 text-slate-300" />
              <p>尚無體重紀錄</p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  {developerMode && <TableHead>系統號</TableHead>}
                  <TableHead>測量日期</TableHead>
                  <TableHead>體重 (kg)</TableHead>
                  <TableHead>記錄者</TableHead>
                  <TableHead>建立時間</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {weights.map((weight) => (
                  <TableRow key={weight.id} data-record-id={weight.id}>
                    {developerMode && <TableCell>{weight.id}</TableCell>}
                    <TableCell>{new Date(weight.measure_date).toLocaleDateString('zh-TW')}</TableCell>
                    <TableCell className="font-medium">{weight.weight}</TableCell>
                    <TableCell>{weight.created_by_name || '-'}</TableCell>
                    <TableCell>{new Date(weight.created_at).toLocaleString('zh-TW')}</TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button variant="ghost" size="icon" title={`系統號: ${weight.id} - 點擊編輯`}>
                          <Edit2 className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => setDeleteTarget(weight.id)}
                          title={`系統號: ${weight.id} - 點擊刪除`}
                        >
                          <Trash2 className="h-4 w-4 text-red-500" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
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
              className="bg-green-600 hover:bg-green-700"
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
}
