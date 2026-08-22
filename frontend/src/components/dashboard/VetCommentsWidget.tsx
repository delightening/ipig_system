import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { MessageSquare, Loader2, User, Calendar } from 'lucide-react'
import api from '@/lib/api'

// 一筆 = 一條巡場觀察（多動物已於後端依 source entry 聚合去重，animals 帶各豬 id + 耳號）。
interface VetAdviceEntry {
    entry_id: string | null
    advice_date: string // 巡場日期 YYYY-MM-DD
    author_name: string
    suggested_treatment: string
    follow_up: string
    animals: { id: string; ear_tag: string; pen_location?: string | null }[]
}

// 前端依「巡場報告」＝（獸醫 + 巡場日期）分組成卡片。
interface AdviceGroup {
    key: string
    author_name: string
    advice_date: string
    entries: VetAdviceEntry[]
}

export function VetCommentsWidget() {
    const { t, i18n } = useTranslation()
    const navigate = useNavigate()

    const { data, isLoading, error } = useQuery({
        queryKey: ['recent-vet-comments'],
        queryFn: async () => {
            const res = await api.get<{ data: VetAdviceEntry[] }>('/animals/vet-comments?per_page=15')
            return res.data.data
        },
        staleTime: 30_000,
    })

    // 後端已 ORDER BY advice_date DESC, author → 依序分組即保序。
    const groups = useMemo<AdviceGroup[]>(() => {
        const map = new Map<string, AdviceGroup>()
        for (const e of data ?? []) {
            const key = `${e.author_name}||${e.advice_date}`
            let g = map.get(key)
            if (!g) {
                g = { key, author_name: e.author_name, advice_date: e.advice_date, entries: [] }
                map.set(key, g)
            }
            g.entries.push(e)
        }
        return Array.from(map.values())
    }, [data])

    // advice_date 為台北日曆日期；釘住 +08:00 避免瀏覽器本地時區把日期偏移一天
    const fmtDate = (d: string) =>
        new Date(`${d}T00:00:00+08:00`).toLocaleDateString(i18n.language, { timeZone: 'Asia/Taipei' })

    const header = (
        <CardHeader className="pt-3 pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
                <MessageSquare className="h-4 w-4 text-status-success-solid" />
                {t('dashboard.widgets.names.vet_comments')}
            </CardTitle>
            <CardDescription className="text-xs">{t('dashboard.widgets.animals.commentDescription')}</CardDescription>
        </CardHeader>
    )

    if (isLoading) {
        return (
            <Card className="h-full">
                {header}
                <CardContent>
                    <div className="flex items-center justify-center py-4">
                        <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                    </div>
                </CardContent>
            </Card>
        )
    }

    if (error) {
        return (
            <Card className="h-full">
                {header}
                <CardContent>
                    <p className="text-sm text-muted-foreground">{t('dashboard.widgets.common.loadFailed')}</p>
                </CardContent>
            </Card>
        )
    }

    return (
        <Card className="h-full flex flex-col overflow-hidden">
            {header}
            <CardContent className="flex-1 overflow-auto">
                {groups.length > 0 ? (
                    <div className="space-y-3">
                        {groups.map((g) => (
                            <div key={g.key} className="p-3 border rounded-lg space-y-2">
                                {/* 卡片表頭：獸醫 + 巡場日期 */}
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-2">
                                        <div className="w-6 h-6 rounded-full bg-emerald-100 flex items-center justify-center shrink-0">
                                            <User className="h-3 w-3 text-status-success-text" />
                                        </div>
                                        <span className="text-xs font-semibold">{g.author_name}</span>
                                    </div>
                                    <span className="text-xs text-muted-foreground flex items-center gap-1 shrink-0">
                                        <Calendar className="h-3 w-3" />
                                        {/* entry_id 為 source_vet_patrol_entry_id；null=非巡場來源（手寫病歷建議） */}
                                        {g.entries.every((e) => e.entry_id === null)
                                            ? t('dashboard.widgets.animals.manualDate', { date: fmtDate(g.advice_date) })
                                            : t('dashboard.widgets.animals.patrolDate', { date: fmtDate(g.advice_date) })}
                                    </span>
                                </div>

                                {/* 每條觀察：可點耳號 + 建議 + 追蹤改善 */}
                                <div className="space-y-2">
                                    {g.entries.map((e, i) => (
                                        <div key={e.entry_id ?? `${g.key}-${i}`} className="pl-1 border-l-2 border-emerald-100">
                                            <div className="flex flex-wrap gap-1 mb-1 pl-1">
                                                {e.animals.map((a) => (
                                                    <button
                                                        key={a.id}
                                                        type="button"
                                                        onClick={() => navigate(`/animals/${a.id}?tab=vet_recommendations`)}
                                                        title={`${t('dashboard.widgets.animals.earTag')} ${a.ear_tag}`}
                                                        className="text-xs bg-muted hover:bg-status-success-solid/15 hover:text-status-success-text px-2 py-0.5 rounded-full font-medium transition-colors cursor-pointer"
                                                    >
                                                        #{a.ear_tag}
                                                    </button>
                                                ))}
                                            </div>
                                            {e.suggested_treatment ? (
                                                <p className="text-xs whitespace-pre-wrap leading-snug pl-1">
                                                    <span className="text-muted-foreground">{t('dashboard.widgets.animals.suggestionLabel')}</span>{e.suggested_treatment}
                                                </p>
                                            ) : null}
                                            {e.follow_up ? (
                                                <p className="text-xs whitespace-pre-wrap leading-snug pl-1">
                                                    <span className="text-muted-foreground">{t('dashboard.widgets.animals.followUpLabel')}</span>{e.follow_up}
                                                </p>
                                            ) : null}
                                        </div>
                                    ))}
                                </div>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="flex flex-col items-center justify-center py-6 text-muted-foreground">
                        <MessageSquare className="h-8 w-8 mb-2 opacity-20" />
                        <p className="text-sm">{t('dashboard.widgets.animals.noComments')}</p>
                    </div>
                )}
            </CardContent>
        </Card>
    )
}
