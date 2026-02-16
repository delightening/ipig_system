import { useState, useEffect, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Link, useSearchParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { useAuthStore } from '@/stores/auth'
import api, {
  Animal,
  AnimalListItem,
  AnimalStatus,
  animalStatusNames,
  allAnimalStatusNames,
  animalBreedNames,
  animalGenderNames,
  AnimalSource,
  AnimalBreed,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
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
import { toast } from '@/components/ui/use-toast'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Plus,
  Search,
  Eye,
  Edit2,
  Loader2,
  Upload,
  Download,
  AlertCircle,
  AlertTriangle,
  CheckCircle2,
  Stethoscope,
  FileSpreadsheet,
  LayoutGrid,
  MapPin,
  ArrowUpDown,
} from 'lucide-react'

// Import Export Dialog
import { ExportDialog } from '@/components/animal/ExportDialog'
import { ImportDialog } from '@/components/animal/ImportDialog'
import { QuickEditAnimalDialog } from '@/components/animal/QuickEditAnimalDialog'
import { AnimalPenReport } from '../../components/animal/AnimalPenReport'

const statusColors: Record<AnimalStatus, string> = {
  unassigned: 'bg-gray-100 text-gray-800',
  in_experiment: 'bg-orange-100 text-orange-800',
  completed: 'bg-green-100 text-green-800',
  euthanized: 'bg-red-100 text-red-800',
  sudden_death: 'bg-rose-100 text-rose-800',
  transferred: 'bg-indigo-100 text-indigo-800',
}

const getPenLocationDisplay = (animal: { status: AnimalStatus; pen_location?: string | null }, t: any) => {
  if (animal.status === 'completed' && !animal.pen_location) {
    return t('animals.sacrificed')
  }
  return animal.pen_location || '-'
}

const buildPenNumbers = (count: number) =>
  Array.from({ length: count }, (_, index) => String(index + 1).padStart(2, '0'))

const penBuildings = [
  { value: 'A', label: 'A 棟 (ACD)' },
  { value: 'B', label: 'B 棟 (BEFG)' },
]

const penZonesByBuilding: Record<string, string[]> = {
  A: ['A', 'C', 'D'],
  B: ['B', 'E', 'F', 'G'],
}

const penNumbersByZone: Record<string, string[]> = {
  A: buildPenNumbers(20),
  B: buildPenNumbers(20),
  C: buildPenNumbers(20),
  D: buildPenNumbers(33),
  E: buildPenNumbers(25),
  F: buildPenNumbers(6),
  G: buildPenNumbers(6),
}

// 區域顏色對應（參照 Excel 示意圖）
const penZoneColors: Record<string, { bg: string; border: string; header: string; text: string }> = {
  A: { bg: 'bg-blue-50', border: 'border-blue-300', header: 'bg-blue-500', text: 'text-blue-700' },
  B: { bg: 'bg-orange-50', border: 'border-orange-300', header: 'bg-orange-500', text: 'text-orange-700' },
  C: { bg: 'bg-yellow-50', border: 'border-yellow-300', header: 'bg-yellow-500', text: 'text-yellow-700' },
  D: { bg: 'bg-cyan-50', border: 'border-cyan-300', header: 'bg-cyan-500', text: 'text-cyan-700' },
  E: { bg: 'bg-purple-50', border: 'border-purple-300', header: 'bg-purple-500', text: 'text-purple-700' },
  F: { bg: 'bg-amber-50', border: 'border-amber-300', header: 'bg-amber-500', text: 'text-amber-700' },
  G: { bg: 'bg-green-50', border: 'border-green-300', header: 'bg-green-500', text: 'text-green-700' },
}

export function AnimalsPage() {
  const queryClient = useQueryClient()
  const [searchParams, setSearchParams] = useSearchParams()
  const { hasRole } = useAuthStore()
  const { t } = useTranslation()

  // Check if user is PI or CLIENT
  const isPIOrClient = hasRole('PI') || hasRole('CLIENT')

  // Building tab state for grouped view
  const [groupedBuildingTab, setGroupedBuildingTab] = useState<'A' | 'B'>('A')

  // Filter state
  const [search, setSearch] = useState('')
  // PI/CLIENT can only see: in_experiment, completed
  // Other users can see: pen, unassigned, in_experiment, completed
  const allowedStatuses = isPIOrClient
    ? ['in_experiment', 'completed', 'euthanized', 'sudden_death', 'transferred']
    : ['pen', 'unassigned', 'in_experiment', 'completed', 'euthanized', 'sudden_death', 'transferred', 'all']
  const urlStatus = searchParams.get('status')
  // Default: PI/CLIENT -> 'in_experiment', others -> 'pen'
  const defaultStatus = isPIOrClient ? 'in_experiment' : 'pen'
  const initialStatus = urlStatus && allowedStatuses.includes(urlStatus) ? urlStatus : defaultStatus
  const [statusFilter, setStatusFilter] = useState<string>(initialStatus)
  const [breedFilter, setBreedFilter] = useState<string>('all')

  // Redirect to allowed tab if user tries to access restricted tabs
  useEffect(() => {
    if (!allowedStatuses.includes(statusFilter)) {
      setStatusFilter(defaultStatus)
      setSearchParams({ status: defaultStatus })
    }
  }, [statusFilter, setSearchParams, allowedStatuses, defaultStatus])

  // Dialog state
  const [showAddDialog, setShowAddDialog] = useState(false)
  const [showBatchAssignDialog, setShowBatchAssignDialog] = useState(false)
  const [showBatchExportDialog, setShowBatchExportDialog] = useState(false)
  const [showImportBasicDialog, setShowImportBasicDialog] = useState(false)
  const [showImportWeightDialog, setShowImportWeightDialog] = useState(false)
  const [selectedAnimals, setSelectedAnimals] = useState<string[]>([])
  const [assignIacucNo, setAssignIacucNo] = useState('')

  // Quick edit dialog state
  const [quickEditAnimalId, setQuickEditAnimalId] = useState<string | null>(null)
  const [showPrintReport, setShowPrintReport] = useState(false)

  // Sorting state
  const [sortColumn, setSortColumn] = useState<string | null>(null)
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc')

  // Quick move state (空欄位快速移動)
  const [editingPenLocation, setEditingPenLocation] = useState<string | null>(null)
  const [editingEarTag, setEditingEarTag] = useState<string>('')

  // 耳號重複警告對話框狀態
  const [showDuplicateWarning, setShowDuplicateWarning] = useState(false)
  const [duplicateWarningData, setDuplicateWarningData] = useState<{
    earTag: string
    existingAnimals: Array<{ id: string; birth_date: string | null; status: string; pen_location: string | null }>
    source: 'create' | 'quickAdd'
    pendingPayload: any
  } | null>(null)

  // Quick add dialog state (快速新增動物對話框)
  const [showQuickAddDialog, setShowQuickAddDialog] = useState(false)
  const [quickAddPending, setQuickAddPending] = useState<{ earTag: string; penLocation: string } | null>(null)
  const [quickAddForm, setQuickAddForm] = useState({
    breed: 'minipig' as AnimalBreed,
    breed_other: '',
    gender: 'male' as 'male' | 'female',
    entry_date: new Date().toISOString().split('T')[0],
    birth_date: '',
    entry_weight: '',
  })

  // Form state for new animal
  const [penBuilding, setPenBuilding] = useState('')
  const [penZone, setPenZone] = useState('')
  const [penNumber, setPenNumber] = useState('')
  const [newAnimal, setNewAnimal] = useState({
    ear_tag: '',
    breed: 'minipig' as AnimalBreed,
    gender: 'male' as const,
    source_id: '',
    entry_date: new Date().toISOString().split('T')[0],
    entry_weight: '',
    birth_date: '',
    pre_experiment_code: '',
    remark: '',
    breed_other: '',
  })

  // Query for all animals (for counting)
  const { data: allAnimalsData } = useQuery({
    queryKey: ['animals-count'],
    queryFn: async () => {
      const res = await api.get<AnimalListItem[]>(`/animals`)
      return res.data
    },
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  // Queries
  const { data: animalsData, isLoading } = useQuery({
    queryKey: ['animals', statusFilter, breedFilter, search],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (statusFilter && statusFilter !== 'all' && statusFilter !== 'pen') params.append('status', statusFilter)
      if (breedFilter && breedFilter !== 'all') params.append('breed', breedFilter)
      if (search) params.append('search', search)
      const res = await api.get<AnimalListItem[]>(`/animals?${params}`)
      return res.data
    },
    staleTime: 0, // Always consider data stale for real-time updates
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: sourcesData } = useQuery({
    queryKey: ['animal-sources'],
    queryFn: async () => {
      const res = await api.get<AnimalSource[]>('/animal-sources')
      return res.data
    },
  })

  // Query grouped by pen
  const { data: groupedData, isLoading: groupedLoading } = useQuery({
    queryKey: ['animals-by-pen'],
    queryFn: async () => {
      const res = await api.get<{ pen_location: string; animals: AnimalListItem[] }[]>('/animals/by-pen')
      return res.data
    },
    enabled: statusFilter === 'pen',
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  // Mutations
  const createAnimalMutation = useMutation({
    mutationFn: async (data: typeof newAnimal) => {
      // 驗證欄位必填
      if (!penZone || !penNumber) {
        throw new Error('欄位為必填，請選擇欄位區和欄位編號')
      }
      const penLocation = `${penZone}${penNumber}`

      // 驗證必填欄位
      if (!data.ear_tag?.trim()) {
        throw new Error('耳號為必填')
      }
      if (!data.entry_date) {
        throw new Error('進場日期為必填')
      }
      if (!data.birth_date) {
        throw new Error('出生日期為必填')
      }
      if (!data.pre_experiment_code?.trim()) {
        throw new Error('實驗前代號為必填')
      }

      // 驗證並轉換 entry_weight
      let entryWeight: number | undefined = undefined
      if (data.entry_weight && data.entry_weight !== '') {
        const weightValue = parseFloat(data.entry_weight)
        if (isNaN(weightValue) || weightValue <= 0) {
          throw new Error('進場體重必須是大於 0 的數字')
        }
        entryWeight = weightValue
      }

      // 驗證日期格式（必須是 YYYY-MM-DD）
      const dateRegex = /^\d{4}-\d{2}-\d{2}$/
      if (!dateRegex.test(data.entry_date)) {
        throw new Error('進場日期格式不正確，必須是 YYYY-MM-DD 格式')
      }
      if (data.birth_date && data.birth_date.trim() !== '' && !dateRegex.test(data.birth_date)) {
        throw new Error('出生日期格式不正確，必須是 YYYY-MM-DD 格式')
      }

      // 格式化耳號：如果是數字則補零至三位數
      let formattedEarTag = data.ear_tag.trim()
      if (/^\d+$/.test(formattedEarTag)) {
        formattedEarTag = formattedEarTag.padStart(3, '0')
      }

      // 清理資料格式
      const payload: any = {
        ear_tag: formattedEarTag,
        breed: data.breed, // 'minipig', 'white', 'other'
        gender: data.gender, // 'male', 'female'
        entry_date: data.entry_date, // YYYY-MM-DD format
        birth_date: data.birth_date && data.birth_date.trim() !== '' ? data.birth_date.trim() : undefined,
        entry_weight: entryWeight, // number or undefined
        pen_location: penLocation, // string (required)
        pre_experiment_code: data.pre_experiment_code && data.pre_experiment_code.trim() !== ''
          ? data.pre_experiment_code.trim()
          : undefined,
        remark: data.remark && data.remark.trim() !== '' ? data.remark.trim() : undefined,
        breed_other: data.breed === 'other' ? data.breed_other : undefined,
      }
      // 只有當 source_id 不是空字串且不是 'none' 時才加入（必須是有效的 UUID）
      if (data.source_id && data.source_id.trim() !== '' && data.source_id.trim() !== 'none') {
        // 驗證 UUID 格式
        const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
        const trimmedSourceId = data.source_id.trim()
        if (uuidRegex.test(trimmedSourceId)) {
          payload.source_id = trimmedSourceId
        } else {
          throw new Error(`來源 ID 格式不正確: ${trimmedSourceId}`)
        }
      }
      // 如果 source_id 是空字串或 'none'，不發送該欄位（後端會視為 None）

      console.log('Sending payload:', payload)
      return api.post('/animals', payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      queryClient.invalidateQueries({ queryKey: ['animals-by-pen'] })
      queryClient.invalidateQueries({ queryKey: ['animals-count'] })
      toast({ title: '成功', description: '動物已新增' })
      setShowAddDialog(false)
      resetNewAnimalForm()
    },
    onError: (error: any) => {
      console.error('Create animal error:', error)

      // 攔截 409 耳號重複警告（可確認後強制建立）
      if (error?.response?.status === 409) {
        const errData = error.response.data?.error
        if (errData?.warning_type === 'duplicate_ear_tag' && errData?.blocking === false) {
          // 取得 payload（從 error.config.data 解析）
          let payload: any = {}
          try { payload = JSON.parse(error.config?.data || '{}') } catch { /* ignore */ }
          setDuplicateWarningData({
            earTag: payload.ear_tag || '',
            existingAnimals: errData.existing_animals || [],
            source: 'create',
            pendingPayload: payload,
          })
          setShowDuplicateWarning(true)
          return
        }
      }

      // 提取錯誤訊息
      let errorMessage = '新增失敗，請檢查輸入資料'

      // 422 錯誤通常是資料格式問題
      if (error?.response?.status === 422) {
        errorMessage = '資料格式錯誤：請檢查所有欄位的格式是否正確（例如：品種應為 minipig/white/other，性別應為 male/female，日期應為 YYYY-MM-DD 格式）'
        if (error?.response?.data?.error?.message) {
          errorMessage = error.response.data.error.message
        } else if (error?.response?.data?.message) {
          errorMessage = error.response.data.message
        }
      } else if (error?.response?.data?.error?.message) {
        errorMessage = error.response.data.error.message
      } else if (error?.response?.data?.message) {
        errorMessage = error.response.data.message
      } else if (error?.message) {
        errorMessage = error.message
      }

      toast({
        title: '錯誤',
        description: errorMessage,
        variant: 'destructive',
      })
    },
  })

  const batchAssignMutation = useMutation({
    mutationFn: async () => {
      return api.post('/animals/batch/assign', {
        animal_ids: selectedAnimals,
        iacuc_no: assignIacucNo,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      toast({ title: '成功', description: '動物已分配至計劃並進入實驗中' })
      setShowBatchAssignDialog(false)
      setSelectedAnimals([])
      setAssignIacucNo('')
    },
    onError: (error: any) => {
      console.error('Batch assign error:', error)
      const errorMessage = error?.response?.data?.error?.message
        || error?.response?.data?.message
        || error?.message
        || '批次分配失敗'
      toast({
        title: '錯誤',
        description: errorMessage,
        variant: 'destructive',
      })
    },
  })

  const batchStartExperimentMutation = useMutation({
    mutationFn: async () => {
      return api.post('/animals/batch/start-experiment', {
        animal_ids: selectedAnimals,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      toast({ title: '成功', description: '動物已新增' })
      setSelectedAnimals([])
    },
    onError: (error: any) => {
      console.error('Batch start experiment error:', error)
      const errorMessage = error?.response?.data?.error?.message
        || error?.response?.data?.message
        || error?.message
        || '批次啟動實驗失敗'
      toast({
        title: '錯誤',
        description: errorMessage,
        variant: 'destructive',
      })
    },
  })

  // Quick move mutation (空欄位快速移動或新增動物)
  const quickMoveMutation = useMutation({
    mutationFn: async ({ earTag, targetPenLocation }: { earTag: string; targetPenLocation: string }) => {
      // 格式化耳號：如果是數字則補零至三位數
      let formattedEarTag = earTag.trim()
      if (/^\d+$/.test(formattedEarTag)) {
        formattedEarTag = formattedEarTag.padStart(3, '0')
      }

      // 先根據耳號查詢動物
      const searchRes = await api.get<AnimalListItem[]>(`/animals?keyword=${encodeURIComponent(formattedEarTag)}`)
      const matchingAnimals = searchRes.data.filter(p => p.ear_tag === formattedEarTag)

      if (matchingAnimals.length === 0) {
        // 找不到該耳號，顯示新增對話框
        return { notFound: true, formattedEarTag, targetPenLocation }
      }

      if (matchingAnimals.length > 1) {
        throw new Error(`找到多隻耳號為 "${formattedEarTag}" 的動物，請使用編輯功能手動移動`)
      }

      const animal = matchingAnimals[0]

      // 檢查動物是否已經在目標欄位
      if (animal.pen_location === targetPenLocation) {
        throw new Error(`動物 ${formattedEarTag} 已經在 ${targetPenLocation} 欄位`)
      }

      // 更新動物的欄位
      return {
        ...await api.put<Animal>(`/animals/${animal.id}`, {
          pen_location: targetPenLocation,
        }), notFound: false
      }
    },
    onSuccess: (data: any, variables) => {
      if (data.notFound) {
        // 顯示新增對話框
        setQuickAddPending({ earTag: data.formattedEarTag, penLocation: data.targetPenLocation })
        setQuickAddForm({
          breed: 'minipig',
          breed_other: '',
          gender: 'male',
          entry_date: new Date().toISOString().split('T')[0],
          birth_date: '',
          entry_weight: '',
        })
        setShowQuickAddDialog(true)
        setEditingPenLocation(null)
        setEditingEarTag('')
        return
      }

      queryClient.invalidateQueries({ queryKey: ['animals'] })
      queryClient.invalidateQueries({ queryKey: ['animals-by-pen'] })
      queryClient.invalidateQueries({ queryKey: ['animals-count'] })
      // 格式化耳號以顯示正確的訊息
      let formattedEarTag = variables.earTag.trim()
      if (/^\d+$/.test(formattedEarTag)) {
        formattedEarTag = formattedEarTag.padStart(3, '0')
      }
      toast({
        title: '成功',
        description: `動物 ${formattedEarTag} 已移動到 ${variables.targetPenLocation}`
      })
      setEditingPenLocation(null)
      setEditingEarTag('')
    },
    onError: (error: any) => {
      console.error('Quick move error:', error)
      const errorMessage = error?.response?.data?.error?.message
        || error?.response?.data?.message
        || error?.message
        || '移動失敗'
      toast({
        title: '錯誤',
        description: errorMessage,
        variant: 'destructive',
      })
      // 保持編輯狀態，讓用戶可以修正
    },
  })

  // Quick add mutation (快速新增動物)
  const quickAddMutation = useMutation({
    mutationFn: async () => {
      if (!quickAddPending) throw new Error('無待處理的新增請求')

      // 驗證必填欄位
      if (!quickAddForm.entry_date) {
        throw new Error('進場日期為必填')
      }
      if (!quickAddForm.birth_date) {
        throw new Error('出生日期為必填')
      }

      const payload = {
        ear_tag: quickAddPending.earTag,
        breed: quickAddForm.breed,
        breed_other: quickAddForm.breed === 'other' ? quickAddForm.breed_other : undefined,
        gender: quickAddForm.gender,
        entry_date: quickAddForm.entry_date,
        birth_date: quickAddForm.birth_date,
        entry_weight: parseFloat(quickAddForm.entry_weight),
        pen_location: quickAddPending.penLocation,
      }
      return api.post<Animal>('/animals', payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      queryClient.invalidateQueries({ queryKey: ['animals-by-pen'] })
      queryClient.invalidateQueries({ queryKey: ['animals-count'] })
      toast({
        title: '成功',
        description: `已新增動物 ${quickAddPending?.earTag} 至 ${quickAddPending?.penLocation}`
      })
      setShowQuickAddDialog(false)
      setQuickAddPending(null)
    },
    onError: (error: any) => {
      console.error('Quick add error:', error)

      // 攔截 409 耳號重複警告（可確認後強制建立）
      if (error?.response?.status === 409) {
        const errData = error.response.data?.error
        if (errData?.warning_type === 'duplicate_ear_tag' && errData?.blocking === false) {
          let payload: any = {}
          try { payload = JSON.parse(error.config?.data || '{}') } catch { /* ignore */ }
          setDuplicateWarningData({
            earTag: payload.ear_tag || quickAddPending?.earTag || '',
            existingAnimals: errData.existing_animals || [],
            source: 'quickAdd',
            pendingPayload: payload,
          })
          setShowDuplicateWarning(true)
          return
        }
      }

      const errorMessage = error?.response?.data?.error?.message
        || error?.response?.data?.message
        || error?.message
        || '新增失敗'
      toast({
        title: '錯誤',
        description: errorMessage,
        variant: 'destructive',
      })
    },
  })

  // 確認強制建立（耳號重複後使用者確認）
  const forceCreateMutation = useMutation({
    mutationFn: async (payload: any) => {
      return api.post('/animals', { ...payload, force_create: true })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      queryClient.invalidateQueries({ queryKey: ['animals-by-pen'] })
      queryClient.invalidateQueries({ queryKey: ['animals-count'] })
      toast({ title: '成功', description: '動物已新增（已確認耳號重複）' })
      setShowDuplicateWarning(false)
      setDuplicateWarningData(null)
      setShowAddDialog(false)
      setShowQuickAddDialog(false)
      setQuickAddPending(null)
      resetNewAnimalForm()
    },
    onError: (error: any) => {
      const errorMessage = error?.response?.data?.error?.message || error?.message || '新增失敗'
      toast({ title: '錯誤', description: errorMessage, variant: 'destructive' })
    },
  })

  const resetNewAnimalForm = () => {
    setPenBuilding('')
    setPenZone('')
    setPenNumber('')
    setNewAnimal({
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
    })
  }

  const toggleAnimalSelection = (id: string) => {
    setSelectedAnimals(prev =>
      prev.includes(id) ? prev.filter(p => p !== id) : [...prev, id]
    )
  }

  const toggleAllAnimals = () => {
    const currentAnimals = animalsData || []
    if (currentAnimals.length === 0) return
    if (selectedAnimals.length === currentAnimals.length) {
      setSelectedAnimals([])
    } else {
      setSelectedAnimals(currentAnimals.map(p => p.id))
    }
  }

  const animals = animalsData || []
  const allAnimals = allAnimalsData || []

  // Sorting handler
  const handleSort = (column: string) => {
    if (sortColumn === column) {
      setSortDirection(prev => prev === 'asc' ? 'desc' : 'asc')
    } else {
      setSortColumn(column)
      setSortDirection('asc')
    }
  }

  // Sorted animals data
  const sortedAnimals = useMemo(() => {
    if (!sortColumn) return animals
    return [...animals].sort((a, b) => {
      let aVal: any = a[sortColumn as keyof typeof a] ?? ''
      let bVal: any = b[sortColumn as keyof typeof b] ?? ''
      if (sortColumn === 'entry_date') {
        aVal = aVal ? new Date(aVal).getTime() : 0
        bVal = bVal ? new Date(bVal).getTime() : 0
      } else if (sortColumn === 'latest_weight') {
        aVal = aVal !== null && aVal !== undefined ? Number(aVal) : (sortDirection === 'asc' ? Infinity : -Infinity)
        bVal = bVal !== null && bVal !== undefined ? Number(bVal) : (sortDirection === 'asc' ? Infinity : -Infinity)
      } else if (typeof aVal === 'string') aVal = aVal.toLowerCase()
      if (typeof bVal === 'string' && sortColumn !== 'latest_weight') bVal = bVal.toLowerCase()

      if (aVal < bVal) return sortDirection === 'asc' ? -1 : 1
      if (aVal > bVal) return sortDirection === 'asc' ? 1 : -1
      return 0
    })
  }, [animals, sortColumn, sortDirection])

  // Sortable header component
  const SortableHeader = ({ column, label }: { column: string; label: string }) => (
    <TableHead
      className="cursor-pointer hover:bg-slate-100 select-none"
      onClick={() => handleSort(column)}
    >
      <div className="flex items-center gap-1">
        {label}
        <ArrowUpDown className={`h-3 w-3 ${sortColumn === column ? 'text-purple-600' : 'text-slate-400'}`} />
      </div>
    </TableHead>
  )

  // 計算狀態計數（基於所有動物，而非過濾後的結果）
  const statusCounts = allAnimals.reduce((acc, animal) => {
    acc[animal.status] = (acc[animal.status] || 0) + 1
    return acc
  }, {} as Record<string, number>)

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-start justify-between gap-4">
        <div>
          <h1 className="text-xl md:text-2xl font-bold text-slate-900">{t('animals.title')}</h1>
          <p className="text-sm md:text-base text-slate-500">{t('animals.description')}</p>
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

      {/* Status Tabs */}
      <div className="flex gap-2 border-b overflow-x-auto flex-nowrap">
        {[
          { value: 'pen', label: t('animals.statusLabels.pen'), count: allAnimals.length, icon: <LayoutGrid className="h-4 w-4" /> },
          { value: 'unassigned', label: t('animals.statusLabels.unassigned'), count: statusCounts['unassigned'] || 0 },
          { value: 'in_experiment', label: t('animals.statusLabels.in_experiment'), count: statusCounts['in_experiment'] || 0 },
          { value: 'completed', label: t('animals.statusLabels.completed'), count: statusCounts['completed'] || 0 },
          { value: 'euthanized', label: t('animals.statusLabels.euthanized'), count: statusCounts['euthanized'] || 0 },
          { value: 'sudden_death', label: t('animals.statusLabels.sudden_death'), count: statusCounts['sudden_death'] || 0 },
          { value: 'transferred', label: t('animals.statusLabels.transferred'), count: statusCounts['transferred'] || 0 },
          { value: 'all', label: t('animals.statusLabels.all'), count: allAnimals.length },
        ]
          .filter(tab => {
            // PI and CLIENT can only see: 實驗中, 實驗完成, 已安樂死, 猝死
            if (isPIOrClient) {
              return ['in_experiment', 'completed', 'euthanized', 'sudden_death', 'transferred'].includes(tab.value)
            }
            return true
          })
          .map(tab => (
            <button
              key={tab.value}
              onClick={() => {
                setStatusFilter(tab.value)
                setSearchParams(tab.value === 'pen' ? {} : { status: tab.value })
              }}
              className={`px-3 md:px-4 py-2 text-xs md:text-sm font-medium border-b-2 transition-colors flex items-center gap-1.5 whitespace-nowrap ${statusFilter === tab.value
                ? 'border-purple-600 text-purple-600'
                : 'border-transparent text-slate-500 hover:text-slate-700'
                }`}
            >
              {'icon' in tab && tab.icon}
              {tab.label} ({tab.count})
            </button>
          ))}
      </div>

      {/* Filters & Actions */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex flex-col md:flex-row md:items-center justify-between gap-3 md:gap-4">
            <div className="flex flex-col md:flex-row md:items-center gap-3 md:gap-4 flex-1">
              <div className="relative flex-1 max-w-sm">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <Input
                  placeholder={t('animals.searchPlaceholder')}
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  className="pl-9"
                />
              </div>
              <Select value={breedFilter} onValueChange={setBreedFilter}>
                <SelectTrigger className="w-[150px]">
                  <SelectValue placeholder="品種" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('animals.allBreeds')}</SelectItem>
                  <SelectItem value="minipig">{t('animals.breedLabels.minipig')}</SelectItem>
                  <SelectItem value="white">{t('animals.breedLabels.white')}</SelectItem>
                  <SelectItem value="other">{t('animals.breedLabels.other')}</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* Batch Actions */}
            {selectedAnimals.length > 0 && (
              <div className="flex items-center gap-2">
                <span className="text-sm text-slate-500">
                  {t('animals.selectedCount', { count: selectedAnimals.length })}
                </span>
                {statusFilter === 'unassigned' && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowBatchAssignDialog(true)}
                  >
                    分配至計畫
                  </Button>
                )}
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* List View */}
      {statusFilter !== 'pen' && (
        <Card>
          <CardContent className="p-0">
            {isLoading ? (
              <div className="flex items-center justify-center py-12">
                <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
              </div>
            ) : animals.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-slate-500">
                <AlertCircle className="h-12 w-12 mb-4" />
                <p>{t('animals.noAnimalsFound')}</p>
              </div>
            ) : (
              <div className="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-12">
                        <input
                          type="checkbox"
                          checked={selectedAnimals.length === animals.length && animals.length > 0}
                          onChange={toggleAllAnimals}
                          className="rounded border-slate-300"
                        />
                      </TableHead>

                      <SortableHeader column="ear_tag" label={t('animals.earTag')} />
                      <SortableHeader column="pen_location" label={t('animals.pen')} />
                      <SortableHeader column="iacuc_no" label={t('animals.iacucNo')} />
                      <TableHead>{t('animals.status')}</TableHead>
                      <TableHead>{t('animals.breed')}</TableHead>
                      <TableHead>{t('animals.gender')}</TableHead>
                      <TableHead>{t('animals.onMedicationShort')}</TableHead>
                      <TableHead>{t('animals.vetRecommendation')}</TableHead>
                      <SortableHeader column="entry_date" label={t('animals.entryDate')} />
                      <SortableHeader column="latest_weight" label={t('animals.currentWeight')} />
                      <TableHead className="text-right">{t('animals.actions')}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {sortedAnimals.map((animal) => (
                      <TableRow
                        key={animal.id}
                        className={animal.has_abnormal_record ? 'bg-yellow-50' : ''}
                      >
                        <TableCell>
                          <input
                            type="checkbox"
                            checked={selectedAnimals.includes(animal.id)}
                            onChange={() => toggleAnimalSelection(animal.id)}
                            className="rounded border-slate-300"
                          />
                        </TableCell>
                        <TableCell>
                          <Link
                            to={`/animals/${animal.id}`}
                            className="text-orange-600 hover:text-orange-700 font-medium"
                            title={`系統號: ${animal.id}`}
                          >
                            {animal.ear_tag}
                          </Link>
                        </TableCell>
                        <TableCell>{getPenLocationDisplay(animal, t)}</TableCell>
                        <TableCell>
                          {animal.iacuc_no || (
                            <span className="text-slate-400">未分配</span>
                          )}
                        </TableCell>
                        <TableCell>
                          <Badge className={statusColors[animal.status]}>
                            {t(`animals.statusLabels.${animal.status}`)}
                          </Badge>
                        </TableCell>
                        <TableCell>{animal.breed === 'other' ? (animal.breed_other || t('animals.breedLabels.other')) : t(`animals.breedLabels.${animal.breed}`)}</TableCell>
                        <TableCell>{t(`animals.genderLabels.${animal.gender}`)}</TableCell>
                        <TableCell>
                          {animal.is_on_medication ? (
                            <Badge variant="destructive" className="text-xs">{t('animals.onMedication')}</Badge>
                          ) : (
                            <span className="text-slate-400">{t('animals.notOnMedication')}</span>
                          )}
                        </TableCell>
                        <TableCell>
                          {animal.vet_recommendation_date ? (
                            <span className="text-sm text-slate-600">
                              {new Date(animal.vet_recommendation_date).toLocaleDateString()}
                            </span>
                          ) : (
                            <span className="text-slate-400">-</span>
                          )}
                        </TableCell>
                        <TableCell>
                          <div className="flex items-center gap-2">
                            <span>{new Date(animal.entry_date).toLocaleDateString()}</span>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-6 w-6"
                              onClick={() => setQuickEditAnimalId(animal.id)}
                              title="快速編輯"
                            >
                              <Edit2 className="h-3 w-3" />
                            </Button>
                          </div>
                        </TableCell>
                        <TableCell>
                          {animal.latest_weight ? (
                            <span className="text-sm text-slate-700 font-medium" title={animal.latest_weight_date ? `量測日期: ${new Date(animal.latest_weight_date).toLocaleDateString()}` : undefined}>
                              {animal.latest_weight} kg
                            </span>
                          ) : (
                            <span className="text-slate-400">-</span>
                          )}
                        </TableCell>
                        <TableCell className="text-right">
                          <div className="flex items-center justify-end gap-1">
                            <Button variant="ghost" size="icon" asChild title={t('common.view')}>
                              <Link to={`/animals/${animal.id}`}>
                                <Eye className="h-4 w-4" />
                              </Link>
                            </Button>
                            <Button variant="ghost" size="icon" asChild title={t('common.edit')}>
                              <Link to={`/animals/${animal.id}/edit`}>
                                <Edit2 className="h-4 w-4" />
                              </Link>
                            </Button>
                          </div>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Grouped View (Pen View) */}
      {statusFilter === 'pen' && (
        <div className="space-y-4">
          {/* Building Tabs */}
          <div className="flex gap-2 border-b">
            {penBuildings.map(building => {
              const zones = penZonesByBuilding[building.value]
              return (
                <button
                  key={building.value}
                  onClick={() => setGroupedBuildingTab(building.value as 'A' | 'B')}
                  className={`px-6 py-3 text-sm font-medium border-b-2 transition-colors flex items-center gap-2 ${groupedBuildingTab === building.value
                    ? 'border-purple-600 text-purple-600 bg-purple-50'
                    : 'border-transparent text-slate-500 hover:text-slate-700 hover:bg-slate-50'
                    }`}
                >
                  <MapPin className="h-4 w-4" />
                  {building.label}
                  <div className="flex gap-1 ml-2">
                    {zones.map(zone => (
                      <span
                        key={zone}
                        className={`w-5 h-5 rounded text-xs font-bold flex items-center justify-center text-white ${penZoneColors[zone]?.header || 'bg-gray-500'}`}
                      >
                        {zone}
                      </span>
                    ))}
                  </div>
                </button>
              )
            })}
          </div>

          {groupedLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
            </div>
          ) : (
            // Generate all pen slots for each zone in the selected building
            (() => {
              const currentZones = penZonesByBuilding[groupedBuildingTab] || []

              // Create a map of pen_location -> animals for quick lookup
              const animalsByPenLocation = new Map<string, AnimalListItem[]>()
              groupedData?.forEach(group => {
                if (group.pen_location) {
                  animalsByPenLocation.set(group.pen_location, group.animals)
                }
              })

              // Helper function to render a single pen cell
              const renderPenCell = (penLocation: string | null, colors: { bg: string; border: string; header: string; text: string }, isLeftColumn: boolean = true) => {
                if (!penLocation) {
                  return <div className="px-3 py-2 text-slate-300"></div>
                }

                const penAnimals = animalsByPenLocation.get(penLocation) || []
                const cellColors = penZoneColors[penLocation.charAt(0)] || colors
                const isEditing = editingPenLocation === penLocation

                if (penAnimals.length === 0) {
                  const handleSubmit = () => {
                    if (editingEarTag.trim()) {
                      quickMoveMutation.mutate({
                        earTag: editingEarTag.trim(),
                        targetPenLocation: penLocation,
                      })
                    } else {
                      setEditingPenLocation(null)
                      setEditingEarTag('')
                    }
                  }

                  return (
                    <div
                      className="grid grid-cols-5 gap-1 px-3 py-2 items-center text-sm group"
                      onMouseEnter={() => {
                        if (!isEditing && !quickMoveMutation.isPending) {
                          setEditingPenLocation(penLocation)
                          setEditingEarTag('')
                        }
                      }}
                    >
                      <div className={`font-semibold ${cellColors.text}`}>{penLocation}</div>
                      {isEditing ? (
                        <Input
                          className="h-7 text-sm"
                          value={editingEarTag}
                          onChange={(e) => setEditingEarTag(e.target.value)}
                          onKeyDown={(e) => {
                            if (e.key === 'Enter' && editingEarTag.trim()) {
                              e.preventDefault()
                              handleSubmit()
                            } else if (e.key === 'Escape') {
                              e.preventDefault()
                              setEditingPenLocation(null)
                              setEditingEarTag('')
                            }
                          }}
                          onBlur={(e) => {
                            setTimeout(() => {
                              if (editingPenLocation === penLocation && editingEarTag.trim()) {
                                handleSubmit()
                              } else if (editingPenLocation === penLocation && !editingEarTag.trim()) {
                                setEditingPenLocation(null)
                                setEditingEarTag('')
                              }
                            }, 150)
                          }}
                          placeholder="輸入耳號"
                          autoFocus
                          disabled={quickMoveMutation.isPending}
                        />
                      ) : (
                        <div className="text-slate-400 italic group-hover:text-slate-600 transition-colors cursor-text">空</div>
                      )}
                      <div className="text-slate-300">-</div>
                      <div className="text-slate-300">-</div>
                      <div></div>
                    </div>
                  )
                }

                return penAnimals.map((animal, animalIdx) => (
                  <div key={animal.id} className={`grid grid-cols-5 gap-1 px-3 py-2 items-center text-sm ${animalIdx > 0 ? 'border-t border-dashed border-slate-200' : ''}`}>
                    <div className={`font-semibold ${cellColors.text}`}>{animalIdx === 0 ? penLocation : ''}</div>
                    <div className={`font-medium ${cellColors.text} truncate`} title={animal.ear_tag}>{animal.ear_tag}</div>
                    <div className="text-xs text-slate-500 truncate" title={animal.vet_last_viewed_at ? new Date(animal.vet_last_viewed_at).toLocaleString('zh-TW') : '-'}>
                      {animal.vet_last_viewed_at ? new Date(animal.vet_last_viewed_at).toLocaleDateString('zh-TW') : '-'}
                    </div>
                    <div className={`text-xs truncate ${animal.has_abnormal_record ? 'text-red-600 font-medium' : 'text-slate-400'}`}>
                      {animal.has_abnormal_record ? '有異常' : '-'}
                    </div>
                    <div className="flex items-center justify-center gap-1">
                      <Button variant="ghost" size="icon" className="h-6 w-6" asChild title="檢視">
                        <Link to={`/animals/${animal.id}`}>
                          <Eye className="h-3 w-3" />
                        </Link>
                      </Button>
                      <Button variant="ghost" size="icon" className="h-6 w-6" asChild title="編輯">
                        <Link to={`/animals/${animal.id}/edit`}>
                          <Edit2 className="h-3 w-3" />
                        </Link>
                      </Button>
                    </div>
                  </div>
                ))
              }

              // Helper function to render standard zone card (for A, C, D, B zones)
              const renderStandardZoneCard = (zone: string) => {
                const colors = penZoneColors[zone] || { bg: 'bg-gray-50', border: 'border-gray-300', header: 'bg-gray-500', text: 'text-gray-700' }
                const penNumbers = penNumbersByZone[zone] || []
                const totalPenNumbers = penNumbers.length

                // Split pen numbers into two columns
                const halfPoint = Math.ceil(totalPenNumbers / 2)
                const leftColumnPens = penNumbers.slice(0, halfPoint)
                const rightColumnPens = penNumbers.slice(halfPoint)

                // Count total animals in this zone
                let totalAnimals = 0
                penNumbers.forEach(num => {
                  const penLocation = `${zone}${num}`
                  const penAnimals = animalsByPenLocation.get(penLocation) || []
                  totalAnimals += penAnimals.length
                })

                return (
                  <Card key={zone} className={`${colors.bg} ${colors.border} border-2`}>
                    <CardHeader className="pb-3">
                      <CardTitle className="flex items-center gap-3 text-lg">
                        <span className={`w-8 h-8 rounded-lg ${colors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>
                          {zone}
                        </span>
                        <span className={colors.text}>{zone} 區</span>
                        <Badge variant="outline" className={`ml-2 ${colors.text} ${colors.border}`}>
                          共 {totalAnimals} 隻
                        </Badge>
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="bg-white rounded-lg shadow-sm overflow-hidden">
                        {/* Table Header */}
                        <div className="grid grid-cols-2 gap-0 border-b">
                          <div className={`grid grid-cols-5 gap-1 px-3 py-2 text-xs font-semibold ${colors.header} text-white`}>
                            <div>欄位</div>
                            <div>耳號</div>
                            <div>獸醫檢視</div>
                            <div>最新異常</div>
                            <div className="text-center">操作</div>
                          </div>
                          <div className={`grid grid-cols-5 gap-1 px-3 py-2 text-xs font-semibold ${colors.header} text-white border-l border-white/30`}>
                            <div>欄位</div>
                            <div>耳號</div>
                            <div>獸醫檢視</div>
                            <div>最新異常</div>
                            <div className="text-center">操作</div>
                          </div>
                        </div>

                        {/* Table Rows */}
                        {leftColumnPens.map((leftNum, idx) => {
                          const rightNum = rightColumnPens[idx]
                          const leftPenLocation = `${zone}${leftNum}`
                          const rightPenLocation = rightNum ? `${zone}${rightNum}` : null

                          return (
                            <div key={leftNum} className={`grid grid-cols-2 gap-0 border-b last:border-b-0 ${idx % 2 === 0 ? 'bg-white' : colors.bg}`}>
                              {/* Left Column Cell */}
                              <div className={`border-r ${colors.border}`}>
                                {renderPenCell(leftPenLocation, colors)}
                              </div>
                              {/* Right Column Cell */}
                              <div>
                                {renderPenCell(rightPenLocation, colors)}
                              </div>
                            </div>
                          )
                        })}
                      </div>
                    </CardContent>
                  </Card>
                )
              }

              // Helper function to render EFG combined zone card (special layout for B building)
              const renderEFGCombinedCard = () => {
                const eColors = penZoneColors['E']
                const fColors = penZoneColors['F']
                const gColors = penZoneColors['G']

                const ePenNumbers = penNumbersByZone['E'] || []
                const fPenNumbers = penNumbersByZone['F'] || []
                const gPenNumbers = penNumbersByZone['G'] || []

                // Build right column: F01-F06 then G01-G06
                const rightColumnPens = [
                  ...fPenNumbers.map(num => `F${num}`),
                  ...gPenNumbers.map(num => `G${num}`),
                ]

                // Left column: E01-E25
                const leftColumnPens = ePenNumbers.map(num => `E${num}`)

                // Max rows
                const maxRows = Math.max(leftColumnPens.length, rightColumnPens.length)

                // Count total animals
                let eTotalAnimals = 0, fTotalAnimals = 0, gTotalAnimals = 0
                ePenNumbers.forEach(num => {
                  const penAnimals = animalsByPenLocation.get(`E${num}`) || []
                  eTotalAnimals += penAnimals.length
                })
                fPenNumbers.forEach(num => {
                  const penAnimals = animalsByPenLocation.get(`F${num}`) || []
                  fTotalAnimals += penAnimals.length
                })
                gPenNumbers.forEach(num => {
                  const penAnimals = animalsByPenLocation.get(`G${num}`) || []
                  gTotalAnimals += penAnimals.length
                })

                return (
                  <Card key="EFG" className="bg-gradient-to-r from-purple-50 via-amber-50 to-green-50 border-2 border-purple-300">
                    <CardHeader className="pb-3">
                      <CardTitle className="flex items-center gap-3 text-lg flex-wrap">
                        <div className="flex items-center gap-2">
                          <span className={`w-8 h-8 rounded-lg ${eColors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>
                            E
                          </span>
                          <span className={eColors.text}>E 區</span>
                          <Badge variant="outline" className={`${eColors.text} ${eColors.border}`}>
                            {eTotalAnimals} 隻
                          </Badge>
                        </div>
                        <span className="text-slate-300">|</span>
                        <div className="flex items-center gap-2">
                          <span className={`w-8 h-8 rounded-lg ${fColors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>
                            F
                          </span>
                          <span className={fColors.text}>F 區</span>
                          <Badge variant="outline" className={`${fColors.text} ${fColors.border}`}>
                            {fTotalAnimals} 隻
                          </Badge>
                        </div>
                        <span className="text-slate-300">|</span>
                        <div className="flex items-center gap-2">
                          <span className={`w-8 h-8 rounded-lg ${gColors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>
                            G
                          </span>
                          <span className={gColors.text}>G 區</span>
                          <Badge variant="outline" className={`${gColors.text} ${gColors.border}`}>
                            {gTotalAnimals} 隻
                          </Badge>
                        </div>
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="bg-white rounded-lg shadow-sm overflow-hidden">
                        {/* Table Header */}
                        <div className="grid grid-cols-2 gap-0 border-b">
                          <div className={`grid grid-cols-5 gap-1 px-3 py-2 text-xs font-semibold ${eColors.header} text-white`}>
                            <div>欄位</div>
                            <div>耳號</div>
                            <div>獸醫檢視</div>
                            <div>最新異常</div>
                            <div className="text-center">操作</div>
                          </div>
                          <div className="grid grid-cols-5 gap-1 px-3 py-2 text-xs font-semibold bg-gradient-to-r from-amber-500 to-green-500 text-white border-l border-white/30">
                            <div>欄位</div>
                            <div>耳號</div>
                            <div>獸醫檢視</div>
                            <div>最新異常</div>
                            <div className="text-center">操作</div>
                          </div>
                        </div>

                        {/* Table Rows */}
                        {Array.from({ length: maxRows }).map((_, idx) => {
                          const leftPenLocation = leftColumnPens[idx] || null
                          const rightPenLocation = rightColumnPens[idx] || null

                          // Determine right column colors based on zone
                          const rightZone = rightPenLocation?.charAt(0) || ''
                          const rightColors = penZoneColors[rightZone] || { bg: 'bg-gray-50', border: 'border-gray-300', header: 'bg-gray-500', text: 'text-gray-700' }

                          // Add separator when transitioning from F to G
                          const isTransition = idx > 0 && rightPenLocation?.startsWith('G') && rightColumnPens[idx - 1]?.startsWith('F')

                          return (
                            <div
                              key={idx}
                              className={`grid grid-cols-2 gap-0 border-b last:border-b-0 ${isTransition ? 'border-t-2 border-t-green-400' : ''
                                } ${idx % 2 === 0 ? 'bg-white' : 'bg-slate-50'}`}
                            >
                              {/* Left Column Cell (E zone) */}
                              <div className={`border-r ${eColors.border}`}>
                                {renderPenCell(leftPenLocation, eColors)}
                              </div>
                              {/* Right Column Cell (F or G zone) */}
                              <div className={rightPenLocation ? rightColors.bg : ''}>
                                {renderPenCell(rightPenLocation, rightColors)}
                              </div>
                            </div>
                          )
                        })}
                      </div>
                    </CardContent>
                  </Card>
                )
              }

              // Render based on building tab
              if (groupedBuildingTab === 'A') {
                // A building: A, C, D zones - each rendered independently
                return (
                  <div className="space-y-6">
                    {currentZones.map(zone => renderStandardZoneCard(zone))}
                  </div>
                )
              } else {
                // B building: B zone independent, then EFG combined
                return (
                  <div className="space-y-6">
                    {renderStandardZoneCard('B')}
                    {renderEFGCombinedCard()}
                  </div>
                )
              }
            })()
          )}
        </div>
      )}

      {/* Add Animal Dialog */}
      <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>新增動物</DialogTitle>
            <DialogDescription>輸入新動物的基本資料</DialogDescription>
          </DialogHeader>
          <div className="grid grid-cols-2 gap-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="ear_tag">耳號 *</Label>
              <Input
                id="ear_tag"
                value={newAnimal.ear_tag}
                onChange={(e) => setNewAnimal({ ...newAnimal, ear_tag: e.target.value })}
                placeholder="輸入耳號"
              />
              <p className="text-[10px] text-slate-400">若輸入數字會自動轉換為三位數（如 001）</p>
            </div>
            <div className="space-y-2">
              <Label>棟別 *</Label>
              <Select
                value={penBuilding}
                onValueChange={(v) => {
                  setPenBuilding(v)
                  setPenZone('')
                  setPenNumber('')
                }}
              >
                <SelectTrigger>
                  <SelectValue placeholder="選擇 A 棟或 B 棟" />
                </SelectTrigger>
                <SelectContent>
                  {penBuildings.map((building) => (
                    <SelectItem key={building.value} value={building.value}>
                      {building.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>欄位區 *</Label>
              <Select
                value={penZone}
                onValueChange={(v) => {
                  setPenZone(v)
                  setPenNumber('')
                }}
                disabled={!penBuilding}
              >
                <SelectTrigger>
                  <SelectValue placeholder={penBuilding ? "選擇欄位區" : "請先選棟別"} />
                </SelectTrigger>
                <SelectContent>
                  {(penZonesByBuilding[penBuilding] || []).map((zone) => (
                    <SelectItem key={zone} value={zone}>
                      {zone}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>欄位編號 *</Label>
              <Select
                value={penNumber}
                onValueChange={(v) => setPenNumber(v)}
                disabled={!penZone}
              >
                <SelectTrigger>
                  <SelectValue placeholder={penZone ? "選擇編號" : "請先選欄位區"} />
                </SelectTrigger>
                <SelectContent>
                  {(penNumbersByZone[penZone] || []).map((value) => (
                    <SelectItem key={value} value={value}>
                      {value}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>品種 *</Label>
              <Select
                value={newAnimal.breed}
                onValueChange={(v) => setNewAnimal({ ...newAnimal, breed: v as any })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="minipig">迷你豬</SelectItem>
                  <SelectItem value="white">白豬</SelectItem>
                  <SelectItem value="other">其他</SelectItem>
                </SelectContent>
              </Select>
            </div>
            {newAnimal.breed === 'other' && (
              <div className="space-y-2">
                <Label htmlFor="breed_other">填寫品種 *</Label>
                <Input
                  id="breed_other"
                  value={newAnimal.breed_other}
                  onChange={(e) => setNewAnimal({ ...newAnimal, breed_other: e.target.value })}
                  placeholder="請輸入品種名稱"
                />
              </div>
            )}
            <div className="space-y-2">
              <Label>性別 *</Label>
              <Select
                value={newAnimal.gender}
                onValueChange={(v) => setNewAnimal({ ...newAnimal, gender: v as any })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="male">公</SelectItem>
                  <SelectItem value="female">母</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label>來源</Label>
              <Select
                value={newAnimal.source_id || 'none'}
                onValueChange={(v) => setNewAnimal({ ...newAnimal, source_id: v === 'none' ? '' : v })}
              >
                <SelectTrigger>
                  <SelectValue placeholder="選擇來源" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">無</SelectItem>
                  {sourcesData?.map((source) => (
                    <SelectItem key={source.id} value={source.id}>
                      {source.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="entry_date">進場日期 *</Label>
              <Input
                id="entry_date"
                type="date"
                value={newAnimal.entry_date}
                onChange={(e) => setNewAnimal({ ...newAnimal, entry_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="birth_date">出生日期 *</Label>
              <Input
                id="birth_date"
                type="date"
                value={newAnimal.birth_date}
                onChange={(e) => setNewAnimal({ ...newAnimal, birth_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="entry_weight">進場體重 (kg) *</Label>
              <Input
                id="entry_weight"
                type="text"
                inputMode="decimal"
                value={newAnimal.entry_weight}
                onChange={(e) => {
                  const value = e.target.value
                  // 只允許數字和一個小數點
                  const numericValue = value.replace(/[^\d.]/g, '')
                  // 確保只有一個小數點
                  const parts = numericValue.split('.')
                  const filteredValue = parts.length > 2
                    ? parts[0] + '.' + parts.slice(1).join('')
                    : numericValue
                  setNewAnimal({ ...newAnimal, entry_weight: filteredValue })
                }}
                placeholder="輸入體重"
              />
            </div>
            <div className="space-y-2 col-span-2">
              <Label htmlFor="pre_experiment_code">實驗前代號 *</Label>
              <Input
                id="pre_experiment_code"
                value={newAnimal.pre_experiment_code}
                onChange={(e) => setNewAnimal({ ...newAnimal, pre_experiment_code: e.target.value })}
                placeholder="例如 PIG-110000"
              />
            </div>
            <div className="space-y-2 col-span-2">
              <Label htmlFor="remark">備註</Label>
              <Input
                id="remark"
                value={newAnimal.remark}
                onChange={(e) => setNewAnimal({ ...newAnimal, remark: e.target.value })}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAddDialog(false)}>
              取消
            </Button>
            <Button
              onClick={() => createAnimalMutation.mutate(newAnimal)}
              disabled={
                createAnimalMutation.isPending ||
                !newAnimal.ear_tag ||
                !penBuilding ||
                !penZone ||
                !penNumber ||
                !newAnimal.birth_date ||
                !newAnimal.entry_weight ||
                !newAnimal.pre_experiment_code ||
                !newAnimal.entry_date ||
                (newAnimal.breed === 'other' && !newAnimal.breed_other)
              }
              className="bg-purple-600 hover:bg-purple-700"
            >
              {createAnimalMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              新增
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      {/* Batch Assign Dialog */}
      <Dialog open={showBatchAssignDialog} onOpenChange={setShowBatchAssignDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>分配動物至計畫</DialogTitle>
            <DialogDescription>
              將選中的 {selectedAnimals.length} 隻動物分配至指定的 IACUC 計畫
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="iacuc_no">IACUC No. *</Label>
              <Input
                id="iacuc_no"
                value={assignIacucNo}
                onChange={(e) => setAssignIacucNo(e.target.value)}
                placeholder="例如 PIG-114017"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowBatchAssignDialog(false)}>
              取消
            </Button>
            <Button
              onClick={() => batchAssignMutation.mutate()}
              disabled={batchAssignMutation.isPending || !assignIacucNo}
            >
              {batchAssignMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              確認分配
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Batch Export Dialog */}
      <ExportDialog
        open={showBatchExportDialog}
        onOpenChange={setShowBatchExportDialog}
        type="batch_project"
      />

      {/* Import Basic Data Dialog */}
      <ImportDialog
        open={showImportBasicDialog}
        onOpenChange={setShowImportBasicDialog}
        type="basic"
      />

      {/* Import Weight Data Dialog */}
      <ImportDialog
        open={showImportWeightDialog}
        onOpenChange={setShowImportWeightDialog}
        type="weight"
      />

      {/* Quick Edit Dialog */}
      {quickEditAnimalId && (
        <QuickEditAnimalDialog
          open={!!quickEditAnimalId}
          onOpenChange={(open) => {
            if (!open) setQuickEditAnimalId(null)
          }}
          animalId={quickEditAnimalId}
        />
      )}

      {/* Quick Add Dialog (從欄位快速新增動物) */}
      <Dialog open={showQuickAddDialog} onOpenChange={(open) => {
        if (!open) {
          setShowQuickAddDialog(false)
          setQuickAddPending(null)
        }
      }}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>新增動物</DialogTitle>
            <DialogDescription>
              耳號 <span className="font-bold text-purple-600">{quickAddPending?.earTag}</span> 不存在，請填寫資料以新增動物至 <span className="font-bold">{quickAddPending?.penLocation}</span>
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="space-y-2">
              <Label>品種 *</Label>
              <Select
                value={quickAddForm.breed}
                onValueChange={(v) => setQuickAddForm({ ...quickAddForm, breed: v as AnimalBreed })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="minipig">迷你豬</SelectItem>
                  <SelectItem value="white">白豬</SelectItem>
                  <SelectItem value="other">其他</SelectItem>
                </SelectContent>
              </Select>
              {quickAddForm.breed === 'other' && (
                <Input
                  placeholder="請輸入品種名稱"
                  value={quickAddForm.breed_other}
                  onChange={(e) => setQuickAddForm({ ...quickAddForm, breed_other: e.target.value })}
                  className="mt-2"
                />
              )}
            </div>
            <div className="space-y-2">
              <Label>性別 *</Label>
              <Select
                value={quickAddForm.gender}
                onValueChange={(v) => setQuickAddForm({ ...quickAddForm, gender: v as 'male' | 'female' })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="male">公</SelectItem>
                  <SelectItem value="female">母</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-2">
              <Label htmlFor="quick_entry_date">進場日期 *</Label>
              <Input
                id="quick_entry_date"
                type="date"
                value={quickAddForm.entry_date}
                onChange={(e) => setQuickAddForm({ ...quickAddForm, entry_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="quick_birth_date">出生日期 *</Label>
              <Input
                id="quick_birth_date"
                type="date"
                value={quickAddForm.birth_date}
                onChange={(e) => setQuickAddForm({ ...quickAddForm, birth_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="quick_entry_weight">進場體重 (kg) *</Label>
              <Input
                id="quick_entry_weight"
                type="text"
                inputMode="decimal"
                value={quickAddForm.entry_weight}
                onChange={(e) => {
                  const value = e.target.value
                  const numericValue = value.replace(/[^\d.]/g, '')
                  const parts = numericValue.split('.')
                  const filteredValue = parts.length > 2
                    ? parts[0] + '.' + parts.slice(1).join('')
                    : numericValue
                  setQuickAddForm({ ...quickAddForm, entry_weight: filteredValue })
                }}
                placeholder="輸入體重"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => {
              setShowQuickAddDialog(false)
              setQuickAddPending(null)
            }}>
              取消
            </Button>
            <Button
              onClick={() => quickAddMutation.mutate()}
              disabled={
                quickAddMutation.isPending ||
                !quickAddForm.entry_date ||
                !quickAddForm.birth_date ||
                !quickAddForm.entry_weight ||
                (quickAddForm.breed === 'other' && !quickAddForm.breed_other)
              }
              className="bg-purple-600 hover:bg-purple-700"
            >
              {quickAddMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              確認新增
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 耳號重複警告確認對話框 */}
      <Dialog open={showDuplicateWarning} onOpenChange={(open) => {
        if (!open) {
          setShowDuplicateWarning(false)
          setDuplicateWarningData(null)
        }
      }}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-amber-600">
              <AlertTriangle className="h-5 w-5" />
              耳號重複警告
            </DialogTitle>
            <DialogDescription>
              耳號 <span className="font-semibold text-slate-900">{duplicateWarningData?.earTag}</span> 已存在以下存活動物：
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-2 my-2">
            {duplicateWarningData?.existingAnimals.map((animal, idx) => (
              <div key={idx} className="flex items-center gap-3 p-3 bg-amber-50 border border-amber-200 rounded-lg text-sm">
                <AlertTriangle className="h-4 w-4 text-amber-500 shrink-0" />
                <div>
                  <div>出生日期: <span className="font-medium">{animal.birth_date || '未設定'}</span></div>
                  <div>欄位: <span className="font-medium">{animal.pen_location || '-'}</span></div>
                </div>
              </div>
            ))}
          </div>
          <p className="text-sm text-slate-600">
            確定仍要以<span className="font-semibold">不同出生日期</span>建立新動物嗎？
          </p>
          <DialogFooter>
            <Button variant="outline" onClick={() => {
              setShowDuplicateWarning(false)
              setDuplicateWarningData(null)
            }}>
              取消
            </Button>
            <Button
              onClick={() => {
                if (duplicateWarningData?.pendingPayload) {
                  forceCreateMutation.mutate(duplicateWarningData.pendingPayload)
                }
              }}
              disabled={forceCreateMutation.isPending}
              className="bg-amber-600 hover:bg-amber-700"
            >
              {forceCreateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              確認建立
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {showPrintReport && (
        <AnimalPenReport
          data={groupedData || []}
          onClose={() => setShowPrintReport(false)}
        />
      )}
    </div>
  )
}







