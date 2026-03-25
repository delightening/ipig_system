import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { treatmentDrugApi } from '@/lib/api'
import api from '@/lib/api'
import { useSelection } from '@/hooks/useSelection'
import { DRUG_CATEGORIES } from '@/types/treatment-drug'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
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
import { Search, Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'

/** ERP 產品型別 */
interface Product {
    id: string
    name: string
    sku: string
    base_uom: string
    spec: string | null
    is_active: boolean
}

interface ErpImportDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
}

export function ErpImportDialog({ open, onOpenChange }: ErpImportDialogProps) {
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
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
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

                    <ProductList
                        products={products}
                        isLoading={productsLoading}
                        searchText={searchText}
                        selection={selection}
                    />

                    {selection.size > 0 && (
                        <p className="text-sm text-primary">
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

function ProductList({
    products,
    isLoading,
    searchText,
    selection,
}: {
    products: Product[]
    isLoading: boolean
    searchText: string
    selection: ReturnType<typeof useSelection<string>>
}) {
    if (isLoading) {
        return (
            <div className="max-h-60 overflow-auto border rounded-md">
                <div className="flex items-center justify-center py-8">
                    <Loader2 className="h-5 w-5 animate-spin text-primary" />
                </div>
            </div>
        )
    }

    if (products.length === 0) {
        return (
            <div className="max-h-60 overflow-auto border rounded-md">
                <div className="text-center py-8 text-muted-foreground text-sm">
                    {searchText ? '無符合的產品' : '請輸入關鍵字搜尋'}
                </div>
            </div>
        )
    }

    return (
        <div className="max-h-60 overflow-auto border rounded-md">
            <div className="divide-y">
                {products.map((product) => (
                    <label
                        key={product.id}
                        className={cn(
                            'flex items-center gap-3 px-4 py-3 cursor-pointer hover:bg-muted transition-colors',
                            selection.has(product.id) && 'bg-primary/10'
                        )}
                    >
                        <input
                            type="checkbox"
                            checked={selection.has(product.id)}
                            onChange={() => selection.toggle(product.id)}
                            className="rounded border-border"
                        />
                        <div className="flex-1 min-w-0">
                            <div className="text-sm font-medium text-foreground truncate">
                                {product.name}
                            </div>
                            <div className="text-xs text-muted-foreground">
                                {product.sku} · {product.base_uom}
                                {product.spec && ` · ${product.spec}`}
                            </div>
                        </div>
                    </label>
                ))}
            </div>
        </div>
    )
}
