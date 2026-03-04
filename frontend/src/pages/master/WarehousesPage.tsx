import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Warehouse } from '@/lib/api'
import { useDebounce } from '@/hooks/useDebounce'
import { STALE_TIME } from '@/lib/query'
import { Button } from '@/components/ui/button'
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
import { getApiErrorMessage } from '@/lib/validation'
import { Plus, Search, Edit, Trash2, Loader2, Warehouse as WarehouseIcon, Upload, Download } from 'lucide-react'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
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
  const [formData, setFormData] = useState({
    name: '',
  })

  const { data: warehouses, isLoading } = useQuery({
    queryKey: ['warehouses', debouncedSearch],
    staleTime: STALE_TIME.LIST,
    queryFn: async () => {
      const params = debouncedSearch ? `?keyword=${encodeURIComponent(debouncedSearch)}` : ''
      const response = await api.get<Warehouse[]>(`/warehouses${params}`)
      return response.data
    },
  })

  const createMutation = useMutation({
    mutationFn: (data: typeof formData) => api.post('/warehouses', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['warehouses'] })
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
    mutationFn: ({ id, data }: { id: string; data: typeof formData }) =>
      api.put(`/warehouses/${id}`, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['warehouses'] })
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
    mutationFn: (id: string) => api.delete(`/warehouses/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['warehouses'] })
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
    setFormData({ name: '' })
    setEditingWarehouse(null)
  }

  const handleEdit = (warehouse: Warehouse) => {
    setEditingWarehouse(warehouse)
    setFormData({
      name: warehouse.name,
    })
    setDialogOpen(true)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (editingWarehouse) {
      updateMutation.mutate({ id: editingWarehouse.id, data: formData })
    } else {
      createMutation.mutate(formData)
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
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">倉庫管理</h1>
          <p className="text-muted-foreground">管理系統中的倉庫資料</p>
        </div>
        <div className="flex items-center gap-2">
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
          <Button onClick={() => { resetForm(); setDialogOpen(true) }}>
            <Plus className="mr-2 h-4 w-4" />
            新增倉庫
          </Button>
        </div>
      </div>

      <div className="flex gap-4">
        <div className="flex gap-2 flex-1 max-w-md">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="搜尋倉庫..."
              aria-label="搜尋倉庫"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); queryClient.invalidateQueries({ queryKey: ['warehouses'] }) } }}
              className="pl-9"
            />
          </div>
          <Button type="button" variant="secondary" onClick={() => queryClient.invalidateQueries({ queryKey: ['warehouses'] })} aria-label="搜尋">
            <Search className="h-4 w-4 md:mr-1.5" />
            <span className="hidden md:inline">搜尋</span>
          </Button>
        </div>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>代碼</TableHead>
              <TableHead>名稱</TableHead>
              <TableHead>狀態</TableHead>
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
            ) : warehouses && warehouses.length > 0 ? (
              warehouses.map((warehouse) => (
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
              <TableRow>
                <TableCell colSpan={4} className="text-center py-8">
                  <WarehouseIcon className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                  <p className="text-muted-foreground">尚無倉庫資料</p>
                </TableCell>
              </TableRow>
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
          <form onSubmit={handleSubmit}>
            <div className="grid gap-4 py-4">
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="name" className="text-right">名稱</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="col-span-3"
                  required
                />
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
