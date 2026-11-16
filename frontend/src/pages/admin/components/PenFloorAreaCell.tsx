import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import type { PenDetails } from '@/types/facility'
import { formatMeters } from '../utils/penSize'

type AreaBadge = { variant: 'success' | 'warning' | 'secondary'; labelKey: string }

/** 欄位地板面積狀態標籤：足夠／超標／缺體重／未設定（只警示，不擋操作） */
function floorAreaBadge(pen: PenDetails): AreaBadge {
  const fa = pen.floor_area
  if (!fa || fa.status === 'no_area') {
    return { variant: 'secondary', labelKey: 'admin.penTab.areaStatusNoArea' }
  }
  if (fa.status === 'exceeded') {
    return { variant: 'warning', labelKey: 'admin.penTab.areaStatusExceeded' }
  }
  // 缺體重的動物不計入所需面積，「足夠」可能被低估，故另外標示
  if (fa.animals_missing_weight > 0) {
    return { variant: 'warning', labelKey: 'admin.penTab.areaStatusMissingWeight' }
  }
  return { variant: 'success', labelKey: 'admin.penTab.areaStatusOk' }
}

/** 地板面積欄：長 × 寬 = 面積、需／餘（或超出）面積，下方附狀態標籤 */
export function PenFloorAreaCell({ pen }: { pen: PenDetails }) {
  const { t } = useTranslation()
  const fa = pen.floor_area
  const badge = floorAreaBadge(pen)

  if (!fa || fa.status === 'no_area' || fa.area_m2 == null) {
    return (
      <div className="flex flex-col gap-1">
        <span className="text-muted-foreground">—</span>
        <Badge variant={badge.variant} className="w-fit">{t(badge.labelKey)}</Badge>
      </div>
    )
  }

  const area = Number(fa.area_m2)
  const required = Number(fa.required_area_m2)
  const diff = Math.abs(area - required)

  return (
    <div className="flex flex-col gap-1">
      <span className="tabular-nums whitespace-nowrap">
        {t('admin.penTab.areaFormula', {
          length: formatMeters(pen.length_m ?? 0),
          width: formatMeters(pen.width_m ?? 0),
          area: formatMeters(area),
        })}
      </span>
      <span className="text-xs text-muted-foreground tabular-nums whitespace-nowrap">
        {t('admin.penTab.areaNeed', { required: formatMeters(required) })}
        {' · '}
        {fa.status === 'exceeded'
          ? t('admin.penTab.areaOver', { over: formatMeters(diff) })
          : t('admin.penTab.areaLeft', { left: formatMeters(diff) })}
      </span>
      {fa.animals_missing_weight > 0 && (
        <span className="text-xs text-muted-foreground whitespace-nowrap">
          {t('admin.penTab.areaMissingWeight', { count: fa.animals_missing_weight })}
        </span>
      )}
      <Badge variant={badge.variant} className="w-fit">{t(badge.labelKey)}</Badge>
    </div>
  )
}
