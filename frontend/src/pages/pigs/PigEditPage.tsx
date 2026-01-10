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

  // Query pig data
  const { data: pig, isLoading: pigLoading } = useQuery({
    queryKey: ['pig', pigId],
    queryFn: async () => {
      const res = await api.get<Pig>(`/pigs/${pigId}`)
      return res.data
    },
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
      setFormData({
        ear_tag: pig.ear_tag,
        status: pig.status,
        breed: pig.breed,
        gender: pig.gender,
        source_id: pig.source_id || undefined,
        birth_date: pig.birth_date || undefined,
        entry_date: pig.entry_date,
        entry_weight: pig.entry_weight || undefined,
        pen_location: pig.pen_location || undefined,
        pre_experiment_code: pig.pre_experiment_code || undefined,
        iacuc_no: pig.iacuc_no || undefined,
        experiment_date: pig.experiment_date || undefined,
        remark: pig.remark || undefined,
      })
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
    if (!formData.ear_tag?.trim()) {
      toast({ title: '錯誤', description: '耳號為必填', variant: 'destructive' })
      return
    }
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
          回到豬隻詳情
        </Link>
      </div>

      {/* Page Header */}
      <div>
        <h1 className="text-2xl font-bold text-slate-900">編輯豬隻資料</h1>
        <p className="text-slate-500">耳號：{pig.ear_tag}</p>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <CardHeader>
            <CardTitle>基本資料</CardTitle>
            <CardDescription>編輯豬隻的基本資訊</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-6">
              {/* 耳號 */}
              <div className="space-y-2">
                <Label htmlFor="ear_tag">耳號 *</Label>
                <Input
                  id="ear_tag"
                  value={formData.ear_tag || ''}
                  onChange={(e) => handleChange('ear_tag', e.target.value)}
                  placeholder="輸入耳號"
                  required
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

              {/* 品種 */}
              <div className="space-y-2">
                <Label>品種 *</Label>
                <Select
                  value={formData.breed}
                  onValueChange={(v) => handleChange('breed', v as PigBreed)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(pigBreedNames).map(([value, label]) => (
                      <SelectItem key={value} value={value}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* 性別 */}
              <div className="space-y-2">
                <Label>性別 *</Label>
                <Select
                  value={formData.gender}
                  onValueChange={(v) => handleChange('gender', v as PigGender)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(pigGenderNames).map(([value, label]) => (
                      <SelectItem key={value} value={value}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* 來源 */}
              <div className="space-y-2">
                <Label>來源</Label>
                <Select
                  value={formData.source_id || 'none'}
                  onValueChange={(v) => handleChange('source_id', v === 'none' ? undefined : v)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="選擇來源" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">未指定</SelectItem>
                    {sources?.map((source) => (
                      <SelectItem key={source.id} value={source.id}>
                        {source.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
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

              {/* 出生日期 */}
              <div className="space-y-2">
                <Label htmlFor="birth_date">出生日期</Label>
                <Input
                  id="birth_date"
                  type="date"
                  value={formData.birth_date?.split('T')[0] || ''}
                  onChange={(e) => handleChange('birth_date', e.target.value)}
                />
              </div>

              {/* 進場日期 */}
              <div className="space-y-2">
                <Label htmlFor="entry_date">進場日期 *</Label>
                <Input
                  id="entry_date"
                  type="date"
                  value={formData.entry_date?.split('T')[0] || ''}
                  onChange={(e) => handleChange('entry_date', e.target.value)}
                  required
                />
              </div>

              {/* 進場體重 */}
              <div className="space-y-2">
                <Label htmlFor="entry_weight">進場體重 (kg)</Label>
                <Input
                  id="entry_weight"
                  type="number"
                  step="0.1"
                  value={formData.entry_weight || ''}
                  onChange={(e) => handleChange('entry_weight', e.target.value ? parseFloat(e.target.value) : undefined)}
                  placeholder="輸入體重"
                />
              </div>

              {/* 實驗前代號 */}
              <div className="space-y-2">
                <Label htmlFor="pre_experiment_code">實驗前代號</Label>
                <Input
                  id="pre_experiment_code"
                  value={formData.pre_experiment_code || ''}
                  onChange={(e) => handleChange('pre_experiment_code', e.target.value)}
                  placeholder="如：PIG-110000"
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
