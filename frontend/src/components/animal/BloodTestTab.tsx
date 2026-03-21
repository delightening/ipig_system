/**
 * 血液檢查分頁元件
 * 顯示在動物詳情頁 (AnimalDetailPage) 中
 */
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
    BloodTestListItem,
    CreateBloodTestRequest,
    UpdateBloodTestRequest,
    bloodTestApi,
    bloodTestTemplateApi,
    bloodTestPanelApi,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { toast } from '@/components/ui/use-toast'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'
import { Loader2, Plus, Eye, Edit2, Trash2, AlertCircle, FileText } from 'lucide-react'

import { LAB_OPTIONS } from './blood-test/constants'
import { BloodTestFormDialog, type BloodTestFormData } from './blood-test/BloodTestFormDialog'
import { BloodTestDetailDialog } from './blood-test/BloodTestDetailDialog'

interface BloodTestTabProps {
    animalId: string
    afterParam?: string
}

const INITIAL_FORM_DATA: BloodTestFormData = {
    test_date: new Date().toISOString().split('T')[0],
    lab_name: '',
    remark: '',
    items: [],
}

export function BloodTestTab({ animalId, afterParam = '' }: BloodTestTabProps) {
    const queryClient = useQueryClient()
    const [showFormDialog, setShowFormDialog] = useState(false)
    const [showDetailDialog, setShowDetailDialog] = useState(false)
    const [editingId, setEditingId] = useState<string | null>(null)
    const [viewingId, setViewingId] = useState<string | null>(null)
    const [deleteTarget, setDeleteTarget] = useState<{ id: string; date: string } | null>(null)
    const [labNameOption, setLabNameOption] = useState<string>('')
    const [formData, setFormData] = useState<BloodTestFormData>({ ...INITIAL_FORM_DATA })

    // 載入血液檢查列表
    const { data: bloodTests = [], isLoading } = useQuery({
        queryKey: ['animal-blood-tests', animalId, afterParam],
        queryFn: async () => {
            const res = await api.get<BloodTestListItem[]>(`/animals/${animalId}/blood-tests${afterParam}`)
            return res.data
        },
        staleTime: 30_000,
    })

    // 載入模板與組合
    const { data: templates = [] } = useQuery({
        queryKey: ['blood-test-templates'],
        queryFn: async () => (await bloodTestTemplateApi.list()).data,
        staleTime: 600_000,
    })

    const { data: panels = [] } = useQuery({
        queryKey: ['blood-test-panels'],
        queryFn: async () => (await bloodTestPanelApi.list()).data,
        staleTime: 600_000,
    })

    // 載入單筆詳情（檢視用）
    const { data: viewDetail } = useQuery({
        queryKey: ['blood-test-detail', viewingId],
        queryFn: async () => {
            if (!viewingId) return null
            return (await bloodTestApi.getById(viewingId)).data
        },
        enabled: !!viewingId,
        staleTime: 30_000,
    })

    const resetForm = () => {
        setFormData({ ...INITIAL_FORM_DATA, test_date: new Date().toISOString().split('T')[0] })
        setLabNameOption('')
    }

    // Mutations
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

    const openCreateForm = () => {
        resetForm()
        setEditingId(null)
        setShowFormDialog(true)
    }

    const loadEditDataMutation = useMutation({
        mutationFn: (id: string) => bloodTestApi.getById(id),
        onSuccess: (res, id) => {
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
        },
        onError: () => {
            toast({ title: '錯誤', description: '載入資料失敗', variant: 'destructive' })
        },
    })

    const openEditForm = (id: string) => {
        loadEditDataMutation.mutate(id)
    }

    const handleSubmit = () => {
        if (!formData.test_date) {
            toast({ title: '錯誤', description: '請填寫檢查日期', variant: 'destructive' })
            return
        }
        if (formData.items.length === 0) {
            toast({ title: '錯誤', description: '至少需要一個檢查項目', variant: 'destructive' })
            return
        }
        if (formData.items.some((item) => !item.item_name.trim())) {
            toast({ title: '錯誤', description: '所有檢查項目都需要填寫名稱', variant: 'destructive' })
            return
        }

        const payload = {
            test_date: formData.test_date,
            lab_name: formData.lab_name || undefined,
            remark: formData.remark || undefined,
            items: formData.items,
        }

        if (editingId) {
            updateMutation.mutate({ id: editingId, data: payload })
        } else {
            createMutation.mutate(payload)
        }
    }

    const handleFormClose = () => {
        setShowFormDialog(false)
        setEditingId(null)
        resetForm()
    }

    const isPending = createMutation.isPending || updateMutation.isPending

    return (
        <>
            <Card>
                <CardHeader className="flex flex-row items-center justify-between">
                    <div>
                        <CardTitle>血液檢查紀錄</CardTitle>
                        <CardDescription>記錄實驗動物的血液檢查結果與檢驗數據</CardDescription>
                    </div>
                    <Button className="bg-red-600 hover:bg-red-700 shrink-0" onClick={openCreateForm}>
                        <Plus className="h-4 w-4 mr-2" />
                        新增血液檢查
                    </Button>
                </CardHeader>
                <CardContent>
                    {isLoading ? (
                        <div className="flex justify-center py-8">
                            <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
                        </div>
                    ) : bloodTests.length === 0 ? (
                        <div className="text-center py-12 text-slate-500">
                            <FileText className="h-12 w-12 mx-auto mb-3 text-slate-300" />
                            <p>尚無血液檢查紀錄</p>
                            <p className="text-sm mt-1">點擊上方按鈕新增</p>
                        </div>
                    ) : (
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
                    )}
                </CardContent>
            </Card>

            {/* 新增/編輯 Dialog */}
            <BloodTestFormDialog
                open={showFormDialog}
                onOpenChange={setShowFormDialog}
                editingId={editingId}
                formData={formData}
                setFormData={setFormData}
                labNameOption={labNameOption}
                setLabNameOption={setLabNameOption}
                templates={templates}
                panels={panels}
                isPending={isPending}
                onSubmit={handleSubmit}
                onClose={handleFormClose}
            />

            {/* 詳情 Dialog */}
            <BloodTestDetailDialog
                open={showDetailDialog}
                onOpenChange={(open) => {
                    if (!open) {
                        setShowDetailDialog(false)
                        setViewingId(null)
                    }
                }}
                detail={viewDetail}
            />

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
        </>
    )
}
