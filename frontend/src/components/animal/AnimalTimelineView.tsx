import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Animal, AnimalObservation, AnimalSurgery, AnimalSacrifice, AnimalSuddenDeath, AnimalWeight, AnimalTransfer, AnimalEvent, RecordType, recordTypeNames, transferStatusNames, transferTypeNames } from '@/lib/api'
import { uiLocale } from '@/lib/utils'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
    ClipboardList, Scissors, CheckCircle2, Eye, Edit2, Trash2, Calendar,
    Skull, Zap, ArrowRightLeft,
} from 'lucide-react'

import { AnimalWeightTrendCard } from './AnimalWeightTrendCard'

interface Props {
    observations: AnimalObservation[]
    surgeries: AnimalSurgery[]
    animalWeights?: AnimalWeight[]
    sacrifice?: AnimalSacrifice
    suddenDeath?: AnimalSuddenDeath
    transfers?: AnimalTransfer[]
    iacucEvents?: AnimalEvent[]
    animal?: Animal
    onView: (type: 'observation' | 'surgery', id: number) => void
    onEdit: (type: 'observation' | 'surgery', record: AnimalObservation | AnimalSurgery) => void
    onCopy: (type: 'observation' | 'surgery', id: number) => void
    onHistory: (type: 'observation' | 'surgery', id: number) => void
    onVet: (type: 'observation' | 'surgery', id: number) => void
    onDelete: (type: 'observation' | 'surgery', id: number) => void
}

type TimelineItemType = 'observation' | 'surgery' | 'created' | 'sacrificed' | 'completed' | 'euthanized' | 'sudden_death' | 'transferred' | 'iacuc_change'

interface TimelineItem {
    id: string
    originalId: number | string
    type: TimelineItemType
    date: Date
    title: string
    content: string
    actor?: string | null
    vetRead?: boolean
    raw?: unknown
}

/** 里程碑型別：以較大軸點 + 粗體標題強調（生命週期 / 臨床重大事件）。 */
const MILESTONE_TYPES = new Set<TimelineItemType>([
    'created', 'surgery', 'sacrificed', 'euthanized', 'sudden_death', 'completed', 'transferred',
])
/** 危急（結局）型別：沿用原紅色語義。 */
const CRITICAL_TYPES = new Set<TimelineItemType>(['sacrificed', 'euthanized', 'sudden_death'])

type FilterKey = 'all' | 'observation' | 'surgery' | 'weight' | 'death' | 'other'
const FILTER_TYPES: Record<Exclude<FilterKey, 'all' | 'weight'>, TimelineItemType[]> = {
    observation: ['observation'],
    surgery: ['surgery'],
    death: ['sacrificed', 'euthanized', 'sudden_death'],
    other: ['created', 'completed', 'transferred', 'iacuc_change'],
}
const FILTER_LABELS: { key: FilterKey; label: string }[] = [
    { key: 'all', label: '全部' },
    { key: 'observation', label: '觀察' },
    { key: 'surgery', label: '手術' },
    { key: 'weight', label: '體重' },
    { key: 'death', label: '犧牲/死亡' },
    { key: 'other', label: '其他事件' },
]

const ICONS: Record<TimelineItemType, typeof ClipboardList> = {
    observation: ClipboardList, surgery: Scissors, created: Calendar,
    sacrificed: Skull, completed: CheckCircle2, euthanized: Skull,
    sudden_death: Zap, transferred: ArrowRightLeft, iacuc_change: Edit2,
}
/** 軸點底色 — 沿用原每型別配色。 */
const DOT_COLOR: Record<TimelineItemType, string> = {
    observation: 'bg-muted text-muted-foreground',
    surgery: 'bg-status-warning-bg text-status-warning-text',
    created: 'bg-status-success-bg text-status-success-text',
    sacrificed: 'bg-status-error-bg text-status-error-text',
    completed: 'bg-status-success-bg text-status-success-text',
    euthanized: 'bg-status-error-bg text-status-error-text',
    sudden_death: 'bg-status-error-bg text-status-error-text',
    transferred: 'bg-status-info-bg text-status-info-text',
    iacuc_change: 'bg-status-warning-bg text-status-warning-text',
}
/** 時間 badge 色 — 沿用原配色（觀察為紫）。 */
const TIME_COLOR: Record<TimelineItemType, string> = {
    observation: 'text-status-purple-text bg-status-purple-bg',
    surgery: 'text-status-warning-text bg-status-warning-bg',
    created: 'text-status-success-text bg-status-success-bg',
    sacrificed: 'text-status-error-text bg-status-error-bg',
    completed: 'text-status-success-text bg-status-success-bg',
    euthanized: 'text-status-error-text bg-status-error-bg',
    sudden_death: 'text-status-error-text bg-status-error-bg',
    transferred: 'text-status-info-text bg-status-info-bg',
    iacuc_change: 'text-status-warning-text bg-status-warning-bg',
}

/** 取得該時刻在 Asia/Taipei 的當日 YYYY-MM-DD（與分組/標籤同時區，避免跨時區差 1 天）。 */
function taipeiYmd(d: Date): string {
    return d.toLocaleDateString('en-CA', { timeZone: 'Asia/Taipei' })
}

function relativeDay(d: Date): string {
    const today = Date.parse(`${taipeiYmd(new Date())}T00:00:00Z`)
    const day = Date.parse(`${taipeiYmd(d)}T00:00:00Z`)
    const diff = Math.round((today - day) / 86_400_000)
    if (diff <= 0) return '今天'
    if (diff === 1) return '昨天'
    if (diff < 7) return `${diff} 天前`
    if (diff < 30) return `${Math.floor(diff / 7)} 週前`
    if (diff < 365) return `${Math.floor(diff / 30)} 個月前`
    return `${Math.floor(diff / 365)} 年前`
}

export function AnimalTimelineView({
    observations, surgeries, animalWeights, sacrifice, suddenDeath, transfers, iacucEvents, animal,
    onView, onEdit, onDelete,
}: Props) {
    const [filter, setFilter] = useState<FilterKey>('all')
    // 訂閱語系：切換 zh/en 時重算日期標頭格式（locale 進 groups deps）。
    const { i18n } = useTranslation()
    const locale = i18n.language || 'zh-TW'

    const items = useMemo<TimelineItem[]>(() => [
        ...observations.map((obs) => ({
            id: `obs-${obs.id}`, originalId: obs.id, type: 'observation' as const,
            date: new Date(obs.event_date), title: recordTypeNames[obs.record_type as RecordType],
            content: obs.content, actor: obs.created_by_name, vetRead: obs.vet_read, raw: obs,
        })),
        ...surgeries.map((surg) => ({
            id: `surg-${surg.id}`, originalId: surg.id, type: 'surgery' as const,
            date: new Date(surg.surgery_date), title: surg.is_first_experiment ? '首次手術' : '手術紀錄',
            content: surg.surgery_site, actor: surg.created_by_name, vetRead: surg.vet_read, raw: surg,
        })),
        ...(animal ? [{
            id: 'animal-created', originalId: 0, type: 'created' as const,
            date: new Date(animal.entry_date || animal.created_at), title: '資料建立 · 進場',
            content: `耳號 ${animal.ear_tag} 進場${animal.entry_weight ? `，進場體重 ${animal.entry_weight} kg` : ''}`,
            actor: null, raw: animal,
        }] : []),
        // 安樂死動物的犧牲/採樣併入下方「已安樂死」單一事件，故此處排除 euthanized
        // （避免同時出現「犧牲/採樣」＋「已安樂死」兩筆重複）。非安樂死的犧牲（如計畫內
        // 犧牲、status=completed）仍獨立顯示。
        ...(sacrifice && sacrifice.sacrifice_date && animal?.status !== 'euthanized' ? [{
            id: 'sacrifice-record', originalId: sacrifice.id, type: 'sacrificed' as const,
            date: new Date(sacrifice.sacrifice_date), title: '犧牲/採樣',
            content: [
                sacrifice.method_electrocution ? '電擊' : null,
                sacrifice.method_bloodletting ? '放血' : null,
                sacrifice.method_other ? sacrifice.method_other : null,
                sacrifice.blood_volume_ml ? `採血 ${sacrifice.blood_volume_ml} ml` : null,
                (sacrifice.sampling || sacrifice.sampling_other)
                    ? `採樣：${[sacrifice.sampling, sacrifice.sampling_other].filter(Boolean).join('、')}` : null,
            ].filter(Boolean).join('、') || '已犧牲',
            actor: sacrifice.created_by_name || null, raw: sacrifice,
        }] : []),
        ...(animal && animal.status === 'completed' && !(sacrifice && sacrifice.sacrifice_date) ? [{
            id: 'experiment-completed', originalId: 0, type: 'completed' as const,
            date: new Date(animal.updated_at), title: '實驗完成',
            content: `耳號 ${animal.ear_tag} 實驗已完成`, actor: null, raw: animal,
        }] : []),
        // 已安樂死＝安樂死與其犧牲/採樣的單一合併事件（含電擊/放血/採血/採樣明細）。
        ...(animal && animal.status === 'euthanized' ? [{
            id: 'euthanized-event', originalId: 0, type: 'euthanized' as const,
            date: new Date(sacrifice?.sacrifice_date || animal.updated_at), title: '已安樂死',
            content: [
                sacrifice?.method_electrocution ? '電擊' : null,
                sacrifice?.method_bloodletting ? '放血' : null,
                sacrifice?.method_other ? sacrifice.method_other : null,
                sacrifice?.blood_volume_ml ? `採血 ${sacrifice.blood_volume_ml} ml` : null,
                (sacrifice?.sampling || sacrifice?.sampling_other)
                    ? `採樣：${[sacrifice?.sampling, sacrifice?.sampling_other].filter(Boolean).join('、')}` : null,
            ].filter(Boolean).join('、') || `耳號 ${animal.ear_tag} 已安樂死`,
            actor: sacrifice?.created_by_name || null, raw: sacrifice || animal,
        }] : []),
        ...(suddenDeath ? [{
            id: 'sudden-death-event', originalId: 0, type: 'sudden_death' as const,
            date: new Date(suddenDeath.discovered_at), title: '猝死',
            content: [
                suddenDeath.probable_cause ? `可能原因：${suddenDeath.probable_cause}` : null,
                suddenDeath.location ? `地點：${suddenDeath.location}` : null,
                suddenDeath.requires_pathology ? '需要病理檢查' : null,
            ].filter(Boolean).join('、') || '動物猝死',
            actor: null, raw: suddenDeath,
        }] : []),
        ...(transfers || []).map((t) => ({
            id: `transfer-${t.id}`, originalId: 0, type: 'transferred' as const,
            date: new Date(t.completed_at || t.created_at),
            title: t.status === 'completed' ? '轉讓完成' : t.status === 'rejected' ? '轉讓拒絕' : '轉讓進行中',
            content: `${t.from_iacuc_no} → ${t.to_iacuc_no || '待定'} · ${transferTypeNames[t.transfer_type === 'external' ? 'external' : 'internal']} (${transferStatusNames[t.status]})`,
            actor: null, raw: t,
        })),
        ...(iacucEvents || []).map((evt) => ({
            id: `iacuc-${evt.id}`, originalId: 0, type: 'iacuc_change' as const,
            date: new Date(evt.created_at), title: 'IACUC No. 變更',
            content: `${evt.before_data?.iacuc_no || '（無）'} → ${evt.after_data?.iacuc_no || '（無）'}`,
            actor: evt.actor_name || null, raw: evt,
        })),
    ].sort((a, b) => b.date.getTime() - a.date.getTime()), [observations, surgeries, animal, sacrifice, suddenDeath, transfers, iacucEvents])

    const rows = useMemo(() => {
        if (filter === 'all') return items
        if (filter === 'weight') return []
        const allowed = FILTER_TYPES[filter]
        return items.filter((i) => allowed.includes(i.type))
    }, [items, filter])

    // 依日期分組（items 已新→舊排序，Map 保留插入序）
    const groups = useMemo(() => {
        const map = new Map<string, TimelineItem[]>()
        for (const it of rows) {
            const key = it.date.toLocaleDateString(locale, { timeZone: 'Asia/Taipei' })
            if (!map.has(key)) map.set(key, [])
            map.get(key)!.push(it)
        }
        return [...map.entries()]
    }, [rows, locale])

    const showTrend = (filter === 'all' || filter === 'weight') && !!animalWeights?.length
    const isEmpty = !showTrend && rows.length === 0

    return (
        <div className="space-y-4">
            {/* 型別篩選 */}
            <div className="flex flex-wrap gap-2">
                {FILTER_LABELS.map(({ key, label }) => (
                    <button
                        key={key}
                        type="button"
                        onClick={() => setFilter(key)}
                        className={`rounded-full border px-3 py-1 text-xs transition-colors ${filter === key
                            ? 'border-primary bg-primary/10 font-medium text-primary'
                            : 'border-border bg-card text-muted-foreground hover:bg-muted'}`}
                    >
                        {label}
                    </button>
                ))}
            </div>

            {isEmpty ? (
                <Card className="border-dashed">
                    <CardContent className="py-12 text-center text-muted-foreground">
                        <ClipboardList className="mx-auto mb-4 h-12 w-12" />
                        <p>此篩選下尚無紀錄</p>
                    </CardContent>
                </Card>
            ) : (
                <div className="relative">
                    {/* 左側時間軸線 */}
                    <div className="absolute left-[15px] top-2 bottom-2 w-0.5 bg-border" aria-hidden />
                    <div className="space-y-3">
                        {showTrend && <AnimalWeightTrendCard weights={animalWeights!} />}
                        {groups.map(([day, dayItems]) => (
                            <div key={day} className="space-y-3">
                                {/* 日期分組標頭（吸頂） */}
                                <div className="sticky top-0 z-10 flex items-center gap-2 bg-background py-1">
                                    <span className="text-sm font-bold text-foreground">{day}</span>
                                    <span className="text-xs text-muted-foreground">{relativeDay(dayItems[0].date)}</span>
                                    <span className="h-px flex-1 bg-border" />
                                </div>
                                {dayItems.map((item) => {
                                    const Icon = ICONS[item.type]
                                    const isMilestone = MILESTONE_TYPES.has(item.type)
                                    const isCritical = CRITICAL_TYPES.has(item.type)
                                    const editable = item.type === 'observation' || item.type === 'surgery'
                                    return (
                                        <div key={item.id} className="relative pl-12">
                                            {/* 軸點：里程碑較大 */}
                                            <div className={`absolute flex items-center justify-center rounded-full border-2 border-card shadow-[0_0_0_1px_hsl(var(--border))] ${DOT_COLOR[item.type]} ${isMilestone ? 'left-0 top-0 h-8 w-8' : 'left-1 top-0 h-6 w-6'}`}>
                                                <Icon className={isMilestone ? 'h-4 w-4' : 'h-3 w-3'} />
                                            </div>
                                            <div className={`rounded-lg border bg-card p-3 shadow-xs transition-shadow hover:shadow-md ${isCritical ? 'border-status-error-border' : 'border-border'}`}>
                                                <div className="flex items-baseline gap-2">
                                                    <span className={isMilestone ? 'text-[15px] font-bold text-foreground' : 'text-sm font-semibold text-foreground'}>{item.title}</span>
                                                    <time className={`ml-auto rounded-full px-2 py-0.5 text-xs font-medium ${TIME_COLOR[item.type]}`}>
                                                        {item.date.toLocaleDateString(uiLocale(), { timeZone: 'Asia/Taipei' })}
                                                    </time>
                                                </div>
                                                <p className="mt-1 text-sm text-muted-foreground">{item.content}</p>
                                                <div className="mt-2 flex items-center justify-between gap-2">
                                                    <span className="flex items-center gap-2 text-xs text-muted-foreground">
                                                        記錄者：{item.actor || '系統'}
                                                        {item.vetRead && (
                                                            <Badge className="bg-status-success-bg text-status-success-text hover:bg-status-success-bg">獸醫已讀</Badge>
                                                        )}
                                                    </span>
                                                    {editable && (
                                                        <span className="flex shrink-0 gap-1">
                                                            <Button variant="ghost" size="icon" className="h-7 w-7" aria-label="編輯"
                                                                onClick={() => item.raw && onEdit(item.type as 'observation' | 'surgery', item.raw as AnimalObservation | AnimalSurgery)}>
                                                                <Edit2 className="h-3.5 w-3.5" />
                                                            </Button>
                                                            <Button variant="ghost" size="icon" className="h-7 w-7" aria-label="檢視"
                                                                onClick={() => onView(item.type as 'observation' | 'surgery', item.originalId as number)}>
                                                                <Eye className="h-3.5 w-3.5" />
                                                            </Button>
                                                            <Button variant="ghost" size="icon" className="h-7 w-7 text-status-error-solid hover:text-status-error-text" aria-label="刪除"
                                                                onClick={() => onDelete(item.type as 'observation' | 'surgery', item.originalId as number)}>
                                                                <Trash2 className="h-3.5 w-3.5" />
                                                            </Button>
                                                        </span>
                                                    )}
                                                </div>
                                            </div>
                                        </div>
                                    )
                                })}
                            </div>
                        ))}
                    </div>
                </div>
            )}
        </div>
    )
}
