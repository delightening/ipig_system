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
import { BloodTestItemInput, BloodTestPanel } from '@/lib/api'
import { Loader2, Plus, X } from 'lucide-react'

import { LAB_OPTIONS } from './constants'
import { BloodTestFormData } from './useBloodTestForm'

interface BloodTestFormDialogProps {
    open: boolean
    editingId: string | null
    labNameOption: string
    formData: BloodTestFormData
    isPending: boolean
    templates: { id: string; code: string; name: string; default_unit?: string; reference_range?: string }[]
    panels: BloodTestPanel[]
    panelActiveStates: Record<string, boolean>
    onLabNameOptionChange: (val: string) => void
    onFormDataChange: (updater: (prev: BloodTestFormData) => BloodTestFormData) => void
    onAddItemFromTemplate: (templateId: string) => void
    onAddCustomItem: () => void
    onTogglePanel: (panel: BloodTestPanel) => void
    onRemoveItem: (index: number) => void
    onUpdateItem: (index: number, field: keyof BloodTestItemInput, value: unknown) => void
    onSubmit: () => void
    onClose: () => void
}

export function BloodTestFormDialog({
    open,
    editingId,
    labNameOption,
    formData,
    isPending,
    templates,
    panels,
    panelActiveStates,
    onLabNameOptionChange,
    onFormDataChange,
    onAddItemFromTemplate,
    onAddCustomItem,
    onTogglePanel,
    onRemoveItem,
    onUpdateItem,
    onSubmit,
    onClose,
}: BloodTestFormDialogProps) {
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
                    <BasicInfoSection
                        formData={formData}
                        labNameOption={labNameOption}
                        onLabNameOptionChange={onLabNameOptionChange}
                        onFormDataChange={onFormDataChange}
                    />

                    <div className="space-y-2">
                        <Label>備註</Label>
                        <Input
                            value={formData.remark}
                            onChange={(e) => onFormDataChange((prev) => ({ ...prev, remark: e.target.value }))}
                            placeholder="選填"
                        />
                    </div>

                    {panels.length > 0 && (
                        <PanelSelector
                            panels={panels}
                            panelActiveStates={panelActiveStates}
                            onTogglePanel={onTogglePanel}
                        />
                    )}

                    <ItemsEditor
                        items={formData.items}
                        templates={templates}
                        onAddItemFromTemplate={onAddItemFromTemplate}
                        onAddCustomItem={onAddCustomItem}
                        onRemoveItem={onRemoveItem}
                        onUpdateItem={onUpdateItem}
                    />
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

/** 基本資訊區塊：檢查日期 + 檢驗機構 */
function BasicInfoSection({
    formData,
    labNameOption,
    onLabNameOptionChange,
    onFormDataChange,
}: Pick<BloodTestFormDialogProps, 'formData' | 'labNameOption' | 'onLabNameOptionChange' | 'onFormDataChange'>) {
    return (
        <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
                <Label>檢查日期 *</Label>
                <Input
                    type="date"
                    value={formData.test_date}
                    onChange={(e) => onFormDataChange((prev) => ({ ...prev, test_date: e.target.value }))}
                />
            </div>
            <div className="space-y-2">
                <Label>檢驗機構</Label>
                <Select
                    value={labNameOption}
                    onValueChange={(val) => {
                        onLabNameOptionChange(val)
                        if (val === '__other__') {
                            onFormDataChange((prev) => ({ ...prev, lab_name: '' }))
                        } else {
                            onFormDataChange((prev) => ({ ...prev, lab_name: val }))
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
                        onChange={(e) => onFormDataChange((prev) => ({ ...prev, lab_name: e.target.value }))}
                        placeholder="請輸入檢驗機構名稱"
                        className="mt-2"
                    />
                )}
            </div>
        </div>
    )
}

/** 組合快速選取按鈕列 */
function PanelSelector({
    panels,
    panelActiveStates,
    onTogglePanel,
}: {
    panels: BloodTestPanel[]
    panelActiveStates: Record<string, boolean>
    onTogglePanel: (panel: BloodTestPanel) => void
}) {
    return (
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
                            onClick={() => onTogglePanel(panel)}
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
    )
}

/** 檢查項目編輯器：模板選取 + 項目表格 */
function ItemsEditor({
    items,
    templates,
    onAddItemFromTemplate,
    onAddCustomItem,
    onRemoveItem,
    onUpdateItem,
}: {
    items: BloodTestItemInput[]
    templates: { id: string; code: string; name: string }[]
    onAddItemFromTemplate: (templateId: string) => void
    onAddCustomItem: () => void
    onRemoveItem: (index: number) => void
    onUpdateItem: (index: number, field: keyof BloodTestItemInput, value: unknown) => void
}) {
    return (
        <div className="space-y-3">
            <div className="flex items-center justify-between">
                <Label className="text-base font-semibold">
                    檢查項目
                    {items.length > 0 && (
                        <span className="ml-2 text-sm font-normal text-muted-foreground">
                            已選 {items.length} 項
                        </span>
                    )}
                </Label>
                <div className="flex gap-2">
                    <Select onValueChange={onAddItemFromTemplate}>
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
                    <Button variant="outline" size="sm" onClick={onAddCustomItem}>
                        <Plus className="h-4 w-4 mr-1" />
                        自訂項目
                    </Button>
                </div>
            </div>

            {items.length === 0 ? (
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
                            {items.map((item, index) => (
                                <TableRow key={index}>
                                    <TableCell>
                                        <Input
                                            value={item.item_name}
                                            onChange={(e) => onUpdateItem(index, 'item_name', e.target.value)}
                                            placeholder="項目名稱"
                                            className="h-8"
                                            readOnly={!!item.template_id}
                                        />
                                    </TableCell>
                                    <TableCell>
                                        <Input
                                            value={item.result_value || ''}
                                            onChange={(e) => onUpdateItem(index, 'result_value', e.target.value)}
                                            placeholder="結果"
                                            className="h-8"
                                        />
                                    </TableCell>
                                    <TableCell>
                                        <Input
                                            value={item.result_unit || ''}
                                            onChange={(e) => onUpdateItem(index, 'result_unit', e.target.value)}
                                            placeholder="單位"
                                            className="h-8"
                                        />
                                    </TableCell>
                                    <TableCell>
                                        <Input
                                            value={item.reference_range || ''}
                                            onChange={(e) => onUpdateItem(index, 'reference_range', e.target.value)}
                                            placeholder="參考範圍"
                                            className="h-8"
                                        />
                                    </TableCell>
                                    <TableCell className="text-center">
                                        <input
                                            type="checkbox"
                                            checked={item.is_abnormal}
                                            onChange={(e) => onUpdateItem(index, 'is_abnormal', e.target.checked)}
                                            className="h-4 w-4 rounded border-gray-300 text-red-600 focus:ring-red-500"
                                            aria-label={`項目 ${index + 1} 異常`}
                                        />
                                    </TableCell>
                                    <TableCell>
                                        <Input
                                            value={item.remark || ''}
                                            onChange={(e) => onUpdateItem(index, 'remark', e.target.value)}
                                            placeholder="備註"
                                            className="h-8"
                                        />
                                    </TableCell>
                                    <TableCell>
                                        <Button variant="ghost" size="icon" className="h-8 w-8 text-red-500" onClick={() => onRemoveItem(index)} aria-label="移除">
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
    )
}
