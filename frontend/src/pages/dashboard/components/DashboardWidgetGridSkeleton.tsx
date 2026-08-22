import { useTranslation } from 'react-i18next'
import { SkeletonPulse } from '@/components/ui/skeleton'

// 佔位卡片高度（對應 widget 常見的 2~4 grid row）。純視覺佔位，與實際佈局無關——
// 真正的座標要等使用者偏好載入後才知道，這裡刻意不套用預設佈局，
// 避免「先畫預設位置、載入完再跳到儲存位置」的閃動。
const PLACEHOLDER_HEIGHTS = ['h-56', 'h-56', 'h-40', 'h-40', 'h-56', 'h-40']

/**
 * 儀表板網格載入骨架：使用者的 widget 佈局偏好尚未取回時顯示。
 * 目的是讓 widget「一出現就在正確位置」，而不是先以預設佈局畫一次再重排。
 */
export function DashboardWidgetGridSkeleton() {
  const { t } = useTranslation()
  return (
    <div
      role="status"
      aria-label={t('common.loading')}
      className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3"
    >
      {PLACEHOLDER_HEIGHTS.map((height, index) => (
        <SkeletonPulse key={index} className={`${height} rounded-lg`} />
      ))}
    </div>
  )
}
