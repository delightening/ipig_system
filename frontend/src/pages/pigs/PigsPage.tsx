import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Link, useSearchParams } from 'react-router-dom'
import api, {
  PigListItem,
  PigStatus,
  pigStatusNames,
  pigBreedNames,
  pigGenderNames,
  PigSource,
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
  List,
  MapPin,
} from 'lucide-react'

// Import Export Dialog
import { ExportDialog } from '@/components/pig/ExportDialog'
import { ImportDialog } from '@/components/pig/ImportDialog'

const statusColors: Record<PigStatus, string> = {
  unassigned: 'bg-gray-100 text-gray-800',
  assigned: 'bg-blue-100 text-blue-800',
  in_experiment: 'bg-orange-100 text-orange-800',
  completed: 'bg-green-100 text-green-800',
}

type ViewMode = 'list' | 'grouped'

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

export function PigsPage() {
  const queryClient = useQueryClient()
  const [searchParams, setSearchParams] = useSearchParams()

  // View mode state
  const [viewMode, setViewMode] = useState<ViewMode>('list')

  // Filter state
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>(searchParams.get('status') || 'all')
  const [breedFilter, setBreedFilter] = useState<string>('all')

  // Dialog state
  const [showAddDialog, setShowAddDialog] = useState(false)
  const [showBatchAssignDialog, setShowBatchAssignDialog] = useState(false)
  const [showBatchExportDialog, setShowBatchExportDialog] = useState(false)
  const [showImportBasicDialog, setShowImportBasicDialog] = useState(false)
  const [showImportWeightDialog, setShowImportWeightDialog] = useState(false)
  const [selectedPigs, setSelectedPigs] = useState<number[]>([])
  const [assignIacucNo, setAssignIacucNo] = useState('')

  // Form state for new pig
  const [penBuilding, setPenBuilding] = useState('')
  const [penZone, setPenZone] = useState('')
  const [penNumber, setPenNumber] = useState('')
  const [newPig, setNewPig] = useState({
    ear_tag: '',
    breed: 'minipig' as const,
    gender: 'male' as const,
    source_id: '',
    entry_date: new Date().toISOString().split('T')[0],
    entry_weight: '',
    birth_date: '',
    pre_experiment_code: '',
    remark: '',
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
    enabled: viewMode === 'grouped',
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
      
      // 清理資料格式
      const payload: any = {
        ear_tag: data.ear_tag.trim(),
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
      toast({ title: '成功', description: '豬隻已新增' })
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

  // 閮??????
  // 計算狀態計數（基於所有豬隻，而非過濾後的結果）
  const statusCounts = allPigs.reduce((acc, pig) => {
    acc[pig.status] = (acc[pig.status] || 0) + 1
    return acc
  }, {} as Record<string, number>)

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">豬隻管理</h1>
          <p className="text-slate-500">管理系統中的所有實驗豬隻</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" className="gap-2" onClick={() => setShowBatchExportDialog(true)}>
            <FileSpreadsheet className="h-4 w-4" />
            批次匯出病歷
          </Button>
          <Button variant="outline" className="gap-2" onClick={() => setShowImportBasicDialog(true)}>
            <Upload className="h-4 w-4" />
            匯入基本資料
          </Button>
          <Button variant="outline" className="gap-2" onClick={() => setShowImportWeightDialog(true)}>
            <Upload className="h-4 w-4" />
            匯入體重
          </Button>
          <Button onClick={() => setShowAddDialog(true)} className="gap-2 bg-purple-600 hover:bg-purple-700">
            <Plus className="h-4 w-4" />
            新增豬隻
          </Button>
        </div>
      </div>

      {/* Status Tabs */}
      <div className="flex gap-2 border-b">
        {[
          { value: 'all', label: '全部', count: allPigs.length },
          { value: 'unassigned', label: '未分配', count: statusCounts['unassigned'] || 0 },
          { value: 'assigned', label: '已分配', count: statusCounts['assigned'] || 0 },
          { value: 'in_experiment', label: '實驗中', count: statusCounts['in_experiment'] || 0 },
          { value: 'completed', label: '實驗完成', count: statusCounts['completed'] || 0 },
        ].map(tab => (
          <button
            key={tab.value}
            onClick={() => {
              setStatusFilter(tab.value)
              setSearchParams(tab.value === 'all' ? {} : { status: tab.value })
            }}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${statusFilter === tab.value
              ? 'border-purple-600 text-purple-600'
              : 'border-transparent text-slate-500 hover:text-slate-700'
              }`}
          >
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

              {/* View Mode Toggle */}
              <div className="flex items-center border rounded-lg overflow-hidden">
                <button
                  onClick={() => setViewMode('list')}
                  className={`flex items-center gap-1 px-3 py-2 text-sm transition-colors ${viewMode === 'list'
                    ? 'bg-purple-100 text-purple-700'
                    : 'bg-white text-slate-600 hover:bg-slate-50'
                    }`}
                >
                  <List className="h-4 w-4" />
                  列表
                </button>
                <button
                  onClick={() => setViewMode('grouped')}
                  className={`flex items-center gap-1 px-3 py-2 text-sm border-l transition-colors ${viewMode === 'grouped'
                    ? 'bg-purple-100 text-purple-700'
                    : 'bg-white text-slate-600 hover:bg-slate-50'
                    }`}
                >
                  <LayoutGrid className="h-4 w-4" />
                  欄位分組
                </button>
              </div>
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
      {viewMode === 'list' && (
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
                      <TableCell>{pigBreedNames[pig.breed]}</TableCell>
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
                        {new Date(pig.entry_date).toLocaleDateString('zh-TW')}
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
      {viewMode === 'grouped' && (
        <div className="space-y-4">
          {groupedLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
            </div>
          ) : !groupedData || groupedData.length === 0 ? (
            <Card>
              <CardContent className="py-12">
                <div className="flex flex-col items-center justify-center text-slate-500">
                  <MapPin className="h-12 w-12 mb-4" />
                  <p>沒有豬隻資料</p>
                </div>
              </CardContent>
            </Card>
          ) : (
            groupedData.map((group) => (
              <Card key={group.pen_location || 'unassigned'}>
                <CardHeader className="pb-3">
                  <CardTitle className="flex items-center gap-2 text-lg">
                    <MapPin className="h-5 w-5 text-purple-600" />
                    {group.pen_location || '未指定欄位'}
                    <Badge variant="outline" className="ml-2">
                      {group.pig_count} 隻
                    </Badge>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
                    {group.pigs.map((pig) => (
                      <Link
                        key={pig.id}
                        to={`/pigs/${pig.id}`}
                        className="flex items-center justify-between p-3 bg-slate-50 hover:bg-slate-100 rounded-lg transition-colors"
                      >
                        <div className="flex-1 min-w-0">
                          <p className="font-medium text-orange-600 truncate">{pig.ear_tag}</p>
                          <p className="text-sm text-slate-500 truncate">
                            {pigBreedNames[pig.breed]} · {pigGenderNames[pig.gender]}
                          </p>
                          {pig.iacuc_no && (
                            <p className="text-xs text-slate-400 truncate">{pig.iacuc_no}</p>
                          )}
                        </div>
                        <Badge className={statusColors[pig.status]}>
                          {pigStatusNames[pig.status]}
                        </Badge>
                      </Link>
                    ))}
                  </div>
                </CardContent>
              </Card>
            ))
          )}
        </div>
      )}

      {/* Add Pig Dialog */}
      <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>新增豬隻</DialogTitle>
            <DialogDescription>輸入新豬隻的基本資料</DialogDescription>
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
                !newPig.entry_date
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
    </div>
  )
}







