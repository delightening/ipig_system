import { useState, useMemo, useCallback } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'

import {
    BloodTestItemInput,
    CreateBloodTestRequest,
    UpdateBloodTestRequest,
    bloodTestApi,
    BloodTestPanel,
} from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { LAB_OPTIONS } from './constants'

export interface BloodTestFormData {
    test_date: string
    lab_name: string
    remark: string
    items: BloodTestItemInput[]
}

function defaultFormData(): BloodTestFormData {
    return {
        test_date: new Date().toISOString().split('T')[0],
        lab_name: '',
        remark: '',
        items: [],
    }
}

export function useBloodTestForm(animalId: string) {
    const queryClient = useQueryClient()
    const [showFormDialog, setShowFormDialog] = useState(false)
    const [editingId, setEditingId] = useState<string | null>(null)
    const [labNameOption, setLabNameOption] = useState<string>('')
    const [formData, setFormData] = useState<BloodTestFormData>(defaultFormData)

    const resetForm = useCallback(() => {
        setFormData(defaultFormData())
        setLabNameOption('')
    }, [])

    const openCreateForm = useCallback(() => {
        resetForm()
        setEditingId(null)
        setShowFormDialog(true)
    }, [resetForm])

    const openEditForm = useCallback(async (id: string) => {
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
    }, [])

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

    const handleSubmit = useCallback(() => {
        if (!formData.test_date) {
            toast({ title: '錯誤', description: '請填寫檢查日期', variant: 'destructive' })
            return
        }
        if (formData.items.length === 0) {
            toast({ title: '錯誤', description: '至少需要一個檢查項目', variant: 'destructive' })
            return
        }
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
    }, [formData, editingId, createMutation, updateMutation])

    const addItemFromTemplate = useCallback((templateId: string, templates: { id: string; name: string; default_unit?: string; reference_range?: string }[]) => {
        const template = templates.find((t) => t.id === templateId)
        if (!template) return

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
    }, [formData.items])

    const addCustomItem = useCallback(() => {
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
    }, [])

    const togglePanel = useCallback((panel: BloodTestPanel, panelActiveStates: Record<string, boolean>) => {
        const isActive = panelActiveStates[panel.id]
        if (isActive) {
            const panelTemplateIds = new Set(panel.items.map(t => t.id))
            setFormData((prev) => ({
                ...prev,
                items: prev.items.filter(item => !item.template_id || !panelTemplateIds.has(item.template_id)),
            }))
        } else {
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
    }, [formData.items])

    const removeItem = useCallback((index: number) => {
        setFormData((prev) => ({
            ...prev,
            items: prev.items.filter((_, i) => i !== index),
        }))
    }, [])

    const updateItem = useCallback((index: number, field: keyof BloodTestItemInput, value: unknown) => {
        setFormData((prev) => ({
            ...prev,
            items: prev.items.map((item, i) =>
                i === index ? { ...item, [field]: value } : item
            ),
        }))
    }, [])

    const closeForm = useCallback(() => {
        setShowFormDialog(false)
        setEditingId(null)
        resetForm()
    }, [resetForm])

    const panelActiveStates = useMemo(() => {
        const itemTemplateIds = new Set(formData.items.map(i => i.template_id).filter(Boolean))
        return (panels: BloodTestPanel[]) =>
            panels.reduce((acc, panel) => {
                if (panel.items.length === 0) {
                    acc[panel.id] = false
                } else {
                    acc[panel.id] = panel.items.every(t => itemTemplateIds.has(t.id))
                }
                return acc
            }, {} as Record<string, boolean>)
    }, [formData.items])

    const isPending = createMutation.isPending || updateMutation.isPending

    return {
        showFormDialog,
        editingId,
        labNameOption,
        setLabNameOption,
        formData,
        setFormData,
        isPending,
        panelActiveStates,
        openCreateForm,
        openEditForm,
        handleSubmit,
        addItemFromTemplate,
        addCustomItem,
        togglePanel,
        removeItem,
        updateItem,
        closeForm,
    }
}
