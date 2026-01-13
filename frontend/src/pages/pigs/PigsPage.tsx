import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Link, useSearchParams } from 'react-router-dom'
import api, {
  Pig,
  PigListItem,
  PigStatus,
  pigStatusNames,
  pigBreedNames,
  pigGenderNames,
  PigSource,
  PigBreed,
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
  Play,
  AlertCircle,
  CheckCircle2,
  Stethoscope,
  FileSpreadsheet,
  LayoutGrid,
  MapPin,
} from 'lucide-react'

// Import Export Dialog
import { ExportDialog } from '@/components/pig/ExportDialog'
import { ImportDialog } from '@/components/pig/ImportDialog'
import { QuickEditPigDialog } from '@/components/pig/QuickEditPigDialog'
import { PigPenReport } from '../../components/pig/PigPenReport'

const statusColors: Record<PigStatus, string> = {
  unassigned: 'bg-gray-100 text-gray-800',
  assigned: 'bg-blue-100 text-blue-800',
  in_experiment: 'bg-orange-100 text-orange-800',
  completed: 'bg-green-100 text-green-800',
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

export function PigsPage() {
  const queryClient = useQueryClient()
  const [searchParams, setSearchParams] = useSearchParams()

  // Building tab state for grouped view
  const [groupedBuildingTab, setGroupedBuildingTab] = useState<'A' | 'B'>('A')

  // Filter state
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>(searchParams.get('status') || 'pen')
  const [breedFilter, setBreedFilter] = useState<string>('all')

  // Dialog state
  const [showAddDialog, setShowAddDialog] = useState(false)
  const [showBatchAssignDialog, setShowBatchAssignDialog] = useState(false)
  const [showBatchExportDialog, setShowBatchExportDialog] = useState(false)
  const [showImportBasicDialog, setShowImportBasicDialog] = useState(false)
  const [showImportWeightDialog, setShowImportWeightDialog] = useState(false)
  const [selectedPigs, setSelectedPigs] = useState<number[]>([])
  const [assignIacucNo, setAssignIacucNo] = useState('')

  // Quick edit dialog state
  const [quickEditPigId, setQuickEditPigId] = useState<number | null>(null)
  const [showPrintReport, setShowPrintReport] = useState(false)

  // Quick move state (空欄位快速移動)
  const [editingPenLocation, setEditingPenLocation] = useState<string | null>(null)
  const [editingEarTag, setEditingEarTag] = useState<string>('')

  // Form state for new pig
  const [penBuilding, setPenBuilding] = useState('')
  const [penZone, setPenZone] = useState('')
  const [penNumber, setPenNumber] = useState('')
  const [newPig, setNewPig] = useState({
    ear_tag: '',
    breed: 'minipig' as PigBreed,
    gender: 'male' as const,
    source_id: '',
    entry_date: new Date().toISOString().split('T')[0],
    entry_weight: '',
    birth_date: '',
    pre_experiment_code: '',
    remark: '',
    breed_other: '',
  })

  // Query for all pigs (for counting)
  const { data: allPigsData } = useQuery({
    queryKey: ['pigs-count'],
    queryFn: async () => {
      const res = await api.get<PigListItem[]>(`/pigs`)
      return res.data
    },
    staleTime: 0,
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  // Queries
  const { data: pigsData, isLoading } = useQuery({
    queryKey: ['pigs', statusFilter, breedFilter, search],
    queryFn: async () => {
      const params = new URLSearchParams()
      if (statusFilter && statusFilter !== 'all') params.append('status', statusFilter)
      if (breedFilter && breedFilter !== 'all') params.append('breed', breedFilter)
      if (search) params.append('search', search)
      const res = await api.get<PigListItem[]>(`/pigs?${params}`)
      return res.data
    },
    staleTime: 0, // Always consider data stale for real-time updates
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  const { data: sourcesData } = useQuery({
    queryKey: ['pig-sources'],
    queryFn: async () => {
      const res = await api.get<PigSource[]>('/pig-sources')
      return res.data
    },
  })

  // Query grouped by pen
  const { data: groupedData, isLoading: groupedLoading } = useQuery({
    queryKey: ['pigs-by-pen'],
    queryFn: async () => {
      const res = await api.get<{ pen_location: string; pigs: PigListItem[]; pig_count: number }[]>('/pigs/by-pen')
      return res.data
    },
    enabled: statusFilter === 'pen',
    staleTime: 0, // Always consider data stale for real-time updates
    refetchOnWindowFocus: true,
    refetchOnMount: true,
  })

  // Mutations
  const createPigMutation = useMutation({
    mutationFn: async (data: typeof newPig) => {
      const penLocation = penZone && penNumber ? `${penZone}${penNumber}` : undefined

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
        pen_location: penLocation, // string or undefined
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
      return api.post('/pigs', payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pigs'] })
      toast({ title: '成功', description: '動物已新增' })
      setShowAddDialog(false)
      resetNewPigForm()
    },
    onError: (error: any) => {
      console.error('Create pig error:', error)
      console.error('Error response:', error?.response?.data)
      console.error('Error status:', error?.response?.status)
      console.error('Request payload:', error?.config?.data)

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
      return api.post('/pigs/batch/assign', {
        pig_ids: selectedPigs,
        iacuc_no: assignIacucNo,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pigs'] })
      toast({ title: '成功', description: '豬隻已新增' })
      setShowBatchAssignDialog(false)
      setSelectedPigs([])
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
      return api.post('/pigs/batch/start-experiment', {
        pig_ids: selectedPigs,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pigs'] })
      toast({ title: '成功', description: '豬隻已新增' })
      setSelectedPigs([])
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

  // Quick move mutation (空欄位快速移動)
  const quickMoveMutation = useMutation({
    mutationFn: async ({ earTag, targetPenLocation }: { earTag: string; targetPenLocation: string }) => {
      // 先根據耳號查詢豬隻
      const searchRes = await api.get<PigListItem[]>(`/pigs?keyword=${encodeURIComponent(earTag)}`)
      const matchingPigs = searchRes.data.filter(p => p.ear_tag === earTag)
      
      if (matchingPigs.length === 0) {
        throw new Error(`找不到耳號為 "${earTag}" 的動物`)
      }
      
      if (matchingPigs.length > 1) {
        throw new Error(`找到多隻耳號為 "${earTag}" 的動物，請使用編輯功能手動移動`)
      }
      
      const pig = matchingPigs[0]
      
      // 檢查豬隻是否已經在目標欄位
      if (pig.pen_location === targetPenLocation) {
        throw new Error(`動物 ${earTag} 已經在 ${targetPenLocation} 欄位`)
      }
      
      // 更新豬隻的欄位
      return api.put<Pig>(`/pigs/${pig.id}`, {
        pen_location: targetPenLocation,
      })
    },
    onSuccess: (data, variables) => {
      queryClient.invalidateQueries({ queryKey: ['pigs'] })
      queryClient.invalidateQueries({ queryKey: ['pigs-by-pen'] })
      queryClient.invalidateQueries({ queryKey: ['pigs-count'] })
      toast({ 
        title: '成功', 
        description: `動物 ${variables.earTag} 已移動到 ${variables.targetPenLocation}` 
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

  const resetNewPigForm = () => {
    setPenBuilding('')
    setPenZone('')
    setPenNumber('')
    setNewPig({
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

  const togglePigSelection = (id: number) => {
    setSelectedPigs(prev =>
      prev.includes(id) ? prev.filter(p => p !== id) : [...prev, id]
    )
  }

  const toggleAllPigs = () => {
    const currentPigs = pigsData || []
    if (currentPigs.length === 0) return
    if (selectedPigs.length === currentPigs.length) {
      setSelectedPigs([])
    } else {
      setSelectedPigs(currentPigs.map(p => p.id))
    }
  }

  const pigs = pigsData || []
  const allPigs = allPigsData || []

  // 計算狀態計數（基於所有豬隻，而非過濾後的結果）
  const statusCounts = allPigs.reduce((acc, pig) => {
    acc[pig.status] = (acc[pig.status] || 0) + 1
    return acc
  }, {} as Record<string, number>)

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">動物列表</h1>
          <p className="text-slate-500">管理系統中的所有實驗動物</p>
        </div>
        <div className="grid grid-cols-3 gap-2">
          <Button variant="outline" className="w-full gap-2 text-orange-600 border-orange-200 hover:bg-orange-50" onClick={() => setShowPrintReport(true)}>
            <Download className="h-4 w-4" />
            產生欄位狀態表
          </Button>
          <Button variant="outline" className="w-full gap-2" onClick={() => setShowImportWeightDialog(true)}>
            <Upload className="h-4 w-4" />
            匯入體重
          </Button>
          <Button variant="outline" className="w-full gap-2" onClick={() => setShowImportBasicDialog(true)}>
            <Upload className="h-4 w-4" />
            匯入基本資料
          </Button>
          <Button variant="outline" className="w-full gap-2" onClick={() => setShowBatchExportDialog(true)}>
            <FileSpreadsheet className="h-4 w-4" />
            批次匯出病歷
          </Button>
          <Button onClick={() => setShowAddDialog(true)} className="col-span-2 w-full gap-2 bg-purple-600 hover:bg-purple-700">
            <Plus className="h-4 w-4" />
            新增動物
          </Button>
        </div>
      </div>

      {/* Status Tabs */}
      <div className="flex gap-2 border-b">
        {[
          { value: 'pen', label: '欄位', count: allPigs.length, icon: <LayoutGrid className="h-4 w-4" /> },
          { value: 'unassigned', label: '未分配', count: statusCounts['unassigned'] || 0 },
          { value: 'assigned', label: '已分配', count: statusCounts['assigned'] || 0 },
          { value: 'in_experiment', label: '實驗中', count: statusCounts['in_experiment'] || 0 },
          { value: 'completed', label: '實驗完成', count: statusCounts['completed'] || 0 },
          { value: 'all', label: '所有', count: allPigs.length },
        ].map(tab => (
          <button
            key={tab.value}
            onClick={() => {
              setStatusFilter(tab.value)
              setSearchParams(tab.value === 'pen' ? {} : { status: tab.value })
            }}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors flex items-center gap-1.5 ${statusFilter === tab.value
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
          <div className="flex items-center justify-between gap-4">
            <div className="flex items-center gap-4 flex-1">
              <div className="relative flex-1 max-w-sm">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
                <Input
                  placeholder="搜尋耳號、欄位、IACUC NO..."
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
                  <SelectItem value="all">全部品種</SelectItem>
                  <SelectItem value="minipig">迷你豬</SelectItem>
                  <SelectItem value="white">白豬</SelectItem>
                  <SelectItem value="other">其他</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* Batch Actions */}
            {selectedPigs.length > 0 && (
              <div className="flex items-center gap-2">
                <span className="text-sm text-slate-500">
                  已選擇 {selectedPigs.length} 隻
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
                {statusFilter === 'assigned' && (
                  <Button
                    size="sm"
                    className="bg-orange-500 hover:bg-orange-600"
                    onClick={() => batchStartExperimentMutation.mutate()}
                    disabled={batchStartExperimentMutation.isPending}
                  >
                    {batchStartExperimentMutation.isPending && (
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    )}
                    <Play className="h-4 w-4 mr-1" />
                    進入實驗
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
            ) : pigs.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-slate-500">
                <AlertCircle className="h-12 w-12 mb-4" />
                <p>沒有符合條件的豬隻</p>
              </div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-12">
                      <input
                        type="checkbox"
                        checked={selectedPigs.length === pigs.length && pigs.length > 0}
                        onChange={toggleAllPigs}
                        className="rounded border-slate-300"
                      />
                    </TableHead>
                    <TableHead>系統號</TableHead>
                    <TableHead>耳號</TableHead>
                    <TableHead>欄位</TableHead>
                    <TableHead>IACUC NO.</TableHead>
                    <TableHead>狀態</TableHead>
                    <TableHead>品種</TableHead>
                    <TableHead>性別</TableHead>
                    <TableHead>用藥中</TableHead>
                    <TableHead>獸醫建議</TableHead>
                    <TableHead>進場日期</TableHead>
                    <TableHead className="text-right">動作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {pigs.map((pig) => (
                    <TableRow
                      key={pig.id}
                      className={pig.has_abnormal_record ? 'bg-yellow-50' : ''}
                    >
                      <TableCell>
                        <input
                          type="checkbox"
                          checked={selectedPigs.includes(pig.id)}
                          onChange={() => togglePigSelection(pig.id)}
                          className="rounded border-slate-300"
                        />
                      </TableCell>
                      <TableCell className="font-medium">{pig.id}</TableCell>
                      <TableCell>
                        <Link
                          to={`/pigs/${pig.id}`}
                          className="text-orange-600 hover:text-orange-700 font-medium"
                        >
                          {pig.ear_tag}
                        </Link>
                      </TableCell>
                      <TableCell>{pig.pen_location || '-'}</TableCell>
                      <TableCell>
                        {pig.iacuc_no || (
                          <span className="text-slate-400">未分配</span>
                        )}
                      </TableCell>
                      <TableCell>
                        <Badge className={statusColors[pig.status]}>
                          {pigStatusNames[pig.status]}
                        </Badge>
                      </TableCell>
                      <TableCell>{pig.breed === 'other' ? (pig.breed_other || '其他') : pigBreedNames[pig.breed]}</TableCell>
                      <TableCell>{pigGenderNames[pig.gender]}</TableCell>
                      <TableCell>
                        {pig.is_on_medication ? (
                          <Badge variant="destructive" className="text-xs">是</Badge>
                        ) : (
                          <span className="text-slate-400">否</span>
                        )}
                      </TableCell>
                      <TableCell>
                        {pig.vet_recommendation_date ? (
                          <span className="text-sm text-slate-600">
                            {new Date(pig.vet_recommendation_date).toLocaleDateString('zh-TW')}
                          </span>
                        ) : (
                          <span className="text-slate-400">-</span>
                        )}
                      </TableCell>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <span>{new Date(pig.entry_date).toLocaleDateString('zh-TW')}</span>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-6 w-6"
                            onClick={() => setQuickEditPigId(pig.id)}
                            title="快速編輯"
                          >
                            <Edit2 className="h-3 w-3" />
                          </Button>
                        </div>
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex items-center justify-end gap-1">
                          <Button variant="ghost" size="icon" asChild>
                            <Link to={`/pigs/${pig.id}`}>
                              <Eye className="h-4 w-4" />
                            </Link>
                          </Button>
                          <Button variant="ghost" size="icon" asChild>
                            <Link to={`/pigs/${pig.id}/edit`}>
                              <Edit2 className="h-4 w-4" />
                            </Link>
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

      {/* Grouped View */}
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

              // Create a map of pen_location -> pigs for quick lookup
              const pigsByPenLocation = new Map<string, PigListItem[]>()
              groupedData?.forEach(group => {
                if (group.pen_location) {
                  pigsByPenLocation.set(group.pen_location, group.pigs)
                }
              })

              // Helper function to render a single pen cell
              const renderPenCell = (penLocation: string | null, colors: { bg: string; border: string; header: string; text: string }, isLeftColumn: boolean = true) => {
                if (!penLocation) {
                  return <div className="px-3 py-2 text-slate-300"></div>
                }

                const pigs = pigsByPenLocation.get(penLocation) || []
                const cellColors = penZoneColors[penLocation.charAt(0)] || colors
                const isEditing = editingPenLocation === penLocation

                if (pigs.length === 0) {
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
                            // 使用 setTimeout 確保 onKeyDown 有機會先執行
                            setTimeout(() => {
                              // 檢查是否還在編輯狀態（可能已經被 onKeyDown 處理了）
                              if (editingPenLocation === penLocation && editingEarTag.trim()) {
                                handleSubmit()
                              } else if (editingPenLocation === penLocation && !editingEarTag.trim()) {
                                // 如果沒有輸入內容，關閉編輯狀態
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

                return pigs.map((pig, pigIdx) => (
                  <div key={pig.id} className={`grid grid-cols-5 gap-1 px-3 py-2 items-center text-sm ${pigIdx > 0 ? 'border-t border-dashed border-slate-200' : ''}`}>
                    <div className={`font-semibold ${cellColors.text}`}>{pigIdx === 0 ? penLocation : ''}</div>
                    <div className={`font-medium ${cellColors.text} truncate`} title={pig.ear_tag}>{pig.ear_tag}</div>
                    <div className="text-xs text-slate-500 truncate" title={pig.vet_last_viewed_at ? new Date(pig.vet_last_viewed_at).toLocaleString('zh-TW') : '-'}>
                      {pig.vet_last_viewed_at ? new Date(pig.vet_last_viewed_at).toLocaleDateString('zh-TW') : '-'}
                    </div>
                    <div className={`text-xs truncate ${pig.has_abnormal_record ? 'text-red-600 font-medium' : 'text-slate-400'}`}>
                      {pig.has_abnormal_record ? '有異常' : '-'}
                    </div>
                    <div className="flex items-center justify-center gap-1">
                      <Button variant="ghost" size="icon" className="h-6 w-6" asChild title="檢視">
                        <Link to={`/pigs/${pig.id}`}>
                          <Eye className="h-3 w-3" />
                        </Link>
                      </Button>
                      <Button variant="ghost" size="icon" className="h-6 w-6" asChild title="編輯">
                        <Link to={`/pigs/${pig.id}/edit`}>
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

                // Count total pigs in this zone
                let totalPigs = 0
                penNumbers.forEach(num => {
                  const penLocation = `${zone}${num}`
                  const pigs = pigsByPenLocation.get(penLocation) || []
                  totalPigs += pigs.length
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
                          共 {totalPigs} 隻
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

                // Count total pigs
                let eTotalPigs = 0, fTotalPigs = 0, gTotalPigs = 0
                ePenNumbers.forEach(num => {
                  const pigs = pigsByPenLocation.get(`E${num}`) || []
                  eTotalPigs += pigs.length
                })
                fPenNumbers.forEach(num => {
                  const pigs = pigsByPenLocation.get(`F${num}`) || []
                  fTotalPigs += pigs.length
                })
                gPenNumbers.forEach(num => {
                  const pigs = pigsByPenLocation.get(`G${num}`) || []
                  gTotalPigs += pigs.length
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
                            {eTotalPigs} 隻
                          </Badge>
                        </div>
                        <span className="text-slate-300">|</span>
                        <div className="flex items-center gap-2">
                          <span className={`w-8 h-8 rounded-lg ${fColors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>
                            F
                          </span>
                          <span className={fColors.text}>F 區</span>
                          <Badge variant="outline" className={`${fColors.text} ${fColors.border}`}>
                            {fTotalPigs} 隻
                          </Badge>
                        </div>
                        <span className="text-slate-300">|</span>
                        <div className="flex items-center gap-2">
                          <span className={`w-8 h-8 rounded-lg ${gColors.header} text-white flex items-center justify-center font-bold text-lg shadow-md`}>
                            G
                          </span>
                          <span className={gColors.text}>G 區</span>
                          <Badge variant="outline" className={`${gColors.text} ${gColors.border}`}>
                            {gTotalPigs} 隻
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

      {/* Add Pig Dialog */}
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
                value={newPig.ear_tag}
                onChange={(e) => setNewPig({ ...newPig, ear_tag: e.target.value })}
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
                value={newPig.breed}
                onValueChange={(v) => setNewPig({ ...newPig, breed: v as any })}
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
            {newPig.breed === 'other' && (
              <div className="space-y-2">
                <Label htmlFor="breed_other">填寫品種 *</Label>
                <Input
                  id="breed_other"
                  value={newPig.breed_other}
                  onChange={(e) => setNewPig({ ...newPig, breed_other: e.target.value })}
                  placeholder="請輸入品種名稱"
                />
              </div>
            )}
            <div className="space-y-2">
              <Label>性別 *</Label>
              <Select
                value={newPig.gender}
                onValueChange={(v) => setNewPig({ ...newPig, gender: v as any })}
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
                value={newPig.source_id || 'none'}
                onValueChange={(v) => setNewPig({ ...newPig, source_id: v === 'none' ? '' : v })}
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
                value={newPig.entry_date}
                onChange={(e) => setNewPig({ ...newPig, entry_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="birth_date">出生日期 *</Label>
              <Input
                id="birth_date"
                type="date"
                value={newPig.birth_date}
                onChange={(e) => setNewPig({ ...newPig, birth_date: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="entry_weight">進場體重 (kg) *</Label>
              <Input
                id="entry_weight"
                type="text"
                inputMode="decimal"
                value={newPig.entry_weight}
                onChange={(e) => {
                  const value = e.target.value
                  // 只允許數字和一個小數點
                  const numericValue = value.replace(/[^\d.]/g, '')
                  // 確保只有一個小數點
                  const parts = numericValue.split('.')
                  const filteredValue = parts.length > 2
                    ? parts[0] + '.' + parts.slice(1).join('')
                    : numericValue
                  setNewPig({ ...newPig, entry_weight: filteredValue })
                }}
                placeholder="輸入體重"
              />
            </div>
            <div className="space-y-2 col-span-2">
              <Label htmlFor="pre_experiment_code">實驗前代號 *</Label>
              <Input
                id="pre_experiment_code"
                value={newPig.pre_experiment_code}
                onChange={(e) => setNewPig({ ...newPig, pre_experiment_code: e.target.value })}
                placeholder="例如 PIG-110000"
              />
            </div>
            <div className="space-y-2 col-span-2">
              <Label htmlFor="remark">備註</Label>
              <Input
                id="remark"
                value={newPig.remark}
                onChange={(e) => setNewPig({ ...newPig, remark: e.target.value })}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAddDialog(false)}>
              取消
            </Button>
            <Button
              onClick={() => createPigMutation.mutate(newPig)}
              disabled={
                createPigMutation.isPending ||
                !newPig.ear_tag ||
                !penBuilding ||
                !penZone ||
                !penNumber ||
                !newPig.birth_date ||
                !newPig.entry_weight ||
                !newPig.pre_experiment_code ||
                !newPig.entry_date ||
                (newPig.breed === 'other' && !newPig.breed_other)
              }
              className="bg-purple-600 hover:bg-purple-700"
            >
              {createPigMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              新增
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      {/* Batch Assign Dialog */}
      <Dialog open={showBatchAssignDialog} onOpenChange={setShowBatchAssignDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>分配豬隻至計畫</DialogTitle>
            <DialogDescription>
              將選中的 {selectedPigs.length} 隻豬分配至指定的 IACUC 計畫
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="iacuc_no">IACUC NO. *</Label>
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
      {quickEditPigId && (
        <QuickEditPigDialog
          open={!!quickEditPigId}
          onOpenChange={(open) => {
            if (!open) setQuickEditPigId(null)
          }}
          pigId={quickEditPigId}
        />
      )}

      {showPrintReport && (
        <PigPenReport
          data={groupedData || []}
          onClose={() => setShowPrintReport(false)}
        />
      )}
    </div>
  )
}







