import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    Legend,
    ResponsiveContainer,
} from 'recharts'

interface PainAssessmentChartProps {
    data: Array<Record<string, string | number | null>>
}

export default function PainAssessmentChart({ data }: PainAssessmentChartProps) {
    return (
        <>
            <ResponsiveContainer width="100%" height={240}>
                <LineChart data={data} margin={{ top: 5, right: 20, left: 0, bottom: 5 }}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="name" fontSize={12} />
                    <YAxis domain={[0, 4]} ticks={[0, 1, 2, 3, 4]} fontSize={12} />
                    <Tooltip />
                    <Legend />
                    <Line type="monotone" dataKey="\u7cbe\u795e" stroke="#f59e0b" strokeWidth={2} connectNulls />
                    <Line type="monotone" dataKey="\u98df\u6175" stroke="#10b981" strokeWidth={2} connectNulls />
                    <Line type="monotone" dataKey="\u7ad9\u7acb" stroke="#6366f1" strokeWidth={2} connectNulls />
                    <Line type="monotone" dataKey="\u884c\u8d70" stroke="#8b5cf6" strokeWidth={2} connectNulls />
                    <Line type="monotone" dataKey="\u884c\u70ba" stroke="#ef4444" strokeWidth={2} connectNulls />
                </LineChart>
            </ResponsiveContainer>
            <p className="text-xs text-muted-foreground mt-2 text-center">
                0 = \u6975\u5dee\u30011 = \u5dee\u30012 = \u56f0\u96e3\u30013 = \u5c1a\u53ef\u30014 = \u826f\u597d/\u6b63\u5e38
            </p>
        </>
    )
}
