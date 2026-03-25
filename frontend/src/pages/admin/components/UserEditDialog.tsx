import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Loader2 } from 'lucide-react'
import {
  editUserSchema,
  type EditUserFormData,
} from '@/lib/validation'
import type { CreateUserData } from '../hooks/useUserManagement'

interface UserEditDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  formData: CreateUserData
  setFormData: (data: CreateUserData | ((prev: CreateUserData) => CreateUserData)) => void
  isPending: boolean
  onSubmit: () => void
}

export function UserEditDialog({
  open,
  onOpenChange,
  formData,
  setFormData,
  isPending,
  onSubmit,
}: UserEditDialogProps) {
  const {
    register,
    handleSubmit,
    watch,
    setValue,
    reset,
    formState: { errors },
  } = useForm<EditUserFormData>({
    resolver: zodResolver(editUserSchema),
    defaultValues: {
      email: formData.email,
      display_name: formData.display_name,
      entry_date: formData.entry_date || '',
      trainings: formData.trainings || [],
    },
  })

  const trainings = watch('trainings') || []

  useEffect(() => {
    if (open) {
      reset({
        email: formData.email,
        display_name: formData.display_name,
        entry_date: formData.entry_date || '',
        trainings: formData.trainings || [],
      })
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, reset])

  const watchedEmail = watch('email')
  const watchedDisplayName = watch('display_name')
  const watchedEntryDate = watch('entry_date')

  useEffect(() => {
    setFormData((prev) => ({
      ...prev,
      email: watchedEmail,
      display_name: watchedDisplayName,
      entry_date: watchedEntryDate || '',
      trainings: trainings,
    }))
  }, [watchedEmail, watchedDisplayName, watchedEntryDate, trainings, setFormData])

  const toggleTraining = (code: string) => {
    const exists = trainings.some((t) => t.code === code)
    const next = exists
      ? trainings.filter((t) => t.code !== code)
      : [...trainings, { code, certificate_no: '', received_date: '' }]
    setValue('trainings', next)
  }

  const updateTrainingField = (
    idx: number,
    field: 'certificate_no' | 'received_date',
    value: string,
  ) => {
    const updated = [...trainings]
    updated[idx] = { ...updated[idx], [field]: value }
    setValue('trainings', updated)
  }

  const onValid = () => {
    onSubmit()
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>編輯使用者</DialogTitle>
          <DialogDescription>修改使用者資訊</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(onValid)} className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="edit-email">Email</Label>
            <Input id="edit-email" type="email" {...register('email')} />
            {errors.email && (
              <p className="text-sm text-destructive">{errors.email.message}</p>
            )}
          </div>
          <div className="space-y-2">
            <Label htmlFor="edit-display_name">顯示名稱</Label>
            <Input id="edit-display_name" {...register('display_name')} />
            {errors.display_name && (
              <p className="text-sm text-destructive">{errors.display_name.message}</p>
            )}
          </div>
          <div className="border-t pt-4 mt-4">
            <h4 className="font-medium mb-3">AUP 人員資料</h4>
            <div className="space-y-2 mb-4">
              <Label htmlFor="edit-entry_date">入職日期 (Entry Date)</Label>
              <Input id="edit-entry_date" type="date" {...register('entry_date')} />
            </div>
            <div className="space-y-2">
              <Label>訓練/資格 (Trainings)</Label>
              <div className="flex flex-wrap gap-2 p-3 border rounded-md">
                {['A', 'B', 'C', 'D', 'E', 'F'].map((code) => (
                  <Badge
                    key={code}
                    variant={trainings.some((t) => t.code === code) ? 'default' : 'outline'}
                    className="cursor-pointer"
                    onClick={() => toggleTraining(code)}
                  >
                    {code}
                  </Badge>
                ))}
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                A.IACUC訓練班 B.IACUC研討會 C.輻射安全 D.生醫產業研習 E.動物法規管理班 F.其他
              </p>
              {trainings.length > 0 && (
                <div className="space-y-2 mt-3">
                  {trainings.map((training, idx) => (
                    <div key={training.code} className="flex gap-2 items-center">
                      <Badge variant="secondary">{training.code}</Badge>
                      <Input
                        placeholder="證書編號"
                        value={training.certificate_no}
                        onChange={(e) => updateTrainingField(idx, 'certificate_no', e.target.value)}
                        className="w-32"
                      />
                      <Input
                        type="date"
                        value={training.received_date}
                        onChange={(e) => updateTrainingField(idx, 'received_date', e.target.value)}
                        className="w-36"
                      />
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              儲存
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
