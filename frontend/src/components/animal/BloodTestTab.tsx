/**
 * 血液檢查分頁元件
 * 顯示在動物詳情頁 (AnimalDetailPage) 中
 */
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import api, {
    BloodTestListItem,
    bloodTestApi,
    bloodTestTemplateApi,
    bloodTestPanelApi,
} from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'

import { BloodTestListCard } from './bloodTest/BloodTestListCard'
import { BloodTestFormDialog } from './bloodTest/BloodTestFormDialog'
import { BloodTestDetailDialog } from './bloodTest/BloodTestDetailDialog'
import { useBloodTestForm } from './bloodTest/useBloodTestForm'

interface BloodTestTabProps {
    animalId: string
    /** 資料隔離 query string，例如 '?after=2024-01-01T00:00:00Z' */
    afterParam?: string
}

export function BloodTestTab({ animalId, afterParam = '' }: BloodTestTabProps) {
    const queryClient = useQueryClient()
    const [showDetailDialog, setShowDetailDialog] = useState(false)
    const [viewingId, setViewingId] = useState<string | null>(null)
    const [deleteTarget, setDeleteTarget] = useState<{ id: string; date: string } | null>(null)

    const form = useBloodTestForm(animalId)

    // 載入血液檢查列表
    const { data: bloodTests = [], isLoading } = useQuery({
        queryKey: ['animal-blood-tests', animalId, afterParam],
        queryFn: async () => {
            const res = await api.get<BloodTestListItem[]>(`/animals/${animalId}/blood-tests${afterParam}`)
            return res.data
        },
        staleTime: 30_000,
    })

    // 載入模板列表
    const { data: templates = [] } = useQuery({
        queryKey: ['blood-test-templates'],
        queryFn: async () => {
            const res = await bloodTestTemplateApi.list()
            return res.data
        },
        staleTime: 600_000,
    })

    // 載入組合列表
    const { data: panels = [] } = useQuery({
        queryKey: ['blood-test-panels'],
        queryFn: async () => {
            const res = await bloodTestPanelApi.list()
            return res.data
        },
        staleTime: 600_000,
    })

    // 載入單筆詳情
    const { data: viewDetail } = useQuery({
        queryKey: ['blood-test-detail', viewingId],
        queryFn: async () => {
            if (!viewingId) return null
            const res = await bloodTestApi.getById(viewingId)
            return res.data
        },
        enabled: !!viewingId,
        staleTime: 30_000,
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

    const panelActiveStates = form.panelActiveStates(panels)

    return (
        <>
            <BloodTestListCard
                bloodTests={bloodTests}
                isLoading={isLoading}
                onCreateClick={form.openCreateForm}
                onViewClick={(id) => {
                    setViewingId(id)
                    setShowDetailDialog(true)
                }}
                onEditClick={(id) => form.openEditForm(id)}
                onDeleteClick={(id, date) => setDeleteTarget({ id, date })}
            />

            <BloodTestFormDialog
                open={form.showFormDialog}
                editingId={form.editingId}
                labNameOption={form.labNameOption}
                formData={form.formData}
                isPending={form.isPending}
                templates={templates}
                panels={panels}
                panelActiveStates={panelActiveStates}
                onLabNameOptionChange={form.setLabNameOption}
                onFormDataChange={form.setFormData}
                onAddItemFromTemplate={(templateId) => form.addItemFromTemplate(templateId, templates)}
                onAddCustomItem={form.addCustomItem}
                onTogglePanel={(panel) => form.togglePanel(panel, panelActiveStates)}
                onRemoveItem={form.removeItem}
                onUpdateItem={form.updateItem}
                onSubmit={form.handleSubmit}
                onClose={form.closeForm}
            />

            <BloodTestDetailDialog
                open={showDetailDialog}
                detail={viewDetail}
                onClose={() => {
                    setShowDetailDialog(false)
                    setViewingId(null)
                }}
            />

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
