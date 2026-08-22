import { LineChart, Line, XAxis, YAxis, Tooltip, CartesianGrid, ResponsiveContainer } from 'recharts'
import { Scale } from 'lucide-react'

import { AnimalWeight } from '@/lib/api'
import { uiLocale } from '@/lib/utils'

interface Props {
    weights: AnimalWeight[]
}

/**
 * 動物病程時間軸的「體重趨勢」卡片。
 *
 * 取代原本逐筆體重事件（避免高頻低訊號事件洗版時間軸），改以單一趨勢圖呈現整段
 * 體重變化；逐筆數值由 Recharts tooltip 提供。單筆時只顯示數值、不畫圖。
 */
export function AnimalWeightTrendCard({ weights }: Props) {
    if (!weights.length) return null

    const sorted = [...weights].sort(
        (a, b) => new Date(a.measure_date).getTime() - new Date(b.measure_date).getTime(),
    )
    const latest = sorted[sorted.length - 1]
    const latestWeight = Number(latest.weight)
    const fmtDate = (d: string) =>
        new Date(d).toLocaleDateString(uiLocale(), { month: 'numeric', day: 'numeric', timeZone: 'Asia/Taipei' })

    const data = sorted.map((w) => ({ name: fmtDate(w.measure_date), weight: Number(w.weight) }))

    return (
        <div className="relative pl-12">
            {/* 軸點（與時間軸對齊；體重沿用原藍色語義） */}
            <div className="absolute left-1 top-0 flex h-6 w-6 items-center justify-center rounded-full border-2 border-card bg-status-info-bg text-status-info-text shadow-[0_0_0_1px_hsl(var(--border))]">
                <Scale className="h-3 w-3" />
            </div>
            <div className="rounded-lg border border-border bg-card p-3 shadow-xs">
                <div className="mb-1 flex items-baseline gap-2">
                    <span className="text-sm font-semibold text-foreground">體重趨勢</span>
                    <span className="text-xs text-muted-foreground">
                        {sorted.length} 筆 · {fmtDate(sorted[0].measure_date)}–{fmtDate(latest.measure_date)}
                    </span>
                    <span className="ml-auto text-sm font-bold text-status-info-text">最新 {latestWeight} kg</span>
                </div>
                {sorted.length >= 2 && (
                    <ResponsiveContainer width="100%" height={120}>
                        <LineChart data={data} margin={{ top: 8, right: 12, left: -16, bottom: 0 }}>
                            <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" />
                            <XAxis dataKey="name" fontSize={11} tickMargin={6} />
                            <YAxis fontSize={11} width={40} unit=" kg" />
                            <Tooltip
                                formatter={(value) => [`${value} kg`, '體重']}
                                contentStyle={{ fontSize: 12, borderRadius: 8 }}
                            />
                            <Line
                                type="monotone"
                                dataKey="weight"
                                stroke="hsl(var(--primary))"
                                strokeWidth={2}
                                dot={{ r: 3, fill: 'hsl(var(--primary))' }}
                            />
                        </LineChart>
                    </ResponsiveContainer>
                )}
            </div>
        </div>
    )
}
