import { useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import api from '@/lib/api'
import { logger } from '@/lib/logger'

interface UseProtocolPdfExportOptions {
  protocolId?: string
  protocolTitle: string
  onExportPDF?: () => void
}

export function useProtocolPdfExport({ protocolId, protocolTitle, onExportPDF }: UseProtocolPdfExportOptions) {
  const { t } = useTranslation()
  const contentRef = useRef<HTMLDivElement>(null)
  const [isExporting, setIsExporting] = useState(false)

  const exportFromBackend = async (): Promise<boolean> => {
    if (!protocolId) return false

    try {
      const response = await api.get(`/protocols/${protocolId}/export-pdf-v2`, {
        responseType: 'blob',
        _silentError: true,
      })

      const blob = new Blob([response.data], { type: 'application/pdf' })
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `${protocolTitle || t('protocols.content.title')}_${new Date().toISOString().split('T')[0]}.pdf`
      a.click()
      window.URL.revokeObjectURL(url)
      return true
    } catch (error) {
      logger.warn('Backend PDF export failed, falling back to client-side:', error)
      return false
    }
  }

  const exportFromClient = async () => {
    if (!contentRef.current) return

    const [{ default: html2canvas }, { default: jsPDF }] = await Promise.all([
      import('html2canvas'),
      import('jspdf'),
    ])

    const canvas = await html2canvas(contentRef.current, {
      scale: 2,
      useCORS: true,
      logging: false,
      backgroundColor: '#ffffff',
    })

    const pdf = new jsPDF('p', 'mm', 'a4')
    const pageWidth = pdf.internal.pageSize.getWidth()
    const pageHeight = pdf.internal.pageSize.getHeight()
    const margin = 10
    const footerHeight = 8
    const usableWidth = pageWidth - 2 * margin
    const usableHeight = pageHeight - 2 * margin - footerHeight

    const pxPerMm = canvas.width / usableWidth
    const maxSlicePx = usableHeight * pxPerMm

    const containerRect = contentRef.current.getBoundingClientRect()
    const domScale = canvas.width / containerRect.width
    const sections = contentRef.current.querySelectorAll('section')

    type Segment = { startPx: number; endPx: number }
    const segments: Segment[] = []

    if (sections.length > 0) {
      const firstY = Math.round(
        (sections[0].getBoundingClientRect().top - containerRect.top) * domScale
      )
      if (firstY > 20) {
        segments.push({ startPx: 0, endPx: firstY })
      }
      for (let i = 0; i < sections.length; i++) {
        const top = Math.round(
          (sections[i].getBoundingClientRect().top - containerRect.top) * domScale
        )
        const bottom =
          i < sections.length - 1
            ? Math.round(
              (sections[i + 1].getBoundingClientRect().top - containerRect.top) * domScale
            )
            : canvas.height
        if (bottom - top > 0) segments.push({ startPx: top, endPx: bottom })
      }
    } else {
      segments.push({ startPx: 0, endPx: canvas.height })
    }

    let isFirstPage = true

    const addSlice = (srcY: number, srcH: number) => {
      if (srcH <= 0) return
      if (!isFirstPage) pdf.addPage()
      isFirstPage = false

      const slice = document.createElement('canvas')
      slice.width = canvas.width
      slice.height = srcH
      slice.getContext('2d')!.drawImage(
        canvas, 0, srcY, canvas.width, srcH, 0, 0, canvas.width, srcH
      )
      pdf.addImage(
        slice.toDataURL('image/png'), 'PNG',
        margin, margin, usableWidth, srcH / pxPerMm
      )
    }

    for (const { startPx, endPx } of segments) {
      const h = endPx - startPx
      if (h <= maxSlicePx) {
        addSlice(startPx, h)
      } else {
        let offset = startPx
        while (offset < endPx) {
          const sliceH = Math.min(maxSlicePx, endPx - offset)
          addSlice(offset, sliceH)
          offset += sliceH
        }
      }
    }

    const totalPdfPages = pdf.getNumberOfPages()
    for (let i = 1; i <= totalPdfPages; i++) {
      pdf.setPage(i)
      pdf.setFontSize(8)
      pdf.setTextColor(150, 150, 150)
      pdf.text(`— ${i} / ${totalPdfPages} —`, pageWidth / 2, pageHeight - 5, {
        align: 'center',
      })
    }

    pdf.save(
      `${protocolTitle || t('protocols.content.title')}_${new Date().toISOString().split('T')[0]}.pdf`
    )
  }

  const handleExportPDF = async () => {
    if (isExporting) return

    try {
      setIsExporting(true)
      if (onExportPDF) onExportPDF()

      const success = await exportFromBackend()

      if (!success) {
        await exportFromClient()
      }
    } catch (error) {
      logger.error('PDF export error:', error)
      alert(t('protocols.content.exportFailed'))
    } finally {
      setIsExporting(false)
    }
  }

  return { contentRef, isExporting, handleExportPDF }
}
