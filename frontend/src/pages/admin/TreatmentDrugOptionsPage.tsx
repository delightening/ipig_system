/**
 * 藥物選單管理頁 — 後台管理介面
 *
 * 功能：
 * - CRUD 藥物選項
 * - 搜尋/篩選（關鍵字、分類、啟用狀態）
 * - 從 ERP 匯入藥物
 */

import { useState } from 'react'
import { useDialogSet } from '@/hooks/useDialogSet'
import { useSelection } from '@/hooks/useSelection'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { treatmentDrugApi } from '@/lib/api'
import api from '@/lib/api'
import type {
    TreatmentDrugOption,
    CreateTreatmentDrugRequest,
    UpdateTreatmentDrugRequest,
} from '@/types/treatment-drug'
import { DRUG_CATEGORIES, DOSAGE_UNITS } from '@/types/treatment-drug'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Plus, Search, Pencil, Trash2, Download, Loader2, Package, Check, XCircle } from 'lucide-react'
import { cn } from '@/lib/utils'

// 產品型別（用於 ERP 匯入）
interface Product {
    id: string
    name: string
    sku: string
    base_uom: string
    spec: string | null
    is_active: boolean
}

export function TreatmentDrugOptionsPage() {
    const queryClient = useQueryClient()

    // 篩選狀態
    const [keyword, setKeyword] = useState('')
    const [filterCategory, setFilterCategory] = useState<string>('all')
    const [filterActive, setFilterActive] = useState<string>('all')

    // Dialog 狀態
    const dialogs = useDialogSet(['create', 'edit', 'import'] as const)
    const [editingDrug, setEditingDrug] = useState<TreatmentDrugOption | null>(null)

    // 表單狀態
    const [form, setForm] = useState<CreateTreatmentDrugRequest>({
        name: '',
        display_name: '',
        default_dosage_unit: '',
        available_units: [],
        category: '',
        sort_order: 0,
    })

    // 資料查詢
    const { data: drugs = [], isLoading } = useQuery({
        queryKey: ['admin-treatment-drugs', keyword, filterCategory, filterActive],
        queryFn: async () => {
            const params: Record<string, string | boolean | undefined> = {}
            if (keyword) params.keyword = keyword
            if (filterCategory !== 'all') params.category = filterCategory
            if (filterActive !== 'all') params.is_active = filterActive === 'active'
            const res = await treatmentDrugApi.adminList(params as Record<string, string | boolean | undefined>)
            return res.data
        },
    })

    // 建立
    const createMutation = useMutation({
        mutationFn: (data: CreateTreatmentDrugRequest) => treatmentDrugApi.create(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['admin-treatment-drugs'] })
            queryClient.invalidateQueries({ queryKey: ['treatment-drugs'] })
            dialogs.close('create')
            resetForm()
            toast({ title: '成功', description: '已新增藥物選項' })
        },
        onError: (err: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(err, '新增失敗'), variant: 'destructive' })
        },
    })

    // 更新
    const updateMutation = useMutation({
        mutationFn: ({ id, data }: { id: string; data: UpdateTreatmentDrugRequest }) =>
            treatmentDrugApi.update(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['admin-treatment-drugs'] })
            queryClient.invalidateQueries({ queryKey: ['treatment-drugs'] })
            dialogs.close('edit')
            setEditingDrug(null)
            toast({ title: '成功', description: '已更新藥物選項' })
        },
        onError: (err: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(err, '更新失敗'), variant: 'destructive' })
        },
    })

    // 刪除（軟刪除）
    const deleteMutation = useMutation({
        mutationFn: (id: string) => treatmentDrugApi.delete(id),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['admin-treatment-drugs'] })
            queryClient.invalidateQueries({ queryKey: ['treatment-drugs'] })
            toast({ title: '成功', description: '已停用藥物選項' })
        },
    })

    const resetForm = () => {
        setForm({
            name: '',
            display_name: '',
            default_dosage_unit: '',
            available_units: [],
            category: '',
            sort_order: 0,
        })
    }

    const openEditDialog = (drug: TreatmentDrugOption) => {
        setEditingDrug(drug)
        setForm({
            name: drug.name,
            display_name: drug.display_name || '',
            default_dosage_unit: drug.default_dosage_unit || '',
            available_units: drug.available_units || [],
            category: drug.category || '',
            sort_order: drug.sort_order,
        })
        dialogs.open('edit')
    }

    const handleCreate = () => {
        if (!form.name.trim()) {
            toast({ title: '錯誤', description: '藥品名稱為必填', variant: 'destructive' })
            return
        }
        createMutation.mutate(form)
    }

    const handleUpdate = () => {
        if (!editingDrug) return
        updateMutation.mutate({
            id: editingDrug.id,
            data: {
                name: form.name,
                display_name: form.display_name || undefined,
                default_dosage_unit: form.default_dosage_unit || undefined,
                available_units: form.available_units?.length ? form.available_units : undefined,
                category: form.category || undefined,
                sort_order: form.sort_order,
            },
        })
    }

    const handleToggleActive = (drug: TreatmentDrugOption) => {
        updateMutation.mutate({
            id: drug.id,
            data: { is_active: !drug.is_active },
        })
    }

    // 單位多選處理
    const toggleUnit = (unit: string) => {
        setForm((prev) => ({
            ...prev,
            available_units: prev.available_units?.includes(unit)
                ? prev.available_units.filter((u) => u !== unit)
                : [...(prev.available_units || []), unit],
        }))
    }

    return (
        <div className="space-y-6">
            {/* 頁面標題 */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">藥物選單管理</h1>
                    <p className="text-muted-foreground">管理治療方式用藥的下拉選單選項</p>
                </div>
                <div className="flex gap-2">
                    <Button
                        variant="outline"
                        onClick={() => dialogs.open('import')}
                    >
                        <Download className="h-4 w-4 mr-2" /> 從 ERP 匯入
                    </Button>
                    <Button onClick={() => { resetForm(); dialogs.open('create') }}>
                        <Plus className="h-4 w-4 mr-2" /> 新增藥物
                    </Button>
                </div>
            </div>

            {/* 篩選列 */}
            <div className="flex flex-wrap gap-3 items-center bg-white p-4 rounded-lg border">
                <div className="relative flex-1 min-w-[200px]">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-400" />
                    <Input
                        placeholder="搜尋藥物名稱..."
                        value={keyword}
                        onChange={(e) => setKeyword(e.target.value)}
                        className="pl-9"
                    />
                </div>
                <Select value={filterCategory} onValueChange={setFilterCategory}>
                    <SelectTrigger className="w-[140px]">
                        <SelectValue placeholder="分類" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="all">全部分類</SelectItem>
                        {DRUG_CATEGORIES.map((cat) => (
                            <SelectItem key={cat} value={cat}>{cat}</SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                <Select value={filterActive} onValueChange={setFilterActive}>
                    <SelectTrigger className="w-[120px]">
                        <SelectValue placeholder="狀態" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="all">全部</SelectItem>
                        <SelectItem value="active">啟用中</SelectItem>
                        <SelectItem value="inactive">已停用</SelectItem>
                    </SelectContent>
                </Select>
            </div>

            {/* 藥物列表 */}
            <div className="bg-white rounded-lg border overflow-hidden">
                {isLoading ? (
                    <div className="flex items-center justify-center py-12">
                        <Loader2 className="h-6 w-6 animate-spin text-blue-500" />
                        <span className="ml-2 text-slate-500">載入中...</span>
                    </div>
                ) : drugs.length === 0 ? (
                    <div className="text-center py-12 text-slate-500">
                        <Package className="h-12 w-12 mx-auto mb-3 text-slate-300" />
                        <p>尚無藥物選項</p>
                        <p className="text-sm mt-1">點擊「新增藥物」或「從 ERP 匯入」開始建立</p>
                    </div>
                ) : (
                    <table className="w-full">
                        <thead className="bg-slate-50 border-b">
                            <tr>
                                <th className="text-left px-4 py-3 text-xs font-medium text-slate-500 uppercase">藥物名稱</th>
                                <th className="text-left px-4 py-3 text-xs font-medium text-slate-500 uppercase">顯示名稱</th>
                                <th className="text-left px-4 py-3 text-xs font-medium text-slate-500 uppercase">分類</th>
                                <th className="text-left px-4 py-3 text-xs font-medium text-slate-500 uppercase">預設單位</th>
                                <th className="text-center px-4 py-3 text-xs font-medium text-slate-500 uppercase">排序</th>
                                <th className="text-center px-4 py-3 text-xs font-medium text-slate-500 uppercase">狀態</th>
                                <th className="text-center px-4 py-3 text-xs font-medium text-slate-500 uppercase">ERP</th>
                                <th className="text-right px-4 py-3 text-xs font-medium text-slate-500 uppercase">操作</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y">
                            {drugs.map((drug) => (
                                <tr key={drug.id} className={cn('hover:bg-slate-50', !drug.is_active && 'opacity-50')}>
                                    <td className="px-4 py-3 text-sm font-medium text-slate-700">{drug.name}</td>
                                    <td className="px-4 py-3 text-sm text-slate-500">{drug.display_name || '—'}</td>
                                    <td className="px-4 py-3 text-sm">
                                        {drug.category ? (
                                            <span className="px-2 py-0.5 rounded-full text-xs bg-blue-50 text-blue-600">
                                                {drug.category}
                                            </span>
                                        ) : '—'}
                                    </td>
                                    <td className="px-4 py-3 text-sm text-slate-500">{drug.default_dosage_unit || '—'}</td>
                                    <td className="px-4 py-3 text-sm text-center text-slate-400">{drug.sort_order}</td>
                                    <td className="px-4 py-3 text-center">
                                        <button
                                            onClick={() => handleToggleActive(drug)}
                                            className={cn(
                                                'px-2 py-1 rounded text-xs font-medium transition-colors',
                                                drug.is_active
                                                    ? 'bg-green-50 text-green-700 hover:bg-green-100'
                                                    : 'bg-red-50 text-red-600 hover:bg-red-100'
                                            )}
                                        >
                                            {drug.is_active ? '啟用' : '停用'}
                                        </button>
                                    </td>
                                    <td className="px-4 py-3 text-center">
                                        {drug.erp_product_id ? (
                                            <Check className="h-4 w-4 text-green-500 mx-auto" />
                                        ) : (
                                            <XCircle className="h-4 w-4 text-slate-300 mx-auto" />
                                        )}
                                    </td>
                                    <td className="px-4 py-3 text-right">
                                        <div className="flex justify-end gap-1">
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() => openEditDialog(drug)}
                                            >
                                                <Pencil className="h-4 w-4" />
                                            </Button>
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() => {
                                                    if (confirm(`確定要${drug.is_active ? '停用' : '啟用'}「${drug.name}」嗎？`)) {
                                                        if (drug.is_active) {
                                                            deleteMutation.mutate(drug.id)
                                                        } else {
                                                            handleToggleActive(drug)
                                                        }
                                                    }
                                                }}
                                            >
                                                <Trash2 className="h-4 w-4 text-red-500" />
                                            </Button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                )}
            </div>

            {/* 新增 / 編輯 Dialog */}
            <DrugFormDialog
                open={dialogs.isOpen('create')}
                onOpenChange={dialogs.setOpen('create')}
                title="新增藥物選項"
                form={form}
                setForm={setForm}
                onSubmit={handleCreate}
                isLoading={createMutation.isPending}
                toggleUnit={toggleUnit}
            />
            <DrugFormDialog
                open={dialogs.isOpen('edit')}
                onOpenChange={(open) => { dialogs.setOpen('edit')(open); if (!open) setEditingDrug(null) }}
                title="編輯藥物選項"
                form={form}
                setForm={setForm}
                onSubmit={handleUpdate}
                isLoading={updateMutation.isPending}
                toggleUnit={toggleUnit}
            />

            {/* ERP 匯入 Dialog */}
            <ErpImportDialog
                open={dialogs.isOpen('import')}
                onOpenChange={dialogs.setOpen('import')}
            />
        </div>
    )
}

// ============================================
// 藥物表單 Dialog
// ============================================

function DrugFormDialog({
    open,
    onOpenChange,
    title,
    form,
    setForm,
    onSubmit,
    isLoading,
    toggleUnit,
}: {
    open: boolean
    onOpenChange: (open: boolean) => void
    title: string
    form: CreateTreatmentDrugRequest
    setForm: (fn: (prev: CreateTreatmentDrugRequest) => CreateTreatmentDrugRequest) => void
    onSubmit: () => void
    isLoading: boolean
    toggleUnit: (unit: string) => void
}) {
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-md">
                <DialogHeader>
                    <DialogTitle>{title}</DialogTitle>
                    <DialogDescription>設定藥物名稱、預設單位和分類</DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-2">
                    <div>
                        <Label>藥品名稱 *</Label>
                        <Input
                            value={form.name}
                            onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                            placeholder="例：Meloxicam"
                        />
                    </div>
                    <div>
                        <Label>顯示名稱</Label>
                        <Input
                            value={form.display_name || ''}
                            onChange={(e) => setForm((prev) => ({ ...prev, display_name: e.target.value }))}
                            placeholder="例：Meloxicam（美洛昔康）"
                        />
                    </div>
                    <div>
                        <Label>分類</Label>
                        <Select
                            value={form.category || ''}
                            onValueChange={(v) => setForm((prev) => ({ ...prev, category: v }))}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder="選擇分類" />
                            </SelectTrigger>
                            <SelectContent>
                                {DRUG_CATEGORIES.map((cat) => (
                                    <SelectItem key={cat} value={cat}>{cat}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div>
                        <Label>預設劑量單位</Label>
                        <Select
                            value={form.default_dosage_unit || ''}
                            onValueChange={(v) => setForm((prev) => ({ ...prev, default_dosage_unit: v }))}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder="選擇單位" />
                            </SelectTrigger>
                            <SelectContent>
                                {DOSAGE_UNITS.map((unit) => (
                                    <SelectItem key={unit} value={unit}>{unit}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div>
                        <Label>可用單位（多選）</Label>
                        <div className="flex flex-wrap gap-2 mt-1">
                            {DOSAGE_UNITS.map((unit) => (
                                <button
                                    key={unit}
                                    type="button"
                                    onClick={() => toggleUnit(unit)}
                                    className={cn(
                                        'px-2 py-1 rounded text-xs border transition-colors',
                                        form.available_units?.includes(unit)
                                            ? 'bg-blue-50 border-blue-300 text-blue-700'
                                            : 'bg-white border-slate-200 text-slate-500 hover:border-slate-300'
                                    )}
                                >
                                    {unit}
                                </button>
                            ))}
                        </div>
                    </div>
                    <div>
                        <Label>排序（數字越小越前面）</Label>
                        <Input
                            type="number"
                            value={form.sort_order || 0}
                            onChange={(e) => setForm((prev) => ({ ...prev, sort_order: parseInt(e.target.value) || 0 }))}
                        />
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        取消
                    </Button>
                    <Button onClick={onSubmit} disabled={isLoading}>
                        {isLoading && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        儲存
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}

// ============================================
// ERP 匯入 Dialog
// ============================================

function ErpImportDialog({
    open,
    onOpenChange,
}: {
    open: boolean
    onOpenChange: (open: boolean) => void
}) {
    const queryClient = useQueryClient()
    const [searchText, setSearchText] = useState('')
    const selection = useSelection<string>()
    const [importCategory, setImportCategory] = useState('其他')

    // 搜尋 ERP 產品
    const { data: products = [], isLoading: productsLoading } = useQuery({
        queryKey: ['erp-products-for-import', searchText],
        queryFn: async () => {
            const res = await api.get<Product[]>('/products', {
                params: { keyword: searchText },
            })
            return res.data
        },
        enabled: open && searchText.length > 0,
    })

    const importMutation = useMutation({
        mutationFn: () =>
            treatmentDrugApi.importFromErp({
                product_ids: Array.from(selection.selectedIds),
                category: importCategory,
            }),
        onSuccess: (res) => {
            queryClient.invalidateQueries({ queryKey: ['admin-treatment-drugs'] })
            queryClient.invalidateQueries({ queryKey: ['treatment-drugs'] })
            toast({
                title: '匯入成功',
                description: `已匯入 ${res.data.length} 個藥物選項`,
            })
            selection.clear()
            onOpenChange(false)
        },
        onError: (err: unknown) => {
            toast({
                title: '匯入失敗',
                description: getApiErrorMessage(err, '匯入失敗'),
                variant: 'destructive',
            })
        },
    })

    const toggleSelect = (id: string) => {
        selection.toggle(id)
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-lg">
                <DialogHeader>
                    <DialogTitle>從 ERP 匯入藥物</DialogTitle>
                    <DialogDescription>搜尋 ERP 產品並匯入為藥物選項</DialogDescription>
                </DialogHeader>
                <div className="space-y-4">
                    <div className="flex gap-2">
                        <div className="relative flex-1">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-400" />
                            <Input
                                placeholder="搜尋 ERP 產品名稱..."
                                value={searchText}
                                onChange={(e) => setSearchText(e.target.value)}
                                className="pl-9"
                            />
                        </div>
                        <Select value={importCategory} onValueChange={setImportCategory}>
                            <SelectTrigger className="w-[120px]">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                {DRUG_CATEGORIES.map((cat) => (
                                    <SelectItem key={cat} value={cat}>{cat}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>

                    <div className="max-h-60 overflow-auto border rounded-md">
                        {productsLoading ? (
                            <div className="flex items-center justify-center py-8">
                                <Loader2 className="h-5 w-5 animate-spin text-blue-500" />
                            </div>
                        ) : products.length === 0 ? (
                            <div className="text-center py-8 text-slate-400 text-sm">
                                {searchText ? '無符合的產品' : '請輸入關鍵字搜尋'}
                            </div>
                        ) : (
                            <div className="divide-y">
                                {products.map((product) => (
                                    <label
                                        key={product.id}
                                        className={cn(
                                            'flex items-center gap-3 px-4 py-3 cursor-pointer hover:bg-slate-50 transition-colors',
                                            selection.has(product.id) && 'bg-blue-50'
                                        )}
                                    >
                                        <input
                                            type="checkbox"
                                            checked={selection.has(product.id)}
                                            onChange={() => toggleSelect(product.id)}
                                            className="rounded border-slate-300"
                                        />
                                        <div className="flex-1 min-w-0">
                                            <div className="text-sm font-medium text-slate-700 truncate">
                                                {product.name}
                                            </div>
                                            <div className="text-xs text-slate-400">
                                                {product.sku} · {product.base_uom}
                                                {product.spec && ` · ${product.spec}`}
                                            </div>
                                        </div>
                                    </label>
                                ))}
                            </div>
                        )}
                    </div>

                    {selection.size > 0 && (
                        <p className="text-sm text-blue-600">
                            已選擇 {selection.size} 個產品
                        </p>
                    )}
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        取消
                    </Button>
                    <Button
                        onClick={() => importMutation.mutate()}
                        disabled={selection.size === 0 || importMutation.isPending}
                    >
                        {importMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        匯入 ({selection.size})
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
