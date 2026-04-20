/**
 * 血液檢查常用組合管理頁面
 * 供血液檢查結果分析頁面一鍵選取使用
 */
import { useState, useMemo } from 'react'
import { STALE_TIME } from '@/lib/query'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import {
  bloodTestPresetApi,
  bloodTestPanelApi,
  BloodTestPreset,
  UpdateBloodTestPresetRequest,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
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
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { toast } from '@/components/ui/use-toast'
import {
  Plus,
  Search,
  Edit,
  Power,
  PowerOff,
  Loader2,
  ArrowLeft,
  Layers,
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { getApiErrorMessage, bloodTestPresetFormSchema, type BloodTestPresetFormData } from '@/lib/validation'
import { PanelIcon } from '@/components/ui/panel-icon'
import { useNavigate } from 'react-router-dom'
import { TableSkeleton } from '@/components/ui/table-skeleton'
import { TableEmptyRow } from '@/components/ui/empty-state'

type ShowFilter = 'all' | 'active' | 'inactive'

export function BloodTestPresetsPage() {
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  const [search, setSearch] = useState('')
  const [showFilter, setShowFilter] = useState<ShowFilter>('all')
  const [dialogOpen, setDialogOpen] = useState(false)
  const [editingPreset, setEditingPreset] = useState<BloodTestPreset | null>(null)
  const { register, handleSubmit: rhfHandleSubmit, reset, setValue, watch, formState: { errors } } = useForm<BloodTestPresetFormData>({
    resolver: zodResolver(bloodTestPresetFormSchema),
    defaultValues: { name: '', icon: '', panel_keys: [], sort_order: 0 },
  })
  const panelKeysValue = watch('panel_keys')

  const { data: presets, isLoading } = useQuery({
    queryKey: ['blood-test-presets-all'],
    staleTime: STALE_TIME.REFERENCE,
    queryFn: async () => {
      const res = await bloodTestPresetApi.listAll()
      return res.data
    },
  })

  const { data: panels } = useQuery({
    queryKey: ['blood-test-panels-all'],
    staleTime: STALE_TIME.REFERENCE,
    queryFn: async () => {
      const res = await bloodTestPanelApi.listAll()
      return res.data
    },
  })

  const createMutation = useMutation({
    mutationFn: (data: BloodTestPresetFormData) =>
      bloodTestPresetApi.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['blood-test-presets'] })
      queryClient.invalidateQueries({ queryKey: ['blood-test-presets-all'] })
      toast({ title: '成功', description: '常用組合已建立' })
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
      data: UpdateBloodTestPresetRequest
    }) => bloodTestPresetApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['blood-test-presets'] })
      queryClient.invalidateQueries({ queryKey: ['blood-test-presets-all'] })
      toast({ title: '成功', description: '常用組合已更新' })
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
      bloodTestPresetApi.update(id, { is_active }),
    onSuccess: (_, vars) => {
      queryClient.invalidateQueries({ queryKey: ['blood-test-presets'] })
      queryClient.invalidateQueries({ queryKey: ['blood-test-presets-all'] })
      toast({
        title: '成功',
        description: vars.is_active ? '常用組合已恢復啟用' : '常用組合已停用',
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

  const resetForm = () => {
    reset({ name: '', icon: '', panel_keys: [], sort_order: 0 })
    setEditingPreset(null)
  }

  const handleEdit = (preset: BloodTestPreset) => {
    setEditingPreset(preset)
    reset({
      name: preset.name,
      icon: preset.icon || '',
      panel_keys: preset.panel_keys || [],
      sort_order: preset.sort_order,
    })
    setDialogOpen(true)
  }

  const onSubmit = (data: BloodTestPresetFormData) => {
    if (editingPreset) {
      updateMutation.mutate({
        id: editingPreset.id,
        data: {
          name: data.name,
          icon: data.icon || undefined,
          panel_keys: data.panel_keys,
          sort_order: data.sort_order,
        },
      })
    } else {
      createMutation.mutate(data)
    }
  }

  const togglePanelKey = (key: string) => {
    const current = panelKeysValue || []
    const next = current.includes(key)
      ? current.filter((k) => k !== key)
      : [...current, key]
    setValue('panel_keys', next)
  }

  const filteredPresets = useMemo(() => {
    if (!presets) return []
    let result = presets
    if (search) {
      const q = search.toLowerCase()
      result = result.filter((p) => p.name.toLowerCase().includes(q))
    }
    if (showFilter === 'active') {
      result = result.filter((p) => p.is_active)
    } else if (showFilter === 'inactive') {
      result = result.filter((p) => !p.is_active)
    }
    result.sort((a, b) => a.sort_order - b.sort_order)
    return result
  }, [presets, search, showFilter])

  const { sortedData, sort, toggleSort } = useTableSort(filteredPresets)

  const totalCount = presets?.length ?? 0
  const activeCount = presets?.filter((p) => p.is_active).length ?? 0
  const activePanels = panels?.filter((p) => p.is_active) ?? []
  const isSaving = createMutation.isPending || updateMutation.isPending

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => navigate('/blood-test-templates')}
          aria-label="返回"
        >
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <PageHeader
          title="血液檢查常用組合管理"
          description={`管理血液檢查結果分析頁面的一鍵選取組合（共 ${totalCount} 個，啟用 ${activeCount} 個）`}
          className="flex-1"
          actions={
            <Button
              size="sm"
              onClick={() => {
                resetForm()
                setDialogOpen(true)
              }}
            >
              <Plus className="mr-2 h-4 w-4" />
              新增常用組合
            </Button>
          }
        />
      </div>

      <div className="flex gap-4 items-center">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜尋名稱..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>
        <div className="flex gap-1">
          {(['all', 'active', 'inactive'] as ShowFilter[]).map((f) => (
            <Button
              key={f}
              variant={showFilter === f ? 'default' : 'outline'}
              size="sm"
              onClick={() => setShowFilter(f)}
            >
              {f === 'all' ? '全部' : f === 'active' ? '啟用中' : '已停用'}
            </Button>
          ))}
        </div>
      </div>

      <div className="rounded-lg border bg-card overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-muted/50 hover:bg-muted/50">
              <TableHead className="w-[60px]">圖示</TableHead>
              <SortableTableHead sortKey="name" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>名稱</SortableTableHead>
              <TableHead>包含分類</TableHead>
              <SortableTableHead sortKey="sort_order" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="w-[80px] text-center">排序</SortableTableHead>
              <SortableTableHead sortKey="is_active" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort} className="w-[80px] text-center">狀態</SortableTableHead>
              <TableHead className="w-[140px] text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={6} className="p-0">
                  <TableSkeleton rows={5} cols={6} />
                </TableCell>
              </TableRow>
            ) : filteredPresets.length === 0 ? (
              <TableEmptyRow colSpan={6} icon={Layers} title="沒有符合條件的常用組合" />
            ) : (
              (sortedData ?? filteredPresets).map((preset) => (
                <TableRow
                  key={preset.id}
                  className={cn(!preset.is_active && 'opacity-50')}
                >
                  <TableCell>
                    <PanelIcon icon={preset.icon} size={24} />
                  </TableCell>
                  <TableCell className="font-medium">{preset.name}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">
                    {(preset.panel_keys || [])
                      .map((k) => panels?.find((p) => p.key === k)?.name ?? k)
                      .join('、') || '—'}
                  </TableCell>
                  <TableCell className="text-center">{preset.sort_order}</TableCell>
                  <TableCell className="text-center">
                    <Badge
                      variant={preset.is_active ? 'default' : 'outline'}
                    >
                      {preset.is_active ? '啟用' : '停用'}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-1">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleEdit(preset)}
                        title="編輯"
                      >
                        <Edit className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() =>
                          toggleMutation.mutate({
                            id: preset.id,
                            is_active: !preset.is_active,
                          })
                        }
                        title={preset.is_active ? '停用' : '啟用'}
                      >
                        {preset.is_active ? (
                          <PowerOff className="h-4 w-4 text-destructive" />
                        ) : (
                          <Power className="h-4 w-4 text-status-success-solid" />
                        )}
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <Dialog
        open={dialogOpen}
        onOpenChange={(open) => {
          if (!open) {
            resetForm()
            setDialogOpen(false)
          }
        }}
      >
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>
              {editingPreset ? '編輯常用組合' : '新增常用組合'}
            </DialogTitle>
            <DialogDescription>
              {editingPreset
                ? '修改組合的名稱、圖示與包含分類'
                : '請輸入常用組合的資訊，供分析頁一鍵選取'}
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={rhfHandleSubmit(onSubmit)}>
            <div className="grid gap-4 py-4">
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="preset-name" className="text-right">
                  名稱
                </Label>
                <div className="col-span-3 space-y-1">
                  <Input
                    id="preset-name"
                    {...register('name')}
                    placeholder="例：肝腎功能"
                  />
                  {errors.name && (
                    <p className="text-sm text-destructive">{errors.name.message}</p>
                  )}
                </div>
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="preset-icon" className="text-right">
                  圖示
                </Label>
                <Input
                  id="preset-icon"
                  {...register('icon')}
                  className="col-span-3"
                  placeholder="emoji 或 /icons/xxx.svg"
                />
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label className="text-right pt-2">包含分類</Label>
                <div className="col-span-3 flex flex-wrap gap-3">
                  {activePanels
                    .filter((p) => p.key !== 'TUBE')
                    .map((p) => (
                      <label
                        key={p.id}
                        className="flex items-center gap-2 cursor-pointer"
                      >
                        <Checkbox
                          checked={(panelKeysValue || []).includes(p.key)}
                          onCheckedChange={() => togglePanelKey(p.key)}
                        />
                        <PanelIcon icon={p.icon} size={16} />
                        <span className="text-sm">{p.name}</span>
                      </label>
                    ))}
                </div>
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="preset-sort" className="text-right">
                  排序
                </Label>
                <Input
                  id="preset-sort"
                  type="number"
                  {...register('sort_order', { valueAsNumber: true })}
                  className="col-span-3 w-24"
                />
              </div>
            </div>
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => setDialogOpen(false)}
              >
                取消
              </Button>
              <Button type="submit" disabled={isSaving}>
                {isSaving && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                {editingPreset ? '儲存' : '建立'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  )
}
