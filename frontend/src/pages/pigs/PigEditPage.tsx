import { useEffect, useState } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  Pig,
  PigSource,
  PigStatus,
  PigBreed,
  PigGender,
  pigStatusNames,
  pigBreedNames,
  pigGenderNames,
  UpdatePigRequest,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import {
  ArrowLeft,
  Loader2,
  Save,
  AlertCircle,
} from 'lucide-react'

export function PigEditPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const pigId = parseInt(id!)

  const [formData, setFormData] = useState<UpdatePigRequest>({})
  const [hasChanges, setHasChanges] = useState(false)
  const [entryWeightInput, setEntryWeightInput] = useState<string>('')

  // Query pig data
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

  // Query sources
  const { data: sources } = useQuery({
    queryKey: ['pig-sources'],
    queryFn: async () => {
      const res = await api.get<PigSource[]>('/pig-sources')
      return res.data
    },
  })

  // Initialize form data when pig loads
  useEffect(() => {
    if (pig) {
      // 以下字段在创建后不可更改，只用于显示，不会提交到后端：
      // - ear_tag, breed, gender, source_id, birth_date, entry_date, entry_weight, pre_experiment_code
      setFormData({
        status: pig.status,
        pen_location: pig.pen_location || undefined,
        iacuc_no: pig.iacuc_no || undefined,
        experiment_date: pig.experiment_date || undefined,
        remark: pig.remark || undefined,
      })
      setEntryWeightInput(pig.entry_weight !== undefined && pig.entry_weight !== null ? String(pig.entry_weight) : '')
    }
  }, [pig])

  // Update mutation
  const updateMutation = useMutation({
    mutationFn: async (data: UpdatePigRequest) => {
      return api.put(`/pigs/${pigId}`, data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig', pigId] })
      queryClient.invalidateQueries({ queryKey: ['pigs'] })
      toast({ title: '成功', description: '豬隻資料已更新' })
      navigate(`/pigs/${pigId}`)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '更新失敗',
        variant: 'destructive',
      })
    },
  })

  const handleChange = (field: keyof UpdatePigRequest, value: any) => {
    setFormData((prev) => ({ ...prev, [field]: value || undefined }))
    setHasChanges(true)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    // 只提交可编辑的字段（已移除不可更改的字段）
    updateMutation.mutate(formData)
  }

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

  return (
    <div className="space-y-6 max-w-4xl mx-auto">
      {/* Back Button */}
      <div className="flex items-center justify-between">
        <Link to={`/pigs/${pigId}`} className="inline-flex items-center text-slate-600 hover:text-slate-900">
          <ArrowLeft className="h-4 w-4 mr-2" />
          回到動物詳情
        </Link>
      </div>

      {/* Page Header */}
      <div>
        <h1 className="text-2xl font-bold text-slate-900">編輯動物資料</h1>
        <p className="text-slate-500">耳號：{pig.ear_tag}</p>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <CardHeader>
            <CardTitle>基本資料</CardTitle>
            <CardDescription>編輯動物的基本資訊</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-6">
              {/* 耳號 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label htmlFor="ear_tag" className="text-slate-500">耳號 *</Label>
                <Input
                  id="ear_tag"
                  value={pig?.ear_tag || ''}
                  disabled
                  className="bg-slate-50"
                />
              </div>

              {/* 狀態 */}
              <div className="space-y-2">
                <Label>狀態 *</Label>
                <Select
                  value={formData.status}
                  onValueChange={(v) => handleChange('status', v as PigStatus)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(pigStatusNames).map(([value, label]) => (
                      <SelectItem key={value} value={value}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* 品種 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label className="text-slate-500">品種 *</Label>
                <Input
                  value={pig ? (pig.breed === 'other' ? (pig.breed_other || '其他') : pigBreedNames[pig.breed]) : ''}
                  disabled
                  className="bg-slate-50"
                />
              </div>

              {/* 性別 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label className="text-slate-500">性別 *</Label>
                <Input
                  value={pig ? pigGenderNames[pig.gender] : ''}
                  disabled
                  className="bg-slate-50"
                />
              </div>

              {/* 來源 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label className="text-slate-500">來源</Label>
                <Input
                  value={pig?.source_id ? sources?.find(s => s.id === pig.source_id)?.name || '' : '未指定'}
                  disabled
                  className="bg-slate-50"
                />
              </div>

              {/* 欄位 */}
              <div className="space-y-2">
                <Label htmlFor="pen_location">欄位</Label>
                <Input
                  id="pen_location"
                  value={formData.pen_location || ''}
                  onChange={(e) => handleChange('pen_location', e.target.value)}
                  placeholder="如：A01"
                />
              </div>

              {/* 出生日期 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label htmlFor="birth_date" className="text-slate-500">出生日期</Label>
                <Input
                  id="birth_date"
                  type="date"
                  value={pig?.birth_date ? new Date(pig.birth_date).toISOString().split('T')[0] : ''}
                  disabled
                  className="bg-slate-50"
                />
              </div>

              {/* 進場日期 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label htmlFor="entry_date" className="text-slate-500">進場日期 *</Label>
                <Input
                  id="entry_date"
                  type="date"
                  value={pig?.entry_date ? new Date(pig.entry_date).toISOString().split('T')[0] : ''}
                  disabled
                  className="bg-slate-50"
                />
              </div>

              {/* 進場體重 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label htmlFor="entry_weight" className="text-slate-500">進場體重 (kg)</Label>
                <Input
                  id="entry_weight"
                  type="text"
                  value={pig?.entry_weight !== undefined && pig.entry_weight !== null ? String(pig.entry_weight) : ''}
                  disabled
                  className="bg-slate-50"
                />
              </div>

              {/* 實驗前代號 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label htmlFor="pre_experiment_code" className="text-slate-500">實驗前代號</Label>
                <Input
                  id="pre_experiment_code"
                  value={pig?.pre_experiment_code || ''}
                  disabled
                  className="bg-slate-50"
                />
              </div>

              {/* IACUC NO. */}
              <div className="space-y-2">
                <Label htmlFor="iacuc_no">IACUC NO.</Label>
                <Input
                  id="iacuc_no"
                  value={formData.iacuc_no || ''}
                  onChange={(e) => handleChange('iacuc_no', e.target.value)}
                  placeholder="如：PIG-114017"
                />
              </div>

              {/* 實驗日期 */}
              <div className="space-y-2">
                <Label htmlFor="experiment_date">實驗日期</Label>
                <Input
                  id="experiment_date"
                  type="date"
                  value={formData.experiment_date?.split('T')[0] || ''}
                  onChange={(e) => handleChange('experiment_date', e.target.value)}
                />
              </div>

              {/* 備註 */}
              <div className="space-y-2 col-span-2">
                <Label htmlFor="remark">備註</Label>
                <Textarea
                  id="remark"
                  value={formData.remark || ''}
                  onChange={(e) => handleChange('remark', e.target.value)}
                  placeholder="其他備註..."
                  className="min-h-[100px]"
                />
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Submit Buttons */}
        <div className="flex items-center justify-end gap-3 mt-6">
          <Button
            type="button"
            variant="outline"
            onClick={() => navigate(`/pigs/${pigId}`)}
          >
            取消
          </Button>
          <Button
            type="submit"
            disabled={updateMutation.isPending || !hasChanges}
            className="bg-purple-600 hover:bg-purple-700"
          >
            {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            <Save className="h-4 w-4 mr-2" />
            儲存變更
          </Button>
        </div>
      </form>
    </div>
  )
}
