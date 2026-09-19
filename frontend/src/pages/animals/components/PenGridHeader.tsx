import { useTranslation } from 'react-i18next'

import type { getZoneColors } from '../hooks/useFacilityLayout'

interface PenGridHeaderProps {
  colors: ReturnType<typeof getZoneColors>
  borderLeft?: boolean
  style?: React.CSSProperties
}

export function PenGridHeader({ colors, borderLeft, style }: PenGridHeaderProps) {
  const { t } = useTranslation()
  return (
    <div
      className={`grid grid-cols-2 @[600px]:grid-cols-4 gap-1 px-3 py-2 text-xs font-semibold ${colors.header} text-white ${borderLeft ? 'border-l border-white/30' : ''}`}
      style={style ?? colors.headerStyle}
    >
      <div>{t('animals.pen')}</div>
      <div>{t('animals.earTag')}</div>
      <div className="hidden @[600px]:block">{t('animalPages.penView.vetViewed')}</div>
      <div className="hidden @[600px]:block">{t('animalPages.penView.latestAbnormal')}</div>
    </div>
  )
}
