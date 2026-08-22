// 巡場報告列表「動作」欄：檢視 / 編輯 / 下載 PDF / 撤回 / 刪除
//
// 自 VetPatrolReportListPage 抽出（維持頁面檔 ≤300 行、單一 JSX return ≤80 行）。
// 圖示鈕皆帶 aria-label（a11y：title 對螢幕報讀器不可靠）。
//
// 撤回/刪除的顯示條件（canRetract / canDelete）由父層依「狀態 + 是否建立者 + 是否 admin」
// 算好傳入；本元件只負責渲染。後端另有權限/狀態強制（撤回=送出未完成前且 created_by 或 admin；
// 刪除=草稿任何人、非草稿限 admin）。
//
// canEdit 對齊 useVetPatrolReport 的 isReadOnly 判斷（見父層算式註解）：點進去若不會真的
// 進入可編輯狀態，按鈕就直接 disabled，不開一個「看起來能編輯、進去才發現被鎖」的 dialog。
// 唯讀需求改用旁邊的「檢視」按鈕。

import { Button } from '@/components/ui/button'
import { Eye, FileDown, Pencil, Trash2, Undo2 } from 'lucide-react'

interface VetPatrolReportRowActionsProps {
    status: string
    patrolDate: string
    canEdit: boolean
    canRetract: boolean
    canDelete: boolean
    onView: () => void
    onEdit: () => void
    onDownload: () => void
    onRetract: () => void
    onDelete: () => void
}

function editDisabledReason(status: string): string {
    switch (status) {
        case 'completed': return '已完成（唯讀鎖定），請用「檢視」'
        case 'awaiting_acknowledgement': return '待追蹤者確認收到，暫不可編輯'
        case 'awaiting_follow_up': return '待追蹤者填寫，非本人負責階段'
        case 'draft': return '草稿僅建立者可編輯'
        default: return '目前不可編輯'
    }
}

export function VetPatrolReportRowActions({
    status, patrolDate, canEdit, canRetract, canDelete, onView, onEdit, onDownload, onRetract, onDelete,
}: VetPatrolReportRowActionsProps) {
    return (
        <div className="flex items-center justify-end gap-1">
            <Button
                variant="ghost"
                size="sm"
                onClick={onView}
                title="檢視（唯讀）"
                aria-label={`檢視 ${patrolDate} 的巡場報告（唯讀）`}
            >
                <Eye className="h-3.5 w-3.5" />
            </Button>
            <Button
                variant="ghost"
                size="sm"
                onClick={onEdit}
                disabled={!canEdit}
                title={canEdit ? '繼續編輯' : editDisabledReason(status)}
                aria-label={`${canEdit ? '編輯' : editDisabledReason(status)} ${patrolDate} 的巡場報告`}
            >
                <Pencil className="h-3.5 w-3.5" />
            </Button>
            <Button
                variant="ghost"
                size="sm"
                onClick={onDownload}
                title="下載 PDF"
                aria-label={`下載 ${patrolDate} 的巡場報告 PDF`}
            >
                <FileDown className="h-3.5 w-3.5" />
            </Button>
            {/* 撤回到草稿：已送出未完成前、由填報獸醫或 admin，供修正後重送 */}
            {canRetract && (
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onRetract}
                    title="撤回成草稿"
                    aria-label={`撤回 ${patrolDate} 的巡場報告成草稿`}
                    className="text-muted-foreground hover:text-foreground"
                >
                    <Undo2 className="h-3.5 w-3.5" />
                </Button>
            )}
            {/* 刪除：草稿任何人可刪；已送出/已完成為 GLP 鎖定紀錄，限 admin */}
            {canDelete && (
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onDelete}
                    title="刪除"
                    aria-label={`刪除 ${patrolDate} 的巡場報告`}
                    className="text-muted-foreground hover:text-destructive"
                >
                    <Trash2 className="h-3.5 w-3.5" />
                </Button>
            )}
        </div>
    )
}
