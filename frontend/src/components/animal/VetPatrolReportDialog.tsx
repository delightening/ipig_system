// 獸醫巡場報告 Dialog (R39 改版；R82-7 拆分)
//
// 主要行為（不變）：
//   1. Auto-save draft：開 dialog 後預建 draft，後續變動以「存草稿」按鈕 PUT。
//      「儲存報告」語意為「送出報告」（draft → submitted）。
//   2. Entry-level 照片：每筆觀察行可掛照片（mobile capture + Canvas 壓縮）。
//      整體環境照保留 report-level 區塊。
//   3. 行動裝置：input[capture="environment"] 觸發後鏡頭、multiple 一次選多張。
//
// 4 類別 × 3 欄（觀察 / 建議 / 追蹤改善），豬隻狀況可選動物。
//
// R82-7：狀態機抽到 useVetPatrolReport；照片 / PDF 為獨立 sub-hooks；
//        版面拆為 BasicInfoSection / CategorySection / EntryCard /
//        ReportPhotosSection / FooterActions；本檔僅組裝。

import { Stethoscope, Loader2 } from 'lucide-react'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { CATEGORIES } from './vet-patrol-report/constants'
import { useVetPatrolReport } from './vet-patrol-report/useVetPatrolReport'
import { BasicInfoSection } from './vet-patrol-report/BasicInfoSection'
import { CategorySection } from './vet-patrol-report/CategorySection'
import { ReportPhotosSection } from './vet-patrol-report/ReportPhotosSection'
import { FooterActions } from './vet-patrol-report/FooterActions'

interface VetPatrolReportDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    editReportId?: string | null
}

export function VetPatrolReportDialog({ open, onOpenChange, editReportId }: VetPatrolReportDialogProps) {
    const vm = useVetPatrolReport({ open, onOpenChange, editReportId })

    return (
        <Dialog open={open} onOpenChange={(o) => { if (!o) vm.handleClose(); else onOpenChange(true) }}>
            <DialogContent size="2xl" className="max-h-[90vh] overflow-y-auto">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-status-success-solid">
                        <Stethoscope className="h-5 w-5" />
                        獸醫巡場報告
                        {vm.isSaving && <Loader2 className="h-3 w-3 animate-spin text-muted-foreground" />}
                    </DialogTitle>
                    <DialogDescription className="sr-only">
                        建立 / 編輯獸醫巡場報告：填寫觀察、建議、追蹤改善，並指派追蹤者完成兩階段流程。
                    </DialogDescription>
                </DialogHeader>

                <BasicInfoSection vm={vm} />

                {/* 4 類別 */}
                <div className="space-y-3">
                    {CATEGORIES.map((cat) => (
                        <CategorySection key={cat.key} vm={vm} cat={cat} />
                    ))}
                </div>

                <ReportPhotosSection vm={vm} />

                <FooterActions vm={vm} />
            </DialogContent>
        </Dialog>
    )
}
