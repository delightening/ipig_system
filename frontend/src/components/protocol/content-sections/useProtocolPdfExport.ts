import { useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { logger } from '@/lib/logger'
import { usePdfServiceHealth } from '@/hooks/usePdfServiceHealth'
import { toast } from '@/components/ui/use-toast'

interface UseProtocolPdfExportOptions {
  protocolId?: string
  protocolTitle: string
  onExportPDF?: () => void
}

/**
 * R32-A8j：AUP 計畫書 PDF 匯出。
 *
 * 統一走 `/protocols/:id/export-aup-v3?format=pdf`（docxtpl + Word COM
 * fallback Gotenberg LibreOffice）。legacy v2（`export-pdf-v2` Tera + Gotenberg
 * + lopdf TOC）已移除，因格式問題；client-side html2canvas fallback 也移除（圖片
 * PDF 無法 Ctrl+F、檔案大、字級放大會糊）。
 */
export function useProtocolPdfExport({ protocolId, protocolTitle, onExportPDF }: UseProtocolPdfExportOptions) {
  const { t } = useTranslation()
  const contentRef = useRef<HTMLDivElement>(null)
  const [isExporting, setIsExporting] = useState(false)
  // AUP 為 GLP 文件，由 print-pdf 渲染。匯出前 ping /pdf-service-health。
  const { glpReady, refetch: refetchHealth } = usePdfServiceHealth()

  const handleExportPDF = async () => {
    if (isExporting || !protocolId) return

    // GLP pre-check：print-pdf 服務未上線時擋下匯出，提示使用者
    if (!glpReady) {
      const fresh = await refetchHealth()
      if (fresh.data?.glp_ready !== true) {
        toast({
          variant: 'destructive',
          title: 'PDF 服務未上線',
          description: '已自動通知管理員。請稍後再試（管理員處理通常 < 5 分鐘）。',
        })
        return
      }
    }

    try {
      setIsExporting(true)
      if (onExportPDF) onExportPDF()

      const response = await api.get(`/protocols/${protocolId}/export-aup-v3?format=pdf`, {
        responseType: 'blob',
      })

      const blob = new Blob([response.data], { type: 'application/pdf' })
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `${protocolTitle || t('protocols.content.title')}_${new Date().toISOString().split('T')[0]}.pdf`
      a.click()
      window.URL.revokeObjectURL(url)
    } catch (error) {
      logger.error('PDF export error:', error)
      alert(t('protocols.content.exportFailed'))
    } finally {
      setIsExporting(false)
    }
  }

  return { contentRef, isExporting, handleExportPDF }
}
