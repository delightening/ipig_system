import { useTranslation } from 'react-i18next'
import type { CreateAnimalRequest } from '@/lib/api'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Loader2, AlertTriangle } from 'lucide-react'

export interface DuplicateWarningData {
  earTag: string
  existingAnimals: Array<{ id: string; birth_date: string | null; status: string; pen_location: string | null }>
  source: 'create' | 'quickAdd'
  pendingPayload: CreateAnimalRequest & { breed_other?: string }
}

interface DuplicateWarningDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  data: DuplicateWarningData | null
  onConfirm: (payload: CreateAnimalRequest & { breed_other?: string }) => void
  isPending: boolean
}

export function DuplicateWarningDialog({
  open,
  onOpenChange,
  data,
  onConfirm,
  isPending,
}: DuplicateWarningDialogProps) {
  const { t } = useTranslation()
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-status-warning-text">
            <AlertTriangle className="h-5 w-5" />
            耳號重複警告
          </DialogTitle>
          <DialogDescription>
            耳號 <span className="font-semibold text-foreground">{data?.earTag}</span> 已存在以下存活動物：
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-2 my-2">
          {data?.existingAnimals.map((animal, idx) => (
            <div key={idx} className="flex items-center gap-3 p-3 bg-status-warning-bg border border-status-warning-border rounded-lg text-sm">
              <AlertTriangle className="h-4 w-4 text-status-warning-text shrink-0" />
              <div>
                <div>出生日期: <span className="font-medium">{animal.birth_date || '未設定'}</span></div>
                <div>欄位: <span className="font-medium">{animal.pen_location || '-'}</span></div>
              </div>
            </div>
          ))}
        </div>
        <p className="text-sm text-muted-foreground">
          確定仍要以<span className="font-semibold">不同出生日期</span>建立新動物嗎？
        </p>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>{t('common.cancel')}</Button>
          <Button
            onClick={() => data?.pendingPayload && onConfirm(data.pendingPayload)}
            disabled={isPending}
            className="bg-status-warning-bg text-status-warning-text border border-status-warning-text/30 hover:bg-status-warning-bg/80"
          >
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            確認建立
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
