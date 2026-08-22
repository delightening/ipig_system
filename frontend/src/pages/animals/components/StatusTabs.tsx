import { StatusHelpPopover } from '@/components/animal/StatusHelpPopover'

export interface StatusTabItem {
  value: string
  label: string
  count: number
}

interface StatusTabsProps {
  tabs: StatusTabItem[]
  statusFilter: string
  onStatusFilterChange: (value: string) => void
}

/** 動物狀態 tag 列（未分配 / 實驗中 / …／所有動物）。棟別 tag 由 BuildingTabs 負責。 */
export function StatusTabs({ tabs, statusFilter, onStatusFilterChange }: StatusTabsProps) {
  return (
    <div className="flex flex-wrap gap-2 border-b">
      {tabs.map(tab => (
        <button
          key={tab.value}
          type="button"
          // data-status 是 status enum，不隨 i18n 變動，供 E2E 穩定選取
          data-testid="status-tab"
          data-status={tab.value}
          aria-pressed={statusFilter === tab.value}
          onClick={() => onStatusFilterChange(tab.value)}
          className={`px-3 md:px-4 py-2 text-xs md:text-sm font-medium border-b-2 transition-colors flex items-center gap-1.5 ${
            statusFilter === tab.value
              ? 'border-primary text-primary'
              : 'border-transparent text-muted-foreground hover:text-foreground'
          }`}
        >
          {tab.label} ({tab.count})
        </button>
      ))}
      <StatusHelpPopover className="self-center" />
    </div>
  )
}
