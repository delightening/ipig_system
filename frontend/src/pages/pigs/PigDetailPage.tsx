import { useState } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  Pig,
  PigObservation,
  PigSurgery,
  PigWeight,
  PigVaccination,
  PigSacrifice,
  PigPathologyReport,
  pigStatusNames,
  pigBreedNames,
  pigGenderNames,
  recordTypeNames,
  PigStatus,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { FileUpload, FileInfo } from '@/components/ui/file-upload'
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
} from 'lucide-react'

// Import form dialog components
import { ObservationFormDialog } from '@/components/pig/ObservationFormDialog'
import { SurgeryFormDialog } from '@/components/pig/SurgeryFormDialog'
import { ExportDialog } from '@/components/pig/ExportDialog'
import { VersionHistoryDialog } from '@/components/pig/VersionHistoryDialog'
import { VetRecommendationDialog } from '@/components/pig/VetRecommendationDialog'

const statusColors: Record<PigStatus, string> = {
  unassigned: 'bg-gray-500',
  assigned: 'bg-blue-500',
  in_experiment: 'bg-orange-500',
  completed: 'bg-green-500',
}

type TabType = 'observations' | 'surgeries' | 'weights' | 'vaccinations' | 'sacrifice' | 'info' | 'pathology'

export function PigDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const pigId = parseInt(id!)

  const [activeTab, setActiveTab] = useState<TabType>('observations')
  
  // Dialog states
  const [showAddObservationDialog, setShowAddObservationDialog] = useState(false)
  const [showAddSurgeryDialog, setShowAddSurgeryDialog] = useState(false)
  const [showAddWeightDialog, setShowAddWeightDialog] = useState(false)
  const [showAddVaccinationDialog, setShowAddVaccinationDialog] = useState(false)
  const [showExportDialog, setShowExportDialog] = useState(false)
  const [showPathologyUploadDialog, setShowPathologyUploadDialog] = useState(false)
  
  // Edit states
  const [editingObservation, setEditingObservation] = useState<PigObservation | null>(null)
  const [editingSurgery, setEditingSurgery] = useState<PigSurgery | null>(null)
  
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

  // Queries
  const { data: pig, isLoading: pigLoading } = useQuery({
    queryKey: ['pig', pigId],
    queryFn: async () => {
      const res = await api.get<Pig>(`/pigs/${pigId}`)
      return res.data
    },
    staleTime: 0, // Always consider data stale for real-time updates
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: observations } = useQuery({
    queryKey: ['pig-observations', pigId],
    queryFn: async () => {
      const res = await api.get<PigObservation[]>(`/pigs/${pigId}/observations`)
      return res.data
    },
    enabled: activeTab === 'observations',
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: surgeries } = useQuery({
    queryKey: ['pig-surgeries', pigId],
    queryFn: async () => {
      const res = await api.get<PigSurgery[]>(`/pigs/${pigId}/surgeries`)
      return res.data
    },
    enabled: activeTab === 'surgeries',
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: weights } = useQuery({
    queryKey: ['pig-weights', pigId],
    queryFn: async () => {
      const res = await api.get<PigWeight[]>(`/pigs/${pigId}/weights`)
      return res.data
    },
    enabled: activeTab === 'weights',
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: vaccinations } = useQuery({
    queryKey: ['pig-vaccinations', pigId],
    queryFn: async () => {
      const res = await api.get<PigVaccination[]>(`/pigs/${pigId}/vaccinations`)
      return res.data
    },
    enabled: activeTab === 'vaccinations',
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: sacrifice } = useQuery({
    queryKey: ['pig-sacrifice', pigId],
    queryFn: async () => {
      const res = await api.get<PigSacrifice>(`/pigs/${pigId}/sacrifice`)
      return res.data
    },
    enabled: activeTab === 'sacrifice',
  })

  const { data: pathology } = useQuery({
    queryKey: ['pig-pathology', pigId],
    queryFn: async () => {
      const res = await api.get<PigPathologyReport>(`/pigs/${pigId}/pathology`)
      return res.data
    },
    enabled: activeTab === 'pathology',
  })

  // Mutations
  const addWeightMutation = useMutation({
    mutationFn: async (data: typeof newWeight) => {
      return api.post(`/pigs/${pigId}/weights`, {
        measure_date: data.measure_date,
        weight: parseFloat(data.weight),
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-weights', pigId] })
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
      return api.post(`/pigs/${pigId}/vaccinations`, data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-vaccinations', pigId] })
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
    mutationFn: async (weightId: number) => {
      return api.delete(`/weights/${weightId}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-weights', pigId] })
      toast({ title: '成功', description: '體重紀錄已刪除' })
    },
  })

  const deleteVaccinationMutation = useMutation({
    mutationFn: async (vaccinationId: number) => {
      return api.delete(`/vaccinations/${vaccinationId}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-vaccinations', pigId] })
      toast({ title: '成功', description: '疫苗紀錄已刪除' })
    },
  })

  const deleteObservationMutation = useMutation({
    mutationFn: async (observationId: number) => {
      return api.delete(`/observations/${observationId}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-observations', pigId] })
      toast({ title: '成功', description: '觀察紀錄已刪除' })
    },
  })

  const deleteSurgeryMutation = useMutation({
    mutationFn: async (surgeryId: number) => {
      return api.delete(`/surgeries/${surgeryId}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-surgeries', pigId] })
      toast({ title: '成功', description: '手術紀錄已刪除' })
    },
  })

  const copyObservationMutation = useMutation({
    mutationFn: async (sourceId: number) => {
      return api.post(`/pigs/${pigId}/observations/copy`, { source_id: sourceId })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-observations', pigId] })
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
      return api.post(`/pigs/${pigId}/surgeries/copy`, { source_id: sourceId })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-surgeries', pigId] })
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
      return api.post(`/pigs/${pigId}/pathology/upload`, {
        files: files.map((f) => ({
          file_name: f.file_name,
          file_size: f.file_size,
        })),
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-pathology', pigId] })
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

  if (pigLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
      </div>
    )
  }

  if (!pig) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px]">
        <AlertCircle className="h-12 w-12 text-slate-400 mb-4" />
        <p className="text-slate-500">找不到此豬隻</p>
        <Button variant="outline" className="mt-4" onClick={() => navigate('/pigs')}>
          返回列表
        </Button>
      </div>
    )
  }

  const tabs = [
    { id: 'observations' as const, label: '觀察試驗紀錄', icon: ClipboardList },
    { id: 'surgeries' as const, label: '手術紀錄', icon: Scissors },
    { id: 'weights' as const, label: '體重紀錄', icon: Scale },
    { id: 'vaccinations' as const, label: '疫苗/驅蟲紀錄', icon: Syringe },
    { id: 'sacrifice' as const, label: '犧牲/採樣紀錄', icon: Heart },
    { id: 'info' as const, label: '豬隻資料', icon: FileText },
    { id: 'pathology' as const, label: '病理組織報告', icon: FileText },
  ]

  return (
    <div className="space-y-6">
      {/* Back Button & Export */}
      <div className="flex items-center justify-between">
        <Link to="/pigs" className="inline-flex items-center text-slate-600 hover:text-slate-900">
          <ArrowLeft className="h-4 w-4 mr-2" />
          回到所有豬隻
        </Link>
        <Button variant="outline" onClick={() => setShowExportDialog(true)}>
          <Download className="h-4 w-4 mr-2" />
          匯出病歷
        </Button>
      </div>

      {/* Pig Header Card */}
      <Card className="bg-gradient-to-r from-slate-50 to-blue-50 border-slate-200">
        <CardContent className="pt-6">
          <div className="grid grid-cols-3 gap-6">
            <div className="space-y-3">
              <div>
                <span className="text-sm text-slate-500">耳號</span>
                <p className="text-2xl font-bold text-orange-600">{pig.ear_tag}</p>
              </div>
              <div>
                <span className="text-sm text-slate-500">欄號</span>
                <p className="font-medium">{pig.pen_location || '-'}</p>
              </div>
              <div>
                <span className="text-sm text-slate-500">品種</span>
                <p className="font-medium">{pigBreedNames[pig.breed]}</p>
              </div>
            </div>
            <div className="space-y-3">
              <div>
                <span className="text-sm text-slate-500">出生日期</span>
                <p className="font-medium">
                  {pig.birth_date ? new Date(pig.birth_date).toLocaleDateString('zh-TW') : '-'}
                </p>
              </div>
              <div>
                <span className="text-sm text-slate-500">IACUC NO.</span>
                <p className="font-medium">{pig.iacuc_no || '未分配'}</p>
              </div>
              <div>
                <span className="text-sm text-slate-500">最近體重</span>
                <p className="font-medium">
                  {weights && weights.length > 0 
                    ? `${weights[0].weight} kg`
                    : pig.entry_weight 
                      ? `${pig.entry_weight} kg (進場)`
                      : '-'
                  }
                </p>
              </div>
            </div>
            <div className="space-y-3">
              <div>
                <span className="text-sm text-slate-500">系統號</span>
                <p className="font-medium">{pig.id}</p>
              </div>
              <div>
                <span className="text-sm text-slate-500">豬隻狀態</span>
                <Badge className={`${statusColors[pig.status]} text-white mt-1`}>
                  {pigStatusNames[pig.status]}
                </Badge>
              </div>
              <div>
                <span className="text-sm text-slate-500">性別</span>
                <p className="font-medium">{pigGenderNames[pig.gender]}</p>
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
                className={`flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors whitespace-nowrap ${
                  activeTab === tab.id
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
      <div>
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
                    {observations.map((obs) => (
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
                            <Badge variant="outline">{recordTypeNames[obs.record_type]}</Badge>
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
                                onClick={() => {
                                  if (confirm('確定要刪除此紀錄？')) {
                                    deleteObservationMutation.mutate(obs.id)
                                  }
                                }}
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
                                      {obs.treatments.map((t, i) => (
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
                                onClick={() => {
                                  if (confirm('確定要刪除此紀錄？')) {
                                    deleteSurgeryMutation.mutate(surgery.id)
                                  }
                                }}
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
                <CardDescription>記錄豬隻體重變化歷程</CardDescription>
              </div>
              <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowAddWeightDialog(true)}>
                <Plus className="h-4 w-4 mr-2" />
                新增紀錄
              </Button>
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
                      <TableHead>系統號</TableHead>
                      <TableHead>測量日期</TableHead>
                      <TableHead>體重 (kg)</TableHead>
                      <TableHead>記錄者</TableHead>
                      <TableHead>建立時間</TableHead>
                      <TableHead className="text-right">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {weights.map((weight) => (
                      <TableRow key={weight.id}>
                        <TableCell>{weight.id}</TableCell>
                        <TableCell>{new Date(weight.measure_date).toLocaleDateString('zh-TW')}</TableCell>
                        <TableCell className="font-medium">{weight.weight}</TableCell>
                        <TableCell>{weight.created_by_name || '-'}</TableCell>
                        <TableCell>{new Date(weight.created_at).toLocaleString('zh-TW')}</TableCell>
                        <TableCell className="text-right">
                          <div className="flex items-center justify-end gap-1">
                            <Button variant="ghost" size="icon">
                              <Edit2 className="h-4 w-4" />
                            </Button>
                            <Button 
                              variant="ghost" 
                              size="icon"
                              onClick={() => {
                                if (confirm('確定要刪除此體重紀錄？')) {
                                  deleteWeightMutation.mutate(weight.id)
                                }
                              }}
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
                              onClick={() => {
                                if (confirm('確定要刪除此紀錄？')) {
                                  deleteVaccinationMutation.mutate(vac.id)
                                }
                              }}
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
                  <Button className="mt-4" variant="outline">
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
                    <Button variant="outline">
                      <Edit2 className="h-4 w-4 mr-2" />
                      編輯
                    </Button>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        )}

        {/* 豬隻資料 Tab */}
        {activeTab === 'info' && (
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <div>
                <CardTitle>豬隻資料</CardTitle>
                <CardDescription>豬隻基本資料</CardDescription>
              </div>
              <Button variant="outline" asChild>
                <Link to={`/pigs/${pig.id}/edit`}>
                  <Edit2 className="h-4 w-4 mr-2" />
                  編輯
                </Link>
              </Button>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 gap-x-8 gap-y-4">
                <div>
                  <Label className="text-slate-500">耳號</Label>
                  <p className="font-medium">{pig.ear_tag}</p>
                </div>
                <div>
                  <Label className="text-slate-500">豬隻狀態</Label>
                  <p className="font-medium">{pigStatusNames[pig.status]}</p>
                </div>
                <div>
                  <Label className="text-slate-500">進場日期</Label>
                  <p className="font-medium">{new Date(pig.entry_date).toLocaleDateString('zh-TW')}</p>
                </div>
                <div>
                  <Label className="text-slate-500">品種</Label>
                  <p className="font-medium">{pigBreedNames[pig.breed]}</p>
                </div>
                <div>
                  <Label className="text-slate-500">來源</Label>
                  <p className="font-medium">{pig.source_name || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">進場體重 (kg)</Label>
                  <p className="font-medium">{pig.entry_weight || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">性別</Label>
                  <p className="font-medium">{pigGenderNames[pig.gender]}</p>
                </div>
                <div>
                  <Label className="text-slate-500">出生日期</Label>
                  <p className="font-medium">
                    {pig.birth_date ? new Date(pig.birth_date).toLocaleDateString('zh-TW') : '-'}
                  </p>
                </div>
                <div>
                  <Label className="text-slate-500">實驗前代號</Label>
                  <p className="font-medium">{pig.pre_experiment_code || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">IACUC NO.</Label>
                  <p className="font-medium">{pig.iacuc_no || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">實驗日期</Label>
                  <p className="font-medium">
                    {pig.experiment_date ? new Date(pig.experiment_date).toLocaleDateString('zh-TW') : '-'}
                  </p>
                </div>
                <div>
                  <Label className="text-slate-500">欄位</Label>
                  <p className="font-medium">{pig.pen_location || '-'}</p>
                </div>
                <div className="col-span-2">
                  <Label className="text-slate-500">備註</Label>
                  <p className="font-medium">{pig.remark || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">系統號</Label>
                  <p className="font-medium">{pig.id}</p>
                </div>
                <div>
                  <Label className="text-slate-500">建立時間</Label>
                  <p className="font-medium">{new Date(pig.created_at).toLocaleString('zh-TW')}</p>
                </div>
              </div>
            </CardContent>
          </Card>
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
        pigId={pigId}
        earTag={pig.ear_tag}
        observation={editingObservation || undefined}
      />

      {/* Surgery Form Dialog */}
      <SurgeryFormDialog
        open={showAddSurgeryDialog}
        onOpenChange={(open) => {
          setShowAddSurgeryDialog(open)
          if (!open) setEditingSurgery(null)
        }}
        pigId={pigId}
        earTag={pig.ear_tag}
        surgery={editingSurgery || undefined}
      />

      {/* Export Dialog */}
      <ExportDialog
        open={showExportDialog}
        onOpenChange={setShowExportDialog}
        type="single_pig"
        pigId={pigId}
        earTag={pig.ear_tag}
      />

      {/* Pathology Upload Dialog */}
      <Dialog open={showPathologyUploadDialog} onOpenChange={setShowPathologyUploadDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>上傳病理組織報告</DialogTitle>
            <DialogDescription>耳號：{pig.ear_tag}</DialogDescription>
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
            <DialogDescription>耳號：{pig.ear_tag}</DialogDescription>
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
            <DialogDescription>耳號：{pig.ear_tag}</DialogDescription>
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
          pigEarTag={pig.ear_tag}
        />
      )}
    </div>
  )
}
