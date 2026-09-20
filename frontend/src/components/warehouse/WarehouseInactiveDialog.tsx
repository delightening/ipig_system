import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Archive, Loader2, RotateCcw } from 'lucide-react'

import api, { Warehouse } from '@/lib/api'
import { getApiErrorMessage } from '@/lib/apiError'
import { formatDateTime } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'

interface WarehouseInactiveDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    /** 已停用的倉庫（父層以 `?is_active=false` 取得） */
    warehouses: Warehouse[]
}

/**
 * 已停用倉庫的檢視與復原。
 *
 * 倉庫刪除是軟刪除（`is_active = false`），但倉庫清單、庫存查詢樹與現況報表
 * 都只撈啟用中的倉庫 —— 誤停用後在 UI 上再也選不到它，等於無法自救
 * （2026-08-05「儲藏室 (2)」事件）。此對話框是唯一的復原入口。
 */
export function WarehouseInactiveDialog({
    open,
    onOpenChange,
    warehouses,
}: WarehouseInactiveDialogProps) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()

    const restoreMutation = useMutation({
        mutationFn: async (id: string) => {
            return api.put(`/warehouses/${id}`, { is_active: true })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['all-warehouses'] })
            queryClient.invalidateQueries({ queryKey: ['warehouses'] })
            toast({ title: t('common.success'), description: t('erpDocs.warehouse.inactive.restored') })
        },
        onError: (error: Error) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('erpDocs.warehouse.inactive.restoreFailed')),
                variant: 'destructive',
            })
        },
    })

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <Archive className="h-4 w-4" />
                        {t('erpDocs.warehouse.inactive.title')}
                    </DialogTitle>
                    <DialogDescription>
                        {t('erpDocs.warehouse.inactive.description')}
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-2 py-2">
                    {warehouses.length === 0 ? (
                        <p className="text-sm text-muted-foreground py-4 text-center">
                            {t('erpDocs.warehouse.inactive.empty')}
                        </p>
                    ) : (
                        warehouses.map((w) => (
                            <div
                                key={w.id}
                                className="flex items-center justify-between gap-3 rounded-md border p-3"
                            >
                                <div className="flex flex-col">
                                    <span className="text-sm font-medium">
                                        {w.code} - {w.name}
                                    </span>
                                    {/* warehouses 沒有 deactivated_at 欄位；updated_at 是「最後一次
                                        任何更新」，停用後再改名稱也會動到它，故不寫成「停用於」 */}
                                    <span className="text-xs text-muted-foreground">
                                        {t('erpDocs.warehouse.inactive.lastUpdated', { time: formatDateTime(w.updated_at) })}
                                    </span>
                                </div>
                                <Button
                                    size="sm"
                                    onClick={() => restoreMutation.mutate(w.id)}
                                    disabled={restoreMutation.isPending}
                                >
                                    {/* 只讓正在復原的那一列轉圈：isPending 是共用的，
                                        不比對 variables 會整份清單一起顯示 spinner */}
                                    {restoreMutation.isPending && restoreMutation.variables === w.id ? (
                                        <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                                    ) : (
                                        <RotateCcw className="h-4 w-4 mr-1" />
                                    )}
                                    {t('erpDocs.warehouse.inactive.restore')}
                                </Button>
                            </div>
                        ))
                    )}
                </div>

                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        {t('common.closeDialog')}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
