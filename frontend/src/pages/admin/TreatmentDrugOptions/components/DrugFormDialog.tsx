import type { CreateTreatmentDrugRequest } from '@/types/treatment-drug'
import { DRUG_CATEGORIES, DOSAGE_UNITS } from '@/types/treatment-drug'
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
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'

interface DrugFormDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    title: string
    form: CreateTreatmentDrugRequest
    setForm: (fn: (prev: CreateTreatmentDrugRequest) => CreateTreatmentDrugRequest) => void
    onSubmit: () => void
    isLoading: boolean
    toggleUnit: (unit: string) => void
}

export function DrugFormDialog({
    open,
    onOpenChange,
    title,
    form,
    setForm,
    onSubmit,
    isLoading,
    toggleUnit,
}: DrugFormDialogProps) {
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-md">
                <DialogHeader>
                    <DialogTitle>{title}</DialogTitle>
                    <DialogDescription>設定藥物名稱、預設單位和分類</DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-2">
                    <div>
                        <Label>藥品名稱 *</Label>
                        <Input
                            value={form.name}
                            onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                            placeholder="例：Meloxicam"
                        />
                        <p className="text-xs text-muted-foreground mt-1">同一「藥物名稱＋分類」僅能有一筆啟用項目，重複時請改為編輯既有項目。</p>
                    </div>
                    <div>
                        <Label>顯示名稱</Label>
                        <Input
                            value={form.display_name || ''}
                            onChange={(e) => setForm((prev) => ({ ...prev, display_name: e.target.value }))}
                            placeholder="例：Meloxicam（美洛昔康）"
                        />
                    </div>
                    <div>
                        <Label>分類</Label>
                        <Select
                            value={form.category || ''}
                            onValueChange={(v) => setForm((prev) => ({ ...prev, category: v }))}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder="選擇分類" />
                            </SelectTrigger>
                            <SelectContent>
                                {DRUG_CATEGORIES.map((cat) => (
                                    <SelectItem key={cat} value={cat}>{cat}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div>
                        <Label>預設劑量單位</Label>
                        <Select
                            value={form.default_dosage_unit || ''}
                            onValueChange={(v) => setForm((prev) => ({ ...prev, default_dosage_unit: v }))}
                        >
                            <SelectTrigger>
                                <SelectValue placeholder="選擇單位" />
                            </SelectTrigger>
                            <SelectContent>
                                {DOSAGE_UNITS.map((unit) => (
                                    <SelectItem key={unit} value={unit}>{unit}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div>
                        <Label>可用單位（多選）</Label>
                        <div className="flex flex-wrap gap-2 mt-1">
                            {DOSAGE_UNITS.map((unit) => (
                                <button
                                    key={unit}
                                    type="button"
                                    onClick={() => toggleUnit(unit)}
                                    className={cn(
                                        'px-2 py-1 rounded text-xs border transition-colors',
                                        form.available_units?.includes(unit)
                                            ? 'bg-primary/10 border-primary/30 text-primary'
                                            : 'bg-background border-border text-muted-foreground hover:border-border'
                                    )}
                                >
                                    {unit}
                                </button>
                            ))}
                        </div>
                    </div>
                    <div>
                        <Label>排序（數字越小越前面）</Label>
                        <Input
                            type="number"
                            value={form.sort_order || 0}
                            onChange={(e) => setForm((prev) => ({ ...prev, sort_order: parseInt(e.target.value) || 0 }))}
                        />
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        取消
                    </Button>
                    <Button onClick={onSubmit} disabled={isLoading}>
                        {isLoading && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                        儲存
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
