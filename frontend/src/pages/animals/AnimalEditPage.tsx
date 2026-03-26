import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  Animal,
  AnimalSource,
  AnimalStatus,
  animalStatusNames,
  animalBreedNames,
  animalGenderNames,
  ProtocolListItem,
} from '@/lib/api'
import { getApiErrorMessage, animalEditSchema, type AnimalEditFormData } from '@/lib/validation'
import { Button } from '@/components/ui/button'
import { PageHeader } from '@/components/ui/page-header'
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
  FileEdit,
} from 'lucide-react'
import { RequestCorrectionDialog } from '@/components/animal/RequestCorrectionDialog'
import { animalFieldCorrectionApi } from '@/lib/api'

export function AnimalEditPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const animalId = id!

  const [correctionDialogOpen, setCorrectionDialogOpen] = useState(false)

  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors, isDirty },
  } = useForm<AnimalEditFormData>({
    resolver: zodResolver(animalEditSchema),
    defaultValues: {
      status: '',
      pen_location: '',
      iacuc_no: '',
      experiment_date: '',
      remark: '',
    },
  })

  const watchedStatus = watch('status')
  const watchedIacucNo = watch('iacuc_no')

  // Query animal data
  const { data: animal, isLoading: animalLoading } = useQuery({
    queryKey: ['animal', animalId],
    queryFn: async () => {
      const res = await api.get<Animal>(`/animals/${animalId}`)
      return res.data
    },
    staleTime: 5 * 60_000, // M10: 5 分鐘內不重新拉取
  })

  // Query sources
  const { data: sources } = useQuery({
    queryKey: ['animal-sources'],
    queryFn: async () => {
      const res = await api.get<AnimalSource[]>('/animal-sources')
      return res.data
    },
    staleTime: 600_000,
  })

  // Query approved protocols (for IACUC No. dropdown)
  const { data: approvedProtocols } = useQuery({
    queryKey: ['approved-protocols'],
    queryFn: async () => {
      const res = await api.get<ProtocolListItem[]>('/protocols')
      return res.data.filter(p => {
        if (p.status === 'CLOSED') return false
        if (!((p.status === 'APPROVED' || p.status === 'APPROVED_WITH_CONDITIONS') && p.iacuc_no)) return false
        return true
      })
    },
  })

  // Initialize form data when animal loads
  useEffect(() => {
    if (animal) {
      reset({
        status: animal.status,
        pen_location: animal.pen_location || '',
        iacuc_no: animal.iacuc_no || '',
        experiment_date: animal.experiment_date || '',
        remark: animal.remark || '',
      })
    }
  }, [animal, reset])

  // Update mutation
  const updateMutation = useMutation({
    mutationFn: async (data: AnimalEditFormData) => {
      return api.put(`/animals/${animalId}`, {
        status: data.status || undefined,
        pen_location: data.pen_location || undefined,
        iacuc_no: data.iacuc_no || undefined,
        experiment_date: data.experiment_date || undefined,
        remark: data.remark || undefined,
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      toast({ title: '成功', description: '動物資料已更新' })
      navigate(`/animals/${animalId}`)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '更新失敗'),
        variant: 'destructive',
      })
    },
  })

  const onValid = (data: AnimalEditFormData) => {
    updateMutation.mutate(data)
  }

  if (animalLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!animal) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px]">
        <AlertCircle className="h-12 w-12 text-muted-foreground mb-4" />
        <p className="text-muted-foreground">找不到此動物</p>
        <Button variant="outline" className="mt-4" onClick={() => navigate('/animals')}>
          返回列表
        </Button>
      </div>
    )
  }

  return (
    <div className="space-y-6 max-w-4xl mx-auto">
      {/* Back Button */}
      <div className="flex items-center justify-between">
        <Link to={`/animals/${animalId}`} className="inline-flex items-center text-muted-foreground hover:text-foreground">
          <ArrowLeft className="h-4 w-4 mr-2" />
          回到動物詳情
        </Link>
      </div>

      <PageHeader
        title="編輯動物資料"
        description={`耳號：${animal.ear_tag}`}
        actions={
          <Button
            size="sm"
            type="button"
            variant="outline"
            onClick={() => setCorrectionDialogOpen(true)}
            className="text-status-warning-text border-status-warning-border hover:bg-status-warning-bg"
          >
            <FileEdit className="h-4 w-4 mr-2" />
            申請修正（耳號/出生日期/性別/品種）
          </Button>
        }
      />

      <RequestCorrectionDialog
        open={correctionDialogOpen}
        onOpenChange={setCorrectionDialogOpen}
        animal={animal}
        onSubmit={async (data) => {
          await animalFieldCorrectionApi.create(animalId, data)
          queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
          toast({ title: '成功', description: '修正申請已提交，待管理員審核' })
        }}
      />

      <form onSubmit={handleSubmit(onValid)}>
        <Card>
          <CardHeader>
            <CardTitle>基本資料</CardTitle>
            <CardDescription>編輯動物的基本資訊</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-6">
              {/* 耳號 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label htmlFor="ear_tag" className="text-muted-foreground">耳號 *</Label>
                <Input
                  id="ear_tag"
                  value={animal?.ear_tag || ''}
                  disabled
                  className="bg-muted"
                />
              </div>

              {/* 狀態 */}
              <div className="space-y-2">
                <Label>狀態 *</Label>
                <Select
                  value={watchedStatus ?? ''}
                  onValueChange={(v) => setValue('status', v as AnimalStatus, { shouldValidate: true, shouldDirty: true })}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(animalStatusNames)
                      .filter(([value]) => value !== 'transferred' && value !== 'sudden_death')
                      .map(([value, label]) => (
                        <SelectItem key={value} value={value}>
                          {label}
                        </SelectItem>
                      ))}
                  </SelectContent>
                </Select>
                {errors.status && (
                  <p className="text-sm text-destructive">{errors.status.message}</p>
                )}
              </div>

              {/* 品種 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label className="text-muted-foreground">品種 *</Label>
                <Input
                  value={animal ? (animal.breed === 'other' ? (animal.breed_other || '其他') : animalBreedNames[animal.breed]) : ''}
                  disabled
                  className="bg-muted"
                />
              </div>

              {/* 性別 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label className="text-muted-foreground">性別 *</Label>
                <Input
                  value={animal ? animalGenderNames[animal.gender] : ''}
                  disabled
                  className="bg-muted"
                />
              </div>

              {/* 來源 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label className="text-muted-foreground">來源</Label>
                <Input
                  value={animal?.source_id ? sources?.find(s => s.id === animal.source_id)?.name || '' : '未指定'}
                  disabled
                  className="bg-muted"
                />
              </div>

              {/* 欄位 */}
              <div className="space-y-2">
                <Label htmlFor="pen_location">欄位</Label>
                <Input
                  id="pen_location"
                  {...register('pen_location')}
                  placeholder="如：A01"
                />
              </div>

              {/* 出生日期 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label htmlFor="birth_date" className="text-muted-foreground">出生日期</Label>
                <Input
                  id="birth_date"
                  type="date"
                  value={animal?.birth_date ? new Date(animal.birth_date).toISOString().split('T')[0] : ''}
                  disabled
                  className="bg-muted"
                />
              </div>

              {/* 進場日期 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label htmlFor="entry_date" className="text-muted-foreground">進場日期 *</Label>
                <Input
                  id="entry_date"
                  type="date"
                  value={animal?.entry_date ? new Date(animal.entry_date).toISOString().split('T')[0] : ''}
                  disabled
                  className="bg-muted"
                />
              </div>

              {/* 進場體重 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label htmlFor="entry_weight" className="text-muted-foreground">進場體重 (kg)</Label>
                <Input
                  id="entry_weight"
                  type="text"
                  value={animal?.entry_weight !== undefined && animal.entry_weight !== null ? String(animal.entry_weight) : ''}
                  disabled
                  className="bg-muted"
                />
              </div>

              {/* 實驗前代號 - 创建后不可更改 */}
              <div className="space-y-2">
                <Label htmlFor="pre_experiment_code" className="text-muted-foreground">實驗前代號</Label>
                <Input
                  id="pre_experiment_code"
                  value={animal?.pre_experiment_code || ''}
                  disabled
                  className="bg-muted"
                />
              </div>

              {/* IACUC No. */}
              <div className="space-y-2">
                <Label>
                  IACUC No.
                  {watchedStatus === 'in_experiment' && <span className="text-destructive ml-1">*</span>}
                </Label>
                {animal?.status === 'in_experiment' ? (
                  <>
                    <Input
                      value={animal?.iacuc_no || ''}
                      disabled
                      className="bg-muted"
                    />
                    <p className="text-xs text-status-warning-text">實驗中的動物無法更改 IACUC No.</p>
                  </>
                ) : (
                  <Select
                    value={watchedIacucNo || ''}
                    onValueChange={(v) => setValue('iacuc_no', v === '' ? '' : v, { shouldValidate: true, shouldDirty: true })}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="選擇 IACUC No." />
                    </SelectTrigger>
                    <SelectContent>
                      {approvedProtocols?.map((protocol) => (
                        <SelectItem key={protocol.id} value={protocol.iacuc_no!}>
                          {protocol.iacuc_no}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
                {errors.iacuc_no && (
                  <p className="text-sm text-destructive">{errors.iacuc_no.message}</p>
                )}
              </div>

              <div className="space-y-2">
                <Label>實驗日期</Label>
                <Input
                  type="date"
                  {...register('experiment_date')}
                />
              </div>

              {/* 備註 */}
              <div className="space-y-2 col-span-2">
                <Label htmlFor="remark">備註</Label>
                <Textarea
                  id="remark"
                  {...register('remark')}
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
            onClick={() => navigate(`/animals/${animalId}`)}
          >
            取消
          </Button>
          <Button
            type="submit"
            disabled={updateMutation.isPending || !isDirty}
            className="bg-primary hover:bg-primary/90"
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
