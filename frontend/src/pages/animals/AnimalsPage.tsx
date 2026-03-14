import { useEffect, useState, useMemo } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { useSearchParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { useAuthStore } from '@/stores/auth'
import { useDebounce } from '@/hooks/useDebounce'
import { Button } from '@/components/ui/button'
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
import { useAnimalsMutations } from './hooks/useAnimalsMutations'
import { useAnimalsQueries } from './hooks/useAnimalsQueries'

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

  const {
    animals, allAnimals, sourcesData, groupedData, isLoading, groupedLoading,
    totalPages, totalAnimals, penViewGroupedData, penViewAnimals, statusCounts, penAnimalsCount,
  } = useAnimalsQueries({ statusFilter, breedFilter, appliedSearch, page, perPage })

  const hasPenSearch = statusFilter === 'pen' && (!!(appliedSearch ?? '').trim() || (breedFilter && breedFilter !== 'all'))

  // ─── Mutations ─────────────────────────────────────────────────────────────
  const { createAnimalMutation, batchAssignMutation, quickMoveMutation, quickAddMutation, forceCreateMutation } = useAnimalsMutations({
    penZone, penNumber,
    selectedAnimals, assignIacucNo,
    newAnimal, quickAddPending, quickAddForm,
    setQuickAddForm,
    setShowAddDialog, setShowBatchAssignDialog, setShowQuickAddDialog, setShowDuplicateWarning,
    setSelectedAnimals, setAssignIacucNo, setQuickAddPending, setDuplicateWarningData,
    setQuickEditAnimalId,
    resetNewAnimalForm,
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
        penAnimalsCount={penAnimalsCount}
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
