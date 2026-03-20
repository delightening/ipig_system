/**
 * 血液檢查分頁元件
 * 顯示在動物詳情頁 (AnimalDetailPage) 中
 */
import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import api, {
    BloodTestListItem,
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
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'
import { Loader2, Plus, Eye, Edit2, Trash2, AlertCircle, FileText } from 'lucide-react'

import { BloodTestFormDialog } from './blood-test/BloodTestFormDialog'
import { BloodTestDetailDialog } from './blood-test/BloodTestDetailDialog'
import { useBloodTestActions } from './blood-test/useBloodTestActions'

interface BloodTestTabProps {
    animalId: string
    afterParam?: string
}

export function BloodTestTab({ animalId, afterParam = '' }: BloodTestTabProps) {
    const [showDetailDialog, setShowDetailDialog] = useState(false)
    const [viewingId, setViewingId] = useState<string | null>(null)

    const actions = useBloodTestActions(animalId)

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

    return (
        <>
            <Card>
                <CardHeader className="flex flex-row items-center justify-between">
                    <div>
                        <CardTitle>血液檢查紀錄</CardTitle>
                        <CardDescription>記錄實驗動物的血液檢查結果與檢驗數據</CardDescription>
                    </div>
                    <Button className="bg-red-600 hover:bg-red-700 shrink-0" onClick={actions.openCreateForm}>
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
                        <BloodTestListTable
                            bloodTests={bloodTests}
                            onView={(id) => { setViewingId(id); setShowDetailDialog(true) }}
                            onEdit={actions.openEditForm}
                            onDelete={(id, date) => actions.setDeleteTarget({ id, date })}
                        />
                    )}
                </CardContent>
            </Card>

            <BloodTestFormDialog
                open={actions.showFormDialog}
                onOpenChange={actions.setShowFormDialog}
                editingId={actions.editingId}
                formData={actions.formData}
                setFormData={actions.setFormData}
                labNameOption={actions.labNameOption}
                setLabNameOption={actions.setLabNameOption}
                templates={templates}
                panels={panels}
                isPending={actions.isPending}
                onSubmit={actions.handleSubmit}
                onClose={actions.handleFormClose}
            />

            <BloodTestDetailDialog
                open={showDetailDialog}
                onOpenChange={(open) => {
                    if (!open) { setShowDetailDialog(false); setViewingId(null) }
                }}
                detail={viewDetail}
            />

            {actions.deleteTarget && (
                <DeleteReasonDialog
                    open={!!actions.deleteTarget}
                    onOpenChange={(open) => !open && actions.setDeleteTarget(null)}
                    onConfirm={(reason) => actions.deleteMutation.mutate({ id: actions.deleteTarget!.id, reason })}
                    title="刪除血液檢查紀錄"
                    description={`確定要刪除 ${actions.deleteTarget.date} 的血液檢查紀錄嗎？此操作無法復原。`}
                    isPending={actions.deleteMutation.isPending}
                />
            )}
        </>
    )
}

// --- 列表表格子元件 ---

interface BloodTestListTableProps {
    bloodTests: BloodTestListItem[]
    onView: (id: string) => void
    onEdit: (id: string) => void
    onDelete: (id: string, date: string) => void
}

function BloodTestListTable({ bloodTests, onView, onEdit, onDelete }: BloodTestListTableProps) {
    return (
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
                            <Button variant="ghost" size="icon" onClick={() => onView(test.id)}>
                                <Eye className="h-4 w-4" />
                            </Button>
                            <Button variant="ghost" size="icon" onClick={() => onEdit(test.id)}>
                                <Edit2 className="h-4 w-4" />
                            </Button>
                            <Button
                                variant="ghost"
                                size="icon"
                                className="text-red-600"
                                onClick={() => onDelete(test.id, test.test_date)}
                            >
                                <Trash2 className="h-4 w-4" />
                            </Button>
                        </TableCell>
                    </TableRow>
                ))}
            </TableBody>
        </Table>
    )
}
