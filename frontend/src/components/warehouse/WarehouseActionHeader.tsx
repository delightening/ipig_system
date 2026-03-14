import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
    deleteResource,
    Warehouse,
} from '@/lib/api'
import { getApiErrorMessage } from '@/lib/validation'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
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
import { Switch } from '@/components/ui/switch'
import { toast } from '@/components/ui/use-toast'
import {
    Loader2,
    Plus,
    Warehouse as WarehouseIcon,
    Trash2,
    Upload,
    Download,
    Edit3,
} from 'lucide-react'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'

interface WarehouseActionHeaderProps {
    selectedWarehouseId: string
    onWarehouseChange: (id: string) => void
    onImportClick: () => void
    onExportClick: () => void
}

interface WarehouseFormData {
    code: string
    name: string
    address: string
    is_active: boolean
}

const initialFormData: WarehouseFormData = {
    code: '',
    name: '',
    address: '',
    is_active: true,
}

export function WarehouseActionHeader({
    selectedWarehouseId,
    onWarehouseChange,
    onImportClick,
    onExportClick,
}: WarehouseActionHeaderProps) {
    const queryClient = useQueryClient()
    const { dialogState, confirm } = useConfirmDialog()
    const [showWarehouseDialog, setShowWarehouseDialog] = useState(false)
    const [editingWarehouse, setEditingWarehouse] = useState<Warehouse | null>(null)
    const [formData, setFormData] = useState<WarehouseFormData>(initialFormData)

    // 取得所有倉庫（包含停用的，以便編輯）
    const { data: allWarehouses, isLoading: loadingWarehouses } = useQuery({
        queryKey: ['all-warehouses'],
        queryFn: async () => {
            const res = await api.get<Warehouse[]>('/warehouses')
            return res.data
        },
    })

    // 過濾出啟用的倉庫供選擇器使用
    const activeWarehouses = allWarehouses?.filter(w => w.is_active) || []

    // 建立倉庫
    const createMutation = useMutation({
        mutationFn: async (data: WarehouseFormData) => {
            return api.post('/warehouses', data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['all-warehouses'] })
            queryClient.invalidateQueries({ queryKey: ['warehouses'] })
            toast({ title: '成功', description: '倉庫已建立' })
            setShowWarehouseDialog(false)
        },
        onError: (error: Error) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '建立失敗'),
                variant: 'destructive',
            })
        },
    })

    // 更新倉庫
    const updateMutation = useMutation({
        mutationFn: async ({ id, data }: { id: string; data: WarehouseFormData }) => {
            return api.put(`/warehouses/${id}`, data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['all-warehouses'] })
            queryClient.invalidateQueries({ queryKey: ['warehouses'] })
            toast({ title: '成功', description: '倉庫已更新' })
            setShowWarehouseDialog(false)
        },
        onError: (error: Error) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '更新失敗'),
                variant: 'destructive',
            })
        },
    })

    // 刪除倉庫
    const deleteMutation = useMutation({
        mutationFn: async (id: string) => {
            return deleteResource(`/warehouses/${id}`)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['all-warehouses'] })
            queryClient.invalidateQueries({ queryKey: ['warehouses'] })
            if (selectedWarehouseId === editingWarehouse?.id) {
                onWarehouseChange('')
            }
            toast({ title: '成功', description: '倉庫已刪除' })
            setShowWarehouseDialog(false)
        },
        onError: (error: Error) => {
            toast({
                title: '錯誤',
                description: getApiErrorMessage(error, '刪除失敗'),
                variant: 'destructive',
            })
        },
    })

    const handleOpenCreate = () => {
        setEditingWarehouse(null)
        setFormData(initialFormData)
        setShowWarehouseDialog(true)
    }

    const handleOpenEdit = () => {
        const warehouse = allWarehouses?.find(w => w.id === selectedWarehouseId)
        if (!warehouse) return
        setEditingWarehouse(warehouse)
        setFormData({
            code: warehouse.code,
            name: warehouse.name,
            address: warehouse.address || '',
            is_active: warehouse.is_active,
        })
        setShowWarehouseDialog(true)
    }

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (editingWarehouse) {
            updateMutation.mutate({ id: editingWarehouse.id, data: formData })
        } else {
            createMutation.mutate(formData)
        }
    }

    const handleDelete = async () => {
        if (!editingWarehouse) return
        const ok = await confirm({
            title: '刪除倉庫',
            description: `確定要刪除倉庫「${editingWarehouse.name}」嗎？此操作無法復原。`,
            variant: 'destructive',
            confirmLabel: '確認刪除',
        })
        if (ok) {
            deleteMutation.mutate(editingWarehouse.id)
        }
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">倉庫</h1>
                    <p className="text-muted-foreground">管理倉庫資料、貨架佈局與儲位庫存</p>
                </div>
                <div className="flex flex-wrap gap-2 justify-end">
                    <Button variant="outline" size="sm" onClick={onImportClick}>
                        <Upload className="mr-2 h-4 w-4" />
                        匯入倉庫
                    </Button>
                    <Button variant="outline" size="sm" onClick={onExportClick} disabled={activeWarehouses.length === 0}>
                        <Download className="mr-2 h-4 w-4" />
                        匯出倉庫
                    </Button>
                </div>
            </div>

            <Card>
                <CardHeader className="pb-3 flex-row items-center justify-between space-y-0">
                    <CardTitle className="text-base flex items-center gap-2">
                        <WarehouseIcon className="h-4 w-4" />
                        選擇倉庫
                    </CardTitle>
                    <div className="flex gap-2">
                        <Button variant="ghost" size="sm" onClick={handleOpenCreate}>
                            <Plus className="h-4 w-4 mr-1" />
                            新增倉庫
                        </Button>
                        <Button 
                            variant="ghost" 
                            size="sm" 
                            onClick={handleOpenEdit}
                            disabled={!selectedWarehouseId}
                        >
                            <Edit3 className="h-4 w-4 mr-1" />
                            編輯倉庫
                        </Button>
                    </div>
                </CardHeader>
                <CardContent>
                    <Select
                        value={selectedWarehouseId}
                        onValueChange={onWarehouseChange}
                    >
                        <SelectTrigger className="w-[300px]">
                            <SelectValue placeholder="請選擇倉庫..." />
                        </SelectTrigger>
                        <SelectContent>
                            {loadingWarehouses ? (
                                <div className="p-2 text-center">
                                    <Loader2 className="h-4 w-4 animate-spin mx-auto" />
                                </div>
                            ) : (
                                activeWarehouses.map((w) => (
                                    <SelectItem key={w.id} value={w.id}>
                                        {w.code} - {w.name}
                                    </SelectItem>
                                ))
                            )}
                        </SelectContent>
                    </Select>
                </CardContent>
            </Card>

            <Dialog open={showWarehouseDialog} onOpenChange={setShowWarehouseDialog}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>{editingWarehouse ? '編輯倉庫' : '新增倉庫'}</DialogTitle>
                        <DialogDescription>
                            填寫倉庫基本資訊
                        </DialogDescription>
                    </DialogHeader>
                    <form onSubmit={handleSubmit}>
                        <div className="grid gap-4 py-4">
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="code" className="text-right">代碼 *</Label>
                                <Input
                                    id="code"
                                    value={formData.code}
                                    onChange={(e) => setFormData({ ...formData, code: e.target.value })}
                                    className="col-span-3"
                                    placeholder="如 WH001"
                                    required
                                    disabled={!!editingWarehouse}
                                />
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="wh-name" className="text-right">名稱 *</Label>
                                <Input
                                    id="wh-name"
                                    value={formData.name}
                                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                                    className="col-span-3"
                                    placeholder="如 大倉庫"
                                    required
                                />
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="address" className="text-right">地址</Label>
                                <Input
                                    id="address"
                                    value={formData.address}
                                    onChange={(e) => setFormData({ ...formData, address: e.target.value })}
                                    className="col-span-3"
                                />
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="is_active" className="text-right">啟用狀態</Label>
                                <div className="col-span-3 flex items-center gap-2">
                                    <Switch
                                        id="is_active"
                                        checked={formData.is_active}
                                        onCheckedChange={(checked) => setFormData({ ...formData, is_active: checked })}
                                    />
                                    <span className="text-sm text-muted-foreground">
                                        {formData.is_active ? '已啟用' : '已停用'}
                                    </span>
                                </div>
                            </div>
                        </div>
                        <DialogFooter className="flex justify-between sm:justify-between">
                            <div>
                                {editingWarehouse && (
                                    <Button
                                        type="button"
                                        variant="destructive"
                                        onClick={handleDelete}
                                        disabled={deleteMutation.isPending}
                                    >
                                        {deleteMutation.isPending ? (
                                            <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                                        ) : (
                                            <Trash2 className="h-4 w-4 mr-1" />
                                        )}
                                        刪除
                                    </Button>
                                )}
                            </div>
                            <div className="flex gap-2">
                                <Button type="button" variant="outline" onClick={() => setShowWarehouseDialog(false)}>
                                    取消
                                </Button>
                                <Button
                                    type="submit"
                                    disabled={createMutation.isPending || updateMutation.isPending}
                                >
                                    {(createMutation.isPending || updateMutation.isPending) && (
                                        <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                                    )}
                                    {editingWarehouse ? '更新' : '建立'}
                                </Button>
                            </div>
                        </DialogFooter>
                    </form>
                </DialogContent>
            </Dialog>
            <ConfirmDialog state={dialogState} />
        </div>
    )
}
