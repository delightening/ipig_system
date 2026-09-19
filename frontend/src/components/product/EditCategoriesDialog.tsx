import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
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
import { Switch } from '@/components/ui/switch'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { useAuthHasRole } from '@/stores/auth'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { Loader2, Plus, Tags, ChevronRight, ChevronDown, FolderOpen, FolderTree, Trash2 } from 'lucide-react'
import { cn } from '@/lib/utils'
import type {
  CategoryForEdit,
  SubcategoryForEdit,
  CategoriesTreeResponse,
  CreateSkuSubcategoryRequest,
  UpdateSkuCategoryRequest,
  UpdateSkuSubcategoryRequest,
} from '@/types/sku'

const EDIT_CATEGORY_VALUE = '__category__'

interface EditCategoriesDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function EditCategoriesDialog({ open, onOpenChange }: EditCategoriesDialogProps) {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const hasRole = useAuthHasRole()
  const isAdmin = hasRole('admin')
  const { dialogState, confirm } = useConfirmDialog()
  const [selectedCategoryCode, setSelectedCategoryCode] = useState<string>('')
  const [selectedTarget, setSelectedTarget] = useState<string>('') // EDIT_CATEGORY_VALUE or sub.code
  const [formName, setFormName] = useState('')
  const [formSortOrder, setFormSortOrder] = useState(0)
  const [formIsActive, setFormIsActive] = useState(true)

  const [expandedCategories, setExpandedCategories] = useState<Record<string, boolean>>({})
  const [newSubCategoryCode, setNewSubCategoryCode] = useState('')
  const [newSubCode, setNewSubCode] = useState('')
  const [newSubName, setNewSubName] = useState('')
  const [newSubSortOrder, setNewSubSortOrder] = useState(0)
  const [newSubIsActive, setNewSubIsActive] = useState(true)

  const { data: tree, isLoading } = useQuery({
    queryKey: ['sku-categories-tree'],
    queryFn: async () => {
      const res = await api.get<CategoriesTreeResponse>('/sku/categories/tree')
      return res.data
    },
    enabled: open,
  })

  const categories = tree?.categories ?? []
  const selectedCategory = categories.find(c => c.code === selectedCategoryCode)

  // 左側選取變更時，右側「新增子類」的所屬品類一併同步
  useEffect(() => {
    if (selectedCategoryCode) setNewSubCategoryCode(selectedCategoryCode)
  }, [selectedCategoryCode])

  useEffect(() => {
    if (!selectedCategory) {
      setFormName('')
      setFormSortOrder(0)
      setFormIsActive(true)
      return
    }
    if (selectedTarget === EDIT_CATEGORY_VALUE) {
      setFormName(selectedCategory.name)
      setFormSortOrder(selectedCategory.sort_order)
      setFormIsActive(selectedCategory.is_active)
    } else {
      const sub = selectedCategory.subcategories.find(s => s.code === selectedTarget)
      if (sub) {
        setFormName(sub.name)
        setFormSortOrder(sub.sort_order)
        setFormIsActive(sub.is_active)
      }
    }
  }, [selectedCategory, selectedTarget, selectedCategoryCode])

  const updateCategoryMutation = useMutation({
    mutationFn: async ({ code, body }: { code: string; body: UpdateSkuCategoryRequest }) => {
      await api.patch(`/sku/categories/${code}`, body)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sku-categories'] })
      queryClient.invalidateQueries({ queryKey: ['sku-categories-tree'] })
      queryClient.invalidateQueries({ queryKey: ['sku-subcategories'] })
      toast({ title: t('erpMaster.categories.toast.saved'), description: t('erpMaster.categories.toast.categoryUpdated') })
    },
    onError: (err: unknown) => {
      toast({
        title: t('erpMaster.categories.toast.saveFailed'),
        description: getApiErrorMessage(err, t('erpMaster.categories.toast.categoryUpdateFailed')),
        variant: 'destructive',
      })
    },
  })

  const updateSubcategoryMutation = useMutation({
    mutationFn: async ({
      categoryCode,
      code,
      body,
    }: {
      categoryCode: string
      code: string
      body: UpdateSkuSubcategoryRequest
    }) => {
      await api.patch(`/sku/categories/${categoryCode}/subcategories/${code}`, body)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sku-categories'] })
      queryClient.invalidateQueries({ queryKey: ['sku-categories-tree'] })
      queryClient.invalidateQueries({ queryKey: ['sku-subcategories'] })
      toast({ title: t('erpMaster.categories.toast.saved'), description: t('erpMaster.categories.toast.subcategoryUpdated') })
    },
    onError: (err: unknown) => {
      toast({
        title: t('erpMaster.categories.toast.saveFailed'),
        description: getApiErrorMessage(err, t('erpMaster.categories.toast.subcategoryUpdateFailed')),
        variant: 'destructive',
      })
    },
  })

  const createSubcategoryMutation = useMutation({
    mutationFn: async ({
      categoryCode,
      body,
    }: {
      categoryCode: string
      body: CreateSkuSubcategoryRequest
    }) => {
      const res = await api.post(
        `/sku/categories/${categoryCode}/subcategories`,
        body
      )
      return res.data
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ['sku-categories'] })
      queryClient.invalidateQueries({ queryKey: ['sku-categories-tree'] })
      queryClient.invalidateQueries({ queryKey: ['sku-subcategories'] })
      toast({ title: t('erpMaster.categories.toast.created'), description: t('erpMaster.categories.toast.subcategoryCreated') })
      setNewSubCategoryCode(variables.categoryCode)
      setNewSubCode('')
      setNewSubName('')
      setNewSubSortOrder(0)
      setNewSubIsActive(true)
      setSelectedCategoryCode(variables.categoryCode)
      setSelectedTarget(variables.body.code.toUpperCase().trim())
    },
    onError: (err: unknown) => {
      toast({
        title: t('erpMaster.categories.toast.createFailed'),
        description: getApiErrorMessage(err, t('erpMaster.categories.toast.subcategoryCreateFailed')),
        variant: 'destructive',
      })
    },
  })

  const deleteCategoryMutation = useMutation({
    mutationFn: async (code: string) => {
      await api.delete(`/sku/categories/${code}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sku-categories'] })
      queryClient.invalidateQueries({ queryKey: ['sku-categories-tree'] })
      queryClient.invalidateQueries({ queryKey: ['sku-subcategories'] })
      toast({ title: t('common.deleted'), description: t('erpMaster.categories.toast.categoryDeleted') })
      setSelectedCategoryCode('')
      setSelectedTarget('')
    },
    onError: (err: unknown) => {
      toast({
        title: t('erpMaster.categories.toast.deleteFailed'),
        description: getApiErrorMessage(err, t('erpMaster.categories.toast.categoryDeleteFailed')),
        variant: 'destructive',
      })
    },
  })

  const deleteSubcategoryMutation = useMutation({
    mutationFn: async ({ categoryCode, code }: { categoryCode: string; code: string }) => {
      await api.delete(`/sku/categories/${categoryCode}/subcategories/${code}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sku-categories'] })
      queryClient.invalidateQueries({ queryKey: ['sku-categories-tree'] })
      queryClient.invalidateQueries({ queryKey: ['sku-subcategories'] })
      toast({ title: t('common.deleted'), description: t('erpMaster.categories.toast.subcategoryDeleted') })
      setSelectedCategoryCode('')
      setSelectedTarget('')
    },
    onError: (err: unknown) => {
      toast({
        title: t('erpMaster.categories.toast.deleteFailed'),
        description: getApiErrorMessage(err, t('erpMaster.categories.toast.subcategoryDeleteFailed')),
        variant: 'destructive',
      })
    },
  })

  const isEditingCategory = selectedTarget === EDIT_CATEGORY_VALUE
  const selectedSub = selectedCategory?.subcategories.find(s => s.code === selectedTarget)
  const displayCode = isEditingCategory ? selectedCategory?.code : selectedSub?.code

  const handleSave = () => {
    if (!selectedCategory) return
    if (isEditingCategory) {
      updateCategoryMutation.mutate({
        code: selectedCategory.code,
        body: { name: formName.trim(), sort_order: formSortOrder, is_active: formIsActive },
      })
    } else if (selectedSub) {
      updateSubcategoryMutation.mutate({
        categoryCode: selectedCategory.code,
        code: selectedSub.code,
        body: { name: formName.trim(), sort_order: formSortOrder, is_active: formIsActive },
      })
    }
  }

  const saving =
    updateCategoryMutation.isPending ||
    updateSubcategoryMutation.isPending ||
    createSubcategoryMutation.isPending ||
    deleteCategoryMutation.isPending ||
    deleteSubcategoryMutation.isPending

  const handleDelete = async () => {
    if (!selectedCategory) return
    const ok = await confirm({
      title: t('erpMaster.categories.deleteTitle'),
      description: isEditingCategory
        ? t('erpMaster.categories.deleteCategoryDescription', { code: selectedCategory.code, name: selectedCategory.name })
        : t('erpMaster.categories.deleteSubcategoryDescription', { code: selectedSub?.code, name: selectedSub?.name }),
      variant: 'destructive',
      confirmLabel: t('common.confirmDelete'),
    })
    if (!ok) return
    if (isEditingCategory) {
      deleteCategoryMutation.mutate(selectedCategory.code)
    } else if (selectedSub) {
      deleteSubcategoryMutation.mutate({
        categoryCode: selectedCategory.code,
        code: selectedSub.code,
      })
    }
  }

  const handleCreateSub = () => {
    const catCode = newSubCategoryCode || selectedCategory?.code
    if (!catCode) {
      toast({
        title: t('erpMaster.categories.toast.selectCategory'),
        description: t('erpMaster.categories.toast.selectCategoryHint'),
        variant: 'destructive',
      })
      return
    }
    const code = newSubCode.trim().toUpperCase()
    if (code.length !== 3) {
      toast({
        title: t('erpMaster.categories.toast.validationFailed'),
        description: t('erpMaster.categories.toast.subcodeLength'),
        variant: 'destructive',
      })
      return
    }
    if (!newSubName.trim()) {
      toast({
        title: t('erpMaster.categories.toast.validationFailed'),
        description: t('erpMaster.categories.toast.subnameRequired'),
        variant: 'destructive',
      })
      return
    }
    createSubcategoryMutation.mutate({
      categoryCode: catCode,
      body: {
        code,
        name: newSubName.trim(),
        sort_order: newSubSortOrder,
        is_active: newSubIsActive,
      },
    })
  }

  const selectCategory = (cat: CategoryForEdit) => {
    setSelectedCategoryCode(cat.code)
    setSelectedTarget(EDIT_CATEGORY_VALUE)
  }

  const selectSubcategory = (catCode: string, sub: SubcategoryForEdit) => {
    setSelectedCategoryCode(catCode)
    setSelectedTarget(sub.code)
  }

  const toggleExpand = (catCode: string, e: React.MouseEvent) => {
    e.stopPropagation()
    setExpandedCategories(prev => ({ ...prev, [catCode]: !prev[catCode] }))
  }

  const isExpanded = (catCode: string) => expandedCategories[catCode] !== false

  const isSelected = (catCode: string, subCode?: string) => {
    if (selectedCategoryCode !== catCode) return false
    if (subCode == null) return selectedTarget === EDIT_CATEGORY_VALUE
    return selectedTarget === subCode
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="xl" className="max-h-[90vh] overflow-hidden flex flex-col p-0">
        <DialogHeader className="px-6 pt-6 pb-2 border-b shrink-0">
          <DialogTitle className="flex items-center gap-2">
            <Tags className="h-5 w-5" />
            {t('erpMaster.products.editCategories')}
          </DialogTitle>
          <DialogDescription>
            {t('erpMaster.categories.description')}
          </DialogDescription>
        </DialogHeader>

        {isLoading ? (
          <div className="flex items-center justify-center py-16">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <div className="flex flex-1 min-h-0">
            {/* 左側：既有分類列表 */}
            <div className="w-[280px] shrink-0 border-r flex flex-col bg-muted/20">
              <div className="px-3 py-2 border-b flex items-center gap-2 text-sm font-medium text-muted-foreground">
                <FolderTree className="h-4 w-4" />
                {t('erpMaster.categories.existing')}
              </div>
              <div className="flex-1 overflow-auto p-2">
                {categories.map(cat => {
                  const expanded = isExpanded(cat.code)
                  const hasSubs = cat.subcategories.length > 0
                  return (
                    <div key={cat.code} className="mb-1">
                      <div
                        className={cn(
                          'w-full text-left px-2 py-2 rounded-md flex items-center gap-1 text-sm transition-colors',
                          isSelected(cat.code)
                            ? 'bg-primary/15 text-primary font-medium border border-primary/30'
                            : 'hover:bg-muted border border-transparent',
                          !cat.is_active && 'opacity-60'
                        )}
                      >
                        <button
                          type="button"
                          onClick={e => hasSubs && toggleExpand(cat.code, e)}
                          className="shrink-0 p-0.5 rounded hover:bg-muted/80"
                          aria-label={expanded ? t('erpMaster.common.collapse') : t('erpMaster.common.expand')}
                        >
                          {hasSubs ? (
                            expanded ? (
                              <ChevronDown className="h-4 w-4 text-muted-foreground" />
                            ) : (
                              <ChevronRight className="h-4 w-4 text-muted-foreground" />
                            )
                          ) : (
                            <span className="w-4 inline-block" />
                          )}
                        </button>
                        <button
                          type="button"
                          onClick={() => selectCategory(cat)}
                          className="flex-1 min-w-0 flex items-center gap-2 text-left"
                        >
                          <FolderOpen className="h-4 w-4 shrink-0" />
                          <span className="font-mono">{cat.code}</span>
                          <span className="truncate">{cat.name}</span>
                          {!cat.is_active && (
                            <span className="text-xs text-muted-foreground ml-auto">{t('erpMaster.common.inactive')}</span>
                          )}
                        </button>
                      </div>
                      {hasSubs && expanded && (
                        <div className="ml-4 mt-0.5 space-y-0.5">
                          {cat.subcategories.map(sub => (
                            <button
                              key={sub.code}
                              type="button"
                              onClick={() => selectSubcategory(cat.code, sub)}
                              className={cn(
                                'w-full text-left pl-3 py-1.5 rounded-md flex items-center gap-2 text-sm transition-colors',
                                isSelected(cat.code, sub.code)
                                  ? 'bg-primary/15 text-primary font-medium border border-primary/30'
                                  : 'hover:bg-muted border border-transparent',
                                !sub.is_active && 'opacity-60'
                              )}
                            >
                              <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                              <span className="font-mono">{sub.code}</span>
                              <span className="truncate">{sub.name}</span>
                              {!sub.is_active && (
                                <span className="text-xs text-muted-foreground ml-auto">{t('erpMaster.common.inactive')}</span>
                              )}
                            </button>
                          ))}
                        </div>
                      )}
                    </div>
                  )
                })}
              </div>
            </div>

            {/* 右側：編輯 + 新增 */}
            <div className="flex-1 flex flex-col min-w-0 overflow-auto">
              <div className="p-4 space-y-6">
                {/* 上區：編輯選取項目 */}
                <section className="space-y-3">
                  <h3 className="text-sm font-medium text-muted-foreground flex items-center gap-2">
                    {t('erpMaster.categories.editSelected')}
                    {selectedCategory && selectedTarget && (
                      <span className="font-normal text-foreground">
                        {isEditingCategory
                          ? `${selectedCategory.code} ${selectedCategory.name}`
                          : `${selectedSub?.code} ${selectedSub?.name}`}
                      </span>
                    )}
                  </h3>
                  {selectedCategory && selectedTarget ? (
                    <div className="space-y-4 rounded-lg border p-4 bg-card">
                      <div className="grid grid-cols-2 gap-4 sm:grid-cols-3">
                        <div className="space-y-2">
                          <Label className="text-muted-foreground">{t('erpMaster.common.code')}</Label>
                          <div className="font-mono text-sm py-2">{displayCode}</div>
                        </div>
                        <div className="space-y-2 col-span-2 sm:col-span-1">
                          <Label className="text-muted-foreground">{t('erpMaster.common.name')}</Label>
                          <Input
                            value={formName}
                            onChange={e => setFormName(e.target.value)}
                            placeholder={t('erpMaster.categories.displayName')}
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-muted-foreground">{t('erpMaster.common.sortOrder')}</Label>
                          <Input
                            type="number"
                            min={0}
                            max={9999}
                            value={formSortOrder}
                            onChange={e => setFormSortOrder(parseInt(e.target.value, 10) || 0)}
                          />
                        </div>
                        <div className="flex items-center gap-2 space-y-2">
                          <Label className="text-muted-foreground">{t('erpMaster.categories.enabled')}</Label>
                          <Switch checked={formIsActive} onCheckedChange={setFormIsActive} />
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <Button onClick={handleSave} disabled={saving}>
                          {saving ? (
                            <>
                              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                              {t('erpMaster.categories.saving')}
                            </>
                          ) : (
                            t('common.save')
                          )}
                        </Button>
                        {isAdmin && (
                          <Button
                            type="button"
                            variant="destructive"
                            disabled={saving}
                            onClick={handleDelete}
                          >
                            <Trash2 className="h-4 w-4 mr-1" />
                            {t('common.delete')}
                          </Button>
                        )}
                      </div>
                    </div>
                  ) : (
                    <p className="text-sm text-muted-foreground py-2">
                      {t('erpMaster.categories.selectPrompt')}
                    </p>
                  )}
                </section>

                {/* 下區：新增子類 */}
                <section className="space-y-3">
                  <h3 className="text-sm font-medium text-muted-foreground flex items-center gap-2">
                    <Plus className="h-4 w-4" />
                    {t('erpMaster.categories.addSubcategory')}
                  </h3>
                  <div className="rounded-lg border border-dashed p-4 bg-muted/20 space-y-4">
                    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                      <div className="space-y-2">
                        <Label className="text-muted-foreground">{t('erpMaster.categories.parentCategory')}</Label>
                        <Select
                          value={newSubCategoryCode || selectedCategoryCode}
                          onValueChange={v => {
                            setNewSubCategoryCode(v)
                            if (!selectedCategoryCode) setSelectedCategoryCode(v)
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder={t('erpMaster.categories.selectCategory')} />
                          </SelectTrigger>
                          <SelectContent>
                            {categories.map(cat => (
                              <SelectItem key={cat.code} value={cat.code}>
                                {cat.code} {cat.name}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                      <div className="space-y-2">
                        <Label className="text-muted-foreground">{t('erpMaster.categories.subcodeLabel')}</Label>
                        <Input
                          value={newSubCode}
                          onChange={e => setNewSubCode(e.target.value.toUpperCase().slice(0, 3))}
                          placeholder={t('erpMaster.categories.subcodePlaceholder')}
                          maxLength={3}
                          className="font-mono"
                        />
                      </div>
                      <div className="space-y-2">
                        <Label className="text-muted-foreground">{t('erpMaster.common.name')}</Label>
                        <Input
                          value={newSubName}
                          onChange={e => setNewSubName(e.target.value)}
                          placeholder={t('erpMaster.categories.displayName')}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label className="text-muted-foreground">{t('erpMaster.common.sortOrder')}</Label>
                        <Input
                          type="number"
                          min={0}
                          max={9999}
                          value={newSubSortOrder}
                          onChange={e => setNewSubSortOrder(parseInt(e.target.value, 10) || 0)}
                        />
                      </div>
                    </div>
                    <div className="flex items-center gap-4">
                      <div className="flex items-center gap-2">
                        <Label className="text-muted-foreground">{t('erpMaster.categories.enabled')}</Label>
                        <Switch checked={newSubIsActive} onCheckedChange={setNewSubIsActive} />
                      </div>
                      <Button
                        size="sm"
                        disabled={createSubcategoryMutation.isPending}
                        onClick={handleCreateSub}
                      >
                        {createSubcategoryMutation.isPending ? (
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        ) : null}
                        {t('erpMaster.categories.createSubcategory')}
                      </Button>
                    </div>
                  </div>
                </section>
              </div>
            </div>
          </div>
        )}
      </DialogContent>
      <ConfirmDialog state={dialogState} />
    </Dialog>
  )
}
