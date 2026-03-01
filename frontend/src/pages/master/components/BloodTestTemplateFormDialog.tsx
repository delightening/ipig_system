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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Loader2 } from 'lucide-react'
import { PanelIcon } from '@/components/ui/panel-icon'
import type { CreateBloodTestTemplateRequest } from '@/lib/api'
import type { BloodTestTemplate, BloodTestPanel } from '@/lib/api'

interface BloodTestTemplateFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  editingTemplate: BloodTestTemplate | null
  formData: CreateBloodTestTemplateRequest
  setFormData: (data: CreateBloodTestTemplateRequest) => void
  panels: BloodTestPanel[] | undefined
  isCreatePending: boolean
  isUpdatePending: boolean
  onSubmit: (e: React.FormEvent) => void
}

export function BloodTestTemplateFormDialog({
  open,
  onOpenChange,
  editingTemplate,
  formData,
  setFormData,
  panels,
  isCreatePending,
  isUpdatePending,
  onSubmit,
}: BloodTestTemplateFormDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>
            {editingTemplate ? '編輯檢查項目' : '新增檢查項目'}
          </DialogTitle>
          <DialogDescription>
            {editingTemplate
              ? `修改 ${editingTemplate.code} 的項目資料`
              : '建立新的血液檢查項目模板'}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={onSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="code" className="text-right">
                代碼 <span className="text-red-500">*</span>
              </Label>
              <Input
                id="code"
                value={formData.code}
                onChange={(e) =>
                  setFormData({ ...formData, code: e.target.value.toUpperCase() })
                }
                className="col-span-3 font-mono"
                placeholder="如: WBC、RBC、AST"
                required
                disabled={!!editingTemplate}
                maxLength={20}
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="name" className="text-right">
                名稱 <span className="text-red-500">*</span>
              </Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="col-span-3"
                placeholder="如: WBC (白血球計數)"
                required
                maxLength={200}
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="default_unit" className="text-right">
                預設單位
              </Label>
              <Input
                id="default_unit"
                value={formData.default_unit}
                onChange={(e) =>
                  setFormData({ ...formData, default_unit: e.target.value })
                }
                className="col-span-3"
                placeholder="如: 10³/μL、mg/dL、U/L"
                maxLength={50}
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="reference_range" className="text-right">
                參考範圍
              </Label>
              <Input
                id="reference_range"
                value={formData.reference_range}
                onChange={(e) =>
                  setFormData({ ...formData, reference_range: e.target.value })
                }
                className="col-span-3"
                placeholder="如: 4.0-10.0"
                maxLength={100}
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="default_price" className="text-right">
                預設價格
              </Label>
              <Input
                id="default_price"
                type="number"
                min="0"
                step="1"
                value={formData.default_price || ''}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    default_price: e.target.value ? Number(e.target.value) : 0,
                  })
                }
                className="col-span-3"
                placeholder="0"
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="panel_id" className="text-right">
                所屬分類
              </Label>
              <Select
                value={formData.panel_id || 'none'}
                onValueChange={(val) =>
                  setFormData({
                    ...formData,
                    panel_id: val === 'none' ? undefined : val,
                  })
                }
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="選擇分類" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">未分類</SelectItem>
                  {panels?.map((panel) => (
                    <SelectItem key={panel.id} value={panel.id}>
                      <PanelIcon icon={panel.icon} /> {panel.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button
              type="submit"
              disabled={isCreatePending || isUpdatePending}
            >
              {(isCreatePending || isUpdatePending) && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              {editingTemplate ? '更新' : '建立'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
