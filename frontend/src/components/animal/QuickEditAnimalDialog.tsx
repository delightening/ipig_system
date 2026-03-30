import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Animal, UpdateAnimalRequest, UpdateAnimalRequestValue, animalBreedNames, animalGenderNames, ProtocolListItem } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Loader2 } from 'lucide-react'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  animalId: string
}

export function QuickEditAnimalDialog({ open, onOpenChange, animalId }: Props) {
  const queryClient = useQueryClient()
  const [formData, setFormData] = useState<UpdateAnimalRequest>({})
  const [entryWeightInput, setEntryWeightInput] = useState<string>('')

  // Query animal data
  const { data: animal, isLoading: animalLoading } = useQuery({
    queryKey: ['animal', animalId],
    queryFn: async () => {
      const res = await api.get<Animal>(`/animals/${animalId}`)
      return res.data
    },
    enabled: open && !!animalId,
    staleTime: 30_000,
    refetchOnWindowFocus: false,
    refetchOnMount: true,
  })

  // Initialize form data when animal loads
  useEffect(() => {
    if (animal) {
      setFormData({
        entry_weight: animal.entry_weight ? Number(animal.entry_weight) : undefined,
        pen_location: animal.pen_location || undefined,
        iacuc_no: animal.iacuc_no || undefined,
      })
      setEntryWeightInput(animal.entry_weight !== undefined && animal.entry_weight !== null ? String(animal.entry_weight) : '')
    }
  }, [animal])

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
    enabled: open,
    staleTime: 600_000,
  })

  // Update mutation
  const updateMutation = useMutation({
    mutationFn: async (data: UpdateAnimalRequest) => {
      return api.put<Animal>(`/animals/${animalId}`, data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
      queryClient.invalidateQueries({ queryKey: ['animals'] })
      queryClient.invalidateQueries({ queryKey: ['animals-stats'] })
      toast({ title: '成功', description: '動物資料已更新' })
      onOpenChange(false)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '更新失敗'),
        variant: 'destructive',
      })
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    updateMutation.mutate(formData)
  }

  const handleChange = (field: keyof UpdateAnimalRequest, value: UpdateAnimalRequestValue) => {
    setFormData((prev) => ({ ...prev, [field]: value || undefined }))
  }

  if (!open) return null

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>快速編輯動物資料</DialogTitle>
          <DialogDescription>編輯動物的基本資訊（僅可編輯標示為可編輯的欄位）</DialogDescription>
        </DialogHeader>

        {animalLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : animal ? (
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* 唯讀資訊區域 */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-foreground border-b pb-2">基本資訊（不可編輯）</h3>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label className="text-muted-foreground">系統號</Label>
                  <Input value={animal.id.slice(0, 8)} disabled className="bg-muted" title={animal.id} />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">耳號</Label>
                  <Input value={animal.ear_tag} disabled className="bg-muted" />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">品種</Label>
                  <Input value={animal.breed === 'other' ? (animal.breed_other || '其他') : animalBreedNames[animal.breed]} disabled className="bg-muted" />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">性別</Label>
                  <Input value={animalGenderNames[animal.gender]} disabled className="bg-muted" />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">出生日期</Label>
                  <Input
                    value={animal.birth_date ? new Date(animal.birth_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' }) : '-'}
                    disabled
                    className="bg-muted"
                  />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">進場日期</Label>
                  <Input
                    value={new Date(animal.entry_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })}
                    disabled
                    className="bg-muted"
                  />
                </div>
              </div>
            </div>

            {/* 可编辑字段区域 */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-status-purple-text border-b border-status-purple-border pb-2">
                可編輯欄位
              </h3>
              <div className="grid grid-cols-2 gap-4">
                {/* 進場體重 */}
                <div className="space-y-2">
                  <Label htmlFor="entry_weight">
                    進場體重 (kg) <span className="text-status-error-solid">*</span>
                  </Label>
                  <Input
                    id="entry_weight"
                    type="text"
                    inputMode="decimal"
                    value={entryWeightInput}
                    onChange={(e) => {
                      const value = e.target.value
                      // 只允許數字和一個小數點
                      const numericValue = value.replace(/[^\d.]/g, '')
                      // 確保只有一個小數點
                      const parts = numericValue.split('.')
                      const filteredValue = parts.length > 2
                        ? parts[0] + '.' + parts.slice(1).join('')
                        : numericValue
                      // 更新輸入值
                      setEntryWeightInput(filteredValue)
                      // 如果為空或只有小數點，設為 undefined，否則轉換為數字
                      if (filteredValue === '' || filteredValue === '.') {
                        handleChange('entry_weight', undefined)
                      } else {
                        const numValue = parseFloat(filteredValue)
                        if (!isNaN(numValue)) {
                          handleChange('entry_weight', numValue)
                        }
                      }
                    }}
                    onBlur={() => {
                      // 當失去焦點時，清理尾部的小數點
                      if (entryWeightInput === '.') {
                        setEntryWeightInput('')
                        handleChange('entry_weight', undefined)
                      } else if (entryWeightInput && entryWeightInput.endsWith('.')) {
                        const cleaned = entryWeightInput.slice(0, -1)
                        setEntryWeightInput(cleaned)
                        const numValue = parseFloat(cleaned)
                        if (!isNaN(numValue)) {
                          handleChange('entry_weight', numValue)
                        }
                      }
                    }}
                    placeholder="輸入體重"
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

                {/* IACUC No. */}
                <div className="space-y-2">
                  <Label htmlFor="iacuc_no">IACUC No.</Label>
                  <Select
                    value={formData.iacuc_no || ''}
                    onValueChange={(v) => handleChange('iacuc_no', v === '' ? undefined : v)}
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
                </div>
              </div>
            </div>

            {/* 資訊提示區域（唯讀） */}
            <div className="space-y-4 pt-4 border-t">
              <h3 className="text-sm font-semibold text-foreground border-b pb-2">狀態資訊（僅供參考）</h3>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label className="text-muted-foreground">用藥中</Label>
                  <Input
                    value="此資訊由系統自動計算，請至詳情頁查看相關記錄"
                    disabled
                    className="bg-muted text-muted-foreground text-xs"
                  />
                </div>
                <div className="space-y-2">
                  <Label className="text-muted-foreground">獸醫師建議</Label>
                  <Input
                    value="請至詳情頁查看獸醫師建議"
                    disabled
                    className="bg-muted text-muted-foreground text-xs"
                  />
                </div>
              </div>
            </div>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={updateMutation.isPending}
              >
                取消
              </Button>
              <Button type="submit" disabled={updateMutation.isPending} className="bg-purple-600 hover:bg-purple-700">
                {updateMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                儲存變更
              </Button>
            </DialogFooter>
          </form>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
