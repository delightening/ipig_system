// 疼痛評估總分趨勢折線圖
// 標記四個疼痛等級的分界線

import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ReferenceLine,
    ResponsiveContainer,
} from 'recharts'

interface PainAssessmentChartProps {
    data: Array<Record<string, string | number | null>>
}

export default function PainAssessmentChart({ data }: PainAssessmentChartProps) {
    return (
        <>
            <ResponsiveContainer width="100%" height={260}>
                <LineChart data={data} margin={{ top: 10, right: 20, left: 0, bottom: 5 }}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="name" fontSize={12} />
                    <YAxis domain={[0, 20]} ticks={[0, 5, 10, 15, 20]} fontSize={12} />
                    <Tooltip
                        formatter={(value) => {
                            const v = typeof value === 'number' ? value : null
                            if (v == null) return ['-', '總分'] as [string, string]
                            let grade: string
                            if (v <= 5) grade = '正常（等級1）'
                            else if (v <= 10) grade = '輕度疼痛（等級2）'
                            else if (v <= 15) grade = '中度疼痛（等級3）'
                            else grade = '重度疼痛（等級4）'
                            return [`${v} 分 — ${grade}`, '疼痛總分'] as [string, string]
                        }}
                    />
                    <ReferenceLine y={5} stroke="#22c55e" strokeDasharray="4 2"
                        label={{ value: '5', position: 'right', fontSize: 10, fill: '#22c55e' }} />
                    <ReferenceLine y={10} stroke="#f59e0b" strokeDasharray="4 2"
                        label={{ value: '10', position: 'right', fontSize: 10, fill: '#f59e0b' }} />
                    <ReferenceLine y={15} stroke="#f97316" strokeDasharray="4 2"
                        label={{ value: '15', position: 'right', fontSize: 10, fill: '#f97316' }} />
                    <Line
                        type="monotone"
                        dataKey="總分"
                        stroke="#7c3aed"
                        strokeWidth={2}
                        dot={{ r: 4, fill: '#7c3aed' }}
                        connectNulls
                    />
                </LineChart>
            </ResponsiveContainer>
            <div className="flex justify-center gap-4 mt-2 text-xs text-muted-foreground">
                <span className="text-green-600">■ 0–5：正常</span>
                <span className="text-yellow-600">■ 6–10：輕度疼痛</span>
                <span className="text-orange-600">■ 11–15：中度疼痛</span>
                <span className="text-red-600">■ 16–20：重度疼痛</span>
            </div>
        </>
    )
}
