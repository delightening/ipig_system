/**
 * 血液檢查新增/編輯 Dialog
 */
import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import type { BloodTestItemInput, BloodTestPanel } from '@/lib/api'
import type { AnimalBloodTestItem } from '@/types'
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
import { History, Loader2, Pencil, Plus, X } from 'lucide-react'
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
  /** R30-16: 編輯模式時的 current items（含 id），用於修正按鈕 */
  editingItems?: AnimalBloodTestItem[]
  /** R30-16: 點 item 上的「修正」按鈕 */
  onCorrectItem?: (item: AnimalBloodTestItem) => void
  /** R30-16: 顯示完整修正歷史 */
  onShowHistory?: () => void
}

export function BloodTestFormDialog({
  open,
  onOpenChange: _onOpenChange,
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
  editingItems,
  onCorrectItem,
  onShowHistory,
}: BloodTestFormDialogProps) {
  const { t } = useTranslation()
  const isEditMode = !!editingId
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
      toast({ title: t('animalRecords.bloodTest.notice'), description: t('animalRecords.bloodTest.itemAlreadyAdded') })
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
        toast({ title: t('animalRecords.bloodTest.addedTitle'), description: t('animalRecords.bloodTest.panelItemsAdded', { panel: panel.name, count: newItems.length }) })
      } else {
        toast({ title: t('animalRecords.bloodTest.notice'), description: t('animalRecords.bloodTest.allItemsAlreadyAdded') })
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
      <DialogContent size="xl">
        <DialogHeader>
          <DialogTitle>
            {editingId ? t('animalRecords.bloodTest.editTitle') : t('animalRecords.bloodTest.add')}
          </DialogTitle>
          <DialogDescription>
            {t('animalRecords.bloodTest.formDescription')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          {/* 基本資訊 */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>{t('animalRecords.bloodTest.testDateRequired')}</Label>
              <Input
                type="date"
                value={formData.test_date}
                onChange={(e) => setFormData((prev) => ({ ...prev, test_date: e.target.value }))}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('animalRecords.bloodTest.labName')}</Label>
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
                  <SelectValue placeholder={t('animalRecords.bloodTest.selectLab')} />
                </SelectTrigger>
                <SelectContent>
                  {LAB_OPTIONS.map((lab) => (
                    <SelectItem key={lab} value={lab}>{lab}</SelectItem>
                  ))}
                  <SelectItem value="__other__">{t('animalRecords.bloodTest.labOther')}</SelectItem>
                </SelectContent>
              </Select>
              {labNameOption === '__other__' && (
                <Input
                  value={formData.lab_name}
                  onChange={(e) => setFormData((prev) => ({ ...prev, lab_name: e.target.value }))}
                  placeholder={t('animalRecords.bloodTest.enterLabName')}
                  className="mt-2"
                />
              )}
            </div>
          </div>

          <div className="space-y-2">
            <Label>{t('animalRecords.shared.remark')}</Label>
            <Input
              value={formData.remark}
              onChange={(e) => setFormData((prev) => ({ ...prev, remark: e.target.value }))}
              placeholder={t('animalRecords.bloodTest.optional')}
            />
          </div>

          {/* 組合快速選取（僅新增模式） */}
          {!isEditMode && panels.length > 0 && (
            <div className="space-y-2">
              <Label className="text-sm text-muted-foreground">{t('animalRecords.bloodTest.quickSelectPanels')}</Label>
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
                        ? 'bg-primary hover:bg-primary/90 text-white shadow-xs'
                        : 'hover:bg-status-info-bg hover:border-primary'
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
                {t('animalRecords.bloodTest.items')}
                {formData.items.length > 0 && (
                  <span className="ml-2 text-sm font-normal text-muted-foreground">
                    {isEditMode ? t('animalRecords.bloodTest.itemsTotal', { count: formData.items.length }) : t('animalRecords.bloodTest.itemsSelected', { count: formData.items.length })}
                  </span>
                )}
              </Label>
              {isEditMode ? (
                <Button variant="outline" size="sm" onClick={onShowHistory}>
                  <History className="h-4 w-4 mr-1" />
                  {t('animalRecords.bloodTest.correctionHistory')}
                </Button>
              ) : (
                <div className="flex gap-2">
                  <Select onValueChange={addItemFromTemplate}>
                    <SelectTrigger className="w-[200px]">
                      <SelectValue placeholder={t('animalRecords.bloodTest.addFromTemplate')} />
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
                    {t('animalRecords.bloodTest.customItem')}
                  </Button>
                </div>
              )}
            </div>

            {isEditMode && (
              <div className="rounded-md border border-status-info-text/40 bg-status-info-bg/40 p-2 text-xs text-status-info-text">
                {t('animalRecords.bloodTest.glpNotice')}
              </div>
            )}

            {formData.items.length === 0 ? (
              <div className="border rounded-lg p-6 text-center text-muted-foreground">
                <p>{t('animalRecords.bloodTest.noItems')}</p>
                <p className="text-sm mt-1">{t('animalRecords.bloodTest.noItemsHint')}</p>
              </div>
            ) : (
              <div className="border rounded-lg overflow-hidden @container">
                <div className="hidden @[700px]:block">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="w-[180px]">{t('animalRecords.bloodTest.itemName')}</TableHead>
                        <TableHead className="w-[120px]">{t('animalRecords.bloodTest.resultValue')}</TableHead>
                        <TableHead className="w-[80px]">{t('animalRecords.shared.unit')}</TableHead>
                        <TableHead className="w-[120px]">{t('animalRecords.bloodTest.referenceRange')}</TableHead>
                        <TableHead className="w-[80px] text-center">{t('animalRecords.bloodTest.abnormal')}</TableHead>
                        <TableHead>{t('animalRecords.shared.remark')}</TableHead>
                        <TableHead className="w-[50px]"></TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {formData.items.map((item, index) => {
                        const editingItem = isEditMode && editingItems
                          ? editingItems[index]
                          : null
                        return (
                          <TableRow key={editingItem?.id || item.template_id || `item-${item.sort_order}`}>
                            <TableCell>
                              <Input value={item.item_name} onChange={(e) => updateItem(index, 'item_name', e.target.value)} placeholder={t('animalRecords.bloodTest.itemName')} className="h-8" readOnly={isEditMode || !!item.template_id} />
                            </TableCell>
                            <TableCell>
                              <Input value={item.result_value || ''} onChange={(e) => updateItem(index, 'result_value', e.target.value)} placeholder={t('animalRecords.bloodTest.resultPlaceholder')} className="h-8" readOnly={isEditMode} />
                            </TableCell>
                            <TableCell>
                              <Input value={item.result_unit || ''} onChange={(e) => updateItem(index, 'result_unit', e.target.value)} placeholder={t('animalRecords.shared.unit')} className="h-8" readOnly={isEditMode} />
                            </TableCell>
                            <TableCell>
                              <Input value={item.reference_range || ''} onChange={(e) => updateItem(index, 'reference_range', e.target.value)} placeholder={t('animalRecords.bloodTest.referenceRange')} className="h-8" readOnly={isEditMode} />
                            </TableCell>
                            <TableCell className="text-center">
                              <input type="checkbox" checked={item.is_abnormal} onChange={(e) => updateItem(index, 'is_abnormal', e.target.checked)} className="h-4 w-4 rounded border-border text-status-error-text focus:ring-destructive" aria-label={t('animalRecords.bloodTest.itemAbnormalAria', { index: index + 1 })} disabled={isEditMode} />
                            </TableCell>
                            <TableCell>
                              <Input value={item.remark || ''} onChange={(e) => updateItem(index, 'remark', e.target.value)} placeholder={t('animalRecords.shared.remark')} className="h-8" readOnly={isEditMode} />
                            </TableCell>
                            <TableCell>
                              {isEditMode && editingItem ? (
                                <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => onCorrectItem?.(editingItem)} aria-label={t('animalRecords.bloodTest.correct')}>
                                  <Pencil className="h-4 w-4" />
                                </Button>
                              ) : (
                                <Button variant="ghost" size="icon" className="h-8 w-8 text-status-error-solid" onClick={() => removeItem(index)} aria-label={t('animalRecords.bloodTest.remove')}>
                                  <X className="h-4 w-4" />
                                </Button>
                              )}
                            </TableCell>
                          </TableRow>
                        )
                      })}
                    </TableBody>
                  </Table>
                </div>

                <div className="@[700px]:hidden divide-y">
                  {formData.items.map((item, index) => {
                    const editingItem = isEditMode && editingItems
                      ? editingItems[index]
                      : null
                    return (
                      <div key={editingItem?.id || item.template_id || `item-${item.sort_order}`} className="p-3 space-y-2">
                        <div className="flex items-start justify-between gap-2">
                          <Input value={item.item_name} onChange={(e) => updateItem(index, 'item_name', e.target.value)} placeholder={t('animalRecords.bloodTest.itemName')} className="h-8 font-medium" readOnly={isEditMode || !!item.template_id} />
                          {isEditMode && editingItem ? (
                            <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0" onClick={() => onCorrectItem?.(editingItem)} aria-label={t('animalRecords.bloodTest.correct')}>
                              <Pencil className="h-4 w-4" />
                            </Button>
                          ) : (
                            <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0 text-status-error-solid" onClick={() => removeItem(index)} aria-label={t('animalRecords.bloodTest.remove')}>
                              <X className="h-4 w-4" />
                            </Button>
                          )}
                        </div>
                        <div className="grid grid-cols-2 gap-2">
                          <div>
                            <Label className="text-xs text-muted-foreground">{t('animalRecords.bloodTest.resultValue')}</Label>
                            <Input value={item.result_value || ''} onChange={(e) => updateItem(index, 'result_value', e.target.value)} placeholder={t('animalRecords.bloodTest.resultPlaceholder')} className="h-8" readOnly={isEditMode} />
                          </div>
                          <div>
                            <Label className="text-xs text-muted-foreground">{t('animalRecords.shared.unit')}</Label>
                            <Input value={item.result_unit || ''} onChange={(e) => updateItem(index, 'result_unit', e.target.value)} placeholder={t('animalRecords.shared.unit')} className="h-8" readOnly={isEditMode} />
                          </div>
                        </div>
                        <div>
                          <Label className="text-xs text-muted-foreground">{t('animalRecords.bloodTest.referenceRange')}</Label>
                          <Input value={item.reference_range || ''} onChange={(e) => updateItem(index, 'reference_range', e.target.value)} placeholder={t('animalRecords.bloodTest.referenceRange')} className="h-8" readOnly={isEditMode} />
                        </div>
                        <div>
                          <Label className="text-xs text-muted-foreground">{t('animalRecords.shared.remark')}</Label>
                          <Input value={item.remark || ''} onChange={(e) => updateItem(index, 'remark', e.target.value)} placeholder={t('animalRecords.shared.remark')} className="h-8" readOnly={isEditMode} />
                        </div>
                        <label className="flex items-center gap-2 text-sm">
                          <input type="checkbox" checked={item.is_abnormal} onChange={(e) => updateItem(index, 'is_abnormal', e.target.checked)} className="h-4 w-4 rounded border-border text-status-error-text focus:ring-destructive" disabled={isEditMode} />
                          {t('animalRecords.bloodTest.markAbnormal')}
                        </label>
                      </div>
                    )
                  })}
                </div>
              </div>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            {t('common.cancel')}
          </Button>
          <Button onClick={onSubmit} disabled={isPending}>
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {editingId ? t('common.update') : t('animalRecords.bloodTest.createButton')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
