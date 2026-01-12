import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Pig, UpdatePigRequest, pigBreedNames, pigGenderNames } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  pigId: number
}

export function QuickEditPigDialog({ open, onOpenChange, pigId }: Props) {
  const queryClient = useQueryClient()
  const [formData, setFormData] = useState<UpdatePigRequest>({})
  const [entryWeightInput, setEntryWeightInput] = useState<string>('')

  // Query pig data
  const { data: pig, isLoading: pigLoading } = useQuery({
    queryKey: ['pig', pigId],
    queryFn: async () => {
      const res = await api.get<Pig>(`/pigs/${pigId}`)
      return res.data
    },
    enabled: open && pigId > 0,
    staleTime: 0,
    refetchOnWindowFocus: false,
    refetchOnMount: true,
  })

  // Initialize form data when pig loads
  useEffect(() => {
    if (pig) {
      setFormData({
        entry_weight: pig.entry_weight ? Number(pig.entry_weight) : undefined,
        pen_location: pig.pen_location || undefined,
        iacuc_no: pig.iacuc_no || undefined,
      })
      setEntryWeightInput(pig.entry_weight !== undefined && pig.entry_weight !== null ? String(pig.entry_weight) : '')
    }
  }, [pig])

  // Update mutation
  const updateMutation = useMutation({
    mutationFn: async (data: UpdatePigRequest) => {
      return api.put<Pig>(`/pigs/${pigId}`, data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig', pigId] })
      queryClient.invalidateQueries({ queryKey: ['pigs'] })
      queryClient.invalidateQueries({ queryKey: ['pigs-count'] })
      toast({ title: '成功', description: '豬隻資料已更新' })
      onOpenChange(false)
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '更新失敗',
        variant: 'destructive',
      })
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    updateMutation.mutate(formData)
  }

  const handleChange = (field: keyof UpdatePigRequest, value: any) => {
    setFormData((prev) => ({ ...prev, [field]: value || undefined }))
  }

  if (!open) return null

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>快速編輯豬隻資料</DialogTitle>
          <DialogDescription>編輯豬隻的基本資訊（僅可編輯標示為可編輯的欄位）</DialogDescription>
        </DialogHeader>

        {pigLoading ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
          </div>
        ) : pig ? (
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* 唯讀資訊區域 */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-slate-700 border-b pb-2">基本資訊（不可編輯）</h3>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label className="text-slate-500">系統號</Label>
                  <Input value={pig.id} disabled className="bg-slate-50" />
                </div>
                <div className="space-y-2">
                  <Label className="text-slate-500">耳號</Label>
                  <Input value={pig.ear_tag} disabled className="bg-slate-50" />
                </div>
                <div className="space-y-2">
                  <Label className="text-slate-500">品種</Label>
                  <Input value={pigBreedNames[pig.breed]} disabled className="bg-slate-50" />
                </div>
                <div className="space-y-2">
                  <Label className="text-slate-500">性別</Label>
                  <Input value={pigGenderNames[pig.gender]} disabled className="bg-slate-50" />
                </div>
                <div className="space-y-2">
                  <Label className="text-slate-500">出生日期</Label>
                  <Input
                    value={pig.birth_date ? new Date(pig.birth_date).toLocaleDateString('zh-TW') : '-'}
                    disabled
                    className="bg-slate-50"
                  />
                </div>
                <div className="space-y-2">
                  <Label className="text-slate-500">進場日期</Label>
                  <Input
                    value={new Date(pig.entry_date).toLocaleDateString('zh-TW')}
                    disabled
                    className="bg-slate-50"
                  />
                </div>
              </div>
            </div>

            {/* 可编辑字段区域 */}
            <div className="space-y-4">
              <h3 className="text-sm font-semibold text-purple-700 border-b border-purple-200 pb-2">
                可編輯欄位
              </h3>
              <div className="grid grid-cols-2 gap-4">
                {/* 進場體重 */}
                <div className="space-y-2">
                  <Label htmlFor="entry_weight">
                    進場體重 (kg) <span className="text-red-500">*</span>
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
              </div>
            </div>

            {/* 資訊提示區域（唯讀） */}
            <div className="space-y-4 pt-4 border-t">
              <h3 className="text-sm font-semibold text-slate-700 border-b pb-2">狀態資訊（僅供參考）</h3>
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label className="text-slate-500">用藥中</Label>
                  <Input
                    value="此資訊由系統自動計算，請至詳情頁查看相關記錄"
                    disabled
                    className="bg-slate-50 text-slate-500 text-xs"
                  />
                </div>
                <div className="space-y-2">
                  <Label className="text-slate-500">獸醫師建議</Label>
                  <Input
                    value="請至詳情頁查看獸醫師建議"
                    disabled
                    className="bg-slate-50 text-slate-500 text-xs"
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
