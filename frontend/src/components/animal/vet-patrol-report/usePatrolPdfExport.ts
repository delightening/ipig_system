// PDF 匯出邏輯（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import { useState } from 'react'
import api from '@/lib/api'
import { usePdfServiceHealth } from '@/hooks/usePdfServiceHealth'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'

export function usePatrolPdfExport({
    savedReportId,
    patrolDate,
}: {
    savedReportId: string | null
    patrolDate: string
}) {
    // 巡場報告為 GLP 文件，由 print-pdf 渲染
    const { glpReady, refetch: refetchPdfHealth } = usePdfServiceHealth()
    const [isExporting, setIsExporting] = useState(false)

    const handleExportPdf = async () => {
        if (!savedReportId) return
        if (!glpReady) {
            const fresh = await refetchPdfHealth()
            if (fresh.data?.glp_ready !== true) {
                toast({
                    variant: 'destructive',
                    title: 'PDF 服務未上線',
                    description: '已自動通知管理員。請稍後再試。',
                })
                return
            }
        }
        setIsExporting(true)
        // R42-7：daemon 冷啟 / Word COM 首次 Documents.Open 可能 20-40s。
        // nginx proxy_read_timeout 180s（frontend/nginx.conf）為硬上限，逾時即 504。
        const progress = toast({
            title: 'PDF 產製中',
            description: '首次產製可能需要 30 秒，請勿關閉視窗或重複點擊。',
            duration: 1000 * 60 * 3,
        })
        try {
            const res = await api.post(`/vet-patrol-reports/${savedReportId}/export-pdf`, {}, {
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
        } catch (error) {
            toast({ title: '錯誤', description: getApiErrorMessage(error, 'PDF 匯出失敗'), variant: 'destructive' })
        } finally {
            progress.dismiss()
            setIsExporting(false)
        }
    }

    return { isExporting, handleExportPdf }
}
