import { useState, useEffect } from 'react'
import { useSearchParams } from 'react-router-dom'
import type { NewAnimalForm, QuickAddForm, DuplicateWarningData } from '../components/AnimalAddDialog'

const defaultNewAnimal: NewAnimalForm = {
  ear_tag: '',
  breed: 'minipig',
  gender: 'male',
  source_id: '',
  entry_date: new Date().toISOString().split('T')[0],
  entry_weight: '',
  birth_date: '',
  pre_experiment_code: '',
  remark: '',
  breed_other: '',
}

const defaultQuickAddForm: QuickAddForm = {
  breed: 'minipig',
  breed_other: '',
  gender: 'male',
  entry_date: new Date().toISOString().split('T')[0],
  birth_date: '',
  entry_weight: '',
}

export function useAnimalFilters({
  allowedStatuses,
  defaultStatus,
}: {
  allowedStatuses: string[]
  defaultStatus: string
}) {
  const [searchParams, setSearchParams] = useSearchParams()
  const urlStatus = searchParams.get('status')
  const initialStatus =
    urlStatus && allowedStatuses.includes(urlStatus) ? urlStatus : defaultStatus

  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>(initialStatus)
  const [breedFilter, setBreedFilter] = useState<string>('all')
  const [page, setPage] = useState(1)
  const [sortColumn, setSortColumn] = useState<string | null>(null)
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc')

  useEffect(() => {
    if (!allowedStatuses.includes(statusFilter)) {
      setStatusFilter(defaultStatus)
      setSearchParams({ status: defaultStatus })
    }
  }, [statusFilter, setSearchParams, allowedStatuses, defaultStatus])

  return {
    search,
    setSearch,
    statusFilter,
    setStatusFilter,
    breedFilter,
    setBreedFilter,
    page,
    setPage,
    sortColumn,
    setSortColumn,
    sortDirection,
    setSortDirection,
    searchParams,
    setSearchParams,
  }
}

export function useAnimalDialogs() {
  const [showAddDialog, setShowAddDialog] = useState(false)
  const [showBatchAssignDialog, setShowBatchAssignDialog] = useState(false)
  const [showBatchExportDialog, setShowBatchExportDialog] = useState(false)
  const [showImportBasicDialog, setShowImportBasicDialog] = useState(false)
  const [showImportWeightDialog, setShowImportWeightDialog] = useState(false)
  const [showPrintReport, setShowPrintReport] = useState(false)
  const [showDuplicateWarning, setShowDuplicateWarning] = useState(false)
  const [showQuickAddDialog, setShowQuickAddDialog] = useState(false)
  const [duplicateWarningData, setDuplicateWarningData] =
    useState<DuplicateWarningData | null>(null)

  return {
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
  }
}

export function useAnimalSelection() {
  const [selectedAnimals, setSelectedAnimals] = useState<string[]>([])
  const [assignIacucNo, setAssignIacucNo] = useState('')
  const [quickEditAnimalId, setQuickEditAnimalId] = useState<string | null>(null)

  return {
    selectedAnimals,
    setSelectedAnimals,
    assignIacucNo,
    setAssignIacucNo,
    quickEditAnimalId,
    setQuickEditAnimalId,
  }
}

export function useAnimalForms() {
  const [newAnimal, setNewAnimal] = useState<NewAnimalForm>(defaultNewAnimal)
  const [quickAddPending, setQuickAddPending] = useState<{
    earTag: string
    penLocation: string
  } | null>(null)
  const [quickAddForm, setQuickAddForm] = useState<QuickAddForm>(defaultQuickAddForm)
  const [penBuilding, setPenBuilding] = useState('')
  const [penZone, setPenZone] = useState('')
  const [penNumber, setPenNumber] = useState('')

  const resetNewAnimalForm = () => {
    setPenBuilding('')
    setPenZone('')
    setPenNumber('')
    setNewAnimal(defaultNewAnimal)
  }

  return {
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
  }
}
