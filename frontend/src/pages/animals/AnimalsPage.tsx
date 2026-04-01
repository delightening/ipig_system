import { useEffect, useState, useMemo } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { useAuthStore } from '@/stores/auth'
import { useDebounce } from '@/hooks/useDebounce'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
import { Plus, Upload, Download, FileSpreadsheet } from 'lucide-react'

import { GuestHide } from '@/components/ui/guest-hide'
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
import {
  useAnimalFilters,
  useAnimalDialogs,
  useAnimalSelection,
  useAnimalForms,
} from './hooks/useAnimalsPageState'
import { useAnimalsMutations } from './hooks/useAnimalsMutations'
import { useAnimalsQueries } from './hooks/useAnimalsQueries'

const ADMIN_ONLY_STATUSES = ['euthanized', 'sudden_death', 'transferred']

export function AnimalsPage() {
  const queryClient = useQueryClient()
  const { hasRole } = useAuthStore()
  const { t } = useTranslation()

  const isPIOrClient = hasRole('PI') || hasRole('CLIENT')
  const isAdmin = hasRole('admin')
  const adminOnlyStatuses = ADMIN_ONLY_STATUSES

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
    animals, sourcesData, groupedData, isLoading, groupedLoading,
    totalPages, totalAnimals, penViewGroupedData, penViewAnimals,
    statusCounts, penAnimalsCount, allAnimalsCount,
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
      <PageHeader
        title={t('animals.title')}
        description={t('animals.description')}
        actions={
          <GuestHide>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
              <Button size="sm" variant="outline" className="w-full gap-2 text-status-warning-text border-status-warning-border hover:bg-status-warning-bg text-xs md:text-sm" onClick={() => setShowPrintReport(true)}>
                <Download className="h-4 w-4 shrink-0" />
                <span className="truncate">{t('animals.generateReport')}</span>
              </Button>
              <Button size="sm" variant="outline" className="w-full gap-2 text-xs md:text-sm" onClick={() => setShowImportWeightDialog(true)}>
                <Upload className="h-4 w-4 shrink-0" />
                <span className="truncate">{t('animals.importWeight')}</span>
              </Button>
              <Button size="sm" variant="outline" className="w-full gap-2 text-xs md:text-sm" onClick={() => setShowImportBasicDialog(true)}>
                <Upload className="h-4 w-4 shrink-0" />
                <span className="truncate">{t('animals.importBasic')}</span>
              </Button>
              <Button size="sm" variant="outline" className="w-full gap-2 text-xs md:text-sm" onClick={() => setShowBatchExportDialog(true)}>
                <FileSpreadsheet className="h-4 w-4 shrink-0" />
                <span className="truncate">{t('animals.batchExport')}</span>
              </Button>
              <Button size="sm" onClick={() => setShowAddDialog(true)} className="col-span-2 md:col-span-2 w-full gap-2 bg-primary hover:bg-primary/90 text-xs md:text-sm">
                <Plus className="h-4 w-4 shrink-0" />
                {t('animals.addAnimal')}
              </Button>
            </div>
          </GuestHide>
        }
      />

      {/* Filters & Tabs */}
      <AnimalFilters
        filters={{
          statusFilter,
          onStatusFilterChange: handleStatusFilterChange,
          breedFilter,
          onBreedFilterChange: setBreedFilter,
          search,
          onSearchChange: setSearch,
          onSearchSubmit: handleSearchSubmit,
        }}
        counts={{
          statusCounts,
          allAnimalsCount,
          penAnimalsCount,
          selectedAnimalsCount: selectedAnimals.length,
        }}
        adminOnlyStatuses={adminOnlyStatuses}
        isPIOrClient={isPIOrClient}
        isAdmin={isAdmin}
        onShowBatchAssign={() => setShowBatchAssignDialog(true)}
      />

      {/* List View（未分配／實驗中／所有動物等，或 欄位＋搜尋時改顯示表格） */}
      {(statusFilter !== 'pen' || hasPenSearch) && (
        <AnimalListTable
          animals={hasPenSearch ? penViewAnimals : animals}
          isLoading={hasPenSearch ? groupedLoading : isLoading}
          onQuickEdit={setQuickEditAnimalId}
          selection={{
            selectedAnimals,
            onToggleSelection: toggleAnimalSelection,
            onToggleAll: toggleAllAnimals,
          }}
          sorting={{
            sortColumn,
            sortDirection,
            onSort: handleSort,
          }}
          pagination={{
            page: hasPenSearch ? 1 : page,
            totalPages: hasPenSearch ? 1 : totalPages,
            totalAnimals: hasPenSearch ? penViewAnimals.length : totalAnimals,
            perPage: hasPenSearch ? Math.max(perPage, penViewAnimals.length) || 50 : perPage,
            onPageChange: hasPenSearch ? () => {} : setPage,
          }}
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
