import type { getZoneColors } from '../hooks/useFacilityLayout'

interface PenGridHeaderProps {
  colors: ReturnType<typeof getZoneColors>
  borderLeft?: boolean
  style?: React.CSSProperties
}

export function PenGridHeader({ colors, borderLeft, style }: PenGridHeaderProps) {
  return (
    <div
      className={`grid grid-cols-2 @[600px]:grid-cols-4 gap-1 px-3 py-2 text-xs font-semibold ${colors.header} text-white ${borderLeft ? 'border-l border-white/30' : ''}`}
      style={style ?? colors.headerStyle}
    >
      <div>欄位</div>
      <div>耳號</div>
      <div className="hidden @[600px]:block">獸醫檢視</div>
      <div className="hidden @[600px]:block">最新異常</div>
    </div>
  )
}
