// 單一類別區塊（可折疊 + 條目卡片）（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import { Plus, ChevronDown, ChevronRight } from 'lucide-react'
import { EntryCard } from './EntryCard'
import type { CATEGORIES } from './constants'
import type { VetPatrolReportVM } from './useVetPatrolReport'

type Category = typeof CATEGORIES[number]

export function CategorySection({ vm, cat }: { vm: VetPatrolReportVM; cat: Category }) {
    const isCollapsed = vm.collapsedCategories.has(cat.key)
    const rows = vm.entries[cat.key]
    const filledCount = rows.filter(r => r.observation || r.suggestion || r.follow_up).length
    // 新增觀察條目僅獸醫在草稿階段可做：update service 於 AWAITING_FOLLOW_UP 對 id=None
    // 的新條目直接回 BusinessRule（#378），已完成則整份鎖定。與 EntryCard 同一組旗標。
    const canAddRow = !vm.isReadOnly && !vm.canEditFollowUpOnly

    return (
        <div className="border rounded-lg overflow-hidden">
            {/* a11y：展開鈕與「新增列」為同層 sibling，不可巢狀（HTML 規範禁止 button 內含
                互動元素，React 會發 validateDOMNesting、輔助科技對焦點/點擊處理不一致）。
                外層改為非互動 div，只保留背景與 hover 樣式。
                padding 必須放在兩個 button 各自身上而非外層 div——否則留白落在非互動容器上，
                點下去不會展開，可點區域比原本的整列 button 縮小（CodeRabbit #39 Minor）。
                展開鈕帶 flex-1 吃掉剩餘寬度，故標題列空白處仍屬展開鈕的可點範圍。
                同時不再需要 stopPropagation——原本那行正是巢狀結構的補丁。 */}
            <div className="w-full flex items-center justify-between bg-muted/50 hover:bg-muted transition-colors">
                <button
                    type="button"
                    onClick={() => vm.toggleCategory(cat.key)}
                    aria-expanded={!isCollapsed}
                    className="flex flex-1 items-center gap-2 px-3 py-2 text-left"
                >
                    {isCollapsed
                        ? <ChevronRight className="h-4 w-4 text-muted-foreground" />
                        : <ChevronDown className="h-4 w-4 text-muted-foreground" />
                    }
                    <span className="text-sm font-semibold">{cat.label}</span>
                    {filledCount > 0 && (
                        <span className="text-xs text-muted-foreground">({filledCount} 筆)</span>
                    )}
                </button>
                {canAddRow && (
                    <button
                        type="button"
                        onClick={() => vm.addRow(cat.key)}
                        className="flex items-center gap-1 px-3 py-2 text-xs text-status-success-solid hover:text-status-success-text transition-colors"
                    >
                        <Plus className="h-3 w-3" /> 新增列
                    </button>
                )}
            </div>

            {!isCollapsed && (
                <div className="p-3 space-y-3 bg-muted/20">
                    {/* Card-per-entry：每筆觀察一張卡片，欄位垂直堆疊
                        解決原 grid table 的 (1) dropdown 卡住、(2) 行動端排版崩、(3) 視覺密度過高 */}
                    {rows.map((row, idx) => (
                        <EntryCard key={row.tempKey} vm={vm} cat={cat} row={row} idx={idx} />
                    ))}
                    {canAddRow && (
                        <button
                            type="button"
                            onClick={() => vm.addRow(cat.key)}
                            className="w-full py-2 border-2 border-dashed border-muted-foreground/30 rounded-lg text-xs text-muted-foreground hover:border-status-success-solid hover:text-status-success-solid transition-colors flex items-center justify-center gap-1"
                        >
                            <Plus className="h-3 w-3" /> 新增列
                        </button>
                    )}
                </div>
            )}
        </div>
    )
}
