import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { Plus, Trash2 } from 'lucide-react'

/** 補登歷史變更的委員審查列。系統內委員選 reviewer_id；院外委員填 reviewer_name。 */
export interface ReviewerRow {
    reviewer_id: string | null
    reviewer_name: string
    decision: 'APPROVE' | 'REJECT' | 'REVISION'
    comment: string
}

// NOTE: 與 pages/protocols/components/import-review/{users,ReviewerSelect} 概念重複；
// ≥2 處使用後可抽至共用 components/protocol/reviewer/（follow-up，見 TODO R64-5）。
interface UserOption {
    id: string
    display_name?: string
    username?: string
    email?: string
    roles?: string[]
}
const OTHER = '__OTHER__'
const userLabel = (u: UserOption) => u.display_name || u.username || u.email || u.id

const DECISION_OPTIONS: { value: ReviewerRow['decision']; label: string }[] = [
    { value: 'APPROVE', label: '同意' },
    { value: 'REVISION', label: '修正後同意' },
    { value: 'REJECT', label: '不同意' },
]

interface Props {
    reviewers: ReviewerRow[]
    onChange: (rows: ReviewerRow[]) => void
}

export function HistoricalReviewersEditor({ reviewers, onChange }: Props) {
    const { data: users = [], isLoading } = useQuery({
        queryKey: ['users', 'reviewer-options'],
        queryFn: async () => (await api.get<UserOption[]>('/users')).data,
    })
    // 補登歷史委員可為任何系統使用者（API 僅需 reviewer_id 或 reviewer_name），不限 REVIEWER 角色
    const reviewerOptions = users

    const update = (idx: number, patch: Partial<ReviewerRow>) =>
        onChange(reviewers.map((r, i) => (i === idx ? { ...r, ...patch } : r)))
    const remove = (idx: number) => onChange(reviewers.filter((_, i) => i !== idx))
    const add = () =>
        onChange([...reviewers, { reviewer_id: null, reviewer_name: '', decision: 'APPROVE', comment: '' }])

    const onSelect = (idx: number, v: string) =>
        v === OTHER
            ? update(idx, { reviewer_id: null })
            : update(idx, { reviewer_id: v, reviewer_name: '' })

    return (
        <div className="space-y-3">
            <Label>委員審查意見（系統內委員下拉選，或選「其他」填院外姓名）</Label>
            {reviewers.map((r, idx) => (
                <div key={idx} className="rounded-md border p-3 space-y-2">
                    <div className="flex flex-wrap items-center gap-2">
                        <Select
                            value={r.reviewer_id ?? OTHER}
                            onValueChange={(v) => onSelect(idx, v)}
                            disabled={isLoading}
                        >
                            <SelectTrigger className="w-44">
                                <SelectValue placeholder={isLoading ? '載入中…' : '選擇委員'} />
                            </SelectTrigger>
                            <SelectContent>
                                {reviewerOptions.map((u) => (
                                    <SelectItem key={u.id} value={u.id}>{userLabel(u)}</SelectItem>
                                ))}
                                <SelectItem value={OTHER}>其他（院外，填姓名）</SelectItem>
                            </SelectContent>
                        </Select>
                        {r.reviewer_id === null && (
                            <Input
                                className="w-40"
                                value={r.reviewer_name}
                                onChange={(e) => update(idx, { reviewer_name: e.target.value })}
                                placeholder="院外委員姓名"
                            />
                        )}
                        <div className="flex gap-1">
                            {DECISION_OPTIONS.map((opt) => (
                                <Button
                                    key={opt.value}
                                    type="button"
                                    size="sm"
                                    variant={r.decision === opt.value ? 'default' : 'outline'}
                                    onClick={() => update(idx, { decision: opt.value })}
                                >
                                    {opt.label}
                                </Button>
                            ))}
                        </div>
                        <Button type="button" size="icon" variant="ghost" onClick={() => remove(idx)}>
                            <Trash2 className="h-4 w-4" />
                        </Button>
                    </div>
                    <Textarea
                        value={r.comment}
                        onChange={(e) => update(idx, { comment: e.target.value })}
                        placeholder="審查意見（選填）"
                        rows={2}
                    />
                </div>
            ))}
            <Button type="button" variant="outline" size="sm" onClick={add}>
                <Plus className="mr-1 h-4 w-4" />
                新增委員
            </Button>
        </div>
    )
}
