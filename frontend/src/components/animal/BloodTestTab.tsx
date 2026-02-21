/**
 * 血液檢查分頁元件
 * 顯示在動物詳情頁 (AnimalDetailPage) 中
 */
import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
    BloodTestListItem,
    BloodTestTemplate,
    AnimalBloodTestWithItems,
    BloodTestItemInput,
    CreateBloodTestRequest,
    UpdateBloodTestRequest,
    bloodTestApi,
    bloodTestTemplateApi,
    bloodTestPanelApi,
    BloodTestPanel,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'
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
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'

import { toast } from '@/components/ui/use-toast'
import { PanelIcon } from '@/components/ui/panel-icon'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'
import {
    Loader2,
    Plus,
    Eye,
    Edit2,
    Trash2,
    AlertCircle,
    FileText,
    X,
} from 'lucide-react'

interface BloodTestTabProps {
    animalId: string
    /** 資料隔離 query string，例如 '?after=2024-01-01T00:00:00Z' */
    afterParam?: string
}



// 預設檢驗機構選項
const LAB_OPTIONS = ['為恭醫院', '里仁動物醫院'] as const

export function BloodTestTab({ animalId, afterParam = '' }: BloodTestTabProps) {
    const queryClient = useQueryClient()
    const [showFormDialog, setShowFormDialog] = useState(false)
    const [showDetailDialog, setShowDetailDialog] = useState(false)
    const [editingId, setEditingId] = useState<string | null>(null)
    const [viewingId, setViewingId] = useState<string | null>(null)
    const [deleteTarget, setDeleteTarget] = useState<{ id: string; date: string } | null>(null)
    // 檢驗機構下拉選項狀態：'為恭醫院' | '里仁動物醫院' | '__other__'
    const [labNameOption, setLabNameOption] = useState<string>('')

    // 表單資料
    const [formData, setFormData] = useState<{
        test_date: string
        lab_name: string
        remark: string
        items: BloodTestItemInput[]
    }>({
        test_date: new Date().toISOString().split('T')[0],
        lab_name: '',
        remark: '',
        items: [],
    })

    // 載入血液檢查列表
    const { data: bloodTests = [], isLoading } = useQuery({
        queryKey: ['animal-blood-tests', animalId, afterParam],
        queryFn: async () => {
            const res = await api.get<BloodTestListItem[]>(`/animals/${animalId}/blood-tests${afterParam}`)
            return res.data
        },
    })

    // 載入模板列表
    const { data: templates = [] } = useQuery({
        queryKey: ['blood-test-templates'],
        queryFn: async () => {
            const res = await bloodTestTemplateApi.list()
            return res.data
        },
    })

    // 載入組合列表
    const { data: panels = [] } = useQuery({
        queryKey: ['blood-test-panels'],
        queryFn: async () => {
            const res = await bloodTestPanelApi.list()
            return res.data
        },
    })

    // 計算每個 panel 是否完全被選取（所有 items 都在 formData.items 中）
    const panelActiveStates = useMemo(() => {
        const itemTemplateIds = new Set(formData.items.map(i => i.template_id).filter(Boolean))
        return panels.reduce((acc, panel) => {
            if (panel.items.length === 0) {
                acc[panel.id] = false
            } else {
                acc[panel.id] = panel.items.every(t => itemTemplateIds.has(t.id))
            }
            return acc
        }, {} as Record<string, boolean>)
    }, [panels, formData.items])

    // 載入單筆詳情
    const { data: viewDetail } = useQuery({
        queryKey: ['blood-test-detail', viewingId],
        queryFn: async () => {
            if (!viewingId) return null
            const res = await bloodTestApi.getById(viewingId)
            return res.data
        },
        enabled: !!viewingId,
    })

    // 建立 mutation
    const createMutation = useMutation({
        mutationFn: (data: CreateBloodTestRequest) => bloodTestApi.create(animalId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-blood-tests', animalId] })
            setShowFormDialog(false)
            resetForm()
            toast({ title: '成功', description: '血液檢查紀錄已建立' })
        },
        onError: () => {
            toast({ title: '錯誤', description: '建立失敗', variant: 'destructive' })
        },
    })

    // 更新 mutation
    const updateMutation = useMutation({
        mutationFn: ({ id, data }: { id: string; data: UpdateBloodTestRequest }) =>
            bloodTestApi.update(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-blood-tests', animalId] })
            queryClient.invalidateQueries({ queryKey: ['blood-test-detail'] })
            setShowFormDialog(false)
            setEditingId(null)
            resetForm()
            toast({ title: '成功', description: '血液檢查紀錄已更新' })
        },
        onError: () => {
            toast({ title: '錯誤', description: '更新失敗', variant: 'destructive' })
        },
    })

    // 刪除 mutation
    const deleteMutation = useMutation({
        mutationFn: ({ id, reason }: { id: string; reason: string }) =>
            bloodTestApi.delete(id, reason),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-blood-tests', animalId] })
            setDeleteTarget(null)
            toast({ title: '成功', description: '血液檢查紀錄已刪除' })
        },
        onError: () => {
            toast({ title: '錯誤', description: '刪除失敗', variant: 'destructive' })
        },
    })

    const resetForm = () => {
        setFormData({
            test_date: new Date().toISOString().split('T')[0],
            lab_name: '',
            remark: '',
            items: [],
        })
        setLabNameOption('')
    }

    /** 開啟新增表單 */
    const openCreateForm = () => {
        resetForm()
        setEditingId(null)
        setShowFormDialog(true)
    }

    /** 開啟編輯表單 */
    const openEditForm = async (id: string) => {
        try {
            const res = await bloodTestApi.getById(id)
            const detail = res.data
            setFormData({
                test_date: detail.blood_test.test_date,
                lab_name: detail.blood_test.lab_name || '',
                remark: detail.blood_test.remark || '',
                items: detail.items.map((item) => ({

                    template_id: item.template_id || undefined,
                    item_name: item.item_name,
                    result_value: item.result_value || '',
                    result_unit: item.result_unit || '',
                    reference_range: item.reference_range || '',
                    is_abnormal: item.is_abnormal,
                    remark: item.remark || '',
                    sort_order: item.sort_order,
                })),
            })
            // 判斷 lab_name 是否為預設選項
            const loadedLabName = detail.blood_test.lab_name || ''
            if (LAB_OPTIONS.includes(loadedLabName as typeof LAB_OPTIONS[number])) {
                setLabNameOption(loadedLabName)
            } else if (loadedLabName) {
                setLabNameOption('__other__')
            } else {
                setLabNameOption('')
            }
            setEditingId(id)
            setShowFormDialog(true)
        } catch {
            toast({ title: '錯誤', description: '載入資料失敗', variant: 'destructive' })
        }
    }

    /** 從模板新增檢查項目 */
    const addItemFromTemplate = (templateId: string) => {
        const template = templates.find((t) => t.id === templateId)
        if (!template) return

        // 檢查是否已存在
        const exists = formData.items.some((item) => item.template_id === templateId)
        if (exists) {
            toast({ title: '提示', description: '該項目已在清單中' })
            return
        }

        setFormData((prev) => ({
            ...prev,
            items: [
                ...prev.items,
                {
                    template_id: templateId,
                    item_name: template.name,
                    result_value: '',
                    result_unit: template.default_unit || '',
                    reference_range: template.reference_range || '',
                    is_abnormal: false,
                    remark: '',
                    sort_order: prev.items.length,
                },
            ],
        }))
    }

    /** 新增自訂項目 */
    const addCustomItem = () => {
        setFormData((prev) => ({
            ...prev,
            items: [
                ...prev.items,
                {
                    item_name: '',
                    result_value: '',
                    result_unit: '',
                    reference_range: '',
                    is_abnormal: false,
                    remark: '',
                    sort_order: prev.items.length,
                },
            ],
        }))
    }

    /** 切換組合（Toggle Panel）*/
    const togglePanel = (panel: BloodTestPanel) => {
        const isActive = panelActiveStates[panel.id]
        if (isActive) {
            // 移除該組合的所有項目
            const panelTemplateIds = new Set(panel.items.map(t => t.id))
            setFormData((prev) => ({
                ...prev,
                items: prev.items.filter(item => !item.template_id || !panelTemplateIds.has(item.template_id)),
            }))
        } else {
            // 加入該組合的項目（跳過已存在的）
            const existingIds = new Set(formData.items.map(i => i.template_id).filter(Boolean))
            const newItems = panel.items
                .filter(t => !existingIds.has(t.id))
                .map((t, idx) => ({
                    template_id: t.id,
                    item_name: t.name,
                    result_value: '',
                    result_unit: t.default_unit || '',
                    reference_range: t.reference_range || '',
                    is_abnormal: false,
                    remark: '',
                    sort_order: formData.items.length + idx,
                }))
            if (newItems.length > 0) {
                setFormData((prev) => ({
                    ...prev,
                    items: [...prev.items, ...newItems],
                }))
                toast({ title: '已加入', description: `${panel.name}：新增 ${newItems.length} 項` })
            } else {
                toast({ title: '提示', description: '所有項目已在清單中' })
            }
        }
    }

    /** 移除項目 */
    const removeItem = (index: number) => {
        setFormData((prev) => ({
            ...prev,
            items: prev.items.filter((_, i) => i !== index),
        }))
    }

    /** 更新項目欄位 */
    const updateItem = (index: number, field: keyof BloodTestItemInput, value: unknown) => {
        setFormData((prev) => ({
            ...prev,
            items: prev.items.map((item, i) =>
                i === index ? { ...item, [field]: value } : item
            ),
        }))
    }

    /** 提交表單 */
    const handleSubmit = () => {
        if (!formData.test_date) {
            toast({ title: '錯誤', description: '請填寫檢查日期', variant: 'destructive' })
            return
        }
        if (formData.items.length === 0) {
            toast({ title: '錯誤', description: '至少需要一個檢查項目', variant: 'destructive' })
            return
        }
        // 檢查是否所有項目都有名稱
        const emptyName = formData.items.some((item) => !item.item_name.trim())
        if (emptyName) {
            toast({ title: '錯誤', description: '所有檢查項目都需要填寫名稱', variant: 'destructive' })
            return
        }

        if (editingId) {
            updateMutation.mutate({
                id: editingId,
                data: {
                    test_date: formData.test_date,
                    lab_name: formData.lab_name || undefined,
                    remark: formData.remark || undefined,
                    items: formData.items,
                },
            })
        } else {
            createMutation.mutate({
                test_date: formData.test_date,
                lab_name: formData.lab_name || undefined,
                remark: formData.remark || undefined,
                items: formData.items,
            })
        }
    }

    const isPending = createMutation.isPending || updateMutation.isPending

    return (
        <div className="space-y-4">
            {/* 標題列 */}
            <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold flex items-center gap-2">
                    <FileText className="h-5 w-5 text-red-600" />
                    血液檢查紀錄
                </h3>
                <Button className="bg-red-600 hover:bg-red-700" onClick={openCreateForm}>
                    <Plus className="h-4 w-4 mr-2" />
                    新增血液檢查
                </Button>
            </div>

            {/* 列表 */}
            {isLoading ? (
                <div className="flex justify-center py-8">
                    <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
                </div>
            ) : bloodTests.length === 0 ? (
                <Card>
                    <CardContent className="py-12 text-center text-gray-500">
                        <FileText className="h-12 w-12 mx-auto mb-3 text-gray-300" />
                        <p>尚無血液檢查紀錄</p>
                        <p className="text-sm mt-1">點擊上方按鈕新增</p>
                    </CardContent>
                </Card>
            ) : (
                <Card>
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>檢查日期</TableHead>
                                <TableHead>檢驗機構</TableHead>
                                <TableHead className="text-center">項目數</TableHead>
                                <TableHead className="text-center">異常項目</TableHead>
                                <TableHead>建立者</TableHead>
                                <TableHead className="text-right">操作</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {bloodTests.map((test) => (
                                <TableRow key={test.id}>
                                    <TableCell className="font-medium">{test.test_date}</TableCell>
                                    <TableCell>{test.lab_name || '-'}</TableCell>
                                    <TableCell className="text-center">{test.item_count}</TableCell>
                                    <TableCell className="text-center">
                                        {test.abnormal_count > 0 ? (
                                            <Badge variant="destructive" className="gap-1">
                                                <AlertCircle className="h-3 w-3" />
                                                {test.abnormal_count}
                                            </Badge>
                                        ) : (
                                            <span className="text-green-600">0</span>
                                        )}
                                    </TableCell>
                                    <TableCell>{test.created_by_name || '-'}</TableCell>
                                    <TableCell className="text-right space-x-1">
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            onClick={() => {
                                                setViewingId(test.id)
                                                setShowDetailDialog(true)
                                            }}
                                        >
                                            <Eye className="h-4 w-4" />
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            onClick={() => openEditForm(test.id)}
                                        >
                                            <Edit2 className="h-4 w-4" />
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            className="text-red-600"
                                            onClick={() => setDeleteTarget({ id: test.id, date: test.test_date })}
                                        >
                                            <Trash2 className="h-4 w-4" />
                                        </Button>
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </Card>
            )}

            {/* 新增/編輯 Dialog */}
            <Dialog open={showFormDialog} onOpenChange={(open) => {
                if (!open) {
                    setShowFormDialog(false)
                    setEditingId(null)
                    resetForm()
                }
            }}>
                <DialogContent className="max-w-4xl">
                    <DialogHeader>
                        <DialogTitle>
                            {editingId ? '編輯血液檢查' : '新增血液檢查'}
                        </DialogTitle>
                        <DialogDescription>
                            填寫檢查基本資訊，並從模板選取或自訂檢查項目
                        </DialogDescription>
                    </DialogHeader>

                    <div className="space-y-6">
                        {/* 基本資訊 */}
                        <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-2">
                                <Label>檢查日期 *</Label>
                                <Input
                                    type="date"
                                    value={formData.test_date}
                                    onChange={(e) => setFormData((prev) => ({ ...prev, test_date: e.target.value }))}
                                />
                            </div>
                            <div className="space-y-2">
                                <Label>檢驗機構</Label>
                                <Select
                                    value={labNameOption}
                                    onValueChange={(val) => {
                                        setLabNameOption(val)
                                        if (val === '__other__') {
                                            setFormData((prev) => ({ ...prev, lab_name: '' }))
                                        } else {
                                            setFormData((prev) => ({ ...prev, lab_name: val }))
                                        }
                                    }}
                                >
                                    <SelectTrigger>
                                        <SelectValue placeholder="請選擇檢驗機構" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {LAB_OPTIONS.map((lab) => (
                                            <SelectItem key={lab} value={lab}>{lab}</SelectItem>
                                        ))}
                                        <SelectItem value="__other__">其他</SelectItem>
                                    </SelectContent>
                                </Select>
                                {labNameOption === '__other__' && (
                                    <Input
                                        value={formData.lab_name}
                                        onChange={(e) => setFormData((prev) => ({ ...prev, lab_name: e.target.value }))}
                                        placeholder="請輸入檢驗機構名稱"
                                        className="mt-2"
                                    />
                                )}
                            </div>
                        </div>



                        <div className="space-y-2">
                            <Label>備註</Label>
                            <Input
                                value={formData.remark}
                                onChange={(e) => setFormData((prev) => ({ ...prev, remark: e.target.value }))}
                                placeholder="選填"
                            />
                        </div>

                        {/* 組合快速選取 */}
                        {panels.length > 0 && (
                            <div className="space-y-2">
                                <Label className="text-sm text-muted-foreground">快速選取組合</Label>
                                <div className="flex flex-wrap gap-2">
                                    {panels.map((panel) => {
                                        const isActive = panelActiveStates[panel.id]
                                        return (
                                            <Button
                                                key={panel.id}
                                                type="button"
                                                variant={isActive ? 'default' : 'outline'}
                                                size="sm"
                                                className={`transition-all ${isActive
                                                    ? 'bg-blue-600 hover:bg-blue-700 text-white shadow-sm'
                                                    : 'hover:bg-blue-50 hover:border-blue-300'
                                                    }`}
                                                onClick={() => togglePanel(panel)}
                                            >
                                                <PanelIcon icon={panel.icon} className="mr-1" />
                                                {panel.name}
                                                {isActive && (
                                                    <span className="ml-1 text-xs">✓</span>
                                                )}
                                            </Button>
                                        )
                                    })}
                                </div>
                            </div>
                        )}

                        {/* 檢查項目 */}
                        <div className="space-y-3">
                            <div className="flex items-center justify-between">
                                <Label className="text-base font-semibold">
                                    檢查項目
                                    {formData.items.length > 0 && (
                                        <span className="ml-2 text-sm font-normal text-muted-foreground">
                                            已選 {formData.items.length} 項
                                        </span>
                                    )}
                                </Label>
                                <div className="flex gap-2">
                                    <Select onValueChange={addItemFromTemplate}>
                                        <SelectTrigger className="w-[200px]">
                                            <SelectValue placeholder="從模板新增..." />
                                        </SelectTrigger>
                                        <SelectContent>
                                            {templates.map((t) => (
                                                <SelectItem key={t.id} value={t.id}>
                                                    {t.code} - {t.name}
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                    <Button variant="outline" size="sm" onClick={addCustomItem}>
                                        <Plus className="h-4 w-4 mr-1" />
                                        自訂項目
                                    </Button>
                                </div>
                            </div>

                            {formData.items.length === 0 ? (
                                <div className="border rounded-lg p-6 text-center text-gray-500">
                                    <p>尚無檢查項目</p>
                                    <p className="text-sm mt-1">從上方模板選取或新增自訂項目</p>
                                </div>
                            ) : (
                                <div className="border rounded-lg overflow-hidden">
                                    <Table>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead className="w-[180px]">項目名稱</TableHead>
                                                <TableHead className="w-[120px]">結果值</TableHead>
                                                <TableHead className="w-[80px]">單位</TableHead>
                                                <TableHead className="w-[120px]">參考範圍</TableHead>
                                                <TableHead className="w-[80px] text-center">異常</TableHead>
                                                <TableHead>備註</TableHead>
                                                <TableHead className="w-[50px]"></TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {formData.items.map((item, index) => (
                                                <TableRow key={index}>
                                                    <TableCell>
                                                        <Input
                                                            value={item.item_name}
                                                            onChange={(e) => updateItem(index, 'item_name', e.target.value)}
                                                            placeholder="項目名稱"
                                                            className="h-8"
                                                            readOnly={!!item.template_id}
                                                        />
                                                    </TableCell>
                                                    <TableCell>
                                                        <Input
                                                            value={item.result_value || ''}
                                                            onChange={(e) => updateItem(index, 'result_value', e.target.value)}
                                                            placeholder="結果"
                                                            className="h-8"
                                                        />
                                                    </TableCell>
                                                    <TableCell>
                                                        <Input
                                                            value={item.result_unit || ''}
                                                            onChange={(e) => updateItem(index, 'result_unit', e.target.value)}
                                                            placeholder="單位"
                                                            className="h-8"
                                                        />
                                                    </TableCell>
                                                    <TableCell>
                                                        <Input
                                                            value={item.reference_range || ''}
                                                            onChange={(e) => updateItem(index, 'reference_range', e.target.value)}
                                                            placeholder="參考範圍"
                                                            className="h-8"
                                                        />
                                                    </TableCell>
                                                    <TableCell className="text-center">
                                                        <input
                                                            type="checkbox"
                                                            checked={item.is_abnormal}
                                                            onChange={(e) => updateItem(index, 'is_abnormal', e.target.checked)}
                                                            className="h-4 w-4 rounded border-gray-300 text-red-600 focus:ring-red-500"
                                                        />
                                                    </TableCell>
                                                    <TableCell>
                                                        <Input
                                                            value={item.remark || ''}
                                                            onChange={(e) => updateItem(index, 'remark', e.target.value)}
                                                            placeholder="備註"
                                                            className="h-8"
                                                        />
                                                    </TableCell>
                                                    <TableCell>
                                                        <Button variant="ghost" size="icon" className="h-8 w-8 text-red-500" onClick={() => removeItem(index)}>
                                                            <X className="h-4 w-4" />
                                                        </Button>
                                                    </TableCell>
                                                </TableRow>
                                            ))}
                                        </TableBody>
                                    </Table>
                                </div>
                            )}
                        </div>
                    </div>

                    <DialogFooter>
                        <Button variant="outline" onClick={() => setShowFormDialog(false)}>
                            取消
                        </Button>
                        <Button onClick={handleSubmit} disabled={isPending}>
                            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                            {editingId ? '更新' : '建立'}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* 詳情 Dialog */}
            <Dialog open={showDetailDialog} onOpenChange={(open) => {
                if (!open) {
                    setShowDetailDialog(false)
                    setViewingId(null)
                }
            }}>
                <DialogContent className="max-w-3xl">
                    <DialogHeader>
                        <DialogTitle>血液檢查詳情</DialogTitle>
                    </DialogHeader>

                    {viewDetail ? (
                        <div className="space-y-4">
                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <Label className="text-gray-500 text-sm">檢查日期</Label>
                                    <p className="font-medium">{viewDetail.blood_test.test_date}</p>
                                </div>
                                <div>
                                    <Label className="text-gray-500 text-sm">檢驗機構</Label>
                                    <p className="font-medium">{viewDetail.blood_test.lab_name || '-'}</p>
                                </div>
                            </div>
                            {viewDetail.blood_test.remark && (
                                <div>
                                    <Label className="text-gray-500 text-sm">備註</Label>
                                    <p>{viewDetail.blood_test.remark}</p>
                                </div>
                            )}
                            <div>
                                <Label className="text-gray-500 text-sm">建立者</Label>
                                <p>{viewDetail.created_by_name || '-'}</p>
                            </div>

                            {/* 檢查項目明細 */}
                            <div>
                                <Label className="text-base font-semibold mb-2 block">
                                    檢查項目 ({viewDetail.items.length})
                                </Label>
                                <Card>
                                    <Table>
                                        <TableHeader>
                                            <TableRow>
                                                <TableHead>項目名稱</TableHead>
                                                <TableHead>結果值</TableHead>
                                                <TableHead>單位</TableHead>
                                                <TableHead>參考範圍</TableHead>
                                                <TableHead className="text-center">狀態</TableHead>
                                                <TableHead>備註</TableHead>
                                            </TableRow>
                                        </TableHeader>
                                        <TableBody>
                                            {viewDetail.items.map((item) => (
                                                <TableRow key={item.id} className={item.is_abnormal ? 'bg-red-50' : ''}>
                                                    <TableCell className="font-medium">{item.item_name}</TableCell>
                                                    <TableCell>{item.result_value || '-'}</TableCell>
                                                    <TableCell>{item.result_unit || '-'}</TableCell>
                                                    <TableCell className="text-sm text-gray-500">{item.reference_range || '-'}</TableCell>
                                                    <TableCell className="text-center">
                                                        {item.is_abnormal ? (
                                                            <Badge variant="destructive" className="gap-1">
                                                                <AlertCircle className="h-3 w-3" />
                                                                異常
                                                            </Badge>
                                                        ) : (
                                                            <Badge variant="outline" className="text-green-600 border-green-300">正常</Badge>
                                                        )}
                                                    </TableCell>
                                                    <TableCell className="text-sm">{item.remark || '-'}</TableCell>
                                                </TableRow>
                                            ))}
                                        </TableBody>
                                    </Table>
                                </Card>
                            </div>
                        </div>
                    ) : (
                        <div className="flex justify-center py-8">
                            <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
                        </div>
                    )}
                </DialogContent>
            </Dialog>

            {/* 刪除確認 Dialog */}
            {deleteTarget && (
                <DeleteReasonDialog
                    open={!!deleteTarget}
                    onOpenChange={(open) => !open && setDeleteTarget(null)}
                    onConfirm={(reason) => deleteMutation.mutate({ id: deleteTarget.id, reason })}
                    title="刪除血液檢查紀錄"
                    description={`確定要刪除 ${deleteTarget.date} 的血液檢查紀錄嗎？此操作無法復原。`}
                    isPending={deleteMutation.isPending}
                />
            )}
        </div>
    )
}
