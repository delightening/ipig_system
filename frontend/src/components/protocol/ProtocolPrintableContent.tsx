import { forwardRef } from 'react'
import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

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
} from './content-sections'

interface ProtocolPrintableContentProps {
  workingContent: ProtocolWorkingContent
  protocolTitle: string
  startDate?: string
  endDate?: string
  /** 疊加於 `.protocol-pdf-view` 的 class（螢幕：shadow / max-width；列印新分頁：max-w-none） */
  className?: string
}

/**
 * AUP 計畫書「可列印文件本體」：標題 + §1~§10 章節 + 列印分頁 CSS。
 *
 * 由 `ProtocolContentView`（live React 渲染）與 `useProtocolBrowserPrint`
 * （`renderToStaticMarkup` → 新分頁瀏覽器列印）共用，確保兩處版面同源（DRY）。
 * 純展示元件：只吃 props + i18n，無 store / query 依賴，故可在 static render 環境安全渲染。
 */
export const ProtocolPrintableContent = forwardRef<HTMLDivElement, ProtocolPrintableContentProps>(
  function ProtocolPrintableContent({ workingContent, protocolTitle, startDate, endDate, className }, ref) {
    const { t } = useTranslation()

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
      <div
        ref={ref}
        className={`protocol-pdf-view bg-white p-8 mx-auto relative ${className ?? ''}`}
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
    )
  },
)
