// 獸醫巡場報告 — 唯讀文件式 HTML 檢視
//
// 與「下載 PDF」分工（對齊使用者決策 2026-06-30）：
//   - 線上檢視：HTML 渲染（本元件），資訊完整、可捲動、方便委員/送審者比對與給意見。
//   - 列印 / 存檔：走 PDF 分頁（list 頁的「下載 PDF」按鈕，完稿等級最少內容）。
//
// 資料來源：GET /vet-patrol-reports/:id（VetPatrolReportWithEntries）。
// 純展示：不可編輯、不轉狀態；編輯走 VetPatrolReportDialog。

import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { FileDown, Loader2, Stethoscope } from 'lucide-react'

// ── 型別（對齊 VetPatrolReportDialog 的 GET 回應）──────────────

interface PatrolPhoto {
    id: string
    file_name: string
    caption: string
    sort_order: number
}

interface EntryPhoto {
    id: string
    entry_id: string
    file_name: string
    caption: string
    sort_order: number
}

interface EntryWithAnimal {
    id: string
    category: string
    ear_tags: string[]
    observation: string
    suggestion: string
    follow_up: string
    sort_order: number
}

interface PatrolReport {
    id: string
    patrol_date: string
    accompanying_personnel: string | null
    status: 'draft' | 'awaiting_acknowledgement' | 'awaiting_follow_up' | 'completed'
    entries: EntryWithAnimal[]
    photos: PatrolPhoto[]
    entry_photos: EntryPhoto[]
}

// 類別顯示順序與標題（對齊後端 pdf_export 的 category_order；本元件僅需 key→label）
const CATEGORY_ORDER: { key: string; label: string }[] = [
    { key: 'pig_condition', label: '豬隻狀況' },
    { key: 'epidemic_prevention', label: '防疫及消毒計畫' },
    { key: 'case_record', label: '病歷紀錄' },
    { key: 'other', label: '其他' },
]

const STATUS_LABEL: Record<PatrolReport['status'], string> = {
    draft: '草稿',
    awaiting_acknowledgement: '已送出',
    awaiting_follow_up: '已送出',
    completed: '已完成',
}

interface VetPatrolReportViewProps {
    reportId: string
    open: boolean
    onOpenChange: (open: boolean) => void
    /** 由父層提供（list 已含 GLP pre-check + 下載邏輯），避免重複實作 */
    onDownloadPdf?: () => void
}

// ── 子元件（拆分以維持單一 JSX return ≤80 行）──────────────

// 單一欄位（觀察 / 建議 / 追蹤改善）；空值不顯示，讓文件更精簡
function Field({ label, value }: { label: string; value?: string | null }) {
    if (!value?.trim()) return null
    return (
        <div>
            <span className="text-xs font-semibold text-muted-foreground">{label}</span>
            <p className="text-sm whitespace-pre-wrap mt-0.5">{value}</p>
        </div>
    )
}

// 條目照片網格
function EntryPhotoGrid({ photos }: { photos: EntryPhoto[] }) {
    if (photos.length === 0) return null
    return (
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
            {photos.map((photo) => (
                <figure key={photo.id} className="space-y-1">
                    <img
                        src={`/api/v1/vet-patrol-entry-photos/${photo.id}/download`}
                        alt={photo.caption || photo.file_name}
                        className="h-20 w-full rounded object-cover"
                    />
                    {photo.caption && (
                        <figcaption className="text-xs text-muted-foreground">{photo.caption}</figcaption>
                    )}
                </figure>
            ))}
        </div>
    )
}

/**
 * 條目卡片標題。
 *
 * 耳號是這張卡片的主體，必須是第一眼看到的東西。舊版把耳號縮成 `text-xs`
 * `text-muted-foreground` 塞在右上角，與完全沒有資訊量的「條目 #N」同一個弱化層級，
 * 讀者第一落點因此落在編號而不是「這是哪幾隻豬」（使用者 2026-08-07 回報）。
 *
 * 無耳號的條目（防疫及消毒計畫 / 病歷紀錄 / 其他）沒有動物主體，改把「條目 #N」升為標題；
 * 不用類別名稱當標題，那會與區塊上方的 `<h3>` 重複。
 */
function EntryHeading({
    label, earTags, index,
}: { label: string; earTags?: string[]; index: number }) {
    const tags = earTags ?? []
    if (tags.length === 0) {
        return <h4 className="text-base font-bold leading-snug">條目 #{index + 1}</h4>
    }
    return (
        <div className="flex items-baseline justify-between gap-3">
            {/* 耳號不截斷（多隻群養時換行顯示），對齊專案「表格禁 truncate」的同一理由。
                aria-label 會**取代**元素內的可見文字成為 accessible name，所以「N 隻」
                必須一併寫進去，否則輔助技術讀不到隻數（CodeRabbit #44）。 */}
            <h4
                className="text-base font-bold leading-snug"
                aria-label={`${label}，耳號 ${tags.join('、')}，共 ${tags.length} 隻`}
            >
                {tags.join('、')}
                <span className="ml-1.5 text-xs font-normal text-muted-foreground">{tags.length} 隻</span>
            </h4>
            <span className="shrink-0 text-xs text-muted-foreground">條目 #{index + 1}</span>
        </div>
    )
}

// 單一類別區塊（條目卡片 + 各條目照片）
function CategorySection({
    label, entries, entryPhotos,
}: { label: string; entries: EntryWithAnimal[]; entryPhotos: EntryPhoto[] }) {
    if (entries.length === 0) return null
    return (
        <section>
            <h3 className="mb-3 border-b pb-1.5 text-base font-bold">{label}</h3>
            <div className="space-y-3">
                {entries.map((entry, idx) => {
                    const photos = entryPhotos
                        .filter((p) => p.entry_id === entry.id)
                        .sort((a, b) => a.sort_order - b.sort_order)
                    return (
                        <div key={entry.id} className="rounded-lg border bg-card p-3 space-y-2">
                            <EntryHeading label={label} earTags={entry.ear_tags} index={idx} />
                            <Field label="觀察內容" value={entry.observation} />
                            <Field label="建議" value={entry.suggestion} />
                            <Field label="追蹤改善" value={entry.follow_up} />
                            <EntryPhotoGrid photos={photos} />
                        </div>
                    )
                })}
            </div>
        </section>
    )
}

// 整體環境照（report-level）
function EnvironmentPhotos({ photos }: { photos: PatrolPhoto[] }) {
    if (!photos || photos.length === 0) return null
    const sorted = photos.slice().sort((a, b) => a.sort_order - b.sort_order)
    return (
        <section>
            <h3 className="mb-3 border-b pb-1.5 text-base font-bold">整體環境照</h3>
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
                {sorted.map((photo) => (
                    <figure key={photo.id} className="space-y-1">
                        <img
                            src={`/api/v1/vet-patrol-photos/${photo.id}/download`}
                            alt={photo.caption || photo.file_name}
                            className="h-32 w-full rounded object-cover"
                        />
                        {photo.caption && (
                            <figcaption className="text-xs text-muted-foreground">{photo.caption}</figcaption>
                        )}
                    </figure>
                ))}
            </div>
        </section>
    )
}

// 報告本體：基本資訊 + 各類別條目 + 整體環境照
function ReportBody({ report }: { report: PatrolReport }) {
    const entryPhotos = report.entry_photos || []
    return (
        <div className="space-y-6">
            <div className="grid grid-cols-2 gap-x-6 gap-y-2 rounded-lg border bg-muted/30 p-4 text-sm">
                <div>
                    <span className="text-xs font-semibold text-muted-foreground">巡場日期</span>
                    <p>{report.patrol_date}</p>
                </div>
                <div>
                    <span className="text-xs font-semibold text-muted-foreground">陪同人員</span>
                    <p>{report.accompanying_personnel || '—'}</p>
                </div>
                <div>
                    <span className="text-xs font-semibold text-muted-foreground">狀態</span>
                    <p>{STATUS_LABEL[report.status]}</p>
                </div>
            </div>

            {CATEGORY_ORDER.map(({ key, label }) => {
                // 保留有文字 / 耳號 / 照片任一的條目（純照片條目也要顯示）
                const catEntries = (report.entries || [])
                    .filter((e) => e.category === key)
                    .filter((e) =>
                        e.observation?.trim()
                        || e.suggestion?.trim()
                        || e.follow_up?.trim()
                        || (e.ear_tags?.length ?? 0) > 0
                        || entryPhotos.some((p) => p.entry_id === e.id),
                    )
                    .sort((a, b) => a.sort_order - b.sort_order)
                return <CategorySection key={key} label={label} entries={catEntries} entryPhotos={entryPhotos} />
            })}

            <EnvironmentPhotos photos={report.photos} />
        </div>
    )
}

export function VetPatrolReportView({ reportId, open, onOpenChange, onDownloadPdf }: VetPatrolReportViewProps) {
    const { t } = useTranslation()

    const { data: report, isLoading, isError } = useQuery({
        queryKey: ['vet-patrol-report', reportId],
        queryFn: async () => {
            const res = await api.get<PatrolReport>(`/vet-patrol-reports/${reportId}`)
            return res.data
        },
        enabled: !!reportId && open,
    })

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent size="xl" className="max-h-[90vh] overflow-y-auto">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-status-success-solid">
                        <Stethoscope className="h-5 w-5" />
                        獸醫巡場報告
                    </DialogTitle>
                    <DialogDescription className="sr-only">
                        唯讀檢視獸醫巡場報告內容；如需列印請使用「下載 PDF」。
                    </DialogDescription>
                </DialogHeader>

                {isLoading ? (
                    <div className="flex items-center justify-center py-20 text-muted-foreground">
                        <Loader2 className="mr-2 h-6 w-6 animate-spin" />
                        載入中…
                    </div>
                ) : isError || !report ? (
                    <div className="py-20 text-center text-sm text-destructive">
                        報告載入失敗，請稍後再試或改用「下載 PDF」。
                    </div>
                ) : (
                    <ReportBody report={report} />
                )}

                {/* 動作列：檢視為 HTML，列印走 PDF */}
                <div className="mt-4 flex justify-end gap-2 border-t pt-3">
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        {t('common.close', '關閉')}
                    </Button>
                    {onDownloadPdf && (
                        <Button variant="outline" onClick={onDownloadPdf}>
                            <FileDown className="mr-1 h-4 w-4" />
                            下載 PDF
                        </Button>
                    )}
                </div>
            </DialogContent>
        </Dialog>
    )
}
