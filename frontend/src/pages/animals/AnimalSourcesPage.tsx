import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import api, { deleteResource, AnimalSource } from '@/lib/api'
import { useTableSort } from '@/hooks/useTableSort'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { StatusBadge } from '@/components/ui/status-badge'
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
import { getApiErrorMessage, animalSourceFormSchema, type AnimalSourceFormData } from '@/lib/validation'
import {
  Plus,
  Edit2,
  Trash2,
  Loader2,
  Building2,
  Phone,
  User,
  MapPin,
} from 'lucide-react'
import { useAuthStore } from '@/stores/auth'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { GuestHide } from '@/components/ui/guest-hide'

const defaultFormValues: AnimalSourceFormData = {
  code: '',
  name: '',
  address: '',
  contact: '',
  phone: '',
  phone_ext: '',
  is_active: true,
  sort_order: 0,
}

export function AnimalSourcesPage() {
  const isGuest = useAuthStore((s) => s.isGuest)()
  const queryClient = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()

  const [showDialog, setShowDialog] = useState(false)
  const [editingSource, setEditingSource] = useState<AnimalSource | null>(null)
  const { register, handleSubmit: rhfHandleSubmit, reset, setValue, watch, formState: { errors } } = useForm<AnimalSourceFormData>({
    resolver: zodResolver(animalSourceFormSchema),
    defaultValues: defaultFormValues,
  })
  const isActiveValue = watch('is_active')

  // Query sources
  const { data: sources, isLoading } = useQuery({
    queryKey: ['animal-sources'],
    queryFn: async () => {
      const res = await api.get<AnimalSource[]>('/animal-sources')
      return res.data
    },
    staleTime: 600_000,
  })

  const { sortedData: sortedSources, sort, toggleSort } = useTableSort(sources)

  // Create mutation
  const createMutation = useMutation({
    mutationFn: async (data: AnimalSourceFormData) => {
      return api.post('/animal-sources', data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-sources'] })
      toast({ title: '成功', description: '動物來源已新增' })
      handleCloseDialog()
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '新增失敗'),
        variant: 'destructive',
      })
    },
  })

  // Update mutation
  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: Partial<AnimalSourceFormData> }) => {
      return api.put(`/animal-sources/${id}`, data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-sources'] })
      toast({ title: '成功', description: '動物來源已更新' })
      handleCloseDialog()
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '更新失敗'),
        variant: 'destructive',
      })
    },
  })

  // Delete mutation
  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      return deleteResource(`/animal-sources/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-sources'] })
      toast({ title: '成功', description: '動物來源已刪除' })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '刪除失敗'),
        variant: 'destructive',
      })
    },
  })

  const handleOpenDialog = (source?: AnimalSource) => {
    if (source) {
      setEditingSource(source)
      reset({
        code: source.code,
        name: source.name,
        address: source.address || '',
        contact: source.contact || '',
        phone: source.phone || '',
        phone_ext: source.phone_ext || '',
        is_active: source.is_active,
        sort_order: source.sort_order,
      })
    } else {
      setEditingSource(null)
      reset(defaultFormValues)
    }
    setShowDialog(true)
  }

  const handleCloseDialog = () => {
    setShowDialog(false)
    setEditingSource(null)
    reset(defaultFormValues)
  }

  const onSubmit = (data: AnimalSourceFormData) => {
    if (editingSource) {
      updateMutation.mutate({ id: editingSource.id, data })
    } else {
      createMutation.mutate(data)
    }
  }

  const handleDelete = async (source: AnimalSource) => {
    const ok = await confirm({ title: '刪除動物來源', description: `確定要刪除來源「${source.name}」嗎？`, variant: 'destructive', confirmLabel: '確認刪除' })
    if (ok) {
      deleteMutation.mutate(source.id)
    }
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="動物來源管理"
        description="管理動物的來源/供應商資訊"
        actions={
          <GuestHide>
            <Button size="sm" onClick={() => handleOpenDialog()} className="gap-2 bg-primary hover:bg-primary/90">
              <Plus className="h-4 w-4" />
              新增來源
            </Button>
          </GuestHide>
        }
      />

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <SortableTableHead sortKey="sort_order" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                排序
              </SortableTableHead>
              <SortableTableHead sortKey="code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                代碼
              </SortableTableHead>
              <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                名稱
              </SortableTableHead>
              <SortableTableHead sortKey="address" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                地址
              </SortableTableHead>
              <SortableTableHead sortKey="contact" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                聯絡人
              </SortableTableHead>
              <SortableTableHead sortKey="phone" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                電話
              </SortableTableHead>
              <SortableTableHead sortKey="is_active" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>
                狀態
              </SortableTableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={8} className="text-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                </TableCell>
              </TableRow>
            ) : sortedSources && sortedSources.length > 0 ? (
              sortedSources.map((source) => (
                <TableRow key={source.id}>
                  <TableCell>{source.sort_order}</TableCell>
                  <TableCell className="font-mono font-medium">{source.code}</TableCell>
                  <TableCell className="font-medium">{source.name}</TableCell>
                  <TableCell>
                    {source.address ? (
                      <span className="flex items-center gap-1 text-sm text-muted-foreground">
                        <MapPin className="h-3 w-3" />
                        {source.address}
                      </span>
                    ) : '-'}
                  </TableCell>
                  <TableCell>
                    {source.contact ? (
                      <span className="flex items-center gap-1 text-sm text-muted-foreground">
                        <User className="h-3 w-3" />
                        {source.contact}
                      </span>
                    ) : '-'}
                  </TableCell>
                  <TableCell>
                    {source.phone ? (
                      <span className="flex items-center gap-1 text-sm text-muted-foreground">
                        <Phone className="h-3 w-3" />
                        {source.phone}
                        {source.phone_ext ? ` #${source.phone_ext}` : ''}
                      </span>
                    ) : '-'}
                  </TableCell>
                  <TableCell>
                    <StatusBadge variant={source.is_active ? 'success' : 'neutral'}>
                      {source.is_active ? '啟用' : '停用'}
                    </StatusBadge>
                  </TableCell>
                  <TableCell className="text-right">
                    <GuestHide>
                      <div className="flex items-center justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleOpenDialog(source)}
                          aria-label="編輯"
                        >
                          <Edit2 className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleDelete(source)}
                          disabled={deleteMutation.isPending}
                          aria-label="刪除"
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      </div>
                    </GuestHide>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow
                colSpan={8}
                icon={Building2}
                title="尚無來源資料"
                action={isGuest ? undefined : { label: '新增第一個來源', onClick: () => handleOpenDialog(), icon: Plus }}
              />
            )}
          </TableBody>
        </Table>
      </div>

      {/* Add/Edit Dialog */}
      <Dialog open={showDialog} onOpenChange={setShowDialog}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>
              {editingSource ? '編輯動物來源' : '新增動物來源'}
            </DialogTitle>
            <DialogDescription>
              {editingSource ? '修改來源資訊' : '輸入新來源的資訊'}
            </DialogDescription>
          </DialogHeader>

          <form onSubmit={rhfHandleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="code">代碼 *</Label>
                <Input
                  id="code"
                  {...register('code')}
                  placeholder="如：SOURCE01"
                />
                {errors.code && (
                  <p className="text-sm text-destructive">{errors.code.message}</p>
                )}
              </div>
              <div className="space-y-2">
                <Label htmlFor="name">名稱 *</Label>
                <Input
                  id="name"
                  {...register('name')}
                  placeholder="來源名稱"
                />
                {errors.name && (
                  <p className="text-sm text-destructive">{errors.name.message}</p>
                )}
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="address">地址</Label>
              <Input
                id="address"
                {...register('address')}
                placeholder="完整地址"
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="contact">聯絡人</Label>
                <Input
                  id="contact"
                  {...register('contact')}
                  placeholder="聯絡人姓名"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="phone">電話</Label>
                <div className="flex gap-2">
                  <Input
                    id="phone"
                    className="flex-1"
                    {...register('phone')}
                    placeholder="聯絡電話"
                  />
                  <div className="flex items-center gap-2 shrink-0">
                    <span className="text-muted-foreground">#</span>
                    <Input
                      className="w-24"
                      placeholder="分機"
                      {...register('phone_ext')}
                    />
                  </div>
                </div>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="sort_order">排序</Label>
                <Input
                  id="sort_order"
                  type="number"
                  {...register('sort_order', { valueAsNumber: true })}
                />
              </div>
              <div className="space-y-2">
                <Label>狀態</Label>
                <div className="flex items-center gap-4 pt-2">
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="is_active_radio"
                      checked={isActiveValue}
                      onChange={() => setValue('is_active', true)}
                      className="w-4 h-4 text-primary"
                    />
                    <span className="text-sm">啟用</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="is_active_radio"
                      checked={!isActiveValue}
                      onChange={() => setValue('is_active', false)}
                      className="w-4 h-4 text-primary"
                    />
                    <span className="text-sm">停用</span>
                  </label>
                </div>
              </div>
            </div>

            <DialogFooter>
              <Button type="button" variant="outline" onClick={handleCloseDialog}>
                取消
              </Button>
              <Button
                type="submit"
                disabled={createMutation.isPending || updateMutation.isPending}
                className="bg-primary hover:bg-primary/90"
              >
                {(createMutation.isPending || updateMutation.isPending) && (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                )}
                {editingSource ? '更新' : '新增'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
