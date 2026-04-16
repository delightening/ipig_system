import React, { useState } from 'react'
import { GuestHide } from '@/components/ui/guest-hide'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { deleteResource, AnimalVaccination } from '@/lib/api'
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
import { Plus, Edit2, Trash2, Syringe, Loader2 } from 'lucide-react'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'

interface VaccinationsTabProps {
  animalId: string
  earTag: string
  afterParam: string
  vaccinations: AnimalVaccination[] | undefined
}

export const VaccinationsTab = React.memo(function VaccinationsTab({ animalId, earTag, afterParam: _afterParam, vaccinations }: VaccinationsTabProps) {
  const queryClient = useQueryClient()
  const { sortedData, sort, toggleSort } = useTableSort(vaccinations)

  const [showAddDialog, setShowAddDialog] = useState(false)
  const [newVaccination, setNewVaccination] = useState({ administered_date: new Date().toISOString().split('T')[0], vaccine: '', deworming_dose: '' })
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null)

  const addMutation = useMutation({
    mutationFn: async (data: typeof newVaccination) => {
      return api.post(`/animals/${animalId}/vaccinations`, data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-vaccinations', animalId] })
      toast({ title: '成功', description: '疫苗紀錄已新增' })
      setShowAddDialog(false)
      setNewVaccination({ administered_date: new Date().toISOString().split('T')[0], vaccine: '', deworming_dose: '' })
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
    mutationFn: async ({ id, reason }: { id: string; reason: string }) => {
      return deleteResource(`/vaccinations/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-vaccinations', animalId] })
      toast({ title: '成功', description: '疫苗紀錄已刪除' })
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
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>疫苗/驅蟲紀錄</CardTitle>
            <CardDescription>記錄疫苗接種與驅蟲紀錄</CardDescription>
          </div>
          <GuestHide>
            <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowAddDialog(true)}>
              <Plus className="h-4 w-4 mr-2" />
              新增紀錄
            </Button>
          </GuestHide>
        </CardHeader>
        <CardContent>
          {!vaccinations || vaccinations.length === 0 ? (
            <div className="text-center py-12 text-muted-foreground">
              <Syringe className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
              <p>尚無疫苗/驅蟲紀錄</p>
              <p className="text-sm mt-1">點擊上方按鈕新增</p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <SortableTableHead sortKey="administered_date" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>施打日期</SortableTableHead>
                  <SortableTableHead sortKey="vaccine" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>疫苗</SortableTableHead>
                  <TableHead>驅蟲劑量</TableHead>
                  <TableHead>記錄者</TableHead>
                  <SortableTableHead sortKey="created_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>建立時間</SortableTableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sortedData?.map((vac) => (
                  <TableRow key={vac.id}>
                    <TableCell>{new Date(vac.administered_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })}</TableCell>
                    <TableCell>{vac.vaccine || '-'}</TableCell>
                    <TableCell>{vac.deworming_dose || '-'}</TableCell>
                    <TableCell>{vac.created_by_name || '-'}</TableCell>
                    <TableCell>{new Date(vac.created_at).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei' })}</TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <GuestHide>
                          <Button variant="ghost" size="icon" aria-label="編輯">
                            <Edit2 className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => setDeleteTarget(vac.id)}
                            aria-label="刪除"
                          >
                            <Trash2 className="h-4 w-4 text-status-error-solid" />
                          </Button>
                        </GuestHide>
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
            <DialogTitle>新增疫苗/驅蟲紀錄</DialogTitle>
            <DialogDescription>耳號：{earTag}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="vac_date">施打日期 *</Label>
              <Input
                id="vac_date"
                type="date"
                value={newVaccination.administered_date}
                onChange={(e) => setNewVaccination({ ...newVaccination, administered_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="vaccine">疫苗</Label>
              <Input
                id="vaccine"
                value={newVaccination.vaccine}
                onChange={(e) => setNewVaccination({ ...newVaccination, vaccine: e.target.value })}
                placeholder="如：SEP、IRON"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="deworming">驅蟲劑量</Label>
              <Input
                id="deworming"
                value={newVaccination.deworming_dose}
                onChange={(e) => setNewVaccination({ ...newVaccination, deworming_dose: e.target.value })}
                placeholder="如：Ivermectin 2mL"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAddDialog(false)}>
              取消
            </Button>
            <Button
              onClick={() => addMutation.mutate(newVaccination)}
              disabled={addMutation.isPending || !newVaccination.administered_date}
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
        title="刪除疫苗紀錄"
        description="此操作將標記紀錄為已刪除，資料將保留於系統中以符合 GLP 規範。"
        onConfirm={(reason) => deleteMutation.mutate({ id: deleteTarget!, reason })}
        isPending={deleteMutation.isPending}
      />
    </>
  )
})
