// 獸醫巡場報告 Dialog
// 4 類別 × 3 欄（觀察/建議/追蹤改善），豬隻狀況可選動物

import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { format } from 'date-fns'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/input'
import { DatePicker } from '@/components/ui/date-picker'
import { SearchableSelect } from '@/components/ui/searchable-select'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import {
    Loader2,
    Plus,
    X,
    Save,
    Stethoscope,
    ChevronDown,
    ChevronRight,
    Trash2,
    FileDown,
} from 'lucide-react'

// ── 型別 ──────────────────────────────

interface AnimalOption {
    id: string
    ear_tag: string
    pen_location?: string
}

interface EntryRow {
    category: string
    animal_id: string
    observation: string
    suggestion: string
    follow_up: string
}

interface PatrolReport {
    id: string
    patrol_date: string
    week_start: string | null
    week_end: string | null
    status: string
    entries: EntryWithAnimal[]
}

interface EntryWithAnimal {
    id: string
    category: string
    animal_id: string | null
    ear_tag: string | null
    observation: string
    suggestion: string
    follow_up: string
    sort_order: number
}

// ── 常數 ──────────────────────────────

const CATEGORIES = [
    { key: 'pig_condition', label: '豬隻狀況', hasAnimal: true },
    { key: 'epidemic_prevention', label: '防疫與消毒劑化', hasAnimal: false },
    { key: 'case_record', label: '病例紀錄', hasAnimal: true },
    { key: 'other', label: '其他', hasAnimal: false },
] as const

type CategoryKey = typeof CATEGORIES[number]['key']

const emptyEntry = (category: CategoryKey): EntryRow => ({
    category,
    animal_id: '',
    observation: '',
    suggestion: '',
    follow_up: '',
})

// ── Props ──────────────────────────────

interface VetPatrolReportDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    editReportId?: string | null
}

export function VetPatrolReportDialog({ open, onOpenChange, editReportId }: VetPatrolReportDialogProps) {
    const queryClient = useQueryClient()
    const today = format(new Date(), 'yyyy-MM-dd')

    const [patrolDate, setPatrolDate] = useState(today)
    const [weekStart, setWeekStart] = useState('')
    const [weekEnd, setWeekEnd] = useState('')
    const [entries, setEntries] = useState<Record<CategoryKey, EntryRow[]>>({
        pig_condition: [emptyEntry('pig_condition')],
        epidemic_prevention: [emptyEntry('epidemic_prevention')],
        case_record: [emptyEntry('case_record')],
        other: [emptyEntry('other')],
    })
    const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(new Set())
    const [savedReportId, setSavedReportId] = useState<string | null>(editReportId ?? null)

    // 取得動物列表（耳號選擇用）
    const { data: animals } = useQuery({
        queryKey: ['animals-for-patrol'],
        queryFn: async () => {
            const res = await api.get<{ data: AnimalOption[] }>('/animals?per_page=9999&status=pen,in_experiment')
            return res.data.data
        },
        staleTime: 60_000,
        enabled: open,
    })

    const animalOptions = (animals ?? []).map(a => ({
        value: a.id,
        label: a.ear_tag,
        description: a.pen_location ?? '',
    }))

    // 載入既有報告
    const { data: existingReport } = useQuery({
        queryKey: ['vet-patrol-report', editReportId],
        queryFn: async () => {
            const res = await api.get<PatrolReport>(`/vet-patrol-reports/${editReportId}`)
            return res.data
        },
        enabled: !!editReportId && open,
    })

    useEffect(() => {
        if (existingReport) {
            setPatrolDate(existingReport.patrol_date)
            setWeekStart(existingReport.week_start ?? '')
            setWeekEnd(existingReport.week_end ?? '')

            const grouped: Record<CategoryKey, EntryRow[]> = {
                pig_condition: [],
                epidemic_prevention: [],
                case_record: [],
                other: [],
            }
            for (const e of existingReport.entries) {
                const cat = e.category as CategoryKey
                if (grouped[cat]) {
                    grouped[cat].push({
                        category: cat,
                        animal_id: e.animal_id ?? '',
                        observation: e.observation,
                        suggestion: e.suggestion,
                        follow_up: e.follow_up,
                    })
                }
            }
            // 確保每個類別至少一行
            for (const cat of CATEGORIES) {
                if (grouped[cat.key].length === 0) {
                    grouped[cat.key] = [emptyEntry(cat.key)]
                }
            }
            setEntries(grouped)
        }
    }, [existingReport])

    // Reset on open
    useEffect(() => {
        if (open && !editReportId) {
            setPatrolDate(today)
            setWeekStart('')
            setWeekEnd('')
            setEntries({
                pig_condition: [emptyEntry('pig_condition')],
                epidemic_prevention: [emptyEntry('epidemic_prevention')],
                case_record: [emptyEntry('case_record')],
                other: [emptyEntry('other')],
            })
            setCollapsedCategories(new Set())
            setSavedReportId(null)
        } else if (open && editReportId) {
            setSavedReportId(editReportId)
        }
    }, [open, editReportId, today])

    const saveMutation = useMutation({
        mutationFn: async () => {
            const allEntries = Object.values(entries).flat()
                .filter(e => e.observation || e.suggestion || e.follow_up || e.animal_id)
                .map((e, i) => ({
                    category: e.category,
                    animal_id: e.animal_id || null,
                    observation: e.observation,
                    suggestion: e.suggestion,
                    follow_up: e.follow_up,
                    sort_order: i,
                }))

            const payload = {
                patrol_date: patrolDate,
                week_start: weekStart || null,
                week_end: weekEnd || null,
                entries: allEntries,
            }

            if (savedReportId) {
                const res = await api.put<{ id: string }>(`/vet-patrol-reports/${savedReportId}`, payload)
                return res.data
            }
            const res = await api.post<{ id: string }>('/vet-patrol-reports', payload)
            return res.data
        },
        onSuccess: (data) => {
            setSavedReportId(data.id)
            queryClient.invalidateQueries({ queryKey: ['vet-patrol-reports'] })
            queryClient.invalidateQueries({ queryKey: ['animal-vet-advice-records'] })
            toast({ title: '成功', description: '獸醫巡場報告已儲存，相關動物建議已同步' })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '儲存失敗'), variant: 'destructive' })
        },
    })

    const [isExporting, setIsExporting] = useState(false)
    const handleExportPdf = async () => {
        if (!savedReportId) return
        setIsExporting(true)
        try {
            const res = await api.post(`/vet-patrol-reports/${savedReportId}/export-pdf`, {}, {
                responseType: 'blob',
                _silentError: true,
            } as never)
            const blob = new Blob([res.data], { type: 'application/pdf' })
            const url = window.URL.createObjectURL(blob)
            const a = document.createElement('a')
            a.href = url
            a.download = `試驗豬場巡場報告_${patrolDate.replace(/-/g, '')}.pdf`
            document.body.appendChild(a)
            a.click()
            window.URL.revokeObjectURL(url)
            document.body.removeChild(a)
        } catch (error) {
            toast({ title: '錯誤', description: getApiErrorMessage(error, 'PDF 匯出失敗'), variant: 'destructive' })
        } finally {
            setIsExporting(false)
        }
    }

    const addRow = (cat: CategoryKey) => {
        setEntries(prev => ({
            ...prev,
            [cat]: [...prev[cat], emptyEntry(cat)],
        }))
    }

    const removeRow = (cat: CategoryKey, idx: number) => {
        setEntries(prev => ({
            ...prev,
            [cat]: prev[cat].length > 1
                ? prev[cat].filter((_, i) => i !== idx)
                : [emptyEntry(cat)],
        }))
    }

    const updateRow = (cat: CategoryKey, idx: number, field: keyof EntryRow, value: string) => {
        setEntries(prev => ({
            ...prev,
            [cat]: prev[cat].map((row, i) => i === idx ? { ...row, [field]: value } : row),
        }))
    }

    const toggleCategory = (cat: string) => {
        setCollapsedCategories(prev => {
            const next = new Set(prev)
            if (next.has(cat)) next.delete(cat)
            else next.add(cat)
            return next
        })
    }

    const isSaving = saveMutation.isPending

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-[900px] max-h-[90vh] overflow-y-auto">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-status-success-solid">
                        <Stethoscope className="h-5 w-5" />
                        獸醫巡場報告
                    </DialogTitle>
                </DialogHeader>

                {/* 報告基本資訊 */}
                <div className="grid grid-cols-3 gap-3 mb-4">
                    <div>
                        <label className="text-sm font-medium mb-1 block">巡場日期</label>
                        <DatePicker value={patrolDate} onChange={setPatrolDate} required />
                    </div>
                    <div>
                        <label className="text-sm font-medium mb-1 block">週起</label>
                        <DatePicker value={weekStart} onChange={setWeekStart} placeholder="選擇週起日" />
                    </div>
                    <div>
                        <label className="text-sm font-medium mb-1 block">週迄</label>
                        <DatePicker value={weekEnd} onChange={setWeekEnd} placeholder="選擇週迄日" />
                    </div>
                </div>

                {/* 4 類別 */}
                <div className="space-y-3">
                    {CATEGORIES.map((cat) => {
                        const isCollapsed = collapsedCategories.has(cat.key)
                        const rows = entries[cat.key]
                        const filledCount = rows.filter(r => r.observation || r.suggestion || r.follow_up).length

                        return (
                            <div key={cat.key} className="border rounded-lg overflow-hidden">
                                {/* 類別 Header */}
                                <button
                                    type="button"
                                    onClick={() => toggleCategory(cat.key)}
                                    className="w-full flex items-center justify-between px-3 py-2 bg-muted/50 hover:bg-muted transition-colors text-left"
                                >
                                    <div className="flex items-center gap-2">
                                        {isCollapsed
                                            ? <ChevronRight className="h-4 w-4 text-muted-foreground" />
                                            : <ChevronDown className="h-4 w-4 text-muted-foreground" />
                                        }
                                        <span className="text-sm font-semibold">{cat.label}</span>
                                        {filledCount > 0 && (
                                            <span className="text-xs text-muted-foreground">({filledCount} 筆)</span>
                                        )}
                                    </div>
                                    <button
                                        type="button"
                                        onClick={(e) => { e.stopPropagation(); addRow(cat.key) }}
                                        className="flex items-center gap-1 text-xs text-status-success-solid hover:text-green-700 transition-colors"
                                    >
                                        <Plus className="h-3 w-3" /> 新增列
                                    </button>
                                </button>

                                {/* 條目列 */}
                                {!isCollapsed && (
                                    <div className="divide-y">
                                        {/* Column headers */}
                                        <div className={`grid ${cat.hasAnimal ? 'grid-cols-[140px_1fr_1fr_1fr_32px]' : 'grid-cols-[1fr_1fr_1fr_32px]'} gap-1 px-2 py-1 bg-muted/30 text-xs text-muted-foreground`}>
                                            {cat.hasAnimal && <span className="px-1">動物</span>}
                                            <span className="px-1">觀察內容</span>
                                            <span className="px-1">建議</span>
                                            <span className="px-1">追蹤改善</span>
                                            <span />
                                        </div>

                                        {rows.map((row, idx) => (
                                            <div
                                                key={idx}
                                                className={`grid ${cat.hasAnimal ? 'grid-cols-[140px_1fr_1fr_1fr_32px]' : 'grid-cols-[1fr_1fr_1fr_32px]'} gap-1 px-2 py-1.5 items-start`}
                                            >
                                                {cat.hasAnimal && (
                                                    <SearchableSelect
                                                        options={animalOptions}
                                                        value={row.animal_id}
                                                        onValueChange={(v) => updateRow(cat.key, idx, 'animal_id', v)}
                                                        placeholder="選擇動物"
                                                        searchPlaceholder="搜尋耳號..."
                                                        className="text-xs"
                                                    />
                                                )}
                                                <Textarea
                                                    value={row.observation}
                                                    onChange={(e) => updateRow(cat.key, idx, 'observation', e.target.value)}
                                                    placeholder="觀察內容..."
                                                    className="min-h-[32px] text-sm resize-none py-1.5 px-2"
                                                    rows={1}
                                                />
                                                <Textarea
                                                    value={row.suggestion}
                                                    onChange={(e) => updateRow(cat.key, idx, 'suggestion', e.target.value)}
                                                    placeholder="建議..."
                                                    className="min-h-[32px] text-sm resize-none py-1.5 px-2"
                                                    rows={1}
                                                />
                                                <Textarea
                                                    value={row.follow_up}
                                                    onChange={(e) => updateRow(cat.key, idx, 'follow_up', e.target.value)}
                                                    placeholder="追蹤改善..."
                                                    className="min-h-[32px] text-sm resize-none py-1.5 px-2"
                                                    rows={1}
                                                />
                                                <button
                                                    type="button"
                                                    onClick={() => removeRow(cat.key, idx)}
                                                    className="p-1 rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors mt-1"
                                                >
                                                    <Trash2 className="h-3.5 w-3.5" />
                                                </button>
                                            </div>
                                        ))}
                                    </div>
                                )}
                            </div>
                        )
                    })}
                </div>

                {/* 儲存 + 匯出 */}
                <div className="flex justify-between items-center mt-4 pt-3 border-t">
                    <p className="text-xs text-muted-foreground">
                        儲存後，豬隻狀況/病例紀錄會自動同步至對應動物的「獸醫師建議」
                    </p>
                    <div className="flex gap-2">
                        <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isSaving}>
                            取消
                        </Button>
                        {savedReportId && (
                            <Button
                                variant="outline"
                                onClick={handleExportPdf}
                                disabled={isExporting}
                            >
                                {isExporting ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <FileDown className="h-4 w-4 mr-1" />}
                                下載 PDF
                            </Button>
                        )}
                        <Button
                            onClick={() => saveMutation.mutate()}
                            disabled={isSaving || !patrolDate}
                            className="bg-status-success-solid hover:bg-green-700"
                        >
                            {isSaving ? <Loader2 className="h-4 w-4 mr-1 animate-spin" /> : <Save className="h-4 w-4 mr-1" />}
                            {savedReportId ? '更新報告' : '儲存報告'}
                        </Button>
                    </div>
                </div>
            </DialogContent>
        </Dialog>
    )
}
