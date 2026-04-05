import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import api, { deleteResource, Warehouse } from '@/lib/api'
import { useDebounce } from '@/hooks/useDebounce'
import { useTableSort } from '@/hooks/useTableSort'
import { STALE_TIME } from '@/lib/query'
import { Button } from '@/components/ui/button'
import { TableEmptyRow } from '@/components/ui/empty-state'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
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
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import { logger } from '@/lib/logger'
import { getApiErrorMessage, warehouseFormSchema, type WarehouseFormData } from '@/lib/validation'
import { PageHeader } from '@/components/ui/page-header'
import { Plus, Search, Edit, Trash2, Loader2, Warehouse as WarehouseIcon, Upload, Download } from 'lucide-react'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { WarehouseImportDialog } from '@/components/warehouse/WarehouseImportDialog'

export function WarehousesPage() {
  const queryClient = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()
  const [search, setSearch] = useState('')
  const debouncedSearch = useDebounce(search, 400)
  const [dialogOpen, setDialogOpen] = useState(false)
  const [showImportDialog, setShowImportDialog] = useState(false)
  const [editingWarehouse, setEditingWarehouse] = useState<Warehouse | null>(null)
  const { register, handleSubmit: rhfHandleSubmit, reset, formState: { errors } } = useForm<WarehouseFormData>({
    resolver: zodResolver(warehouseFormSchema),
    defaultValues: { name: '', code: '', address: '', description: '', is_active: true },
  })

  const { data: warehouses, isLoading } = useQuery({
    queryKey: ['warehouses', debouncedSearch],
    staleTime: STALE_TIME.LIST,
    queryFn: async () => {
      const searchParams = new URLSearchParams()
      searchParams.set('is_active', 'true')
      if (debouncedSearch) searchParams.set('keyword', debouncedSearch)
      const qs = searchParams.toString()
      const url = qs ? `/warehouses?${qs}` : '/warehouses'
      const response = await api.get<Warehouse[]>(url)
      return response.data
    },
  })

  const { sortedData: sortedWarehouses, sort, toggleSort } = useTableSort(warehouses)

  const invalidateWarehouseRelated = () => {
    queryClient.invalidateQueries({ queryKey: ['warehouses'] })
    queryClient.invalidateQueries({ queryKey: ['storage-locations'] })
    queryClient.invalidateQueries({ queryKey: ['storage-location-inventory'] })
    queryClient.invalidateQueries({ queryKey: ['unassigned-inventory'] })
    queryClient.invalidateQueries({ queryKey: ['inventory'] })
    queryClient.invalidateQueries({ queryKey: ['stock-ledger'] })
    queryClient.invalidateQueries({ queryKey: ['warehouse-report'] })
  }

  const createMutation = useMutation({
    mutationFn: (data: WarehouseFormData) => api.post('/warehouses', data),
    onSuccess: () => {
      invalidateWarehouseRelated()
      toast({ title: '成功', description: '倉庫已建立' })
      setDialogOpen(false)
      resetForm()
    },
    onError: (error: unknown) => {
      logger.error('Create warehouse error:', error)
      const errorMessage = getApiErrorMessage(error, '建立失敗')
      toast({
        title: '錯誤',
        description: errorMessage,
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: WarehouseFormData }) =>
      api.put(`/warehouses/${id}`, data),
    onSuccess: () => {
      invalidateWarehouseRelated()
      toast({ title: '成功', description: '倉庫已更新' })
      setDialogOpen(false)
      resetForm()
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '更新失敗'),
        variant: 'destructive',
      })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteResource(`/warehouses/${id}`),
    onSuccess: () => {
      invalidateWarehouseRelated()
      toast({ title: '成功', description: '倉庫已刪除' })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '刪除失敗'),
        variant: 'destructive',
      })
    },
  })

  const resetForm = () => {
    reset({ name: '', code: '', address: '', description: '', is_active: true })
    setEditingWarehouse(null)
  }

  const handleEdit = (warehouse: Warehouse) => {
    setEditingWarehouse(warehouse)
    reset({ name: warehouse.name, code: '', address: '', description: '', is_active: true })
    setDialogOpen(true)
  }

  const onSubmit = (data: WarehouseFormData) => {
    if (editingWarehouse) {
      updateMutation.mutate({ id: editingWarehouse.id, data })
    } else {
      createMutation.mutate(data)
    }
  }

  const handleExportCSV = () => {
    if (!warehouses || warehouses.length === 0) {
      toast({ title: '無資料可匯出', description: '請先新增倉庫', variant: 'destructive' })
      return
    }
    const headers = ['代碼', '名稱', '地址', '狀態']
    const rows = warehouses.map(w => [
      w.code,
      w.name,
      w.address || '',
      w.is_active ? '啟用' : '停用',
    ])
    const csvContent = [headers, ...rows]
      .map(row => row.map(cell => `"${String(cell).replace(/"/g, '""')}"`).join(','))
      .join('\n')
    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8;' })
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = `warehouses_${new Date().toISOString().split('T')[0]}.csv`
    link.click()
    URL.revokeObjectURL(link.href)
    toast({ title: '匯出成功', description: `已匯出 ${warehouses.length} 筆倉庫` })
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="倉庫管理"
        description="管理系統中的倉庫資料"
        actions={
          <>
            <Button variant="outline" size="sm" onClick={() => setShowImportDialog(true)}>
              <Upload className="mr-2 h-4 w-4" />
              匯入
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleExportCSV}
              disabled={!warehouses || warehouses.length === 0}
            >
              <Download className="mr-2 h-4 w-4" />
              匯出
            </Button>
            <Button size="sm" onClick={() => { resetForm(); setDialogOpen(true) }}>
              <Plus className="mr-2 h-4 w-4" />
              新增倉庫
            </Button>
          </>
        }
      />

      <div className="flex gap-4">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜尋倉庫..."
            aria-label="搜尋倉庫"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>
      </div>

      <div className="rounded-xl border bg-card shadow-sm overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow>
              <SortableTableHead sortKey="code" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>代碼</SortableTableHead>
              <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>名稱</SortableTableHead>
              <SortableTableHead sortKey="is_active" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>狀態</SortableTableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={4} className="p-0">
                  <TableSkeleton rows={5} cols={4} />
                </TableCell>
              </TableRow>
            ) : sortedWarehouses && sortedWarehouses.length > 0 ? (
              sortedWarehouses.map((warehouse) => (
                <TableRow key={warehouse.id}>
                  <TableCell className="font-mono">{warehouse.code}</TableCell>
                  <TableCell className="font-medium">{warehouse.name}</TableCell>
                  <TableCell>
                    {warehouse.is_active ? (
                      <Badge variant="success">啟用</Badge>
                    ) : (
                      <Badge variant="destructive">停用</Badge>
                    )}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button variant="ghost" size="icon" onClick={() => handleEdit(warehouse)} aria-label="編輯">
                      <Edit className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={async () => {
                        const ok = await confirm({ title: '刪除倉庫', description: '確定要刪除此倉庫嗎？', variant: 'destructive', confirmLabel: '確認刪除' })
                        if (ok) {
                          deleteMutation.mutate(warehouse.id)
                        }
                      }}
                      aria-label="刪除"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableEmptyRow colSpan={4} icon={WarehouseIcon} title="尚無倉庫資料" />
            )}
          </TableBody>
        </Table>
      </div>

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editingWarehouse ? '編輯倉庫' : '新增倉庫'}</DialogTitle>
            <DialogDescription>
              {editingWarehouse ? '修改倉庫資料' : '建立新的倉庫'}
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={rhfHandleSubmit(onSubmit)}>
            <div className="grid gap-4 py-4">
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="name" className="text-right">名稱</Label>
                <div className="col-span-3 space-y-1">
                  <Input
                    id="name"
                    {...register('name')}
                  />
                  {errors.name && (
                    <p className="text-sm text-destructive">{errors.name.message}</p>
                  )}
                </div>
              </div>
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setDialogOpen(false)}>
                取消
              </Button>
              <Button type="submit" disabled={createMutation.isPending || updateMutation.isPending}>
                {(createMutation.isPending || updateMutation.isPending) && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                {editingWarehouse ? '更新' : '建立'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
      <ConfirmDialog state={dialogState} />
      <WarehouseImportDialog open={showImportDialog} onOpenChange={setShowImportDialog} />
    </div>
  )
}
