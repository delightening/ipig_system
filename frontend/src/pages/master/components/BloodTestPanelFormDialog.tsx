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
import type { CreateBloodTestPanelRequest } from '@/lib/api'

interface BloodTestPanelFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  formData: CreateBloodTestPanelRequest
  setFormData: (data: CreateBloodTestPanelRequest) => void
  isPending: boolean
  onSubmit: (e: React.FormEvent) => void
}

export function BloodTestPanelFormDialog({
  open,
  onOpenChange,
  formData,
  setFormData,
  isPending,
  onSubmit,
}: BloodTestPanelFormDialogProps) {
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
                代碼 <span className="text-red-500">*</span>
              </Label>
              <Input
                id="panel_key"
                value={formData.key}
                onChange={(e) =>
                  setFormData({ ...formData, key: e.target.value.toUpperCase() })
                }
                className="col-span-3 font-mono"
                placeholder="如: CBC、LIVER、RENAL"
                required
                maxLength={20}
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="panel_name" className="text-right">
                名稱 <span className="text-red-500">*</span>
              </Label>
              <Input
                id="panel_name"
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
                className="col-span-3"
                placeholder="如: 全血球計數、肝臟功能"
                required
                maxLength={100}
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="panel_icon" className="text-right">
                圖示
              </Label>
              <div className="col-span-3 flex items-center gap-2">
                <Input
                  id="panel_icon"
                  value={formData.icon || ''}
                  onChange={(e) =>
                    setFormData({ ...formData, icon: e.target.value })
                  }
                  placeholder="Emoji 或 SVG 路徑"
                  maxLength={200}
                />
                {formData.icon && (
                  <PanelIcon icon={formData.icon} size={24} />
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
