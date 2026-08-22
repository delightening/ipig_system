import { useState, useEffect } from 'react'
import { useSearchParams } from 'react-router-dom'
import type { SetURLSearchParams } from 'react-router-dom'
import type { NewAnimalForm, QuickAddForm, DuplicateWarningData } from '../components/AnimalAddDialog'

const defaultNewAnimal: NewAnimalForm = {
  ear_tag: '',
  species_id: '',
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
  species_id: '',
  breed_other: '',
  gender: 'male',
  entry_date: new Date().toISOString().split('T')[0],
  birth_date: '',
  entry_weight: '',
}

interface UrlFilterSyncOptions {
  searchParams: URLSearchParams
  setSearchParams: SetURLSearchParams
  allowedStatuses: string[]
  defaultStatus: string
  statusFilter: string
  setStatusFilter: (value: string) => void
  setBuildingCode: (value: string | null) => void
}

/**
 * 網址 ↔ tag 狀態的雙向同步。
 *
 * 網址是 tag 的真相源：外部連結、書籤、瀏覽器上一頁／下一頁都要切到對應的 tag。
 * 少了這段，網址只有初次載入會被讀取，站內按上一頁畫面就不會跟著回去。
 */
function useUrlFilterSync({
  searchParams,
  setSearchParams,
  allowedStatuses,
  defaultStatus,
  statusFilter,
  setStatusFilter,
  setBuildingCode,
}: UrlFilterSyncOptions) {
  // 權限變動導致當前 status 不再允許（例如 PI 開到 admin-only 連結）→ 退回預設並導正網址
  useEffect(() => {
    if (!allowedStatuses.includes(statusFilter)) {
      setStatusFilter(defaultStatus)
      setSearchParams({ status: defaultStatus })
    }
  }, [statusFilter, setSearchParams, allowedStatuses, defaultStatus, setStatusFilter])

  useEffect(() => {
    const urlStatus = searchParams.get('status')
    // 缺少或不允許的 status 一律回復預設；否則從 ?status=completed 按上一頁回到
    // 無參數網址時，畫面會卡在 completed 而與網址不同步。
    setStatusFilter(
      urlStatus && allowedStatuses.includes(urlStatus) ? urlStatus : defaultStatus
    )
    setBuildingCode(searchParams.get('building'))
  }, [searchParams, allowedStatuses, defaultStatus, setStatusFilter, setBuildingCode])
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
  /**
   * 欄位檢視選中的棟舍 code（網址 `?building=A`）。
   * null = 網址未指定，交由 BuildingTabs fallback 到第一棟；不在此處預設，
   * 因為棟舍清單是非同步載入的，這裡還不知道第一棟是誰。
   */
  const [buildingCode, setBuildingCode] = useState<string | null>(() =>
    searchParams.get('building')
  )
  const [breedFilter, setBreedFilter] = useState<string>('all')
  const [page, setPage] = useState(1)
  const [sortColumn, setSortColumn] = useState<string | null>(null)
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc')

  useUrlFilterSync({
    searchParams,
    setSearchParams,
    allowedStatuses,
    defaultStatus,
    statusFilter,
    setStatusFilter,
    setBuildingCode,
  })

  return {
    search,
    setSearch,
    statusFilter,
    setStatusFilter,
    buildingCode,
    setBuildingCode,
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
  const [showVetPatrolDialog, setShowVetPatrolDialog] = useState(false)
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
    showVetPatrolDialog,
    setShowVetPatrolDialog,
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
  const [penCode, setPenCode] = useState('')

  const resetNewAnimalForm = () => {
    setPenBuilding('')
    setPenZone('')
    setPenCode('')
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
    penCode,
    setPenCode,
    resetNewAnimalForm,
  }
}
