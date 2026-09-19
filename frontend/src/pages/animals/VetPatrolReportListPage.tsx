// 巡場報告列表頁
//
// R39-16 升級：原本「N/A 沒列表」 → 補上完整列表 UI。
// 改版：原本多分頁（已完成/我的草稿/待我回覆）→ 單一列表 + 狀態篩選器（全部/草稿/已送出/已完成）。
//   後端以 `status=relevant` 一次撈「與我相關」的全部報告（我的草稿 ∪ 我送出/被指派的在途 ∪
//   全系統已完成），前端再依 row.status 在 client 端切分類。讓獸醫送出後仍能看到已送出的報告。
// 功能：狀態篩選 + 表格列出 + 點列開啟編輯 dialog + 下載 PDF + 刪除

import { useState, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { usePdfServiceHealth } from '@/hooks/usePdfServiceHealth'
import { useAuthHasPermission, useAuthUser, useAuthHasRole } from '@/stores/auth'
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { PendingOwnerBadge } from '@/components/PendingOwnerBadge'
import type { PendingOwner } from '@/types/pendingOwner'
import { VetPatrolReportDialog } from '@/components/animal/VetPatrolReportDialog'
import { VetPatrolReportView } from '@/components/animal/VetPatrolReportView'
import { VetPatrolReportRowActions } from './VetPatrolReportRowActions'
import {
    FileText, Plus, Loader2, Stethoscope,
} from 'lucide-react'
import { format } from 'date-fns'

interface ReportRow {
    id: string
    patrol_date: string
    accompanying_personnel: string | null
    status: 'draft' | 'awaiting_acknowledgement' | 'awaiting_follow_up' | 'completed'
    created_by: string | null
    created_by_name: string | null
    follow_up_user_id: string | null
    created_at: string
    submitted_at?: string | null
    updated_at: string
    last_message_at?: string
    /** 這份現在卡在誰手上；僅送出後未完成的兩個狀態有值 */
    pending_owner?: PendingOwner
}

// 單一列表的狀態篩選分類；'all' = 全部，其餘對應報告生命週期三段。
type StatusFilter = 'all' | 'draft' | 'submitted' | 'completed'

type StatusBucket = Exclude<StatusFilter, 'all'>

// 把後端四值 status 折成前端三分類：在途的兩個狀態（待確認/待追蹤）皆歸「已送出」。
function statusBucket(status: ReportRow['status']): StatusBucket {
    if (status === 'draft') return 'draft'
    if (status === 'completed') return 'completed'
    return 'submitted'
}

// label 存 i18n 鍵，渲染時才 t()（避免語言切換後不更新）
const BUCKET_META: Record<StatusBucket, { labelKey: string; className: string }> = {
    draft: { labelKey: 'animalPages.patrolList.status.draft', className: 'bg-muted text-muted-foreground' },
    submitted: { labelKey: 'animalPages.patrolList.status.submitted', className: 'bg-status-warning-bg text-status-warning-text' },
    completed: { labelKey: 'animalPages.patrolList.status.completed', className: 'bg-status-success-bg text-status-success-text' },
}

const FILTER_OPTIONS: { value: StatusFilter; labelKey: string }[] = [
    { value: 'all', labelKey: 'animalPages.shared.all' },
    { value: 'draft', labelKey: 'animalPages.patrolList.status.draft' },
    { value: 'submitted', labelKey: 'animalPages.patrolList.status.submitted' },
    { value: 'completed', labelKey: 'animalPages.patrolList.status.completed' },
]

const EMPTY_HINT_KEY: Record<StatusFilter, string> = {
    all: 'animalPages.patrolList.empty.all',
    draft: 'animalPages.patrolList.empty.draft',
    submitted: 'animalPages.patrolList.empty.submitted',
    completed: 'animalPages.patrolList.empty.completed',
}

export default function VetPatrolReportListPage() {
    const { t } = useTranslation()
    const qc = useQueryClient()
    // 巡場報告為 GLP 文件，由 print-pdf 渲染，匯出前 pre-check
    const { glpReady, refetch: refetchPdfHealth } = usePdfServiceHealth()
    // 具獸醫建議權限者（對齊後端 require_permission!("animal.vet.recommend")）可新增報告。
    const isVet = useAuthHasPermission()('animal.vet.recommend')
    // 撤回/刪除的顯示條件用：建立者本人（created_by）或 admin（比照後端 is_admin 短路）
    const currentUserId = useAuthUser()?.id ?? null
    const hasRole = useAuthHasRole()
    const isAdmin = hasRole('admin') || hasRole('SYSTEM_ADMIN')
    const [filter, setFilter] = useState<StatusFilter>('all')
    // 檢視範圍：'relevant'=與我相關（預設）；'all'=全系統所有人的報告（僅 admin，後端亦 gate）
    const [scope, setScope] = useState<'relevant' | 'all'>('relevant')
    const effectiveScope = isAdmin ? scope : 'relevant'
    const [editingId, setEditingId] = useState<string | null>(null)
    const [viewingId, setViewingId] = useState<string | null>(null)
    const [showNew, setShowNew] = useState(false)

    // 一次撈「與我相關」的全部報告（我的草稿 ∪ 我送出/被指派的在途 ∪ 全系統已完成），
    // 篩選器在 client 端切分類，切換不需重新請求。
    const { data: reports = [], isLoading } = useQuery({
        queryKey: ['vet-patrol-reports', effectiveScope],
        queryFn: async () => {
            const res = await api.get<ReportRow[]>(`/vet-patrol-reports?status=${effectiveScope}`)
            return res.data
        },
    })

    const visibleReports = useMemo(
        () => (filter === 'all' ? reports : reports.filter((r) => statusBucket(r.status) === filter)),
        [reports, filter],
    )

    const deleteMutation = useMutation({
        mutationFn: async (id: string) => {
            await api.post(`/vet-patrol-reports/${id}/delete`)
        },
        onSuccess: () => {
            toast({ title: t('common.deleted') })
            qc.invalidateQueries({ queryKey: ['vet-patrol-reports'] })
        },
        onError: (err) => toast({
            title: t('common.error'),
            description: getApiErrorMessage(err, t('animalPages.shared.deleteFailed')),
            variant: 'destructive',
        }),
    })

    const retractMutation = useMutation({
        mutationFn: async (id: string) => {
            await api.post(`/vet-patrol-reports/${id}/retract`)
        },
        onSuccess: (_data, id) => {
            toast({ title: t('animalPages.patrolList.toast.retracted') })
            qc.invalidateQueries({ queryKey: ['vet-patrol-reports'] })
            // 編輯/檢視 dialog 查的是單數 key（['vet-patrol-report', id]），全域 staleTime
            // 2 分鐘內若不特別 invalidate，重開會吃到撤回前的舊快取（isReadOnly 誤判仍鎖死），
            // 要整頁重新整理清空記憶體快取才會抓到新資料。
            qc.invalidateQueries({ queryKey: ['vet-patrol-report', id] })
        },
        onError: (err) => toast({
            title: t('common.error'),
            description: getApiErrorMessage(err, t('animalPages.patrolList.toast.retractFailed')),
            variant: 'destructive',
        }),
    })

    const handleRetract = (id: string, patrolDate: string) => {
        if (!window.confirm(t('animalPages.patrolList.confirmRetract', { date: patrolDate }))) return
        retractMutation.mutate(id)
    }

    const handleDownloadPdf = async (id: string, patrolDate: string) => {
        // GLP pre-check：print-pdf 服務未上線不開放匯出
        if (!glpReady) {
            const fresh = await refetchPdfHealth()
            if (fresh.data?.glp_ready !== true) {
                toast({
                    variant: 'destructive',
                    title: t('animalPages.patrolList.pdfOffline.title'),
                    description: t('animalPages.patrolList.pdfOffline.description'),
                })
                return
            }
        }
        try {
            const res = await api.post(`/vet-patrol-reports/${id}/export-pdf`, {}, {
                responseType: 'blob',
                _silentError: true,
            } as never)
            const blob = new Blob([res.data], { type: 'application/pdf' })
            const url = window.URL.createObjectURL(blob)
            const a = document.createElement('a')
            a.href = url
            a.download = `試驗豬場巡場報告_${patrolDate.replace(/-/g, '')}.pdf`
            document.body.appendChild(a)
            a.click()
            window.URL.revokeObjectURL(url)
            document.body.removeChild(a)
        } catch (err) {
            toast({
                title: t('common.error'),
                description: getApiErrorMessage(err, t('animalPages.patrolList.toast.pdfExportFailed')),
                variant: 'destructive',
            })
        }
    }

    const handleDelete = (id: string, patrolDate: string) => {
        if (!window.confirm(t('animalPages.patrolList.confirmDelete', { date: patrolDate }))) return
        deleteMutation.mutate(id)
    }

    return (
        <div className="p-4 md:p-6 space-y-4">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold flex items-center gap-2">
                        <Stethoscope className="h-6 w-6" />
                        {t('animalPages.patrolList.title')}
                    </h1>
                    <p className="text-sm text-muted-foreground mt-1">
                        {t('animalPages.patrolList.subtitle')}
                    </p>
                </div>
                {isVet && (
                    <Button onClick={() => setShowNew(true)}>
                        <Plus className="h-4 w-4 mr-1" />
                        {t('animalPages.patrolList.newReport')}
                    </Button>
                )}
            </div>

            <div className="flex items-center gap-2">
                <span className="text-sm text-muted-foreground">{t('animalPages.patrolList.statusFilter')}</span>
                <Select value={filter} onValueChange={(v) => setFilter(v as StatusFilter)}>
                    <SelectTrigger className="w-36">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        {FILTER_OPTIONS.map((opt) => (
                            <SelectItem key={opt.value} value={opt.value}>{t(opt.labelKey)}</SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                {/* 管理員專屬：切換檢視範圍（與我相關 / 全系統所有人）。非 admin 不顯示、後端亦 gate。 */}
                {isAdmin && (
                    <>
                        <span className="text-sm text-muted-foreground ml-2">{t('animalPages.patrolList.viewScope')}</span>
                        <Select value={scope} onValueChange={(v) => setScope(v as 'relevant' | 'all')}>
                            <SelectTrigger className="w-40">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="relevant">{t('animalPages.patrolList.scopeRelevant')}</SelectItem>
                                <SelectItem value="all">{t('animalPages.patrolList.scopeAll')}</SelectItem>
                            </SelectContent>
                        </Select>
                    </>
                )}
            </div>

            <div className="border rounded-lg overflow-hidden">
                <table className="w-full text-sm">
                    <thead className="bg-muted/50 text-muted-foreground">
                        <tr>
                            <th className="px-4 py-2 text-left font-medium">{t('animalPages.patrolList.columns.patrolDate')}</th>
                            <th className="px-4 py-2 text-left font-medium">{t('animalPages.patrolList.columns.veterinarian')}</th>
                            <th className="px-4 py-2 text-left font-medium">{t('animalPages.patrolList.columns.accompanying')}</th>
                            <th className="px-4 py-2 text-left font-medium">{t('animals.status')}</th>
                            <th className="px-4 py-2 text-left font-medium">{t('animalPages.patrolList.columns.lastUpdated')}</th>
                            <th className="px-4 py-2 text-right font-medium">{t('animals.actions')}</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y">
                        {isLoading ? (
                            <tr><td colSpan={6} className="p-8 text-center text-muted-foreground">
                                <Loader2 className="h-5 w-5 animate-spin inline mr-2" />
                                {t('common.loading')}
                            </td></tr>
                        ) : visibleReports.length === 0 ? (
                            <tr><td colSpan={6} className="p-8 text-center text-muted-foreground">
                                <FileText className="h-8 w-8 mx-auto mb-2 opacity-50" />
                                {t(EMPTY_HINT_KEY[filter])}
                            </td></tr>
                        ) : visibleReports.map((r) => {
                            const bucket = BUCKET_META[statusBucket(r.status)]
                            const isCreator = currentUserId != null && r.created_by === currentUserId
                            const isTracker = currentUserId != null && r.follow_up_user_id === currentUserId
                            const isSubmitted = r.status === 'awaiting_acknowledgement' || r.status === 'awaiting_follow_up'
                            // 撤回：已送出未完成前 + 建立者或 admin；刪除：草稿限獸醫、admin 不限狀態。
                            // 兩者後端皆為 require_permission!("animal.vet.recommend")（admin 於
                            // has_permission 短路放行），故先過 isVet/isAdmin 再套狀態規則——
                            // 否則失去 VET 角色的舊建立者仍會看到按鈕、點下去才吃 403。
                            const canRetract = isSubmitted && (isAdmin || (isVet && isCreator))
                            const canDelete = isAdmin || (isVet && r.status === 'draft')
                            // 編輯按鈕實際可點條件：對齊 useVetPatrolReport 的 isReadOnly 判斷
                            // （draft 限建立者；awaiting_acknowledgement/awaiting_follow_up 限追蹤者
                            // ——前者靠開 dialog 時的自動確認收到轉進可編輯階段；completed 恆唯讀）。
                            // 刻意不比照 canRetract/canDelete 給 admin 特權：dialog 本身沒有 admin
                            // bypass，給了會變成「按鈕能點、進去仍被鎖」，正是這次要修的體驗。
                            const canEdit = r.status === 'draft' ? isCreator
                                : r.status === 'completed' ? false
                                : isTracker
                            return (
                                <tr key={r.id} className="hover:bg-muted/30 transition-colors">
                                    <td className="px-4 py-3 font-medium">
                                        {/* patrol_date 已是 'YYYY-MM-DD' date-only 字串；直接顯示，
                                            不經 new Date()（會以 UTC 解析，UTC 負時區會顯示前一天） */}
                                        {r.patrol_date}
                                    </td>
                                    <td className="px-4 py-3">
                                        {r.created_by_name || <span className="text-muted-foreground">—</span>}
                                    </td>
                                    <td className="px-4 py-3">
                                        {r.accompanying_personnel || <span className="text-muted-foreground">—</span>}
                                    </td>
                                    <td className="px-4 py-3">
                                        {/* awaiting_acknowledgement 與 awaiting_follow_up 皆歸「已送出」分類；
                                            兩者在等的人與動作不同，靠 hover 補回被折疊掉的那層資訊 */}
                                        <PendingOwnerBadge owner={r.pending_owner}>
                                            <span className={`inline-block px-2 py-0.5 rounded text-xs ${bucket.className}`}>
                                                {t(bucket.labelKey)}
                                            </span>
                                        </PendingOwnerBadge>
                                    </td>
                                    <td className="px-4 py-3 text-muted-foreground text-xs">
                                        {format(new Date(r.updated_at), 'yyyy-MM-dd HH:mm')}
                                    </td>
                                    <td className="px-4 py-3">
                                        <VetPatrolReportRowActions
                                            status={r.status}
                                            patrolDate={r.patrol_date}
                                            canEdit={canEdit}
                                            canRetract={canRetract}
                                            canDelete={canDelete}
                                            onView={() => setViewingId(r.id)}
                                            onEdit={() => setEditingId(r.id)}
                                            onDownload={() => handleDownloadPdf(r.id, r.patrol_date)}
                                            onRetract={() => handleRetract(r.id, r.patrol_date)}
                                            onDelete={() => handleDelete(r.id, r.patrol_date)}
                                        />
                                    </td>
                                </tr>
                            )
                        })}
                    </tbody>
                </table>
            </div>

            {/* 唯讀文件式檢視（HTML 渲染；列印走「下載 PDF」） */}
            {viewingId && (
                <VetPatrolReportView
                    reportId={viewingId}
                    open={!!viewingId}
                    onOpenChange={(open) => { if (!open) setViewingId(null) }}
                    onDownloadPdf={() => {
                        const r = reports.find((row) => row.id === viewingId)
                        if (r) handleDownloadPdf(r.id, r.patrol_date)
                    }}
                />
            )}

            {/* 新增報告 dialog */}
            <VetPatrolReportDialog
                open={showNew}
                onOpenChange={(open) => {
                    setShowNew(open)
                    if (!open) qc.invalidateQueries({ queryKey: ['vet-patrol-reports'] })
                }}
            />

            {/* 編輯既有報告 dialog */}
            {editingId && (
                <VetPatrolReportDialog
                    open={!!editingId}
                    onOpenChange={(open) => {
                        if (!open) {
                            setEditingId(null)
                            qc.invalidateQueries({ queryKey: ['vet-patrol-reports'] })
                        }
                    }}
                    editReportId={editingId}
                />
            )}
        </div>
    )
}

// Allow both default and named import
export { VetPatrolReportListPage }
