/**
 * 設備與校準紀錄管理頁 — 實驗室 GLP 合規
 *
 * 功能：
 * - 設備 CRUD
 * - 校準紀錄 CRUD（依設備篩選）
 */

import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Plus, Pencil, Trash2, Loader2, Wrench, Ruler } from 'lucide-react'
import { format } from 'date-fns'
import { zhTW } from 'date-fns/locale'
import { useAuthStore } from '@/stores/auth'

interface Equipment {
  id: string
  name: string
  model: string | null
  serial_number: string | null
  location: string | null
  notes: string | null
  is_active: boolean
}

interface CalibrationWithEquipment {
  id: string
  equipment_id: string
  equipment_name: string
  calibrated_at: string
  next_due_at: string | null
  result: string | null
  notes: string | null
  created_at: string
}

interface PaginatedResponse<T> {
  data: T[]
  total: number
  page: number
  per_page: number
  total_pages: number
}

export function EquipmentPage() {
  const queryClient = useQueryClient()
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('equipment.manage')

  const [equipKeyword, setEquipKeyword] = useState('')
  const [equipPage, setEquipPage] = useState(1)
  const [calibEquipmentFilter, setCalibEquipmentFilter] = useState<string>('')
  const [calibPage, setCalibPage] = useState(1)

  const [showEquipCreate, setShowEquipCreate] = useState(false)
  const [showEquipEdit, setShowEquipEdit] = useState(false)
  const [editingEquip, setEditingEquip] = useState<Equipment | null>(null)
  const [equipForm, setEquipForm] = useState({
    name: '',
    model: '',
    serial_number: '',
    location: '',
    notes: '',
  })

  const [showCalibCreate, setShowCalibCreate] = useState(false)
  const [showCalibEdit, setShowCalibEdit] = useState(false)
  const [editingCalib, setEditingCalib] = useState<CalibrationWithEquipment | null>(null)
  const [calibForm, setCalibForm] = useState({
    equipment_id: '',
    calibrated_at: format(new Date(), 'yyyy-MM-dd'),
    next_due_at: '',
    result: '',
    notes: '',
  })

  const { data: equipmentList = [] } = useQuery({
    queryKey: ['equipment-all'],
    queryFn: async () => {
      const res = await api.get<PaginatedResponse<Equipment>>('/equipment', {
        params: { per_page: 500 },
      })
      return res.data.data
    },
  })

  const { data: equipData, isLoading: equipLoading } = useQuery({
    queryKey: ['equipment', equipKeyword, equipPage],
    queryFn: async () => {
      const params: Record<string, string | number> = { page: equipPage, per_page: 20 }
      if (equipKeyword) params.keyword = equipKeyword
      const res = await api.get<PaginatedResponse<Equipment>>('/equipment', { params })
      return res.data
    },
  })

  const { data: calibData, isLoading: calibLoading } = useQuery({
    queryKey: ['equipment-calibrations', calibEquipmentFilter || undefined, calibPage],
    queryFn: async () => {
      const params: Record<string, string | number> = { page: calibPage, per_page: 20 }
      if (calibEquipmentFilter) params.equipment_id = calibEquipmentFilter
      const res = await api.get<PaginatedResponse<CalibrationWithEquipment>>(
        '/equipment-calibrations',
        { params }
      )
      return res.data
    },
  })

  const createEquipMutation = useMutation({
    mutationFn: (payload: typeof equipForm) =>
      api.post('/equipment', {
        name: payload.name,
        model: payload.model || null,
        serial_number: payload.serial_number || null,
        location: payload.location || null,
        notes: payload.notes || null,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment'] })
      queryClient.invalidateQueries({ queryKey: ['equipment-all'] })
      setShowEquipCreate(false)
      setEquipForm({ name: '', model: '', serial_number: '', location: '', notes: '' })
      toast({ title: '成功', description: '已新增設備' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '新增失敗'), variant: 'destructive' })
    },
  })

  const updateEquipMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: Record<string, unknown> }) =>
      api.put(`/equipment/${id}`, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment'] })
      queryClient.invalidateQueries({ queryKey: ['equipment-all'] })
      setShowEquipEdit(false)
      setEditingEquip(null)
      toast({ title: '成功', description: '已更新設備' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
    },
  })

  const deleteEquipMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/equipment/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment'] })
      queryClient.invalidateQueries({ queryKey: ['equipment-all'] })
      queryClient.invalidateQueries({ queryKey: ['equipment-calibrations'] })
      toast({ title: '成功', description: '已刪除設備' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '刪除失敗'), variant: 'destructive' })
    },
  })

  const createCalibMutation = useMutation({
    mutationFn: (payload: typeof calibForm) =>
      api.post('/equipment-calibrations', {
        equipment_id: payload.equipment_id,
        calibrated_at: payload.calibrated_at,
        next_due_at: payload.next_due_at || null,
        result: payload.result || null,
        notes: payload.notes || null,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-calibrations'] })
      setShowCalibCreate(false)
      setCalibForm({
        equipment_id: '',
        calibrated_at: format(new Date(), 'yyyy-MM-dd'),
        next_due_at: '',
        result: '',
        notes: '',
      })
      toast({ title: '成功', description: '已新增校準紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '新增失敗'), variant: 'destructive' })
    },
  })

  const updateCalibMutation = useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: Record<string, unknown> }) =>
      api.put(`/equipment-calibrations/${id}`, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-calibrations'] })
      setShowCalibEdit(false)
      setEditingCalib(null)
      toast({ title: '成功', description: '已更新校準紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
    },
  })

  const deleteCalibMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/equipment-calibrations/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['equipment-calibrations'] })
      toast({ title: '成功', description: '已刪除校準紀錄' })
    },
    onError: (err: unknown) => {
      toast({ title: '錯誤', description: getApiErrorMessage(err, '刪除失敗'), variant: 'destructive' })
    },
  })

  const handleCreateEquip = () => {
    if (!equipForm.name.trim()) {
      toast({ title: '錯誤', description: '設備名稱為必填', variant: 'destructive' })
      return
    }
    createEquipMutation.mutate(equipForm)
  }

  const handleUpdateEquip = () => {
    if (!editingEquip) return
    if (!equipForm.name.trim()) {
      toast({ title: '錯誤', description: '設備名稱為必填', variant: 'destructive' })
      return
    }
    updateEquipMutation.mutate({
      id: editingEquip.id,
      payload: {
        name: equipForm.name,
        model: equipForm.model || null,
        serial_number: equipForm.serial_number || null,
        location: equipForm.location || null,
        notes: equipForm.notes || null,
      },
    })
  }

  const handleCreateCalib = () => {
    if (!calibForm.equipment_id || !calibForm.calibrated_at) {
      toast({ title: '錯誤', description: '請選擇設備並填寫校準日期', variant: 'destructive' })
      return
    }
    createCalibMutation.mutate(calibForm)
  }

  const handleUpdateCalib = () => {
    if (!editingCalib) return
    if (!calibForm.calibrated_at) {
      toast({ title: '錯誤', description: '校準日期為必填', variant: 'destructive' })
      return
    }
    updateCalibMutation.mutate({
      id: editingCalib.id,
      payload: {
        calibrated_at: calibForm.calibrated_at,
        next_due_at: calibForm.next_due_at || null,
        result: calibForm.result || null,
        notes: calibForm.notes || null,
      },
    })
  }

  const equipRecords = equipData?.data ?? []
  const equipTotalPages = equipData?.total_pages ?? 1
  const calibRecords = calibData?.data ?? []
  const calibTotalPages = calibData?.total_pages ?? 1

  return (
    <div className="space-y-6 p-6">
      <div>
        <h1 className="text-2xl font-bold flex items-center gap-2">
          <Wrench className="h-7 w-7" />
          設備與校準紀錄
        </h1>
        <p className="text-muted-foreground text-sm mt-1">實驗室 GLP 合規：設備管理與校準紀錄追蹤</p>
      </div>

      <Tabs defaultValue="equipment">
        <TabsList>
          <TabsTrigger value="equipment" className="flex items-center gap-2">
            <Wrench className="h-4 w-4" />
            設備管理
          </TabsTrigger>
          <TabsTrigger value="calibrations" className="flex items-center gap-2">
            <Ruler className="h-4 w-4" />
            校準紀錄
          </TabsTrigger>
        </TabsList>

        <TabsContent value="equipment" className="space-y-4 mt-4">
          <div className="flex flex-wrap gap-4">
            <Input
              placeholder="搜尋設備名稱或型號..."
              value={equipKeyword}
              onChange={(e) => setEquipKeyword(e.target.value)}
              className="max-w-sm"
            />
            {canManage && (
              <Button onClick={() => setShowEquipCreate(true)}>
                <Plus className="h-4 w-4 mr-2" />
                新增設備
              </Button>
            )}
          </div>
          <div className="rounded-md border">
            {equipLoading ? (
              <div className="flex justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : equipRecords.length === 0 ? (
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
                  {equipRecords.map((r) => (
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
                              onClick={() => {
                                setEditingEquip(r)
                                setEquipForm({
                                  name: r.name,
                                  model: r.model || '',
                                  serial_number: r.serial_number || '',
                                  location: r.location || '',
                                  notes: r.notes || '',
                                })
                                setShowEquipEdit(true)
                              }}
                            >
                              <Pencil className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="text-destructive hover:text-destructive"
                              onClick={() => {
                                if (window.confirm(`確定要刪除設備「${r.name}」嗎？`)) {
                                  deleteEquipMutation.mutate(r.id)
                                }
                              }}
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
          {equipTotalPages > 1 && (
            <div className="flex justify-center gap-2">
              <Button variant="outline" size="sm" disabled={equipPage <= 1} onClick={() => setEquipPage((p) => p - 1)}>
                上一頁
              </Button>
              <span className="flex items-center px-4 text-sm text-muted-foreground">
                第 {equipPage} / {equipTotalPages} 頁
              </span>
              <Button
                variant="outline"
                size="sm"
                disabled={equipPage >= equipTotalPages}
                onClick={() => setEquipPage((p) => p + 1)}
              >
                下一頁
              </Button>
            </div>
          )}
        </TabsContent>

        <TabsContent value="calibrations" className="space-y-4 mt-4">
          <div className="flex flex-wrap gap-4">
            <Select value={calibEquipmentFilter} onValueChange={setCalibEquipmentFilter}>
              <SelectTrigger className="w-[220px]">
                <SelectValue placeholder="篩選設備" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">全部設備</SelectItem>
                {equipmentList.map((e) => (
                  <SelectItem key={e.id} value={e.id}>
                    {e.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {canManage && (
              <Button
                onClick={() => {
                  setCalibForm({
                    equipment_id: calibEquipmentFilter || (equipmentList[0]?.id ?? ''),
                    calibrated_at: format(new Date(), 'yyyy-MM-dd'),
                    next_due_at: '',
                    result: '',
                    notes: '',
                  })
                  setShowCalibCreate(true)
                }}
                disabled={equipmentList.length === 0}
              >
                <Plus className="h-4 w-4 mr-2" />
                新增校準紀錄
              </Button>
            )}
          </div>
          <div className="rounded-md border">
            {calibLoading ? (
              <div className="flex justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : calibRecords.length === 0 ? (
              <div className="py-12 text-center text-muted-foreground">尚無校準紀錄</div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>設備</TableHead>
                    <TableHead>校準日期</TableHead>
                    <TableHead>下次校準</TableHead>
                    <TableHead>結果</TableHead>
                    <TableHead>備註</TableHead>
                    {canManage && <TableHead className="w-[100px]">操作</TableHead>}
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {calibRecords.map((r) => (
                    <TableRow key={r.id}>
                      <TableCell className="font-medium">{r.equipment_name}</TableCell>
                      <TableCell>{format(new Date(r.calibrated_at), 'yyyy/MM/dd', { locale: zhTW })}</TableCell>
                      <TableCell>
                        {r.next_due_at
                          ? format(new Date(r.next_due_at), 'yyyy/MM/dd', { locale: zhTW })
                          : '—'}
                      </TableCell>
                      <TableCell>{r.result || '—'}</TableCell>
                      <TableCell className="max-w-[200px] truncate">{r.notes || '—'}</TableCell>
                      {canManage && (
                        <TableCell>
                          <div className="flex gap-2">
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => {
                                setEditingCalib(r)
                                setCalibForm({
                                  equipment_id: r.equipment_id,
                                  calibrated_at: r.calibrated_at,
                                  next_due_at: r.next_due_at || '',
                                  result: r.result || '',
                                  notes: r.notes || '',
                                })
                                setShowCalibEdit(true)
                              }}
                            >
                              <Pencil className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="text-destructive hover:text-destructive"
                              onClick={() => {
                                if (window.confirm('確定要刪除此校準紀錄嗎？')) {
                                  deleteCalibMutation.mutate(r.id)
                                }
                              }}
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
          {calibTotalPages > 1 && (
            <div className="flex justify-center gap-2">
              <Button variant="outline" size="sm" disabled={calibPage <= 1} onClick={() => setCalibPage((p) => p - 1)}>
                上一頁
              </Button>
              <span className="flex items-center px-4 text-sm text-muted-foreground">
                第 {calibPage} / {calibTotalPages} 頁
              </span>
              <Button
                variant="outline"
                size="sm"
                disabled={calibPage >= calibTotalPages}
                onClick={() => setCalibPage((p) => p + 1)}
              >
                下一頁
              </Button>
            </div>
          )}
        </TabsContent>
      </Tabs>

      {/* 新增設備 Dialog */}
      <Dialog open={showEquipCreate} onOpenChange={setShowEquipCreate}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新增設備</DialogTitle>
            <DialogDescription>填寫設備基本資料</DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div>
              <Label>名稱 *</Label>
              <Input
                value={equipForm.name}
                onChange={(e) => setEquipForm({ ...equipForm, name: e.target.value })}
                placeholder="例：電子天平"
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>型號</Label>
                <Input
                  value={equipForm.model}
                  onChange={(e) => setEquipForm({ ...equipForm, model: e.target.value })}
                />
              </div>
              <div>
                <Label>序號</Label>
                <Input
                  value={equipForm.serial_number}
                  onChange={(e) => setEquipForm({ ...equipForm, serial_number: e.target.value })}
                />
              </div>
            </div>
            <div>
              <Label>位置</Label>
              <Input
                value={equipForm.location}
                onChange={(e) => setEquipForm({ ...equipForm, location: e.target.value })}
              />
            </div>
            <div>
              <Label>備註</Label>
              <Input
                value={equipForm.notes}
                onChange={(e) => setEquipForm({ ...equipForm, notes: e.target.value })}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowEquipCreate(false)}>取消</Button>
            <Button onClick={handleCreateEquip} disabled={createEquipMutation.isPending}>
              {createEquipMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              新增
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 編輯設備 Dialog */}
      <Dialog open={showEquipEdit} onOpenChange={(open) => { if (!open) setEditingEquip(null); setShowEquipEdit(open) }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>編輯設備</DialogTitle>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div>
              <Label>名稱 *</Label>
              <Input
                value={equipForm.name}
                onChange={(e) => setEquipForm({ ...equipForm, name: e.target.value })}
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>型號</Label>
                <Input
                  value={equipForm.model}
                  onChange={(e) => setEquipForm({ ...equipForm, model: e.target.value })}
                />
              </div>
              <div>
                <Label>序號</Label>
                <Input
                  value={equipForm.serial_number}
                  onChange={(e) => setEquipForm({ ...equipForm, serial_number: e.target.value })}
                />
              </div>
            </div>
            <div>
              <Label>位置</Label>
              <Input
                value={equipForm.location}
                onChange={(e) => setEquipForm({ ...equipForm, location: e.target.value })}
              />
            </div>
            <div>
              <Label>備註</Label>
              <Input
                value={equipForm.notes}
                onChange={(e) => setEquipForm({ ...equipForm, notes: e.target.value })}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowEquipEdit(false)}>取消</Button>
            <Button onClick={handleUpdateEquip} disabled={updateEquipMutation.isPending}>
              {updateEquipMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 新增校準 Dialog */}
      <Dialog open={showCalibCreate} onOpenChange={setShowCalibCreate}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新增校準紀錄</DialogTitle>
            <DialogDescription>填寫校準日期與結果</DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div>
              <Label>設備 *</Label>
              <Select
                value={calibForm.equipment_id}
                onValueChange={(v) => setCalibForm({ ...calibForm, equipment_id: v })}
              >
                <SelectTrigger>
                  <SelectValue placeholder="選擇設備" />
                </SelectTrigger>
                <SelectContent>
                  {equipmentList.map((e) => (
                    <SelectItem key={e.id} value={e.id}>
                      {e.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>校準日期 *</Label>
                <Input
                  type="date"
                  value={calibForm.calibrated_at}
                  onChange={(e) => setCalibForm({ ...calibForm, calibrated_at: e.target.value })}
                />
              </div>
              <div>
                <Label>下次校準</Label>
                <Input
                  type="date"
                  value={calibForm.next_due_at}
                  onChange={(e) => setCalibForm({ ...calibForm, next_due_at: e.target.value })}
                />
              </div>
            </div>
            <div>
              <Label>結果</Label>
              <Input
                value={calibForm.result}
                onChange={(e) => setCalibForm({ ...calibForm, result: e.target.value })}
                placeholder="例：合格"
              />
            </div>
            <div>
              <Label>備註</Label>
              <Input
                value={calibForm.notes}
                onChange={(e) => setCalibForm({ ...calibForm, notes: e.target.value })}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCalibCreate(false)}>取消</Button>
            <Button onClick={handleCreateCalib} disabled={createCalibMutation.isPending}>
              {createCalibMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              新增
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 編輯校準 Dialog */}
      <Dialog open={showCalibEdit} onOpenChange={(open) => { if (!open) setEditingCalib(null); setShowCalibEdit(open) }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>編輯校準紀錄</DialogTitle>
            {editingCalib && (
              <DialogDescription>設備：{editingCalib.equipment_name}</DialogDescription>
            )}
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div>
              <Label>校準日期 *</Label>
              <Input
                type="date"
                value={calibForm.calibrated_at}
                onChange={(e) => setCalibForm({ ...calibForm, calibrated_at: e.target.value })}
              />
            </div>
            <div>
              <Label>下次校準</Label>
              <Input
                type="date"
                value={calibForm.next_due_at}
                onChange={(e) => setCalibForm({ ...calibForm, next_due_at: e.target.value })}
              />
            </div>
            <div>
              <Label>結果</Label>
              <Input
                value={calibForm.result}
                onChange={(e) => setCalibForm({ ...calibForm, result: e.target.value })}
              />
            </div>
            <div>
              <Label>備註</Label>
              <Input
                value={calibForm.notes}
                onChange={(e) => setCalibForm({ ...calibForm, notes: e.target.value })}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCalibEdit(false)}>取消</Button>
            <Button onClick={handleUpdateCalib} disabled={updateCalibMutation.isPending}>
              {updateCalibMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
