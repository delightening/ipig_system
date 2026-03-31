import { useState, useMemo, useCallback } from 'react'
import { STALE_TIME } from '@/lib/query'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import {
  bloodTestTemplateApi,
  bloodTestPanelApi,
  BloodTestTemplate,
  BloodTestPanel,
  UpdateBloodTestTemplateRequest,
} from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import {
  getApiErrorMessage,
  bloodTestTemplateFormSchema,
  bloodTestPanelFormSchema,
  type BloodTestTemplateFormData,
  type BloodTestPanelFormData,
} from '@/lib/validation'

export type SortField = 'code' | 'name' | 'sort_order' | 'default_unit' | 'default_price'
export type SortOrder = 'asc' | 'desc'
export type ShowFilter = 'all' | 'active' | 'inactive'

const defaultTemplateValues: BloodTestTemplateFormData = {
  code: '',
  name: '',
  default_unit: '',
  reference_range: '',
  default_price: 0,
  sort_order: 0,
  panel_id: undefined,
}

const defaultPanelValues: BloodTestPanelFormData = {
  key: '',
  name: '',
  icon: '',
  sort_order: 0,
}

export function useBloodTestTemplates() {
  const queryClient = useQueryClient()

  const [search, setSearch] = useState('')
  const [showFilter, setShowFilter] = useState<ShowFilter>('all')
  const [sortField, setSortField] = useState<SortField>('sort_order')
  const [sortOrder, setSortOrder] = useState<SortOrder>('asc')
  const [dialogOpen, setDialogOpen] = useState(false)
  const [editingTemplate, setEditingTemplate] = useState<BloodTestTemplate | null>(null)
  const [selectedPanel, setSelectedPanel] = useState<string>('all')
  const [panelDialogOpen, setPanelDialogOpen] = useState(false)

  const templateForm = useForm<BloodTestTemplateFormData>({
    resolver: zodResolver(bloodTestTemplateFormSchema),
    defaultValues: defaultTemplateValues,
  })

  const panelForm = useForm<BloodTestPanelFormData>({
    resolver: zodResolver(bloodTestPanelFormSchema),
    defaultValues: defaultPanelValues,
  })

  const { data: templates, isLoading } = useQuery({
    queryKey: ['blood-test-templates-all'],
    staleTime: STALE_TIME.REFERENCE,
    queryFn: async () => {
      const response = await bloodTestTemplateApi.listAll()
      return response.data
    },
  })

  const { data: panels } = useQuery({
    queryKey: ['blood-test-panels-all'],
    staleTime: STALE_TIME.REFERENCE,
    queryFn: async () => {
      const response = await bloodTestPanelApi.listAll()
      return response.data
    },
  })

  const templatePanelMap = useMemo(() => {
    if (!panels) return new Map<string, string>()
    const map = new Map<string, string>()
    for (const panel of panels) {
      for (const item of panel.items) {
        map.set(item.id, panel.key)
      }
    }
    return map
  }, [panels])

  const sortTemplates = useCallback(
    (list: BloodTestTemplate[]) => {
      return [...list].sort((a, b) => {
        let cmp = 0
        switch (sortField) {
          case 'code':
            cmp = a.code.localeCompare(b.code)
            break
          case 'name':
            cmp = a.name.localeCompare(b.name)
            break
          case 'sort_order':
            cmp = a.sort_order - b.sort_order
            break
          case 'default_unit':
            cmp = (a.default_unit || '').localeCompare(b.default_unit || '')
            break
          case 'default_price':
            cmp = (a.default_price || 0) - (b.default_price || 0)
            break
        }
        return sortOrder === 'asc' ? cmp : -cmp
      })
    },
    [sortField, sortOrder]
  )

  const { groupedData, flatFiltered } = useMemo(() => {
    if (!templates) return { groupedData: [], flatFiltered: [] }

    let result = [...templates]
    if (search) {
      const q = search.toLowerCase()
      result = result.filter(
        (t) =>
          t.code.toLowerCase().includes(q) ||
          t.name.toLowerCase().includes(q) ||
          (t.default_unit && t.default_unit.toLowerCase().includes(q))
      )
    }
    if (showFilter === 'active') result = result.filter((t) => t.is_active)
    else if (showFilter === 'inactive') result = result.filter((t) => !t.is_active)
    if (selectedPanel !== 'all' && panels) {
      const panel = panels.find((p) => p.key === selectedPanel)
      if (panel) {
        const panelTemplateIds = new Set(panel.items.map((i) => i.id))
        result = result.filter((t) => panelTemplateIds.has(t.id))
      }
    }
    const sorted = sortTemplates(result)
    if (selectedPanel === 'all' && panels && !search) {
      const grouped: { panel: BloodTestPanel | null; items: BloodTestTemplate[] }[] = []
      const usedIds = new Set<string>()
      for (const panel of panels) {
        const panelTemplateIds = new Set(panel.items.map((i) => i.id))
        const panelItems = sorted.filter((t) => panelTemplateIds.has(t.id))
        if (panelItems.length > 0) {
          grouped.push({ panel, items: panelItems })
          panelItems.forEach((t) => usedIds.add(t.id))
        }
      }
      const uncategorized = sorted.filter((t) => !usedIds.has(t.id))
      if (uncategorized.length > 0) {
        grouped.push({ panel: null, items: uncategorized })
      }
      return { groupedData: grouped, flatFiltered: sorted }
    }
    return { groupedData: [{ panel: null, items: sorted }], flatFiltered: sorted }
  }, [templates, panels, search, showFilter, selectedPanel, sortTemplates])

  const createMutation = useMutation({
    mutationFn: (data: BloodTestTemplateFormData) => bloodTestTemplateApi.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['blood-test-templates'] })
      queryClient.invalidateQueries({ queryKey: ['blood-test-templates-all'] })
      queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
      toast({ title: '成功', description: '檢查項目已建立' })
      setDialogOpen(false)
      resetForm()
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '建立失敗'),
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({
      id,
      data,
    }: {
      id: string
      data: UpdateBloodTestTemplateRequest
    }) => bloodTestTemplateApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['blood-test-templates'] })
      queryClient.invalidateQueries({ queryKey: ['blood-test-templates-all'] })
      queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
      toast({ title: '成功', description: '檢查項目已更新' })
      setDialogOpen(false)
      resetForm()
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '更新失敗'),
        variant: 'destructive',
      })
    },
  })

  const toggleMutation = useMutation({
    mutationFn: ({ id, is_active }: { id: string; is_active: boolean }) =>
      is_active ? bloodTestTemplateApi.update(id, { is_active: true }) : bloodTestTemplateApi.delete(id),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['blood-test-templates'] })
      queryClient.invalidateQueries({ queryKey: ['blood-test-templates-all'] })
      toast({
        title: '成功',
        description: variables.is_active ? '項目已恢復啟用' : '項目已停用',
      })
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '操作失敗'),
        variant: 'destructive',
      })
    },
  })

  const createPanelMutation = useMutation({
    mutationFn: (data: BloodTestPanelFormData) => bloodTestPanelApi.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['blood-test-panels-all'] })
      toast({ title: '成功', description: '分類已建立' })
      setPanelDialogOpen(false)
      panelForm.reset(defaultPanelValues)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '建立分類失敗'),
        variant: 'destructive',
      })
    },
  })

  const resetForm = () => {
    templateForm.reset(defaultTemplateValues)
    setEditingTemplate(null)
  }

  const handleEdit = (template: BloodTestTemplate) => {
    setEditingTemplate(template)
    const panelKey = templatePanelMap.get(template.id)
    const panel = panels?.find((p) => p.key === panelKey)
    templateForm.reset({
      code: template.code,
      name: template.name,
      default_unit: template.default_unit || '',
      reference_range: template.reference_range || '',
      default_price: template.default_price || 0,
      sort_order: template.sort_order,
      panel_id: panel?.id,
    })
    setDialogOpen(true)
  }

  const onTemplateSubmit = (data: BloodTestTemplateFormData) => {
    if (editingTemplate) {
      updateMutation.mutate({
        id: editingTemplate.id,
        data: {
          name: data.name,
          default_unit: data.default_unit || undefined,
          reference_range: data.reference_range || undefined,
          default_price: data.default_price || undefined,
          sort_order: data.sort_order,
          panel_id: data.panel_id || undefined,
        },
      })
    } else {
      createMutation.mutate(data)
    }
  }

  const handleSubmit = templateForm.handleSubmit(onTemplateSubmit)

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortOrder((o) => (o === 'asc' ? 'desc' : 'asc'))
    } else {
      setSortField(field)
      setSortOrder('asc')
    }
  }

  const activeCount = templates?.filter((t) => t.is_active).length || 0
  const totalCount = templates?.length || 0

  return {
    search,
    setSearch,
    showFilter,
    setShowFilter,
    sortField,
    sortOrder,
    selectedPanel,
    setSelectedPanel,
    dialogOpen,
    setDialogOpen,
    editingTemplate,
    templateForm,
    panelDialogOpen,
    setPanelDialogOpen,
    panelForm,
    templates,
    panels,
    isLoading,
    groupedData,
    flatFiltered,
    createMutation,
    updateMutation,
    toggleMutation,
    createPanelMutation,
    resetForm,
    handleEdit,
    handleSubmit,
    handleSort,
    activeCount,
    totalCount,
  }
}
