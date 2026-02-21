import { useState, useEffect } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  Animal,
  AnimalObservation,
  AnimalSurgery,
  AnimalWeight,
  AnimalVaccination,
  AnimalSacrifice,
  AnimalPathologyReport,
  AnimalSuddenDeath,
  AnimalEvent,
  transferApi,
  animalStatusNames,
  allAnimalStatusNames,
  animalBreedNames,
  animalGenderNames,
  recordTypeNames,
  RecordType,
  AnimalStatus,
  ProtocolListItem,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { FileUpload, FileInfo } from '@/components/ui/file-upload'
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
import { toast } from '@/components/ui/use-toast'
import {
  ArrowLeft,
  Loader2,
  Plus,
  Eye,
  Edit2,
  Trash2,
  History,
  CheckCircle2,
  AlertCircle,
  Scale,
  Syringe,
  FileText,
  Scissors,
  ClipboardList,
  Heart,
  Download,
  ChevronDown,
  Upload,
  Copy,
  Stethoscope,
  AlertTriangle,
  AlertOctagon,
  Droplets,
  ArrowRightLeft,
  Zap,
} from 'lucide-react'

import { AnimalTimelineView } from '@/components/animal/AnimalTimelineView'

// Import form dialog components
import { ObservationFormDialog } from '@/components/animal/ObservationFormDialog'
import { SurgeryFormDialog } from '@/components/animal/SurgeryFormDialog'
import { SacrificeFormDialog } from '@/components/animal/SacrificeFormDialog'
import { ExportDialog } from '@/components/animal/ExportDialog'
import { VersionHistoryDialog } from '@/components/animal/VersionHistoryDialog'
import { VetRecommendationDialog } from '@/components/animal/VetRecommendationDialog'
import { DeleteReasonDialog } from '@/components/ui/delete-reason-dialog'
import { EmergencyMedicationDialog } from '@/components/animal/EmergencyMedicationDialog'
import { EuthanasiaOrderDialog } from '@/components/animal/EuthanasiaOrderDialog'
import { BloodTestTab } from '@/components/animal/BloodTestTab'
import { TransferTab } from '@/components/animal/TransferTab'
import { PainAssessmentTab } from '@/components/animal/PainAssessmentTab'
import { useAuthStore } from '@/stores/auth'
import { useUIPreferences } from '@/stores/uiPreferences'

const statusColors: Record<AnimalStatus, string> = {
  unassigned: 'bg-gray-500',
  in_experiment: 'bg-orange-500',
  completed: 'bg-green-500',
  euthanized: 'bg-red-500',
  sudden_death: 'bg-rose-600',
  transferred: 'bg-indigo-500',
}

// 輔助函數：判斷欄位顯示文字
const getPenLocationDisplay = (animal: { status: AnimalStatus; pen_location?: string | null }) => {
  if (animal.status === 'completed' && !animal.pen_location) {
    return '犧牲'
  }
  return animal.pen_location || '-'
}

type TabType = 'timeline' | 'observations' | 'surgeries' | 'weights' | 'vaccinations' | 'sacrifice' | 'info' | 'pathology' | 'blood_tests' | 'pain_assessment' | 'transfer'

export function AnimalDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const animalId = id!

  // Auth and UI preferences
  const { hasRole } = useAuthStore()
  const { developerMode, toggleDeveloperMode } = useUIPreferences()

  const [activeTab, setActiveTab] = useState<TabType>('timeline')

  // Dialog states
  const [showAddObservationDialog, setShowAddObservationDialog] = useState(false)
  const [showAddSurgeryDialog, setShowAddSurgeryDialog] = useState(false)
  const [showAddWeightDialog, setShowAddWeightDialog] = useState(false)
  const [showAddVaccinationDialog, setShowAddVaccinationDialog] = useState(false)
  const [showExportDialog, setShowExportDialog] = useState(false)
  const [showPathologyUploadDialog, setShowPathologyUploadDialog] = useState(false)
  const [showSacrificeDialog, setShowSacrificeDialog] = useState(false)
  const [showEmergencyMedicationDialog, setShowEmergencyMedicationDialog] = useState(false)
  const [showEuthanasiaOrderDialog, setShowEuthanasiaOrderDialog] = useState(false)
  const [showSuddenDeathDialog, setShowSuddenDeathDialog] = useState(false)
  const [showTrialSelect, setShowTrialSelect] = useState(false)

  // Edit states
  const [editingObservation, setEditingObservation] = useState<AnimalObservation | null>(null)
  const [editingSurgery, setEditingSurgery] = useState<AnimalSurgery | null>(null)

  // Version history states
  const [versionHistoryType, setVersionHistoryType] = useState<'observation' | 'surgery'>('observation')
  const [versionHistoryRecordId, setVersionHistoryRecordId] = useState<number | null>(null)
  const [showVersionHistoryDialog, setShowVersionHistoryDialog] = useState(false)

  // Vet recommendation states
  const [vetRecommendationType, setVetRecommendationType] = useState<'observation' | 'surgery'>('observation')
  const [vetRecommendationRecordId, setVetRecommendationRecordId] = useState<number | null>(null)
  const [showVetRecommendationDialog, setShowVetRecommendationDialog] = useState(false)

  // Expanded row states
  const [expandedObservation, setExpandedObservation] = useState<number | null>(null)
  const [expandedSurgery, setExpandedSurgery] = useState<number | null>(null)

  // Form states
  const [newWeight, setNewWeight] = useState({ measure_date: new Date().toISOString().split('T')[0], weight: '' })
  const [newVaccination, setNewVaccination] = useState({ administered_date: new Date().toISOString().split('T')[0], vaccine: '', deworming_dose: '' })
  const [pathologyFiles, setPathologyFiles] = useState<FileInfo[]>([])

  // 猝死登記表單
  const [suddenDeathForm, setSuddenDeathForm] = useState({
    discovered_at: new Date().toISOString().slice(0, 16), // datetime-local 格式
    probable_cause: '',
    location: '',
    remark: '',
    requires_pathology: false,
  })

  // Delete dialog states (GLP compliance)
  const [deleteObservationTarget, setDeleteObservationTarget] = useState<number | null>(null)
  const [deleteSurgeryTarget, setDeleteSurgeryTarget] = useState<number | null>(null)
  const [deleteWeightTarget, setDeleteWeightTarget] = useState<number | null>(null)
  const [deleteVaccinationTarget, setDeleteVaccinationTarget] = useState<number | null>(null)

  // Queries
  const { data: animal, isLoading: animalLoading } = useQuery({
    queryKey: ['animal', animalId],
    queryFn: async () => {
      const res = await api.get<Animal>(`/animals/${animalId}`)
      return res.data
    },
    staleTime: 0, // Always consider data stale for real-time updates
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  // 資料隔離界線查詢
  const { data: dataBoundary } = useQuery({
    queryKey: ['animal-data-boundary', animalId],
    queryFn: async () => {
      const res = await transferApi.getDataBoundary(animalId)
      return res.data
    },
  })
  const afterParam = dataBoundary?.boundary ? `?after=${encodeURIComponent(dataBoundary.boundary)}` : ''

  // 取得已核准的試驗列表（用於「未分配」狀態下拉式選單）
  const { data: approvedProtocols } = useQuery({
    queryKey: ['approved-protocols'],
    queryFn: async () => {
      const res = await api.get<ProtocolListItem[]>('/protocols?status=APPROVED')
      // 同時取得附條件核准的試驗
      const res2 = await api.get<ProtocolListItem[]>('/protocols?status=APPROVED_WITH_CONDITIONS')
      return [...res.data, ...res2.data].filter(p => p.iacuc_no)
    },
    enabled: animal?.status === 'unassigned',
  })

  // 分配動物到試驗的 mutation
  const assignTrialMutation = useMutation({
    mutationFn: async (iacucNo: string) => {
      return api.put(`/animals/${animalId}`, {
        iacuc_no: iacucNo,
        status: 'in_experiment',
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      toast({ title: '成功', description: '動物已成功分配到試驗' })
      setShowTrialSelect(false)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '分配失敗',
        variant: 'destructive',
      })
    },
  })

  const { data: observations, error: observationsError } = useQuery({
    queryKey: ['animal-observations', animalId, afterParam],
    queryFn: async () => {
      const res = await api.get<AnimalObservation[]>(`/animals/${animalId}/observations${afterParam}`)
      return res.data
    },
    enabled: activeTab === 'observations' || activeTab === 'timeline',
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  // Handle observations error
  useEffect(() => {
    if (observationsError) {
      console.error('Failed to load observations:', observationsError)
      const error = observationsError as any
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || error?.message || '載入觀察紀錄失敗',
        variant: 'destructive',
      })
    }
  }, [observationsError])

  const { data: surgeries } = useQuery({
    queryKey: ['animal-surgeries', animalId, afterParam],
    queryFn: async () => {
      const res = await api.get<AnimalSurgery[]>(`/animals/${animalId}/surgeries${afterParam}`)
      return res.data
    },
    enabled: activeTab === 'surgeries' || activeTab === 'timeline',
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: weights } = useQuery({
    queryKey: ['animal-weights', animalId, afterParam],
    queryFn: async () => {
      const res = await api.get<AnimalWeight[]>(`/animals/${animalId}/weights${afterParam}`)
      return res.data
    },
    enabled: activeTab === 'weights' || activeTab === 'timeline',
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: vaccinations } = useQuery({
    queryKey: ['animal-vaccinations', animalId, afterParam],
    queryFn: async () => {
      const res = await api.get<AnimalVaccination[]>(`/animals/${animalId}/vaccinations${afterParam}`)
      return res.data
    },
    enabled: activeTab === 'vaccinations',
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: sacrifice } = useQuery({
    queryKey: ['animal-sacrifice', animalId],
    queryFn: async () => {
      const res = await api.get<AnimalSacrifice>(`/animals/${animalId}/sacrifice`)
      return res.data
    },
    enabled: activeTab === 'sacrifice' || activeTab === 'timeline',
  })

  // 猝死記錄
  const { data: suddenDeath } = useQuery({
    queryKey: ['animal-sudden-death', animalId],
    queryFn: async () => {
      const res = await api.get<AnimalSuddenDeath>(`/animals/${animalId}/sudden-death`)
      return res.data
    },
    enabled: activeTab === 'timeline',
  })

  // IACUC No. 變更事件（時間軸用）
  const { data: iacucEvents } = useQuery({
    queryKey: ['animal-iacuc-events', animalId],
    queryFn: async () => {
      const res = await api.get<AnimalEvent[]>(`/animals/${animalId}/events`)
      return res.data
    },
    enabled: activeTab === 'timeline',
  })

  // 轉讓紀錄
  const { data: transfers } = useQuery({
    queryKey: ['animal-transfers', animalId],
    queryFn: async () => {
      const res = await transferApi.list(animalId)
      return res.data
    },
  })

  const { data: pathology } = useQuery({
    queryKey: ['animal-pathology', animalId],
    queryFn: async () => {
      const res = await api.get<AnimalPathologyReport>(`/animals/${animalId}/pathology`)
      return res.data
    },
    enabled: activeTab === 'pathology',
  })

  // Mutations
  const addWeightMutation = useMutation({
    mutationFn: async (data: typeof newWeight) => {
      return api.post(`/animals/${animalId}/weights`, {
        measure_date: data.measure_date,
        weight: parseFloat(data.weight),
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-weights', animalId] })
      toast({ title: '成功', description: '體重紀錄已新增' })
      setShowAddWeightDialog(false)
      setNewWeight({ measure_date: new Date().toISOString().split('T')[0], weight: '' })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '新增失敗',
        variant: 'destructive',
      })
    },
  })

  const addVaccinationMutation = useMutation({
    mutationFn: async (data: typeof newVaccination) => {
      return api.post(`/animals/${animalId}/vaccinations`, data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-vaccinations', animalId] })
      toast({ title: '成功', description: '疫苗紀錄已新增' })
      setShowAddVaccinationDialog(false)
      setNewVaccination({ administered_date: new Date().toISOString().split('T')[0], vaccine: '', deworming_dose: '' })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '新增失敗',
        variant: 'destructive',
      })
    },
  })

  const deleteWeightMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: number; reason: string }) => {
      return api.delete(`/weights/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-weights', animalId] })
      toast({ title: '成功', description: '體重紀錄已刪除' })
      setDeleteWeightTarget(null)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '刪除失敗',
        variant: 'destructive',
      })
    },
  })

  const deleteVaccinationMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: number; reason: string }) => {
      return api.delete(`/vaccinations/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-vaccinations', animalId] })
      toast({ title: '成功', description: '疫苗紀錄已刪除' })
      setDeleteVaccinationTarget(null)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '刪除失敗',
        variant: 'destructive',
      })
    },
  })

  const deleteObservationMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: number; reason: string }) => {
      return api.delete(`/observations/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-observations', animalId] })
      toast({ title: '成功', description: '觀察紀錄已刪除' })
      setDeleteObservationTarget(null)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '刪除失敗',
        variant: 'destructive',
      })
    },
  })

  const deleteSurgeryMutation = useMutation({
    mutationFn: async ({ id, reason }: { id: number; reason: string }) => {
      return api.delete(`/surgeries/${id}`, { data: { reason } })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-surgeries', animalId] })
      toast({ title: '成功', description: '手術紀錄已刪除' })
      setDeleteSurgeryTarget(null)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '刪除失敗',
        variant: 'destructive',
      })
    },
  })

  const copyObservationMutation = useMutation({
    mutationFn: async (sourceId: number) => {
      return api.post(`/animals/${animalId}/observations/copy`, { source_id: sourceId })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-observations', animalId] })
      toast({ title: '成功', description: '觀察紀錄已複製，請編輯新紀錄' })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '複製失敗',
        variant: 'destructive',
      })
    },
  })

  const copySurgeryMutation = useMutation({
    mutationFn: async (sourceId: number) => {
      return api.post(`/animals/${animalId}/surgeries/copy`, { source_id: sourceId })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-surgeries', animalId] })
      toast({ title: '成功', description: '手術紀錄已複製，請編輯新紀錄' })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '複製失敗',
        variant: 'destructive',
      })
    },
  })

  // 猝死登記 mutation
  const createSuddenDeathMutation = useMutation({
    mutationFn: async (data: typeof suddenDeathForm) => {
      return api.post(`/animals/${animalId}/sudden-death`, {
        discovered_at: new Date(data.discovered_at).toISOString(),
        probable_cause: data.probable_cause || undefined,
        location: data.location || undefined,
        remark: data.remark || undefined,
        requires_pathology: data.requires_pathology,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      queryClient.invalidateQueries({ queryKey: ['animal-sudden-death', animalId] })
      toast({ title: '已登記', description: '猝死紀錄已登記，動物狀態已自動更新' })
      setShowSuddenDeathDialog(false)
      setSuddenDeathForm({
        discovered_at: new Date().toISOString().slice(0, 16),
        probable_cause: '',
        location: '',
        remark: '',
        requires_pathology: false,
      })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '猝死登記失敗',
        variant: 'destructive',
      })
    },
  })

  const handleShowVersionHistory = (type: 'observation' | 'surgery', id: number) => {
    setVersionHistoryType(type)
    setVersionHistoryRecordId(id)
    setShowVersionHistoryDialog(true)
  }

  const handleShowVetRecommendation = (type: 'observation' | 'surgery', id: number) => {
    setVetRecommendationType(type)
    setVetRecommendationRecordId(id)
    setShowVetRecommendationDialog(true)
  }

  const uploadPathologyMutation = useMutation({
    mutationFn: async (files: FileInfo[]) => {
      // TODO: 實際檔案上傳實作
      return api.post(`/animals/${animalId}/pathology/upload`, {
        files: files.map((f) => ({
          file_name: f.file_name,
          file_size: f.file_size,
        })),
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-pathology', animalId] })
      toast({ title: '成功', description: '病理報告已上傳' })
      setShowPathologyUploadDialog(false)
      setPathologyFiles([])
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '上傳失敗',
        variant: 'destructive',
      })
    },
  })

  if (animalLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
      </div>
    )
  }

  if (!animal) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px]">
        <AlertCircle className="h-12 w-12 text-slate-400 mb-4" />
        <p className="text-slate-500">找不到此動物</p>
        <Button variant="outline" className="mt-4" onClick={() => navigate('/animals')}>
          返回列表
        </Button>
      </div>
    )
  }

  const tabs = [
    { id: 'timeline' as const, label: '紀錄時間軸', icon: History },
    { id: 'observations' as const, label: '觀察試驗紀錄', icon: ClipboardList },
    { id: 'surgeries' as const, label: '手術紀錄', icon: Scissors },
    { id: 'weights' as const, label: '體重紀錄', icon: Scale },
    { id: 'vaccinations' as const, label: '疫苗/驅蟲紀錄', icon: Syringe },
    { id: 'sacrifice' as const, label: '犧牲/採樣紀錄', icon: Heart },
    { id: 'blood_tests' as const, label: '血液檢查', icon: Droplets },
    { id: 'pain_assessment' as const, label: '疼痛評估', icon: Stethoscope },
    { id: 'info' as const, label: '動物資料', icon: FileText },
    { id: 'pathology' as const, label: '病理組織報告', icon: FileText },
    // 轉讓 Tab：僅在 completed / transferred 狀態顯示
    ...((animal.status === 'completed' || animal.status === 'transferred')
      ? [{ id: 'transfer' as const, label: '轉讓管理', icon: ArrowRightLeft }]
      : []),
  ]

  return (
    <div className="space-y-6">
      {/* Back Button & Export */}
      <div className="flex items-center justify-between">
        <Link to="/animals" className="inline-flex items-center text-slate-600 hover:text-slate-900">
          <ArrowLeft className="h-4 w-4 mr-2" />
          回到動物列表
        </Link>
        <Button variant="outline" onClick={() => setShowExportDialog(true)}>
          <Download className="h-4 w-4 mr-2" />
          匯出病歷
        </Button>
      </div>

      {/* Emergency Actions - Only for VET role and active animals */}
      {(animal.status === 'in_experiment' || animal.status === 'completed') && (
        <div className="flex gap-2">
          {animal.status === 'in_experiment' && (
            <>
              <Button
                variant="outline"
                className="border-amber-500 text-amber-600 hover:bg-amber-50"
                onClick={() => setShowEmergencyMedicationDialog(true)}
              >
                <AlertTriangle className="h-4 w-4 mr-2" />
                緊急給藥
              </Button>
              <Button
                variant="outline"
                className="border-red-500 text-red-600 hover:bg-red-50"
                onClick={() => setShowEuthanasiaOrderDialog(true)}
              >
                <AlertOctagon className="h-4 w-4 mr-2" />
                開立安樂死單
              </Button>
            </>
          )}
          <Button
            variant="outline"
            className="border-rose-500 text-rose-600 hover:bg-rose-50"
            onClick={() => setShowSuddenDeathDialog(true)}
          >
            <Zap className="h-4 w-4 mr-2" />
            登記猝死
          </Button>
        </div>
      )}

      {/* Animal Header Card */}
      <Card className="bg-gradient-to-r from-slate-50 to-blue-50 border-slate-200">
        <CardContent className="pt-6">
          <div className="grid grid-cols-3 gap-6">
            <div className="space-y-3">
              <div>
                <span className="text-sm text-slate-500">耳號</span>
                <p className="text-2xl font-bold text-orange-600">{animal.ear_tag}</p>
              </div>
              <div>
                <span className="text-sm text-slate-500">欄號</span>
                <p className="font-medium">{getPenLocationDisplay(animal)}</p>
              </div>
              <div>
                <span className="text-sm text-slate-500">品種</span>
                <p className="font-medium">{animal.breed === 'other' ? (animal.breed_other || '其他') : animalBreedNames[animal.breed]}</p>
              </div>
            </div>
            <div className="space-y-3">
              <div>
                <span className="text-sm text-slate-500">出生日期</span>
                <p className="font-medium">
                  {animal.birth_date ? new Date(animal.birth_date).toLocaleDateString('zh-TW') : '-'}
                </p>
              </div>
              <div>
                <span className="text-sm text-slate-500">IACUC No.</span>
                <p className="font-medium">{animal.iacuc_no || '未分配'}</p>
              </div>
              {animal.status !== 'unassigned' && (animal.experiment_assigned_by_name || animal.experiment_date) && (
                <div>
                  <span className="text-sm text-slate-500">實驗分配</span>
                  <p className="font-medium text-sm">
                    {animal.experiment_assigned_by_name && (
                      <span>{animal.experiment_assigned_by_name}</span>
                    )}
                    {animal.experiment_date && (
                      <span className="text-slate-400 ml-1">
                        ({new Date(animal.experiment_date).toLocaleDateString('zh-TW')})
                      </span>
                    )}
                  </p>
                </div>
              )}
              <div>
                <span className="text-sm text-slate-500">最近體重</span>
                <p className="font-medium">
                  {weights && weights.length > 0
                    ? `${weights[0].weight} kg`
                    : animal.entry_weight
                      ? `${animal.entry_weight} kg (進場)`
                      : '-'
                  }
                </p>
              </div>
            </div>
            <div className="space-y-3">
              <div>
                <span className="text-sm text-slate-500">系統號</span>
                <p className="font-medium" title={animal.id}>{animal.id.slice(0, 8)}</p>
              </div>
              <div>
                <span className="text-sm text-slate-500">動物狀態</span>
                <p className="mt-1">
                  {animal.status === 'unassigned' && !showTrialSelect ? (
                    <Badge
                      className={`${statusColors[animal.status]} text-white cursor-pointer hover:bg-gray-600 transition-colors`}
                      onClick={() => setShowTrialSelect(true)}
                      title="點擊分配試驗"
                    >
                      {allAnimalStatusNames[animal.status]} ▾
                    </Badge>
                  ) : animal.status === 'unassigned' && showTrialSelect ? (
                    <div className="flex flex-col gap-1">
                      <Select
                        onValueChange={(value) => {
                          if (value === '__cancel__') {
                            setShowTrialSelect(false)
                          } else {
                            assignTrialMutation.mutate(value)
                          }
                        }}
                        disabled={assignTrialMutation.isPending}
                      >
                        <SelectTrigger className="w-[220px] h-8 text-xs">
                          <SelectValue placeholder="選擇試驗..." />
                        </SelectTrigger>
                        <SelectContent>
                          {approvedProtocols && approvedProtocols.length > 0 ? (
                            approvedProtocols.map((protocol) => (
                              <SelectItem key={protocol.id} value={protocol.iacuc_no!}>
                                {protocol.iacuc_no} - {protocol.title}
                              </SelectItem>
                            ))
                          ) : (
                            <SelectItem value="__none__" disabled>
                              目前無進行中的試驗
                            </SelectItem>
                          )}
                        </SelectContent>
                      </Select>
                      <button
                        className="text-xs text-slate-400 hover:text-slate-600 text-left"
                        onClick={() => setShowTrialSelect(false)}
                      >
                        取消
                      </button>
                    </div>
                  ) : (
                    <Badge className={`${statusColors[animal.status]} text-white`}>
                      {allAnimalStatusNames[animal.status]}
                    </Badge>
                  )}
                </p>
              </div>
              <div>
                <span className="text-sm text-slate-500">性別</span>
                <p className="font-medium">{animalGenderNames[animal.gender]}</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Tabs */}
      <div className="border-b">
        <div className="flex space-x-1 overflow-x-auto">
          {tabs.map((tab) => {
            const Icon = tab.icon
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors whitespace-nowrap ${activeTab === tab.id
                  ? 'border-orange-500 text-orange-600'
                  : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
                  }`}
              >
                <Icon className="h-4 w-4" />
                {tab.label}
              </button>
            )
          })}
        </div>
      </div>

      {/* Tab Content */}
      <div className="mt-6">
        {/* 紀錄時間軸 Tab */}
        {activeTab === 'timeline' && (
          <AnimalTimelineView
            observations={observations || []}
            surgeries={surgeries || []}
            animalWeights={weights || []}
            sacrifice={sacrifice || undefined}
            suddenDeath={suddenDeath || undefined}
            transfers={transfers || []}
            iacucEvents={iacucEvents || []}
            animal={animal}
            onView={(type, id) => {
              if (type === 'observation') setExpandedObservation(expandedObservation === id ? null : id)
              else setExpandedSurgery(expandedSurgery === id ? null : id)
              setActiveTab(type === 'observation' ? 'observations' : 'surgeries')
            }}
            onEdit={(type, record) => {
              if (type === 'observation') {
                setEditingObservation(record)
                setShowAddObservationDialog(true)
              } else {
                setEditingSurgery(record)
                setShowAddSurgeryDialog(true)
              }
            }}
            onCopy={(type, id) => {
              if (type === 'observation') {
                if (confirm('確定要複製此紀錄？')) copyObservationMutation.mutate(id)
              } else {
                if (confirm('確定要複製此紀錄？')) copySurgeryMutation.mutate(id)
              }
            }}
            onHistory={handleShowVersionHistory}
            onVet={handleShowVetRecommendation}
            onDelete={(type, id) => {
              if (type === 'observation') setDeleteObservationTarget(id)
              else setDeleteSurgeryTarget(id)
            }}
          />
        )}

        {/* 觀察試驗紀錄 Tab */}
        {activeTab === 'observations' && (
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>觀察試驗紀錄</CardTitle>
                <CardDescription>記錄日常觀察、異常狀況與試驗操作</CardDescription>
              </div>
              <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowAddObservationDialog(true)}>
                <Plus className="h-4 w-4 mr-2" />
                新增紀錄
              </Button>
            </CardHeader>
            <CardContent>
              {!observations || observations.length === 0 ? (
                <div className="text-center py-12 text-slate-500">
                  <ClipboardList className="h-12 w-12 mx-auto mb-4 text-slate-300" />
                  <p>尚無觀察試驗紀錄</p>
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-10"></TableHead>
                      <TableHead>事件日期</TableHead>
                      <TableHead>紀錄性質</TableHead>
                      <TableHead>內容</TableHead>
                      <TableHead>停止用藥</TableHead>
                      <TableHead>獸醫師讀取</TableHead>
                      <TableHead>記錄者</TableHead>
                      <TableHead className="text-right">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {observations.map((obs: AnimalObservation) => (
                      <>
                        <TableRow key={obs.id} className="cursor-pointer hover:bg-slate-50">
                          <TableCell>
                            <button
                              onClick={() => setExpandedObservation(expandedObservation === obs.id ? null : obs.id)}
                              className="p-1 hover:bg-slate-200 rounded"
                            >
                              <ChevronDown
                                className={`h-4 w-4 transition-transform ${expandedObservation === obs.id ? 'rotate-180' : ''}`}
                              />
                            </button>
                          </TableCell>
                          <TableCell>{new Date(obs.event_date).toLocaleDateString('zh-TW')}</TableCell>
                          <TableCell>
                            <Badge variant="outline">{recordTypeNames[obs.record_type as RecordType]}</Badge>
                          </TableCell>
                          <TableCell className="max-w-xs truncate">{obs.content}</TableCell>
                          <TableCell>
                            {obs.no_medication_needed ? (
                              <CheckCircle2 className="h-4 w-4 text-green-500" />
                            ) : (
                              <span className="text-slate-400">-</span>
                            )}
                          </TableCell>
                          <TableCell>
                            {obs.vet_read ? (
                              <Badge className="bg-green-100 text-green-800">已讀</Badge>
                            ) : (
                              <Badge variant="outline" className="text-slate-500">未讀</Badge>
                            )}
                          </TableCell>
                          <TableCell>{obs.created_by_name || '-'}</TableCell>
                          <TableCell className="text-right">
                            <div className="flex items-center justify-end gap-1">
                              <Button variant="ghost" size="icon" onClick={() => setExpandedObservation(obs.id)} title="檢視詳情">
                                <Eye className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => {
                                  setEditingObservation(obs)
                                  setShowAddObservationDialog(true)
                                }}
                                title="編輯"
                              >
                                <Edit2 className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => {
                                  if (confirm('確定要複製此紀錄？將建立一份新紀錄供編輯。')) {
                                    copyObservationMutation.mutate(obs.id)
                                  }
                                }}
                                disabled={copyObservationMutation.isPending}
                                title="複製"
                              >
                                <Copy className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => handleShowVersionHistory('observation', obs.id)}
                                title="版本歷史"
                              >
                                <History className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => handleShowVetRecommendation('observation', obs.id)}
                                title="獸醫師建議"
                                className="text-green-600 hover:text-green-700"
                              >
                                <Stethoscope className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => setDeleteObservationTarget(obs.id)}
                                title="刪除"
                              >
                                <Trash2 className="h-4 w-4 text-red-500" />
                              </Button>
                            </div>
                          </TableCell>
                        </TableRow>
                        {/* 展開的詳細內容 */}
                        {expandedObservation === obs.id && (
                          <TableRow>
                            <TableCell colSpan={8} className="bg-slate-50 p-4">
                              <div className="grid grid-cols-2 gap-4">
                                <div>
                                  <Label className="text-slate-500">使用儀器</Label>
                                  <p>{obs.equipment_used?.join(', ') || '-'}</p>
                                </div>
                                <div>
                                  <Label className="text-slate-500">麻醉時間</Label>
                                  <p>
                                    {obs.anesthesia_start && obs.anesthesia_end
                                      ? `${obs.anesthesia_start} - ${obs.anesthesia_end}`
                                      : '-'}
                                  </p>
                                </div>
                                <div className="col-span-2">
                                  <Label className="text-slate-500">詳細內容</Label>
                                  <p className="whitespace-pre-wrap">{obs.content}</p>
                                </div>
                                {obs.treatments && obs.treatments.length > 0 && (
                                  <div className="col-span-2">
                                    <Label className="text-slate-500">治療方式</Label>
                                    <div className="space-y-1 mt-1">
                                      {obs.treatments.map((t: { drug: string; dosage: string; end_date?: string }, i: number) => (
                                        <p key={i}>
                                          {t.drug} - {t.dosage}
                                          {t.end_date && ` (至 ${t.end_date})`}
                                        </p>
                                      ))}
                                    </div>
                                  </div>
                                )}
                                {obs.remark && (
                                  <div className="col-span-2">
                                    <Label className="text-slate-500">備註</Label>
                                    <p>{obs.remark}</p>
                                  </div>
                                )}
                              </div>
                            </TableCell>
                          </TableRow>
                        )}
                      </>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        )}

        {/* 手術紀錄 Tab */}
        {activeTab === 'surgeries' && (
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>手術紀錄</CardTitle>
                <CardDescription>記錄手術過程、麻醉資訊與術後照護</CardDescription>
              </div>
              <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowAddSurgeryDialog(true)}>
                <Plus className="h-4 w-4 mr-2" />
                新增紀錄
              </Button>
            </CardHeader>
            <CardContent>
              {!surgeries || surgeries.length === 0 ? (
                <div className="text-center py-12 text-slate-500">
                  <Scissors className="h-12 w-12 mx-auto mb-4 text-slate-300" />
                  <p>尚無手術紀錄</p>
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-10"></TableHead>
                      <TableHead>是否首次</TableHead>
                      <TableHead>手術日期</TableHead>
                      <TableHead>手術部位</TableHead>
                      <TableHead>停止用藥</TableHead>
                      <TableHead>獸醫師讀取</TableHead>
                      <TableHead>記錄者</TableHead>
                      <TableHead className="text-right">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {surgeries.map((surgery) => (
                      <>
                        <TableRow key={surgery.id} className="cursor-pointer hover:bg-slate-50">
                          <TableCell>
                            <button
                              onClick={() => setExpandedSurgery(expandedSurgery === surgery.id ? null : surgery.id)}
                              className="p-1 hover:bg-slate-200 rounded"
                            >
                              <ChevronDown
                                className={`h-4 w-4 transition-transform ${expandedSurgery === surgery.id ? 'rotate-180' : ''}`}
                              />
                            </button>
                          </TableCell>
                          <TableCell>{surgery.is_first_experiment ? '是' : '否'}</TableCell>
                          <TableCell>{new Date(surgery.surgery_date).toLocaleDateString('zh-TW')}</TableCell>
                          <TableCell className="max-w-xs truncate">{surgery.surgery_site}</TableCell>
                          <TableCell>
                            {surgery.no_medication_needed ? (
                              <CheckCircle2 className="h-4 w-4 text-green-500" />
                            ) : (
                              <span className="text-slate-400">-</span>
                            )}
                          </TableCell>
                          <TableCell>
                            {surgery.vet_read ? (
                              <Badge className="bg-green-100 text-green-800">已讀</Badge>
                            ) : (
                              <Badge variant="outline" className="text-slate-500">未讀</Badge>
                            )}
                          </TableCell>
                          <TableCell>{surgery.created_by_name || '-'}</TableCell>
                          <TableCell className="text-right">
                            <div className="flex items-center justify-end gap-1">
                              <Button variant="ghost" size="icon" onClick={() => setExpandedSurgery(surgery.id)} title="檢視詳情">
                                <Eye className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => {
                                  setEditingSurgery(surgery)
                                  setShowAddSurgeryDialog(true)
                                }}
                                title="編輯"
                              >
                                <Edit2 className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => {
                                  if (confirm('確定要複製此紀錄？將建立一份新紀錄供編輯。')) {
                                    copySurgeryMutation.mutate(surgery.id)
                                  }
                                }}
                                disabled={copySurgeryMutation.isPending}
                                title="複製"
                              >
                                <Copy className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => handleShowVersionHistory('surgery', surgery.id)}
                                title="版本歷史"
                              >
                                <History className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => handleShowVetRecommendation('surgery', surgery.id)}
                                title="獸醫師建議"
                                className="text-green-600 hover:text-green-700"
                              >
                                <Stethoscope className="h-4 w-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() => setDeleteSurgeryTarget(surgery.id)}
                                title="刪除"
                              >
                                <Trash2 className="h-4 w-4 text-red-500" />
                              </Button>
                            </div>
                          </TableCell>
                        </TableRow>
                        {/* 展開的詳細內容 */}
                        {expandedSurgery === surgery.id && (
                          <TableRow>
                            <TableCell colSpan={8} className="bg-slate-50 p-4">
                              <div className="grid grid-cols-3 gap-4">
                                <div>
                                  <Label className="text-slate-500">誘導麻醉</Label>
                                  <p>
                                    {surgery.induction_anesthesia
                                      ? Object.entries(surgery.induction_anesthesia as Record<string, string>)
                                        .filter(([k]) => k !== 'others')
                                        .map(([k, v]) => `${k}: ${v}`)
                                        .join(', ') || '-'
                                      : '-'}
                                  </p>
                                </div>
                                <div>
                                  <Label className="text-slate-500">麻醉維持</Label>
                                  <p>
                                    {surgery.anesthesia_maintenance
                                      ? Object.entries(surgery.anesthesia_maintenance as Record<string, string>)
                                        .filter(([k]) => k !== 'others')
                                        .map(([k, v]) => `${k}: ${v}`)
                                        .join(', ') || '-'
                                      : '-'}
                                  </p>
                                </div>
                                <div>
                                  <Label className="text-slate-500">固定姿勢</Label>
                                  <p>{surgery.positioning || '-'}</p>
                                </div>
                                {surgery.anesthesia_observation && (
                                  <div className="col-span-3">
                                    <Label className="text-slate-500">麻醉觀察過程</Label>
                                    <p className="whitespace-pre-wrap">{surgery.anesthesia_observation}</p>
                                  </div>
                                )}
                                {surgery.vital_signs && surgery.vital_signs.length > 0 && (
                                  <div className="col-span-3">
                                    <Label className="text-slate-500">生理數值</Label>
                                    <div className="mt-2 overflow-x-auto">
                                      <table className="min-w-full text-sm">
                                        <thead>
                                          <tr className="border-b">
                                            <th className="px-2 py-1 text-left">時間</th>
                                            <th className="px-2 py-1 text-left">心跳</th>
                                            <th className="px-2 py-1 text-left">呼吸</th>
                                            <th className="px-2 py-1 text-left">體溫</th>
                                            <th className="px-2 py-1 text-left">SPO2</th>
                                          </tr>
                                        </thead>
                                        <tbody>
                                          {surgery.vital_signs.map((vs, i) => (
                                            <tr key={i} className="border-b">
                                              <td className="px-2 py-1">{vs.time}</td>
                                              <td className="px-2 py-1">{vs.heart_rate}/分</td>
                                              <td className="px-2 py-1">{vs.respiration_rate}/分</td>
                                              <td className="px-2 py-1">{vs.temperature}°C</td>
                                              <td className="px-2 py-1">{vs.spo2}%</td>
                                            </tr>
                                          ))}
                                        </tbody>
                                      </table>
                                    </div>
                                  </div>
                                )}
                                {surgery.reflex_recovery && (
                                  <div className="col-span-3">
                                    <Label className="text-slate-500">反射恢復觀察</Label>
                                    <p>{surgery.reflex_recovery}</p>
                                  </div>
                                )}
                                {surgery.remark && (
                                  <div className="col-span-3">
                                    <Label className="text-slate-500">備註</Label>
                                    <p>{surgery.remark}</p>
                                  </div>
                                )}
                              </div>
                            </TableCell>
                          </TableRow>
                        )}
                      </>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        )}

        {/* 體重紀錄 Tab */}
        {activeTab === 'weights' && (
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>體重紀錄</CardTitle>
                <CardDescription>記錄動物體重變化歷程</CardDescription>
              </div>
              <div className="flex items-center gap-2">
                {hasRole('admin') && (
                  <label className="flex items-center gap-2 text-sm text-slate-500 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={developerMode}
                      onChange={() => toggleDeveloperMode()}
                      className="rounded"
                    />
                    顯示系統號
                  </label>
                )}
                <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowAddWeightDialog(true)}>
                  <Plus className="h-4 w-4 mr-2" />
                  新增紀錄
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              {!weights || weights.length === 0 ? (
                <div className="text-center py-12 text-slate-500">
                  <Scale className="h-12 w-12 mx-auto mb-4 text-slate-300" />
                  <p>尚無體重紀錄</p>
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      {developerMode && <TableHead>系統號</TableHead>}
                      <TableHead>測量日期</TableHead>
                      <TableHead>體重 (kg)</TableHead>
                      <TableHead>記錄者</TableHead>
                      <TableHead>建立時間</TableHead>
                      <TableHead className="text-right">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {weights.map((weight) => (
                      <TableRow key={weight.id} data-record-id={weight.id}>
                        {developerMode && <TableCell>{weight.id}</TableCell>}
                        <TableCell>{new Date(weight.measure_date).toLocaleDateString('zh-TW')}</TableCell>
                        <TableCell className="font-medium">{weight.weight}</TableCell>
                        <TableCell>{weight.created_by_name || '-'}</TableCell>
                        <TableCell>{new Date(weight.created_at).toLocaleString('zh-TW')}</TableCell>
                        <TableCell className="text-right">
                          <div className="flex items-center justify-end gap-1">
                            <Button
                              variant="ghost"
                              size="icon"
                              title={`系統號: ${weight.id} - 點擊編輯`}
                            >
                              <Edit2 className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => setDeleteWeightTarget(weight.id)}
                              title={`系統號: ${weight.id} - 點擊刪除`}
                            >
                              <Trash2 className="h-4 w-4 text-red-500" />
                            </Button>
                          </div>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        )}

        {/* 疫苗/驅蟲紀錄 Tab */}
        {activeTab === 'vaccinations' && (
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>疫苗/驅蟲紀錄</CardTitle>
                <CardDescription>記錄疫苗接種與驅蟲紀錄</CardDescription>
              </div>
              <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowAddVaccinationDialog(true)}>
                <Plus className="h-4 w-4 mr-2" />
                新增紀錄
              </Button>
            </CardHeader>
            <CardContent>
              {!vaccinations || vaccinations.length === 0 ? (
                <div className="text-center py-12 text-slate-500">
                  <Syringe className="h-12 w-12 mx-auto mb-4 text-slate-300" />
                  <p>尚無疫苗/驅蟲紀錄</p>
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>施打日期</TableHead>
                      <TableHead>疫苗</TableHead>
                      <TableHead>驅蟲劑量</TableHead>
                      <TableHead>記錄者</TableHead>
                      <TableHead>建立時間</TableHead>
                      <TableHead className="text-right">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {vaccinations.map((vac) => (
                      <TableRow key={vac.id}>
                        <TableCell>{new Date(vac.administered_date).toLocaleDateString('zh-TW')}</TableCell>
                        <TableCell>{vac.vaccine || '-'}</TableCell>
                        <TableCell>{vac.deworming_dose || '-'}</TableCell>
                        <TableCell>{vac.created_by_name || '-'}</TableCell>
                        <TableCell>{new Date(vac.created_at).toLocaleString('zh-TW')}</TableCell>
                        <TableCell className="text-right">
                          <div className="flex items-center justify-end gap-1">
                            <Button variant="ghost" size="icon">
                              <Edit2 className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => setDeleteVaccinationTarget(vac.id)}
                            >
                              <Trash2 className="h-4 w-4 text-red-500" />
                            </Button>
                          </div>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        )}

        {/* 犧牲/採樣紀錄 Tab */}
        {activeTab === 'sacrifice' && (
          <Card>
            <CardHeader>
              <CardTitle>犧牲/採樣紀錄</CardTitle>
              <CardDescription>記錄實驗結束後的犧牲與採樣資訊</CardDescription>
            </CardHeader>
            <CardContent>
              {!sacrifice ? (
                <div className="text-center py-12 text-slate-500">
                  <Heart className="h-12 w-12 mx-auto mb-4 text-slate-300" />
                  <p>尚無犧牲/採樣紀錄</p>
                  <Button
                    className="mt-4 bg-purple-600 hover:bg-purple-700 text-white"
                    onClick={() => setShowSacrificeDialog(true)}
                  >
                    <Plus className="h-4 w-4 mr-2" />
                    建立紀錄
                  </Button>
                </div>
              ) : (
                <div className="space-y-4">
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label className="text-slate-500">犧牲日期</Label>
                      <p className="font-medium">
                        {sacrifice.sacrifice_date
                          ? new Date(sacrifice.sacrifice_date).toLocaleDateString('zh-TW')
                          : '-'
                        }
                      </p>
                    </div>
                    <div>
                      <Label className="text-slate-500">確定犧牲</Label>
                      <p className="font-medium">
                        {sacrifice.confirmed_sacrifice ? (
                          <Badge className="bg-red-100 text-red-800">已確認</Badge>
                        ) : '否'}
                      </p>
                    </div>
                    <div>
                      <Label className="text-slate-500">Zoletil-50 (ml)</Label>
                      <p className="font-medium">{sacrifice.zoletil_dose || '-'}</p>
                    </div>
                    <div>
                      <Label className="text-slate-500">200V電擊</Label>
                      <p className="font-medium">{sacrifice.method_electrocution ? '是' : '否'}</p>
                    </div>
                    <div>
                      <Label className="text-slate-500">放血</Label>
                      <p className="font-medium">{sacrifice.method_bloodletting ? '是' : '否'}</p>
                    </div>
                    <div>
                      <Label className="text-slate-500">其他方式</Label>
                      <p className="font-medium">{sacrifice.method_other || '-'}</p>
                    </div>
                    <div>
                      <Label className="text-slate-500">採樣</Label>
                      <p className="font-medium">{sacrifice.sampling || '-'}</p>
                    </div>
                    <div>
                      <Label className="text-slate-500">血液採樣 (ml)</Label>
                      <p className="font-medium">{sacrifice.blood_volume_ml || '-'}</p>
                    </div>
                  </div>
                  <div className="flex justify-end">
                    <Button
                      className="bg-purple-600 hover:bg-purple-700 text-white"
                      onClick={() => setShowSacrificeDialog(true)}
                    >
                      <Edit2 className="h-4 w-4 mr-2" />
                      編輯
                    </Button>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        )}

        {/* 動物資料 Tab */}
        {activeTab === 'info' && (
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>動物資料</CardTitle>
                <CardDescription>動物基本資料</CardDescription>
              </div>
              <Button className="bg-purple-600 hover:bg-purple-700 text-white" asChild>
                <Link to={`/animals/${animal.id}/edit`}>
                  <Edit2 className="h-4 w-4 mr-2" />
                  編輯
                </Link>
              </Button>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 gap-x-8 gap-y-4">
                <div>
                  <Label className="text-slate-500">耳號</Label>
                  <p className="font-medium">{animal.ear_tag}</p>
                </div>
                <div>
                  <Label className="text-slate-500">動物狀態</Label>
                  <p className="font-medium">{allAnimalStatusNames[animal.status]}</p>
                </div>
                <div>
                  <Label className="text-slate-500">進場日期</Label>
                  <p className="font-medium">{new Date(animal.entry_date).toLocaleDateString('zh-TW')}</p>
                </div>
                <div>
                  <Label className="text-slate-500">品種</Label>
                  <p className="font-medium">{animalBreedNames[animal.breed]}</p>
                </div>
                <div>
                  <Label className="text-slate-500">來源</Label>
                  <p className="font-medium">{animal.source_name || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">進場體重 (kg)</Label>
                  <p className="font-medium">{animal.entry_weight || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">性別</Label>
                  <p className="font-medium">{animalGenderNames[animal.gender]}</p>
                </div>
                <div>
                  <Label className="text-slate-500">出生日期</Label>
                  <p className="font-medium">
                    {animal.birth_date ? new Date(animal.birth_date).toLocaleDateString('zh-TW') : '-'}
                  </p>
                </div>
                <div>
                  <Label className="text-slate-500">實驗前代號</Label>
                  <p className="font-medium">{animal.pre_experiment_code || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">IACUC No.</Label>
                  <p className="font-medium">{animal.iacuc_no || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">實驗日期</Label>
                  <p className="font-medium">
                    {animal.experiment_date ? new Date(animal.experiment_date).toLocaleDateString('zh-TW') : '-'}
                  </p>
                </div>
                <div>
                  <Label className="text-slate-500">欄位</Label>
                  <p className="font-medium">{getPenLocationDisplay(animal)}</p>
                </div>
                <div className="col-span-2">
                  <Label className="text-slate-500">備註</Label>
                  <p className="font-medium">{animal.remark || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">系統號</Label>
                  <p className="font-medium" title={animal.id}>{animal.id.slice(0, 8)}</p>
                </div>
                <div>
                  <Label className="text-slate-500">建立時間</Label>
                  <p className="font-medium">{new Date(animal.created_at).toLocaleString('zh-TW')}</p>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* 血液檢查 Tab */}
        {activeTab === 'blood_tests' && (
          <BloodTestTab animalId={animalId} afterParam={afterParam} />
        )}

        {/* 疼痛評估 Tab */}
        {activeTab === 'pain_assessment' && (
          <PainAssessmentTab
            animalId={animalId}
            observations={(observations || []).map(o => ({ id: o.id, observation_date: o.event_date }))}
            surgeries={(surgeries || []).map(s => ({ id: s.id, surgery_date: s.surgery_date }))}
          />
        )}

        {/* 轉讓管理 Tab */}
        {activeTab === 'transfer' && (
          <TransferTab
            animalId={animalId}
            animalStatus={animal.status}
            earTag={animal.ear_tag}
          />
        )}

        {/* 病理組織報告 Tab */}
        {activeTab === 'pathology' && (
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>病理組織報告</CardTitle>
                <CardDescription>病理組織報告檔案</CardDescription>
              </div>
              <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowPathologyUploadDialog(true)}>
                <Upload className="h-4 w-4 mr-2" />
                上傳檔案
              </Button>
            </CardHeader>
            <CardContent>
              {!pathology || !pathology.attachments || pathology.attachments.length === 0 ? (
                <div className="text-center py-12 text-slate-500">
                  <FileText className="h-12 w-12 mx-auto mb-4 text-slate-300" />
                  <p>尚無病理組織報告</p>
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>檔案名稱</TableHead>
                      <TableHead>檔案大小</TableHead>
                      <TableHead>上傳時間</TableHead>
                      <TableHead className="text-right">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {pathology.attachments.map((file) => (
                      <TableRow key={file.id}>
                        <TableCell className="font-medium">{file.file_name}</TableCell>
                        <TableCell>{(file.file_size / 1024).toFixed(2)} KB</TableCell>
                        <TableCell>{new Date(file.created_at).toLocaleString('zh-TW')}</TableCell>
                        <TableCell className="text-right">
                          <Button variant="outline" size="sm">
                            <Download className="h-4 w-4 mr-2" />
                            下載
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </CardContent>
          </Card>
        )}
      </div>

      {/* Observation Form Dialog */}
      <ObservationFormDialog
        open={showAddObservationDialog}
        onOpenChange={(open) => {
          setShowAddObservationDialog(open)
          if (!open) setEditingObservation(null)
        }}
        animalId={animalId}
        earTag={animal.ear_tag}
        observation={editingObservation || undefined}
      />

      {/* Surgery Form Dialog */}
      <SurgeryFormDialog
        open={showAddSurgeryDialog}
        onOpenChange={(open) => {
          setShowAddSurgeryDialog(open)
          if (!open) setEditingSurgery(null)
        }}
        animalId={animalId}
        earTag={animal.ear_tag}
        surgery={editingSurgery || undefined}
      />

      {/* Export Dialog */}
      <ExportDialog
        open={showExportDialog}
        onOpenChange={setShowExportDialog}
        type="single_animal"
        animalId={animalId}
        earTag={animal.ear_tag}
      />

      {/* Pathology Upload Dialog */}
      <Dialog open={showPathologyUploadDialog} onOpenChange={setShowPathologyUploadDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>上傳病理組織報告</DialogTitle>
            <DialogDescription>耳號：{animal.ear_tag}</DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <FileUpload
              value={pathologyFiles}
              onChange={setPathologyFiles}
              accept=".pdf,.doc,.docx,.jpg,.jpeg,.png,.tiff"
              placeholder="拖曳病理報告檔案到此處，或點擊選擇檔案"
              maxSize={50}
              maxFiles={20}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowPathologyUploadDialog(false)}>
              取消
            </Button>
            <Button
              onClick={() => uploadPathologyMutation.mutate(pathologyFiles)}
              disabled={uploadPathologyMutation.isPending || pathologyFiles.length === 0}
              className="bg-green-600 hover:bg-green-700"
            >
              {uploadPathologyMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              上傳
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Add Weight Dialog */}
      <Dialog open={showAddWeightDialog} onOpenChange={setShowAddWeightDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新增體重紀錄</DialogTitle>
            <DialogDescription>耳號：{animal.ear_tag}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="weight_date">測量日期 *</Label>
              <Input
                id="weight_date"
                type="date"
                value={newWeight.measure_date}
                onChange={(e) => setNewWeight({ ...newWeight, measure_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="weight_value">體重 (kg) *</Label>
              <Input
                id="weight_value"
                type="number"
                step="0.1"
                value={newWeight.weight}
                onChange={(e) => setNewWeight({ ...newWeight, weight: e.target.value })}
                placeholder="輸入體重"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAddWeightDialog(false)}>
              取消
            </Button>
            <Button
              onClick={() => addWeightMutation.mutate(newWeight)}
              disabled={addWeightMutation.isPending || !newWeight.weight || !newWeight.measure_date}
              className="bg-green-600 hover:bg-green-700"
            >
              {addWeightMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Add Vaccination Dialog */}
      <Dialog open={showAddVaccinationDialog} onOpenChange={setShowAddVaccinationDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新增疫苗/驅蟲紀錄</DialogTitle>
            <DialogDescription>耳號：{animal.ear_tag}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="vac_date">施打日期 *</Label>
              <Input
                id="vac_date"
                type="date"
                value={newVaccination.administered_date}
                onChange={(e) => setNewVaccination({ ...newVaccination, administered_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="vaccine">疫苗</Label>
              <Input
                id="vaccine"
                value={newVaccination.vaccine}
                onChange={(e) => setNewVaccination({ ...newVaccination, vaccine: e.target.value })}
                placeholder="如：SEP、IRON"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="deworming">驅蟲劑量</Label>
              <Input
                id="deworming"
                value={newVaccination.deworming_dose}
                onChange={(e) => setNewVaccination({ ...newVaccination, deworming_dose: e.target.value })}
                placeholder="如：Ivermectin 2mL"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAddVaccinationDialog(false)}>
              取消
            </Button>
            <Button
              onClick={() => addVaccinationMutation.mutate(newVaccination)}
              disabled={addVaccinationMutation.isPending || !newVaccination.administered_date}
              className="bg-green-600 hover:bg-green-700"
            >
              {addVaccinationMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Version History Dialog */}
      {versionHistoryRecordId && (
        <VersionHistoryDialog
          open={showVersionHistoryDialog}
          onOpenChange={setShowVersionHistoryDialog}
          recordType={versionHistoryType}
          recordId={versionHistoryRecordId}
        />
      )}

      {/* Vet Recommendation Dialog */}
      {vetRecommendationRecordId && (
        <VetRecommendationDialog
          open={showVetRecommendationDialog}
          onOpenChange={setShowVetRecommendationDialog}
          recordType={vetRecommendationType}
          recordId={vetRecommendationRecordId}
          animalEarTag={animal.ear_tag}
        />
      )}

      {/* Sacrifice Form Dialog */}
      <SacrificeFormDialog
        open={showSacrificeDialog}
        onOpenChange={setShowSacrificeDialog}
        animalId={animalId}
        earTag={animal?.ear_tag || ''}
        sacrifice={sacrifice || undefined}
      />

      {/* Delete Reason Dialogs (GLP Compliance) */}
      <DeleteReasonDialog
        open={deleteObservationTarget !== null}
        onOpenChange={(open) => !open && setDeleteObservationTarget(null)}
        title="刪除觀察紀錄"
        description="此操作將標記紀錄為已刪除，資料將保留於系統中以符合 GLP 規範。"
        onConfirm={(reason) => deleteObservationMutation.mutate({ id: deleteObservationTarget!, reason })}
        isPending={deleteObservationMutation.isPending}
      />

      <DeleteReasonDialog
        open={deleteSurgeryTarget !== null}
        onOpenChange={(open) => !open && setDeleteSurgeryTarget(null)}
        title="刪除手術紀錄"
        description="此操作將標記紀錄為已刪除，資料將保留於系統中以符合 GLP 規範。"
        onConfirm={(reason) => deleteSurgeryMutation.mutate({ id: deleteSurgeryTarget!, reason })}
        isPending={deleteSurgeryMutation.isPending}
      />

      <DeleteReasonDialog
        open={deleteWeightTarget !== null}
        onOpenChange={(open) => !open && setDeleteWeightTarget(null)}
        title="刪除體重紀錄"
        description="此操作將標記紀錄為已刪除，資料將保留於系統中以符合 GLP 規範。"
        onConfirm={(reason) => deleteWeightMutation.mutate({ id: deleteWeightTarget!, reason })}
        isPending={deleteWeightMutation.isPending}
      />

      <DeleteReasonDialog
        open={deleteVaccinationTarget !== null}
        onOpenChange={(open) => !open && setDeleteVaccinationTarget(null)}
        title="刪除疫苗紀錄"
        description="此操作將標記紀錄為已刪除，資料將保留於系統中以符合 GLP 規範。"
        onConfirm={(reason) => deleteVaccinationMutation.mutate({ id: deleteVaccinationTarget!, reason })}
        isPending={deleteVaccinationMutation.isPending}
      />

      {/* Emergency Medication Dialog */}
      <EmergencyMedicationDialog
        open={showEmergencyMedicationDialog}
        onOpenChange={setShowEmergencyMedicationDialog}
        animalId={animalId}
        earTag={animal.ear_tag}
      />

      {/* Euthanasia Order Dialog */}
      <EuthanasiaOrderDialog
        open={showEuthanasiaOrderDialog}
        onOpenChange={setShowEuthanasiaOrderDialog}
        animalId={animalId}
        earTag={animal.ear_tag}
        iacucNo={animal.iacuc_no}
      />

      {/* 猝死登記 Dialog */}
      <Dialog open={showSuddenDeathDialog} onOpenChange={setShowSuddenDeathDialog}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-rose-600">
              <Zap className="h-5 w-5" />
              登記猝死 — 耳號 {animal.ear_tag}
            </DialogTitle>
            <DialogDescription>
              登記後動物狀態將自動更新為「猝死」，此操作不可復原。
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="sd-discovered-at">發現時間 *</Label>
              <Input
                id="sd-discovered-at"
                type="datetime-local"
                value={suddenDeathForm.discovered_at}
                onChange={(e) => setSuddenDeathForm(prev => ({ ...prev, discovered_at: e.target.value }))}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="sd-location">發現地點</Label>
              <Input
                id="sd-location"
                placeholder="如：A01 欄位"
                value={suddenDeathForm.location}
                onChange={(e) => setSuddenDeathForm(prev => ({ ...prev, location: e.target.value }))}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="sd-probable-cause">可能原因</Label>
              <Textarea
                id="sd-probable-cause"
                placeholder="描述可能的死因..."
                value={suddenDeathForm.probable_cause}
                onChange={(e) => setSuddenDeathForm(prev => ({ ...prev, probable_cause: e.target.value }))}
                className="min-h-[80px]"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="sd-remark">備註</Label>
              <Textarea
                id="sd-remark"
                placeholder="其他備註..."
                value={suddenDeathForm.remark}
                onChange={(e) => setSuddenDeathForm(prev => ({ ...prev, remark: e.target.value }))}
                className="min-h-[60px]"
              />
            </div>
            <div className="flex items-center space-x-2">
              <input
                id="sd-requires-pathology"
                type="checkbox"
                checked={suddenDeathForm.requires_pathology}
                onChange={(e) => setSuddenDeathForm(prev => ({ ...prev, requires_pathology: e.target.checked }))}
                className="h-4 w-4 rounded border-slate-300 text-rose-600 focus:ring-rose-500"
              />
              <Label htmlFor="sd-requires-pathology" className="text-sm font-normal cursor-pointer">
                需要病理檢查
              </Label>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowSuddenDeathDialog(false)}>
              取消
            </Button>
            <Button
              className="bg-rose-600 hover:bg-rose-700 text-white"
              disabled={!suddenDeathForm.discovered_at || createSuddenDeathMutation.isPending}
              onClick={() => {
                if (confirm(`確定要將耳號 ${animal.ear_tag} 登記為猝死？此操作不可復原。`)) {
                  createSuddenDeathMutation.mutate(suddenDeathForm)
                }
              }}
            >
              {createSuddenDeathMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              確認登記猝死
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
