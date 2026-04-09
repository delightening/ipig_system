// 手術紀錄內嵌 — 疼痛評估可收合區塊
// 術後給藥使用自由格式 Repeater + DrugCombobox

import { useState, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Repeater } from '@/components/ui/repeater'
import { DrugCombobox } from '@/components/animal/DrugCombobox'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { Plus, Trash2, ChevronDown, ChevronUp } from 'lucide-react'
import {
    INCISION_OPTIONS,
    ATTITUDE_OPTIONS,
    APPETITE_OPTIONS,
    FECES_OPTIONS,
    URINE_OPTIONS,
    PAIN_SCORE_OPTIONS,
    calcTotal,
    getPainGrade,
    type PainAssessmentEntry,
    type AssessmentOption,
    type MedicationItem,
    emptyPainEntry,
} from './painAssessmentConstants'

interface CareRecordFromAPI {
    id: string
    post_op_days: number | null
    time_period: string | null
    incision: number | null
    attitude_behavior: number | null
    appetite: number | null
    feces: number | null
    urine: number | null
    pain_score: number | null
    post_medications: MedicationItem[] | null
}

interface Props {
    surgeryId?: string
    entries: PainAssessmentEntry[]
    onChange: (entries: PainAssessmentEntry[]) => void
}

export function SurgeryPainSection({ surgeryId, entries, onChange }: Props) {
    const [isOpen, setIsOpen] = useState(false)
    const [showAddForm, setShowAddForm] = useState(false)
    const [addForm, setAddForm] = useState<PainAssessmentEntry>({ ...emptyPainEntry })

    // 編輯模式：載入此手術的既有疼痛評估
    const { data: existingRecords } = useQuery({
        queryKey: ['surgery-care-records', surgeryId],
        queryFn: async () => {
            const res = await api.get<CareRecordFromAPI[]>(`/surgeries/${surgeryId}/care-records`)
            return res.data
        },
        enabled: !!surgeryId,
        staleTime: 30_000,
    })

    useEffect(() => {
        if (existingRecords && existingRecords.length > 0 && entries.length === 0) {
            const loaded: PainAssessmentEntry[] = existingRecords.map((r) => ({
                id: r.id,
                post_op_days: r.post_op_days != null ? String(r.post_op_days) : '',
                time_period: r.time_period || 'AM',
                incision: r.incision != null ? String(r.incision) : '',
                attitude_behavior: r.attitude_behavior != null ? String(r.attitude_behavior) : '',
                appetite: r.appetite != null ? String(r.appetite) : '',
                feces: r.feces != null ? String(r.feces) : '',
                urine: r.urine != null ? String(r.urine) : '',
                pain_score: r.pain_score != null ? String(r.pain_score) : '',
                post_medications: r.post_medications || [],
            }))
            onChange(loaded)
            setIsOpen(true)
        }
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [existingRecords])

    const totalCount = entries.length
    const hasNewEntries = entries.some((e) => !e.id)

    const handleAddEntry = () => {
        onChange([...entries, { ...addForm }])
        setAddForm({ ...emptyPainEntry })
        setShowAddForm(false)
    }

    const handleRemoveEntry = (index: number) => {
        onChange(entries.filter((_, i) => i !== index))
    }

    const addFormTotal = calcTotal(
        addForm.incision, addForm.attitude_behavior, addForm.appetite,
        addForm.feces, addForm.urine, addForm.pain_score,
    )
    const addFormGrade = getPainGrade(addFormTotal)

    return (
        <div className="border rounded-lg">
            <button
                type="button"
                className="flex items-center justify-between w-full px-4 py-3 text-left font-medium bg-muted hover:bg-muted rounded-t-lg"
                onClick={() => setIsOpen(!isOpen)}
            >
                <span className="flex items-center gap-2">
                    疼痛評估
                    {totalCount > 0 && (
                        <Badge variant="secondary" className="text-xs">{totalCount} 筆</Badge>
                    )}
                    {hasNewEntries && (
                        <Badge variant="outline" className="text-xs text-orange-600 border-orange-300">待儲存</Badge>
                    )}
                </span>
                {isOpen ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
            </button>

            {isOpen && (
                <div className="px-4 py-3 space-y-3">
                    {entries.length > 0 && (
                        <div className="space-y-2">
                            {entries.map((entry, idx) => {
                                const total = calcTotal(
                                    entry.incision, entry.attitude_behavior, entry.appetite,
                                    entry.feces, entry.urine, entry.pain_score,
                                )
                                const grade = getPainGrade(total)
                                const meds = entry.post_medications
                                    .map((m) => m.name + (m.dose ? ` ${m.dose}${m.dosage_unit || ''}` : ''))
                                    .join('、')

                                return (
                                    <div key={entry.id || `new-${idx}`}
                                        className="flex items-center justify-between px-3 py-2 bg-muted/50 rounded-md text-sm"
                                    >
                                        <div className="flex items-center gap-3 flex-wrap">
                                            <span className="font-medium">
                                                D{entry.post_op_days || '?'}-{entry.time_period}
                                            </span>
                                            {total !== null && (
                                                <span>總分: <strong>{total}</strong></span>
                                            )}
                                            {grade && (
                                                <Badge variant={grade.variant} className="text-xs">
                                                    {grade.label.split('（')[0]}
                                                </Badge>
                                            )}
                                            {meds && (
                                                <span className="text-xs text-muted-foreground">{meds}</span>
                                            )}
                                            {!entry.id && (
                                                <Badge variant="outline" className="text-xs text-orange-600 border-orange-300">新增</Badge>
                                            )}
                                        </div>
                                        {!entry.id && (
                                            <Button
                                                type="button"
                                                variant="ghost"
                                                size="icon"
                                                className="h-7 w-7 text-destructive"
                                                onClick={() => handleRemoveEntry(idx)}
                                            >
                                                <Trash2 className="h-3.5 w-3.5" />
                                            </Button>
                                        )}
                                    </div>
                                )
                            })}
                        </div>
                    )}

                    {showAddForm ? (
                        <div className="border rounded-md p-4 space-y-4 bg-background">
                            <div className="flex items-center justify-between">
                                <Label className="text-sm font-semibold">新增疼痛評估</Label>
                                <Button type="button" variant="ghost" size="sm"
                                    onClick={() => { setShowAddForm(false); setAddForm({ ...emptyPainEntry }) }}>
                                    取消
                                </Button>
                            </div>

                            <div className="grid grid-cols-2 gap-3">
                                <div className="space-y-1">
                                    <Label className="text-xs">術後天數</Label>
                                    <Input type="number" min={0} value={addForm.post_op_days}
                                        onChange={(e) => setAddForm({ ...addForm, post_op_days: e.target.value })}
                                        placeholder="D1, D2..." className="h-8" />
                                </div>
                                <div className="space-y-1">
                                    <Label className="text-xs">時段</Label>
                                    <Select value={addForm.time_period}
                                        onValueChange={(v) => setAddForm({ ...addForm, time_period: v })}>
                                        <SelectTrigger className="h-8"><SelectValue /></SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="AM">上午 (AM)</SelectItem>
                                            <SelectItem value="PM">下午 (PM)</SelectItem>
                                        </SelectContent>
                                    </Select>
                                </div>
                            </div>

                            <div className="space-y-2">
                                <CompactAssessmentRow label="傷口狀況" options={INCISION_OPTIONS}
                                    value={addForm.incision} onChange={(v) => setAddForm({ ...addForm, incision: v })} />
                                <CompactAssessmentRow label="態度/行為" options={ATTITUDE_OPTIONS}
                                    value={addForm.attitude_behavior} onChange={(v) => setAddForm({ ...addForm, attitude_behavior: v })} />
                                <CompactAssessmentRow label="食慾" options={APPETITE_OPTIONS}
                                    value={addForm.appetite} onChange={(v) => setAddForm({ ...addForm, appetite: v })} />
                                <CompactAssessmentRow label="排便" options={FECES_OPTIONS}
                                    value={addForm.feces} onChange={(v) => setAddForm({ ...addForm, feces: v })} />
                                <CompactAssessmentRow label="排尿" options={URINE_OPTIONS}
                                    value={addForm.urine} onChange={(v) => setAddForm({ ...addForm, urine: v })} />
                                <CompactAssessmentRow label="疼痛分數" options={PAIN_SCORE_OPTIONS}
                                    value={addForm.pain_score} onChange={(v) => setAddForm({ ...addForm, pain_score: v })} />
                            </div>

                            {addFormTotal !== null && addFormGrade && (
                                <div className="flex items-center gap-4 rounded-md bg-muted/40 px-3 py-2 text-sm">
                                    <span>總分: <strong className="text-lg">{addFormTotal}</strong></span>
                                    <Badge variant={addFormGrade.variant}>
                                        第{addFormGrade.grade}級：{addFormGrade.label.split('（')[0]}
                                    </Badge>
                                    <span className="text-xs text-muted-foreground">{addFormGrade.advice}</span>
                                </div>
                            )}

                            <div className="space-y-1">
                                <Label className="text-xs font-medium">術後給藥</Label>
                                <Repeater<MedicationItem>
                                    value={addForm.post_medications}
                                    onChange={(post_medications) => setAddForm({ ...addForm, post_medications })}
                                    defaultItem={() => ({ name: '', dose: '', drug_option_id: undefined, dosage_unit: '' })}
                                    addLabel="新增藥物"
                                    renderItem={(item, _index, onItemChange) => (
                                        <DrugCombobox
                                            value={{
                                                drug_option_id: item.drug_option_id,
                                                drug_name: item.name,
                                                dosage_value: item.dose,
                                                dosage_unit: item.dosage_unit || '',
                                            }}
                                            onChange={(sel) =>
                                                onItemChange({
                                                    ...item,
                                                    name: sel.drug_name,
                                                    dose: sel.dosage_value,
                                                    drug_option_id: sel.drug_option_id,
                                                    dosage_unit: sel.dosage_unit,
                                                })
                                            }
                                        />
                                    )}
                                />
                                <p className="text-xs text-muted-foreground pt-1">
                                    Ketorolac (IM)：體重 &gt;50kg 給 60mg，≤50kg 給 30mg
                                </p>
                                <p className="text-xs text-muted-foreground">
                                    Meloxicam：0.1–0.4 mg/kg (SID)
                                </p>
                            </div>

                            <Button type="button" size="sm" onClick={handleAddEntry}
                                className="w-full bg-purple-600 hover:bg-purple-700">
                                確認新增此筆評估
                            </Button>
                        </div>
                    ) : (
                        <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            className="w-full"
                            onClick={() => setShowAddForm(true)}
                        >
                            <Plus className="h-4 w-4 mr-1" />
                            新增疼痛評估
                        </Button>
                    )}
                </div>
            )}
        </div>
    )
}

function CompactAssessmentRow({ label, options, value, onChange }: {
    label: string
    options: AssessmentOption[]
    value: string
    onChange: (v: string) => void
}) {
    return (
        <div className="grid grid-cols-[90px_1fr] items-center gap-2">
            <Label className="text-xs text-right">{label}</Label>
            <Select value={value} onValueChange={onChange}>
                <SelectTrigger className="h-8 text-xs">
                    <SelectValue placeholder="選擇..." />
                </SelectTrigger>
                <SelectContent>
                    {options.map((opt) => (
                        <SelectItem key={opt.score} value={String(opt.score)}>
                            <span className="inline-flex items-center gap-1.5">
                                <span className="inline-flex items-center justify-center w-4 h-4 rounded-full bg-muted text-[10px] font-bold shrink-0">
                                    {opt.score}
                                </span>
                                <span className="text-xs">{opt.label}</span>
                            </span>
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </div>
    )
}
