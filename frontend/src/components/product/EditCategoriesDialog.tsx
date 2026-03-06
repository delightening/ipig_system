import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
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
import { getApiErrorMessage } from '@/lib/validation'
import { Loader2, Plus, Tags, ChevronRight, ChevronDown, FolderOpen, FolderTree } from 'lucide-react'
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
  const queryClient = useQueryClient()
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
  const subcategories = selectedCategory?.subcategories ?? []

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
      toast({ title: '已儲存', description: '品類已更新' })
    },
    onError: (err: unknown) => {
      toast({
        title: '儲存失敗',
        description: getApiErrorMessage(err, '品類更新失敗'),
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
      toast({ title: '已儲存', description: '子類已更新' })
    },
    onError: (err: unknown) => {
      toast({
        title: '儲存失敗',
        description: getApiErrorMessage(err, '子類更新失敗'),
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
      toast({ title: '已建立', description: '子類已新增' })
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
        title: '建立失敗',
        description: getApiErrorMessage(err, '子類建立失敗'),
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
    createSubcategoryMutation.isPending

  const handleCreateSub = () => {
    const catCode = newSubCategoryCode || selectedCategory?.code
    if (!catCode) {
      toast({
        title: '請選擇品類',
        description: '新增子類前請先選擇所屬品類',
        variant: 'destructive',
      })
      return
    }
    const code = newSubCode.trim().toUpperCase()
    if (code.length !== 3) {
      toast({
        title: '驗證失敗',
        description: '子類代碼須為 3 碼',
        variant: 'destructive',
      })
      return
    }
    if (!newSubName.trim()) {
      toast({
        title: '驗證失敗',
        description: '請輸入子類名稱',
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
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-hidden flex flex-col p-0">
        <DialogHeader className="px-6 pt-6 pb-2 border-b shrink-0">
          <DialogTitle className="flex items-center gap-2">
            <Tags className="h-5 w-5" />
            編輯分類
          </DialogTitle>
          <DialogDescription>
            左側為既有品類與子類，點選後於右側編輯；右側下方可新增子類，既有與新增一目了然。
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
                既有分類
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
                          aria-label={expanded ? '收合' : '展開'}
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
                            <span className="text-xs text-muted-foreground ml-auto">停用</span>
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
                                <span className="text-xs text-muted-foreground ml-auto">停用</span>
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
                    編輯選取項目
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
                          <Label className="text-muted-foreground">代碼</Label>
                          <div className="font-mono text-sm py-2">{displayCode}</div>
                        </div>
                        <div className="space-y-2 col-span-2 sm:col-span-1">
                          <Label className="text-muted-foreground">名稱</Label>
                          <Input
                            value={formName}
                            onChange={e => setFormName(e.target.value)}
                            placeholder="顯示名稱"
                          />
                        </div>
                        <div className="space-y-2">
                          <Label className="text-muted-foreground">排序</Label>
                          <Input
                            type="number"
                            min={0}
                            max={9999}
                            value={formSortOrder}
                            onChange={e => setFormSortOrder(parseInt(e.target.value, 10) || 0)}
                          />
                        </div>
                        <div className="flex items-center gap-2 space-y-2">
                          <Label className="text-muted-foreground">啟用</Label>
                          <Switch checked={formIsActive} onCheckedChange={setFormIsActive} />
                        </div>
                      </div>
                      <Button onClick={handleSave} disabled={saving}>
                        {saving ? (
                          <>
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                            儲存中…
                          </>
                        ) : (
                          '儲存'
                        )}
                      </Button>
                    </div>
                  ) : (
                    <p className="text-sm text-muted-foreground py-2">
                      請從左側點選一個品類或子類以編輯。
                    </p>
                  )}
                </section>

                {/* 下區：新增子類 */}
                <section className="space-y-3">
                  <h3 className="text-sm font-medium text-muted-foreground flex items-center gap-2">
                    <Plus className="h-4 w-4" />
                    新增子類
                  </h3>
                  <div className="rounded-lg border border-dashed p-4 bg-muted/20 space-y-4">
                    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                      <div className="space-y-2">
                        <Label className="text-muted-foreground">所屬品類</Label>
                        <Select
                          value={newSubCategoryCode || selectedCategoryCode}
                          onValueChange={v => {
                            setNewSubCategoryCode(v)
                            if (!selectedCategoryCode) setSelectedCategoryCode(v)
                          }}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder="選擇品類" />
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
                        <Label className="text-muted-foreground">子類代碼（3 碼）</Label>
                        <Input
                          value={newSubCode}
                          onChange={e => setNewSubCode(e.target.value.toUpperCase().slice(0, 3))}
                          placeholder="例如 OTH"
                          maxLength={3}
                          className="font-mono"
                        />
                      </div>
                      <div className="space-y-2">
                        <Label className="text-muted-foreground">名稱</Label>
                        <Input
                          value={newSubName}
                          onChange={e => setNewSubName(e.target.value)}
                          placeholder="顯示名稱"
                        />
                      </div>
                      <div className="space-y-2">
                        <Label className="text-muted-foreground">排序</Label>
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
                        <Label className="text-muted-foreground">啟用</Label>
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
                        建立子類
                      </Button>
                    </div>
                  </div>
                </section>
              </div>
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
