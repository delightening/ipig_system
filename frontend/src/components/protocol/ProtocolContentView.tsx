import { FileText, Download, Loader2, MessageSquare, Printer } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

import { ProtocolSectionNav } from './ProtocolSectionNav'
import { ReviewCommentPanel } from './ReviewCommentPanel'
import { ProtocolPrintableContent } from './ProtocolPrintableContent'
import { useCurrentSection } from '@/pages/protocols/hooks/useCurrentSection'
import {
  useProtocolPdfExport,
  useProtocolBrowserPrint,
} from './content-sections'

interface ProtocolContentViewProps {
  workingContent: ProtocolWorkingContent
  protocolTitle: string
  startDate?: string
  endDate?: string
  protocolId?: string
  onExportPDF?: () => void
  onToggleCommentPanel?: () => void
  showReviewButton?: boolean
  showCommentPanel?: boolean
  onSubmitComment?: (content: string) => void
  isSubmittingComment?: boolean
  sectionOptions?: string[]
}

export function ProtocolContentView({
  workingContent,
  protocolTitle,
  startDate,
  endDate,
  protocolId,
  onExportPDF,
  onToggleCommentPanel,
  showReviewButton,
  showCommentPanel,
  onSubmitComment,
  isSubmittingComment,
  sectionOptions,
}: ProtocolContentViewProps) {
  const { t } = useTranslation()
  const currentSection = useCurrentSection()
  const { contentRef, isExporting, handleExportPDF } = useProtocolPdfExport({
    protocolId,
    protocolTitle,
    onExportPDF,
  })
  // 純前端「瀏覽器列印」：React 渲染 → 新分頁 → window.print()（不經 print-pdf）。
  const { handleBrowserPrint } = useProtocolBrowserPrint({
    workingContent,
    protocolTitle,
    startDate,
    endDate,
  })

  const sectionItems = useMemo(() => [
    { id: 'section-1', label: t('protocols.content.sections.researchInfo') },
    { id: 'section-2', label: t('protocols.content.sections.purpose') },
    { id: 'section-3', label: t('protocols.content.sections.items') },
    { id: 'section-4', label: t('protocols.content.sections.design') },
    { id: 'section-5', label: t('protocols.content.sections.guidelines') },
    { id: 'section-6', label: t('protocols.content.sections.surgery') },
    { id: 'section-7', label: t('protocols.content.sections.animals') },
    { id: 'section-8', label: t('protocols.content.sections.personnel') },
    { id: 'section-9', label: t('protocols.content.sections.attachments') },
    { id: 'section-10', label: t('protocols.content.sections.signatures') },
  ], [t])

  if (!workingContent) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        <FileText className="h-12 w-12 mx-auto mb-2" />
        <p>{t('protocols.content.noContent')}</p>
      </div>
    )
  }

  // 「瀏覽器列印」按鈕（純前端渲染 → 新分頁 → window.print）。三個工具列共用同一份。
  const browserPrintButton = (
    <Button
      onClick={handleBrowserPrint}
      variant="outline"
      className="w-full text-xs"
      size="sm"
      title={t('protocols.content.browserPrint')}
    >
      <Printer className="mr-1.5 h-4 w-4" />
      {t('protocols.content.browserPrint')}
    </Button>
  )

  return (
    <div className="flex gap-6">
      {/* Left sidebar — section navigation */}
      <aside className="hidden lg:block w-48 shrink-0">
        <ProtocolSectionNav sections={sectionItems} currentSection={currentSection} />
      </aside>

      {/* Center — main content（與「瀏覽器列印」共用同一份 ProtocolPrintableContent） */}
      <div className="flex-1 min-w-0">
        <ProtocolPrintableContent
          ref={contentRef}
          workingContent={workingContent}
          protocolTitle={protocolTitle}
          startDate={startDate}
          endDate={endDate}
          className={`shadow-lg ${showCommentPanel ? 'max-w-3xl' : 'max-w-4xl'}`}
        />
      </div>

      {/* Right sidebar — action toolbar / inline review panel */}
      <aside className={`hidden md:block shrink-0 transition-all duration-300 ${showCommentPanel ? 'w-80' : 'w-28'}`}>
        <div className="sticky top-20 space-y-3">
          {showCommentPanel && onToggleCommentPanel && onSubmitComment ? (
            <>
              <Button
                onClick={handleExportPDF}
                variant="outline"
                disabled={isExporting}
                className="w-full text-xs"
                size="sm"
              >
                {isExporting ? <Loader2 className="mr-1.5 h-4 w-4 animate-spin" /> : <Download className="mr-1.5 h-4 w-4" />}
                {isExporting ? t('protocols.content.exporting') : t('protocols.content.exportPDF')}
              </Button>
              {browserPrintButton}
              <ReviewCommentPanel
                onClose={onToggleCommentPanel}
                onSubmit={onSubmitComment}
                isSubmitting={isSubmittingComment ?? false}
                sectionOptions={sectionOptions ?? []}
                currentSection={currentSection}
              />
            </>
          ) : (
            <>
              <Button
                onClick={handleExportPDF}
                variant="outline"
                disabled={isExporting}
                className="w-full text-xs"
                size="sm"
              >
                {isExporting ? <Loader2 className="mr-1.5 h-4 w-4 animate-spin" /> : <Download className="mr-1.5 h-4 w-4" />}
                {isExporting ? t('protocols.content.exporting') : t('protocols.content.exportPDF')}
              </Button>
              {browserPrintButton}
              {showReviewButton && onToggleCommentPanel && (
                <Button
                  onClick={onToggleCommentPanel}
                  variant="outline"
                  className="w-full text-xs"
                  size="sm"
                >
                  <MessageSquare className="mr-1.5 h-4 w-4" />
                  {t('protocols.content.reviewComment', '審查意見')}
                </Button>
              )}
            </>
          )}
        </div>
      </aside>
    </div>
  )
}
