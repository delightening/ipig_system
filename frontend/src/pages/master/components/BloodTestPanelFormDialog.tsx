import { type UseFormReturn } from 'react-hook-form'
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
import { Loader2 } from 'lucide-react'
import { PanelIcon } from '@/components/ui/panel-icon'
import type { BloodTestPanelFormData } from '@/lib/validation'

interface BloodTestPanelFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  form: UseFormReturn<BloodTestPanelFormData>
  isPending: boolean
  onSubmit: (e?: React.BaseSyntheticEvent) => Promise<void>
}

export function BloodTestPanelFormDialog({
  open,
  onOpenChange,
  form,
  isPending,
  onSubmit,
}: BloodTestPanelFormDialogProps) {
  const { register, watch, formState: { errors } } = form
  const iconValue = watch('icon')

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[450px]">
        <DialogHeader>
          <DialogTitle>新增檢查分類</DialogTitle>
          <DialogDescription>
            建立新的血液檢查分類（如：CBC、肝臟、腎臟等）
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={onSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="panel_key" className="text-right">
                代碼 <span className="text-destructive">*</span>
              </Label>
              <div className="col-span-3 space-y-1">
                <Input
                  id="panel_key"
                  {...register('key', {
                    onChange: (e) => {
                      e.target.value = e.target.value.toUpperCase()
                    },
                  })}
                  className="font-mono"
                  placeholder="如: CBC、LIVER、RENAL"
                  maxLength={20}
                />
                {errors.key && (
                  <p className="text-sm text-destructive">{errors.key.message}</p>
                )}
              </div>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="panel_name" className="text-right">
                名稱 <span className="text-destructive">*</span>
              </Label>
              <div className="col-span-3 space-y-1">
                <Input
                  id="panel_name"
                  {...register('name')}
                  placeholder="如: 全血球計數、肝臟功能"
                  maxLength={100}
                />
                {errors.name && (
                  <p className="text-sm text-destructive">{errors.name.message}</p>
                )}
              </div>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="panel_icon" className="text-right">
                圖示
              </Label>
              <div className="col-span-3 flex items-center gap-2">
                <Input
                  id="panel_icon"
                  {...register('icon')}
                  placeholder="Emoji 或 SVG 路徑"
                  maxLength={200}
                />
                {iconValue && (
                  <PanelIcon icon={iconValue} size={24} />
                )}
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              建立
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
