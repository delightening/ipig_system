import type { BoxPlotData } from '../hooks/useBloodTestAnalysis'

export function BoxPlotChart({ data }: { data: BoxPlotData[] }) {
  if (data.length === 0) return null

  const maxVal = Math.max(...data.map(d => d.max))
  const chartHeight = Math.max(200, data.length * 60)

  return (
    <div className="w-full overflow-x-auto">
      <svg width="100%" height={chartHeight} viewBox={`0 0 700 ${chartHeight}`} className="text-sm">
        {data.map((item, idx) => {
          const y = idx * 56 + 30
          const barHeight = 24
          const plotStart = 180
          const plotWidth = 480
          const scale = (v: number) => plotStart + (v / (maxVal * 1.1)) * plotWidth

          return (
            <g key={item.name}>
              <text x={0} y={y + barHeight / 2 + 4} className="fill-current text-xs" fontSize="11">
                {item.name} {item.unit ? `(${item.unit})` : ''} n={item.count}
              </text>
              <line x1={scale(item.min)} y1={y + barHeight / 2} x2={scale(item.q1)} y2={y + barHeight / 2}
                stroke="currentColor" strokeWidth="1" className="text-muted-foreground" />
              <line x1={scale(item.min)} y1={y + 4} x2={scale(item.min)} y2={y + barHeight - 4}
                stroke="currentColor" strokeWidth="1" className="text-muted-foreground" />
              <rect x={scale(item.q1)} y={y} width={scale(item.q3) - scale(item.q1)} height={barHeight}
                fill="hsl(var(--primary) / 0.15)" stroke="hsl(var(--primary))" strokeWidth="1.5" rx="3" />
              <line x1={scale(item.median)} y1={y} x2={scale(item.median)} y2={y + barHeight}
                stroke="hsl(var(--primary))" strokeWidth="2.5" />
              <line x1={scale(item.q3)} y1={y + barHeight / 2} x2={scale(item.max)} y2={y + barHeight / 2}
                stroke="currentColor" strokeWidth="1" className="text-muted-foreground" />
              <line x1={scale(item.max)} y1={y + 4} x2={scale(item.max)} y2={y + barHeight - 4}
                stroke="currentColor" strokeWidth="1" className="text-muted-foreground" />
            </g>
          )
        })}
      </svg>
    </div>
  )
}
