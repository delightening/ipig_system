import { useEffect, useMemo, useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useSearchParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { useAuthStore } from '@/stores/auth'
import { useDebounce } from '@/hooks/useDebounce'
import { STALE_TIME } from '@/lib/query'
import axios from 'axios'
import api, {
  Animal,
  AnimalListItem,
  AnimalSource,
  CreateAnimalRequest,
} from '@/lib/api'
import type { PaginatedResponse } from '@/types/common'
import { getApiErrorMessage } from '@/lib/validation'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import { Plus, Upload, Download, FileSpreadsheet } from 'lucide-react'

import { ExportDialog } from '@/components/animal/ExportDialog'
import { ImportDialog } from '@/components/animal/ImportDialog'
import { QuickEditAnimalDialog } from '@/components/animal/QuickEditAnimalDialog'
import { AnimalPenReport } from '../../components/animal/AnimalPenReport'

import { AnimalFilters } from './components/AnimalFilters'
import { AnimalListTable } from './components/AnimalListTable'
import { AnimalPenView } from './components/AnimalPenView'
import {
  AnimalAddDialog,
  BatchAssignDialog,
  QuickAddDialog,
  DuplicateWarningDialog,
} from './components/AnimalAddDialog'
import type { NewAnimalForm } from './components/AnimalAddDialog'
import {
  useAnimalFilters,
  useAnimalDialogs,
  useAnimalSelection,
  useAnimalForms,
} from './hooks/useAnimalsPageState'

export function AnimalsPage() {
  const queryClient = useQueryClient()
  const { hasRole } = useAuthStore()
  const { t } = useTranslation()

  const isPIOrClient = hasRole('PI') || hasRole('CLIENT')
  const isAdmin = hasRole('admin')
  const adminOnlyStatuses = useMemo(() => ['euthanized', 'sudden_death', 'transferred'], [])

  const allowedStatuses = useMemo(
    () =>
      isPIOrClient
        ? ['in_experiment', 'completed', ...(isAdmin ? adminOnlyStatuses : [])]
        : ['pen', 'unassigned', 'in_experiment', 'completed', ...(isAdmin ? adminOnlyStatuses : []), 'all'],
    [isPIOrClient, isAdmin, adminOnlyStatuses]
  )
  const defaultStatus = isPIOrClient ? 'in_experiment' : 'pen'

  const filters = useAnimalFilters({ allowedStatuses, defaultStatus })
  const { search, setSearch, statusFilter, setStatusFilter, breedFilter, setBreedFilter, page, setPage, sortColumn, setSortColumn, sortDirection, setSortDirection, setSearchParams } = filters
  const debouncedSearch = useDebounce(search, 400)
  const [appliedSearch, setAppliedSearch] = useState('')
  // 僅在 debounce 已跟上輸入時同步，避免使用者按 Enter/搜尋後被舊的 debouncedSearch 覆寫
  useEffect(() => {
    if (debouncedSearch === search) {
      setAppliedSearch(debouncedSearch)
    }
  }, [debouncedSearch, search])
  const handleSearchSubmit = () => {
    setAppliedSearch(search)
    setPage(1)
    queryClient.invalidateQueries({ queryKey: ['animals'] })
  }

  const dialogs = useAnimalDialogs()
  const selection = useAnimalSelection()
  const forms = useAnimalForms()
  const {
    showAddDialog,
    setShowAddDialog,
    showBatchAssignDialog,
    setShowBatchAssignDialog,
    showBatchExportDialog,
    setShowBatchExportDialog,
    showImportBasicDialog,
    setShowImportBasicDialog,
    showImportWeightDialog,
    setShowImportWeightDialog,
    showPrintReport,
    setShowPrintReport,
    showDuplicateWarning,
    setShowDuplicateWarning,
    showQuickAddDialog,
    setShowQuickAddDialog,
    duplicateWarningData,
    setDuplicateWarningData,
  } = dialogs
  const {
    selectedAnimals,
    setSelectedAnimals,
    assignIacucNo,
    setAssignIacucNo,
    quickEditAnimalId,
    setQuickEditAnimalId,
  } = selection
  const {
    newAnimal,
    setNewAnimal,
    quickAddPending,
    setQuickAddPending,
    quickAddForm,
    setQuickAddForm,
    penBuilding,
    setPenBuilding,
    penZone,
    setPenZone,
    penNumber,
    setPenNumber,
    resetNewAnimalForm,
  } = forms

  useEffect(() => { setPage(1) }, [statusFilter, breedFilter, appliedSearch, setPage])

  const perPage = 50

  // ─── Queries ───────────────────────────────────────────────────────────────
  const { data: allAnimalsResp } = useQuery({
    queryKey: ['animals-count'],
    queryFn: async () => {
      const res = await api.get<PaginatedResponse<AnimalListItem>>(`/animals`)
      return res.data
    },
    staleTime: STALE_TIME.LIST,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: animalsResp, isLoading } = useQuery({
    queryKey: ['animals', statusFilter, breedFilter, appliedSearch, page],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (statusFilter && statusFilter !== 'all' && statusFilter !== 'pen') params.append('status', statusFilter)
      if (breedFilter && breedFilter !== 'all') params.append('breed', breedFilter)
      if (appliedSearch) params.append('keyword', appliedSearch)
      params.append('page', String(page))
      params.append('per_page', String(perPage))
      const res = await api.get<PaginatedResponse<AnimalListItem>>(`/animals?${params}`)
      return res.data
    },
    staleTime: STALE_TIME.LIST,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: sourcesData } = useQuery({
    queryKey: ['animal-sources'],
    queryFn: async () => {
      const res = await api.get<AnimalSource[]>('/animal-sources')
      return res.data
    },
    staleTime: STALE_TIME.REFERENCE,
  })

  const { data: groupedData, isLoading: groupedLoading } = useQuery({
    queryKey: ['animals-by-pen'],
    queryFn: async () => {
      const res = await api.get<{ pen_location: string; animals: AnimalListItem[] }[]>('/animals/by-pen')
      return res.data
    },
    enabled: statusFilter === 'pen',
    staleTime: STALE_TIME.REALTIME,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  // ─── Derived data ──────────────────────────────────────────────────────────
  const animals = animalsResp?.data ?? []
  const allAnimals = allAnimalsResp?.data ?? []
  const totalPages = animalsResp?.total_pages ?? 1
  const totalAnimals = animalsResp?.total ?? 0

  /** 欄位頁：搜尋／品種僅套用在「有欄位」的動物（by-pen 資料），不包含無欄位歷史動物 */
  const penViewGroupedData = useMemo(() => {
    if (statusFilter !== 'pen' || !groupedData) return groupedData
    const kw = (appliedSearch ?? '').trim().toLowerCase()
    const hasKeyword = kw.length > 0
    const breedOk = (a: AnimalListItem) => breedFilter === 'all' || (a.breed && String(a.breed).toLowerCase() === breedFilter.toLowerCase())
    const keywordOk = (a: AnimalListItem) =>
      !hasKeyword ||
      [a.ear_tag, a.pen_location, a.iacuc_no].some(
        (v) => v && String(v).toLowerCase().includes(kw)
      )
    return groupedData
      .map((group) => ({
        pen_location: group.pen_location,
        animals: group.animals.filter((a) => breedOk(a) && keywordOk(a)),
      }))
      .filter((group) => group.animals.length > 0)
  }, [statusFilter, groupedData, appliedSearch, breedFilter])

  /** 欄位頁有搜尋/品種時：攤平為列表，用於顯示表格（圖二） */
  const penViewAnimals = useMemo(
    () => (statusFilter === 'pen' && penViewGroupedData ? penViewGroupedData.flatMap((g) => g.animals) : []),
    [statusFilter, penViewGroupedData]
  )
  const hasPenSearch = statusFilter === 'pen' && (!!(appliedSearch ?? '').trim() || (breedFilter && breedFilter !== 'all'))

  const statusCounts = allAnimals.reduce((acc, animal) => {
    acc[animal.status] = (acc[animal.status] || 0) + 1
    return acc
  }, {} as Record<string, number>)

  // ─── Helpers ───────────────────────────────────────────────────────────────
  const invalidateAll = () => {
    queryClient.invalidateQueries({ queryKey: ['animals'] })
    queryClient.invalidateQueries({ queryKey: ['animals-by-pen'] })
    queryClient.invalidateQueries({ queryKey: ['animals-count'] })
  }

  const extractErrorMessage = (error: unknown, fallback: string) => {
    return getApiErrorMessage(error, fallback)
  }

  const handleDuplicate409 = (error: unknown, source: 'create' | 'quickAdd') => {
    if (!axios.isAxiosError(error) || error.response?.status !== 409) return false
    const errData = error.response.data?.error
    if (errData?.warning_type !== 'duplicate_ear_tag' || errData?.blocking !== false) return false
    let payload: Record<string, unknown> = {}
    try { payload = JSON.parse(error.config?.data || '{}') } catch { /* ignore */ }
    setDuplicateWarningData({
      earTag: (payload.ear_tag as string) || quickAddPending?.earTag || '',
      existingAnimals: errData.existing_animals || [],
      source,
      pendingPayload: payload as unknown as CreateAnimalRequest & { breed_other?: string },
    })
    setShowDuplicateWarning(true)
    return true
  }

  // ─── Mutations ─────────────────────────────────────────────────────────────
  const createAnimalMutation = useMutation({
    mutationFn: async (data: NewAnimalForm) => {
      if (!penZone || !penNumber) throw new Error('欄位為必填，請選擇欄位區和欄位編號')
      const penLocation = `${penZone}${penNumber}`

      if (!data.ear_tag?.trim()) throw new Error('耳號為必填')
      if (!data.entry_date) throw new Error('進場日期為必填')
      if (!data.birth_date) throw new Error('出生日期為必填')
      if (!data.pre_experiment_code?.trim()) throw new Error('實驗前代號為必填')

      let entryWeight: number | undefined
      if (data.entry_weight && data.entry_weight !== '') {
        const weightValue = parseFloat(data.entry_weight)
        if (isNaN(weightValue) || weightValue <= 0) throw new Error('進場體重必須是大於 0 的數字')
        entryWeight = weightValue
      }

      const dateRegex = /^\d{4}-\d{2}-\d{2}$/
      if (!dateRegex.test(data.entry_date)) throw new Error('進場日期格式不正確，必須是 YYYY-MM-DD 格式')
      if (data.birth_date && data.birth_date.trim() !== '' && !dateRegex.test(data.birth_date))
        throw new Error('出生日期格式不正確，必須是 YYYY-MM-DD 格式')

      let formattedEarTag = data.ear_tag.trim()
      if (/^\d+$/.test(formattedEarTag)) formattedEarTag = formattedEarTag.padStart(3, '0')

      const payload: CreateAnimalRequest & { breed_other?: string } = {
        ear_tag: formattedEarTag,
        breed: data.breed,
        gender: data.gender,
        entry_date: data.entry_date,
        birth_date: data.birth_date && data.birth_date.trim() !== '' ? data.birth_date.trim() : undefined,
        entry_weight: entryWeight,
        pen_location: penLocation,
        pre_experiment_code: data.pre_experiment_code?.trim() || undefined,
        remark: data.remark?.trim() || undefined,
        breed_other: data.breed === 'other' ? data.breed_other : undefined,
      }

      if (data.source_id && data.source_id.trim() !== '' && data.source_id.trim() !== 'none') {
        const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
        const trimmedSourceId = data.source_id.trim()
        if (uuidRegex.test(trimmedSourceId)) payload.source_id = trimmedSourceId
        else throw new Error(`來源 ID 格式不正確: ${trimmedSourceId}`)
      }

      return api.post('/animals', payload)
    },
    onSuccess: () => {
      invalidateAll()
      toast({ title: '成功', description: '動物已新增' })
      setShowAddDialog(false)
      resetNewAnimalForm()
    },
    onError: (error: unknown) => {
      if (handleDuplicate409(error, 'create')) return

      let errorMessage = '新增失敗，請檢查輸入資料'
      if (axios.isAxiosError(error) && error.response?.status === 422) {
        const data = error.response.data as { error?: { message?: string }; message?: string } | undefined
        errorMessage = data?.error?.message || data?.message || '資料格式錯誤：請檢查所有欄位的格式是否正確'
      } else {
        errorMessage = extractErrorMessage(error, errorMessage)
      }
      toast({ title: '錯誤', description: errorMessage, variant: 'destructive' })
    },
  })

  const batchAssignMutation = useMutation({
    mutationFn: () => api.post('/animals/batch/assign', { animal_ids: selectedAnimals, iacuc_no: assignIacucNo }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      toast({ title: '成功', description: '動物已分配至計劃並進入實驗中' })
      setShowBatchAssignDialog(false)
      setSelectedAnimals([])
      setAssignIacucNo('')
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: extractErrorMessage(error, '批次分配失敗'), variant: 'destructive' })
    },
  })

  const quickMoveMutation = useMutation({
    mutationFn: async ({ earTag, targetPenLocation }: { earTag: string; targetPenLocation: string }) => {
      let formattedEarTag = earTag.trim()
      if (/^\d+$/.test(formattedEarTag)) formattedEarTag = formattedEarTag.padStart(3, '0')

      const searchRes = await api.get<PaginatedResponse<AnimalListItem>>(`/animals?keyword=${encodeURIComponent(formattedEarTag)}`)
      const matchingAnimals = (searchRes.data.data ?? []).filter(p => p.ear_tag === formattedEarTag)

      if (matchingAnimals.length === 0) return { notFound: true, formattedEarTag, targetPenLocation }
      if (matchingAnimals.length > 1) throw new Error(`找到多隻耳號為 "${formattedEarTag}" 的動物，請使用編輯功能手動移動`)

      const animal = matchingAnimals[0]
      if (animal.pen_location === targetPenLocation) throw new Error(`動物 ${formattedEarTag} 已經在 ${targetPenLocation} 欄位`)

      return { ...await api.put<Animal>(`/animals/${animal.id}`, { pen_location: targetPenLocation }), notFound: false }
    },
    onSuccess: (data: { notFound?: boolean; formattedEarTag?: string; targetPenLocation?: string }, variables: { earTag: string; targetPenLocation: string }) => {
      if (data.notFound && data.formattedEarTag != null && data.targetPenLocation != null) {
        setQuickAddPending({ earTag: data.formattedEarTag, penLocation: data.targetPenLocation })
        setQuickAddForm({
          breed: 'minipig', breed_other: '', gender: 'male',
          entry_date: new Date().toISOString().split('T')[0], birth_date: '', entry_weight: '',
        })
        setShowQuickAddDialog(true)
        return
      }
      invalidateAll()
      let formattedEarTag = variables.earTag.trim()
      if (/^\d+$/.test(formattedEarTag)) formattedEarTag = formattedEarTag.padStart(3, '0')
      toast({ title: '成功', description: `動物 ${formattedEarTag} 已移動到 ${variables.targetPenLocation}` })
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: extractErrorMessage(error, '移動失敗'), variant: 'destructive' })
    },
  })

  const quickAddMutation = useMutation({
    mutationFn: async () => {
      if (!quickAddPending) throw new Error('無待處理的新增請求')
      if (!quickAddForm.entry_date) throw new Error('進場日期為必填')
      if (!quickAddForm.birth_date) throw new Error('出生日期為必填')

      return api.post<Animal>('/animals', {
        ear_tag: quickAddPending.earTag,
        breed: quickAddForm.breed,
        breed_other: quickAddForm.breed === 'other' ? quickAddForm.breed_other : undefined,
        gender: quickAddForm.gender,
        entry_date: quickAddForm.entry_date,
        birth_date: quickAddForm.birth_date,
        entry_weight: parseFloat(quickAddForm.entry_weight),
        pen_location: quickAddPending.penLocation,
      })
    },
    onSuccess: () => {
      invalidateAll()
      toast({ title: '成功', description: `已新增動物 ${quickAddPending?.earTag} 至 ${quickAddPending?.penLocation}` })
      setShowQuickAddDialog(false)
      setQuickAddPending(null)
    },
    onError: (error: unknown) => {
      if (handleDuplicate409(error, 'quickAdd')) return
      toast({ title: '錯誤', description: extractErrorMessage(error, '新增失敗'), variant: 'destructive' })
    },
  })

  const forceCreateMutation = useMutation({
    mutationFn: (payload: CreateAnimalRequest & { breed_other?: string }) => api.post('/animals', { ...payload, force_create: true }),
    onSuccess: () => {
      invalidateAll()
      toast({ title: '成功', description: '動物已新增（已確認耳號重複）' })
      setShowDuplicateWarning(false)
      setDuplicateWarningData(null)
      setShowAddDialog(false)
      setShowQuickAddDialog(false)
      setQuickAddPending(null)
      resetNewAnimalForm()
    },
    onError: (error: unknown) => {
      toast({ title: '錯誤', description: extractErrorMessage(error, '新增失敗'), variant: 'destructive' })
    },
  })

  // ─── Handlers ──────────────────────────────────────────────────────────────
  const handleStatusFilterChange = (value: string) => {
    setStatusFilter(value)
    setSearchParams(value === 'pen' ? {} : { status: value })
  }

  const handleSort = (column: string) => {
    if (sortColumn === column) setSortDirection(prev => prev === 'asc' ? 'desc' : 'asc')
    else { setSortColumn(column); setSortDirection('asc') }
  }

  const toggleAnimalSelection = (id: string) => {
    setSelectedAnimals(prev => prev.includes(id) ? prev.filter(p => p !== id) : [...prev, id])
  }

  const toggleAllAnimals = () => {
    if (animals.length === 0) return
    setSelectedAnimals(selectedAnimals.length === animals.length ? [] : animals.map(p => p.id))
  }

  // ─── Render ────────────────────────────────────────────────────────────────
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-start justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">{t('animals.title')}</h1>
          <p className="text-muted-foreground">{t('animals.description')}</p>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
          <Button variant="outline" className="w-full gap-2 text-orange-600 border-orange-200 hover:bg-orange-50 text-xs md:text-sm" onClick={() => setShowPrintReport(true)}>
            <Download className="h-4 w-4 shrink-0" />
            <span className="truncate">{t('animals.generateReport')}</span>
          </Button>
          <Button variant="outline" className="w-full gap-2 text-xs md:text-sm" onClick={() => setShowImportWeightDialog(true)}>
            <Upload className="h-4 w-4 shrink-0" />
            <span className="truncate">{t('animals.importWeight')}</span>
          </Button>
          <Button variant="outline" className="w-full gap-2 text-xs md:text-sm" onClick={() => setShowImportBasicDialog(true)}>
            <Upload className="h-4 w-4 shrink-0" />
            <span className="truncate">{t('animals.importBasic')}</span>
          </Button>
          <Button variant="outline" className="w-full gap-2 text-xs md:text-sm" onClick={() => setShowBatchExportDialog(true)}>
            <FileSpreadsheet className="h-4 w-4 shrink-0" />
            <span className="truncate">{t('animals.batchExport')}</span>
          </Button>
          <Button onClick={() => setShowAddDialog(true)} className="col-span-2 md:col-span-2 w-full gap-2 bg-purple-600 hover:bg-purple-700 text-xs md:text-sm">
            <Plus className="h-4 w-4 shrink-0" />
            {t('animals.addAnimal')}
          </Button>
        </div>
      </div>

      {/* Filters & Tabs */}
      <AnimalFilters
        statusFilter={statusFilter}
        onStatusFilterChange={handleStatusFilterChange}
        breedFilter={breedFilter}
        onBreedFilterChange={setBreedFilter}
        search={search}
        onSearchChange={setSearch}
        onSearchSubmit={handleSearchSubmit}
        allowedStatuses={allowedStatuses}
        adminOnlyStatuses={adminOnlyStatuses}
        isPIOrClient={isPIOrClient}
        isAdmin={isAdmin}
        statusCounts={statusCounts}
        allAnimalsCount={allAnimals.length}
        selectedAnimalsCount={selectedAnimals.length}
        onShowBatchAssign={() => setShowBatchAssignDialog(true)}
      />

      {/* List View（未分配／實驗中／所有動物等，或 欄位＋搜尋時改顯示表格） */}
      {(statusFilter !== 'pen' || hasPenSearch) && (
        <AnimalListTable
          animals={hasPenSearch ? penViewAnimals : animals}
          isLoading={hasPenSearch ? groupedLoading : isLoading}
          selectedAnimals={selectedAnimals}
          onToggleSelection={toggleAnimalSelection}
          onToggleAll={toggleAllAnimals}
          onQuickEdit={setQuickEditAnimalId}
          sortColumn={sortColumn}
          sortDirection={sortDirection}
          onSort={handleSort}
          page={hasPenSearch ? 1 : page}
          totalPages={hasPenSearch ? 1 : totalPages}
          totalAnimals={hasPenSearch ? penViewAnimals.length : totalAnimals}
          perPage={hasPenSearch ? Math.max(perPage, penViewAnimals.length) || 50 : perPage}
          onPageChange={hasPenSearch ? () => {} : setPage}
        />
      )}

      {/* Pen View：無搜尋時顯示欄位格線圖（圖一）；有搜尋時改由上方表格顯示（圖二） */}
      {statusFilter === 'pen' && !hasPenSearch && (
        <AnimalPenView
          groupedData={penViewGroupedData}
          isLoading={groupedLoading}
          onQuickMove={(earTag, target) => quickMoveMutation.mutate({ earTag, targetPenLocation: target })}
          isQuickMovePending={quickMoveMutation.isPending}
        />
      )}

      {/* ── Dialogs ─────────────────────────────────────────────────────────── */}
      <AnimalAddDialog
        open={showAddDialog}
        onOpenChange={setShowAddDialog}
        newAnimal={newAnimal}
        onNewAnimalChange={setNewAnimal}
        penBuilding={penBuilding}
        onPenBuildingChange={setPenBuilding}
        penZone={penZone}
        onPenZoneChange={setPenZone}
        penNumber={penNumber}
        onPenNumberChange={setPenNumber}
        sourcesData={sourcesData}
        onSubmit={() => createAnimalMutation.mutate(newAnimal)}
        isPending={createAnimalMutation.isPending}
      />

      <BatchAssignDialog
        open={showBatchAssignDialog}
        onOpenChange={setShowBatchAssignDialog}
        selectedCount={selectedAnimals.length}
        iacucNo={assignIacucNo}
        onIacucNoChange={setAssignIacucNo}
        onSubmit={() => batchAssignMutation.mutate()}
        isPending={batchAssignMutation.isPending}
      />

      <ExportDialog open={showBatchExportDialog} onOpenChange={setShowBatchExportDialog} type="batch_project" />
      <ImportDialog open={showImportBasicDialog} onOpenChange={setShowImportBasicDialog} type="basic" />
      <ImportDialog open={showImportWeightDialog} onOpenChange={setShowImportWeightDialog} type="weight" />

      {quickEditAnimalId && (
        <QuickEditAnimalDialog
          open={!!quickEditAnimalId}
          onOpenChange={(open) => { if (!open) setQuickEditAnimalId(null) }}
          animalId={quickEditAnimalId}
        />
      )}

      <QuickAddDialog
        open={showQuickAddDialog}
        onOpenChange={(open) => {
          if (!open) { setShowQuickAddDialog(false); setQuickAddPending(null) }
        }}
        earTag={quickAddPending?.earTag ?? ''}
        penLocation={quickAddPending?.penLocation ?? ''}
        form={quickAddForm}
        onFormChange={setQuickAddForm}
        onSubmit={() => quickAddMutation.mutate()}
        isPending={quickAddMutation.isPending}
      />

      <DuplicateWarningDialog
        open={showDuplicateWarning}
        onOpenChange={(open) => {
          if (!open) { setShowDuplicateWarning(false); setDuplicateWarningData(null) }
        }}
        data={duplicateWarningData}
        onConfirm={(payload) => forceCreateMutation.mutate(payload)}
        isPending={forceCreateMutation.isPending}
      />

      {showPrintReport && (
        <AnimalPenReport data={groupedData || []} onClose={() => setShowPrintReport(false)} />
      )}
    </div>
  )
}
