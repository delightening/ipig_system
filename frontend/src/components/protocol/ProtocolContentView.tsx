import { Label } from '@/components/ui/label'
import { formatDate } from '@/lib/utils'
import { FileText, Download, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useRef, useState } from 'react'
// jsPDF & html2canvas loaded lazily at PDF export time (~360KB savings)
import api from '@/lib/api'
import { useTranslation } from 'react-i18next'

interface ProtocolContentViewProps {
  workingContent: any
  protocolTitle: string
  startDate?: string
  endDate?: string
  protocolId?: string  // For backend PDF export
  onExportPDF?: () => void
}

export function ProtocolContentView({ workingContent, protocolTitle, startDate, endDate, protocolId, onExportPDF }: ProtocolContentViewProps) {
  const { t } = useTranslation()
  const contentRef = useRef<HTMLDivElement>(null)
  const [isExporting, setIsExporting] = useState(false)

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

  // Backend PDF export
  const exportFromBackend = async () => {
    if (!protocolId) return false

    try {
      const response = await api.get(`/protocols/${protocolId}/export-pdf`, {
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
      console.warn('Backend PDF export failed, falling back to client-side:', error)
      return false
    }
  }

  // Client-side PDF export (fallback) — section-aware page breaking
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

    // --- Build section-aware segments (in canvas px) ---
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

    // --- Render each segment, one section per page (split if taller than a page) ---
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

    // --- Page number footer on every page ---
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

      // Prioritize backend API
      const success = await exportFromBackend()

      // Fallback to client if backend failed
      if (!success) {
        await exportFromClient()
      }
    } catch (error) {
      console.error('PDF export error:', error)
      alert(t('protocols.content.exportFailed'))
    } finally {
      setIsExporting(false)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-end">
        <Button onClick={handleExportPDF} variant="outline" disabled={isExporting}>
          {isExporting ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Download className="mr-2 h-4 w-4" />}
          {isExporting ? t('protocols.content.exporting') : t('protocols.content.exportPDF')}
        </Button>
      </div>

      <div ref={contentRef} className="protocol-pdf-view bg-white p-8 shadow-lg max-w-4xl mx-auto">
        {/* Header */}
        <div className="text-center mb-8 border-b pb-4">
          <h1 className="text-3xl font-bold mb-2">{t('protocols.content.title')}</h1>
          <p className="text-lg text-muted-foreground">{protocolTitle}</p>
        </div>

        {/* 1. Research Information */}
        <section className="mb-8 section-1" data-section={t('protocols.content.sections.researchInfo')}>
          <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.researchInfo')}</h2>

          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.glpAttribute')}</Label>
                <p className="mt-1">{basic.is_glp ? t('protocols.content.sections.glpCompliant') : t('protocols.content.sections.glpNonCompliant')}</p>
              </div>
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.projectName')}</Label>
                <p className="mt-1">{protocolTitle || '-'}</p>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.projectType')}</Label>
                <p className="mt-1">
                  {basic.project_type ? t(`aup.projectTypes.${basic.project_type}`) : '-'}
                  {basic.project_type_other && ` (${basic.project_type_other})`}
                </p>
              </div>
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.projectCategory')}</Label>
                <p className="mt-1">
                  {basic.project_category ? t(`aup.projectCategories.${basic.project_category}`) : '-'}
                  {basic.project_category_other && ` (${basic.project_category_other})`}
                </p>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="text-sm font-semibold">{t('protocols.content.sections.expectedSchedule')}</Label>
                <p className="mt-1">
                  {(startDate || basic.start_date) && (endDate || basic.end_date)
                    ? `${formatDate(startDate || basic.start_date)} ~ ${formatDate(endDate || basic.end_date)}`
                    : '-'}
                </p>
              </div>
            </div>

            {/* PI Info */}
            {basic.pi && (
              <div className="border-t pt-4 mt-4">
                <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.piInfo')}</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label className="text-sm font-semibold">{t('protocols.content.sections.piName')}</Label>
                    <p className="mt-1">{basic.pi.name || '-'}</p>
                  </div>
                  <div>
                    <Label className="text-sm font-semibold">{t('protocols.content.sections.piPhone')}</Label>
                    <p className="mt-1">{basic.pi.phone || '-'}</p>
                  </div>
                  <div>
                    <Label className="text-sm font-semibold">{t('protocols.content.sections.piEmail')}</Label>
                    <p className="mt-1">{basic.pi.email || '-'}</p>
                  </div>
                  <div>
                    <Label className="text-sm font-semibold">{t('protocols.content.sections.piAddress')}</Label>
                    <p className="mt-1">{basic.pi.address || '-'}</p>
                  </div>
                </div>
              </div>
            )}

            {/* Sponsor Info */}
            {basic.sponsor && (
              <div className="border-t pt-4 mt-4">
                <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.sponsorInfo')}</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label className="text-sm font-semibold">{t('protocols.content.sections.sponsorName')}</Label>
                    <p className="mt-1">{basic.sponsor.name || '-'}</p>
                  </div>
                  <div>
                    <Label className="text-sm font-semibold">{t('protocols.content.sections.contactPerson')}</Label>
                    <p className="mt-1">{basic.sponsor.contact_person || '-'}</p>
                  </div>
                  <div>
                    <Label className="text-sm font-semibold">{t('protocols.content.sections.contactPhone')}</Label>
                    <p className="mt-1">{basic.sponsor.contact_phone || '-'}</p>
                  </div>
                  <div>
                    <Label className="text-sm font-semibold">{t('protocols.content.sections.contactEmail')}</Label>
                    <p className="mt-1">{basic.sponsor.contact_email || '-'}</p>
                  </div>
                </div>
              </div>
            )}

            {/* Institution and Facilities */}
            <div className="border-t pt-4 mt-4">
              <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.facility')}</h3>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label className="text-sm font-semibold">{t('protocols.content.sections.facilityName')}</Label>
                  <p className="mt-1">{basic.facility?.title || '-'}</p>
                </div>
                <div>
                  <Label className="text-sm font-semibold">{t('protocols.content.sections.location')}</Label>
                  <p className="mt-1">{basic.housing_location || '-'}</p>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* 2. Research Purpose */}
        {(purpose.significance || purpose.replacement || purpose.reduction) && (
          <section className="mb-8 border-t pt-6 section-2" data-section={t('protocols.content.sections.purpose')}>
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.purpose')}</h2>

            {/* 2.1 Purpose and Significance */}
            {purpose.significance && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.significance')}</h3>
                <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.significance}</p>
              </div>
            )}

            {/* 2.2 Replacement Principles */}
            {purpose.replacement && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.replacement')}</h3>

                {purpose.replacement.rationale && (
                  <div className="mb-3">
                    <h4 className="text-base font-medium mb-1">{t('protocols.content.sections.replacementRationale')}</h4>
                    <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.replacement.rationale}</p>
                  </div>
                )}

                {purpose.replacement.alt_search && (
                  <div className="mb-3">
                    <h4 className="text-base font-medium mb-1">{t('protocols.content.sections.alternativeSearch')}</h4>
                    {purpose.replacement.alt_search.platforms && purpose.replacement.alt_search.platforms.length > 0 && (
                      <ul className="list-disc list-inside text-sm mb-2">
                        {purpose.replacement.alt_search.platforms.map((p: string, idx: number) => (
                          <li key={idx}>{p}</li>
                        ))}
                      </ul>
                    )}
                    {purpose.replacement.alt_search.keywords && (
                      <p className="text-sm mb-2"><strong>{t('protocols.content.sections.searchKeywords')}: </strong>{purpose.replacement.alt_search.keywords}</p>
                    )}
                    {purpose.replacement.alt_search.conclusion && (
                      <div>
                        <p className="text-sm font-medium mb-1">{t('protocols.content.sections.searchConclusion')}:</p>
                        <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.replacement.alt_search.conclusion}</p>
                      </div>
                    )}
                  </div>
                )}

                {purpose.duplicate && (
                  <div className="mb-3">
                    <h4 className="text-base font-medium mb-1">{t('protocols.content.sections.duplicate')}</h4>
                    <p className="text-sm mb-2">{purpose.duplicate.experiment ? t('protocols.content.sections.duplicateYes') : t('protocols.content.sections.duplicateNo')}</p>
                    {purpose.duplicate.experiment && purpose.duplicate.justification && (
                      <div>
                        <p className="text-sm font-medium mb-1">{t('protocols.content.sections.duplicateJustification')}</p>
                        <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.duplicate.justification}</p>
                      </div>
                    )}
                  </div>
                )}
              </div>
            )}

            {/* 2.3 Reduction Principles */}
            {purpose.reduction && purpose.reduction.design && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.reduction')}</h3>
                <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{purpose.reduction.design}</p>
              </div>
            )}
          </section>
        )}

        {/* 3. Test and Control Items */}
        <section className="mb-8 border-t pt-6 section-3" data-section={t('protocols.content.sections.items')}>
          <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.items')}</h2>

          {items.use_test_item === true ? (
            <>
              {/* Test Items */}
              {items.test_items && items.test_items.length > 0 && (
                <div className="mb-4">
                  <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.testItems')}</h3>
                  {items.test_items.map((item: any, index: number) => (
                    <div key={index} className="mb-4 p-4 border rounded bg-slate-50">
                      <h4 className="font-medium mb-2">{t('protocols.content.sections.testItems')} #{index + 1}</h4>
                      <div className="grid grid-cols-2 gap-3 text-sm">
                        <div>
                          <Label className="font-semibold">{t('protocols.content.sections.itemName')}: </Label>
                          <span>{item.name || '-'}</span>
                        </div>
                        <div>
                          <Label className="font-semibold">{t('protocols.content.sections.itemForm')}: </Label>
                          <span>{item.form || '-'}</span>
                        </div>
                        <div>
                          <Label className="font-semibold">{t('protocols.content.sections.itemPurpose')}: </Label>
                          <span>{item.purpose || '-'}</span>
                        </div>
                        <div>
                          <Label className="font-semibold">{t('protocols.content.sections.itemStorage')}: </Label>
                          <span>{item.storage_conditions || '-'}</span>
                        </div>
                        <div>
                          <Label className="font-semibold">{t('protocols.content.sections.isSterile')}: </Label>
                          <span>{item.is_sterile ? t('protocols.content.sections.sterileYes') : t('protocols.content.sections.sterileNo')}</span>
                        </div>
                        {!item.is_sterile && item.non_sterile_justification && (
                          <div className="col-span-2">
                            <Label className="font-semibold">{t('protocols.content.sections.nonSterileJustification')} </Label>
                            <p className="mt-1 text-sm whitespace-pre-wrap">{item.non_sterile_justification}</p>
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}

              {/* Control Items */}
              {items.control_items && items.control_items.length > 0 && (
                <div className="mb-4">
                  <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.controlItems')}</h3>
                  {items.control_items.map((item: any, index: number) => (
                    <div key={index} className="mb-4 p-4 border rounded bg-slate-50">
                      <h4 className="font-medium mb-2">{t('protocols.content.sections.controlItems')} #{index + 1}</h4>
                      <div className="grid grid-cols-2 gap-3 text-sm">
                        <div>
                          <Label className="font-semibold">{t('protocols.content.sections.itemName')}: </Label>
                          <span>{item.name || '-'}</span>
                        </div>
                        <div>
                          <Label className="font-semibold">{t('protocols.content.sections.itemPurpose')}: </Label>
                          <span>{item.purpose || '-'}</span>
                        </div>
                        <div>
                          <Label className="font-semibold">{t('protocols.content.sections.itemStorage')}: </Label>
                          <span>{item.storage_conditions || '-'}</span>
                        </div>
                        <div>
                          <Label className="font-semibold">{t('protocols.content.sections.isSterile')}: </Label>
                          <span>{item.is_sterile ? t('protocols.content.sections.sterileYes') : t('protocols.content.sections.sterileNo')}</span>
                        </div>
                        {!item.is_sterile && item.non_sterile_justification && (
                          <div className="col-span-2">
                            <Label className="font-semibold">{t('protocols.content.sections.nonSterileJustification')} </Label>
                            <p className="mt-1 text-sm whitespace-pre-wrap">{item.non_sterile_justification}</p>
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </>
          ) : (
            <p className="text-sm text-muted-foreground">{t('protocols.content.sections.omitted')}</p>
          )}
        </section>

        {/* 4. Research Design and Methods */}
        {(design.procedures || design.anesthesia || design.pain || design.endpoints) && (
          <section className="mb-8 border-t pt-6 section-4" data-section={t('protocols.content.sections.design')}>
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.design')}</h2>

            {design.procedures && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.procedures')}</h3>
                <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{design.procedures}</p>
              </div>
            )}

            {design.anesthesia && design.anesthesia.is_under_anesthesia !== null && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.anesthesia')}</h3>
                <p className="text-sm">{design.anesthesia.is_under_anesthesia ? t('protocols.content.sections.sterileYes') : t('protocols.content.sections.sterileNo')}</p>
                {design.anesthesia.anesthesia_type && (
                  <p className="text-sm mt-2">{t('protocols.content.sections.anesthesiaType')}: {design.anesthesia.anesthesia_type}</p>
                )}
              </div>
            )}

            {design.pain && design.pain.category && (
              <div className="mb-4">
                <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.painCategory')}</h3>
                <p className="text-sm">{design.pain.category}</p>
                {design.pain.management_plan && (
                  <div className="mt-2">
                    <p className="text-sm font-medium mb-1">{t('protocols.content.sections.painManagement')}</p>
                    <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{design.pain.management_plan}</p>
                  </div>
                )}
              </div>
            )}

            {design.endpoints && (
              <div className="mb-4">
                {design.endpoints.experimental_endpoint && (
                  <div className="mb-3">
                    <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.experimentalEndpoint')}</h3>
                    <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{design.endpoints.experimental_endpoint}</p>
                  </div>
                )}
                {design.endpoints.humane_endpoint && (
                  <div>
                    <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.humaneEndpoint')}</h3>
                    <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{design.endpoints.humane_endpoint}</p>
                  </div>
                )}
              </div>
            )}
          </section>
        )}

        {/* 5. Relevant Guidelines and References */}
        <section className="mb-8 border-t pt-6 section-5" data-section={t('protocols.content.sections.guidelines')}>
          <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.guidelines')}</h2>

          {(guidelines.content || (guidelines.databases && guidelines.databases.some((db: any) => db.checked)) || (guidelines.references && guidelines.references.length > 0)) ? (
            <>
              {guidelines.content && (
                <div className="mb-4">
                  <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.guidelinesContent')}</h3>
                  <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{guidelines.content}</p>
                </div>
              )}

              {/* 資料庫搜尋紀錄 */}
              {guidelines.databases && guidelines.databases.some((db: any) => db.checked) && (
                <div className="mb-4">
                  <h3 className="text-lg font-semibold mb-3">{t('aup.guidelines.databasesTitle')}</h3>
                  <ul className="space-y-2">
                    {guidelines.databases.filter((db: any) => db.checked).map((db: any) => (
                      <li key={db.code} className="text-sm p-2 bg-slate-50 rounded">
                        <span className="font-medium">{db.code}. {t(`aup.guidelines.databases.${db.code}`)}</span>
                        {db.keywords && (
                          <span className="ml-2 text-muted-foreground">
                            — {t('aup.guidelines.keywordsLabel')}: {db.keywords}
                          </span>
                        )}
                        {db.note && (
                          <p className="mt-1 text-muted-foreground ml-4">{db.note}</p>
                        )}
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              {guidelines.references && guidelines.references.length > 0 && (
                <div className="mb-4">
                  <h3 className="text-lg font-semibold mb-3">{t('aup.guidelines.referencesTitle')}</h3>
                  <ol className="list-decimal list-inside space-y-2">
                    {guidelines.references.map((ref: any, index: number) => (
                      <li key={index} className="text-sm">
                        {ref.citation || '-'}
                        {ref.url && (
                          <span className="text-blue-600 ml-2">
                            <a href={ref.url} target="_blank" rel="noopener noreferrer">{ref.url}</a>
                          </span>
                        )}
                      </li>
                    ))}
                  </ol>
                </div>
              )}
            </>
          ) : (
            <p className="text-sm text-muted-foreground">{t('protocols.content.sections.omitted')}</p>
          )}
        </section>

        {/* 6. Surgery Protocol */}
        {(() => {
          // Check if surgery plan is needed: trials under anesthesia and survival or non-survival surgery
          const needsSurgeryPlan = design.anesthesia?.is_under_anesthesia === true &&
            (design.anesthesia?.anesthesia_type === 'survival_surgery' || design.anesthesia?.anesthesia_type === 'non_survival_surgery')

          return (
            <section className="mb-8 border-t pt-6 section-6" data-section={t('protocols.content.sections.surgery')}>
              <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.surgery')}</h2>

              {needsSurgeryPlan ? (
                <>
                  {surgery.surgery_type && (
                    <div className="mb-4">
                      <Label className="text-sm font-semibold">{t('protocols.content.sections.surgeryType')}</Label>
                      <p className="mt-1">{surgery.surgery_type}</p>
                    </div>
                  )}

                  {surgery.preop_preparation && (
                    <div className="mb-4">
                      <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.preop_Preparation')}</h3>
                      <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.preop_preparation}</p>
                    </div>
                  )}

                  {surgery.surgery_description && (
                    <div className="mb-4">
                      <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.surgeryDescription')}</h3>
                      <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.surgery_description}</p>
                    </div>
                  )}

                  {surgery.monitoring && (
                    <div className="mb-4">
                      <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.monitoring')}</h3>
                      <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.monitoring}</p>
                    </div>
                  )}

                  {surgery.postop_care && (
                    <div className="mb-4">
                      <h3 className="text-lg font-semibold mb-2">{t('protocols.content.sections.postopCare')}</h3>
                      <p className="text-sm whitespace-pre-wrap bg-slate-50 p-3 rounded">{surgery.postop_care}</p>
                    </div>
                  )}

                  {surgery.drugs && surgery.drugs.length > 0 && (
                    <div className="mb-4">
                      <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.drugPlan')}</h3>
                      <div className="space-y-2">
                        {surgery.drugs.map((drug: any, index: number) => (
                          <div key={index} className="p-3 border rounded bg-slate-50">
                            <p className="text-sm font-medium">{drug.drug_name || '-'}</p>
                            <p className="text-xs text-muted-foreground">
                              {t('protocols.content.sections.dose')}: {drug.dose || '-'} | {t('protocols.content.sections.route')}: {drug.route || '-'} | {t('protocols.content.sections.frequency')}: {drug.frequency || '-'} | {t('protocols.content.sections.itemPurpose')}: {drug.purpose || '-'}
                            </p>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </>
              ) : (
                <p className="text-sm text-muted-foreground">{t('protocols.content.sections.omitted')}</p>
              )}
            </section>
          )
        })()}

        {/* 7. Experimental Animal Data */}
        {animals.animals && animals.animals.length > 0 && (
          <section className="mb-8 border-t pt-6 section-7" data-section={t('protocols.content.sections.animals')}>
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.animals')}</h2>

            <div className="space-y-4">
              {animals.animals.map((animal: any, index: number) => (
                <div key={index} className="p-4 border rounded bg-slate-50">
                  <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.animalGroup')} #{index + 1}</h3>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.species')}</Label>
                      <p className="mt-1">{animal.species || '-'}{animal.species_other ? ` (${animal.species_other})` : ''}</p>
                    </div>
                    {animal.strain && (
                      <div>
                        <Label className="font-semibold">{t('protocols.content.sections.strain')}</Label>
                        <p className="mt-1">{animal.strain}{animal.strain_other ? ` (${animal.strain_other})` : ''}</p>
                      </div>
                    )}
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.sex')}</Label>
                      <p className="mt-1">{animal.sex || '-'}</p>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.number')}</Label>
                      <p className="mt-1">{animal.number || '-'}</p>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.ageRange')}</Label>
                      <p className="mt-1">
                        {animal.age_unlimited ? t('protocols.content.sections.unlimited') : `${animal.age_min || t('protocols.content.sections.unlimited')} ~ ${animal.age_max || t('protocols.content.sections.unlimited')}`}
                      </p>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.weightRange')}</Label>
                      <p className="mt-1">
                        {animal.weight_unlimited ? t('protocols.content.sections.unlimited') : `${animal.weight_min || t('protocols.content.sections.unlimited')}kg ~ ${animal.weight_max || t('protocols.content.sections.unlimited')}kg`}
                      </p>
                    </div>
                    {animal.housing_location && (
                      <div className="col-span-2">
                        <Label className="font-semibold">{t('protocols.content.sections.housingLocation')}</Label>
                        <p className="mt-1">{animal.housing_location}</p>
                      </div>
                    )}
                  </div>
                </div>
              ))}

              {animals.total_animals && (
                <div className="mt-4 p-4 bg-blue-50 rounded">
                  <Label className="text-lg font-semibold">{t('protocols.content.sections.totalAnimals')}: {animals.total_animals}</Label>
                </div>
              )}
            </div>
          </section>
        )}

        {/* 8. Study Personnel Data */}
        {personnel.length > 0 && (
          <section className="mb-8 border-t pt-6 section-8" data-section={t('protocols.content.sections.personnel')}>
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.personnel')}</h2>

            <div className="space-y-4">
              {personnel.map((person: any, index: number) => (
                <div key={index} className="p-4 border rounded bg-slate-50">
                  <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.person')} #{index + 1}</h3>
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.piName')}</Label>
                      <p className="mt-1">{person.name || '-'}</p>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.position')}</Label>
                      <p className="mt-1">{person.position || '-'}</p>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.yearsExperience')}</Label>
                      <p className="mt-1">{person.years_experience || '-'} {t('protocols.content.sections.years')}</p>
                    </div>
                    {person.roles && person.roles.length > 0 && (
                      <div className="col-span-2">
                        <Label className="font-semibold">{t('protocols.content.sections.workContent')}</Label>
                        <div className="mt-1 flex flex-wrap gap-2">
                          {person.roles.map((role: string, roleIndex: number) => (
                            <span key={roleIndex} className="px-2 py-1 bg-blue-100 rounded text-xs">{role}</span>
                          ))}
                        </div>
                      </div>
                    )}
                    {person.trainings && person.trainings.length > 0 && (
                      <div className="col-span-2">
                        <Label className="font-semibold">{t('protocols.content.sections.training')}</Label>
                        <div className="mt-1 flex flex-wrap gap-2">
                          {person.trainings.map((training: string, trainingIndex: number) => (
                            <span key={trainingIndex} className="px-2 py-1 bg-green-100 rounded text-xs">{training}</span>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </section>
        )}

        {/* 9. Attachments */}
        {attachments.length > 0 && (
          <section className="mb-8 border-t pt-6 section-9" data-section={t('protocols.content.sections.attachments')}>
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.attachments')}</h2>

            <div className="space-y-2">
              {attachments.map((attachment: any, index: number) => (
                <div key={index} className="p-3 border rounded">
                  <p className="text-sm">{index + 1}. {attachment.file_name || '-'}</p>
                </div>
              ))}
            </div>
          </section>
        )}

        {/* 10. Electronic Signatures */}
        {signature.length > 0 && (
          <section className="mb-8 border-t pt-6 section-10" data-section={t('protocols.content.sections.signatures')}>
            <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.signatures')}</h2>

            <div className="space-y-4">
              {signature.map((sig: any, index: number) => (
                <div key={index} className="p-4 border rounded">
                  {sig.preview_url ? (
                    <img src={sig.preview_url} alt={sig.file_name || `${t('protocols.content.sections.signatures')} ${index + 1}`} className="max-w-xs max-h-32 object-contain" />
                  ) : (
                    <p className="text-sm">{index + 1}. {sig.file_name || '-'}</p>
                  )}
                </div>
              ))}
            </div>
          </section>
        )}

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
  )
}
