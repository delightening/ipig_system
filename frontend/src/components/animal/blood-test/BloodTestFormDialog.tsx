/**
 * 血液檢查新增/編輯 Dialog
 */
import { useMemo } from 'react'
import type { BloodTestItemInput, BloodTestPanel } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { PanelIcon } from '@/components/ui/panel-icon'
import { Loader2, Plus, X } from 'lucide-react'
import { toast } from '@/components/ui/use-toast'
import { LAB_OPTIONS } from './constants'

export interface BloodTestFormData {
  test_date: string
  lab_name: string
  remark: string
  items: BloodTestItemInput[]
}

interface BloodTestTemplate {
  id: string
  code: string
  name: string
  default_unit?: string
  reference_range?: string
}

interface BloodTestFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  editingId: string | null
  formData: BloodTestFormData
  setFormData: React.Dispatch<React.SetStateAction<BloodTestFormData>>
  labNameOption: string
  setLabNameOption: (v: string) => void
  templates: BloodTestTemplate[]
  panels: BloodTestPanel[]
  isPending: boolean
  onSubmit: () => void
  onClose: () => void
}

export function BloodTestFormDialog({
  open,
  onOpenChange,
  editingId,
  formData,
  setFormData,
  labNameOption,
  setLabNameOption,
  templates,
  panels,
  isPending,
  onSubmit,
  onClose,
}: BloodTestFormDialogProps) {
  // 計算每個 panel 是否完全被選取
  const panelActiveStates = useMemo(() => {
    const itemTemplateIds = new Set(formData.items.map(i => i.template_id).filter(Boolean))
    return panels.reduce((acc, panel) => {
      if (panel.items.length === 0) {
        acc[panel.id] = false
      } else {
        acc[panel.id] = panel.items.every(t => itemTemplateIds.has(t.id))
      }
      return acc
    }, {} as Record<string, boolean>)
  }, [panels, formData.items])

  const addItemFromTemplate = (templateId: string) => {
    const template = templates.find((t) => t.id === templateId)
    if (!template) return

    if (formData.items.some((item) => item.template_id === templateId)) {
      toast({ title: '提示', description: '該項目已在清單中' })
      return
    }

    setFormData((prev) => ({
      ...prev,
      items: [
        ...prev.items,
        {
          template_id: templateId,
          item_name: template.name,
          result_value: '',
          result_unit: template.default_unit || '',
          reference_range: template.reference_range || '',
          is_abnormal: false,
          remark: '',
          sort_order: prev.items.length,
        },
      ],
    }))
  }

  const addCustomItem = () => {
    setFormData((prev) => ({
      ...prev,
      items: [
        ...prev.items,
        {
          item_name: '',
          result_value: '',
          result_unit: '',
          reference_range: '',
          is_abnormal: false,
          remark: '',
          sort_order: prev.items.length,
        },
      ],
    }))
  }

  const togglePanel = (panel: BloodTestPanel) => {
    const isActive = panelActiveStates[panel.id]
    if (isActive) {
      const panelTemplateIds = new Set(panel.items.map(t => t.id))
      setFormData((prev) => ({
        ...prev,
        items: prev.items.filter(item => !item.template_id || !panelTemplateIds.has(item.template_id)),
      }))
    } else {
      const existingIds = new Set(formData.items.map(i => i.template_id).filter(Boolean))
      const newItems = panel.items
        .filter(t => !existingIds.has(t.id))
        .map((t, idx) => ({
          template_id: t.id,
          item_name: t.name,
          result_value: '',
          result_unit: t.default_unit || '',
          reference_range: t.reference_range || '',
          is_abnormal: false,
          remark: '',
          sort_order: formData.items.length + idx,
        }))
      if (newItems.length > 0) {
        setFormData((prev) => ({
          ...prev,
          items: [...prev.items, ...newItems],
        }))
        toast({ title: '已加入', description: `${panel.name}：新增 ${newItems.length} 項` })
      } else {
        toast({ title: '提示', description: '所有項目已在清單中' })
      }
    }
  }

  const removeItem = (index: number) => {
    setFormData((prev) => ({
      ...prev,
      items: prev.items.filter((_, i) => i !== index),
    }))
  }

  const updateItem = (index: number, field: keyof BloodTestItemInput, value: unknown) => {
    setFormData((prev) => ({
      ...prev,
      items: prev.items.map((item, i) =>
        i === index ? { ...item, [field]: value } : item
      ),
    }))
  }

  return (
    <Dialog open={open} onOpenChange={(o) => { if (!o) onClose() }}>
      <DialogContent className="max-w-4xl">
        <DialogHeader>
          <DialogTitle>
            {editingId ? '編輯血液檢查' : '新增血液檢查'}
          </DialogTitle>
          <DialogDescription>
            填寫檢查基本資訊，並從模板選取或自訂檢查項目
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          {/* 基本資訊 */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>檢查日期 *</Label>
              <Input
                type="date"
                value={formData.test_date}
                onChange={(e) => setFormData((prev) => ({ ...prev, test_date: e.target.value }))}
              />
            </div>
            <div className="space-y-2">
              <Label>檢驗機構</Label>
              <Select
                value={labNameOption}
                onValueChange={(val) => {
                  setLabNameOption(val)
                  if (val === '__other__') {
                    setFormData((prev) => ({ ...prev, lab_name: '' }))
                  } else {
                    setFormData((prev) => ({ ...prev, lab_name: val }))
                  }
                }}
              >
                <SelectTrigger>
                  <SelectValue placeholder="請選擇檢驗機構" />
                </SelectTrigger>
                <SelectContent>
                  {LAB_OPTIONS.map((lab) => (
                    <SelectItem key={lab} value={lab}>{lab}</SelectItem>
                  ))}
                  <SelectItem value="__other__">其他</SelectItem>
                </SelectContent>
              </Select>
              {labNameOption === '__other__' && (
                <Input
                  value={formData.lab_name}
                  onChange={(e) => setFormData((prev) => ({ ...prev, lab_name: e.target.value }))}
                  placeholder="請輸入檢驗機構名稱"
                  className="mt-2"
                />
              )}
            </div>
          </div>

          <div className="space-y-2">
            <Label>備註</Label>
            <Input
              value={formData.remark}
              onChange={(e) => setFormData((prev) => ({ ...prev, remark: e.target.value }))}
              placeholder="選填"
            />
          </div>

          {/* 組合快速選取 */}
          {panels.length > 0 && (
            <div className="space-y-2">
              <Label className="text-sm text-muted-foreground">快速選取組合</Label>
              <div className="flex flex-wrap gap-2">
                {panels.map((panel) => {
                  const isActive = panelActiveStates[panel.id]
                  return (
                    <Button
                      key={panel.id}
                      type="button"
                      variant={isActive ? 'default' : 'outline'}
                      size="sm"
                      className={`transition-all ${isActive
                        ? 'bg-blue-600 hover:bg-blue-700 text-white shadow-sm'
                        : 'hover:bg-blue-50 hover:border-blue-300'
                      }`}
                      onClick={() => togglePanel(panel)}
                    >
                      <PanelIcon icon={panel.icon} className="mr-1" />
                      {panel.name}
                      {isActive && (
                        <span className="ml-1 text-xs">✓</span>
                      )}
                    </Button>
                  )
                })}
              </div>
            </div>
          )}

          {/* 檢查項目 */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <Label className="text-base font-semibold">
                檢查項目
                {formData.items.length > 0 && (
                  <span className="ml-2 text-sm font-normal text-muted-foreground">
                    已選 {formData.items.length} 項
                  </span>
                )}
              </Label>
              <div className="flex gap-2">
                <Select onValueChange={addItemFromTemplate}>
                  <SelectTrigger className="w-[200px]">
                    <SelectValue placeholder="從模板新增..." />
                  </SelectTrigger>
                  <SelectContent>
                    {templates.map((t) => (
                      <SelectItem key={t.id} value={t.id}>
                        {t.code} - {t.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Button variant="outline" size="sm" onClick={addCustomItem}>
                  <Plus className="h-4 w-4 mr-1" />
                  自訂項目
                </Button>
              </div>
            </div>

            {formData.items.length === 0 ? (
              <div className="border rounded-lg p-6 text-center text-gray-500">
                <p>尚無檢查項目</p>
                <p className="text-sm mt-1">從上方模板選取或新增自訂項目</p>
              </div>
            ) : (
              <div className="border rounded-lg overflow-hidden">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[180px]">項目名稱</TableHead>
                      <TableHead className="w-[120px]">結果值</TableHead>
                      <TableHead className="w-[80px]">單位</TableHead>
                      <TableHead className="w-[120px]">參考範圍</TableHead>
                      <TableHead className="w-[80px] text-center">異常</TableHead>
                      <TableHead>備註</TableHead>
                      <TableHead className="w-[50px]"></TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {formData.items.map((item, index) => (
                      <TableRow key={index}>
                        <TableCell>
                          <Input
                            value={item.item_name}
                            onChange={(e) => updateItem(index, 'item_name', e.target.value)}
                            placeholder="項目名稱"
                            className="h-8"
                            readOnly={!!item.template_id}
                          />
                        </TableCell>
                        <TableCell>
                          <Input
                            value={item.result_value || ''}
                            onChange={(e) => updateItem(index, 'result_value', e.target.value)}
                            placeholder="結果"
                            className="h-8"
                          />
                        </TableCell>
                        <TableCell>
                          <Input
                            value={item.result_unit || ''}
                            onChange={(e) => updateItem(index, 'result_unit', e.target.value)}
                            placeholder="單位"
                            className="h-8"
                          />
                        </TableCell>
                        <TableCell>
                          <Input
                            value={item.reference_range || ''}
                            onChange={(e) => updateItem(index, 'reference_range', e.target.value)}
                            placeholder="參考範圍"
                            className="h-8"
                          />
                        </TableCell>
                        <TableCell className="text-center">
                          <input
                            type="checkbox"
                            checked={item.is_abnormal}
                            onChange={(e) => updateItem(index, 'is_abnormal', e.target.checked)}
                            className="h-4 w-4 rounded border-gray-300 text-red-600 focus:ring-red-500"
                            aria-label={`項目 ${index + 1} 異常`}
                          />
                        </TableCell>
                        <TableCell>
                          <Input
                            value={item.remark || ''}
                            onChange={(e) => updateItem(index, 'remark', e.target.value)}
                            placeholder="備註"
                            className="h-8"
                          />
                        </TableCell>
                        <TableCell>
                          <Button variant="ghost" size="icon" className="h-8 w-8 text-red-500" onClick={() => removeItem(index)} aria-label="移除">
                            <X className="h-4 w-4" />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            取消
          </Button>
          <Button onClick={onSubmit} disabled={isPending}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {editingId ? '更新' : '建立'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
