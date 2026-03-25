import type { getZoneColors } from '../hooks/useFacilityLayout'

interface PenGridHeaderProps {
  colors: ReturnType<typeof getZoneColors>
  borderLeft?: boolean
  style?: React.CSSProperties
}

export function PenGridHeader({ colors, borderLeft, style }: PenGridHeaderProps) {
  return (
    <div
      className={`grid grid-cols-5 gap-1 px-3 py-2 text-xs font-semibold ${colors.header} text-white ${borderLeft ? 'border-l border-white/30' : ''}`}
      style={style ?? colors.headerStyle}
    >
      <div>欄位</div>
      <div>耳號</div>
      <div>獸醫檢視</div>
      <div>最新異常</div>
      <div className="text-center">操作</div>
    </div>
  )
}
