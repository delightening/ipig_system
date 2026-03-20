import { FileText, Download, Loader2, MessageSquare } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

import { ProtocolSectionNav } from './ProtocolSectionNav'
import { ReviewCommentPanel } from './ReviewCommentPanel'
import { useCurrentSection } from '@/pages/protocols/hooks/useCurrentSection'
import {
  ResearchInfoSection,
  PurposeSection,
  ItemsSection,
  DesignSection,
  GuidelinesSection,
  SurgerySection,
  AnimalsSection,
  PersonnelSection,
  AttachmentsSignaturesSection,
  useProtocolPdfExport,
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

  const basic = workingContent.basic || {}
  const purpose = workingContent.purpose || {}
  const items = workingContent.items || {}
  const design = workingContent.design || {}
  const guidelines = workingContent.guidelines || {}
  const surgery = workingContent.surgery || {}
  const animals = workingContent.animals || {}
  const personnel = workingContent.personnel || []
  const attachments = workingContent.attachments || []
  const signature = workingContent.signature || []

  return (
    <div className="flex gap-6">
      {/* Left sidebar — section navigation */}
      <aside className="hidden lg:block w-48 shrink-0">
        <ProtocolSectionNav sections={sectionItems} currentSection={currentSection} />
      </aside>

      {/* Center — main content */}
      <div className="flex-1 min-w-0">
        <div
          ref={contentRef}
          className={`protocol-pdf-view bg-white p-8 shadow-lg mx-auto relative ${showCommentPanel ? 'max-w-3xl' : 'max-w-4xl'}`}
        >
          {/* Header */}
          <div className="text-center mb-8 border-b pb-4">
            <h1 className="text-3xl font-bold mb-2">{t('protocols.content.title')}</h1>
            <p className="text-lg text-muted-foreground">{protocolTitle}</p>
          </div>

          <ResearchInfoSection basic={basic} protocolTitle={protocolTitle} startDate={startDate} endDate={endDate} />
          <PurposeSection purpose={purpose} />
          <ItemsSection items={items} />
          <DesignSection design={design} />
          <GuidelinesSection guidelines={guidelines} />
          <SurgerySection design={design} surgery={surgery} />
          <AnimalsSection animals={animals} />
          <PersonnelSection personnel={personnel} />
          <AttachmentsSignaturesSection attachments={attachments} signature={signature} />

          {/* Print / PDF Styles */}
          <style>{`
            @media print {
              .protocol-pdf-view {
                box-shadow: none !important;
                padding: 10mm !important;
                max-width: none !important;
              }
              .protocol-pdf-view > section {
                page-break-before: always;
                break-before: page;
              }
              .protocol-pdf-view > section:first-of-type {
                page-break-before: always;
              }
              .protocol-pdf-view section {
                page-break-inside: avoid;
                break-inside: avoid;
              }
              .protocol-pdf-view h2,
              .protocol-pdf-view h3 {
                page-break-after: avoid;
                break-after: avoid;
              }
              .protocol-pdf-view .p-4.border.rounded,
              .protocol-pdf-view .p-3.border.rounded {
                page-break-inside: avoid;
                break-inside: avoid;
              }
            }
          `}</style>
        </div>
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
