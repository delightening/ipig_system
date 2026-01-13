import { useState, useEffect } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import api, { PigSacrifice } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { FileUpload, FileInfo } from '@/components/ui/file-upload'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { Loader2 } from 'lucide-react'

// 採樣部位選項
const SAMPLING_OPTIONS = [
  { value: '心', label: '心' },
  { value: '肝', label: '肝' },
  { value: '脾', label: '脾' },
  { value: '肺', label: '肺' },
  { value: '腎', label: '腎' },
  { value: '眼', label: '眼' },
  { value: '耳', label: '耳' },
  { value: '舌', label: '舌' },
  { value: '腦', label: '腦' },
  { value: '骨組織', label: '骨組織' },
  { value: '脂肪', label: '脂肪' },
  { value: '肌肉', label: '肌肉' },
  { value: '皮膚', label: '皮膚' },
  { value: '其他', label: '其他' },
]

// 犧牲方式選項
const SACRIFICE_METHOD_OPTIONS = [
  { value: 'bloodletting', label: '放血' },
  { value: 'electrocution', label: '電暈' },
  { value: 'other', label: '其他' },
]

// 表單狀態
interface SacrificeFormData {
  sacrifice_date: string
  zoletil_dose: string
  method_electrocution: boolean
  method_bloodletting: boolean
  method_other: string
  method_other_enabled: boolean // 追蹤「其他」選項是否被選中
  sampling: string[]
  sampling_other: string
  blood_volume_ml: string
  confirmed_sacrifice: boolean
  photos: FileInfo[]
}

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  pigId: number
  earTag: string
  sacrifice?: PigSacrifice // 編輯時傳入
}

export function SacrificeFormDialog({ open, onOpenChange, pigId, earTag, sacrifice }: Props) {
  const queryClient = useQueryClient()
  const isEdit = !!sacrifice

  const defaultFormData: SacrificeFormData = {
    sacrifice_date: new Date().toISOString().split('T')[0],
    zoletil_dose: '',
    method_electrocution: false,
    method_bloodletting: false,
    method_other: '',
    method_other_enabled: false,
    sampling: [],
    sampling_other: '',
    blood_volume_ml: '',
    confirmed_sacrifice: false,
    photos: [],
  }

  const [formData, setFormData] = useState<SacrificeFormData>(defaultFormData)

  // 編輯時填入資料
  useEffect(() => {
    if (sacrifice) {
      const samplingArray = sacrifice.sampling
        ? sacrifice.sampling.split(',').map((s) => s.trim()).filter(Boolean)
        : []
      
      setFormData({
        sacrifice_date: sacrifice.sacrifice_date
          ? sacrifice.sacrifice_date.split('T')[0]
          : new Date().toISOString().split('T')[0],
        zoletil_dose: sacrifice.zoletil_dose || '',
        method_electrocution: sacrifice.method_electrocution,
        method_bloodletting: sacrifice.method_bloodletting,
        method_other: sacrifice.method_other || '',
        method_other_enabled: !!sacrifice.method_other,
        sampling: samplingArray,
        sampling_other: sacrifice.sampling_other || '',
        blood_volume_ml: sacrifice.blood_volume_ml?.toString() || '',
        confirmed_sacrifice: sacrifice.confirmed_sacrifice,
        photos: [],
      })
    } else {
      setFormData(defaultFormData)
    }
  }, [sacrifice, open])

  const handleSamplingChange = (value: string, checked: boolean) => {
    if (checked) {
      setFormData((prev) => ({
        ...prev,
        sampling: [...prev.sampling, value],
      }))
    } else {
      setFormData((prev) => ({
        ...prev,
        sampling: prev.sampling.filter((s) => s !== value),
      }))
    }
  }

  // 處理照片上傳
  const handlePhotoUpload = async (file: File): Promise<FileInfo> => {
    const formData = new FormData()
    formData.append('file', file)

    const response = await api.post<{
      id: string
      file_name: string
      file_path: string
      file_size: number
      mime_type: string
    }>(`/pigs/${pigId}/sacrifice/photos`, formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    })

    return {
      id: response.data.id,
      file_name: response.data.file_name,
      file_path: response.data.file_path,
      file_size: response.data.file_size,
      file_type: response.data.mime_type,
      preview_url: response.data.file_path.startsWith('http')
        ? response.data.file_path
        : `/api/files/${response.data.file_path}`,
    }
  }

  const mutation = useMutation({
    mutationFn: async (data: SacrificeFormData) => {
      const payload = {
        sacrifice_date: data.sacrifice_date || null,
        zoletil_dose: data.zoletil_dose || null,
        method_electrocution: data.method_electrocution,
        method_bloodletting: data.method_bloodletting,
        method_other: data.method_other || null,
        sampling: data.sampling.length > 0 ? data.sampling.join(',') : null,
        sampling_other: data.sampling_other || null,
        blood_volume_ml: data.blood_volume_ml ? parseFloat(data.blood_volume_ml) : null,
        confirmed_sacrifice: data.confirmed_sacrifice,
      }

      return api.put(`/pigs/${pigId}/sacrifice`, payload)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-sacrifice', pigId] })
      queryClient.invalidateQueries({ queryKey: ['pig', pigId] })
      toast({ title: '成功', description: isEdit ? '犧牲紀錄已更新' : '犧牲紀錄已建立' })
      onOpenChange(false)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '儲存失敗',
        variant: 'destructive',
      })
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    mutation.mutate(formData)
  }

  const hasOtherSampling = formData.sampling.includes('其他')

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{isEdit ? '編輯犧牲/採樣紀錄' : '新增犧牲/採樣紀錄'}</DialogTitle>
          <DialogDescription>耳號：{earTag}</DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* 犧牲日期 */}
          <div className="space-y-2">
            <Label htmlFor="sacrifice_date">犧牲日期 *</Label>
            <Input
              id="sacrifice_date"
              type="date"
              value={formData.sacrifice_date}
              onChange={(e) => setFormData({ ...formData, sacrifice_date: e.target.value })}
              required
            />
          </div>

          {/* 犧牲方式 */}
          <div className="space-y-4">
            <Label>犧牲方式</Label>
            
            {/* 麻醉（填寫劑量） */}
            <div className="space-y-2">
              <Label htmlFor="zoletil_dose" className="text-sm font-normal">
                Zoletil-50 (ml)
              </Label>
              <Input
                id="zoletil_dose"
                type="text"
                value={formData.zoletil_dose}
                onChange={(e) => setFormData({ ...formData, zoletil_dose: e.target.value })}
                placeholder="請輸入劑量"
              />
            </div>

            {/* 其他方式選項 */}
            <div className="space-y-2">
              <div className="flex flex-wrap gap-4">
                <Checkbox
                  label="220V電擊"
                  checked={formData.method_electrocution}
                  onCheckedChange={(checked) =>
                    setFormData({ ...formData, method_electrocution: checked })
                  }
                />
                <Checkbox
                  label="放血"
                  checked={formData.method_bloodletting}
                  onCheckedChange={(checked) =>
                    setFormData({ ...formData, method_bloodletting: checked })
                  }
                />
                <Checkbox
                  label="其他"
                  checked={formData.method_other_enabled}
                  onCheckedChange={(checked) => {
                    setFormData({ 
                      ...formData, 
                      method_other_enabled: checked,
                      method_other: checked ? (formData.method_other || '') : ''
                    })
                  }}
                />
              </div>
              {formData.method_other_enabled && (
                <Input
                  type="text"
                  value={formData.method_other}
                  onChange={(e) => setFormData({ ...formData, method_other: e.target.value })}
                  placeholder="請輸入其他方式"
                  className="mt-2"
                />
              )}
            </div>
          </div>

          {/* 採樣部位 */}
          <div className="space-y-4">
            <Label>採樣部位</Label>
            <div className="grid grid-cols-4 gap-3">
              {SAMPLING_OPTIONS.map((option) => (
                <Checkbox
                  key={option.value}
                  label={option.label}
                  checked={formData.sampling.includes(option.value)}
                  onCheckedChange={(checked) => handleSamplingChange(option.value, checked)}
                />
              ))}
            </div>
            {hasOtherSampling && (
              <div className="mt-2">
                <Input
                  type="text"
                  value={formData.sampling_other}
                  onChange={(e) => setFormData({ ...formData, sampling_other: e.target.value })}
                  placeholder="請輸入其他採樣部位說明"
                />
              </div>
            )}
          </div>

          {/* 採樣血液 */}
          <div className="space-y-2">
            <Label htmlFor="blood_volume_ml">採樣血液 (ml)</Label>
            <Input
              id="blood_volume_ml"
              type="number"
              step="0.1"
              value={formData.blood_volume_ml}
              onChange={(e) => setFormData({ ...formData, blood_volume_ml: e.target.value })}
              placeholder="請輸入血液採樣量"
            />
          </div>

          {/* 確定犧牲 */}
          <div className="space-y-2">
            <Checkbox
              label="確定犧牲"
              checked={formData.confirmed_sacrifice}
              onCheckedChange={(checked) =>
                setFormData({ ...formData, confirmed_sacrifice: checked })
              }
            />
          </div>

          {/* 上傳照片 */}
          <div className="space-y-2">
            <Label>上傳照片</Label>
            <FileUpload
              value={formData.photos}
              onChange={(photos) => setFormData({ ...formData, photos })}
              onUpload={handlePhotoUpload}
              accept="image/*"
              placeholder="拖曳照片到此處，或點擊選擇照片"
              maxSize={10}
              maxFiles={10}
            />
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button
              type="submit"
              disabled={mutation.isPending}
              className="bg-green-600 hover:bg-green-700"
            >
              {mutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
