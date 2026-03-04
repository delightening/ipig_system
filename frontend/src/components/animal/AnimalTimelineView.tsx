import { Animal, AnimalObservation, AnimalSurgery, AnimalSacrifice, AnimalSuddenDeath, AnimalWeight, AnimalTransfer, AnimalEvent, RecordType, recordTypeNames, transferStatusNames } from '@/lib/api'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
    ClipboardList,
    Scissors,
    CheckCircle2,
    Eye,
    Edit2,
    Trash2,
    Scale,
    Calendar,
    Skull,
    Zap,
    ArrowRightLeft,
} from 'lucide-react'

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

type TimelineItemType = 'observation' | 'surgery' | 'weight' | 'created' | 'sacrificed' | 'completed' | 'euthanized' | 'sudden_death' | 'transferred' | 'iacuc_change'

interface TimelineItem {
    id: string
    /** 觀察/手術/體重為 number；犧牲等為 string (UUID) 或 0 */
    originalId: number | string
    type: TimelineItemType
    date: Date
    title: string
    content: string
    actor?: string | null
    vetRead?: boolean
    isNoMed?: boolean
    raw?: unknown
    isInfoOnly?: boolean // 唯讀型項目（無操作按鈕）
}

export function AnimalTimelineView({
    observations,
    surgeries,
    animalWeights,
    sacrifice,
    suddenDeath,
    transfers,
    iacucEvents,
    animal,
    onView,
    onEdit,
    onCopy: _onCopy,
    onHistory: _onHistory,
    onVet: _onVet,
    onDelete,
}: Props) {

    // 合併並排序紀錄
    const timelineItems: TimelineItem[] = [
        ...observations.map(obs => ({
            id: `obs-${obs.id}`,
            originalId: obs.id,
            type: 'observation' as const,
            date: new Date(obs.event_date),
            title: recordTypeNames[obs.record_type as RecordType],
            content: obs.content,
            actor: obs.created_by_name,
            vetRead: obs.vet_read,
            isNoMed: obs.no_medication_needed,
            raw: obs,
            isInfoOnly: false,
        })),
        ...surgeries.map(surg => ({
            id: `surg-${surg.id}`,
            originalId: surg.id,
            type: 'surgery' as const,
            date: new Date(surg.surgery_date),
            title: surg.is_first_experiment ? '首次手術' : '手術紀錄',
            content: surg.surgery_site,
            actor: surg.created_by_name,
            vetRead: surg.vet_read,
            isNoMed: surg.no_medication_needed,
            raw: surg,
            isInfoOnly: false,
        })),
        // 體重測量紀錄
        ...(animalWeights || []).map(w => ({
            id: `weight-${w.id}`,
            originalId: w.id,
            type: 'weight' as const,
            date: new Date(w.measure_date),
            title: '體重測量',
            content: `${w.weight} kg`,
            actor: w.created_by_name || null,
            vetRead: false,
            isNoMed: false,
            raw: w,
            isInfoOnly: true,
        })),
        // 動物建立日期
        ...(animal ? [{
            id: 'animal-created',
            originalId: 0,
            type: 'created' as const,
            date: new Date(animal.entry_date || animal.created_at),
            title: '資料建立',
            content: `耳號 ${animal.ear_tag} 進場${animal.entry_weight ? `，進場體重 ${animal.entry_weight} kg` : ''}`,
            actor: null as string | null,
            vetRead: false,
            isNoMed: false,
            raw: animal,
            isInfoOnly: true,
        }] : []),
        // 犧牲/採樣紀錄
        ...(sacrifice && sacrifice.sacrifice_date ? [{
            id: 'sacrifice-record',
            originalId: sacrifice.id,
            type: 'sacrificed' as const,
            date: new Date(sacrifice.sacrifice_date),
            title: '犧牲/採樣',
            content: [
                sacrifice.method_electrocution ? '電擊' : null,
                sacrifice.method_bloodletting ? '放血' : null,
                sacrifice.method_other ? sacrifice.method_other : null,
                sacrifice.blood_volume_ml ? `採血 ${sacrifice.blood_volume_ml} ml` : null,
                sacrifice.sampling ? `採樣：${sacrifice.sampling}` : null,
            ].filter(Boolean).join('、') || '已犧牲',
            actor: sacrifice.created_by_name || null,
            vetRead: false,
            isNoMed: false,
            raw: sacrifice,
            isInfoOnly: true,
        }] : []),
        // 實驗完成事件（status = completed 但無犧牲紀錄時顯示）
        ...(animal && animal.status === 'completed' && !(sacrifice && sacrifice.sacrifice_date) ? [{
            id: 'experiment-completed',
            originalId: 0,
            type: 'completed' as const,
            date: new Date(animal.updated_at),
            title: '實驗完成',
            content: `耳號 ${animal.ear_tag} 實驗已完成`,
            actor: null as string | null,
            vetRead: false,
            isNoMed: false,
            raw: animal,
            isInfoOnly: true,
        }] : []),
        // 安樂死事件
        ...(animal && animal.status === 'euthanized' ? [{
            id: 'euthanized-event',
            originalId: 0,
            type: 'euthanized' as const,
            date: new Date(sacrifice?.sacrifice_date || animal.updated_at),
            title: '已安樂死',
            content: [
                sacrifice?.method_electrocution ? '電擊' : null,
                sacrifice?.method_bloodletting ? '放血' : null,
                sacrifice?.method_other ? sacrifice.method_other : null,
                sacrifice?.blood_volume_ml ? `採血 ${sacrifice.blood_volume_ml} ml` : null,
            ].filter(Boolean).join('、') || `耳號 ${animal.ear_tag} 已安樂死`,
            actor: sacrifice?.created_by_name || null,
            vetRead: false,
            isNoMed: false,
            raw: sacrifice || animal,
            isInfoOnly: true,
        }] : []),
        // 猝死事件
        ...(suddenDeath ? [{
            id: 'sudden-death-event',
            originalId: 0,
            type: 'sudden_death' as const,
            date: new Date(suddenDeath.discovered_at),
            title: '猝死',
            content: [
                suddenDeath.probable_cause ? `可能原因：${suddenDeath.probable_cause}` : null,
                suddenDeath.location ? `地點：${suddenDeath.location}` : null,
                suddenDeath.requires_pathology ? '需要病理檢查' : null,
            ].filter(Boolean).join('、') || '動物猝死',
            actor: null as string | null,
            vetRead: false,
            isNoMed: false,
            raw: suddenDeath,
            isInfoOnly: true,
        }] : []),
        // 轉讓事件
        ...(transfers || []).map(t => ({
            id: `transfer-${t.id}`,
            originalId: 0,
            type: 'transferred' as const,
            date: new Date(t.completed_at || t.created_at),
            title: t.status === 'completed' ? '轉讓完成' : t.status === 'rejected' ? '轉讓拒絕' : '轉讓進行中',
            content: `${t.from_iacuc_no} → ${t.to_iacuc_no || '待定'} (${transferStatusNames[t.status]})`,
            actor: null as string | null,
            vetRead: false,
            isNoMed: false,
            raw: t,
            isInfoOnly: true,
        })),
        // IACUC No. 變更事件
        ...(iacucEvents || []).map(evt => ({
            id: `iacuc-${evt.id}`,
            originalId: 0,
            type: 'iacuc_change' as const,
            date: new Date(evt.created_at),
            title: 'IACUC No. 變更',
            content: `${evt.before_data?.iacuc_no || '（無）'} → ${evt.after_data?.iacuc_no || '（無）'}`,
            actor: evt.actor_name || null,
            vetRead: false,
            isNoMed: false,
            raw: evt,
            isInfoOnly: true,
        })),
    ].sort((a, b) => b.date.getTime() - a.date.getTime())

    // 取得時間軸項目的圖示
    const getIcon = (type: TimelineItemType) => {
        switch (type) {
            case 'observation': return <ClipboardList className="h-5 w-5" />
            case 'surgery': return <Scissors className="h-5 w-5" />
            case 'weight': return <Scale className="h-5 w-5" />
            case 'created': return <Calendar className="h-5 w-5" />
            case 'sacrificed': return <Skull className="h-5 w-5" />
            case 'completed': return <CheckCircle2 className="h-5 w-5" />
            case 'euthanized': return <Skull className="h-5 w-5" />
            case 'sudden_death': return <Zap className="h-5 w-5" />
            case 'transferred': return <ArrowRightLeft className="h-5 w-5" />
            case 'iacuc_change': return <Edit2 className="h-5 w-5" />
            default: return <ClipboardList className="h-5 w-5" />
        }
    }

    // 取得時間軸項目的底色
    const getDotColor = (type: TimelineItemType) => {
        switch (type) {
            case 'observation': return 'bg-slate-100 dark:bg-slate-700 text-slate-500'
            case 'surgery': return 'bg-orange-100 dark:bg-orange-900 text-orange-600'
            case 'weight': return 'bg-blue-100 dark:bg-blue-900 text-blue-600'
            case 'created': return 'bg-green-100 dark:bg-green-900 text-green-600'
            case 'sacrificed': return 'bg-red-100 dark:bg-red-900 text-red-600'
            case 'completed': return 'bg-emerald-100 dark:bg-emerald-900 text-emerald-600'
            case 'euthanized': return 'bg-red-100 dark:bg-red-900 text-red-600'
            case 'sudden_death': return 'bg-rose-100 dark:bg-rose-900 text-rose-600'
            case 'transferred': return 'bg-indigo-100 dark:bg-indigo-900 text-indigo-600'
            case 'iacuc_change': return 'bg-amber-100 dark:bg-amber-900 text-amber-600'
            default: return 'bg-slate-100 dark:bg-slate-700 text-slate-500'
        }
    }

    // 取得時間標籤色
    const getTimeBadgeColor = (type: TimelineItemType) => {
        switch (type) {
            case 'observation': return 'text-purple-600 bg-purple-50'
            case 'surgery': return 'text-orange-600 bg-orange-50'
            case 'weight': return 'text-blue-600 bg-blue-50'
            case 'created': return 'text-green-600 bg-green-50'
            case 'sacrificed': return 'text-red-600 bg-red-50'
            case 'completed': return 'text-emerald-600 bg-emerald-50'
            case 'euthanized': return 'text-red-600 bg-red-50'
            case 'sudden_death': return 'text-rose-600 bg-rose-50'
            case 'transferred': return 'text-indigo-600 bg-indigo-50'
            case 'iacuc_change': return 'text-amber-600 bg-amber-50'
            default: return 'text-purple-600 bg-purple-50'
        }
    }

    return (
        <div className="relative space-y-8 before:absolute before:inset-0 before:ml-5 before:-translate-x-px md:before:mx-auto md:before:translate-x-0 before:h-full before:w-0.5 before:bg-gradient-to-b before:from-transparent before:via-slate-300 before:to-transparent">
            {timelineItems.length === 0 ? (
                <Card className="border-dashed">
                    <CardContent className="py-12 text-center text-slate-500">
                        <ClipboardList className="h-12 w-12 mx-auto mb-4 text-slate-300" />
                        <p>尚無任何紀錄</p>
                    </CardContent>
                </Card>
            ) : (
                timelineItems.map((item) => (
                    <div key={item.id} className="relative flex items-center justify-between md:justify-normal md:odd:flex-row-reverse group is-active">
                        {/* Dot */}
                        <div className={`flex items-center justify-center w-10 h-10 rounded-full border border-white ${getDotColor(item.type)} shadow shrink-0 md:order-1 md:group-odd:-translate-x-1/2 md:group-even:translate-x-1/2`}>
                            {getIcon(item.type)}
                        </div>
                        {/* Card */}
                        <div className="w-[calc(100%-4rem)] md:w-[45%] bg-white dark:bg-slate-800 p-4 rounded-xl border border-slate-200 dark:border-slate-700 shadow-sm transition-all hover:shadow-md">
                            <div className="flex items-center justify-between mb-2">
                                <div className="font-bold text-slate-900 dark:text-slate-100">{item.title}</div>
                                <time className={`text-xs font-medium ${getTimeBadgeColor(item.type)} px-2 py-1 rounded-full`}>{item.date.toLocaleDateString('zh-TW')}</time>
                            </div>
                            <div className="text-slate-600 dark:text-slate-400 text-sm mb-4 line-clamp-2">{item.content}</div>
                            <div className="flex items-center justify-between text-xs text-slate-500">
                                <div className="flex items-center gap-2">
                                    <span>記錄者: {item.actor || '系統'}</span>
                                    {item.vetRead && <Badge className="bg-green-100 text-green-800 hover:bg-green-200">獸醫已讀</Badge>}
                                </div>
                                {/* 只有觀察和手術紀錄才有操作按鈕 */}
                                {!item.isInfoOnly && (item.type === 'observation' || item.type === 'surgery') && (
                                    <div className="flex gap-1">
                                        <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => item.raw && (item.type === 'observation' || item.type === 'surgery') && onEdit(item.type, item.raw as AnimalObservation | AnimalSurgery)} aria-label="編輯"><Edit2 className="h-3.5 w-3.5" /></Button>
                                        <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => onView(item.type as 'observation' | 'surgery', item.originalId as number)} aria-label="檢視"><Eye className="h-3.5 w-3.5" /></Button>
                                        <Button variant="ghost" size="icon" className="h-7 w-7 text-red-500 hover:text-red-600" onClick={() => onDelete(item.type as 'observation' | 'surgery', item.originalId as number)} aria-label="刪除"><Trash2 className="h-3.5 w-3.5" /></Button>
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>
                ))
            )}
        </div>
    )
}
