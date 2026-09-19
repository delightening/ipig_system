import { useState } from 'react'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api, {
    deleteResource,
    Warehouse,
} from '@/lib/api'
import { getApiErrorMessage } from '@/lib/apiError'
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
    Archive,
    Loader2,
    Plus,
    Warehouse as WarehouseIcon,
    Trash2,
    Upload,
    Download,
    Edit3,
    Printer,
} from 'lucide-react'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { WarehouseInactiveDialog } from './WarehouseInactiveDialog'

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
    exclude_from_alerts: boolean
    skip_routine_stocktake: boolean
    is_default_issue_source: boolean
}

const initialFormData: WarehouseFormData = {
    code: '',
    name: '',
    address: '',
    is_active: true,
    // 新倉庫預設「照常盤點、照常警報」——與 migration 014 的欄位預設一致。
    // 例外要由人明確勾選，不能靠預設值悄悄生效。
    exclude_from_alerts: false,
    skip_routine_stocktake: false,
    // 同理（migration 015）：新倉庫預設不是領用來源。勾錯的代價是開單時
    // 自動帶到不該領貨的地點，所以寧可預設不帶、由人明確指定。
    is_default_issue_source: false,
}

export function WarehouseActionHeader({
    selectedWarehouseId,
    onWarehouseChange,
    onImportClick,
    onExportClick,
}: WarehouseActionHeaderProps) {
    const { t } = useTranslation()
    const navigate = useNavigate()
    const queryClient = useQueryClient()
    const { dialogState, confirm } = useConfirmDialog()
    const [showWarehouseDialog, setShowWarehouseDialog] = useState(false)
    const [showInactiveDialog, setShowInactiveDialog] = useState(false)
    const [editingWarehouse, setEditingWarehouse] = useState<Warehouse | null>(null)
    const [formData, setFormData] = useState<WarehouseFormData>(initialFormData)

    // 取得所有倉庫。GET /warehouses 未帶 is_active 時後端預設只回啟用中的
    // （services/warehouse.rs `list`），停用的必須另外用 ?is_active=false 撈——
    // 少了這一支，誤停用的倉庫在 UI 上完全消失、無從復原（2026-08-05 事件）。
    const { data: allWarehouses, isLoading: loadingWarehouses } = useQuery({
        queryKey: ['all-warehouses'],
        queryFn: async () => {
            // 停用清單是輔助資訊，用 allSettled 而非 all：它掛掉不該讓整個 query 失敗，
            // 否則選擇器一個倉庫都列不出來、匯出鈕也一起被停用。
            const [active, inactive] = await Promise.allSettled([
                api.get<Warehouse[]>('/warehouses'),
                api.get<Warehouse[]>('/warehouses?is_active=false'),
            ])
            if (active.status === 'rejected') throw active.reason
            return [
                ...active.value.data,
                ...(inactive.status === 'fulfilled' ? inactive.value.data : []),
            ]
        },
    })

    // 過濾出啟用的倉庫供選擇器使用
    const activeWarehouses = allWarehouses?.filter(w => w.is_active) || []
    const inactiveWarehouses = allWarehouses?.filter(w => !w.is_active) || []

    // 建立倉庫
    const createMutation = useMutation({
        mutationFn: async (data: WarehouseFormData) => {
            return api.post('/warehouses', data)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['all-warehouses'] })
            queryClient.invalidateQueries({ queryKey: ['warehouses'] })
            toast({ title: t('common.success'), description: t('erpDocs.warehouse.header.created') })
            setShowWarehouseDialog(false)
        },
        onError: (error: Error) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('erpDocs.warehouse.header.createFailed')),
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
            toast({ title: t('common.success'), description: t('erpDocs.warehouse.header.updated') })
            setShowWarehouseDialog(false)
        },
        onError: (error: Error) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('erpDocs.warehouse.header.updateFailed')),
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
            toast({ title: t('common.success'), description: t('erpDocs.warehouse.header.deactivated') })
            setShowWarehouseDialog(false)
        },
        onError: (error: Error) => {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(error, t('erpDocs.shared.deleteFailed')),
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
            exclude_from_alerts: warehouse.exclude_from_alerts,
            skip_routine_stocktake: warehouse.skip_routine_stocktake,
            is_default_issue_source: warehouse.is_default_issue_source,
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
        // 刪除是軟刪除（is_active = false），原文案寫「無法復原」是錯的；
        // 現在有「已停用倉庫」入口可復原，文案照實描述。
        const ok = await confirm({
            title: t('erpDocs.warehouse.header.deactivateTitle'),
            description: t('erpDocs.warehouse.header.deactivateDescription', { name: editingWarehouse.name }),
            variant: 'destructive',
            confirmLabel: t('erpDocs.warehouse.header.deactivateLabel'),
        })
        if (ok) {
            deleteMutation.mutate(editingWarehouse.id)
        }
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">{t('nav.erpWarehouses')}</h1>
                    <p className="text-muted-foreground">{t('erpDocs.warehouse.header.subtitle')}</p>
                </div>
                <div className="flex flex-wrap gap-2 justify-end">
                    <Can permission={PERMISSIONS.ERP_WAREHOUSE_CREATE}>
                      <Button variant="outline" size="sm" onClick={onImportClick}>
                          <Upload className="mr-2 h-4 w-4" />
                          {t('erpDocs.warehouse.importWarehouses')}
                      </Button>
                    </Can>
                    <Can permission={PERMISSIONS.ERP_WAREHOUSE_VIEW}>
                      <Button variant="outline" size="sm" onClick={onExportClick} disabled={activeWarehouses.length === 0}>
                          <Download className="mr-2 h-4 w-4" />
                          {t('erpDocs.warehouse.header.exportWarehouses')}
                      </Button>
                    </Can>
                    <Can permission={PERMISSIONS.ERP_WAREHOUSE_VIEW}>
                      <Button
                          variant="outline"
                          size="sm"
                          onClick={() => navigate(`/inventory/warehouse-report/${selectedWarehouseId}`)}
                          disabled={!selectedWarehouseId}
                      >
                          <Printer className="mr-2 h-4 w-4" />
                          {t('erpDocs.warehouse.header.printStatus')}
                      </Button>
                    </Can>
                </div>
            </div>

            <Card>
                <CardHeader className="pb-3 flex-row items-center justify-between space-y-0">
                    <CardTitle className="text-base flex items-center gap-2">
                        <WarehouseIcon className="h-4 w-4" />
                        {t('erpDocs.shared.selectWarehouse')}
                    </CardTitle>
                    <div className="flex gap-2">
                    <Can permission={PERMISSIONS.ERP_WAREHOUSE_EDIT}>
                          {inactiveWarehouses.length > 0 && (
                              <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => setShowInactiveDialog(true)}
                              >
                                  <Archive className="h-4 w-4 mr-1" />
                                  {t('erpDocs.warehouse.header.inactiveWarehouses')}
                                  <span className="ml-1.5 inline-flex h-[18px] min-w-[18px] items-center justify-center rounded-full bg-destructive px-1.5 text-[11px] font-semibold text-destructive-foreground">
                                      {inactiveWarehouses.length}
                                  </span>
                              </Button>
                          )}
                    </Can>
                        <Can permission={PERMISSIONS.ERP_WAREHOUSE_CREATE}>
                          <Button variant="ghost" size="sm" onClick={handleOpenCreate}>
                              <Plus className="h-4 w-4 mr-1" />
                              {t('erpDocs.warehouse.header.addWarehouse')}
                          </Button>
                        </Can>
                        <Can permission={PERMISSIONS.ERP_WAREHOUSE_EDIT}>
                          <Button 
                              variant="ghost" 
                              size="sm" 
                              onClick={handleOpenEdit}
                              disabled={!selectedWarehouseId}
                          >
                              <Edit3 className="h-4 w-4 mr-1" />
                              {t('erpDocs.warehouse.header.editWarehouse')}
                          </Button>
                        </Can>
                    </div>
                </CardHeader>
                <CardContent>
                    <Select
                        value={selectedWarehouseId}
                        onValueChange={onWarehouseChange}
                    >
                        <SelectTrigger className="w-[300px]">
                            <SelectValue placeholder={t('erpDocs.warehouse.header.selectPlaceholder')} />
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
                        <DialogTitle>{editingWarehouse ? t('erpDocs.warehouse.header.editWarehouse') : t('erpDocs.warehouse.header.addWarehouse')}</DialogTitle>
                        <DialogDescription>
                            {t('erpDocs.warehouse.header.dialogDescription')}
                        </DialogDescription>
                    </DialogHeader>
                    <form onSubmit={handleSubmit}>
                        <div className="grid gap-4 py-4">
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="code" className="text-right">{t('erpDocs.warehouse.header.codeRequired')}</Label>
                                <Input
                                    id="code"
                                    value={formData.code}
                                    onChange={(e) => setFormData({ ...formData, code: e.target.value })}
                                    className="col-span-3"
                                    placeholder={t('erpDocs.warehouse.header.codePlaceholder')}
                                    required
                                    disabled={!!editingWarehouse}
                                />
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="wh-name" className="text-right">{t('erpDocs.shared.nameRequired')}</Label>
                                <Input
                                    id="wh-name"
                                    value={formData.name}
                                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                                    className="col-span-3"
                                    placeholder={t('erpDocs.warehouse.header.namePlaceholder')}
                                    required
                                />
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="address" className="text-right">{t('erpDocs.shared.address')}</Label>
                                <Input
                                    id="address"
                                    value={formData.address}
                                    onChange={(e) => setFormData({ ...formData, address: e.target.value })}
                                    className="col-span-3"
                                />
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4">
                                <Label htmlFor="is_active" className="text-right">{t('erpDocs.warehouse.header.activeStatus')}</Label>
                                <div className="col-span-3 flex items-center gap-2">
                                    <Switch
                                        id="is_active"
                                        checked={formData.is_active}
                                        onCheckedChange={(checked) => setFormData({ ...formData, is_active: checked })}
                                    />
                                    <span className="text-sm text-muted-foreground">
                                        {formData.is_active ? t('erpDocs.warehouse.header.enabled') : t('erpDocs.warehouse.header.disabled')}
                                    </span>
                                </div>
                            </div>
                            {/* 盤點與警報政策（migration 014）。
                              * 這兩項刻意分開而不是一個「倉庫類型」下拉：儲藏室是「有庫存但不維護
                              * 帳面準確度」，廢棄物處理區是「根本不是庫存資產」——目前兩者的勾選
                              * 剛好一樣，但語意不同，合併後日後出現「要盤但不要警報」的倉庫就回不去了。 */}
                            <div className="grid grid-cols-4 items-start gap-4">
                                <Label htmlFor="exclude_from_alerts" className="text-right pt-2">{t('erpDocs.warehouse.header.lowStockAlert')}</Label>
                                <div className="col-span-3 flex items-start gap-2">
                                    <Switch
                                        id="exclude_from_alerts"
                                        checked={!formData.exclude_from_alerts}
                                        onCheckedChange={(checked) => setFormData({ ...formData, exclude_from_alerts: !checked })}
                                    />
                                    <span className="text-sm text-muted-foreground">
                                        {formData.exclude_from_alerts
                                            ? t('erpDocs.warehouse.header.alertOff')
                                            : t('erpDocs.warehouse.header.alertOn')}
                                    </span>
                                </div>
                            </div>
                            <div className="grid grid-cols-4 items-start gap-4">
                                <Label htmlFor="skip_routine_stocktake" className="text-right pt-2">{t('erpDocs.warehouse.header.routineStocktake')}</Label>
                                <div className="col-span-3 flex items-start gap-2">
                                    <Switch
                                        id="skip_routine_stocktake"
                                        checked={!formData.skip_routine_stocktake}
                                        onCheckedChange={(checked) => setFormData({ ...formData, skip_routine_stocktake: !checked })}
                                    />
                                    <span className="text-sm text-muted-foreground">
                                        {formData.skip_routine_stocktake
                                            ? t('erpDocs.warehouse.header.stocktakeOff')
                                            : t('erpDocs.warehouse.header.stocktakeOn')}
                                    </span>
                                </div>
                            </div>
                            {/* 領用來源（migration 015）。與上面兩項不同，這個開關是「正向」的
                              * ——勾起來才生效，因為它會主動改變開單時的預設值。
                              * 可以多個倉庫同時勾：藥品在準備室、耗材在儲藏室都是正當領用點，
                              * 後端再依該品項實際哪裡有貨去挑，故沒有唯一性約束。 */}
                            <div className="grid grid-cols-4 items-start gap-4">
                                <Label htmlFor="is_default_issue_source" className="text-right pt-2">{t('erpDocs.warehouse.header.issueSource')}</Label>
                                <div className="col-span-3 flex items-start gap-2">
                                    <Switch
                                        id="is_default_issue_source"
                                        checked={formData.is_default_issue_source}
                                        onCheckedChange={(checked) => setFormData({ ...formData, is_default_issue_source: checked })}
                                    />
                                    <span className="text-sm text-muted-foreground">
                                        {formData.is_default_issue_source
                                            ? t('erpDocs.warehouse.header.issueSourceOn')
                                            : t('erpDocs.warehouse.header.issueSourceOff')}
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
                                        {t('common.delete')}
                                    </Button>
                                )}
                            </div>
                            <div className="flex gap-2">
                                <Button type="button" variant="outline" onClick={() => setShowWarehouseDialog(false)}>
                                    {t('common.cancel')}
                                </Button>
                                <Button
                                    type="submit"
                                    disabled={createMutation.isPending || updateMutation.isPending}
                                >
                                    {(createMutation.isPending || updateMutation.isPending) && (
                                        <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                                    )}
                                    {editingWarehouse ? t('common.update') : t('erpDocs.warehouse.header.createButton')}
                                </Button>
                            </div>
                        </DialogFooter>
                    </form>
                </DialogContent>
            </Dialog>
            <WarehouseInactiveDialog
                open={showInactiveDialog}
                onOpenChange={setShowInactiveDialog}
                warehouses={inactiveWarehouses}
            />
            <ConfirmDialog state={dialogState} />
        </div>
    )
}
