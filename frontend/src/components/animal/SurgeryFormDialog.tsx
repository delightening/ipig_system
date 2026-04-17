import { useEffect } from 'react'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Loader2, FastForward } from 'lucide-react'
import { useSurgeryForm } from './useSurgeryForm'
import { SurgeryBasicInfoSection } from './SurgeryBasicInfoSection'
import { SurgeryAnesthesiaSection } from './SurgeryAnesthesiaSection'
import { SurgeryProcedureSection } from './SurgeryProcedureSection'
import type { AnimalSurgery } from '@/lib/api'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  animalId: string
  earTag: string
  surgery?: AnimalSurgery
}

export function SurgeryFormDialog({
  open,
  onOpenChange,
  animalId,
  earTag,
  surgery,
}: Props) {
  const isEdit = !!surgery
  const { formData, setFormData, mutation, jumpToNextEmptyField } = useSurgeryForm({
    open,
    onOpenChange,
    animalId,
    surgery,
  })

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.altKey && e.key === 'n') {
        e.preventDefault()
        jumpToNextEmptyField()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [jumpToNextEmptyField])

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!formData.surgery_site.trim()) {
      toast({ title: '錯誤', description: '請填寫手術部位', variant: 'destructive' })
      return
    }
    mutation.mutate(formData)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:w-[90vw] sm:max-w-[1200px]">
        <DialogHeader>
          <div className="flex items-center justify-between">
            <div>
              <DialogTitle>
                {isEdit ? '編輯手術紀錄' : '新增手術紀錄'}
              </DialogTitle>
              <DialogDescription>耳號：{earTag}</DialogDescription>
            </div>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={jumpToNextEmptyField}
              className="flex items-center gap-2 border-status-purple-border text-status-purple-text hover:bg-status-purple-bg mr-4"
              title="快捷鍵: Alt + N"
            >
              <FastForward className="h-4 w-4" />
              下一個空白欄位
            </Button>
          </div>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          <SurgeryBasicInfoSection formData={formData} onChange={setFormData} />
          <SurgeryAnesthesiaSection formData={formData} onChange={setFormData} />
          <SurgeryProcedureSection formData={formData} onChange={setFormData} surgeryId={surgery?.id != null ? String(surgery.id) : undefined} />

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button
              type="submit"
              disabled={mutation.isPending}
              className="bg-status-success-solid hover:bg-green-700"
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
