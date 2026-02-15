import { Animal, AnimalObservation, AnimalSurgery, AnimalWeight, RecordType, recordTypeNames } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
    ClipboardList,
    Scissors,
    Stethoscope,
    CheckCircle2,
    ChevronDown,
    Eye,
    Edit2,
    Copy,
    History,
    Trash2,
    Scale,
    Calendar,
    Skull,
} from 'lucide-react'
import { useState } from 'react'

interface Props {
    observations: AnimalObservation[]
    surgeries: AnimalSurgery[]
    animalWeights?: AnimalWeight[]
    animal?: Animal
    onView: (type: 'observation' | 'surgery', id: number) => void
    onEdit: (type: 'observation' | 'surgery', record: any) => void
    onCopy: (type: 'observation' | 'surgery', id: number) => void
    onHistory: (type: 'observation' | 'surgery', id: number) => void
    onVet: (type: 'observation' | 'surgery', id: number) => void
    onDelete: (type: 'observation' | 'surgery', id: number) => void
}

type TimelineItemType = 'observation' | 'surgery' | 'weight' | 'created' | 'sacrificed'

interface TimelineItem {
    id: string
    originalId: number
    type: TimelineItemType
    date: Date
    title: string
    content: string
    actor?: string | null
    vetRead?: boolean
    isNoMed?: boolean
    raw?: any
    isInfoOnly?: boolean // 唯讀型項目（無操作按鈕）
}

export function AnimalTimelineView({
    observations,
    surgeries,
    animalWeights,
    animal,
    onView,
    onEdit,
    onCopy,
    onHistory,
    onVet,
    onDelete,
}: Props) {
    const [expandedId, setExpandedId] = useState<string | null>(null)

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
    ].sort((a, b) => b.date.getTime() - a.date.getTime())

    // 取得時間軸項目的圖示
    const getIcon = (type: TimelineItemType) => {
        switch (type) {
            case 'observation': return <ClipboardList className="h-5 w-5" />
            case 'surgery': return <Scissors className="h-5 w-5" />
            case 'weight': return <Scale className="h-5 w-5" />
            case 'created': return <Calendar className="h-5 w-5" />
            case 'sacrificed': return <Skull className="h-5 w-5" />
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
                timelineItems.map((item, index) => (
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
                                        <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => onEdit(item.type as 'observation' | 'surgery', item.raw)}><Edit2 className="h-3.5 w-3.5" /></Button>
                                        <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => onView(item.type as 'observation' | 'surgery', item.originalId)}><Eye className="h-3.5 w-3.5" /></Button>
                                        <Button variant="ghost" size="icon" className="h-7 w-7 text-red-500 hover:text-red-600" onClick={() => onDelete(item.type as 'observation' | 'surgery', item.originalId)}><Trash2 className="h-3.5 w-3.5" /></Button>
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
