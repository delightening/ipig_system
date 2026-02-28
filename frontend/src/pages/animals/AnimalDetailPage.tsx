import React, { useState, useEffect, useMemo, useCallback } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { getErrorMessage } from '@/types/error'
import api, {
  Animal,
  AnimalObservation,
  AnimalSurgery,
  AnimalWeight,
  AnimalVaccination,
  AnimalSacrifice,
  AnimalSuddenDeath,
  AnimalEvent,
  transferApi,
  allAnimalStatusNames,
  animalBreedNames,
  animalGenderNames,
  AnimalStatus,
  ProtocolListItem,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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
  AlertCircle,
  Scale,
  Syringe,
  FileText,
  Scissors,
  ClipboardList,
  Heart,
  Download,
  Stethoscope,
  AlertTriangle,
  AlertOctagon,
  Droplets,
  ArrowRightLeft,
  Zap,
  History,
} from 'lucide-react'

import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { AnimalTimelineView } from '@/components/animal/AnimalTimelineView'

import { ExportDialog } from '@/components/animal/ExportDialog'
import { EmergencyMedicationDialog } from '@/components/animal/EmergencyMedicationDialog'
import { EuthanasiaOrderDialog } from '@/components/animal/EuthanasiaOrderDialog'
import { ObservationsTab } from '@/components/animal/ObservationsTab'
import { SurgeriesTab } from '@/components/animal/SurgeriesTab'
import { WeightsTab } from '@/components/animal/WeightsTab'
import { VaccinationsTab } from '@/components/animal/VaccinationsTab'
import { SacrificeTab } from '@/components/animal/SacrificeTab'
import { AnimalInfoTab } from '@/components/animal/AnimalInfoTab'
import { PathologyTab } from '@/components/animal/PathologyTab'
import { BloodTestTab } from '@/components/animal/BloodTestTab'
import { TransferTab } from '@/components/animal/TransferTab'
import { PainAssessmentTab } from '@/components/animal/PainAssessmentTab'
import { useAuthStore } from '@/stores/auth'
import { useUIPreferences } from '@/stores/uiPreferences'
import { useTranslation } from 'react-i18next'

const statusColors: Record<AnimalStatus, string> = {
  unassigned: 'bg-gray-500',
  in_experiment: 'bg-orange-500',
  completed: 'bg-green-500',
  euthanized: 'bg-red-500',
  sudden_death: 'bg-rose-600',
  transferred: 'bg-indigo-500',
}

const getPenLocationDisplay = (animal: { status: AnimalStatus; pen_location?: string | null }) => {
  if (animal.status === 'completed' && !animal.pen_location) return '犧牲'
  return animal.pen_location || '-'
}

type TabType = 'timeline' | 'observations' | 'surgeries' | 'weights' | 'vaccinations' | 'sacrifice' | 'info' | 'pathology' | 'blood_tests' | 'pain_assessment' | 'transfer'

export function AnimalDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { dialogState, confirm } = useConfirmDialog()
  const { t } = useTranslation()
  const animalId = id!

  // Auth and UI preferences
  const { hasRole } = useAuthStore()
  const { developerMode, toggleDeveloperMode } = useUIPreferences()

  const [activeTab, setActiveTab] = useState<TabType>('timeline')

  const [showExportDialog, setShowExportDialog] = useState(false)
  const [showEmergencyMedicationDialog, setShowEmergencyMedicationDialog] = useState(false)
  const [showEuthanasiaOrderDialog, setShowEuthanasiaOrderDialog] = useState(false)
  const [showSuddenDeathDialog, setShowSuddenDeathDialog] = useState(false)
  const [showTrialSelect, setShowTrialSelect] = useState(false)

  const [suddenDeathForm, setSuddenDeathForm] = useState({
    discovered_at: new Date().toISOString().slice(0, 16),
    probable_cause: '',
    location: '',
    remark: '',
    requires_pathology: false,
  })

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
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getErrorMessage(error) || '分配失敗',
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
      toast({
        title: '錯誤',
        description: getErrorMessage(observationsError) || '載入觀察紀錄失敗',
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
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getErrorMessage(error) || '猝死登記失敗',
        variant: 'destructive',
      })
    },
  })

  const tabs = useMemo(() => [
    { id: 'timeline' as const, label: t('animalDetail.tabs.timeline', '紀錄時間軸'), icon: History },
    { id: 'observations' as const, label: t('animalDetail.tabs.observations', '觀察試驗紀錄'), icon: ClipboardList },
    { id: 'surgeries' as const, label: t('animalDetail.tabs.surgeries', '手術紀錄'), icon: Scissors },
    { id: 'weights' as const, label: t('animalDetail.tabs.weights', '體重紀錄'), icon: Scale },
    { id: 'vaccinations' as const, label: t('animalDetail.tabs.vaccinations', '疫苗/驅蟲紀錄'), icon: Syringe },
    { id: 'sacrifice' as const, label: t('animalDetail.tabs.sacrifice', '犧牲/採樣紀錄'), icon: Heart },
    { id: 'blood_tests' as const, label: t('animalDetail.tabs.bloodTests', '血液檢查'), icon: Droplets },
    { id: 'pain_assessment' as const, label: t('animalDetail.tabs.painAssessment', '疼痛評估'), icon: Stethoscope },
    { id: 'info' as const, label: t('animalDetail.tabs.info', '動物資料'), icon: FileText },
    { id: 'pathology' as const, label: t('animalDetail.tabs.pathology', '病理組織報告'), icon: FileText },
    ...((animal?.status === 'completed' || animal?.status === 'transferred')
      ? [{ id: 'transfer' as const, label: t('animalDetail.tabs.transfer', '轉讓管理'), icon: ArrowRightLeft }]
      : []),
  ], [t, animal?.status])

  const handleTimelineAction = useCallback((type: string) => {
    setActiveTab(type === 'observation' ? 'observations' : 'surgeries')
  }, [])

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
        <p className="text-slate-500">{t('animalDetail.notFound', '找不到此動物')}</p>
        <Button variant="outline" className="mt-4" onClick={() => navigate('/animals')}>
          {t('animalDetail.backToList', '返回列表')}
        </Button>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Back Button & Export */}
      <div className="flex items-center justify-between">
        <Link to="/animals" className="inline-flex items-center text-slate-600 hover:text-slate-900">
          <ArrowLeft className="h-4 w-4 mr-2" />
          {t('animalDetail.backToAnimalList', '回到動物列表')}
        </Link>
        <Button variant="outline" onClick={() => setShowExportDialog(true)}>
          <Download className="h-4 w-4 mr-2" />
          {t('animalDetail.exportRecord', '匯出病歷')}
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
        <div className="flex flex-wrap gap-1">
          {tabs.map((tab) => {
            const Icon = tab.icon
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors ${activeTab === tab.id
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
            onView={handleTimelineAction}
            onEdit={handleTimelineAction}
            onCopy={handleTimelineAction}
            onHistory={handleTimelineAction}
            onVet={handleTimelineAction}
            onDelete={handleTimelineAction}
          />
        )}

        {/* 觀察試驗紀錄 Tab */}
        {activeTab === 'observations' && (
          <ObservationsTab animalId={animalId} earTag={animal.ear_tag} afterParam={afterParam} observations={observations} />
        )}

        {/* 手術紀錄 Tab */}
        {activeTab === 'surgeries' && (
          <SurgeriesTab animalId={animalId} earTag={animal.ear_tag} afterParam={afterParam} surgeries={surgeries} />
        )}

        {/* 體重紀錄 Tab */}
        {activeTab === 'weights' && (
          <WeightsTab
            animalId={animalId}
            earTag={animal.ear_tag}
            afterParam={afterParam}
            weights={weights}
            hasAdminRole={hasRole('admin')}
            developerMode={developerMode}
            toggleDeveloperMode={toggleDeveloperMode}
          />
        )}

        {/* 疫苗/驅蟲紀錄 Tab */}
        {activeTab === 'vaccinations' && (
          <VaccinationsTab animalId={animalId} earTag={animal.ear_tag} afterParam={afterParam} vaccinations={vaccinations} />
        )}

        {/* 犧牲/採樣紀錄 Tab */}
        {activeTab === 'sacrifice' && (
          <SacrificeTab animalId={animalId} earTag={animal.ear_tag} sacrifice={sacrifice} />
        )}

        {/* 動物資料 Tab */}
        {activeTab === 'info' && (
          <AnimalInfoTab animal={animal} />
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
          <PathologyTab animalId={animalId} earTag={animal.ear_tag} />
        )}
      </div>


      {/* Export Dialog */}
      <ExportDialog
        open={showExportDialog}
        onOpenChange={setShowExportDialog}
        type="single_animal"
        animalId={animalId}
        earTag={animal.ear_tag}
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
              onClick={async () => {
                const ok = await confirm({ title: '登記猝死', description: `確定要將耳號 ${animal.ear_tag} 登記為猝死？此操作不可復原。`, variant: 'destructive', confirmLabel: '確認登記' })
                if (ok) {
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
      <ConfirmDialog state={dialogState} />
    </div>
  )
}
