// 疼痛評估總分趨勢折線圖
// 標記四個疼痛等級的分界線

import { useTranslation } from 'react-i18next'
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
    const { t } = useTranslation()
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
                            if (v == null) return ['-', t('animalRecords.painAssessment.total')] as [string, string]
                            let gradeKey: string
                            let level: number
                            if (v <= 5) { gradeKey = 'animalRecords.painAssessment.grade.normal'; level = 1 }
                            else if (v <= 10) { gradeKey = 'animalRecords.painAssessment.grade.mild'; level = 2 }
                            else if (v <= 15) { gradeKey = 'animalRecords.painAssessment.grade.moderate'; level = 3 }
                            else { gradeKey = 'animalRecords.painAssessment.grade.severe'; level = 4 }
                            const grade = t('animalRecords.painAssessment.chart.gradeWithLevel', { label: t(gradeKey), level })
                            return [
                                t('animalRecords.painAssessment.chart.tooltipValue', { value: v, grade }),
                                t('animalRecords.painAssessment.chart.totalPainScore'),
                            ] as [string, string]
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
                        dataKey="score"
                        stroke="#7c3aed"
                        strokeWidth={2}
                        dot={{ r: 4, fill: '#7c3aed' }}
                        connectNulls
                    />
                </LineChart>
            </ResponsiveContainer>
            <div className="flex justify-center gap-4 mt-2 text-xs text-muted-foreground">
                <span className="text-green-600">{t('animalRecords.painAssessment.chart.legend', { range: '0–5', label: t('animalRecords.painAssessment.grade.normal') })}</span>
                <span className="text-yellow-600">{t('animalRecords.painAssessment.chart.legend', { range: '6–10', label: t('animalRecords.painAssessment.grade.mild') })}</span>
                <span className="text-orange-600">{t('animalRecords.painAssessment.chart.legend', { range: '11–15', label: t('animalRecords.painAssessment.grade.moderate') })}</span>
                <span className="text-red-600">{t('animalRecords.painAssessment.chart.legend', { range: '16–20', label: t('animalRecords.painAssessment.grade.severe') })}</span>
            </div>
        </>
    )
}
