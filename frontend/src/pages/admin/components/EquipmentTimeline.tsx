/**
 * 設備履歷 Timeline — 整合維修/保養、校正/確效/查核、狀態變更紀錄
 */
import { StatusBadge } from '@/components/ui/status-badge'
import type { StatusVariant } from '@/components/ui/status-badge'
import { Clock, Wrench, Ruler, RefreshCw, History } from 'lucide-react'

import type { EquipmentTimelineEntry, TimelineEventType } from '../types'

interface Props {
  entries: EquipmentTimelineEntry[]
}

const EVENT_CONFIG: Record<TimelineEventType, {
  label: string
  variant: StatusVariant
  icon: typeof Wrench
  dotClass: string
}> = {
  maintenance: {
    label: '維修/保養',
    variant: 'warning',
    icon: Wrench,
    dotClass: 'bg-amber-500',
  },
  calibration: {
    label: '校正/確效/查核',
    variant: 'info',
    icon: Ruler,
    dotClass: 'bg-blue-500',
  },
  status_change: {
    label: '狀態變更',
    variant: 'neutral',
    icon: RefreshCw,
    dotClass: 'bg-muted-foreground',
  },
}

function formatDateTime(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-TW', {
    timeZone: 'Asia/Taipei',
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function EquipmentTimeline({ entries }: Props) {
  if (entries.length === 0) {
    return (
      <div className="text-center py-12 text-muted-foreground">
        <History className="h-12 w-12 mx-auto mb-4" />
        <p>尚無設備履歷紀錄</p>
      </div>
    )
  }

  return (
    <div className="relative">
      {entries.map((entry, index) => (
        <TimelineItem
          key={entry.id}
          entry={entry}
          isLast={index === entries.length - 1}
        />
      ))}
    </div>
  )
}

function TimelineItem({ entry, isLast }: { entry: EquipmentTimelineEntry; isLast: boolean }) {
  const config = EVENT_CONFIG[entry.event_type] ?? EVENT_CONFIG.status_change
  const Icon = config.icon
  const detail = entry.detail as Record<string, unknown>
  const notes = detail?.notes ? String(detail.notes) : null
  const actorName = detail?.actor_name ? String(detail.actor_name) : null

  return (
    <div className="relative flex gap-4 pb-6 last:pb-0">
      {!isLast && (
        <div className="absolute left-[13px] top-7 bottom-0 w-0.5 bg-muted" />
      )}
      <div className={`relative z-10 flex h-7 w-7 shrink-0 items-center justify-center rounded-full ${config.dotClass} ring-4 ring-background`}>
        <Icon className="h-3.5 w-3.5 text-white" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between mb-1">
          <div className="flex items-center gap-2 text-sm">
            <StatusBadge variant={config.variant}>{config.label}</StatusBadge>
            <span className="font-medium">{entry.title}</span>
          </div>
          <div className="flex items-center gap-1 text-sm text-muted-foreground shrink-0 ml-2">
            <Clock className="h-3.5 w-3.5" />
            <span>{formatDateTime(entry.occurred_at)}</span>
          </div>
        </div>

        {(entry.subtitle || notes || actorName) && (
          <div className="p-3 bg-muted rounded-lg text-sm space-y-1">
            {entry.subtitle && (
              <p>{entry.subtitle}</p>
            )}
            {notes && (
              <p className="text-muted-foreground">{'備註：'}{notes}</p>
            )}
            {actorName && (
              <p className="text-muted-foreground">{'執行者：'}{actorName}</p>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
