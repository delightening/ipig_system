/**
 * StatusHelpPopover — 動物狀態說明彈出框
 *
 * 一顆 (i) 圖示 + Radix Popover，點按 / tap / 鍵盤皆可展開狀態說明。
 * - 傳 `status`：只顯示該狀態的說明（用於詳情頁 header 徽章旁）。
 * - 不傳 `status`：圖例模式，列出全部六個狀態（用於列表頁篩選列）。
 *
 * 文案集中於 i18n `animals.statusHelp.*`（中英雙語）。純說明層，不動狀態機。
 */

import * as PopoverPrimitive from '@radix-ui/react-popover'
import { Info } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { cn } from '@/lib/utils'
import type { AnimalStatus } from '@/types'

const ALL_STATUSES: AnimalStatus[] = [
  'unassigned',
  'in_experiment',
  'completed',
  'euthanized',
  'sudden_death',
  'transferred',
]

/** 生死 / 在場 分類（對映後端 is_terminal / is_active_in_facility）*/
type LifeTone = 'alive' | 'dead' | 'transfer'

const LIFE_TONE: Record<AnimalStatus, LifeTone> = {
  unassigned: 'alive',
  in_experiment: 'alive',
  completed: 'alive',
  euthanized: 'dead',
  sudden_death: 'dead',
  transferred: 'transfer',
}

/** life pill 顏色（一律用 status token）*/
const LIFE_PILL: Record<LifeTone, string> = {
  alive: 'bg-status-success-bg text-status-success-text',
  dead: 'bg-status-error-bg text-status-error-text',
  transfer: 'bg-status-purple-bg text-status-purple-text',
}

function StatusHelpBlock({ status }: { status: AnimalStatus }) {
  const { t } = useTranslation()
  const tone = LIFE_TONE[status]
  const warn = t(`animals.statusHelp.${status}.warn`, { defaultValue: '' })

  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-2">
        <span className="text-sm font-semibold">{t(`animals.statusLabels.${status}`)}</span>
        <span
          className={cn(
            'rounded-full px-2 py-0.5 text-[11px] font-semibold',
            LIFE_PILL[tone],
          )}
        >
          {t(`animals.statusHelp.life.${tone}`)}
        </span>
      </div>
      <p className="text-xs text-muted-foreground">
        <span className="font-medium text-foreground">{t('animals.statusHelp.defLabel')}</span>
        {t(`animals.statusHelp.${status}.def`)}
      </p>
      <p className="text-xs text-muted-foreground">
        <span className="font-medium text-foreground">{t('animals.statusHelp.nextLabel')}</span>
        {t(`animals.statusHelp.${status}.next`)}
      </p>
      {warn && (
        <p className="rounded-md border-l-2 border-status-warning-solid bg-status-warning-bg px-2 py-1.5 text-xs text-status-warning-text">
          {`⚠️ ${warn}`}
        </p>
      )}
    </div>
  )
}

interface StatusHelpPopoverProps {
  /** 只顯示單一狀態說明；省略則為圖例模式（全部狀態）*/
  status?: AnimalStatus
  /** 套用到 (i) 觸發按鈕的 className */
  className?: string
}

export function StatusHelpPopover({ status, className }: StatusHelpPopoverProps) {
  const { t } = useTranslation()
  const statuses = status ? [status] : ALL_STATUSES

  return (
    <PopoverPrimitive.Root>
      <PopoverPrimitive.Trigger asChild>
        <button
          type="button"
          aria-label={t('animals.statusHelp.aria')}
          className={cn(
            'inline-flex h-5 w-5 items-center justify-center rounded-full text-muted-foreground transition-colors',
            'hover:bg-accent hover:text-primary focus:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1',
            className,
          )}
        >
          <Info className="h-4 w-4" />
        </button>
      </PopoverPrimitive.Trigger>
      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          align="start"
          sideOffset={6}
          collisionPadding={12}
          className="z-[9999] w-[300px] max-w-[80vw] rounded-lg border border-border bg-popover p-3.5 text-popover-foreground shadow-lg"
        >
          {!status && (
            <p className="mb-2 text-xs font-semibold text-muted-foreground">
              {t('animals.statusHelp.legendTitle')}
            </p>
          )}
          {statuses.map((s, i) => (
            <div key={s} className={i > 0 ? 'mt-3 border-t border-border pt-3' : undefined}>
              <StatusHelpBlock status={s} />
            </div>
          ))}
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  )
}
