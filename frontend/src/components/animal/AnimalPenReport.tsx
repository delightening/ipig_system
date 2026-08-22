import React from 'react'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { Loader2 } from 'lucide-react'

interface AnimalPenReportProps {
    onClose: () => void
}

/**
 * 產生欄位狀態表 — PDF 預覽 dialog。
 *
 * 後端走 vet_patrol xlsx 範本 → Excel COM 出 PDF，前端只負責預覽（iframe）+ 下載 PDF。
 *
 * 由「產生欄位狀態表」按鈕觸發開啟。
 */
export const AnimalPenReport: React.FC<AnimalPenReportProps> = ({ onClose }) => {
    const { t } = useTranslation()

    // Asia/Taipei 本地日期（避免 UTC 跨日）
    const today = new Date().toLocaleDateString('sv-SE', { timeZone: 'Asia/Taipei' })

    // 預覽用：透過 axios fetch（帶認證 + 不被 Cloudflare 提早中斷）→ blob URL → iframe
    const [previewUrl, setPreviewUrl] = React.useState<string | null>(null)
    const [previewError, setPreviewError] = React.useState<string | null>(null)

    React.useEffect(() => {
        let revoked = false
        let objUrl: string | null = null
        ;(async () => {
            try {
                const res = await api.get(
                    `/animals/vet-patrol/export-v3?format=pdf&patrol_date=${today}&inline=1`,
                    { responseType: 'blob', _silentError: true, timeout: 120_000 },
                )
                if (revoked) return
                objUrl = window.URL.createObjectURL(new Blob([res.data], { type: 'application/pdf' }))
                setPreviewUrl(objUrl)
            } catch {
                if (!revoked) setPreviewError('PDF 預覽載入失敗，請稍後再試或直接下載 PDF')
            }
        })()
        return () => {
            revoked = true
            if (objUrl) window.URL.revokeObjectURL(objUrl)
        }
    }, [today])

    async function downloadVetPatrolPdf(): Promise<void> {
        const url = `/animals/vet-patrol/export-v3?format=pdf&patrol_date=${today}`
        const response = await api.get(url, { responseType: 'blob', _silentError: true })
        const blob = new Blob([response.data], { type: 'application/pdf' })
        const objUrl = window.URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = objUrl
        a.download = `欄位狀態表_${today}.pdf`
        a.style.display = 'none'
        document.body.appendChild(a)
        a.click()
        window.URL.revokeObjectURL(objUrl)
        document.body.removeChild(a)
    }

    const exportPDFMutation = useMutation({
        mutationFn: downloadVetPatrolPdf,
        onError: () => toast({ title: t('common.exportFailed'), variant: 'destructive' }),
    })

    return (
        <div className="fixed inset-0 bg-black/60 z-[9999] flex items-start justify-center p-4 overflow-y-auto">
            <div className="bg-card rounded-lg shadow-xl flex flex-col w-full max-w-5xl h-[90vh] mt-4">
                {/* Header */}
                <div className="flex items-center justify-between border-b px-4 py-3">
                    <h2 className="text-lg font-semibold">欄位狀態表預覽</h2>
                    <span className="text-sm text-muted-foreground">巡視日期：{today}</span>
                </div>

                {/* PDF preview (iframe via blob URL) */}
                <div className="relative flex-1 overflow-hidden bg-muted">
                    {!previewUrl && !previewError && (
                        <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-background/80 z-10">
                            <Loader2 className="h-8 w-8 animate-spin text-primary" />
                            <p className="text-sm text-muted-foreground animate-pulse">PDF 產生中…（首次約需 10–20 秒）</p>
                        </div>
                    )}
                    {previewError && (
                        <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 p-4 text-center">
                            <p className="text-sm text-destructive">{previewError}</p>
                        </div>
                    )}
                    {previewUrl && (
                        <iframe
                            title="欄位狀態表 PDF 預覽"
                            src={previewUrl}
                            className="w-full h-full border-0"
                        />
                    )}
                </div>

                {/* Footer actions */}
                <div className="flex justify-end gap-3 border-t px-4 py-3">
                    <button
                        onClick={onClose}
                        className="px-5 py-2 border border-border rounded-md hover:bg-muted font-medium"
                    >
                        {t('common.closeDialog')}
                    </button>
                    <button
                        onClick={() => exportPDFMutation.mutate()}
                        disabled={exportPDFMutation.isPending}
                        className="px-5 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-80 font-medium disabled:opacity-50"
                    >
                        {exportPDFMutation.isPending ? '匯出中…' : '下載 PDF'}
                    </button>
                </div>
            </div>
        </div>
    )
}
